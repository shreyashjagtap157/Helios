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
//! The Omni programming language compiler for Project HELIOS.
//! Supports multiple backends: LLVM (native), OVM (bytecode), and hybrid.
//! Features hardware-adaptive compilation and universal execution model.

mod codegen;
mod ir;
mod lexer;
mod modes;
mod monitor;
mod parser;
mod resolver;
mod runtime;
mod semantic;
use sysinfo::{ProcessExt, System, SystemExt};
// `pprof` is only available on Unix targets (native signal-based sampling).
#[cfg(unix)]
use pprof::ProfilerGuard;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Code generation target
#[derive(Debug, Clone, Copy, ValueEnum, Default, PartialEq)]
pub enum Target {
    #[cfg(feature = "llvm")]
    Llvm, // LLVM IR -> native code
    #[default]
    Ovm, // OVM bytecode for managed execution
    #[cfg(feature = "llvm")]
    Hybrid, // Both native and managed
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

    /// Dump typed AST (after semantic analysis)
    #[arg(long)]
    emit_typed_ast: bool,

    /// Arguments to pass to the program when using --run
    #[arg(last = true)]
    program_args: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    if args.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
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
            let _guard: Option<()> = None; // profiler not available on non-unix targets
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
                    log::info!(
                        "monitor: tokens={} items={} cpu={:.2}% mem={} KB virt={} KB last_hb={}",
                        tokens,
                        items,
                        p.cpu_usage(),
                        p.memory(),
                        p.virtual_memory(),
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
                                p.cpu_usage(),
                                last,
                                cur,
                                preview.join("\n"),
                                errors.join("\n")
                            );
                        let fname = format!(
                            "diagnostics/monitor_stall_{}.log",
                            chrono::Utc::now().format("%Y%m%dT%H%M%S")
                        );
                        let _ = std::fs::write(&fname, dump.as_bytes());
                        log::warn!("monitor: detected stall; wrote {}", fname);
                        // Also attempt to write an in-process CPU flamegraph snapshot if profiling is active
                        #[cfg(unix)]
                        {
                            if let Some(g) = &guard {
                                if let Ok(report) = g.report().build() {
                                    let fg_name = format!(
                                        "diagnostics/monitor_flame_{}.svg",
                                        chrono::Utc::now().format("%Y%m%dT%H%M%S")
                                    );
                                    match File::create(&fg_name) {
                                        Ok(mut f) => {
                                            if let Err(e) = report.flamegraph(&mut f) {
                                                log::error!(
                                                    "monitor: failed to write flamegraph {}: {}",
                                                    fg_name,
                                                    e
                                                );
                                            } else {
                                                log::warn!("monitor: wrote flamegraph {}", fg_name);
                                            }
                                        }
                                        Err(e) => {
                                            log::error!(
                                                "monitor: failed to create flamegraph file {}: {}",
                                                fg_name,
                                                e
                                            );
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
    let tokens = lexer::tokenize(source)?;
    // heartbeat: tokens produced
    monitor::update_heartbeat();

    // Phase 2: Parsing
    log::debug!("Phase 2: Parsing");
    monitor::update_heartbeat();
    let ast = parser::parse(tokens)?;

    // Phase 2.0: Import resolution
    log::debug!("Phase 2.0: Import resolution");
    let ast = resolve_imports(ast, &args.input)?;

    // Phase 2.0.1: Conditional compilation (#[cfg(...)] filtering)
    log::debug!("Phase 2.0.1: Conditional compilation (cfg filtering)");
    let ast = apply_cfg_attributes(ast);

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
            eprintln!("warning[E007]: memory zone: {}", v);
        }
    }

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
            let hard_errors: Vec<_> = errors.iter().filter(|e| is_hard_type_error(e)).collect();
            let warnings: Vec<_> = errors.iter().filter(|e| !is_hard_type_error(e)).collect();
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
        monitor::update_heartbeat();
        return codegen::ovm_direct::generate_ovm_direct(&typed_ast, &output_path)
            .map_err(|e| anyhow::anyhow!("{}", e));
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
) -> Result<parser::ast::Module> {
    use parser::ast::{ImportDecl, Item, Module};

    let base_dir = current_file
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));

    let mut resolved_items: Vec<Item> = Vec::new();

    for item in module.items {
        match &item {
            Item::Import(ImportDecl { path, alias }) => {
                // Convert path segments to file path: ["std", "io"] → "std/io.omni"
                let relative = path.join("/");
                let file_path = base_dir.join(format!("{}.omni", relative));

                if file_path.exists() {
                    log::debug!("Resolving import {:?} → {:?}", path, file_path);
                    match std::fs::read_to_string(&file_path) {
                        Ok(src) => match lexer::tokenize(&src) {
                            Ok(tokens) => match parser::parse(tokens) {
                                Ok(mut imported) => {
                                    // Recursively resolve imports in the imported module
                                    imported = resolve_imports(imported, &file_path)?;
                                    resolved_items.extend(imported.items);
                                }
                                Err(e) => {
                                    eprintln!(
                                        "warning: failed to parse imported module {:?}: {}",
                                        file_path, e
                                    );
                                    // Keep the unresolved import so downstream knows
                                    resolved_items.push(item);
                                }
                            },
                            Err(e) => {
                                eprintln!(
                                    "warning: failed to tokenize imported module {:?}: {}",
                                    file_path, e
                                );
                                resolved_items.push(item);
                            }
                        },
                        Err(e) => {
                            eprintln!(
                                "warning: failed to read imported module {:?}: {}",
                                file_path, e
                            );
                            resolved_items.push(item);
                        }
                    }
                } else {
                    // File not found — warn but don't fail compilation
                    if let Some(alias) = alias {
                        eprintln!(
                            "warning: unresolved import '{}' (as '{}'): file not found {:?}",
                            relative, alias, file_path
                        );
                    } else {
                        eprintln!(
                            "warning: unresolved import '{}': file not found {:?}",
                            relative, file_path
                        );
                    }
                    log::warn!(
                        "Unresolved import: {:?} — file not found at {:?}",
                        path,
                        file_path
                    );
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
fn apply_cfg_attributes(module: parser::ast::Module) -> parser::ast::Module {
    let os = std::env::consts::OS; // "linux", "macos", "windows", etc.
    let family = std::env::consts::FAMILY; // "unix" or "windows"

    let items = module
        .items
        .into_iter()
        .filter(|item| {
            let attrs = item_attributes(item);
            cfg_attrs_match(attrs, os, family)
        })
        .map(|item| apply_cfg_to_inner_items(item, os, family))
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
fn apply_cfg_to_inner_items(item: parser::ast::Item, os: &str, family: &str) -> parser::ast::Item {
    match item {
        parser::ast::Item::Module(mut m) => {
            m.items = m
                .items
                .into_iter()
                .filter(|i| cfg_attrs_match(item_attributes(i), os, family))
                .map(|i| apply_cfg_to_inner_items(i, os, family))
                .collect();
            parser::ast::Item::Module(m)
        }
        parser::ast::Item::Impl(mut imp) => {
            imp.methods = imp
                .methods
                .into_iter()
                .filter(|f| cfg_attrs_match(&f.attributes, os, family))
                .collect();
            parser::ast::Item::Impl(imp)
        }
        parser::ast::Item::Trait(mut t) => {
            t.methods = t
                .methods
                .into_iter()
                .filter(|f| cfg_attrs_match(&f.attributes, os, family))
                .collect();
            parser::ast::Item::Trait(t)
        }
        parser::ast::Item::Struct(mut s) => {
            s.methods = s
                .methods
                .into_iter()
                .filter(|f| cfg_attrs_match(&f.attributes, os, family))
                .collect();
            parser::ast::Item::Struct(s)
        }
        parser::ast::Item::Extern(mut e) => {
            e.functions = e
                .functions
                .into_iter()
                .filter(|f| cfg_attrs_match(&f.attributes, os, family))
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
fn cfg_attrs_match(attrs: &[String], os: &str, family: &str) -> bool {
    for attr in attrs {
        // Normalize @cfg(...) → #[cfg(...)] for matching purposes
        let normalized = if attr.starts_with("@cfg(") {
            format!("#[cfg({})]", &attr[5..attr.len().saturating_sub(1)])
        } else {
            attr.clone()
        };

        if let Some(condition) = extract_cfg_condition(&normalized) {
            if !evaluate_cfg_condition(condition, os, family) {
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
fn evaluate_cfg_condition(condition: &str, os: &str, family: &str) -> bool {
    let cond = condition.trim();

    // Simple platform family: cfg(unix), cfg(windows)
    match cond {
        "unix" => return family == "unix",
        "windows" => return family == "windows",
        _ => {}
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
