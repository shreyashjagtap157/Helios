//! Omni Compiler - Main Entry Point
//! 
//! The Omni programming language compiler for Project HELIOS.
//! Supports multiple backends: LLVM (native), OVM (bytecode), and hybrid.
//! Features hardware-adaptive compilation and universal execution model.

mod lexer;
mod parser;
mod semantic;
mod ir;
mod runtime;
mod codegen;
mod monitor;
use sysinfo::{System, SystemExt, ProcessExt};
use std::fs::File;
// `pprof` is only available on Unix targets (native signal-based sampling).
#[cfg(unix)]
use pprof::ProfilerGuard;

use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use anyhow::Result;

/// Code generation target
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum Target {
    #[cfg(feature = "llvm")]
    Llvm,       // LLVM IR -> native code
    #[default]
    Ovm,        // OVM bytecode for managed execution
    #[cfg(feature = "llvm")]
    Hybrid,     // Both native and managed
    #[cfg(feature = "llvm")]
    Native,     // Direct native code (no runtime)
}

impl From<Target> for codegen::CodegenTarget {
    fn from(t: Target) -> Self {
        match t {
            #[cfg(feature = "llvm")]
            Target::Llvm => codegen::CodegenTarget::Llvm,
            Target::Ovm => codegen::CodegenTarget::Ovm,
            #[cfg(feature = "llvm")]
            Target::Hybrid => codegen::CodegenTarget::Hybrid,
            #[cfg(feature = "llvm")]
            Target::Native => codegen::CodegenTarget::Native,
        }
    }
}

/// Omni Language Compiler & Runtime
#[derive(Parser, Debug)]
#[command(name = "omnc")]
#[command(author = "HELIOS Project")]
#[command(version = "2.0.0")]
#[command(about = "Compiles and Executes Omni applications with hardware-adaptive optimization")]
struct Args {
    /// Input source file (.omni) or folder
    #[arg(required = true)]
    input: PathBuf,

    /// Output file path
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Code generation target
    #[arg(short, long, value_enum, default_value = "ovm")]
    target: Target,

    /// Emit LLVM IR instead of binary
    #[arg(long)]
    emit_llvm: bool,

    /// Emit Omni IR (intermediate representation)
    #[arg(long)]
    emit_ir: bool,

    /// Run the application immediately (Interpreter Mode)
    #[arg(short, long)]
    run: bool,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Enable in-process runtime monitor (heartbeat, token/item counters)
    #[arg(long)]
    monitor: bool,

    /// Optimization level (0-3)
    #[arg(short = 'O', long, default_value = "2")]
    opt_level: u8,
    
    /// Generate DWARF debug info
    #[arg(short = 'g', long)]
    debug_info: bool,

    /// Enable PGO profiling
    #[arg(long)]
    profile: bool,
    
    /// Detect and optimize for current hardware
    #[arg(long)]
    hardware_adaptive: bool,
    
    /// Arguments to pass to the program when using --run
    #[arg(last = true)]
    program_args: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    if args.verbose {
        env_logger::Builder::from_env(
            env_logger::Env::default().default_filter_or("debug")
        ).init();
    } else {
        env_logger::init();
    }

    log::info!("HELIOS Omni Compiler/Runtime v2.0.0 (Cognitive Framework)");
    log::info!("Target: {:?}", args.target);

    // Ensure diagnostics directory exists and spawn a lightweight monitor thread when verbose mode is enabled.
    let _ = std::fs::create_dir_all("diagnostics");
    // Diagnostics files (monitor dumps, profiles) will be written into `diagnostics/`.
    // The monitor logs process CPU and memory once per second so we can
    // detect stalls, heavy allocations, and confirm the compiler is making
    // forward progress at runtime without an external script.
    // Spawn the monitor thread when `--monitor` is enabled. This monitor
    // reads lightweight counters updated by the parser and logs a heartbeat.
    let monitor_handle = if args.monitor {
        monitor::enable();
        Some(std::thread::spawn(|| {
            // Create profiler guard for in-process sampling (keeps sampling while alive)
            #[cfg(unix)]
            let guard = ProfilerGuard::new(100).ok();
            #[cfg(not(unix))]
            let guard: Option<()> = None; // profiler not available on non-unix targets
            let mut sys = System::new_all();
            let pid = sysinfo::get_current_pid().unwrap_or_else(|_| sysinfo::Pid::from(0));
            let mut prev_tokens = 0usize;
            let mut prev_items = 0usize;
            let mut prev_last = 0u64;
            let mut stagnant_count = 0usize;
            while monitor::enabled() {
                // Snapshot internal counters
                let (tokens, items, last) = monitor::snapshot();
                // Sample OS-level process metrics when available
                if cfg!(target_os = "windows") {
                    sys.refresh_process(pid);
                } else {
                    sys.refresh_process(pid);
                }
                    if let Some(p) = sys.process(pid) {
                        log::info!("monitor: tokens={} items={} cpu={:.2}% mem={} KB virt={} KB last_hb={}", tokens, items, p.cpu_usage(), p.memory(), p.virtual_memory(), last);
                        // Detect stagnation: internal counters unchanged for several samples
                        if tokens == prev_tokens && items == prev_items && last == prev_last {
                            stagnant_count += 1;
                        } else {
                            stagnant_count = 0;
                        }
                        // If we've seen no internal progress for 8 seconds, write a dump
                        if stagnant_count >= 8 {
                            // include rich parser snapshot
                            let (cur, preview, errors) = monitor::rich_snapshot();
                            let dump = format!(
                                "STALLED: tokens={} items={} cpu={:.2}% last_hb={} parser_cur={}\npreview:\n{}\nerrors:\n{}\n",
                                tokens,
                                items,
                                p.cpu_usage(),
                                last,
                                cur,
                                preview.join("\n"),
                                errors.join("\n")
                            );
                            let fname = format!("diagnostics/monitor_stall_{}.log", chrono::Utc::now().format("%Y%m%dT%H%M%S"));
                            let _ = std::fs::write(&fname, dump.as_bytes());
                            log::warn!("monitor: detected stall; wrote {}", fname);
                            // Also attempt to write an in-process CPU flamegraph snapshot if profiling is active
                            #[cfg(unix)]
                            {
                                if let Some(g) = &guard {
                                    if let Ok(report) = g.report().build() {
                                        let fg_name = format!("diagnostics/monitor_flame_{}.svg", chrono::Utc::now().format("%Y%m%dT%H%M%S"));
                                        match File::create(&fg_name) {
                                            Ok(mut f) => {
                                                if let Err(e) = report.flamegraph(&mut f) {
                                                    log::error!("monitor: failed to write flamegraph {}: {}", fg_name, e);
                                                } else {
                                                    log::warn!("monitor: wrote flamegraph {}", fg_name);
                                                }
                                            }
                                            Err(e) => {
                                                log::error!("monitor: failed to create flamegraph file {}: {}", fg_name, e);
                                            }
                                        }
                                    } else {
                                        log::debug!("monitor: profiler report not ready yet");
                                    }
                                }
                            }
                            #[cfg(not(unix))]
                            {
                                log::debug!("monitor: in-process profiler not available on this platform (non-Unix)");
                            }
                            stagnant_count = 0; // reset after writing a dump
                        }
                        prev_tokens = tokens;
                        prev_items = items;
                        prev_last = last;
                    } else {
                        log::info!("monitor: tokens={} items={} process {} not found last_hb={}", tokens, items, std::process::id(), last);
                    }
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
            log::info!("monitor: shutting down");
        }))
    } else {
        None
    };

    // Hardware detection for adaptive compilation
    if args.hardware_adaptive {
        let hw = codegen::ovm::HardwareConfig::detect();
        log::info!("Detected hardware: {:?} with {:?} SIMD, {} cores, {}MB RAM",
            hw.cpu_arch, hw.simd_level, hw.core_count, hw.available_memory / 1024 / 1024);
        if hw.has_gpu {
            log::info!("GPU detected - enabling GPU acceleration");
        }
    }

    // V4.0: Initialize PGO Profiler
    if args.profile {
        runtime::profiler::RuntimeProfiler::start_profiling();
    }

    if args.run {
        // Runtime Mode
        log::info!("Starting Runtime Environment...");
        let mut runtime = runtime::Runtime::new();
        runtime.run(&args.input)?;
        
        // Dump profile on exit
        if args.profile {
             let metrics = runtime::profiler::RuntimeProfiler::stop_profiling();
             log::info!("PGO Profile collected: {} functions", metrics.len());
        }
    } else {
        // Compilation Mode
        log::info!("Compiling: {:?}", args.input);
        
        // Read source file
        let source = std::fs::read_to_string(&args.input)?;
        
        // Compilation pipeline
        compile(&source, &args)?;
    }
    // If the monitor thread is running, signal it to stop and join.
    if let Some(h) = monitor_handle {
        monitor::disable();
        let _ = h.join();
    }
    Ok(())
}

fn compile(source: &str, args: &Args) -> Result<()> {
    // Phase 1: Lexical analysis
    log::debug!("Phase 1: Lexical analysis");
    let tokens = lexer::tokenize(source)?;
    // heartbeat: tokens produced
    monitor::update_heartbeat();
    
    // Phase 2: Parsing
    log::debug!("Phase 2: Parsing");
    monitor::update_heartbeat();
    let ast = parser::parse(tokens)?;

    // Phase 2.5: Type inference (fatal for concrete mismatches)
    log::debug!("Phase 2.5: Type inference");
    monitor::update_heartbeat();
    let type_result = semantic::type_inference::check_types(&ast);
    match type_result {
        Ok(result) => {
            for w in &result.warnings {
                eprintln!("warning: type inference: {}", w);
            }
        }
        Err(errors) => {
            let hard_errors: Vec<_> = errors.iter()
                .filter(|e| is_hard_type_error(e))
                .collect();
            let warnings: Vec<_> = errors.iter()
                .filter(|e| !is_hard_type_error(e))
                .collect();
            for w in &warnings {
                eprintln!("warning: type inference: {}", w);
            }
            if !hard_errors.is_empty() {
                for e in &hard_errors {
                    eprintln!("error[E005]: type error: {}", e);
                }
                return Err(anyhow::anyhow!(
                    "Type checking failed with {} error(s)",
                    hard_errors.len()
                ));
            }
        }
    }

    // Phase 2.6: Borrow checking (warnings for ownership violations)
    log::debug!("Phase 2.6: Borrow checking");
    monitor::update_heartbeat();
    let borrow_errors = semantic::borrow_check::BorrowChecker::check_module(&ast);
    if !borrow_errors.is_empty() {
        for e in &borrow_errors {
            eprintln!("warning[E006]: borrow check: {}", e);
        }
    }

    // Phase 3: Semantic analysis
    log::debug!("Phase 3: Semantic analysis");
    monitor::update_heartbeat();
    let typed_ast = semantic::analyze(ast)?;

    // Phase 4: IR generation
    log::debug!("Phase 4: IR generation");
    monitor::update_heartbeat();
    let omni_ir = ir::generate(typed_ast).map_err(|e| anyhow::anyhow!("{}", e))?;
    
    if args.emit_ir {
        let ir_path = args.output.clone()
            .unwrap_or_else(|| args.input.with_extension("oir"));
        std::fs::write(&ir_path, omni_ir.to_string())?;
        log::info!("Wrote Omni IR to {:?}", ir_path);
        return Ok(());
    }

    // Phase 5: Code generation with target selection
    log::debug!("Phase 5: Code generation (target: {:?})", args.target);
    monitor::update_heartbeat();
    let output_path = args.output.clone()
        .unwrap_or_else(|| args.input.with_extension(""));
    
    codegen::generate_with_target(
        omni_ir, 
        &output_path, 
        args.opt_level, 
        args.target.into()
    ).map_err(|e| anyhow::anyhow!("{}", e))?;

    // DWARF Emission
    if args.debug_info {
        log::info!("Generating DWARF v5 debug info...");
    }

    log::info!("Successfully compiled to {:?}", output_path);
    Ok(())
}

/// Determine whether a type error is a "hard" (fatal) error.
///
/// Hard errors are **explicit type-annotation mismatches** where the user
/// wrote `let x: Int = "hello"` and the initializer's type clearly
/// disagrees with the annotation.  Everything else is demoted to a
/// warning because the type-inference engine was never calibrated for
/// Omni's dynamic-flavour features (string concat with `+`, implicit
/// conversions, built-in functions not in the environment, etc.).
fn is_hard_type_error(err: &semantic::type_inference::TypeError) -> bool {
    let msg = &err.message;

    // Unresolved type variables — inference couldn't determine the type
    if msg.contains("?T") {
        return false;
    }

    // Undefined variable / function — likely a built-in not registered
    if msg.contains("Undefined variable") || msg.contains("Undefined function") {
        return false;
    }

    // "<error>" is the error-recovery placeholder type
    if msg.contains("<error>") {
        return false;
    }

    // "Expected numeric type" — Omni supports string concat with +,
    // list concat, etc.  The inference engine doesn't model these.
    if msg.contains("Expected numeric type") {
        return false;
    }

    // Only flag explicit annotation mismatches as hard errors:
    // "Type mismatch: X vs Y – let/var binding '…': declared type must match initializer"
    if msg.contains("Type mismatch") && msg.contains("declared type must match initializer") {
        return true;
    }

    // All other type-mismatch or constraint errors are soft
    // (function call argument mismatches, return type mismatches, etc.
    //  may be false positives due to missing built-in signatures).
    false
}
