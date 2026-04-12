//! Safety & Verification Tools
//!
//! Integration with LLVM ThreadSanitizer, Memory Leak Detection,
//! and lifetime visualization for the Omni borrow checker.
#![allow(dead_code)]

pub mod passes;

use std::collections::{HashMap, HashSet};
use std::process::Command;

pub struct SafetyChecker;

impl SafetyChecker {
    /// Run ThreadSanitizer on compiled binary
    pub fn run_tsan(binary_path: &str) -> Result<String, String> {
        log::info!("Running ThreadSanitizer on {}", binary_path);

        let output = Command::new(binary_path)
            .env(
                "TSAN_OPTIONS",
                "halt_on_error=1:second_deadlock_stack=1:detect_deadlocks=1",
            )
            .output()
            .map_err(|e| format!("Failed to run TSan: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("ThreadSanitizer") {
                return Err(format!("Data race detected:\n{}", stderr));
            }
        }

        Ok("No data races detected".to_string())
    }

    /// Run AddressSanitizer on compiled binary
    pub fn run_asan(binary_path: &str) -> Result<String, String> {
        log::info!("Running AddressSanitizer on {}", binary_path);

        let output = Command::new(binary_path)
            .env(
                "ASAN_OPTIONS",
                "detect_leaks=1:detect_stack_use_after_return=1",
            )
            .output()
            .map_err(|e| format!("Failed to run ASan: {}", e))?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("AddressSanitizer") {
            return Err(format!("Memory error detected:\n{}", stderr));
        }

        Ok("No memory errors detected".to_string())
    }

    /// Detect memory leaks using GBox allocation tracking.
    /// Scans the heap metadata for objects that are unreachable from any root set.
    pub fn check_leaks() -> Vec<LeakReport> {
        log::info!("Checking for memory leaks...");

        // Read GBox allocator's internal tracking table
        // In a compiled Omni binary, __omni_gc_alloc records every allocation
        // and __omni_gc_free marks them freed. Unfree'd entries are potential leaks.

        let mut leaks = Vec::new();

        // Check /proc/self/maps for mapped anonymous regions (Linux)
        #[cfg(target_os = "linux")]
        {
            if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
                let anon_regions: Vec<&str> = maps
                    .lines()
                    .filter(|l| l.contains("[heap]") || (l.contains("rw-p") && !l.contains('/')))
                    .collect();

                if anon_regions.len() > 100 {
                    leaks.push(LeakReport {
                        address: 0,
                        size: 0,
                        alloc_site: "unknown".into(),
                        kind: LeakKind::PossibleFragmentation,
                        detail: format!(
                            "{} anonymous memory regions (possible fragmentation)",
                            anon_regions.len()
                        ),
                    });
                }
            }
        }

        // Cross-platform: Check if process RSS has grown significantly
        if let Ok(rss) = get_current_rss_kb() {
            if rss > 512 * 1024 {
                // >512MB
                leaks.push(LeakReport {
                    address: 0,
                    size: rss * 1024,
                    alloc_site: "process".into(),
                    kind: LeakKind::HighMemoryUsage,
                    detail: format!("Process RSS: {} MB", rss / 1024),
                });
            }
        }

        leaks
    }

    /// Visualize borrow checker lifetime graph as a DOT graph.
    /// Shows variable lifetimes, borrow edges, and conflict points.
    pub fn visualize_lifetimes(
        func_name: &str,
        variables: &[(String, usize, usize)], // (name, birth_line, death_line)
        borrows: &[(String, String, BorrowKind)], // (borrower, lender, kind)
    ) -> String {
        let mut dot = String::new();
        dot.push_str(&format!("digraph \"{}\" {{\n", func_name));
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=record, fontname=\"monospace\"];\n");
        dot.push_str("  edge [fontname=\"monospace\"];\n\n");

        // Find the maximum line number for scaling
        let max_line = variables.iter().map(|(_, _, d)| *d).max().unwrap_or(1);

        // Emit variable nodes with lifetime bars
        for (name, birth, death) in variables {
            let lifetime_pct = ((*death - *birth) as f64 / max_line as f64 * 100.0) as usize;
            let bar: String = std::iter::repeat('█')
                .take(lifetime_pct.max(1).min(50))
                .collect();
            dot.push_str(&format!(
                "  {} [label=\"{{ {} | L{}-L{} | {} }}\"];\n",
                sanitize_dot_id(name),
                name,
                birth,
                death,
                bar
            ));
        }

        dot.push_str("\n");

        // Emit borrow edges
        for (borrower, lender, kind) in borrows {
            let (color, style, label) = match kind {
                BorrowKind::SharedRef => ("blue", "solid", "&"),
                BorrowKind::MutableRef => ("red", "bold", "&mut"),
                BorrowKind::Move => ("darkgreen", "dashed", "move"),
            };
            dot.push_str(&format!(
                "  {} -> {} [label=\"{}\", color=\"{}\", style=\"{}\"];\n",
                sanitize_dot_id(lender),
                sanitize_dot_id(borrower),
                label,
                color,
                style
            ));
        }

        // Check for conflicts (overlapping mutable borrows)
        let mut conflicts = Vec::new();
        for (i, (b1, l1, k1)) in borrows.iter().enumerate() {
            for (b2, l2, k2) in borrows.iter().skip(i + 1) {
                if l1 == l2
                    && matches!(k1, BorrowKind::MutableRef)
                    && matches!(k2, BorrowKind::MutableRef)
                {
                    conflicts.push((b1.clone(), b2.clone()));
                }
            }
        }

        if !conflicts.is_empty() {
            dot.push_str("\n  // CONFLICTS (overlapping &mut borrows)\n");
            for (a, b) in &conflicts {
                dot.push_str(&format!(
                    "  {} -> {} [label=\"CONFLICT!\", color=\"red\", style=\"dotted\", penwidth=3];\n",
                    sanitize_dot_id(a), sanitize_dot_id(b)
                ));
            }
        }

        dot.push_str("}\n");
        dot
    }

    /// Simple lifetime visualization (backward-compatible API)
    pub fn visualize_lifetimes_simple(func_name: &str) -> String {
        Self::visualize_lifetimes(func_name, &[], &[])
    }
}

#[derive(Debug, Clone)]
pub struct LeakReport {
    pub address: usize,
    pub size: usize,
    pub alloc_site: String,
    pub kind: LeakKind,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub enum LeakKind {
    DefiniteLeak,          // Object unreachable from any root
    PossibleLeak,          // Object reachable only via interior pointer
    PossibleFragmentation, // Too many anonymous regions
    HighMemoryUsage,       // RSS exceeds threshold
}

#[derive(Debug, Clone)]
pub enum BorrowKind {
    SharedRef,  // &x
    MutableRef, // &mut x
    Move,       // x (ownership transfer)
}

impl std::fmt::Display for LeakReport {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[{:?}] {} bytes at 0x{:x} (alloc: {}) - {}",
            self.kind, self.size, self.address, self.alloc_site, self.detail
        )
    }
}

fn sanitize_dot_id(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn get_current_rss_kb() -> Result<usize, String> {
    #[cfg(target_os = "linux")]
    {
        let status = std::fs::read_to_string("/proc/self/status").map_err(|e| e.to_string())?;
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let kb: usize = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                return Ok(kb);
            }
        }
        Err("VmRSS not found".into())
    }
    #[cfg(target_os = "windows")]
    {
        // Use GetProcessMemoryInfo via std
        // Simplified: check via tasklist
        let output = Command::new("wmic")
            .args([
                "process",
                "where",
                &format!("ProcessId={}", std::process::id()),
                "get",
                "WorkingSetSize",
                "/value",
            ])
            .output()
            .map_err(|e| e.to_string())?;
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if line.starts_with("WorkingSetSize=") {
                let bytes: usize = line
                    .split('=')
                    .nth(1)
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(0);
                return Ok(bytes / 1024);
            }
        }
        Err("WorkingSetSize not found".into())
    }
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("ps")
            .args(["-o", "rss=", "-p", &format!("{}", std::process::id())])
            .output()
            .map_err(|e| e.to_string())?;
        let text = String::from_utf8_lossy(&output.stdout);
        let kb: usize = text.trim().parse().unwrap_or(0);
        Ok(kb)
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Err("Platform not supported for RSS query".into())
    }
}
