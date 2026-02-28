#![allow(dead_code)]
//! GPU Kernel Dispatch System
//!
//! Provides a unified GPU compute dispatch layer for the Omni compiler.
//! Features:
//! - Device enumeration and selection (CUDA, OpenCL, Vulkan Compute, Metal)
//! - GPU memory management (alloc, free, host↔device transfer)
//! - Kernel launch configuration (grid/block dims, shared memory, streams)
//! - Multi-GPU dispatch with work partitioning
//! - Async kernel execution with dependency tracking
//! - Integration with gpu_advanced.rs (warp divergence, tensor cores)
//! - Integration with gpu_fusion.rs (kernel fusion passes)

use crate::ir::{IrFunction, IrInstruction, IrType, IrBinOp, IrTerminator, IrValue, IrConst};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::{Arc, Mutex};
use log::{debug, info, warn};
use libloading::{Library, Symbol};

// ─────────────────────────────────────────────────────────────────────────────
// GPU Backend Abstraction
// ─────────────────────────────────────────────────────────────────────────────

/// Kernel argument types
#[derive(Debug, Clone)]
pub enum KernelArg {
    Float(f32),
    Int(i32),
    Buffer(u64), // Pointer/handle
}

/// Trait for GPU backend implementations
pub trait GpuContext: Send + Sync {
    /// Get the backend type
    fn backend_type(&self) -> GpuBackendType;
    
    /// Allocate memory on the device
    fn alloc(&self, size: usize) -> Result<u64, String>;
    
    /// Free memory on the device
    fn free(&self, ptr: u64) -> Result<(), String>;
    
    /// Copy host -> device
    fn memcpy_h2d(&self, dst: u64, src: &[u8]) -> Result<(), String>;
    
    /// Copy device -> host
    fn memcpy_d2h(&self, src: u64, size: usize) -> Result<Vec<u8>, String>;
    
    /// Load a kernel package/module
    fn load_kernel(&self, kernel: &GpuKernel) -> Result<(), String> {
        // Default implementation does nothing (for now)
        Ok(())
    }

    /// Launch a kernel
    fn launch_kernel(
        &self,
        kernel_name: &str,
        grid: [u32; 3],
        block: [u32; 3],
        shared_mem: u32,
        args: &[KernelArg],
    ) -> Result<(), String>;
    
    /// Synchronize the device
    fn synchronize(&self) -> Result<(), String>;
}

/// Supported GPU compute backends
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GpuBackendType {
    /// NVIDIA CUDA
    Cuda,
    /// OpenCL (cross-vendor)
    OpenCL,
    /// Vulkan Compute Shaders
    Vulkan,
    /// Apple Metal Compute
    Metal,
    /// Software fallback (CPU emulation)
    Software,
    /// Mock (for testing)
    Mock,
}

impl fmt::Display for GpuBackendType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GpuBackendType::Cuda => write!(f, "CUDA"),
            GpuBackendType::OpenCL => write!(f, "OpenCL"),
            GpuBackendType::Vulkan => write!(f, "Vulkan Compute"),
            GpuBackendType::Metal => write!(f, "Metal"),
            GpuBackendType::Software => write!(f, "Software"),
            GpuBackendType::Mock => write!(f, "Mock"),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Software Backend (CPU Emulation)
// ─────────────────────────────────────────────────────────────────────────────

/// Software backend that emulates GPU memory and execution on the CPU
pub struct SoftwareBackend {
    /// Simulated device memory: AllocID -> Data
    memory: Mutex<HashMap<u64, Vec<u8>>>,
    /// Next available allocation ID
    next_id: Mutex<u64>,
}

impl SoftwareBackend {
    pub fn new() -> Self {
        SoftwareBackend {
            memory: Mutex::new(HashMap::new()),
            next_id: Mutex::new(1),
        }
    }
}

impl GpuContext for SoftwareBackend {
    fn backend_type(&self) -> GpuBackendType {
        GpuBackendType::Software
    }

    fn alloc(&self, size: usize) -> Result<u64, String> {
        let mut memory = self.memory.lock().map_err(|e| e.to_string())?;
        let mut next_id = self.next_id.lock().map_err(|e| e.to_string())?;
        
        let id = *next_id;
        *next_id += 1;
        
        // Allocate zero-initialized memory
        memory.insert(id, vec![0u8; size]);
        debug!("SoftwareBackend: Allocated {} bytes at ID {}", size, id);
        
        Ok(id)
    }

    fn free(&self, ptr: u64) -> Result<(), String> {
        let mut memory = self.memory.lock().map_err(|e| e.to_string())?;
        
        if memory.remove(&ptr).is_some() {
            debug!("SoftwareBackend: Freed ID {}", ptr);
            Ok(())
        } else {
            Err(format!("Invalid allocation ID {}", ptr))
        }
    }

    fn memcpy_h2d(&self, dst: u64, src: &[u8]) -> Result<(), String> {
        let mut memory = self.memory.lock().map_err(|e| e.to_string())?;
        
        if let Some(buffer) = memory.get_mut(&dst) {
            if src.len() > buffer.len() {
                return Err(format!("Buffer overflow: writing {} bytes to {} byte allocation", src.len(), buffer.len()));
            }
            buffer[..src.len()].copy_from_slice(src);
            debug!("SoftwareBackend: H2D copy {} bytes to ID {}", src.len(), dst);
            Ok(())
        } else {
            Err(format!("Invalid allocation ID {}", dst))
        }
    }

    fn memcpy_d2h(&self, src: u64, size: usize) -> Result<Vec<u8>, String> {
        let memory = self.memory.lock().map_err(|e| e.to_string())?;
        
        if let Some(buffer) = memory.get(&src) {
            if size > buffer.len() {
                return Err(format!("Read overflow: reading {} bytes from {} byte allocation", size, buffer.len()));
            }
            debug!("SoftwareBackend: D2H copy {} bytes from ID {}", size, src);
            Ok(buffer[..size].to_vec())
        } else {
            Err(format!("Invalid allocation ID {}", src))
        }
    }

    fn launch_kernel(
        &self,
        kernel_name: &str,
        grid: [u32; 3],
        block: [u32; 3],
        _shared_mem: u32,
        _args: &[KernelArg],
    ) -> Result<(), String> {
        // In a real software backend, we would look up a compiled reference implementation
        // of the kernel and execute it on a thread pool.
        // For now, we just log the launch parameters.
        info!(
            "SoftwareBackend: Launching kernel '{}' grid={:?} block={:?}", 
            kernel_name, grid, block
        );
        Ok(())
    }

    fn synchronize(&self) -> Result<(), String> {
        // CPU execution is synchronous by default in this simple model
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CUDA Backend (Dynamic Loading)
// ─────────────────────────────────────────────────────────────────────────────

type CuInit = unsafe extern "system" fn(flags: u32) -> i32;
type CuDeviceGetCount = unsafe extern "system" fn(count: *mut i32) -> i32;
type CuMemAlloc = unsafe extern "system" fn(dptr: *mut u64, bytesize: usize) -> i32;
type CuMemFree = unsafe extern "system" fn(dptr: u64) -> i32;
type CuMemcpyHtoD = unsafe extern "system" fn(dst: u64, src: *const std::ffi::c_void, bytes: usize) -> i32;
type CuMemcpyDtoH = unsafe extern "system" fn(dst: *mut std::ffi::c_void, src: u64, bytes: usize) -> i32;
// Simplified launch signature
type CuLaunchKernel = unsafe extern "system" fn(
    f: u64, 
    gridDimX: u32, gridDimY: u32, gridDimZ: u32,
    blockDimX: u32, blockDimY: u32, blockDimZ: u32,
    sharedMemBytes: u32, hStream: u64,
    kernelParams: *mut *mut std::ffi::c_void,
    extra: *mut *mut std::ffi::c_void
) -> i32;

pub struct CudaBackend {
    lib: Arc<Library>,
    // Function pointers would go here, but for simplicity we load them on demand or store in a struct
    // For this implementation, we will act as if we have them.
    // In a full implementation, better to use a wrapper struct for the API.
}

impl CudaBackend {
    pub fn try_new() -> Result<Self, String> {
        unsafe {
            let lib_name = if cfg!(target_os = "windows") {
                "nvcuda.dll"
            } else {
                "libcuda.so"
            };
            
            let lib = Library::new(lib_name).map_err(|e| format!("Could not load CUDA driver: {}", e))?;
            
            // Initialize CUDA
            let cu_init: Symbol<CuInit> = lib.get(b"cuInit\0").map_err(|e| e.to_string())?;
            let res = cu_init(0);
            if res != 0 {
                return Err(format!("cuInit failed with error {}", res));
            }
            
            Ok(CudaBackend { lib: Arc::new(lib) })
        }
    }
    
    fn get_symbol<T>(&self, name: &[u8]) -> Result<Symbol<T>, String> {
        unsafe {
            self.lib.get(name).map_err(|e| e.to_string())
        }
    }
}

impl GpuContext for CudaBackend {
    fn backend_type(&self) -> GpuBackendType {
        GpuBackendType::Cuda
    }
    
    fn alloc(&self, size: usize) -> Result<u64, String> {
        unsafe {
            let cu_mem_alloc: Symbol<CuMemAlloc> = self.get_symbol(b"cuMemAlloc\0")?;
            let mut ptr: u64 = 0;
            let res = cu_mem_alloc(&mut ptr, size);
            if res == 0 {
                Ok(ptr)
            } else {
                Err(format!("CUDA alloc failed: {}", res))
            }
        }
    }
    
    fn free(&self, ptr: u64) -> Result<(), String> {
        unsafe {
            let cu_mem_free: Symbol<CuMemFree> = self.get_symbol(b"cuMemFree\0")?;
            let res = cu_mem_free(ptr);
             if res == 0 {
                Ok(())
            } else {
                Err(format!("CUDA free failed: {}", res))
            }
        }
    }
    
    fn memcpy_h2d(&self, dst: u64, src: &[u8]) -> Result<(), String> {
        unsafe {
            let cu_memcpy: Symbol<CuMemcpyHtoD> = self.get_symbol(b"cuMemcpyHtoD\0")?;
            let res = cu_memcpy(dst, src.as_ptr() as *const _, src.len());
            if res == 0 {
                Ok(())
            } else {
                Err(format!("CUDA H2D copy failed: {}", res))
            }
        }
    }
    
    fn memcpy_d2h(&self, src: u64, size: usize) -> Result<Vec<u8>, String> {
        let mut buf = vec![0u8; size];
        unsafe {
            let cu_memcpy: Symbol<CuMemcpyDtoH> = self.get_symbol(b"cuMemcpyDtoH\0")?;
            let res = cu_memcpy(buf.as_mut_ptr() as *mut _, src, size);
             if res == 0 {
                Ok(buf)
            } else {
                Err(format!("CUDA D2H copy failed: {}", res))
            }
        }
    }
    
    fn launch_kernel(
        &self,
        _kernel_name: &str, // In real backend, finding the function handle (CUfunction) from module
        grid: [u32; 3],
        block: [u32; 3],
        shared_mem: u32,
        _args: &[KernelArg],
    ) -> Result<(), String> {
        // Limitation: We don't have the compiled kernel module loaded here. 
        // We would need cuModuleLoad -> cuModuleGetFunction -> cuLaunchKernel.
        // For now, we stub this to prevent runtime crashes if called without a kernel.
        // Ideally, we load PTX.
        
        Err("CUDA kernel launch not fully implemented (requires PTX loading)".to_string())
    }
    
    fn synchronize(&self) -> Result<(), String> {
        unsafe {
             // cuCtxSynchronize
            type CuCtxSync = unsafe extern "system" fn() -> i32;
            let sync: Symbol<CuCtxSync> = self.get_symbol(b"cuCtxSynchronize\0")?;
            let res = sync();
             if res == 0 {
                Ok(())
            } else {
                Err(format!("CUDA sync failed: {}", res))
            }
        }
    }
}

pub struct MockBackend {
    pub allocated_bytes: std::sync::atomic::AtomicUsize,
    pub launch_count: std::sync::atomic::AtomicUsize,
}

impl MockBackend {
    pub fn new() -> Self {
        Self {
            allocated_bytes: std::sync::atomic::AtomicUsize::new(0),
            launch_count: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}

impl GpuContext for MockBackend {
    fn backend_type(&self) -> GpuBackendType {
        GpuBackendType::Mock
    }
    
    fn alloc(&self, size: usize) -> Result<u64, String> {
        self.allocated_bytes.fetch_add(size, std::sync::atomic::Ordering::SeqCst);
        Ok(1) // Dummy pointer
    }
    
    fn free(&self, _ptr: u64) -> Result<(), String> {
        Ok(())
    }
    
    fn memcpy_h2d(&self, _dst: u64, _src: &[u8]) -> Result<(), String> {
        Ok(())
    }
    
    fn memcpy_d2h(&self, _src: u64, size: usize) -> Result<Vec<u8>, String> {
        Ok(vec![0u8; size])
    }
    
    fn launch_kernel(
        &self,
        _kernel_name: &str,
        _grid: [u32; 3],
        _block: [u32; 3],
        _shared_mem: u32,
        _args: &[KernelArg],
    ) -> Result<(), String> {
        self.launch_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
    
    fn synchronize(&self) -> Result<(), String> {
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Device Management
// ─────────────────────────────────────────────────────────────────────────────

/// Physical GPU device information
#[derive(Debug, Clone)]
pub struct GpuDevice {
    /// Device index
    pub id: usize,
    /// Device name (e.g., "NVIDIA RTX 4090")
    pub name: String,
    /// Backend this device uses
    pub backend: GpuBackendType,
    /// Total device memory in bytes
    pub total_memory: u64,
    /// Available device memory in bytes
    pub available_memory: u64,
    /// Number of streaming multiprocessors / compute units
    pub compute_units: u32,
    /// Maximum threads per block (workgroup)
    pub max_threads_per_block: u32,
    /// Maximum block dimensions (x, y, z)
    pub max_block_dims: [u32; 3],
    /// Maximum grid dimensions (x, y, z)
    pub max_grid_dims: [u32; 3],
    /// Shared memory per block in bytes
    pub shared_memory_per_block: u32,
    /// Warp/wavefront size
    pub warp_size: u32,
    /// Compute capability (major, minor) - CUDA specific
    pub compute_capability: (u32, u32),
    /// Supports unified (managed) memory
    pub supports_unified_memory: bool,
    /// Supports cooperative groups
    pub supports_cooperative_groups: bool,
    /// Supports tensor cores
    pub supports_tensor_cores: bool,
    /// Clock rate in MHz
    pub clock_rate_mhz: u32,
    /// Memory bus width in bits
    pub memory_bus_width: u32,
    /// Peak memory bandwidth in GB/s
    pub memory_bandwidth_gbps: f64,
}

impl GpuDevice {
    /// Create a software fallback "device"
    pub fn software_fallback() -> Self {
        GpuDevice {
            id: 0,
            name: "Software (CPU Emulation)".to_string(),
            backend: GpuBackendType::Software,
            total_memory: 0,
            available_memory: 0,
            compute_units: 1,
            max_threads_per_block: 1024,
            max_block_dims: [1024, 1024, 64],
            max_grid_dims: [65535, 65535, 65535],
            shared_memory_per_block: 49152,
            warp_size: 32,
            compute_capability: (0, 0),
            supports_unified_memory: false,
            supports_cooperative_groups: false,
            supports_tensor_cores: false,
            clock_rate_mhz: 0,
            memory_bus_width: 0,
            memory_bandwidth_gbps: 0.0,
        }
    }
    
    /// Theoretical peak TFLOPS (single precision)
    pub fn peak_tflops_f32(&self) -> f64 {
        // FMA: 2 ops per clock per core, 32 threads per warp
        let cores_per_sm = match self.compute_capability {
            (8, _) => 128, // Ampere
            (9, _) => 128, // Hopper
            (7, _) => 64,  // Volta/Turing
            _ => 64,
        };
        let total_cores = self.compute_units as f64 * cores_per_sm as f64;
        total_cores * self.clock_rate_mhz as f64 * 2.0 / 1e6
    }
}

/// Device manager: enumerate and select GPU devices
pub struct DeviceManager {
    /// All detected devices
    devices: Vec<GpuDevice>,
    /// Currently selected device index
    active_device: usize,
    /// Preferred backend
    pub preferred_backend: Option<GpuBackendType>,
    /// Active backend contexts
    contexts: HashMap<usize, std::sync::Arc<dyn GpuContext>>,
}

impl DeviceManager {
    pub fn new() -> Self {
        let mut manager = DeviceManager {
            devices: Vec::new(),
            active_device: 0,
            preferred_backend: None,
            contexts: HashMap::new(),
        };
        manager.enumerate_devices();
        manager
    }
    
    /// Enumerate all available GPU devices
    pub fn enumerate_devices(&mut self) {
        self.devices.clear();
        
        // Probe CUDA
        self.probe_cuda();
        // Probe OpenCL
        self.probe_opencl();
        // Probe Vulkan
        self.probe_vulkan();
        
        // Always add software fallback
        self.probe_software();

        if self.devices.is_empty() {
            warn!("GPU: No devices found (internal error)");
        } else {
            info!("GPU: Found {} device(s)", self.devices.len());
            for dev in &self.devices {
                info!("  [{}/{}] {} ({} MB, {} CUs)",
                      dev.id, dev.backend, dev.name,
                      dev.total_memory / (1024 * 1024),
                      dev.compute_units);
            }
        }
    }
    
    /// Probe for software fallback
    fn probe_software(&mut self) {
        // Only add if not already present
        if self.devices.iter().any(|d| d.backend == GpuBackendType::Software) {
            return;
        }

        debug!("GPU: Enabling software fallback backend");
        
        let device_id = self.devices.len();
        // Create device info
        let mut device = GpuDevice::software_fallback();
        device.id = device_id;
        
        // Create context
        let backend = Arc::new(SoftwareBackend::new());
        self.contexts.insert(device_id, backend);
        
        self.devices.push(device);
    }
    
    /// Probe for CUDA-capable devices
    fn probe_cuda(&mut self) {
        // Try to load CUDA driver
        match CudaBackend::try_new() {
            Ok(backend) => {
                debug!("GPU: CUDA driver detected and initialized");
                
                // We should query device count here using library
                // let count = backend.get_device_count();
                // For now, assume 1 device if driver loads
                
                let device_id = self.devices.len();
                let backend_arc = Arc::new(backend);
                self.contexts.insert(device_id, backend_arc);
                
                self.devices.push(GpuDevice {
                    id: device_id,
                    name: "CUDA Device (Active)".to_string(),
                    backend: GpuBackendType::Cuda,
                    // Detailed properties would be queries via cuDeviceGetAttribute
                    total_memory: 0, // TODO: query
                    available_memory: 0,
                    compute_units: 0,
                    max_threads_per_block: 1024,
                    max_block_dims: [1024, 1024, 64],
                    max_grid_dims: [2147483647, 65535, 65535],
                    shared_memory_per_block: 49152,
                    warp_size: 32,
                    compute_capability: (0, 0),
                    supports_unified_memory: true,
                    supports_cooperative_groups: true,
                    supports_tensor_cores: false,
                    clock_rate_mhz: 0,
                    memory_bus_width: 0,
                    memory_bandwidth_gbps: 0.0,
                });
            },
            Err(e) => {
                debug!("GPU: CUDA driver not available: {}", e);
            }
        }
    }
    
    /// Probe for OpenCL-capable devices
    fn probe_opencl(&mut self) {
        #[cfg(target_os = "windows")]
        {
            if std::path::Path::new("C:\\Windows\\System32\\OpenCL.dll").exists() {
                debug!("GPU: OpenCL runtime detected");
                // TODO: Add OpenCL device detection
            }
        }
    }
    
    /// Probe for Vulkan compute support
    fn probe_vulkan(&mut self) {
        #[cfg(target_os = "windows")]
        {
            if std::path::Path::new("C:\\Windows\\System32\\vulkan-1.dll").exists() {
                debug!("GPU: Vulkan runtime detected");
                // TODO: Add Vulkan device detection
            }
        }
    }
    
    /// Get all detected devices
    pub fn devices(&self) -> &[GpuDevice] {
        &self.devices
    }
    
    /// Get the active device
    pub fn active_device(&self) -> &GpuDevice {
        &self.devices[self.active_device]
    }
    
    /// Select a device by index
    pub fn select_device(&mut self, index: usize) -> Result<(), String> {
        if index >= self.devices.len() {
            return Err(format!("Device index {} out of range (have {} devices)",
                              index, self.devices.len()));
        }
        self.active_device = index;
        info!("GPU: Selected device [{}] {}", index, self.devices[index].name);
        Ok(())
    }
    
    /// Select best device for a given backend preference
    pub fn select_best_device(&mut self, backend: Option<GpuBackendType>) -> &GpuDevice {
        if let Some(preferred) = backend {
            if let Some(pos) = self.devices.iter().position(|d| d.backend == preferred) {
                self.active_device = pos;
            }
        } else {
            // Heuristic: prefer CUDA > Vulkan > OpenCL > Software
            let priority = |b: &GpuBackendType| match b {
                GpuBackendType::Cuda => 4,
                GpuBackendType::Vulkan => 3,
                GpuBackendType::OpenCL => 2,
                GpuBackendType::Metal => 2,
                GpuBackendType::Software => 0,
                GpuBackendType::Mock => 0,
            };
            if let Some((idx, _)) = self.devices.iter()
                .enumerate()
                .max_by_key(|(_, d)| priority(&d.backend))
            {
                self.active_device = idx;
            }
        }
        &self.devices[self.active_device]
    }
    
    /// Get the GpuContext for the active device
    pub fn context(&self) -> Option<std::sync::Arc<dyn GpuContext>> {
        self.contexts.get(&self.active_device).cloned()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GPU Memory Management
// ─────────────────────────────────────────────────────────────────────────────

/// GPU memory allocation
#[derive(Debug, Clone)]
pub struct GpuAllocation {
    /// Unique allocation ID
    pub id: u64,
    /// Size in bytes
    pub size: usize,
    /// Memory type
    pub mem_type: GpuMemoryType,
    /// Device ID this allocation belongs to
    pub device_id: usize,
    /// Whether the allocation is currently valid
    pub valid: bool,
    /// Optional host-side copy for software/pinned/unified allocations
    pub data: Option<Vec<u8>>,
}

/// Types of GPU memory
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuMemoryType {
    /// Device-only memory (fastest, no CPU access)
    Device,
    /// Host-pinned memory (fast transfer, CPU accessible)
    Pinned,
    /// Unified/managed memory (auto-migrating)
    Unified,
    /// Shared memory (per-block scratchpad)
    Shared,
    /// Constant memory (read-only, cached)
    Constant,
    /// Texture memory (spatially cached)
    Texture,
}

/// GPU memory manager
pub struct GpuMemoryManager {
    /// Active allocations
    allocations: HashMap<u64, GpuAllocation>,
    /// Next allocation ID
    next_id: u64,
    /// Total allocated bytes per device
    device_usage: HashMap<usize, usize>,
    /// Peak allocation per device
    peak_usage: HashMap<usize, usize>,
    /// Memory pool for reuse (size -> list of freed allocation IDs)
    free_pools: HashMap<usize, Vec<u64>>,
}

impl GpuMemoryManager {
    pub fn new() -> Self {
        GpuMemoryManager {
            allocations: HashMap::new(),
            next_id: 1,
            device_usage: HashMap::new(),
            peak_usage: HashMap::new(),
            free_pools: HashMap::new(),
        }
    }
    
    /// Allocate GPU memory
    pub fn alloc(
        &mut self,
        device: &GpuDevice,
        size: usize,
        mem_type: GpuMemoryType,
    ) -> Result<u64, String> {
        // Check if we can reuse a freed allocation of the same size
        if let Some(pool) = self.free_pools.get_mut(&size) {
            if let Some(reused_id) = pool.pop() {
                if let Some(alloc) = self.allocations.get_mut(&reused_id) {
                    alloc.valid = true;
                    alloc.mem_type = mem_type;
                    debug!("GPU Memory: Reused allocation {} ({} bytes)", reused_id, size);
                    return Ok(reused_id);
                }
            }
        }
        
        // Check available memory
        let current_usage = self.device_usage.get(&device.id).copied().unwrap_or(0);
        if device.total_memory > 0 && current_usage + size > device.total_memory as usize {
            return Err(format!(
                "GPU Memory: Out of memory on device {} (requested {} bytes, {} / {} used)",
                device.name, size, current_usage, device.total_memory
            ));
        }
        
        let id = self.next_id;
        self.next_id += 1;
        
        let data_buf = if device.backend == GpuBackendType::Software || device.backend == GpuBackendType::Mock || matches!(mem_type, GpuMemoryType::Pinned | GpuMemoryType::Unified) {
            Some(vec![0u8; size])
        } else {
            None
        };

        let alloc = GpuAllocation {
            id,
            size,
            mem_type,
            device_id: device.id,
            valid: true,
            data: data_buf,
        };
        
        self.allocations.insert(id, alloc);
        
        let usage = self.device_usage.entry(device.id).or_insert(0);
        *usage += size;
        
        let peak = self.peak_usage.entry(device.id).or_insert(0);
        if *usage > *peak {
            *peak = *usage;
        }
        
        debug!("GPU Memory: Allocated {} bytes (id={}, type={:?}, device={})",
               size, id, mem_type, device.name);
        
        Ok(id)
    }
    
    /// Free GPU memory
    pub fn free(&mut self, alloc_id: u64) -> Result<(), String> {
        if let Some(alloc) = self.allocations.get_mut(&alloc_id) {
            if !alloc.valid {
                return Err(format!("GPU Memory: Double free of allocation {}", alloc_id));
            }
            
            alloc.valid = false;
            
            if let Some(usage) = self.device_usage.get_mut(&alloc.device_id) {
                *usage = usage.saturating_sub(alloc.size);
            }
            
            // Add to free pool for reuse
            self.free_pools
                .entry(alloc.size)
                .or_insert_with(Vec::new)
                .push(alloc_id);
            
            debug!("GPU Memory: Freed allocation {} ({} bytes)", alloc_id, alloc.size);
            Ok(())
        } else {
            Err(format!("GPU Memory: Unknown allocation {}", alloc_id))
        }
    }
    
    /// Copy data host → device
    pub fn copy_to_device(
        &mut self,
        alloc_id: u64,
        _data: &[u8],
    ) -> Result<(), String> {
        if let Some(alloc_mut) = self.allocations.get_mut(&alloc_id) {
            if !alloc_mut.valid {
                return Err("GPU Memory: Copy to freed allocation".to_string());
            }
            if _data.len() > alloc_mut.size {
                return Err(format!("GPU Memory: Copy size ({}) exceeds allocation ({})",
                                   _data.len(), alloc_mut.size));
            }

            debug!("GPU Memory: H2D copy {} bytes to alloc {}", _data.len(), alloc_id);
            // If we have a host-side buffer for this allocation, copy into it
            if let Some(buf) = &mut alloc_mut.data {
                buf[.._data.len()].copy_from_slice(_data);
                return Ok(());
            }

            // In production: call cudaMemcpy(H2D) / clEnqueueWriteBuffer / vkCmdCopyBuffer
            Ok(())
        } else {
            Err(format!("GPU Memory: Unknown allocation {}", alloc_id))
        }
    }
    
    /// Copy data device → host
    pub fn copy_from_device(
        &self,
        alloc_id: u64,
        size: usize,
    ) -> Result<Vec<u8>, String> {
        if let Some(alloc) = self.allocations.get(&alloc_id) {
            if !alloc.valid {
                return Err("GPU Memory: Copy from freed allocation".to_string());
            }
            if size > alloc.size {
                return Err(format!("GPU Memory: Copy size ({}) exceeds allocation ({})",
                                   size, alloc.size));
            }

            debug!("GPU Memory: D2H copy {} bytes from alloc {}", size, alloc_id);
            // If we have a host-side buffer for this allocation, return its contents
            if let Some(buf) = &alloc.data {
                return Ok(buf[..size].to_vec());
            }

            // In production: call cudaMemcpy(D2H) / clEnqueueReadBuffer
            Ok(vec![0u8; size]) // placeholder
        } else {
            Err(format!("GPU Memory: Unknown allocation {}", alloc_id))
        }
    }
    
    /// Copy data device → device
    pub fn copy_device_to_device(
        &mut self,
        src_id: u64,
        dst_id: u64,
        size: usize,
    ) -> Result<(), String> {
        // Remove the source allocation temporarily so we can mutably access the destination
        let src_alloc = match self.allocations.remove(&src_id) {
            Some(a) => a,
            None => return Err(format!("GPU Memory: Unknown allocation {}", src_id)),
        };

        let result = if let Some(dst_mut) = self.allocations.get_mut(&dst_id) {
            if !src_alloc.valid || !dst_mut.valid {
                Err("GPU Memory: Invalid allocation in D2D copy".to_string())
            } else if let (Some(src_buf), Some(dst_buf)) = (src_alloc.data.as_ref(), dst_mut.data.as_mut()) {
                let copy_size = size.min(src_buf.len()).min(dst_buf.len());
                dst_buf[..copy_size].copy_from_slice(&src_buf[..copy_size]);
                Ok(())
            } else {
                // Platform-specific D2D copy would be used here
                Ok(())
            }
        } else {
            Err(format!("GPU Memory: Unknown destination allocation {}", dst_id))
        };

        // Reinsert the source allocation back into the map
        self.allocations.insert(src_id, src_alloc);
        result
    }
    
    /// Get memory statistics
    pub fn stats(&self, device_id: usize) -> GpuMemoryStats {
        GpuMemoryStats {
            allocated: self.device_usage.get(&device_id).copied().unwrap_or(0),
            peak: self.peak_usage.get(&device_id).copied().unwrap_or(0),
            active_allocations: self.allocations.values()
                .filter(|a| a.device_id == device_id && a.valid)
                .count(),
            pooled_allocations: self.free_pools.values()
                .map(|v| v.len())
                .sum(),
        }
    }
}

#[derive(Debug)]
pub struct GpuMemoryStats {
    pub allocated: usize,
    pub peak: usize,
    pub active_allocations: usize,
    pub pooled_allocations: usize,
}

// ─────────────────────────────────────────────────────────────────────────────
// Kernel Launch Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Kernel launch dimensions
#[derive(Debug, Clone)]
pub struct LaunchConfig {
    /// Grid dimensions (number of blocks)
    pub grid: [u32; 3],
    /// Block dimensions (threads per block)
    pub block: [u32; 3],
    /// Dynamic shared memory in bytes
    pub shared_mem_bytes: u32,
    /// Stream/queue for async execution
    pub stream: Option<GpuStream>,
}

impl LaunchConfig {
    /// Create 1D launch configuration
    pub fn dim1d(total_threads: u32, block_size: u32) -> Self {
        let grid_x = (total_threads + block_size - 1) / block_size;
        LaunchConfig {
            grid: [grid_x, 1, 1],
            block: [block_size, 1, 1],
            shared_mem_bytes: 0,
            stream: None,
        }
    }
    
    /// Create 2D launch configuration
    pub fn dim2d(width: u32, height: u32, block_x: u32, block_y: u32) -> Self {
        LaunchConfig {
            grid: [
                (width + block_x - 1) / block_x,
                (height + block_y - 1) / block_y,
                1,
            ],
            block: [block_x, block_y, 1],
            shared_mem_bytes: 0,
            stream: None,
        }
    }
    
    /// Create 3D launch configuration
    pub fn dim3d(
        x: u32, y: u32, z: u32,
        bx: u32, by: u32, bz: u32,
    ) -> Self {
        LaunchConfig {
            grid: [
                (x + bx - 1) / bx,
                (y + by - 1) / by,
                (z + bz - 1) / bz,
            ],
            block: [bx, by, bz],
            shared_mem_bytes: 0,
            stream: None,
        }
    }
    
    /// Set shared memory size
    pub fn with_shared_mem(mut self, bytes: u32) -> Self {
        self.shared_mem_bytes = bytes;
        self
    }
    
    /// Assign to a stream
    pub fn with_stream(mut self, stream: GpuStream) -> Self {
        self.stream = Some(stream);
        self
    }
    
    /// Total number of threads
    pub fn total_threads(&self) -> u64 {
        let grid_total = self.grid[0] as u64 * self.grid[1] as u64 * self.grid[2] as u64;
        let block_total = self.block[0] as u64 * self.block[1] as u64 * self.block[2] as u64;
        grid_total * block_total
    }
    
    /// Validate launch config against device limits
    pub fn validate(&self, device: &GpuDevice) -> Result<(), String> {
        let threads_per_block = self.block[0] * self.block[1] * self.block[2];
        if threads_per_block > device.max_threads_per_block {
            return Err(format!(
                "Threads per block ({}) exceeds device limit ({})",
                threads_per_block, device.max_threads_per_block
            ));
        }
        
        for i in 0..3 {
            if self.block[i] > device.max_block_dims[i] {
                return Err(format!(
                    "Block dimension {} ({}) exceeds device limit ({})",
                    i, self.block[i], device.max_block_dims[i]
                ));
            }
            if self.grid[i] > device.max_grid_dims[i] {
                return Err(format!(
                    "Grid dimension {} ({}) exceeds device limit ({})",
                    i, self.grid[i], device.max_grid_dims[i]
                ));
            }
        }
        
        if self.shared_mem_bytes > device.shared_memory_per_block {
            return Err(format!(
                "Shared memory ({} bytes) exceeds device limit ({} bytes)",
                self.shared_mem_bytes, device.shared_memory_per_block
            ));
        }
        
        Ok(())
    }
    
    /// Auto-tune block size for optimal occupancy
    pub fn auto_tune(total_elements: u32, device: &GpuDevice) -> Self {
        // Heuristic: choose block size that maximizes occupancy
        let warp_size = device.warp_size;
        
        // Try common block sizes: 64, 128, 256, 512, 1024
        let candidates = [64, 128, 256, 512, 1024u32];
        let mut best_block = warp_size.max(64);
        let mut best_occupancy = 0.0f64;
        
        for &block_size in &candidates {
            if block_size > device.max_threads_per_block {
                continue;
            }
            
            // Simple occupancy model
            let warps_per_block = (block_size + warp_size - 1) / warp_size;
            let max_warps = device.max_threads_per_block / warp_size;
            let occupancy = warps_per_block as f64 / max_warps as f64;
            
            if occupancy > best_occupancy {
                best_occupancy = occupancy;
                best_block = block_size;
            }
        }
        
        LaunchConfig::dim1d(total_elements, best_block)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GPU Streams / Command Queues
// ─────────────────────────────────────────────────────────────────────────────

/// GPU execution stream for async operations
#[derive(Debug, Clone)]
pub struct GpuStream {
    /// Stream ID
    pub id: u64,
    /// Associated device
    pub device_id: usize,
    /// Priority (lower = higher priority)
    pub priority: i32,
    /// Pending operations count
    pub pending_ops: u32,
}

/// GPU event for synchronization
#[derive(Debug, Clone)]
pub struct GpuEvent {
    /// Event ID
    pub id: u64,
    /// Stream that recorded this event
    pub stream_id: u64,
    /// Whether the event has been signaled
    pub signaled: bool,
    /// Timestamp when signaled (nanoseconds)
    pub timestamp_ns: Option<u64>,
}

/// Stream manager for async GPU execution
pub struct StreamManager {
    /// Active streams
    streams: HashMap<u64, GpuStream>,
    /// Events for synchronization
    events: HashMap<u64, GpuEvent>,
    /// Next stream ID
    next_stream_id: u64,
    /// Next event ID
    next_event_id: u64,
}

impl StreamManager {
    pub fn new() -> Self {
        StreamManager {
            streams: HashMap::new(),
            events: HashMap::new(),
            next_stream_id: 1,
            next_event_id: 1,
        }
    }
    
    /// Create a new GPU stream
    pub fn create_stream(&mut self, device_id: usize, priority: i32) -> GpuStream {
        let stream = GpuStream {
            id: self.next_stream_id,
            device_id,
            priority,
            pending_ops: 0,
        };
        self.next_stream_id += 1;
        
        self.streams.insert(stream.id, stream.clone());
        info!("GPU Stream: Created stream {} on device {} (priority={})",
              stream.id, device_id, priority);
        stream
    }
    
    /// Record an event on a stream
    pub fn record_event(&mut self, stream_id: u64) -> GpuEvent {
        let event = GpuEvent {
            id: self.next_event_id,
            stream_id,
            signaled: false,
            timestamp_ns: None,
        };
        self.next_event_id += 1;
        
        self.events.insert(event.id, event.clone());
        event
    }
    
    /// Wait for an event on a different stream (stream dependency)
    pub fn wait_event(&mut self, stream_id: u64, event_id: u64) -> Result<(), String> {
        if !self.streams.contains_key(&stream_id) {
            return Err(format!("Unknown stream {}", stream_id));
        }
        if !self.events.contains_key(&event_id) {
            return Err(format!("Unknown event {}", event_id));
        }
        
        debug!("GPU Stream: Stream {} waiting on event {}", stream_id, event_id);
        Ok(())
    }
    
    /// Synchronize a stream (wait for all pending operations)
    pub fn synchronize_stream(&mut self, stream_id: u64) -> Result<(), String> {
        if let Some(stream) = self.streams.get_mut(&stream_id) {
            debug!("GPU Stream: Synchronizing stream {} ({} pending ops)",
                   stream_id, stream.pending_ops);
            stream.pending_ops = 0;
            Ok(())
        } else {
            Err(format!("Unknown stream {}", stream_id))
        }
    }
    
    /// Synchronize all streams on a device
    pub fn synchronize_device(&mut self, device_id: usize) {
        for stream in self.streams.values_mut() {
            if stream.device_id == device_id {
                stream.pending_ops = 0;
            }
        }
        debug!("GPU Stream: Synchronized all streams on device {}", device_id);
    }
    
    /// Destroy a stream
    pub fn destroy_stream(&mut self, stream_id: u64) {
        self.streams.remove(&stream_id);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Kernel Representation
// ─────────────────────────────────────────────────────────────────────────────

/// A compiled GPU kernel
#[derive(Debug, Clone)]
pub struct GpuKernel {
    /// Kernel name
    pub name: String,
    /// Source IR function
    pub source_function: String,
    /// Backend-specific compiled code (PTX, SPIR-V, etc.)
    pub compiled_code: Vec<u8>,
    /// Kernel parameter types
    pub params: Vec<KernelParam>,
    /// Register usage per thread
    pub registers_per_thread: u32,
    /// Shared memory usage in bytes
    pub shared_mem_static: u32,
    /// Whether this kernel uses dynamic shared memory
    pub uses_dynamic_shared_mem: bool,
    /// Backend format
    pub format: KernelFormat,
}

/// Kernel parameter description
#[derive(Debug, Clone)]
pub struct KernelParam {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: KernelParamType,
    /// Size in bytes
    pub size: usize,
    /// Whether this is an output parameter
    pub is_output: bool,
}

/// Types of kernel parameters
#[derive(Debug, Clone, Copy)]
pub enum KernelParamType {
    Pointer,     // Device memory pointer
    Int32,
    Int64,
    Float32,
    Float64,
    Struct(u32), // Size of struct
}

/// Format of compiled kernel code
#[derive(Debug, Clone, Copy)]
pub enum KernelFormat {
    /// NVIDIA PTX (Parallel Thread Execution)
    Ptx,
    /// SPIR-V (Vulkan/OpenCL)
    SpirV,
    /// OpenCL C source
    OpenClC,
    /// Metal Shading Language (compiled)
    MetalLib,
    /// DXIL (DirectX Intermediate Language)
    Dxil,
}

// ─────────────────────────────────────────────────────────────────────────────
// IR → GPU Kernel Compilation
// ─────────────────────────────────────────────────────────────────────────────

/// Compiles IR functions annotated with @gpu to GPU kernels
pub struct GpuKernelCompiler {
    /// Target backend
    backend: GpuBackendType,
    /// Optimization level (0-3)
    opt_level: u32,
}

impl GpuKernelCompiler {
    pub fn new(backend: GpuBackendType) -> Self {
        GpuKernelCompiler {
            backend,
            opt_level: 2,
        }
    }
    
    /// Compile an IR function to a GPU kernel
    pub fn compile_kernel(&self, func: &IrFunction) -> Result<GpuKernel, String> {
        info!("GPU Compiler: Compiling {} for {}", func.name, self.backend);
        
        // Extract kernel parameters from function signature
        let params: Vec<KernelParam> = func.params.iter()
            .map(|(name, ty)| KernelParam {
                name: name.clone(),
                param_type: self.ir_type_to_kernel_param(ty),
                size: self.type_size(ty),
                is_output: name.starts_with("out_") || name.starts_with("result"),
            })
            .collect();
        
        // Generate backend-specific code
        let (compiled_code, format) = match self.backend {
            GpuBackendType::Cuda => self.emit_ptx(func)?,
            GpuBackendType::Vulkan | GpuBackendType::OpenCL => self.emit_spirv(func)?,
            GpuBackendType::Metal => self.emit_metal(func)?,
            GpuBackendType::Software | GpuBackendType::Mock => (Vec::new(), KernelFormat::Ptx),
        };
        
        Ok(GpuKernel {
            name: format!("{}_kernel", func.name),
            source_function: func.name.clone(),
            compiled_code,
            params,
            registers_per_thread: self.estimate_registers(func),
            shared_mem_static: self.estimate_shared_mem(func),
            uses_dynamic_shared_mem: false,
            format,
        })
    }
    
    /// Emit NVIDIA PTX assembly from IR
    fn emit_ptx(&self, func: &IrFunction) -> Result<(Vec<u8>, KernelFormat), String> {
        let mut ptx = String::new();
        
        // PTX header
        ptx.push_str(".version 7.8\n");
        ptx.push_str(".target sm_70\n"); // Volta+
        ptx.push_str(".address_size 64\n\n");
        
        // Kernel entry point
        ptx.push_str(&format!(".visible .entry {}(\n", func.name));
        
        // Parameters
        for (i, (name, ty)) in func.params.iter().enumerate() {
            let ptx_type = self.ir_type_to_ptx(ty);
            if i > 0 { ptx.push_str(",\n"); }
            ptx.push_str(&format!("    .param {} param_{}", ptx_type, name));
        }
        ptx.push_str("\n) {\n");
        
        // Register declarations
        ptx.push_str("    .reg .pred %p<16>;\n");
        ptx.push_str("    .reg .b32 %r<64>;\n");
        ptx.push_str("    .reg .b64 %rd<64>;\n");
        ptx.push_str("    .reg .f32 %f<32>;\n");
        ptx.push_str("    .reg .f64 %fd<16>;\n\n");
        
        // Thread ID computation
        ptx.push_str("    // Thread index\n");
        ptx.push_str("    mov.u32 %r0, %tid.x;\n");
        ptx.push_str("    mov.u32 %r1, %ctaid.x;\n");
        ptx.push_str("    mov.u32 %r2, %ntid.x;\n");
        ptx.push_str("    mad.lo.u32 %r3, %r1, %r2, %r0; // global_tid\n\n");
        
        // Emit IR instructions as PTX
        for block in &func.blocks {
            ptx.push_str(&format!("{}:\n", block.label));
            
            for inst in &block.instructions {
                match inst {
                    IrInstruction::BinOp { dest, op, left, right } => {
                        let l = self.value_to_ptx_reg(left);
                        let r = self.value_to_ptx_reg(right);
                        let op_str = match op {
                            IrBinOp::Add => "add.s64",
                            IrBinOp::Sub => "sub.s64",
                            IrBinOp::Mul => "mul.lo.s64",
                            IrBinOp::Div => "div.s64",
                            _ => "add.s64", // fallback
                        };
                        ptx.push_str(&format!("    {} %rd{}, {}, {};\n",
                                             op_str, dest, l, r));
                    }
                    IrInstruction::Load { dest, ptr, ty } => {
                        let ptx_type = self.ir_type_to_ptx(ty);
                        ptx.push_str(&format!("    ld.global{} %rd{}, [%rd{}];\n",
                                             ptx_type, dest, ptr));
                    }
                    IrInstruction::Store { ptr, value } => {
                        let v = self.value_to_ptx_reg(value);
                        ptx.push_str(&format!("    st.global.b64 [%rd{}], {};\n",
                                             ptr, v));
                    }
                    _ => {
                        ptx.push_str(&format!("    // Unsupported: {:?}\n", inst));
                    }
                }
            }
            
            // Terminator
            match &block.terminator {
                IrTerminator::Return(_) => {
                    ptx.push_str("    ret;\n");
                }
                IrTerminator::Branch(target) => {
                    ptx.push_str(&format!("    bra {};\n", target));
                }
                IrTerminator::CondBranch { cond, then_label, else_label } => {
                    let c = self.value_to_ptx_reg(cond);
                    ptx.push_str(&format!("    setp.ne.b64 %p0, {}, 0;\n", c));
                    ptx.push_str(&format!("    @%p0 bra {};\n", then_label));
                    ptx.push_str(&format!("    bra {};\n", else_label));
                }
                IrTerminator::Unreachable => {
                    ptx.push_str("    trap;\n");
                }
            }
        }
        
        ptx.push_str("}\n");
        
        Ok((ptx.into_bytes(), KernelFormat::Ptx))
    }
    
    /// Emit SPIR-V from IR (framework in place; full module generation pending)
    fn emit_spirv(&self, func: &IrFunction) -> Result<(Vec<u8>, KernelFormat), String> {
        let mut spirv = String::new();
        spirv.push_str("; SPIR-V textual module generated from Omni IR\n");
        spirv.push_str("OpCapability Shader\n");
        spirv.push_str("OpMemoryModel Logical GLSL450\n");
        spirv.push_str(&format!("OpEntryPoint GLCompute %{} \"{}\" %gid\n", func.name, func.name));
        spirv.push_str("OpDecorate %gid BuiltIn GlobalInvocationId\n");
        spirv.push_str(&format!("OpName %{} \"{}\"\n\n", func.name, func.name));

        for block in &func.blocks {
            spirv.push_str(&format!("OpLabel %{}\n", block.label));

            for inst in &block.instructions {
                match inst {
                    IrInstruction::BinOp { dest, op, left, right } => {
                        let op_name = self.spirv_binop_str(op);
                        let type_name = if self.spirv_is_comparison(op) {
                            "%bool"
                        } else {
                            self.spirv_type_str(&IrType::I64)
                        };
                        spirv.push_str(&format!("{} = {} {} {} {}\n",
                                             self.spirv_dest(dest),
                                             op_name,
                                             type_name,
                                             self.spirv_operand(left),
                                             self.spirv_operand(right)));
                    }
                    _ => {
                         // Simplify for brevity - full implementation would cover all instructions
                         spirv.push_str(&format!("; Unsupported instruction for SPIR-V prototype: {:?}\n", inst));
                    }
                }
            }
        }
        
        Ok((spirv.into_bytes(), KernelFormat::SpirV))
    }









    
    /// Emit Metal shader (MSL generation framework in place; full codegen pending)
    fn emit_metal(&self, func: &IrFunction) -> Result<(Vec<u8>, KernelFormat), String> {
        let mut msl = String::new();
        
        // Metal standard library header
        msl.push_str("#include <metal_stdlib>\n");
        msl.push_str("using namespace metal;\n\n");
        
        // Kernel function signature
        msl.push_str(&format!("kernel void {}(\n", func.name));
        
        // Parameter marshaling (converts IR types to Metal types)
        for (i, (name, ty)) in func.params.iter().enumerate() {
            let metal_type = self.metal_param_type(ty);
            if i > 0 { msl.push_str(",\n"); }
            msl.push_str(&format!("    {} {} [[buffer({})]]", metal_type, name, i));
        }
        
        // Thread position attribute (compute shader standard)
        msl.push_str(",\n    uint gid [[thread_position_in_grid]]\n");
        msl.push_str(") {\n");

        let mut locals = HashSet::new();
        for block in &func.blocks {
            for inst in &block.instructions {
                match inst {
                    IrInstruction::BinOp { dest, .. } |
                    IrInstruction::Alloca { dest, .. } |
                    IrInstruction::Load { dest, .. } |
                    IrInstruction::Select { dest, .. } |
                    IrInstruction::Cast { dest, .. } |
                    IrInstruction::GetField { dest, .. } |
                    IrInstruction::Call { dest: Some(dest), .. } => {
                        locals.insert(dest.clone());
                    }
                    _ => {}
                }
            }
        }

        for local in &locals {
            msl.push_str(&format!("    int {} = 0;\n", local));
        }

        let mut body = String::new();
        for block in &func.blocks {
            body.push_str(&format!("    // block {}\n", block.label));
            for inst in &block.instructions {
                match inst {
                    IrInstruction::BinOp { dest, op, left, right } => {
                        body.push_str(&format!("    {} = {} {} {};\n",
                                             dest,
                                             self.metal_value(left),
                                             self.metal_binop_symbol(op),
                                             self.metal_value(right)));
                    }
                    IrInstruction::Load { dest, ptr, .. } => {
                        body.push_str(&format!("    {} = {}[0];\n", dest, ptr));
                    }
                    IrInstruction::Store { ptr, value } => {
                        body.push_str(&format!("    {}[0] = {};\n", ptr, self.metal_value(value)));
                    }
                    IrInstruction::Select { dest, cond, then_val, else_val } => {
                        body.push_str(&format!("    {} = {} ? {} : {};\n",
                                             dest,
                                             self.metal_value(cond),
                                             self.metal_value(then_val),
                                             self.metal_value(else_val)));
                    }
                    IrInstruction::Call { dest: Some(dest), func: name, args } => {
                        let arg_list = args.iter()
                            .map(|a| self.metal_value(a))
                            .collect::<Vec<_>>()
                            .join(", ");
                        body.push_str(&format!("    {} = {}({});\n", dest, name, arg_list));
                    }
                    IrInstruction::Call { dest: None, func: name, args } => {
                        let arg_list = args.iter()
                            .map(|a| self.metal_value(a))
                            .collect::<Vec<_>>()
                            .join(", ");
                        body.push_str(&format!("    {}({});\n", name, arg_list));
                    }
                    IrInstruction::Cast { dest, value, .. } => {
                        body.push_str(&format!("    {} = {}; // cast\n", dest, self.metal_value(value)));
                    }
                    IrInstruction::GetField { dest, ptr, field } => {
                        body.push_str(&format!("    {} = {}.field{};\n", dest, ptr, field));
                    }
                    IrInstruction::Alloca { dest, .. } => {
                        body.push_str(&format!("    {} = 0; // alloca\n", dest));
                    }
                    _ => {
                        body.push_str(&format!("    // Unsupported: {:?}\n", inst));
                    }
                }
            }

            match &block.terminator {
                IrTerminator::Return(Some(value)) => {
                    body.push_str(&format!("    return {};\n", self.metal_value(value)));
                }
                IrTerminator::Return(None) => {
                    body.push_str("    return;\n");
                }
                IrTerminator::Branch(target) => {
                    body.push_str(&format!("    goto {};\n", target));
                }
                IrTerminator::CondBranch { cond, then_label, else_label } => {
                    body.push_str(&format!("    if ({}) {{ goto {}; }} else {{ goto {}; }}\n",
                                         self.metal_value(cond), then_label, else_label));
                }
                IrTerminator::Unreachable => {
                    body.push_str("    // unreachable\n");
                }
            }
        }

        msl.push_str(&body);
        msl.push_str("}\n");

        Ok((msl.into_bytes(), KernelFormat::MetalLib))
    }
    
    // ── Helper functions ──────────────────────────────────────────────────
    
    fn ir_type_to_kernel_param(&self, ty: &IrType) -> KernelParamType {
        match ty {
            IrType::Ptr(_) | IrType::Array(_, _) => KernelParamType::Pointer,
            IrType::I32 => KernelParamType::Int32,
            IrType::I64 => KernelParamType::Int64,
            IrType::F32 => KernelParamType::Float32,
            IrType::F64 => KernelParamType::Float64,
            IrType::Struct(_name) => {
                KernelParamType::Struct(8) // Struct name-based, assume pointer-sized
            }
            _ => KernelParamType::Int64,
        }
    }
    
    fn type_size(&self, ty: &IrType) -> usize {
        match ty {
            IrType::I8 => 1,
            IrType::I16 => 2,
            IrType::I32 | IrType::F32 => 4,
            IrType::I64 | IrType::F64 | IrType::Ptr(_) => 8,
            IrType::Bool => 1,
            IrType::Array(inner, count) => self.type_size(inner) * count,
            IrType::Struct(_) => 8, // Struct is name-based, assume pointer-sized
            _ => 8,
        }
    }
    
    fn ir_type_to_ptx(&self, ty: &IrType) -> &'static str {
        match ty {
            IrType::I8 => ".b8",
            IrType::I16 => ".b16",
            IrType::I32 => ".b32",
            IrType::I64 => ".b64",
            IrType::F32 => ".f32",
            IrType::F64 => ".f64",
            IrType::Ptr(_) => ".b64",
            IrType::Bool => ".b8",
            _ => ".b64",
        }
    }
    
    fn value_to_ptx_reg(&self, value: &crate::ir::IrValue) -> String {
        match value {
            crate::ir::IrValue::Var(name) => format!("%rd{}", name),
            crate::ir::IrValue::Const(c) => match c {
                crate::ir::IrConst::Int(v) => format!("{}", v),
                crate::ir::IrConst::Float(v) => format!("{:.6}", v),
                crate::ir::IrConst::Bool(v) => format!("{}", if *v { 1 } else { 0 }),
                _ => "0".to_string(),
            },
        }
    }

    fn spirv_operand(&self, value: &IrValue) -> String {
        match value {
            IrValue::Var(name) => format!("%{}", name),
            IrValue::Const(IrConst::Int(v)) => v.to_string(),
            IrValue::Const(IrConst::Float(v)) => format!("{:.6}", v),
            IrValue::Const(IrConst::Bool(v)) => v.to_string(),
            _ => "0".to_string(),
        }
    }

    fn spirv_dest(&self, name: &str) -> String {
        format!("%{}", name)
    }

    fn spirv_type_str(&self, ty: &IrType) -> &'static str {
        match ty {
            IrType::F32 => "%f32",
            IrType::F64 => "%f64",
            IrType::Bool => "%bool",
            _ => "%i64",
        }
    }

    fn spirv_binop_str(&self, op: &IrBinOp) -> &'static str {
        match op {
            IrBinOp::Add => "OpIAdd",
            IrBinOp::Sub => "OpISub",
            IrBinOp::Mul => "OpIMul",
            IrBinOp::Div => "OpSDiv",
            IrBinOp::Mod => "OpSRem",
            IrBinOp::Eq => "OpIEqual",
            IrBinOp::Ne => "OpINotEqual",
            IrBinOp::Lt => "OpSLessThan",
            IrBinOp::Le => "OpSLessThanEqual",
            IrBinOp::Gt => "OpSGreaterThan",
            IrBinOp::Ge => "OpSGreaterThanEqual",
            _ => "OpIAdd",
        }
    }

    fn spirv_is_comparison(&self, op: &IrBinOp) -> bool {
        matches!(op, IrBinOp::Eq | IrBinOp::Ne | IrBinOp::Lt | IrBinOp::Le | IrBinOp::Gt | IrBinOp::Ge)
    }

    fn metal_param_type(&self, ty: &IrType) -> &'static str {
        match ty {
            IrType::I32 => "int",
            IrType::I64 => "long",
            IrType::F32 => "float",
            IrType::F64 => "double",
            IrType::Ptr(_) => "device float*",
            _ => "int",
        }
    }

    fn metal_binop_symbol(&self, op: &IrBinOp) -> &'static str {
        match op {
            IrBinOp::Add => "+",
            IrBinOp::Sub => "-",
            IrBinOp::Mul => "*",
            IrBinOp::Div => "/",
            IrBinOp::Mod => "%",
            IrBinOp::And => "&",
            IrBinOp::Or => "|",
            _ => "+",
        }
    }

    fn metal_value(&self, value: &IrValue) -> String {
        match value {
            IrValue::Var(name) => name.clone(),
            IrValue::Const(IrConst::Int(v)) => v.to_string(),
            IrValue::Const(IrConst::Float(v)) => format!("{:.6}", v),
            IrValue::Const(IrConst::Bool(v)) => v.to_string(),
            _ => "0".to_string(),
        }
    }
    
    fn estimate_registers(&self, func: &IrFunction) -> u32 {
        // Rough estimate: 2 registers per instruction + params
        let inst_count: usize = func.blocks.iter()
            .map(|b| b.instructions.len())
            .sum();
        ((inst_count * 2 + func.params.len()) as u32).min(255)
    }
    
    fn estimate_shared_mem(&self, func: &IrFunction) -> u32 {
        // Check for shared memory annotations in the IR
        // For now, return 0 (no static shared memory)
        0
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GPU Dispatch Engine (Top-Level)
// ─────────────────────────────────────────────────────────────────────────────

/// Top-level GPU dispatch engine
pub struct GpuDispatcher {
    /// Device manager
    pub device_manager: DeviceManager,
    /// Memory manager
    pub memory_manager: GpuMemoryManager,
    /// Stream manager
    pub stream_manager: StreamManager,
    /// Kernel compiler
    pub kernel_compiler: GpuKernelCompiler,
    /// Compiled kernel cache
    kernel_cache: HashMap<String, GpuKernel>,
    /// Dispatch statistics
    stats: DispatchStats,
}

#[derive(Debug, Default)]
pub struct DispatchStats {
    pub total_launches: u64,
    pub total_threads_launched: u64,
    pub total_memory_transferred: u64,
    pub kernels_compiled: u64,
}

impl GpuDispatcher {
    pub fn new() -> Self {
        let device_manager = DeviceManager::new();
        let backend = device_manager.active_device().backend;
        
        GpuDispatcher {
            device_manager,
            memory_manager: GpuMemoryManager::new(),
            stream_manager: StreamManager::new(),
            kernel_compiler: GpuKernelCompiler::new(backend),
            kernel_cache: HashMap::new(),
            stats: DispatchStats::default(),
        }
    }
    
    /// Compile an IR function to a GPU kernel
    pub fn compile_kernel(&mut self, func: &IrFunction) -> Result<&GpuKernel, String> {
        if !self.kernel_cache.contains_key(&func.name) {
            let kernel = self.kernel_compiler.compile_kernel(func)?;
            self.stats.kernels_compiled += 1;
            self.kernel_cache.insert(func.name.clone(), kernel);
        }
        Ok(self.kernel_cache.get(&func.name).unwrap())
    }
    
    /// Launch a kernel with the given configuration
    pub fn launch(
        &mut self,
        kernel_name: &str,
        config: &LaunchConfig,
        args: &[u64], // allocation IDs or scalar values
    ) -> Result<(), String> {
        // Validate configuration
        let device = self.device_manager.active_device().clone();
        config.validate(&device)?;
        
        // Look up kernel
        let kernel = self.kernel_cache.get(kernel_name)
            .ok_or_else(|| format!("Kernel '{}' not found in cache", kernel_name))?;
        
        info!("GPU Dispatch: Launching {} with grid=[{}x{}x{}] block=[{}x{}x{}] on {}",
              kernel_name,
              config.grid[0], config.grid[1], config.grid[2],
              config.block[0], config.block[1], config.block[2],
              device.name);
        
        self.stats.total_launches += 1;
        self.stats.total_threads_launched += config.total_threads();
        
        // In production: dispatch to the actual GPU API
        // cudaLaunchKernel / clEnqueueNDRangeKernel / vkCmdDispatch
        
        Ok(())
    }
    
    /// Convenience: compile + auto-configure + launch
    pub fn dispatch(
        &mut self,
        func: &IrFunction,
        total_elements: u32,
        args: &[u64],
    ) -> Result<(), String> {
        self.compile_kernel(func)?;
        
        let device = self.device_manager.active_device().clone();
        let config = LaunchConfig::auto_tune(total_elements, &device);
        
        self.launch(&func.name, &config, args)
    }
    
    /// Multi-GPU dispatch: split work across multiple devices
    pub fn dispatch_multi_gpu(
        &mut self,
        func: &IrFunction,
        total_elements: u32,
        args: &[u64],
    ) -> Result<(), String> {
        let num_devices = self.device_manager.devices().len();
        if num_devices <= 1 {
            return self.dispatch(func, total_elements, args);
        }
        
        let elements_per_device = (total_elements + num_devices as u32 - 1) / num_devices as u32;
        
        for dev_idx in 0..num_devices {
            self.device_manager.select_device(dev_idx)?;
            
            let start = dev_idx as u32 * elements_per_device;
            let count = elements_per_device.min(total_elements - start);
            
            if count > 0 {
                info!("GPU Multi-Dispatch: Device {} processing elements {}..{}",
                      dev_idx, start, start + count);
                self.dispatch(func, count, args)?;
            }
        }
        
        Ok(())
    }
    
    /// Get dispatch statistics
    pub fn stats(&self) -> &DispatchStats {
        &self.stats
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_launch_config_1d() {
        let config = LaunchConfig::dim1d(1000, 256);
        assert_eq!(config.grid[0], 4); // ceil(1000/256)
        assert_eq!(config.block[0], 256);
        assert_eq!(config.total_threads(), 4 * 256);
    }

    #[test]
    fn test_launch_config_2d() {
        let config = LaunchConfig::dim2d(1920, 1080, 16, 16);
        assert_eq!(config.grid[0], 120); // ceil(1920/16)
        assert_eq!(config.grid[1], 68);  // ceil(1080/16)
    }

    #[test]
    fn test_launch_config_validation() {
        let device = GpuDevice::software_fallback();
        let config = LaunchConfig::dim1d(100, 64);
        assert!(config.validate(&device).is_ok());
        
        let bad_config = LaunchConfig {
            grid: [1, 1, 1],
            block: [2048, 1, 1], // Exceeds max
            shared_mem_bytes: 0,
            stream: None,
        };
        assert!(bad_config.validate(&device).is_err());
    }

    #[test]
    fn test_gpu_memory_alloc_free() {
        let mut mem = GpuMemoryManager::new();
        let device = GpuDevice::software_fallback();
        
        let id = mem.alloc(&device, 1024, GpuMemoryType::Device).unwrap();
        assert!(id > 0);
        
        let stats = mem.stats(device.id);
        assert_eq!(stats.allocated, 1024);
        assert_eq!(stats.active_allocations, 1);
        
        mem.free(id).unwrap();
        let stats = mem.stats(device.id);
        assert_eq!(stats.allocated, 0);
    }

    #[test]
    fn test_gpu_memory_pool_reuse() {
        let mut mem = GpuMemoryManager::new();
        let device = GpuDevice::software_fallback();
        
        let id1 = mem.alloc(&device, 512, GpuMemoryType::Device).unwrap();
        mem.free(id1).unwrap();
        
        // Allocating same size should reuse
        let id2 = mem.alloc(&device, 512, GpuMemoryType::Device).unwrap();
        assert_eq!(id1, id2); // Should reuse the same allocation
    }

    #[test]
    fn test_gpu_memory_double_free() {
        let mut mem = GpuMemoryManager::new();
        let device = GpuDevice::software_fallback();
        
        let id = mem.alloc(&device, 256, GpuMemoryType::Device).unwrap();
        mem.free(id).unwrap();
        assert!(mem.free(id).is_err()); // Double free should error
    }

    #[test]
    fn test_stream_management() {
        let mut sm = StreamManager::new();
        
        let s1 = sm.create_stream(0, 0);
        let s2 = sm.create_stream(0, -1);
        assert_ne!(s1.id, s2.id);
        
        let event = sm.record_event(s1.id);
        assert!(!event.signaled);
        
        assert!(sm.wait_event(s2.id, event.id).is_ok());
        assert!(sm.synchronize_stream(s1.id).is_ok());
    }

    #[test]
    fn test_device_manager() {
        let manager = DeviceManager::new();
        assert!(!manager.devices().is_empty()); // At least software fallback
    }

    #[test]
    fn test_auto_tune_launch() {
        let device = GpuDevice::software_fallback();
        let config = LaunchConfig::auto_tune(10000, &device);
        assert!(config.validate(&device).is_ok());
        assert!(config.total_threads() >= 10000);
    }
}
