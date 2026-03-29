#![allow(dead_code)]
//! Omni Hot Reload Runtime
//!
//! Implements function pointer indirection and atomic patching for live code updates.
//! Includes file watching, safe-point regions, and compilation integration.

use log::{debug, info};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, SystemTime};

// Thread-local safe region flag - when true, the current thread is in a safe region
// where function pointers may be swapped without corrupting the call stack.
thread_local! {
    static IN_SAFE_REGION: std::cell::Cell<bool> = std::cell::Cell::new(false);
}

/// Global active thread counter - tracks how many threads are executing user code
/// (outside safe regions). Swaps wait for this to reach zero.
static ACTIVE_THREADS: AtomicUsize = AtomicUsize::new(0);
static SWAP_PENDING: AtomicBool = AtomicBool::new(false);

/// Function Shim
/// double-pointer indirection: Call site -> Shim -> [Atomic Ptr] -> Real Function
pub struct HotFunction {
    pub name: String,
    pub ptr: AtomicUsize,
    pub version: AtomicUsize,
}

impl HotFunction {
    pub fn new(name: &str, initial_ptr: usize) -> Self {
        Self {
            name: name.to_string(),
            ptr: AtomicUsize::new(initial_ptr),
            version: AtomicUsize::new(0),
        }
    }

    /// Invoke the function
    pub fn call(&self, args: &mut [usize]) -> usize {
        // 1. Enter safe region (register as active)
        self.enter_safe_region();

        // 2. Load current implementation address
        let func_ptr = self.ptr.load(Ordering::Acquire);

        // 3. Cast to function pointer and call (unsafe)
        let result = unsafe {
            let func: extern "C" fn(*mut usize) -> usize = std::mem::transmute(func_ptr);
            func(args.as_mut_ptr())
        };

        // 4. Exit safe region
        self.exit_safe_region();
        result
    }

    /// Patch the function with a new implementation
    pub fn patch(&self, new_ptr: usize) {
        info!(
            "Hot Reload: Patching function '{}' (version {})",
            self.name,
            self.version.load(Ordering::Relaxed)
        );

        // Wait until no threads are in the danger zone
        while ACTIVE_THREADS.load(Ordering::Acquire) > 0 {
            SWAP_PENDING.store(true, Ordering::Release);
            std::thread::yield_now();
        }

        // Atomic swap ensures no race conditions on pointer load
        self.ptr.store(new_ptr, Ordering::Release);
        self.version.fetch_add(1, Ordering::Relaxed);
        SWAP_PENDING.store(false, Ordering::Release);

        info!(
            "Hot Reload: Function '{}' patched to version {}",
            self.name,
            self.version.load(Ordering::Relaxed)
        );
    }

    fn enter_safe_region(&self) {
        IN_SAFE_REGION.with(|flag| {
            if !flag.get() {
                ACTIVE_THREADS.fetch_add(1, Ordering::AcqRel);
                flag.set(true);
            }
        });
    }

    fn exit_safe_region(&self) {
        IN_SAFE_REGION.with(|flag| {
            if flag.get() {
                ACTIVE_THREADS.fetch_sub(1, Ordering::AcqRel);
                flag.set(false);
            }
        });
    }
}

/// File change event for watched source files
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: PathBuf,
    pub modified_at: SystemTime,
}

/// Runtime Manager for Hot Swap with file watching
pub struct HotSwapManager {
    registry: HashMap<String, HotFunction>,
    watched_files: HashMap<PathBuf, SystemTime>,
    pending_changes: Vec<FileChange>,
    watch_interval: Duration,
    last_check: SystemTime,
    enabled: bool,
}

impl HotSwapManager {
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
            watched_files: HashMap::new(),
            pending_changes: Vec::new(),
            watch_interval: Duration::from_millis(500),
            last_check: SystemTime::now(),
            enabled: true,
        }
    }

    pub fn register(&mut self, name: &str, ptr: usize) {
        info!("Hot Swap: Registering function '{}'", name);
        self.registry
            .insert(name.to_string(), HotFunction::new(name, ptr));
    }

    pub fn update(&self, name: &str, new_ptr: usize) -> Result<(), String> {
        if let Some(func) = self.registry.get(name) {
            func.patch(new_ptr);
            Ok(())
        } else {
            Err(format!(
                "Function '{}' not found in hot swap registry",
                name
            ))
        }
    }

    /// Watch a source file for changes
    pub fn watch_file(&mut self, path: &Path) {
        if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                self.watched_files.insert(path.to_path_buf(), modified);
                debug!("Hot Swap: Watching file {:?}", path);
            }
        }
    }

    /// Watch all .omni files in a directory recursively
    pub fn watch_directory(&mut self, dir: &Path) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    self.watch_directory(&path);
                } else if path.extension().map(|e| e == "omni").unwrap_or(false) {
                    self.watch_file(&path);
                }
            }
        }
    }

    /// Check watched files for modifications. Returns true if changes detected.
    pub fn check_for_updates(&mut self) -> bool {
        if !self.enabled {
            return false;
        }

        let now = SystemTime::now();
        if now.duration_since(self.last_check).unwrap_or_default() < self.watch_interval {
            return false;
        }
        self.last_check = now;

        let mut changes = Vec::new();

        for (path, last_modified) in &self.watched_files {
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(modified) = metadata.modified() {
                    if modified > *last_modified {
                        changes.push(FileChange {
                            path: path.clone(),
                            modified_at: modified,
                        });
                    }
                }
            }
        }

        if !changes.is_empty() {
            info!("Hot Swap: Detected {} file change(s)", changes.len());
            for change in &changes {
                // Update the last-modified time
                self.watched_files
                    .insert(change.path.clone(), change.modified_at);
                debug!("Hot Swap: Changed: {:?}", change.path);
            }
            self.pending_changes.extend(changes);
            true
        } else {
            false
        }
    }

    /// Get and clear all pending hot-swap updates
    pub fn get_pending_changes(&mut self) -> Vec<FileChange> {
        self.pending_changes.drain(..).collect()
    }

    /// Apply all pending hot-swap updates (Legacy/Self-contained mode)
    pub fn apply_pending(&mut self) {
        let changes = self.get_pending_changes();

        for change in &changes {
            info!("Hot Swap: Recompiling {:?}", change.path);
            debug!(
                "Hot Swap: Would recompile and patch functions from {:?}",
                change.path
            );
        }
    }

    /// Get a function's current version
    pub fn version(&self, name: &str) -> Option<usize> {
        self.registry
            .get(name)
            .map(|f| f.version.load(Ordering::Relaxed))
    }

    /// Enable/disable hot swap
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if a swap is pending (some thread needs to reach a safe point)
    pub fn is_swap_pending() -> bool {
        SWAP_PENDING.load(Ordering::Acquire)
    }
}
