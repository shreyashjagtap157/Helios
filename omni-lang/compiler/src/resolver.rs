//! Resolver Engines for Omni
//!
//! Implements the three resolver engines from the master canvas:
//! - Execution Strategy Resolver (ESR): chooses AOT/JIT/Interp/VM
//! - Memory Strategy Resolver (MSR): chooses GC/ownership/manual/region
//! - Concurrency Strategy Resolver (CSR): chooses threads/async/channels
//!
//! All resolver decisions are:
//! - Logged as machine-readable JSON
//! - Overrideable by annotations
//! - Deterministic under --deterministic flag

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::modes::{Feature, ModuleMode};

/// Execution strategy chosen by the ESR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecMode {
    /// Ahead-of-time static compilation
    AotStatic,
    /// JIT with tracing
    JitTracing,
    /// Tiered compilation (starts interpreter, hot paths JIT)
    Tiered,
    /// Bytecode VM interpretation
    BytecodeVm,
    /// Tree-walking interpreter
    Interpreter,
}

impl std::fmt::Display for ExecMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecMode::AotStatic => write!(f, "AOT_STATIC"),
            ExecMode::JitTracing => write!(f, "JIT_TRACING"),
            ExecMode::Tiered => write!(f, "TIERED"),
            ExecMode::BytecodeVm => write!(f, "BYTECODE_VM"),
            ExecMode::Interpreter => write!(f, "INTERPRETER"),
        }
    }
}

/// Memory strategy chosen by the MSR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryStrategy {
    /// Garbage collected (tracing GC)
    Gc,
    /// Ownership/borrow checked (Rust-style)
    Ownership,
    /// Manual allocation (C-style)
    Manual,
    /// Region/arena allocator
    Region,
    /// Reference counting
    RefCounted,
}

impl std::fmt::Display for MemoryStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryStrategy::Gc => write!(f, "GC"),
            MemoryStrategy::Ownership => write!(f, "OWNERSHIP"),
            MemoryStrategy::Manual => write!(f, "MANUAL"),
            MemoryStrategy::Region => write!(f, "REGION"),
            MemoryStrategy::RefCounted => write!(f, "REF_COUNTED"),
        }
    }
}

/// Concurrency strategy chosen by the CSR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConcurrencyStrategy {
    /// OS preemptive threads
    OsThreads,
    /// Async/await with event loop
    Async,
    /// CSP channels
    Channels,
    /// Cooperative fibers
    Cooperative,
    /// No concurrency (single-threaded)
    SingleThreaded,
}

impl std::fmt::Display for ConcurrencyStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConcurrencyStrategy::OsThreads => write!(f, "OS_THREADS"),
            ConcurrencyStrategy::Async => write!(f, "ASYNC"),
            ConcurrencyStrategy::Channels => write!(f, "CHANNELS"),
            ConcurrencyStrategy::Cooperative => write!(f, "COOPERATIVE"),
            ConcurrencyStrategy::SingleThreaded => write!(f, "SINGLE_THREADED"),
        }
    }
}

/// Annotations that override resolver decisions.
#[derive(Debug, Clone, Default)]
pub struct Annotations {
    pub aot: bool,
    pub jit: bool,
    pub no_gc: bool,
    pub bare_metal: bool,
    pub ownership: bool,
    pub profile: Option<String>,
}

/// Context provided to resolvers for decision making.
#[derive(Debug, Clone)]
pub struct ResolverContext {
    pub module_name: String,
    pub mode: ModuleMode,
    pub annotations: Annotations,
    pub target_triple: String,
    pub opt_level: u8,
    pub deterministic: bool,
    pub hotness_data: Option<HotnessData>,
}

/// Hotness data from profiling (used by ESR heuristics).
#[derive(Debug, Clone)]
pub struct HotnessData {
    pub hot_functions: Vec<String>,
    pub hot_loops: Vec<String>,
}

/// A single step in the resolver decision path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionStep {
    pub rule: String,
    pub result: String,
}

/// Complete resolver log entry for a single module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolverLog {
    pub module: String,
    pub profile: String,
    pub annotations: Vec<String>,
    pub decision_path: Vec<DecisionStep>,
    pub final_decision: ResolverDecision,
}

/// The final decisions from all three resolvers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolverDecision {
    pub execution: String,
    pub memory: String,
    pub concurrency: String,
}

// ─── Execution Strategy Resolver (ESR) ─────────────────────────────────────

pub struct ExecutionStrategyResolver;

impl ExecutionStrategyResolver {
    pub fn resolve(ctx: &ResolverContext) -> (ExecMode, Vec<DecisionStep>) {
        let mut steps = Vec::new();

        // Annotation overrides take priority
        if ctx.annotations.aot {
            steps.push(DecisionStep {
                rule: "annotation_aot".to_string(),
                result: "AOT_STATIC".to_string(),
            });
            return (ExecMode::AotStatic, steps);
        }

        if ctx.annotations.jit {
            steps.push(DecisionStep {
                rule: "annotation_jit".to_string(),
                result: "JIT_TRACING".to_string(),
            });
            return (ExecMode::JitTracing, steps);
        }

        // Deterministic flag: use profile defaults, no heuristics
        if ctx.deterministic {
            let default = Self::default_for_mode(ctx.mode);
            steps.push(DecisionStep {
                rule: "deterministic_flag".to_string(),
                result: format!("{}", default),
            });
            return (default, steps);
        }

        // Mode-based defaults
        let default = Self::default_for_mode(ctx.mode);
        steps.push(DecisionStep {
            rule: "mode_default".to_string(),
            result: format!("{}", default),
        });

        // Hotness heuristics (only in non-deterministic mode)
        if let Some(ref hotness) = ctx.hotness_data {
            if !hotness.hot_functions.is_empty() && ctx.mode == ModuleMode::Hosted {
                steps.push(DecisionStep {
                    rule: "hotness_heuristic".to_string(),
                    result: "TIERED (hot functions detected)".to_string(),
                });
                return (ExecMode::Tiered, steps);
            }
        }

        (default, steps)
    }

    fn default_for_mode(mode: ModuleMode) -> ExecMode {
        match mode {
            ModuleMode::Script => ExecMode::Interpreter,
            ModuleMode::Hosted => ExecMode::BytecodeVm,
            ModuleMode::BareMetal => ExecMode::AotStatic,
        }
    }
}

// ─── Memory Strategy Resolver (MSR) ────────────────────────────────────────

pub struct MemoryStrategyResolver;

impl MemoryStrategyResolver {
    pub fn resolve(ctx: &ResolverContext) -> (MemoryStrategy, Vec<DecisionStep>) {
        let mut steps = Vec::new();

        // Annotation overrides
        if ctx.annotations.no_gc || ctx.annotations.bare_metal {
            steps.push(DecisionStep {
                rule: "annotation_no_gc".to_string(),
                result: "MANUAL".to_string(),
            });
            return (MemoryStrategy::Manual, steps);
        }

        if ctx.annotations.ownership {
            steps.push(DecisionStep {
                rule: "annotation_ownership".to_string(),
                result: "OWNERSHIP".to_string(),
            });
            return (MemoryStrategy::Ownership, steps);
        }

        // Deterministic flag
        if ctx.deterministic {
            let default = Self::default_for_mode(ctx.mode);
            steps.push(DecisionStep {
                rule: "deterministic_flag".to_string(),
                result: format!("{}", default),
            });
            return (default, steps);
        }

        // Mode defaults
        let default = Self::default_for_mode(ctx.mode);
        steps.push(DecisionStep {
            rule: "mode_default".to_string(),
            result: format!("{}", default),
        });

        (default, steps)
    }

    fn default_for_mode(mode: ModuleMode) -> MemoryStrategy {
        match mode {
            ModuleMode::Script => MemoryStrategy::Gc,
            ModuleMode::Hosted => MemoryStrategy::Gc,
            ModuleMode::BareMetal => MemoryStrategy::Ownership,
        }
    }
}

// ─── Concurrency Strategy Resolver (CSR) ───────────────────────────────────

pub struct ConcurrencyStrategyResolver;

impl ConcurrencyStrategyResolver {
    pub fn resolve(ctx: &ResolverContext) -> (ConcurrencyStrategy, Vec<DecisionStep>) {
        let mut steps = Vec::new();

        // Bare metal always uses cooperative
        if ctx.annotations.bare_metal || ctx.mode == ModuleMode::BareMetal {
            steps.push(DecisionStep {
                rule: "bare_metal_constraint".to_string(),
                result: "COOPERATIVE".to_string(),
            });
            return (ConcurrencyStrategy::Cooperative, steps);
        }

        // Deterministic flag
        if ctx.deterministic {
            let default = Self::default_for_mode(ctx.mode);
            steps.push(DecisionStep {
                rule: "deterministic_flag".to_string(),
                result: format!("{}", default),
            });
            return (default, steps);
        }

        // Mode defaults
        let default = Self::default_for_mode(ctx.mode);
        steps.push(DecisionStep {
            rule: "mode_default".to_string(),
            result: format!("{}", default),
        });

        (default, steps)
    }

    fn default_for_mode(mode: ModuleMode) -> ConcurrencyStrategy {
        match mode {
            ModuleMode::Script => ConcurrencyStrategy::SingleThreaded,
            ModuleMode::Hosted => ConcurrencyStrategy::Async,
            ModuleMode::BareMetal => ConcurrencyStrategy::Cooperative,
        }
    }
}

// ─── Unified Resolver ──────────────────────────────────────────────────────

/// Run all three resolvers and produce a complete log entry.
pub fn resolve_all(ctx: &ResolverContext) -> ResolverLog {
    let (exec, mut exec_steps) = ExecutionStrategyResolver::resolve(ctx);
    let (mem, mut mem_steps) = MemoryStrategyResolver::resolve(ctx);
    let (conc, mut conc_steps) = ConcurrencyStrategyResolver::resolve(ctx);

    let mut decision_path = Vec::new();
    decision_path.append(&mut exec_steps);
    decision_path.append(&mut mem_steps);
    decision_path.append(&mut conc_steps);

    let annotations: Vec<String> = Vec::new();
    let annotations = {
        let mut a = annotations;
        if ctx.annotations.aot {
            a.push("@aot".to_string());
        }
        if ctx.annotations.jit {
            a.push("@jit".to_string());
        }
        if ctx.annotations.no_gc {
            a.push("@no_gc".to_string());
        }
        if ctx.annotations.bare_metal {
            a.push("@bare_metal".to_string());
        }
        if ctx.annotations.ownership {
            a.push("@ownership".to_string());
        }
        a
    };

    ResolverLog {
        module: ctx.module_name.clone(),
        profile: ctx.mode.to_string(),
        annotations,
        decision_path,
        final_decision: ResolverDecision {
            execution: exec.to_string(),
            memory: mem.to_string(),
            concurrency: conc.to_string(),
        },
    }
}

/// Write resolver log to a `.resolver.json` file alongside artifacts.
pub fn write_resolver_log(log: &ResolverLog, output_dir: &Path) -> Result<(), String> {
    let filename = format!("{}.resolver.json", log.module.replace("::", "_"));
    let path = output_dir.join(filename);
    let json = serde_json::to_string_pretty(log)
        .map_err(|e| format!("Cannot serialize resolver log: {}", e))?;
    std::fs::write(&path, json).map_err(|e| format!("Cannot write resolver log: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_ctx(mode: ModuleMode, deterministic: bool) -> ResolverContext {
        ResolverContext {
            module_name: "test_module".to_string(),
            mode,
            annotations: Annotations::default(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            opt_level: 2,
            deterministic,
            hotness_data: None,
        }
    }

    #[test]
    fn test_esr_script_mode() {
        let ctx = test_ctx(ModuleMode::Script, false);
        let (mode, _) = ExecutionStrategyResolver::resolve(&ctx);
        assert_eq!(mode, ExecMode::Interpreter);
    }

    #[test]
    fn test_esr_hosted_mode() {
        let ctx = test_ctx(ModuleMode::Hosted, false);
        let (mode, _) = ExecutionStrategyResolver::resolve(&ctx);
        assert_eq!(mode, ExecMode::BytecodeVm);
    }

    #[test]
    fn test_esr_bare_metal_mode() {
        let ctx = test_ctx(ModuleMode::BareMetal, false);
        let (mode, _) = ExecutionStrategyResolver::resolve(&ctx);
        assert_eq!(mode, ExecMode::AotStatic);
    }

    #[test]
    fn test_esr_aot_override() {
        let mut ctx = test_ctx(ModuleMode::Script, false);
        ctx.annotations.aot = true;
        let (mode, steps) = ExecutionStrategyResolver::resolve(&ctx);
        assert_eq!(mode, ExecMode::AotStatic);
        assert_eq!(steps[0].rule, "annotation_aot");
    }

    #[test]
    fn test_esr_deterministic() {
        let ctx = test_ctx(ModuleMode::Hosted, true);
        let (mode, steps) = ExecutionStrategyResolver::resolve(&ctx);
        assert_eq!(mode, ExecMode::BytecodeVm);
        assert_eq!(steps[0].rule, "deterministic_flag");
    }

    #[test]
    fn test_msr_script_mode() {
        let ctx = test_ctx(ModuleMode::Script, false);
        let (strategy, _) = MemoryStrategyResolver::resolve(&ctx);
        assert_eq!(strategy, MemoryStrategy::Gc);
    }

    #[test]
    fn test_msr_bare_metal_mode() {
        let ctx = test_ctx(ModuleMode::BareMetal, false);
        let (strategy, _) = MemoryStrategyResolver::resolve(&ctx);
        assert_eq!(strategy, MemoryStrategy::Ownership);
    }

    #[test]
    fn test_msr_no_gc_override() {
        let mut ctx = test_ctx(ModuleMode::Hosted, false);
        ctx.annotations.no_gc = true;
        let (strategy, _) = MemoryStrategyResolver::resolve(&ctx);
        assert_eq!(strategy, MemoryStrategy::Manual);
    }

    #[test]
    fn test_csr_hosted_mode() {
        let ctx = test_ctx(ModuleMode::Hosted, false);
        let (strategy, _) = ConcurrencyStrategyResolver::resolve(&ctx);
        assert_eq!(strategy, ConcurrencyStrategy::Async);
    }

    #[test]
    fn test_csr_bare_metal_mode() {
        let ctx = test_ctx(ModuleMode::BareMetal, false);
        let (strategy, _) = ConcurrencyStrategyResolver::resolve(&ctx);
        assert_eq!(strategy, ConcurrencyStrategy::Cooperative);
    }

    #[test]
    fn test_full_resolution() {
        let ctx = test_ctx(ModuleMode::Hosted, false);
        let log = resolve_all(&ctx);
        assert_eq!(log.module, "test_module");
        assert_eq!(log.profile, "hosted");
        assert_eq!(log.final_decision.execution, "BYTECODE_VM");
        assert_eq!(log.final_decision.memory, "GC");
        assert_eq!(log.final_decision.concurrency, "ASYNC");
        assert!(!log.decision_path.is_empty());
    }

    #[test]
    fn test_deterministic_reproducibility() {
        let ctx1 = test_ctx(ModuleMode::Hosted, true);
        let ctx2 = test_ctx(ModuleMode::Hosted, true);
        let log1 = resolve_all(&ctx1);
        let log2 = resolve_all(&ctx2);
        assert_eq!(log1.final_decision.execution, log2.final_decision.execution);
        assert_eq!(log1.final_decision.memory, log2.final_decision.memory);
        assert_eq!(
            log1.final_decision.concurrency,
            log2.final_decision.concurrency
        );
    }

    #[test]
    fn test_hotness_triggers_tiered() {
        let mut ctx = test_ctx(ModuleMode::Hosted, false);
        ctx.hotness_data = Some(HotnessData {
            hot_functions: vec!["hot_func".to_string()],
            hot_loops: vec![],
        });
        let (mode, steps) = ExecutionStrategyResolver::resolve(&ctx);
        assert_eq!(mode, ExecMode::Tiered);
        assert!(steps.iter().any(|s| s.rule == "hotness_heuristic"));
    }
}
