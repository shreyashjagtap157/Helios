// Copyright 2024 Shreyash Jagtap
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Omni Compiler - Main Entry Point
//!
//! The Omni programming language compiler.
//! Supports multiple backends: LLVM (native), OVM (bytecode), and hybrid.
//! Features hardware-adaptive compilation and universal execution model.

#[path = "codegen/mod.rs"]
mod codegen;
#[path = "ir/mod.rs"]
mod ir;
#[path = "lexer/mod.rs"]
mod lexer;
mod modes;
mod monitor;
#[path = "parser/mod.rs"]
mod parser;
mod resolver;
mod runtime;
mod semantic;
// `pprof` is only available on Unix targets (native signal-based sampling).
use anyhow::Result;
use clap::{Parser, ValueEnum};
use serde_json::json;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Default, Clone)]
struct ExternalLinkDirectives {
    link_libs: Vec<String>,
    link_paths: Vec<String>,
}

fn stage_trace_enabled() -> bool {
    std::env::var("OMNI_STAGE_TRACE")
        .map(|v| {
            let t = v.trim().to_ascii_lowercase();
            matches!(t.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(false)
}

fn stage_enter(name: &str) -> Instant {
    if stage_trace_enabled() {
        eprintln!("STAGE_ENTER {}", name);
    }
    monitor::update_heartbeat();
    Instant::now()
}

fn stage_exit(name: &str, started: Instant) {
    if stage_trace_enabled() {
        eprintln!("STAGE_EXIT {} ms={}", name, started.elapsed().as_millis());
    }
    monitor::update_heartbeat();
}

fn import_guard_limit() -> Option<usize> {
    std::env::var("OMNI_IMPORT_MAX")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
}

/// Code generation target
#[derive(Debug, Clone, Copy, ValueEnum, Default, PartialEq)]
pub enum Target {
    #[cfg(feature = "llvm")]
    Llvm, // LLVM IR -> native code
    #[default]
    Ovm, // OVM bytecode for managed execution
    #[cfg(feature = "llvm")]
    Hybrid, // Both native and managed
    #[cfg(feature = "experimental")]
    Native, // Direct native code via built-in codegen (no LLVM required)
}

impl From<Target> for codegen::CodegenTarget {
    fn from(t: Target) -> Self {
        match t {
            #[cfg(feature = "llvm")]
            Target::Llvm => codegen::CodegenTarget::Llvm,
            Target::Ovm => codegen::CodegenTarget::Ovm,
            #[cfg(feature = "llvm")]
            Target::Hybrid => codegen::CodegenTarget::Hybrid,
            #[cfg(feature = "experimental")]
            Target::Native => codegen::CodegenTarget::Native,
        }
    }
}

/// Omni Language Compiler & Runtime
#[derive(Parser, Debug)]
#[command(name = "omnc")]
#[command(author = "Omni Project")]
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

    /// Enable deterministic mode (freeze resolver behavior, no heuristics)
    #[arg(long)]
    deterministic: bool,

    /// Module mode: script, hosted, or bare_metal
    #[arg(long, default_value = "hosted")]
    mode: String,

    /// Resolver log output directory
    #[arg(long)]
    resolver_log: Option<PathBuf>,

    /// Dump parsed AST (before semantic analysis)
    #[arg(long)]
    emit_ast: bool,

    /// Dump lexer tokens to a file and exit
    #[arg(long)]
    emit_tokens: bool,

    /// Maximum parser tick limit to detect runaway parsing (0 = disabled)
    #[arg(long)]
    parser_tick_limit: Option<usize>,

    /// Dump typed AST (after semantic analysis)
    #[arg(long)]
    emit_typed_ast: bool,

    /// Emit diagnostics as JSON lines (partial coverage in compile pipeline)
    #[arg(long)]
    diagnostics_json: bool,

    /// Arguments to pass to the program when using --run
    #[arg(last = true)]
    program_args: Vec<String>,
}

fn emit_diagnostic(args: &Args, level: &str, code: Option<&str>, message: &str) {
    let machine_fix = suggested_machine_fix(code, message);

    if args.diagnostics_json {
        let payload = json!({
            "level": level,
            "code": code,
            "message": message,
            "fix": machine_fix,
        });
        eprintln!("{}", payload);
        return;
    }

    if let Some(code) = code {
        eprintln!("{}[{}]: {}", level, code, message);
    } else {
        eprintln!("{}: {}", level, message);
    }

    if let Some(fix) = machine_fix {
        eprintln!("help: suggested fix: {}", fix);
    }
}

fn suggested_machine_fix(code: Option<&str>, message: &str) -> Option<&'static str> {
    match code {
        Some("E005") => Some("Add an explicit type annotation or insert a cast to align expected and actual types."),
        Some("E006") => Some("Reorder uses to avoid overlapping borrows, or clone/split the value before borrowing."),
        Some("E007") if message.contains("effect:") => {
            Some("Annotate the function with the required effect or propagate the effect in the caller signature.")
        }
        Some("E007") => Some("Use a memory operation compatible with the selected mode/zone, or move the operation into an allowed zone."),
        _ => None,
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    if args.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::init();
    }

    log::info!("Omni Compiler/Runtime v2.0.0");
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
            let _guard: Option<()> = None; // profiler not available on non-unix targets
            let _sys_fake = 1;
            let _pid = std::process::id();
            let mut prev_tokens = 0usize;
            let mut prev_items = 0usize;
            let mut prev_last = 0u64;
            let mut stagnant_count = 0usize;
            while monitor::enabled() {
                // Snapshot internal counters
                let (tokens, items, last) = monitor::snapshot();
                // Sample OS-level process metrics when available
                if cfg!(target_os = "windows") {
                } else {
                }
                if true {
                    log::info!(
                        "monitor: tokens={} items={} cpu={:.2}% mem={} KB virt={} KB last_hb={}",
                        tokens,
                        items,
                        0.0,
                        0,
                        0,
                        last
                    );
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
                                0.0,
                                last,
                                cur,
                                preview.join("\n"),
                                errors.join("\n")
                            );
                        let fname = format!(
                            "diagnostics/monitor_stall_{}.log",
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs()
                        );
                        let _ = std::fs::write(&fname, dump.as_bytes());
                        log::warn!("monitor: detected stall; wrote {}", fname);
                        // Also attempt to write an in-process CPU flamegraph snapshot if profiling is active
                        #[cfg(unix)]
                        { /* pprof removed */ }
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
                    log::info!(
                        "monitor: tokens={} items={} process {} not found last_hb={}",
                        tokens,
                        items,
                        std::process::id(),
                        last
                    );
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
        log::info!(
            "Detected hardware: {:?} with {:?} SIMD, {} cores, {}MB RAM",
            hw.cpu_arch,
            hw.simd_level,
            hw.core_count,
            hw.available_memory / 1024 / 1024
        );
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
    let external_link_directives = parse_external_link_directives();

    // Validate --mode early with a clear error
    let module_mode: modes::ModuleMode = args
        .mode
        .parse()
        .map_err(|e: String| anyhow::anyhow!("{}", e))?;

    // Run module checker
    let _checker = modes::ModuleChecker::new(module_mode);
    // TODO: detect actual feature usage from AST and check against mode

    // Phase 0: Resolver engines
    log::debug!("Phase 0: Resolver engines (mode: {})", module_mode);
    let annotations = resolver::Annotations::default(); // TODO: parse from AST annotations
    let resolver_ctx = resolver::ResolverContext {
        module_name: args
            .input
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        mode: module_mode,
        annotations,
        target_triple: "x86_64-unknown-linux-gnu".to_string(),
        opt_level: args.opt_level,
        deterministic: args.deterministic,
        hotness_data: None,
    };
    let resolver_log = resolver::resolve_all(&resolver_ctx);
    log::info!(
        "Resolver: exec={}, mem={}, conc={}",
        resolver_log.final_decision.execution,
        resolver_log.final_decision.memory,
        resolver_log.final_decision.concurrency
    );

    // Write resolver log if requested
    if let Some(ref log_dir) = args.resolver_log {
        std::fs::create_dir_all(log_dir)?;
        resolver::write_resolver_log(&resolver_log, log_dir)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
    }

    if args.deterministic {
        log::info!("Deterministic mode: resolver decisions frozen to profile defaults");
    }

    // Phase 1: Lexical analysis
    log::debug!("Phase 1: Lexical analysis");
    let lex_t0 = stage_enter("phase1.lex");
    let tokens = lexer::tokenize(source)?;
    stage_exit("phase1.lex", lex_t0);
    // If requested, write tokens to file and exit early
    if args.emit_tokens {
        let out_path = args
            .output
            .clone()
            .unwrap_or_else(|| args.input.with_extension("tokens"));
        if let Some(parent) = out_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let mut s = String::new();
        for tok in &tokens {
            s.push_str(&format!("{:?}\n", tok));
        }
        std::fs::write(&out_path, s.as_bytes())?;
        log::info!("Wrote tokens to {:?}", out_path);
        return Ok(());
    }
    // heartbeat: tokens produced
    monitor::update_heartbeat();

    // Phase 2: Parsing
    log::debug!("Phase 2: Parsing");
    let parse_t0 = stage_enter("phase2.parse");
    let ast = parser::parse(tokens, args.parser_tick_limit)?;
    stage_exit("phase2.parse", parse_t0);

    // Phase 2.0: Import resolution
    log::debug!("Phase 2.0: Import resolution");
    let import_t0 = stage_enter("phase2.import_resolution");
    let ast = resolve_imports(ast, &args.input, args.parser_tick_limit)?;
    stage_exit("phase2.import_resolution", import_t0);

    // Phase 2.0.1: Conditional compilation (#[cfg(...)] filtering)
    log::debug!("Phase 2.0.1: Conditional compilation (cfg filtering)");
    let ast = apply_cfg_attributes(ast, &parse_external_cfg_flags());

    // --emit-ast: dump parsed AST and exit
    if args.emit_ast {
        let ast_path = args
            .output
            .clone()
            .unwrap_or_else(|| args.input.with_extension("ast.txt"));
        let dump = format!("{:#?}", ast);
        if let Some(parent) = ast_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&ast_path, &dump)?;
        log::info!("Wrote parsed AST to {:?}", ast_path);
        return Ok(());
    }

    // Phase 2.1: Memory zone enforcement
    log::debug!("Phase 2.1: Memory zone enforcement (mode: {})", module_mode);
    let mut mz_checker = modes::MemoryZoneChecker::new(module_mode);
    // Walk AST items to detect memory operations
    for item in &ast.items {
        match item {
            parser::ast::Item::Function(func) => {
                for stmt in &func.body.statements {
                    check_memory_ops_stmt(stmt, &mut mz_checker, &func.name);
                }
            }
            _ => {}
        }
    }

    if !mz_checker.is_valid() {
        for v in mz_checker.validate() {
            emit_diagnostic(args, "warning", Some("E007"), &format!("memory zone: {}", v));
        }
    }

    // Phase 2.5: Type inference (fatal for concrete mismatches)
    log::debug!("Phase 2.5: Type inference");
    let type_t0 = stage_enter("phase2_5.type_inference");
    let type_result = semantic::type_inference::check_types(&ast);
    match type_result {
        Ok(result) => {
            for w in &result.warnings {
                emit_diagnostic(args, "warning", None, &format!("type inference: {}", w));
            }
        }
        Err(errors) => {
            let hard_errors: Vec<_> = errors.iter().filter(|e| is_hard_type_error(e)).collect();
            let warnings: Vec<_> = errors.iter().filter(|e| !is_hard_type_error(e)).collect();
            for w in &warnings {
                emit_diagnostic(args, "warning", None, &format!("type inference: {}", w));
            }
            if !hard_errors.is_empty() {
                for e in &hard_errors {
                    emit_diagnostic(args, "error", Some("E005"), &format!("type error: {}", e));
                }
                return Err(anyhow::anyhow!(
                    "Type checking failed with {} error(s)",
                    hard_errors.len()
                ));
            }
        }
    }
    stage_exit("phase2_5.type_inference", type_t0);

    // Phase 2.6: Borrow checking (errors for ownership violations)
    // Using Polonius algorithm per v2.0 spec for more precise borrow checking
    log::debug!("Phase 2.6: Borrow checking (Polonius)");
    monitor::update_heartbeat();
    let borrow_errors = semantic::polonius::run_polonius(&ast);
    if !borrow_errors.is_empty() {
        for e in &borrow_errors {
            emit_diagnostic(args, "error", Some("E006"), &format!("borrow check: {}", e));
        }
        anyhow::bail!(
            "{} borrow checking error(s) — ownership violations must be fixed",
            borrow_errors.len()
        );
    }

    // Phase 3: Semantic analysis
    log::debug!("Phase 3: Semantic analysis");
    let sem_t0 = stage_enter("phase3.semantic_analysis");
    let typed_ast = semantic::analyze(ast)?;
    stage_exit("phase3.semantic_analysis", sem_t0);

    // Phase 3.5: Effect system validation (Phase 8)
    // Validates effect handlers, effect polymorphism, structured concurrency
    log::debug!("Phase 3.5: Effect system validation");
    monitor::update_heartbeat();
    let effect_result = semantic::phase8_effects::validate_effects(&typed_ast);
    match effect_result {
        Ok(_) => {}
        Err(effects_errors) => {
            for e in &effects_errors {
                emit_diagnostic(args, "error", Some("E007"), &format!("effect: {}", e));
            }
            anyhow::bail!(
                "{} effect error(s) — effects must be handled",
                effects_errors.len()
            );
        }
    }

    // --emit-typed-ast: dump typed AST and exit
    if args.emit_typed_ast {
        let typed_path = args
            .output
            .clone()
            .unwrap_or_else(|| args.input.with_extension("typed.ast.txt"));
        let dump = format!("{:#?}", typed_ast);
        if let Some(parent) = typed_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&typed_path, &dump)?;
        log::info!("Wrote typed AST to {:?}", typed_path);
        return Ok(());
    }

    // Determine output path early (needed by both OVM direct and IR paths)
    let output_path = args
        .output
        .clone()
        .unwrap_or_else(|| args.input.with_extension(""));

    // Phase 4-5: Code generation
    // For OVM target, use direct typed AST → OVM bytecode (bypasses IR)
    if args.target == Target::Ovm && !args.emit_ir {
        log::debug!("Phase 4-5: Direct OVM codegen from typed AST");
        let ovm_t0 = stage_enter("phase4_5.ovm_direct_codegen");
        let out = codegen::ovm_direct::generate_ovm_direct(&typed_ast, &output_path)
            .map_err(|e| anyhow::anyhow!("{}", e));
        stage_exit("phase4_5.ovm_direct_codegen", ovm_t0);
        out?;
        write_link_directives_sidecar(&output_path, &external_link_directives)?;
        return Ok(());
    }

    // Phase 4: IR generation (for LLVM/native targets or --emit-ir)
    log::debug!("Phase 4: IR generation");
    monitor::update_heartbeat();
    let omni_ir = ir::generate(typed_ast).map_err(|e| anyhow::anyhow!("{}", e))?;

    if args.emit_ir {
        let ir_path = args
            .output
            .clone()
            .unwrap_or_else(|| args.input.with_extension("oir"));
        std::fs::write(&ir_path, omni_ir.to_string())?;
        log::info!("Wrote Omni IR to {:?}", ir_path);
        return Ok(());
    }

    // Phase 5: Code generation with target selection
    log::debug!("Phase 5: Code generation (target: {:?})", args.target);
    monitor::update_heartbeat();

    codegen::generate_with_target(omni_ir, &output_path, args.opt_level, args.target.into())
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    write_link_directives_sidecar(&output_path, &external_link_directives)?;

    // DWARF Emission
    if args.debug_info {
        eprintln!(
            "warning: --debug-info / -g is accepted but DWARF generation is not yet implemented"
        );
        log::warn!("DWARF debug info generation not yet implemented; flag has no effect");
    }

    log::info!("Successfully compiled to {:?}", output_path);
    Ok(())
}

/// Resolve import declarations: look up each imported module's `.omni` file
/// relative to the current source file, parse it, and inline its items.
///
/// Import path resolution rules (O-092):
///   `import foo`       → `<dir_of_current_file>/foo.omni`
///   `import std::io`   → `<dir_of_current_file>/std/io.omni`
///   `import foo as f`  → same file lookup, alias is kept on the ImportDecl
///
/// If the file does not exist, a warning is emitted and the import is left
/// as-is (unresolved) so downstream phases can surface the issue.
fn resolve_imports(
    module: parser::ast::Module,
    current_file: &std::path::Path,
    tick_limit: Option<usize>,
) -> Result<parser::ast::Module> {
    use std::collections::HashSet;
    let mut visited = HashSet::new();
    resolve_imports_impl(module, current_file, tick_limit, &mut visited)
}

/// Internal implementation of resolve_imports with circular import detection.
fn resolve_imports_impl(
    module: parser::ast::Module,
    current_file: &std::path::Path,
    tick_limit: Option<usize>,
    visited: &mut std::collections::HashSet<std::path::PathBuf>,
) -> Result<parser::ast::Module> {
    use parser::ast::{ImportDecl, Item, Module};
    use std::collections::HashSet;

    fn normalized_import_paths(path: &[String]) -> Vec<Vec<String>> {
        // `import foo::bar` is parsed as ["foo", "bar"].
        // `import foo::{A, B}` is parsed as ["foo::A", "foo::B"].
        // For symbol-style imports (e.g. std::collections::HashMap), also
        // try the parent module path as a fallback (`std::collections`).
        let mut variants: Vec<Vec<String>> = if path.iter().any(|p| p.contains("::")) {
            path.iter()
                .map(|p| {
                    p.split("::")
                        .filter(|seg| !seg.is_empty())
                        .map(|seg| seg.to_string())
                        .collect::<Vec<String>>()
                })
                .collect()
        } else {
            vec![path.to_vec()]
        };

        let mut with_fallbacks = Vec::new();
        for variant in variants.drain(..) {
            if variant.is_empty() {
                continue;
            }
            with_fallbacks.push(variant.clone());
            if variant.len() > 1 {
                with_fallbacks.push(variant[..variant.len() - 1].to_vec());
            }
        }

        // Keep deterministic order while removing duplicates.
        let mut seen = HashSet::new();
        with_fallbacks
            .into_iter()
            .filter(|v| seen.insert(v.clone()))
            .collect()
    }

    fn detect_project_root(current_file: &std::path::Path) -> Option<std::path::PathBuf> {
        current_file.ancestors().find_map(|ancestor| {
            let has_compiler = ancestor.join("compiler").is_dir();
            let has_omni = ancestor.join("omni").is_dir();
            if has_compiler && has_omni {
                Some(ancestor.to_path_buf())
            } else {
                None
            }
        })
    }

    fn push_omni_candidates(
        candidates: &mut Vec<std::path::PathBuf>,
        omni_root: &std::path::Path,
        rel_parts: &[String],
    ) {
        if rel_parts.is_empty() {
            return;
        }

        let rel = rel_parts.join("/");
        candidates.push(omni_root.join(format!("{}.omni", rel)));
        candidates.push(omni_root.join(rel).join("mod.omni"));
    }

    fn candidate_import_files(
        base_dir: &std::path::Path,
        path_parts: &[String],
        project_root: Option<&std::path::Path>,
    ) -> Vec<std::path::PathBuf> {
        let mut candidates = Vec::new();

        if path_parts.is_empty() {
            return candidates;
        }

        let rel = path_parts.join("/");
        candidates.push(base_dir.join(format!("{}.omni", rel)));
        candidates.push(base_dir.join(&rel).join("mod.omni"));

        if let Some(root) = project_root {
            let omni_root = root.join("omni");
            push_omni_candidates(&mut candidates, &omni_root, path_parts);

            let head = path_parts[0].as_str();
            let tail = &path_parts[1..];

            match head {
                "std" => push_omni_candidates(&mut candidates, &omni_root.join("stdlib"), tail),
                "compiler" => {
                    push_omni_candidates(&mut candidates, &omni_root.join("compiler"), tail)
                }
                "core" => push_omni_candidates(&mut candidates, &omni_root.join("core"), tail),
                "ovm" => push_omni_candidates(&mut candidates, &omni_root.join("ovm"), tail),
                _ => {}
            }
        }

        // Keep order stable while removing duplicates.
        let mut seen = HashSet::new();
        candidates
            .into_iter()
            .filter(|p| seen.insert(p.clone()))
            .collect()
    }

    let base_dir = current_file
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    let project_root = detect_project_root(current_file);

    let mut resolved_items: Vec<Item> = Vec::new();

    for item in module.items {
        match &item {
            Item::Import(ImportDecl { path, alias }) => {
                let path_variants = normalized_import_paths(path);
                let mut had_unresolved = false;
                let mut found_any_candidate_file = false;
                let mut searched_candidates: Vec<std::path::PathBuf> = Vec::new();

                for variant in &path_variants {
                    let candidate_files =
                        candidate_import_files(base_dir, variant, project_root.as_deref());

                    let file_path = candidate_files.iter().find(|p| p.exists()).cloned();

                    if let Some(file_path) = file_path {
                        found_any_candidate_file = true;
                        // Check for circular imports: normalize the path and check if already visited.
                        // We use absolute path resolution to avoid issues with relative paths that
                        // may compare differently despite pointing to the same file.
                        let normalized_path = if file_path.is_absolute() {
                            file_path.clone()
                        } else {
                            match std::env::current_dir() {
                                Ok(cwd) => cwd.join(&file_path),
                                Err(_) => file_path.clone(),
                            }
                        };

                        if visited.contains(&normalized_path) {
                            log::debug!(
                                "Skipping circular import {:?} (already visited)",
                                &file_path
                            );
                            // Don't add the import again; just continue
                            continue;
                        }

                        if let Some(limit) = import_guard_limit() {
                            if visited.len() >= limit {
                                return Err(anyhow::anyhow!(
                                    "import expansion guard tripped: visited={} limit={} last={}",
                                    visited.len(),
                                    limit,
                                    file_path.display()
                                ));
                            }
                        }

                        // Mark as visited before processing to detect cycles
                        visited.insert(normalized_path.clone());

                        if stage_trace_enabled() && visited.len() % 100 == 0 {
                            eprintln!(
                                "IMPORT_PROGRESS visited={} current={}",
                                visited.len(),
                                file_path.display()
                            );
                        }

                        log::debug!("Resolving import {:?} → {:?}", variant, file_path);
                        match std::fs::read_to_string(&file_path) {
                            Ok(src) => match lexer::tokenize(&src) {
                                Ok(tokens) => match parser::parse(tokens, tick_limit) {
                                    Ok(mut imported) => {
                                        // Recursively resolve imports in the imported module
                                        imported = resolve_imports_impl(
                                            imported, &file_path, tick_limit, visited,
                                        )?;
                                        resolved_items.extend(imported.items);
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "warning: failed to parse imported module {:?}: {}",
                                            file_path, e
                                        );
                                        had_unresolved = true;
                                    }
                                },
                                Err(e) => {
                                    eprintln!(
                                        "warning: failed to tokenize imported module {:?}: {}",
                                        file_path, e
                                    );
                                    had_unresolved = true;
                                }
                            },
                            Err(e) => {
                                eprintln!(
                                    "warning: failed to read imported module {:?}: {}",
                                    file_path, e
                                );
                                had_unresolved = true;
                            }
                        }
                    } else {
                        searched_candidates.extend(candidate_files);
                    }
                }

                if !found_any_candidate_file {
                    // None of the import path variants resolved to a file.
                    // Emit one warning for the original import path to keep
                    // diagnostics concise and focused.
                    let mut seen = HashSet::new();
                    let searched_candidates: Vec<std::path::PathBuf> = searched_candidates
                        .into_iter()
                        .filter(|p| seen.insert(p.clone()))
                        .collect();
                    let relative = path.join("/");

                    if let Some(alias) = alias {
                        eprintln!(
                            "warning: unresolved import '{}' (as '{}'): searched {:?}",
                            relative, alias, searched_candidates
                        );
                    } else {
                        eprintln!(
                            "warning: unresolved import '{}': searched {:?}",
                            relative, searched_candidates
                        );
                    }
                    log::warn!(
                        "Unresolved import: {:?} — searched {:?}",
                        path,
                        searched_candidates
                    );
                    had_unresolved = true;
                }

                if had_unresolved {
                    // Keep unresolved import so downstream diagnostics can still surface context.
                    resolved_items.push(item);
                }
            }
            _ => resolved_items.push(item),
        }
    }

    Ok(Module {
        items: resolved_items,
    })
}

/// Apply `#[cfg(...)]` conditional compilation attributes.
///
/// Walks the top-level items in a module and removes any item whose `#[cfg(...)]`
/// attribute does not match the current compilation target. Also normalizes the
/// `@cfg(...)` decorator syntax (used in `std/time.omni`) to `#[cfg(...)]`.
///
/// Supported conditions:
///   - `#[cfg(unix)]`                     — matches when OS family is unix
///   - `#[cfg(windows)]`                  — matches when OS family is windows
///   - `#[cfg(target_os = "<os>")]`       — matches a specific OS (linux, macos, windows)
///
/// Items without any `#[cfg(...)]` attribute are always retained.
fn parse_external_cfg_flags() -> std::collections::HashSet<String> {
    let raw = std::env::var("OMNI_CFG_FLAGS").unwrap_or_default();
    raw.split(';')
        .map(str::trim)
        .filter(|flag| !flag.is_empty())
        .map(|flag| flag.to_string())
        .collect()
}

fn parse_external_link_directives() -> ExternalLinkDirectives {
    fn split_env_list(var_name: &str) -> Vec<String> {
        std::env::var(var_name)
            .unwrap_or_default()
            .split(';')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string())
            .collect()
    }

    ExternalLinkDirectives {
        link_libs: split_env_list("OMNI_LINK_LIBS"),
        link_paths: split_env_list("OMNI_LINK_PATHS"),
    }
}

fn write_link_directives_sidecar(
    output_path: &std::path::Path,
    directives: &ExternalLinkDirectives,
) -> Result<()> {
    if directives.link_libs.is_empty() && directives.link_paths.is_empty() {
        return Ok(());
    }

    let file_name = output_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("output");
    let sidecar = output_path.with_file_name(format!("{file_name}.link"));

    let mut content = String::new();
    content.push_str("# omni link directives\n");
    for lib in &directives.link_libs {
        content.push_str("link_lib=");
        content.push_str(lib);
        content.push('\n');
    }
    for path in &directives.link_paths {
        content.push_str("link_path=");
        content.push_str(path);
        content.push('\n');
    }

    std::fs::write(&sidecar, content.as_bytes())
        .map_err(|e| anyhow::anyhow!("failed to write link sidecar {:?}: {}", sidecar, e))?;
    log::info!("Wrote linker directives sidecar to {:?}", sidecar);
    Ok(())
}

fn apply_cfg_attributes(
    module: parser::ast::Module,
    external_cfg_flags: &std::collections::HashSet<String>,
) -> parser::ast::Module {
    let os = std::env::consts::OS; // "linux", "macos", "windows", etc.
    let family = std::env::consts::FAMILY; // "unix" or "windows"

    let items = module
        .items
        .into_iter()
        .filter(|item| {
            let attrs = item_attributes(item);
            cfg_attrs_match(attrs, os, family, external_cfg_flags)
        })
        .map(|item| apply_cfg_to_inner_items(item, os, family, external_cfg_flags))
        .collect();

    parser::ast::Module { items }
}

/// Extract the attributes slice from an item, if it carries attributes.
fn item_attributes(item: &parser::ast::Item) -> &[String] {
    match item {
        parser::ast::Item::Function(f) => &f.attributes,
        parser::ast::Item::Struct(s) => &s.attributes,
        parser::ast::Item::Enum(e) => &e.attributes,
        parser::ast::Item::Module(m) => &m.attributes,
        parser::ast::Item::Trait(t) => &t.attributes,
        parser::ast::Item::Impl(i) => &i.attributes,
        parser::ast::Item::Const(c) => &c.attributes,
        parser::ast::Item::Static(s) => &s.attributes,
        // Items that don't carry attributes are always retained
        parser::ast::Item::Import(_)
        | parser::ast::Item::TypeAlias(_)
        | parser::ast::Item::Extern(_)
        | parser::ast::Item::Comptime(_)
        | parser::ast::Item::Macro(_) => &[],
    }
}

/// Recursively apply cfg filtering to nested items (e.g. methods inside
/// impl blocks, functions inside modules).
fn apply_cfg_to_inner_items(
    item: parser::ast::Item,
    os: &str,
    family: &str,
    external_cfg_flags: &std::collections::HashSet<String>,
) -> parser::ast::Item {
    match item {
        parser::ast::Item::Module(mut m) => {
            m.items = m
                .items
                .into_iter()
                .filter(|i| cfg_attrs_match(item_attributes(i), os, family, external_cfg_flags))
                .map(|i| apply_cfg_to_inner_items(i, os, family, external_cfg_flags))
                .collect();
            parser::ast::Item::Module(m)
        }
        parser::ast::Item::Impl(mut imp) => {
            imp.methods = imp
                .methods
                .into_iter()
                .filter(|f| cfg_attrs_match(&f.attributes, os, family, external_cfg_flags))
                .collect();
            parser::ast::Item::Impl(imp)
        }
        parser::ast::Item::Trait(mut t) => {
            t.methods = t
                .methods
                .into_iter()
                .filter(|f| cfg_attrs_match(&f.attributes, os, family, external_cfg_flags))
                .collect();
            parser::ast::Item::Trait(t)
        }
        parser::ast::Item::Struct(mut s) => {
            s.methods = s
                .methods
                .into_iter()
                .filter(|f| cfg_attrs_match(&f.attributes, os, family, external_cfg_flags))
                .collect();
            parser::ast::Item::Struct(s)
        }
        parser::ast::Item::Extern(mut e) => {
            e.functions = e
                .functions
                .into_iter()
                .filter(|f| cfg_attrs_match(&f.attributes, os, family, external_cfg_flags))
                .collect();
            parser::ast::Item::Extern(e)
        }
        other => other,
    }
}

/// Return `true` if all `#[cfg(...)]` attributes on an item match the current platform.
/// If there are no cfg attributes, the item is unconditionally included.
///
/// Also recognizes the `@cfg(...)` decorator syntax and normalizes it to `#[cfg(...)]`.
fn cfg_attrs_match(
    attrs: &[String],
    os: &str,
    family: &str,
    external_cfg_flags: &std::collections::HashSet<String>,
) -> bool {
    for attr in attrs {
        // Normalize @cfg(...) → #[cfg(...)] for matching purposes
        let normalized = if attr.starts_with("@cfg(") {
            format!("#[cfg({})]", &attr[5..attr.len().saturating_sub(1)])
        } else {
            attr.clone()
        };

        if let Some(condition) = extract_cfg_condition(&normalized) {
            if !evaluate_cfg_condition(condition, os, family, external_cfg_flags) {
                return false;
            }
        }
    }
    true
}

/// Extract the inner condition string from a `#[cfg(...)]` attribute.
/// Returns `None` if the attribute is not a cfg attribute.
///
/// Examples:
///   `#[cfg(unix)]`                          → Some("unix")
///   `#[cfg(target_os, =, "linux")]`         → Some("target_os, =, \"linux\"")
///   `#[cfg(windows)]`                       → Some("windows")
///   `#[inline]`                             → None
fn extract_cfg_condition(attr: &str) -> Option<&str> {
    let trimmed = attr.trim();
    if trimmed.starts_with("#[cfg(") && trimmed.ends_with(")]") {
        // Strip "#[cfg(" from front and ")]" from back
        Some(&trimmed[6..trimmed.len() - 2])
    } else {
        None
    }
}

/// Evaluate a cfg condition against the current platform.
///
/// The condition string comes from the parsed attribute. Due to how the parser
/// joins tokens, `target_os = "linux"` is stored as `target_os, =, "linux"`.
fn evaluate_cfg_condition(
    condition: &str,
    os: &str,
    family: &str,
    external_cfg_flags: &std::collections::HashSet<String>,
) -> bool {
    let cond = condition.trim();

    if let Some(inner) = cfg_function_args(cond, "any") {
        let args = split_cfg_args(inner);
        if args.is_empty() {
            return false;
        }
        return args
            .iter()
            .any(|arg| evaluate_cfg_condition(arg, os, family, external_cfg_flags));
    }

    if let Some(inner) = cfg_function_args(cond, "all") {
        let args = split_cfg_args(inner);
        if args.is_empty() {
            return false;
        }
        return args
            .iter()
            .all(|arg| evaluate_cfg_condition(arg, os, family, external_cfg_flags));
    }

    if let Some(inner) = cfg_function_args(cond, "not") {
        let args = split_cfg_args(inner);
        if args.len() != 1 {
            return false;
        }
        return !evaluate_cfg_condition(&args[0], os, family, external_cfg_flags);
    }

    if external_cfg_flags.contains(cond) {
        return true;
    }

    // Simple platform family: cfg(unix), cfg(windows)
    match cond {
        "unix" => return family == "unix",
        "windows" => return family == "windows",
        _ => {}
    }

    if cond.starts_with("feature") {
        let parts: Vec<&str> = cond.splitn(3, ", ").collect();
        if parts.len() == 3 && parts[0] == "feature" && parts[1] == "=" {
            let value = parts[2].trim_matches('"');
            return external_cfg_flags.contains(&format!("feature={value}"));
        }
        if let Some(rest) = cond.strip_prefix("feature") {
            let rest = rest.trim();
            if let Some(rest) = rest.strip_prefix('=') {
                let value = rest.trim().trim_matches('"');
                return external_cfg_flags.contains(&format!("feature={value}"));
            }
        }
        return false;
    }

    if cond
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return false;
    }

    // target_os = "value" — stored as "target_os, =, \"value\"" by the parser
    if cond.starts_with("target_os") {
        // Try parsing "target_os, =, \"<os_name>\""
        let parts: Vec<&str> = cond.splitn(3, ", ").collect();
        if parts.len() == 3 && parts[0] == "target_os" && parts[1] == "=" {
            let value = parts[2].trim_matches('"');
            return os == value;
        }
        // Also handle the direct form "target_os = \"<os_name>\"" (in case
        // attributes are synthesized without the parser's comma separation)
        if let Some(rest) = cond.strip_prefix("target_os") {
            let rest = rest.trim();
            if let Some(rest) = rest.strip_prefix('=') {
                let value = rest.trim().trim_matches('"');
                return os == value;
            }
        }
    }

    // Unknown cfg condition — conservatively include the item
    log::warn!(
        "Unknown #[cfg({})] condition; including item by default",
        cond
    );
    true
}

fn cfg_function_args<'a>(cond: &'a str, name: &str) -> Option<&'a str> {
    let prefix = format!("{name}(");
    if cond.starts_with(&prefix) && cond.ends_with(')') {
        return Some(&cond[prefix.len()..cond.len() - 1]);
    }

    // Parser-tokenized fallback form: `name, (, ... , )`
    let tokenized_prefix = format!("{name}, (");
    let tokenized_suffix = ", )";
    if cond.starts_with(&tokenized_prefix) && cond.ends_with(tokenized_suffix) {
        return Some(&cond[tokenized_prefix.len()..cond.len() - tokenized_suffix.len()]);
    }

    None
}

fn split_cfg_args(inner: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut depth = 0usize;
    let mut current = String::new();
    let chars: Vec<char> = inner.chars().collect();
    let mut idx = 0usize;

    while idx < chars.len() {
        let ch = chars[idx];
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth = depth.saturating_sub(1);
                current.push(ch);
            }
            ',' => {
                let next_is_space = idx + 1 < chars.len() && chars[idx + 1] == ' ';
                if depth == 0 && next_is_space {
                    let candidate = current.trim();
                    if !candidate.is_empty() {
                        args.push(candidate.to_string());
                    }
                    current.clear();
                    idx += 1;
                } else {
                    current.push(ch);
                }
            }
            _ => current.push(ch),
        }
        idx += 1;
    }

    let tail = current.trim();
    if !tail.is_empty() {
        args.push(tail.to_string());
    }

    args
}

#[cfg(test)]
mod cfg_condition_tests {
    use super::*;

    #[test]
    fn feature_condition_respects_external_flags() {
        let mut flags = std::collections::HashSet::new();
        flags.insert("feature=demo".to_string());

        assert!(evaluate_cfg_condition(
            "feature, =, \"demo\"",
            "linux",
            "unix",
            &flags
        ));
        assert!(!evaluate_cfg_condition(
            "feature, =, \"other\"",
            "linux",
            "unix",
            &flags
        ));
    }

    #[test]
    fn custom_flag_condition_respects_external_flags() {
        let mut flags = std::collections::HashSet::new();
        flags.insert("my_custom_cfg".to_string());

        assert!(evaluate_cfg_condition(
            "my_custom_cfg",
            "linux",
            "unix",
            &flags
        ));
        assert!(!evaluate_cfg_condition(
            "missing_custom_cfg",
            "linux",
            "unix",
            &flags
        ));
    }

    #[test]
    fn cfg_any_all_not_conditions_are_supported() {
        let mut flags = std::collections::HashSet::new();
        flags.insert("feature=demo".to_string());
        flags.insert("custom_build_flag".to_string());

        assert!(evaluate_cfg_condition(
            "any(windows, feature = \"demo\")",
            "linux",
            "unix",
            &flags
        ));
        assert!(evaluate_cfg_condition(
            "all(unix, not(windows), feature = \"demo\")",
            "linux",
            "unix",
            &flags
        ));
        assert!(!evaluate_cfg_condition(
            "all(unix, feature = \"missing\")",
            "linux",
            "unix",
            &flags
        ));
        assert!(evaluate_cfg_condition(
            "not(windows)",
            "linux",
            "unix",
            &flags
        ));
        assert!(evaluate_cfg_condition(
            "any(feature = \"missing\", custom_build_flag)",
            "linux",
            "unix",
            &flags
        ));
    }

    #[test]
    fn parse_external_link_directives_splits_values() {
        std::env::set_var("OMNI_LINK_LIBS", "static=foo; dylib=bar ");
        std::env::set_var("OMNI_LINK_PATHS", " /tmp/a ;/tmp/b");

        let directives = parse_external_link_directives();
        assert_eq!(directives.link_libs, vec!["static=foo", "dylib=bar"]);
        assert_eq!(directives.link_paths, vec!["/tmp/a", "/tmp/b"]);

        std::env::remove_var("OMNI_LINK_LIBS");
        std::env::remove_var("OMNI_LINK_PATHS");
    }

    #[test]
    fn write_link_directives_sidecar_writes_expected_entries() {
        let root = std::env::temp_dir().join(format!(
            "omni-link-sidecar-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("create temp dir");
        let output = root.join("demo.ovm");

        let directives = ExternalLinkDirectives {
            link_libs: vec!["static=foo".to_string()],
            link_paths: vec!["/tmp/omni".to_string()],
        };

        write_link_directives_sidecar(&output, &directives).expect("write sidecar");

        let sidecar = root.join("demo.ovm.link");
        let content = std::fs::read_to_string(&sidecar).expect("read sidecar");
        assert!(content.contains("link_lib=static=foo"));
        assert!(content.contains("link_path=/tmp/omni"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn diagnostics_json_payload_contains_expected_fields() {
        let payload = json!({
            "level": "error",
            "code": "E005",
            "message": "type error: mismatch",
            "fix": "Add an explicit type annotation or insert a cast to align expected and actual types.",
        });
        let value: serde_json::Value =
            serde_json::from_str(&payload.to_string()).expect("valid json payload");

        assert_eq!(value["level"], "error");
        assert_eq!(value["code"], "E005");
        assert_eq!(value["message"], "type error: mismatch");
        assert_eq!(
            value["fix"],
            "Add an explicit type annotation or insert a cast to align expected and actual types."
        );
    }

    #[test]
    fn suggested_machine_fix_returns_expected_message() {
        let type_fix = suggested_machine_fix(Some("E005"), "type error: mismatch");
        assert!(type_fix
            .expect("type fix")
            .contains("explicit type annotation"));

        let borrow_fix = suggested_machine_fix(Some("E006"), "borrow check: overlap");
        assert!(borrow_fix.expect("borrow fix").contains("overlapping borrows"));

        let none_fix = suggested_machine_fix(None, "informational");
        assert!(none_fix.is_none());
    }
}

/// Determine whether a type error is a "hard" (fatal) error.
///
/// Delegates to the canonical implementation in `semantic::type_inference`.
/// See that module's documentation for the full classification rules (O-097).
fn is_hard_type_error(err: &semantic::type_inference::TypeError) -> bool {
    semantic::type_inference::is_hard_type_error(err)
}

/// Walk a statement tree and check memory operations against the zone checker.
fn check_memory_ops_stmt(
    stmt: &parser::ast::Statement,
    checker: &mut modes::MemoryZoneChecker,
    func_name: &str,
) {
    use parser::ast::Statement;

    match stmt {
        Statement::Let {
            value: Some(value), ..
        }
        | Statement::Var {
            value: Some(value), ..
        } => {
            check_memory_ops_expr(value, checker, func_name);
        }
        Statement::Return(Some(expr)) => {
            check_memory_ops_expr(expr, checker, func_name);
        }
        Statement::Expression(expr) => {
            check_memory_ops_expr(expr, checker, func_name);
        }
        Statement::If {
            condition,
            then_block,
            else_block,
        } => {
            check_memory_ops_expr(condition, checker, func_name);
            for s in &then_block.statements {
                check_memory_ops_stmt(s, checker, func_name);
            }
            if let Some(eb) = else_block {
                for s in &eb.statements {
                    check_memory_ops_stmt(s, checker, func_name);
                }
            }
        }
        Statement::While { condition, body }
        | Statement::For {
            iter: condition,
            body,
            ..
        } => {
            check_memory_ops_expr(condition, checker, func_name);
            for s in &body.statements {
                check_memory_ops_stmt(s, checker, func_name);
            }
        }
        Statement::Assignment { value, .. } => {
            check_memory_ops_expr(value, checker, func_name);
        }
        _ => {}
    }
}

/// Walk an expression tree and detect memory operations.
fn check_memory_ops_expr(
    expr: &parser::ast::Expression,
    checker: &mut modes::MemoryZoneChecker,
    func_name: &str,
) {
    use parser::ast::Expression;

    match expr {
        // Array literal — GC allocation
        Expression::Array(items) => {
            checker.check_operation(
                modes::MemoryOperation::GcAlloc,
                &format!("{}: array literal", func_name),
            );
            for item in items {
                check_memory_ops_expr(item, checker, func_name);
            }
        }
        // Struct literal — GC allocation
        Expression::StructLiteral { fields, .. } => {
            checker.check_operation(
                modes::MemoryOperation::GcAlloc,
                &format!("{}: struct literal", func_name),
            );
            for (_, val) in fields {
                check_memory_ops_expr(val, checker, func_name);
            }
        }
        // Borrow — ownership operation
        Expression::Borrow { mutable, .. } => {
            if *mutable {
                checker.check_operation(
                    modes::MemoryOperation::MutableBorrow,
                    &format!("{}: mutable borrow", func_name),
                );
            } else {
                checker.check_operation(
                    modes::MemoryOperation::SharedBorrow,
                    &format!("{}: shared borrow", func_name),
                );
            }
        }
        // Ownership annotations
        Expression::Shared(inner) => {
            checker.check_operation(
                modes::MemoryOperation::SharedBorrow,
                &format!("{}: shared ownership", func_name),
            );
            check_memory_ops_expr(inner, checker, func_name);
        }
        Expression::Own(inner) => {
            checker.check_operation(
                modes::MemoryOperation::OwnershipMove,
                &format!("{}: own ownership", func_name),
            );
            check_memory_ops_expr(inner, checker, func_name);
        }
        // Function calls — check for allocation patterns
        Expression::Call(callee, args) => {
            if let Expression::Path(_, method) = callee.as_ref() {
                match method.as_str() {
                    "new" | "from" => {
                        checker.check_operation(
                            modes::MemoryOperation::GcAlloc,
                            &format!("{}: {}::{}()", func_name, "type", method),
                        );
                    }
                    _ => {}
                }
            }
            for arg in args {
                check_memory_ops_expr(arg, checker, func_name);
            }
        }
        // Recurse into sub-expressions
        Expression::Binary(left, _, right) => {
            check_memory_ops_expr(left, checker, func_name);
            check_memory_ops_expr(right, checker, func_name);
        }
        Expression::Unary(_, inner) | Expression::Deref(inner) | Expression::Await(inner) => {
            check_memory_ops_expr(inner, checker, func_name);
        }
        Expression::Field(inner, _) | Expression::Path(inner, _) => {
            check_memory_ops_expr(inner, checker, func_name);
        }
        Expression::Index(obj, idx) => {
            check_memory_ops_expr(obj, checker, func_name);
            check_memory_ops_expr(idx, checker, func_name);
        }
        Expression::Tuple(items) => {
            for item in items {
                check_memory_ops_expr(item, checker, func_name);
            }
        }
        Expression::MethodCall { receiver, args, .. } => {
            check_memory_ops_expr(receiver, checker, func_name);
            for arg in args {
                check_memory_ops_expr(arg, checker, func_name);
            }
        }
        _ => {}
    }
}
