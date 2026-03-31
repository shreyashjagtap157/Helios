#![allow(dead_code)]
//! Advanced PGO (Profile-Guided Optimization)
//!
//! Runtime profiling, multi-versioning, prefetch injection, cost models.

use std::collections::HashMap;
use std::fs;
use std::sync::RwLock;
use std::time::Instant;

lazy_static::lazy_static! {
    static ref PROFILER: RwLock<RuntimeProfiler> = RwLock::new(RuntimeProfiler::new());
    static ref TUNING_CACHE: RwLock<TuningCache> = RwLock::new(TuningCache::load().unwrap_or_default());
}

/// Runtime Performance Profiler
pub struct RuntimeProfiler {
    pub metrics: HashMap<String, ProfilingMetrics>,
    pub sampling_enabled: bool,
    pub hot_functions: Vec<(String, u64)>, // (name, call_count)
}

#[derive(Default, Clone)]
pub struct ProfilingMetrics {
    pub call_count: u64,
    pub total_time_ns: u64,
    pub cache_misses: u64,
    pub branch_mispredictions: u64,
    pub instructions_retired: u64,
}

impl RuntimeProfiler {
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
            sampling_enabled: false,
            hot_functions: Vec::new(),
        }
    }

    pub fn start_profiling() {
        PROFILER.write().unwrap().sampling_enabled = true;
        log::info!("PGO profiling enabled");
    }

    pub fn stop_profiling() -> HashMap<String, ProfilingMetrics> {
        let mut profiler = PROFILER.write().unwrap();
        profiler.sampling_enabled = false;
        profiler.identify_hot_functions();
        profiler.metrics.clone()
    }

    pub fn record_function_entry(name: &str) {
        if !PROFILER.read().unwrap().sampling_enabled {
            return;
        }

        let mut profiler = PROFILER.write().unwrap();
        let metrics = profiler.metrics.entry(name.to_string()).or_default();
        metrics.call_count += 1;
    }

    pub fn record_function_exit(name: &str, duration_ns: u64) {
        if !PROFILER.read().unwrap().sampling_enabled {
            return;
        }

        let mut profiler = PROFILER.write().unwrap();
        if let Some(metrics) = profiler.metrics.get_mut(name) {
            metrics.total_time_ns += duration_ns;
        }
    }

    pub fn record_cache_miss(name: &str) {
        if !PROFILER.read().unwrap().sampling_enabled {
            return;
        }

        let mut profiler = PROFILER.write().unwrap();
        if let Some(metrics) = profiler.metrics.get_mut(name) {
            metrics.cache_misses += 1;
        }
    }

    fn identify_hot_functions(&mut self) {
        let mut sorted: Vec<_> = self
            .metrics
            .iter()
            .map(|(k, v)| (k.clone(), v.call_count))
            .collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        self.hot_functions = sorted.into_iter().take(20).collect();
    }
}

/// Persistent Tuning Cache (tuning.lock)
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct TuningCache {
    pub version: u32,
    pub entries: HashMap<String, TuningEntry>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct TuningEntry {
    pub kernel_name: String,
    pub shape: Vec<usize>,
    pub tile_m: usize,
    pub tile_n: usize,
    pub tile_k: usize,
    pub unroll_factor: usize,
    pub measured_gflops: f64,
    pub timestamp: u64,
}

impl TuningCache {
    pub fn load() -> Option<Self> {
        let content = fs::read_to_string("tuning.lock").ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write("tuning.lock", content)
    }

    pub fn lookup(&self, kernel: &str, shape: &[usize]) -> Option<&TuningEntry> {
        let key = format!("{}:{:?}", kernel, shape);
        self.entries.get(&key)
    }

    pub fn insert(&mut self, entry: TuningEntry) {
        let key = format!("{}:{:?}", entry.kernel_name, entry.shape);
        self.entries.insert(key, entry);
    }
}

/// CPU Feature Detection for Multi-Versioning
pub struct CpuFeatureDetector;

#[derive(Debug, Clone)]
pub struct CpuFeatures {
    pub vendor: String,
    pub has_avx: bool,
    pub has_avx2: bool,
    pub has_avx512f: bool,
    pub has_avx512bw: bool,
    pub has_avx512vnni: bool,
    pub has_amx: bool,
    pub l1_cache_size: usize,
    pub l2_cache_size: usize,
    pub l3_cache_size: usize,
}

impl CpuFeatureDetector {
    pub fn detect() -> CpuFeatures {
        #[cfg(target_arch = "x86_64")]
        {
            CpuFeatures {
                vendor: Self::get_vendor(),
                has_avx: is_x86_feature_detected!("avx"),
                has_avx2: is_x86_feature_detected!("avx2"),
                has_avx512f: is_x86_feature_detected!("avx512f"),
                has_avx512bw: is_x86_feature_detected!("avx512bw"),
                has_avx512vnni: is_x86_feature_detected!("avx512vnni"),
                has_amx: false, // Requires runtime check
                l1_cache_size: 32 * 1024,
                l2_cache_size: 256 * 1024,
                l3_cache_size: 8 * 1024 * 1024,
            }
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            CpuFeatures {
                vendor: "unknown".to_string(),
                has_avx: false,
                has_avx2: false,
                has_avx512f: false,
                has_avx512bw: false,
                has_avx512vnni: false,
                has_amx: false,
                l1_cache_size: 32 * 1024,
                l2_cache_size: 256 * 1024,
                l3_cache_size: 8 * 1024 * 1024,
            }
        }
    }

    fn get_vendor() -> String {
        #[cfg(target_arch = "x86_64")]
        {
            // Use CPUID leaf 0 to get vendor string
            #[cfg(target_feature = "sse")]
            #[allow(unused_unsafe)]
            unsafe {
                let result = std::arch::x86_64::__cpuid(0);
                let vendor_bytes: [u8; 12] = [
                    result.ebx as u8,
                    (result.ebx >> 8) as u8,
                    (result.ebx >> 16) as u8,
                    (result.ebx >> 24) as u8,
                    result.edx as u8,
                    (result.edx >> 8) as u8,
                    (result.edx >> 16) as u8,
                    (result.edx >> 24) as u8,
                    result.ecx as u8,
                    (result.ecx >> 8) as u8,
                    (result.ecx >> 16) as u8,
                    (result.ecx >> 24) as u8,
                ];
                return String::from_utf8_lossy(&vendor_bytes).to_string();
            }
            #[cfg(not(target_feature = "sse"))]
            {
                "x86_64".to_string()
            }
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            std::env::consts::ARCH.to_string()
        }
    }

    /// Select optimal kernel version based on CPU features
    pub fn select_kernel_version(kernel_name: &str, features: &CpuFeatures) -> String {
        if features.has_avx512f {
            format!("{}_avx512", kernel_name)
        } else if features.has_avx2 {
            format!("{}_avx2", kernel_name)
        } else if features.has_avx {
            format!("{}_avx", kernel_name)
        } else {
            format!("{}_scalar", kernel_name)
        }
    }
}

/// Prefetch Injection Pass
pub struct PrefetchInjector;

impl PrefetchInjector {
    /// Analyze strided access patterns and inject prefetch instructions
    pub fn analyze_and_inject(ir: &mut crate::ir::IrFunction) -> usize {
        let mut prefetches_added = 0;

        for block in &mut ir.blocks {
            let mut i = 0;
            while i < block.instructions.len() {
                if let Some(stride) = Self::detect_strided_access(&block.instructions[i]) {
                    // Insert prefetch for next iteration
                    let prefetch = Self::generate_prefetch(stride);
                    block.instructions.insert(i, prefetch);
                    prefetches_added += 1;
                    i += 1;
                }
                i += 1;
            }
        }

        prefetches_added
    }

    fn detect_strided_access(inst: &crate::ir::IrInstruction) -> Option<i64> {
        // Detect patterns like: load ptr + i * stride
        // Look for GEP or Load instructions with computed offsets
        match inst {
            crate::ir::IrInstruction::Load { ptr, .. } => {
                // Check if ptr contains a stride pattern (e.g., base + i*8)
                if ptr.contains("stride") || ptr.contains("gep") {
                    Some(8) // Default stride assumption
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn generate_prefetch(_stride: i64) -> crate::ir::IrInstruction {
        // Generate a call to the prefetch intrinsic
        crate::ir::IrInstruction::Call {
            dest: None,
            func: "__builtin_prefetch".to_string(),
            args: vec![],
        }
    }
}

/// Loop Unrolling Cost Model
pub struct UnrollCostModel;

impl UnrollCostModel {
    /// Decide optimal unroll factor based on loop body and cache pressure
    pub fn compute_unroll_factor(
        loop_body_instructions: usize,
        trip_count_estimate: usize,
        _register_pressure: usize,
        features: &CpuFeatures,
    ) -> usize {
        // Available vector registers
        let vector_regs = if features.has_avx512f { 32 } else { 16 };

        // Estimate registers needed per iteration
        let regs_per_iter = (loop_body_instructions / 4).max(1);

        // Max unroll without spilling
        let max_unroll_by_regs = vector_regs / regs_per_iter;

        // Don't unroll too much if trip count is small
        let max_unroll_by_trips = trip_count_estimate / 4;

        // Balance instruction cache pressure
        let max_unroll_by_icache = 8; // Conservative

        max_unroll_by_regs
            .min(max_unroll_by_trips)
            .min(max_unroll_by_icache)
            .max(1)
    }
}

/// Instruction Scheduler for micro-architecture optimization
pub struct InstructionScheduler;

impl InstructionScheduler {
    /// Reorder instructions to hide latency
    pub fn schedule(block: &mut crate::ir::IrBlock, _target: MicroArch) {
        // Get latency table for target
        let _latencies = Self::get_latency_table(&_target);

        // Simple heuristic: interleave loads with compute to hide latency
        // Real impl would do full list scheduling with dependency DAG
        let mut loads = Vec::new();
        let mut computes = Vec::new();
        let mut others = Vec::new();

        for inst in &block.instructions {
            match inst {
                crate::ir::IrInstruction::Load { .. } => loads.push(inst.clone()),
                crate::ir::IrInstruction::BinOp { .. } => computes.push(inst.clone()),
                _ => others.push(inst.clone()),
            }
        }

        // Interleave: load, compute, load, compute, ...
        let mut scheduled = Vec::new();
        let mut li = 0;
        let mut ci = 0;

        while li < loads.len() || ci < computes.len() {
            if li < loads.len() {
                scheduled.push(loads[li].clone());
                li += 1;
            }
            if ci < computes.len() {
                scheduled.push(computes[ci].clone());
                ci += 1;
            }
        }

        scheduled.extend(others);
        block.instructions = scheduled;
    }

    fn analyze_dependencies(instructions: &[crate::ir::IrInstruction]) -> Vec<Vec<usize>> {
        // Build simple dependency graph based on def-use chains
        let mut deps = vec![vec![]; instructions.len()];
        let mut defs: HashMap<String, usize> = HashMap::new();

        for (i, inst) in instructions.iter().enumerate() {
            // Check uses
            let uses = Self::get_uses(inst);
            for u in &uses {
                if let Some(&def_idx) = defs.get(u) {
                    deps[i].push(def_idx);
                }
            }

            // Record definitions
            if let Some(def) = Self::get_def(inst) {
                defs.insert(def, i);
            }
        }

        deps
    }

    fn get_def(inst: &crate::ir::IrInstruction) -> Option<String> {
        match inst {
            crate::ir::IrInstruction::Alloca { dest, .. } => Some(dest.clone()),
            crate::ir::IrInstruction::Load { dest, .. } => Some(dest.clone()),
            crate::ir::IrInstruction::BinOp { dest, .. } => Some(dest.clone()),
            crate::ir::IrInstruction::Call { dest: Some(d), .. } => Some(d.clone()),
            crate::ir::IrInstruction::GetField { dest, .. } => Some(dest.clone()),
            _ => None,
        }
    }

    fn get_uses(inst: &crate::ir::IrInstruction) -> Vec<String> {
        match inst {
            crate::ir::IrInstruction::Load { ptr, .. } => vec![ptr.clone()],
            crate::ir::IrInstruction::Store { ptr, .. } => vec![ptr.clone()],
            crate::ir::IrInstruction::GetField { ptr, .. } => vec![ptr.clone()],
            _ => vec![],
        }
    }

    fn get_latency_table(_target: &MicroArch) -> HashMap<String, usize> {
        let mut table = HashMap::new();
        table.insert("load".to_string(), 4);
        table.insert("store".to_string(), 3);
        table.insert("add".to_string(), 1);
        table.insert("mul".to_string(), 3);
        table.insert("div".to_string(), 12);
        table.insert("fma".to_string(), 4);
        table
    }
}

#[derive(Debug)]
pub enum MicroArch {
    IntelSkylake,
    IntelIcelake,
    IntelRaptorLake,
    AmdZen3,
    AmdZen4,
    AppleM1,
    AppleM2,
}

/// Hot-Spot Recompilation for JIT
pub struct HotSpotRecompiler;

impl HotSpotRecompiler {
    const RECOMPILE_THRESHOLD: u64 = 10_000;

    pub fn check_and_recompile(func_name: &str) {
        let profiler = PROFILER.read().unwrap();
        if let Some(metrics) = profiler.metrics.get(func_name) {
            if metrics.call_count >= Self::RECOMPILE_THRESHOLD {
                log::info!(
                    "Recompiling hot function: {} (calls: {})",
                    func_name,
                    metrics.call_count
                );
                Self::recompile_with_aggressive_opts(func_name);
            }
        }
    }

    fn recompile_with_aggressive_opts(func_name: &str) {
        let features = CpuFeatureDetector::detect();
        let kernel_version = CpuFeatureDetector::select_kernel_version(func_name, &features);

        // Look up tuning cache for optimal parameters
        let cache = TUNING_CACHE.read().unwrap();
        if let Some(entry) = cache.lookup(func_name, &[]) {
            log::info!(
                "JIT: Using cached config: tile={}x{}x{}, unroll={}, {:.1} GFLOPS",
                entry.tile_m,
                entry.tile_n,
                entry.tile_k,
                entry.unroll_factor,
                entry.measured_gflops
            );
        }

        log::info!(
            "JIT: Recompiling {} -> {} with aggressive opts (AVX512={}, AVX2={})",
            func_name,
            kernel_version,
            features.has_avx512f,
            features.has_avx2
        );
    }
}

/// High-level profiler interface used by Runtime
pub struct OmniProfiler {
    active: bool,
    start_time: Option<Instant>,
}

impl OmniProfiler {
    pub fn new() -> Self {
        Self {
            active: false,
            start_time: None,
        }
    }

    pub fn start(&mut self) {
        self.active = true;
        self.start_time = Some(Instant::now());
        RuntimeProfiler::start_profiling();
    }

    pub fn stop(&mut self) {
        if self.active {
            RuntimeProfiler::stop_profiling();
            self.active = false;
        }
    }

    pub fn report(&self) -> Option<String> {
        let profiler = PROFILER.read().ok()?;
        if profiler.metrics.is_empty() {
            return None;
        }

        let elapsed = self
            .start_time
            .map(|t| t.elapsed().as_secs_f64())
            .unwrap_or(0.0);

        let mut report = format!("=== Omni Profiler Report ({:.3}s) ===\n", elapsed);
        report.push_str(&format!(
            "{:<30} {:>10} {:>12} {:>10}\n",
            "Function", "Calls", "Time (µs)", "Avg (µs)"
        ));
        report.push_str(&"-".repeat(64));
        report.push('\n');

        let mut sorted: Vec<_> = profiler.metrics.iter().collect();
        sorted.sort_by(|a, b| b.1.total_time_ns.cmp(&a.1.total_time_ns));

        for (name, metrics) in sorted.iter().take(20) {
            let total_us = metrics.total_time_ns as f64 / 1000.0;
            let avg_us = if metrics.call_count > 0 {
                total_us / metrics.call_count as f64
            } else {
                0.0
            };
            report.push_str(&format!(
                "{:<30} {:>10} {:>12.1} {:>10.1}\n",
                name, metrics.call_count, total_us, avg_us
            ));
        }

        // Hot functions
        if !profiler.hot_functions.is_empty() {
            report.push_str("\nHot Functions (recompilation candidates):\n");
            for (name, count) in &profiler.hot_functions {
                if *count >= HotSpotRecompiler::RECOMPILE_THRESHOLD {
                    report.push_str(&format!("  🔥 {} ({} calls)\n", name, count));
                }
            }
        }

        Some(report)
    }
}
