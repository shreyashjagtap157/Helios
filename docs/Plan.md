# HELIOS / Omni — Deployment-Ready Implementation Plan

**Version:** 2.0 — Generated 2026-03-12  
**Scope:** Every task required to bring the HELIOS Cognitive Framework and Omni Language to deployment on Windows 10/11 as a background service with a native GUI, including detailed sub-task breakdowns, file paths, code-level guidance, and verification criteria.

> **How to use this plan:** Each numbered section is a work stream. Within each section, tasks are ordered by dependency. Checkboxes (`[ ]`) track completion. Verification blocks at the end of each section describe exactly how to confirm the work is done.

---

## Table of Contents

1. [Codebase Reality Audit](#1-codebase-reality-audit)
2. [Compiler & Language Stabilization](#2-compiler--language-stabilization)
3. [OVM Runtime Consolidation](#3-ovm-runtime-consolidation)
4. [Knowledge Store Hardening](#4-knowledge-store-hardening)
5. [HELIOS Framework Completion](#5-helios-framework-completion)
6. [Service Layer & IPC](#6-service-layer--ipc)
7. [WinUI 3 Desktop GUI](#7-winui-3-desktop-gui)
8. [Plugin Subsystem](#8-plugin-subsystem)
9. [Standard Library Completion](#9-standard-library-completion)
10. [Testing Infrastructure](#10-testing-infrastructure)
11. [Build, Package & Deploy](#11-build-package--deploy)
12. [CI/CD Pipeline](#12-cicd-pipeline)
13. [Documentation](#13-documentation)
14. [Deployment Verification Checklist](#14-deployment-verification-checklist)

---

## 1. Codebase Reality Audit

Before any implementation, the team must understand what actually exists. This section documents the ground truth discovered during the codebase analysis.

### 1.1 Active Directory Structure

```
d:\Project\Helios\
├── omni-lang/                     ← Omni language: compiler, std, tools
│   ├── compiler/                  ← Rust-based compiler (omnc)
│   │   ├── src/main.rs            ← CLI entrypoint (clap-based, 285 LOC)
│   │   ├── src/runtime/           ← OVM bytcode VM lives HERE (not ovm/)
│   │   │   ├── vm.rs              ← Stack-based VM, tri-color GC (1790 LOC)
│   │   │   ├── interpreter.rs     ← Tree-walk interpreter (135KB)
│   │   │   ├── bytecode_compiler.rs ← AST→bytecode (49KB)
│   │   │   ├── bytecode.rs        ← OpCode enum, OvmModule (37KB)
│   │   │   ├── native.rs          ← Native function bindings
│   │   │   ├── network.rs         ← Distributed execution stubs
│   │   │   ├── hot_swap.rs        ← Hot code replacement
│   │   │   ├── profiler.rs        ← Runtime profiling
│   │   │   └── tests.rs           ← Runtime unit tests (2KB — very thin)
│   │   ├── src/codegen/           ← 27 files: JIT, LLVM, GPU, linker, optimizer
│   │   │   ├── self_hosting.rs    ← Self-hosting infra (30KB)
│   │   │   ├── ovm.rs             ← OVM bytecode codegen (31KB)
│   │   │   ├── native_codegen.rs  ← Native machine code gen (81KB)
│   │   │   ├── linker.rs          ← Object linker (81KB)
│   │   │   └── optimizer.rs       ← IR optimizer (92KB)
│   │   ├── src/semantic/          ← 19 files: type checking, borrow checking
│   │   │   ├── mod.rs             ← Semantic analysis core (85KB)
│   │   │   ├── type_inference.rs  ← HM type inference (77KB)
│   │   │   └── borrow_check.rs    ← Ownership/borrow checker (56KB)
│   │   ├── src/brain/             ← 4 files: knowledge graph, reasoning
│   │   ├── src/lexer/             ← Lexer (logos-based)
│   │   ├── src/parser/            ← Parser (hand-written recursive descent, 1619 LOC)
│   │   ├── src/language_features/ ← 5 files
│   │   ├── src/optimizer/         ← 5 files
│   │   ├── src/safety/            ← 2 files
│   │   ├── src/ir/                ← 1 file
│   │   ├── src/diagnostics.rs     ← Error reporting (12KB)
│   │   ├── Cargo.toml             ← deps: tokio, reqwest, serde, clap, etc.
│   │   └── (NO tests/ directory)  ← Tests are inline #[cfg(test)] in modules
│   ├── core/                      ← 11 Omni core modules (math, io, networking, etc.)
│   ├── std/                       ← 33 Omni standard library modules
│   ├── tools/                     ← opm, omni-fmt, omni-lsp, omni-dap, vscode-omni
│   ├── tests/                     ← 6 test files (moved here from root)
│   ├── examples/                  ← 15 example files
│   ├── docs/                      ← 1 file
│   └── omni/                      ← 38 files (Omni runtime/framework in Omni)
│
├── helios-framework/              ← HELIOS cognitive framework (Omni source)
│   ├── main.omni                  ← Service entrypoint: --service, --repl, --capabilities
│   ├── helios/                    ← 10 Omni modules:
│   │   ├── runtime.omni           ← Core runtime: Helios struct, process_input (338 LOC)
│   │   ├── knowledge.omni         ← KnowledgeStore: query, verify, flush-to-JSON (798 LOC)
│   │   ├── experience.omni        ← ExperienceLog: session, events (13KB)
│   │   ├── capability.omni        ← CapabilityRegistry: 15+ capabilities (29KB)
│   │   ├── cognitive.omni         ← CognitiveCortex stub (10KB)
│   │   ├── input.omni             ← NLU input processing (15KB)
│   │   ├── output.omni            ← Response formatting (7KB)
│   │   ├── api.omni               ← HTTP API server (6KB)
│   │   ├── service.omni           ← Service lifecycle, health, requests (168 LOC)
│   │   └── self_model.omni        ← Self-model introspection (6KB)
│   ├── brain/                     ← 16 files: reasoning, knowledge graph, query processing
│   ├── config/                    ← default.toml, loader.omni, default.omni
│   ├── app/                       ← 4 files: app.omni, gui.omni, extensions, os_integration
│   ├── training/                  ← pipeline/ (5 files), checkpoints/
│   ├── safety/, kernel/, os-hooks/ ← 1 file each (stubs)
│   └── (NO tests/ directory)      ← Zero test files
│
├── docs/                          ← Comprehensive spec (380KB) + grammar.bnf
├── config/                        ← default.toml + loader.omni
├── examples/                      ← 2 Omni example files
├── legacy/                        ← 4 zip archives (1.3GB total)
└── omni.toml                      ← Project config
```

- #### 1.1.1 Directory Analysis and Subsystem Mapping
  - ##### 1.1.1.1 Omni Language Toolchain (`omni-lang/`)
    - ###### 1.1.1.1.1 Compiler Subsystem Architecture (`compiler/`)
      - **1.1.1.1.1.1** Rust Implementation Architecture
        - **1.1.1.1.1.1.1** Executable Entrypoint Design (`src/main.rs`)
          - **1.1.1.1.1.1.1.1** Command Line Argument Parsing Engine (Clap)
            - **1.1.1.1.1.1.1.1.1** Subcommand Routing Logic
              - **1.1.1.1.1.1.1.1.1.1** Process Exit Code Standards for CI Integrations
  - ##### 1.1.1.2 HELIOS Framework (`helios-framework/`)
    - ###### 1.1.1.2.1 Service Initialization (`main.omni`)
      - **1.1.1.2.1.1** Flag Argument Resolution
        - **1.1.1.2.1.1.1** `--service` Mode Activation Toggle
          - **1.1.1.2.1.1.1.1** Named Pipe Binding Deferral
            - **1.1.1.2.1.1.1.1.1** Thread Management Strategies
              - **1.1.1.2.1.1.1.1.1.1** Main Tokio Runtime Async Wrapping

### 1.2 Critical Findings

| Finding | Impact | Plan Section |
|---------|--------|-------------|
| `ovm/` directory is **empty** — VM lives in `compiler/src/runtime/` | Plan §2 references `ovm/src/allocator.rs` etc. that don't exist at those paths | §3 |
| No `compiler/tests/` directory | Tests are inline `#[cfg(test)]` in each module, not in separate test files | §10 |
| `KnowledgeStore` serializes to **JSON** (`knowledge.json`) | Spec requires binary `.omk` page format with compression and encryption | §4 |
| `helios-framework/` has **zero tests** | No test coverage for the entire cognitive framework | §10 |
| Service uses **HTTP API** (`api::run_server`) not **named pipe IPC** | Spec §12.5 requires named pipe for GUI↔service communication | §6 |
| `helios-framework/helios/cognitive.omni` is a **stub** | No RETE network, no backward chaining, no cognitive layers | §5 |
| Tools workspace builds but `omni-lsp` may have incomplete Cargo.toml | Previously reported as blocking `opm` build | §2 |
| `codegen/self_hosting.rs` exists (30KB) but no bootstrap `.omni` driver | Self-hosting infrastructure present in Rust; no Omni-side entrypoint | §2 |
| Brain modules (`brain/`) in both `compiler/src/` and `helios-framework/` | Potential duplication; need to clarify which is canonical | §5 |
| GUI code in `helios-framework/app/gui.omni` is a **design document** (13KB) | Not runnable WinUI3 code; contains structural definitions only | §7 |

- #### 1.2.1 Architectural Deviations Protocol
  - ##### 1.2.1.1 Missing `ovm/` Directory Investigation Track
    - ###### 1.2.1.1.1 VM Relocation Impact Assessment
      - **1.2.1.1.1.1** `compiler/src/runtime/` Encapsulation Validity
        - **1.2.1.1.1.1.1** Build System Configuration Ripple Effects
          - **1.2.1.1.1.1.1.1** Cargo.toml Workspace Roots Modifications
            - **1.2.1.1.1.1.1.1.1** Dependency Resolution Graphs Traversal
              - **1.2.1.1.1.1.1.1.1.1** Circular Dependency Prevention Protocols
  - ##### 1.2.1.2 Missing Test Directory Ramifications
    - ###### 1.2.1.2.1 Inline Testing Extraction
      - **1.2.1.2.1.1** `#[cfg(test)]` Rust Module Harvesting
        - **1.2.1.2.1.1.1** Integration Test Rig Generation
          - **1.2.1.2.1.1.1.1** Test Runner Process Isolation
            - **1.2.1.2.1.1.1.1.1** Cross-Platform Exit Status Codes
              - **1.2.1.2.1.1.1.1.1.1** CI Quality Gate Halts On Failure

### 1.3 Housekeeping Status

- [x] Legacy directories archived to `legacy/legacy_raw.zip`
- [x] Loose test files moved from root to `omni-lang/tests/`
- [x] `.gitignore` updated to track docs and block legacy zips
- [x] `compile_output.txt` removed from root

- #### 1.3.1 Clean Repository Maintenance Cycle
  - ##### 1.3.1.1 Legacy Artifact Archival Processes
    - ###### 1.3.1.1.1 Zip Format Default Settings
      - **1.3.1.1.1.1** Deflate Compression Ratio Analysis
        - **1.3.1.1.1.1.1** File System Storage Footprint Modeling
          - **1.3.1.1.1.1.1.1** Alternative Source Control Solutions (Git LFS)
            - **1.3.1.1.1.1.1.1.1** Repository Clone Time Optimizations
              - **1.3.1.1.1.1.1.1.1.1** Subversion Tag Equivalent References
  - ##### 1.3.1.2 Test Case Reorganization
    - ###### 1.3.1.2.1 Orphan Status Resolution
      - **1.3.1.2.1.1** `omni-lang/tests/` Target Layout
        - **1.3.1.2.1.1.1** Module-level Test Naming Conventions
          - **1.3.1.2.1.1.1.1** Regex Filename Matchers in Glob Imports
            - **1.3.1.2.1.1.1.1.1** Iteration Protocol for Directory Walking
              - **1.3.1.2.1.1.1.1.1.1** UTF-8 Safe Filename Processing

### 1.4 Architectural Contradictions — Resolution Matrix

| Contradiction | Files Involved | Resolution |
|---------------|---------------|------------|
| Two runtime engines (interpreter vs VM) with incompatible value types | `runtime/interpreter.rs`, `runtime/vm.rs`, `runtime/native.rs` | Add `VmValue↔RuntimeValue` conversion; add `NativeManager` to `OmniVM`; gradually deprecate interpreter for deployment |
| `cognitive.omni` doesn't query `KnowledgeStore` | `helios/cognitive.omni`, `helios/knowledge.omni` | Add `knowledge: &mut KnowledgeStore` to `HeliosCognitive`; query in `think()` |
| `cognitive.omni` doesn't record to `ExperienceLog` | `helios/cognitive.omni`, `helios/experience.omni` | Add `experience: &mut ExperienceLog` to `HeliosCognitive`; record in `process_input()` |
| Service uses HTTP (`api.omni`) but spec requires named pipe IPC | `helios/api.omni`, `helios/service.omni` | Create `helios/ipc.omni`; service starts both |
| `runtime.omni` does its own NLU parsing but `cognitive.omni` also has `classify_intent` | `helios/runtime.omni`, `helios/cognitive.omni` | Unify: `runtime.omni::process_input()` should delegate to `cognitive.omni::process_input()` |
| `NewStruct` opcode generates `field_0, field_1` instead of real field names | `runtime/vm.rs` line 818, `runtime/bytecode.rs` | Modify opcode to carry field name vec; update serialization |
| `brain/` modules exist in BOTH `compiler/src/brain/` and `helios-framework/brain/` | Both dirs | Compiler brain = knowledge graph embedding; framework brain = reasoning engine. Document distinction in README |
| GNN (§36) and KGE (§26) use gradient descent but spec says "not gradient-trained" | Spec document | Already resolved: add architectural note saying GNNs/KGEs are "auxiliary read-only oracles trained offline" |

- #### 1.4.1 Inter-Component Friction Points Mitigation
  - ##### 1.4.1.1 Dual Engine Paradigm Unification
    - ###### 1.4.1.1.1 Interpreter vs Virtual Machine State Split
      - **1.4.1.1.1.1** Value Type Data Representation Incompatibilities
        - **1.4.1.1.1.1.1** `VmValue` Discriminant Size Evaluation
          - **1.4.1.1.1.1.1.1** `RuntimeValue` Memory Alignment
            - **1.4.1.1.1.1.1.1.1** Deep Memory Copy Performance Costs
              - **1.4.1.1.1.1.1.1.1.1** Zero-Copy FFI Marshalling Strategy
  - ##### 1.4.1.2 Cognitive Network Disconnect Tracking
    - ###### 1.4.1.2.1 Data Store Interface Omission
      - **1.4.1.2.1.1** `cognitive.omni` Capability Rectification
        - **1.4.1.2.1.1.1** `KnowledgeStore` Global Hook Injection
          - **1.4.1.2.1.1.1.1** Singleton Concurrency Locks (RwLock)
            - **1.4.1.2.1.1.1.1.1** Thread-Safe Immutable Read Traversal
              - **1.4.1.2.1.1.1.1.1.1** Pointer Aliasing Safety Audits

## 2. Compiler & Language Stabilization

**Goal:** A robust `omnc` compiler binary that can compile all HELIOS framework Omni source to OVM bytecode, with proper error handling, diagnostics, and test coverage.

### 2.1 Fix Tools Workspace Build

- [ ] **Verify `omni-lsp/Cargo.toml` exists and is valid**
  - **File:** `omni-lang/tools/omni-lsp/Cargo.toml`
  - **Action:** Confirm the file exists. If it's missing or malformed, create it with:
    ```toml
    [package]
    name = "omni-lsp"
    version = "0.1.0"
    edition = "2021"
    
    [dependencies]
    omni_compiler = { path = "../../compiler" }
    tower-lsp = "0.20"
    tokio = { version = "1", features = ["full"] }
    serde_json = "1.0"
    ```
  - **Check existing:** Run `dir omni-lang\tools\omni-lsp\` to see what files exist. The LSP needs at minimum `Cargo.toml` and `src/main.rs`.
  - **Verification:** `cd omni-lang\tools && cargo check` completes without errors.
  - #### 2.1.1 LSP Server Entry Point
    - `omni-lsp/src/main.rs` must define `#[tokio::main]` calling `Server::new(stdin, stdout).serve(backend)`.
    - The `OmniLanguageServer` struct implements `LanguageServer` trait from `tower-lsp`.
    - ##### 2.1.1.1 Required Trait Methods
      - `initialize()` — return `ServerCapabilities { text_document_sync, completion_provider, hover_provider, definition_provider }`
      - `did_open()` — parse file, run semantic analysis, push diagnostics via `client.publish_diagnostics()`
      - `did_change()` — re-lex + re-parse changed file, incremental update
      - `completion()` — use `semantic::Scope` symbol table for completions
      - `hover()` — look up symbol at cursor position, return type info
      - ###### 2.1.1.1.1 Diagnostic Mapping
        - Map `ParseError::UnexpectedToken` → LSP `DiagnosticSeverity::Error` with span from `Token.line`/`Token.column`
        - Map `SemanticError::TypeMismatch` → LSP error with suggestion from `diagnostics.rs`
        - Map `SemanticError::UndefinedSymbol` → LSP error with "did you mean?" from `Parser::suggest_hint()`
      - ###### 2.1.1.1.2 Incremental Re-Parsing Strategy
        - On `did_change`: re-tokenize only changed range using `Lexer::new(changed_text)`
        - Maintain a `HashMap<Url, (Vec<Token>, Module)>` document cache
        - Full re-parse if change spans > 50% of file; incremental otherwise
        - **2.1.1.1.2.1** Syntax Tree Patching Mechanism
          - **2.1.1.1.2.1.1** Unchanged Node Preservation
            - **2.1.1.1.2.1.1.1** Memory Address Identity Check
              - **2.1.1.1.2.1.1.1.1** Reference Counting Re-use Optimization
    - ##### 2.1.1.2 Dependency on Compiler lib.rs
      - LSP imports `omni_compiler::{lexer, parser, semantic, diagnostics}`
      - Requires all 4 modules to be `pub` in `lib.rs` (see §2.3)
  - #### 2.1.2 opm Package Manager
    - `omni-lang/tools/opm/` — currently builds, verify it can resolve and install packages
    - ##### 2.1.2.1 Core Commands to Verify
      - `opm init` — creates `omni.toml` project manifest
      - `opm add <dep>` — adds dependency to `[dependencies]` section
      - `opm build` — invokes `omnc compile` on the project entry point
      - `opm test` — invokes `omnc test` on files matching `tests/**/*.omni`
      - `opm publish` — (deferred) uploads package to registry
    - ##### 2.1.2.2 omni.toml Manifest Format
      - Must parse: `[package] name, version, edition`, `[dependencies]`, `[dev-dependencies]`, `[build]`
      - Verify opm reads `omni.toml` at project root (the file exists at `d:\Project\Helios\omni.toml`)
      - ###### 2.1.2.2.1 Dependency Version Resolution
        - **2.1.2.2.1.1** Semantic Versioning Compatibility Matrix
          - **2.1.2.2.1.1.1** SAT Solver Constraint Matching
            - **2.1.2.2.1.1.1.1** Lockfile Generation (`omni.lock`)
              - **2.1.2.2.1.1.1.1.1** Cryptographic Hash Pinning for Package Integrity
  - #### 2.1.3 omni-fmt Formatter
    - Must format `.omni` files according to style rules (indentation = 4 spaces, max line width 100)
    - ##### 2.1.3.1 AST Pretty-Printer
      - Re-lex + parse → walk AST → emit formatted source text
      - Preserve comments by attaching them to adjacent AST nodes during parsing
    - ##### 2.1.3.2 Integration with Editor
      - LSP `textDocument/formatting` calls `omni-fmt` under the hood
      - ###### 2.1.3.2.1 Format On Save Triggers
        - **2.1.3.2.1.1** Partial Document Formatting
          - **2.1.3.2.1.1.1** Character Range Selection Mapping
            - **2.1.3.2.1.1.1.1** AST Node Boundary Snapping
              - **2.1.3.2.1.1.1.1.1** Trailing Comma Injection Rule Application
  - #### 2.1.4 omni-dap Debug Adapter
    - Implements Debug Adapter Protocol for VS Code debugging of Omni programs
    - ##### 2.1.4.1 Required DAP Events
      - `StoppedEvent` — when breakpoint hit in `OmniVM::execute()`
      - `ThreadEvent` — when spawning concurrent tasks
      - `OutputEvent` — for `Print`/`PrintLn` opcode execution
    - ##### 2.1.4.2 Breakpoint Mechanism
      - Inject `Breakpoint` opcode at specified source line (requires source map from bytecode_compiler)
      - Source map: `Vec<(usize, usize)>` mapping instruction index → source line number
      - ###### 2.1.4.2.1 Conditional Breakpoint Evaluation
        - **2.1.4.2.1.1** VM State Introspection
          - **2.1.4.2.1.1.1** Expression Compilation in Current Scope Environment
            - **2.1.4.2.1.1.1.1** Hit Count Evaluation Tracker Counter
              - **2.1.4.2.1.1.1.1.1** JIT De-optimization for Debug Session Halts

- [ ] **Verify all 4 workspace members build**
  - **Command:** `cd omni-lang\tools && cargo build --workspace`
  - **Expected:** `opm.exe`, `omni-fmt.exe`, `omni-lsp.exe`, `omni-dap.exe` produced in `target/debug/`.
  - **Fix any dependency issues** by checking each member's `Cargo.toml` individually.

### 2.2 Compiler Test Infrastructure

- [ ] **Create `omni-lang/compiler/tests/` directory for integration tests**
  - Currently all tests are inline `#[cfg(test)]` blocks in each `.rs` file.
  - Create external integration tests:
    - `compiler/tests/parse_roundtrip.rs` — Parse a set of `.omni` files and verify AST structure.
    - `compiler/tests/compile_examples.rs` — Compile all files in `examples/` and `omni-lang/tests/` and verify no errors.
    - `compiler/tests/bytecode_execute.rs` — Compile to bytecode, load into `OmniVM`, execute, and check output.
  - #### 2.2.1 parse_roundtrip.rs — Detailed Design
    - For each `.omni` file, call `tokenize() → Parser::new(tokens).parse_module()`
    - Verify: no `ParseError` returned, `module.items.len() > 0`
    - ##### 2.2.1.1 Test Case Generation
      - Enumerate all `.omni` files across: `omni-lang/tests/` (6 files), `omni-lang/examples/` (15 files), `helios-framework/helios/` (10 files), `helios-framework/brain/` (14 .omni files)
      - Total: ~45 input files — one `#[test]` per file or a single parameterized test via `test_case` macro
    - ##### 2.2.1.2 AST Structural Assertion
      - For known files, assert specific structure:
        - `knowledge.omni` → must contain `Item::Struct` named "KnowledgeStore" with ≥15 fields
        - `cognitive.omni` → must contain `Item::Struct` named "HeliosCognitive"
        - `main.omni` → must contain `Item::Function` named "main"
      - ###### 2.2.1.2.1 Pattern-Match Helpers
        - `fn has_struct(module: &Module, name: &str) -> bool` — walks `module.items` for `Item::Struct(s) where s.name == name`
        - `fn has_function(module: &Module, name: &str) -> bool` — walks for `Item::Function(f) where f.name == name`
        - `fn struct_field_count(module: &Module, name: &str) -> usize` — counts fields in named struct
        - **2.2.1.2.1.1** Combinator Test Assertions
          - **2.2.1.2.1.1.1** Deep Sub-tree structural validation macros
            - **2.2.1.2.1.1.1.1** Node Equality Hash generation
              - **2.2.1.2.1.1.1.1.1** Custom AST Debug String Formatter Matcher
  - #### 2.2.2 compile_examples.rs — Detailed Design
    - **Implementation guidance:**
      ```rust
      use std::fs;
      use omni_compiler::compile; // use the lib.rs public API
      
      #[test]
      fn compile_all_examples() {
          let examples_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../examples");
          for entry in fs::read_dir(examples_dir).unwrap() {
              let path = entry.unwrap().path();
              if path.extension().map_or(false, |e| e == "omni") {
                  let source = fs::read_to_string(&path).unwrap();
                  let result = compile(&source); // adapt to actual API
                  assert!(result.is_ok(), "Failed to compile {}: {:?}", path.display(), result.err());
              }
          }
      }
      ```
    - ##### 2.2.2.1 Full Compilation Pipeline per File
      1. `tokenize(&source)` → `Vec<Token>`
      2. `Parser::new(tokens).parse_module()` → `Module`
      3. `SemanticAnalyzer::new().analyze(&module)` → `TypedModule`
      4. `BytecodeCompiler::new().compile_module(&module)` → `OvmModule`
      5. `module.serialize()` → `Vec<u8>` (verify binary format round-trips)
      - ###### 2.2.2.1.1 Error Classification
        - Lexer errors → `LexerError::InvalidToken` or `LexerError::InvalidNumber`
        - Parser errors → `ParseError::UnexpectedToken`, `ParseError::InvalidSyntax`
        - Semantic errors → `SemanticError::TypeMismatch`, `SemanticError::UndefinedSymbol`, etc.
        - Codegen errors → currently returns `anyhow::Error`; should be typed
      - ###### 2.2.2.1.2 Soft vs Hard Type Errors
        - `main.rs::is_hard_type_error()` (lines 242-284) classifies type errors
        - Hard errors: explicit type annotation mismatches (e.g., `let x: Int = "hello"`)
        - Soft errors: dynamic-flavour features (string concat with `+`, built-in functions not in environment)
        - **Decision:** For compilation tests, only hard errors should fail the test; soft errors are warnings
        - **2.2.2.1.2.1** Warning Suppression Flags in Tests
          - **2.2.2.1.2.1.1** Environment Variable Override Injection
            - **2.2.2.1.2.1.1.1** Per-Test Case Configuration Macros (`#[omni_test(allow_soft)]`)
              - **2.2.2.1.2.1.1.1.1** Rust Procedural Macro Expansion for Test Attributes
    - ##### 2.2.2.2 Test Coverage Gaps to Address
      - No existing tests cover: `match` exhaustiveness, `enum` variant construction, `trait` method dispatch, `impl` block validation, `extern` blocks, `comptime` blocks, `macro` definitions
      - Add specific test `.omni` files for each missing construct
  - #### 2.2.3 bytecode_execute.rs — Detailed Design
    - Compile an `.omni` file to `OvmModule`, then execute in `OmniVM::new().execute(&module)`
    - ##### 2.2.3.1 Test Scenarios
      - Simple arithmetic: `fn main(): let x = 2 + 3; println(x)` → verify output "5"
      - String operations: `fn main(): let s = "hello " + "world"; println(s)` → verify "hello world"
      - Control flow: `fn main(): if true: println("yes")` → verify "yes"
      - Function calls: define `fn add(a, b): return a + b; fn main(): println(add(3, 4))` → verify "7"
      - Struct creation: define struct, instantiate, access field
      - Loop: `for i in range(5): println(i)` → verify 0 through 4
      - ###### 2.2.3.1.1 Output Capture Mechanism
        - `OmniVM::execute()` currently calls `println!()` for `PrintLn` opcode
        - Must refactor: add `output: Vec<String>` field to `OmniVM` struct
        - `PrintLn` opcode pushes to `self.output` instead of `println!()` in test mode
        - Set via `OmniVM::new_with_capture()` constructor
      - ###### 2.2.3.1.2 Error Case Testing
        - Divide by zero → should return `Err("Division by zero")` not panic
        - Stack overflow → recursive function should hit `call_frames` depth limit (currently unchecked — add limit of 1024)
        - Type error at runtime → string + int should return `Err` not panic
        - **2.2.3.1.2.1** Call Frame Depth Limit Implementation
          - **2.2.3.1.2.1.1** Dynamic Re-sizing Threshold Checks
            - **2.2.3.1.2.1.1.1** Tail-Call Optimization (TCO) Bypass
              - **2.2.3.1.2.1.1.1.1** Mutual Recursion Cycle Detection Heuristics
    - ##### 2.2.3.2 GC Stress Test
      - Allocate 10,000+ objects in a loop, verify no memory leak
      - Verify `gc_collect()` is triggered automatically when `gc_alloc_count > gc_threshold` (current threshold = 256)
      - After test: `vm.heap.iter().filter(|h| !h.0.marked).count() == 0` (all live objects are reachable)
  - **Look at** `compiler/src/lib.rs` to understand what's currently exported as the public API. The file is 767 bytes — likely a thin re-export. Check if `compile()` or equivalent is public.

- [ ] **Ensure `cargo test` in compiler runs all inline tests**
  - **Command:** `cd omni-lang\compiler && cargo test 2>&1 | Tee-Object compile_test.log`
  - **Record:** Total tests run, pass/fail count.
  - **Fix any failures** before proceeding.
  - **Known warnings:** The `main.rs` has `is_hard_type_error` which treats some type errors as soft — verify this behavior aligns with spec §14 type system.
  - #### 2.2.4 Existing Inline Test Inventory
    - **Lexer** (`lexer/mod.rs` lines 490-601): 10 tests — `test_basic_tokens`, `test_struct_definition`, `test_pass_keyword`, `test_line_comments`, `test_hash_is_not_comment`, `test_attribute_tokens`, `test_all_keywords`, `test_operators`, `test_string_literal`, `test_numeric_literals`
    - **Semantic** (`semantic/tests.rs`, 10KB): type checking tests
    - **Semantic comprehensive** (`semantic/comprehensive_tests.rs`, 879 bytes): minimal cross-module tests
    - **VM** (`runtime/vm.rs` lines 952+): ~30 tests covering opcodes, GC, structs, maps
    - **Runtime** (`runtime/tests.rs`, 2KB): basic runtime integration tests
    - **Codegen** (`codegen/comprehensive_tests.rs`, 31KB): full codegen pipeline tests
    - ##### 2.2.4.1 Missing Test Areas
      - **Lexer gaps:** No test for multi-line strings, no test for block comments (`/* */`), no test for nested block comments, no test for Unicode identifiers, no escape sequence test (`\n`, `\t`, `\\`)
      - **Parser gaps:** No test for `parse_enum()`, no test for `parse_trait()`, no test for `parse_impl()`, no test for `parse_match()` with complex patterns, no test for `parse_spawn()`, no test for `parse_select()`, no test for error recovery (multiple errors in one file)
      - **Semantic gaps:** No test for borrow checker interaction with closures, no test for lifetime inference across function calls, no test for trait coherence (overlapping impl blocks)
      - ###### 2.2.4.1.1 Priority Test Additions for Deployment
        - P0: `parse_import()` with double-colon paths — critical for multi-file compilation
          - **2.2.4.1.1.1** Test: `test_parse_import_double_colon`
            - Input: `import helios::knowledge::KnowledgeStore`
            - Expected: `Item::Import(ImportDecl { path: ["helios", "knowledge"], items: ["KnowledgeStore"] })`
            - **2.2.4.1.1.1.1** Edge Case Variants to Test
              - `import helios::knowledge` → module import, `items: []`
              - `import helios::knowledge::{KnowledgeStore, Query}` → multi-item `items: ["KnowledgeStore", "Query"]`
              - `import helios::knowledge::*` → wildcard — requires `ImportKind::Wildcard` variant
              - **2.2.4.1.1.1.1.1** Wildcard Import AST Extension
                - Add `import_kind: ImportKind` field to `ImportDecl`
                - `enum ImportKind { Named(Vec<String>), Wildcard, Module }`
                - Parser change: after `::`, check for `Star` token → `Wildcard`
                - **2.2.4.1.1.1.1.1.1** Impact on Semantic Analysis
                  - Wildcard must resolve all `pub` symbols from target module
                  - `resolve_import()` enumerates module's exported types and functions
                  - Name collision: if wildcard brings in `Query` and local has `Query` → emit E102 DuplicateDefinition
                  - Shadowing rule: local definition wins, emit W001 warning
        - P0: `compile_statement()` for `match` with `enum` variants — knowledge.omni uses this
          - **2.2.4.1.1.2** Test: `test_compile_match_enum`
            - Input: `enum Color: Red, Green, Blue` + `match c: Color::Red: println("red")`
            - **2.2.4.1.1.2.1** Expected Bytecode Sequence
              - `LoadLocal(c_slot)` → `Push(Int(0))` → `Eq` → `JumpIfNot(green_addr)` → `Push("red")` → `PrintLn` → `Jump(end)`
              - **2.2.4.1.1.2.1.1** Wildcard Pattern `_` Compilation
                - Emits NO comparison — always matches
                - Must be last arm; warn if `_` not last and subsequent arms are unreachable
                - **2.2.4.1.1.2.1.1.1** Exhaustiveness Check Integration
                  - If all enum variants covered explicitly → `_` is redundant, emit W002 warning
                  - If not all variants covered and no `_` → emit E107 exhaustiveness failure
                  - Count variants from `EnumDecl.variants.len()`, count match arms, compare
        - P0: `compile_expression()` for method calls (`obj.method(args)`) — every framework file uses this
          - **2.2.4.1.1.3** Test: `test_compile_method_call`
            - Input: `obj.len()` where `obj: String`
            - **2.2.4.1.1.3.1** Current Emission: `LoadLocal(obj) → CallNamed("len", 1)` — incorrect, looks up global "len"
              - **2.2.4.1.1.3.1.1** Fix: Name-Mangled Dispatch
                - Resolve `obj` type via `expr.resolved_type` → e.g., `String`
                - Emit `CallNamed("String__len", 1)` — mangled name
                - **2.2.4.1.1.3.1.1.1** Type Info Propagation Pipeline
                  - `SemanticAnalyzer` annotates `Expression::MethodCall` with `resolved_type: Type`
                  - `BytecodeCompiler` reads annotation to build mangled name
                  - Add `resolved_type: Option<Type>` field to all `Expression` variants
                  - Fallback: if type unknown (generic), emit `CallNamed("len", 1)` + runtime dispatch
        - P1: `compile_expression()` for closure/lambda `|x| x + 1` — used in capability executors
          - **2.2.4.1.1.4** Test: `test_compile_closure`
            - Input: `let add_one = |x| x + 1; add_one(5)`
            - **2.2.4.1.1.4.1** Expected Compilation
              - Closure body compiled as separate `CompiledFunction` with 1 param
              - `Push(func_index)` → `StoreLocal(add_one_slot)`
              - Call: `LoadLocal(add_one_slot)` → `Push(5)` → `Call(1)`
              - **2.2.4.1.1.4.1.1** Closure Capture Mechanics
                - If closure captures outer `y`: `let y=10; let f = |x| x + y`
                - Emit `LoadLocal(y_slot)` → `NewClosure(func_idx, 1)` (1 captured var)
                - New opcodes: `NewClosure(func_idx, capture_count)`, `LoadUpValue(idx)`
                - **2.2.4.1.1.4.1.1.1** UpValue Lifecycle
                  - On closure creation: copy captured values from stack frame into closure object
                  - On closure call: push captured values into new frame's upvalue slots
                  - On function exit: `CloseUpValue` moves stack locals to heap if captured

### 2.3 Compiler Public API

- [ ] **Audit `compiler/src/lib.rs`** (767 bytes — very small)
  - This file should re-export the key modules needed by tools and tests:
    ```rust
    pub mod lexer;
    pub mod parser;
    pub mod semantic;
    pub mod codegen;
    pub mod runtime;
    pub mod diagnostics;
    pub mod brain;
    ```
  - **If missing:** Add re-exports so that external tests and tools can use `omni_compiler::runtime::vm::OmniVM` etc.
  - **Verification:** After editing, `cargo doc --no-deps` generates docs for all public items.
  - #### 2.3.1 Required Public Types
    - `lexer::tokenize(source: &str) -> Result<Vec<Token>, LexerError>` — the main entry point
    - `lexer::TokenKind` — enum with 100+ variants (keywords, operators, literals, etc.)
    - `lexer::Token` — struct with `kind`, `lexeme`, `line`, `column`, `span`
    - `parser::Parser::new(tokens).parse_module()` — returns `Result<Module, ParseError>`
    - `parser::ast::*` — all 43 AST node types (Module, Item, Statement, Expression, etc.)
    - `semantic::SemanticAnalyzer` — main type checker
    - `runtime::vm::OmniVM` — bytecode VM
    - `runtime::bytecode::{OpCode, Value, OvmModule, CompiledFunction}` — bytecode types
    - `runtime::bytecode_compiler::BytecodeCompiler` — AST→bytecode
    - ##### 2.3.1.1 Visibility Audit
      - Check every `struct`/`enum`/`fn` in the above modules has `pub` visibility
      - Specifically: `BytecodeCompiler.compile_module()` must be `pub` (currently may be `pub(crate)`)
      - `OmniVM.execute()` must be `pub`
      - `OvmModule.serialize()` and `OvmModule.deserialize()` must be `pub`
      - ###### 2.3.1.1.1 Internal vs Public Split
        - Move internal helpers (like `BytecodeCompiler.emit()`, `BytecodeCompiler.patch_jump()`) to `pub(crate)`
        - Keep only the following as fully `pub`: `new()`, `compile_module()`, `compile_function()`
        - **2.3.1.1.1.1** Complete Internal Method Inventory
          - `emit(op: OpCode)` → `pub(crate)` — appends to instruction vec
          - `emit_jump(op: OpCode) -> usize` → `pub(crate)` — emits jump, returns patch address
          - `patch_jump(addr: usize)` → `pub(crate)` — patches placeholder jump target
          - `push_scope()` / `pop_scope()` → `pub(crate)` — scope management
          - `declare_local(name: &str) -> usize` → `pub(crate)` — local variable slot allocation
          - `resolve_local(name: &str) -> Option<usize>` → `pub(crate)` — name resolution
          - **2.3.1.1.1.1.1** Scope Depth Tracking
            - Each `Scope` struct: `{ base: usize, locals: HashMap<String, usize>, depth: usize }`
            - Max scope depth: enforce limit of 256 (prevent stack overflow in deeply nested code)
            - On `push_scope()`: increment depth, check limit, error if exceeded
            - **2.3.1.1.1.1.1.1** Error on Excessive Nesting
              - Return `CompileError::Codegen("Maximum nesting depth exceeded (256)")`
              - This catches malicious/malformed input with 1000+ nested blocks
              - Log warning at depth 128: "deeply nested code detected, consider refactoring"
              - **2.3.1.1.1.1.1.1.1** Diagnostic Event Hook
                - **2.3.1.1.1.1.1.1.1.1** Telemetry Payload Generation for IDE Warning
    - ##### 2.3.1.2 Re-Export Convenience Functions
      - Add top-level `pub fn compile(source: &str) -> Result<OvmModule, CompileError>` that chains tokenize → parse → analyze → bytecode
      - Add top-level `pub fn run(source: &str) -> Result<(), RuntimeError>` that compiles and executes
      - ###### 2.3.1.2.1 CompileError Enum
        ```rust
        pub enum CompileError {
            Lex(LexerError),
            Parse(Vec<ParseError>),
            Semantic(Vec<SemanticError>),
            Codegen(anyhow::Error),
        }
        ```
        - **2.3.1.2.1.1** Error Aggregation Interface
          - **2.3.1.2.1.1.1** Multi-Phase Error Collection
            - **2.3.1.2.1.1.1.1** Iterator Adaptors for Diagnostic Streams
              - **2.3.1.2.1.1.1.1.1** First-Fatal Exit Threshold Handling

### 2.4 Error Reporting Enhancement

- [ ] **Audit `compiler/src/diagnostics.rs`** (12KB)
  - Check if it provides Rust-style error diagnostics with:
    - Source spans (line:col)
    - Colored output
    - Suggestion text
    - Multi-file traces
  - #### 2.4.1 Current diagnostics.rs Analysis
    - File defines diagnostic structures and rendering
    - ##### 2.4.1.1 Existing Diagnostic Types
      - Check for: `Diagnostic`, `DiagnosticLevel`/`Severity`, `SourceSpan`, `Suggestion`
      - If `Diagnostic` struct exists, verify fields: `severity`, `message`, `span`, `suggestions`, `notes`
      - If missing, add:
        ```rust
        pub struct Diagnostic {
            pub severity: Severity,
            pub code: String,         // e.g., "E001"
            pub message: String,
            pub span: SourceSpan,
            pub suggestions: Vec<Suggestion>,
            pub notes: Vec<String>,
            pub source_file: Option<String>,
        }
        
        pub enum Severity { Error, Warning, Info, Hint }
        
        pub struct SourceSpan {
            pub start_line: usize,
            pub start_col: usize,
            pub end_line: usize,
            pub end_col: usize,
        }
        
        pub struct Suggestion {
            pub message: String,
            pub replacement: Option<String>,
            pub span: SourceSpan,
        }
        ```
      - ###### 2.4.1.1.1 Error Code Registry
        - Parser: E001 (unexpected token), E002 (unclosed paren), E003 (unclosed string), E004 (unexpected EOF), E005 (unclosed bracket), E006 (invalid syntax), E007 (expected item), E008 (missing token), E009 (too many errors)
        - Semantic: E100 (type mismatch), E101 (undefined symbol), E102 (duplicate definition), E103 (borrow error), E104 (move error), E105 (lifetime error), E106 (missing return), E107 (exhaustiveness failure)
        - Runtime: R001 (stack overflow), R002 (division by zero), R003 (null reference), R004 (type coercion failure), R005 (permission denied)
        - **2.4.1.1.1.1** Error Code Data Structure
          - `struct ErrorCode { code: &'static str, category: &'static str, severity: Severity, message_template: &'static str }`
          - Store as `const ERROR_REGISTRY: &[ErrorCode]` — static lookup table
          - Allow `--explain E100` CLI flag to print detailed explanation of an error code
          - **2.4.1.1.1.1.1** Error Explanation Texts
            - Each error code gets a multi-paragraph explanation file in `docs/errors/E100.md`
            - Format: description, common causes, examples of incorrect code, fix suggestions
            - Embedded into binary via `include_str!()` or fetched from docs directory
            - **2.4.1.1.1.1.1.1** Example: E100 Type Mismatch
              - "Error E100 occurs when the compiler expects one type but finds another."
              - Common causes: wrong function return type, mismatched assignment, arithmetic on non-numeric
              - Fix patterns: add explicit cast, change variable type, use conversion function
            - **2.4.1.1.1.1.1.2** Example: E103 Borrow Error
              - "Error E103 occurs when a value is borrowed mutably while an immutable borrow exists."
              - Common causes: iterating while mutating, aliased mutable references
              - Fix patterns: clone the value, restructure to avoid overlap, use `RefCell` pattern
              - **2.4.1.1.1.1.1.2.1** IDE Quick-Fix Code Actions Generation
          - **2.4.1.1.1.1.2** Error Code Range Conventions
            - E001-E099: Lexer/Parser errors (syntax)
            - E100-E199: Semantic errors (type system, scoping)
            - E200-E299: Borrow checker / ownership errors
            - E300-E399: Module resolution errors
            - W001-W099: Warnings (unused variables, unreachable code, deprecated features)
            - R001-R099: Runtime errors (VM execution)
            - **2.4.1.1.1.1.2.1** Reserved Ranges for Plugins
              - **2.4.1.1.1.1.2.1.1** P001-P999 Allocation Namespace
      - ###### 2.4.1.1.2 Rendering Pipeline
        - `Diagnostic` → `render_diagnostic(source: &str, diag: &Diagnostic) -> String`
        - Outputs:
          ```
          error[E100]: Type mismatch: expected Int, got String
           --> main.omni:15:10
            |
          15 |     let x: Int = "hello"
            |                  ^^^^^^^ expected Int
            |
            = help: use `int("hello")` to parse a string as integer
          ```
        - Requires `ariadne` or `codespan-reporting` crate, or custom implementation
        - **2.4.1.1.2.1** Rendering Function Internals
          - Step 1: Extract source line(s) from `source` using `span.start_line..=span.end_line`
          - Step 2: Build gutter (line numbers, left-padded to max line number width)
          - Step 3: Build underline carets `^^^^^^^` from `span.start_col` to `span.end_col`
          - Step 4: Colorize: errors=red, warnings=yellow, info=blue, hints=green (using `termcolor` or ANSI codes)
          - Step 5: Append suggestions as `= help:` or `= note:` lines
          - **2.4.1.1.2.1.1** Multi-Span Diagnostics
            - Some errors reference multiple locations (e.g., "first defined here" + "redefined here")
            - `Diagnostic.related_spans: Vec<(SourceSpan, String)>` — secondary locations with labels
            - Render each related span indented under the primary span
            - **2.4.1.1.2.1.1.1** Cross-File Spans
              - If error spans two files: `main.omni:10:5` and `utils.omni:20:3`
              - Render both files' source lines with `-->` prefix showing file path
              - Requires `source_map: HashMap<String, String>` mapping file paths to source text
              - Load on demand when multi-file module compilation is active
              - **2.4.1.1.2.1.1.1.1** Virtual File System Abstraction Boundary
          - **2.4.1.1.2.1.2** Terminal Width Adaptation
            - Detect terminal width via `terminal_size` crate
            - If line exceeds width, truncate with `...` and show column range
            - In non-TTY mode (piped output): disable colors, use plain text markers
            - **2.4.1.1.2.1.2.1** Hardcoded Fallback Width
              - **2.4.1.1.2.1.2.1.1** 80-Column Display Truncation Rule
    - ##### 2.4.1.2 Integration with Parser Error Recovery
      - `parser/mod.rs` has `record_error()` (line 114) that accumulates errors
      - After `parse_module()`, collected errors are in `parser.errors: Vec<ParseError>`
      - Each `ParseError` contains `line`, `column`, `hint` fields
      - **Connect:** Convert each `ParseError` to a `Diagnostic` and render
      - ###### 2.4.1.2.1 Parser Hint System
        - `Parser::suggest_hint()` (line 175) maps common typos:
          - "stuct" → "did you mean `struct`?"
          - "functoin" → "did you mean `fn`?"
          - "reutrn" → "did you mean `return`?"
        - Extend with more common typos from Omni syntax
        - **2.4.1.2.1.1** Typo Detection Algorithm
          - Compare unrecognized identifier against all keywords using Levenshtein distance
          - If distance ≤ 2: suggest "did you mean `keyword`?"
          - Implementation: `fn levenshtein(a: &str, b: &str) -> usize` — O(n*m) DP
          - **2.4.1.2.1.1.1** Keyword Similarity Table (precomputed)
            - Store sorted keywords in a `Vec<&str>` — binary search for exact match first
            - On miss: linear scan with distance check (58 keywords, fast enough)
            - Threshold: distance 1 for keywords ≤ 4 chars, distance 2 for keywords > 4 chars
            - **2.4.1.2.1.1.1.1** False Positive Prevention
              - Don't suggest keyword if it would be syntactically invalid in context
              - Example: don't suggest `fn` if current position expects an expression
              - Use parser state (`expecting_item`, `expecting_expression`) to filter suggestions
              - **2.4.1.2.1.1.1.1.1** Contextual Keyword Allowlist Mask
          - **2.4.1.2.1.1.2** Common Omni-Specific Typos
            - `elif` vs `elsif` vs `elseif` — suggest `elif`
            - `function` → suggest `fn`
            - `string` (lowercase type) → suggest `String`
            - `integer` → suggest `Int`
            - `boolean` → suggest `Bool`
            - `lambda` → suggest `|args| body` closure syntax
            - **2.4.1.2.1.1.2.1** Frequency-Weighted Typo Tries
              - **2.4.1.2.1.1.2.1.1** O(1) Common Mistake Lookup Table
  - **Cross-reference with** `compiler/src/semantic/error_recovery.rs` (11KB) — this may already have recovery logic.
  - **Verification:** Compile a file with deliberate type errors and verify output includes line numbers, suggestions, and colored severity labels.

### 2.5 Lexer Deep Analysis

- [ ] **Audit `compiler/src/lexer/mod.rs`** (603 LOC)
  - #### 2.5.1 Token Types (TokenKind Enum, lines 24-280)
    - **Keywords (58+):** `module`, `struct`, `fn`, `let`, `mut`, `if`, `else`, `for`, `while`, `loop`, `return`, `break`, `continue`, `match`, `enum`, `trait`, `impl`, `import`, `use`, `pub`, `self`, `Self`, `const`, `type`, `as`, `in`, `true`, `false`, `async`, `await`, `unsafe`, `extern`, `where`, `yield`, `spawn`, `select`, `comptime`, `var`, `defer`, `try`, `catch`, `finally`, `elif`, `pass`, `case`, `macro`, `shared`, `channel`, `dyn`, `Some`, `Ok`, `Err`, `None`
    - **Operators:** `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`/`and`, `||`/`or`, `!`/`not`, `=`, `+=`, `-=`, `*=`, `/=`, `..`, `::`, `->`, `=>`, `|>` (pipe), `&`, `&mut`, `?`
    - **Delimiters:** `(`, `)`, `{`, `}`, `[`, `]`, `:`, `,`, `.`, `;`, `@`, `#`
    - **Literals:** `IntLiteral`, `FloatLiteral`, `StringLiteral`, `Identifier`
    - **Whitespace:** `Newline`, `Indent`, `Dedent`, `Comment`, `BlockComment`
    - ##### 2.5.1.1 Indentation Handling
      - Omni uses Python-style significant indentation (`:` + newline + indent block)
      - `tokenize()` function (lines 349-484) manages an indent stack: `indent_stack: Vec<usize>`
      - After each `Newline`, measure leading spaces → compare to `indent_stack.last()`
      - If greater → push new level, emit `Indent` token
      - If less → pop levels until match, emit `Dedent` tokens for each pop
      - If equal → no indent/dedent tokens
      - **`next_nonblank_indent()`** (lines 328-347) skips blank lines to find the *next* meaningful indent level
      - ###### 2.5.1.1.1 Known Edge Cases
        - Mixed tabs and spaces: currently treats tab as 1 character — should be 4 or configurable
          - **2.5.1.1.1.1** Tab Width Configuration
            - Add `tab_width: usize` parameter to `tokenize()` function (default: 4)
            - Before measuring indent, expand tabs: `indent = line.chars().take_while(|c| c.is_whitespace()).map(|c| if c == '\t' { tab_width } else { 1 }).sum()`
            - **2.5.1.1.1.1.1** Configurable via CLI Flag
              - `omnc compile --tab-width 4 file.omni`
              - Store in compiler options struct: `CompilerOptions { tab_width: usize, max_errors: usize, ... }`
              - **2.5.1.1.1.1.1.1** File-Level Override
                - Support `# omni: tab-width=4` pragma comment at file top
                - Lexer scans first line for pragma before processing
                - **2.5.1.1.1.1.1.1.1** Pragma Pre-processor Phase
        - Empty blocks (`pass` keyword): must emit at least one statement inside indent block
          - **2.5.1.1.1.2** Pass Statement Handling
            - `pass` token is lexed as `TokenKind::Pass`
            - Parser converts to `Statement::Pass` which emits `OpCode::Nop`
            - **2.5.1.1.1.2.1** Interaction with Block Detection
              - After `:` + `Indent`, parser expects at least one statement
              - If next token is `Dedent` without any statement → parser error
              - `pass` satisfies the "at least one statement" requirement
              - **2.5.1.1.1.2.1.1** AST Pruning Trigger
                - **2.5.1.1.1.2.1.1.1** Omission in Release Code Gen
        - Nested indent levels > 20: no depth limit check — could stack overflow on malformed input
          - **2.5.1.1.1.3** Depth Limit Implementation
            - Add `const MAX_INDENT_DEPTH: usize = 100;`
            - In `tokenize()`, before pushing to `indent_stack`, check `indent_stack.len() < MAX_INDENT_DEPTH`
            - On exceed: return `LexerError::IndentTooDeep { depth: n, max: MAX_INDENT_DEPTH }`
            - **2.5.1.1.1.3.1** Error Recovery on Deep Indent
              - Don't abort — record error, clamp to max depth, continue lexing
              - This allows the parser to still provide useful errors for the rest of the file
              - **2.5.1.1.1.3.1.1** Synthesized Dedent Injection
                - **2.5.1.1.1.3.1.1.1** Graceful Parsing Degradation State
        - Trailing whitespace on blank lines: `next_nonblank_indent()` handles this correctly
      - ###### 2.5.1.1.2 Verification Test Cases
        - File with 5+ nesting levels → correct Indent/Dedent count
          - **2.5.1.1.2.1** Test: `test_deep_nesting_indent_dedent`
            - Input: 6 nested `if` blocks, each indented 4 more spaces
            - Expected: 6 `Indent` tokens, then at EOF 6 `Dedent` tokens
            - Assert: `tokens.iter().filter(|t| t.kind == Indent).count() == 6`
            - **2.5.1.1.2.1.1** Token Stream Assertion Helper
              - Create `fn assert_token_sequence(tokens: &[Token], expected: &[TokenKind])` utility
              - Compare only `kind` fields, ignoring lexemes — simplifies test assertions
              - **2.5.1.1.2.1.1.1** Diff Output Formatter
                - **2.5.1.1.2.1.1.1.1** Myers Diff Output on Failure
        - File ending without final newline → proper Dedent emission at EOF
          - **2.5.1.1.2.2** Test: `test_no_trailing_newline`
            - Input: `"fn foo():\n    pass"` (no trailing `\n`)
            - Expected: tokens end with `..., Pass, Dedent, EOF`
            - Tokenizer must emit synthetic `Newline` + `Dedent` before `EOF` if indent stack is non-empty
            - **2.5.1.1.2.2.1** EOF Peek Normalization
              - **2.5.1.1.2.2.1.1** Synthetic Token Allocation
                - **2.5.1.1.2.2.1.1.1** Span Propagation from Previous Node
        - File with only comments → no items but no crash
          - **2.5.1.1.2.3** Test: `test_comment_only_file`
            - Input: `"# this is a comment\n# another comment\n"`
            - Expected: `[Comment, Newline, Comment, Newline, EOF]`
            - Indent stack remains `[0]` — no Indent/Dedent emitted
            - **2.5.1.1.2.3.1** Empty File Optimization
              - **2.5.1.1.2.3.1.1** Fast-Path Return Vector
                - **2.5.1.1.2.3.1.1.1** Zero-Allocation Empty Check
    - ##### 2.5.1.2 String Literal Handling
      - `#[regex(r#""([^"\\]|\\.)*""#)]` StringLiteral — double-quoted with backslash escapes
      - **Missing:** No raw strings (`r"...""`), no multi-line strings (`"""..."""`), no string interpolation (`f"...{expr}..."`)
      - Escape sequences supported by regex: `\\`, `\"`, `\n`, `\t` — but these are NOT processed during lexing; they remain as literal `\n` in the lexeme string
      - **Action needed:** Add an `unescape()` function that processes escape sequences after lexing
      - ###### 2.5.1.2.1 unescape() Implementation
        ```rust
        fn unescape(s: &str) -> String {
            let mut result = String::new();
            let mut chars = s[1..s.len()-1].chars(); // strip quotes
            while let Some(c) = chars.next() {
                if c == '\\' {
                    match chars.next() {
                        Some('n') => result.push('\n'),
                        Some('t') => result.push('\t'),
                        Some('\\') => result.push('\\'),
                        Some('"') => result.push('"'),
                        Some('0') => result.push('\0'),
                        Some(other) => { result.push('\\'); result.push(other); }
                        None => result.push('\\'),
                    }
                } else { result.push(c); }
            }
            result
        }
        ```
        - **2.5.1.2.1.1** Where to Call unescape()
          - In `BytecodeCompiler::compile_expression()`, when emitting `Push(Value::String(s))`
          - Transform: `Push(Value::String(unescape(&literal.lexeme)))` instead of raw lexeme
          - **2.5.1.2.1.1.1** Unicode Escape Support
            - Add `\u{XXXX}` for Unicode codepoints: `Some('u') => { parse \u{hex_digits} }`
            - Parse hex digits between `{` and `}`, convert to `char`
            - Validate: codepoint must be valid Unicode scalar value (0..=0x10FFFF, excluding surrogates)
            - **2.5.1.2.1.1.1.1** Error on Invalid Escape
              - Unknown escape `\q` → emit warning W003: "unknown escape sequence '\q', treating as literal"
              - Invalid Unicode `\u{FFFFFFFF}` → emit error E003: "invalid Unicode codepoint"
              - Truncated escape at end of string `"hello\` → emit error E003: "unterminated escape"
              - **2.5.1.2.1.1.1.1.1** Diagnostics Spanning for Escapes
        - **2.5.1.2.1.2** Multi-Line String Implementation (Future)
          - Regex: `r#""""([^"]*|"[^"]|""[^"])*""""#` — triple-quoted strings
          - Strips common leading whitespace (like Python `textwrap.dedent`)
          - Does NOT process escape sequences (raw-ish multi-line)
          - **2.5.1.2.1.2.1** Dedent Algorithm for Multi-Line Strings
            - Find minimum leading whitespace on non-empty lines (excluding first line)
            - Strip that many characters from the start of each line
            - Strip leading newline if first line is empty
            - **2.5.1.2.1.2.1.1** Tab vs Space Dedent Consistency Check
              - **2.5.1.2.1.2.1.1.1** Heterogenous Indent Warning W005
    - ##### 2.5.1.3 Numeric Literal Handling
      - `#[regex(r"[0-9]+")]` IntLiteral — decimal integers only
      - `#[regex(r"[0-9]+\.[0-9]+")]` FloatLiteral — requires digits on both sides of `.`
      - **Missing:** No hex (`0xFF`), no octal (`0o77`), no binary (`0b1010`), no underscore separators (`1_000_000`), no scientific notation (`1.5e10`)
      - ###### 2.5.1.3.1 Extended Numeric Regex
        ```
        IntLiteral: r"0[xX][0-9a-fA-F][0-9a-fA-F_]*|0[oO][0-7][0-7_]*|0[bB][01][01_]*|[0-9][0-9_]*"
        FloatLiteral: r"[0-9][0-9_]*\.[0-9][0-9_]*([eE][+-]?[0-9]+)?"
        ```
        - **2.5.1.3.1.1** Numeric Parsing in Bytecode Compiler
          - After lexing, `IntLiteral` lexeme is a string like `"0xFF"` or `"1_000"`
          - Must be parsed to `i64` in `compile_expression()` before emitting `Push(Value::Int(n))`
          - **2.5.1.3.1.1.1** Parse Function
            - `fn parse_int_literal(s: &str) -> Result<i64, ParseIntError>`
            - Strip underscores: `s.replace('_', "")`
            - Detect prefix: `0x` → `i64::from_str_radix(&stripped[2..], 16)`
            - `0o` → `i64::from_str_radix(&stripped[2..], 8)`
            - `0b` → `i64::from_str_radix(&stripped[2..], 2)`
            - No prefix → `stripped.parse::<i64>()`
            - **2.5.1.3.1.1.1.1** Overflow Handling
              - If value exceeds `i64::MAX`: emit error E010: "integer literal too large"
              - Suggest: "use BigInt for arbitrary-precision integers"
              - For hex: warn if > 16 hex digits (likely accidental repetition)
              - **2.5.1.3.1.1.1.1.1** BigInt Fallback Parser Path
          - **2.5.1.3.1.1.2** Float Parsing
            - `fn parse_float_literal(s: &str) -> Result<f64, ParseFloatError>`
            - Strip underscores, parse with `f64::from_str()`
            - Handle `NaN` and `Infinity` as special keywords, not literals
            - **2.5.1.3.1.1.2.1** Precision Warning
              - If float literal has > 17 significant digits: emit W004 "float precision loss"
              - **2.5.1.3.1.1.2.1.1** Exact Literal Round-Trip Verification

### 2.6 Parser Deep Analysis

- [ ] **Audit `compiler/src/parser/mod.rs`** (1619 LOC, 66 functions)
  - #### 2.6.1 Parser Architecture
    - Hand-written recursive descent parser (NOT lalrpop-generated)
    - `Parser` struct holds: `tokens: Vec<Token>`, `pos: usize`, `errors: Vec<ParseError>`, `error_count: usize`, `panic_mode: bool`
    - Error limit (`MAX_ERRORS`): stops parsing after N errors (prevents cascading error flood)
    - ##### 2.6.1.1 Core Navigation Methods
      - `peek()` (line 203) — returns `Option<&Token>` at current position
      - `peek_kind()` (line 207) — returns `Option<&TokenKind>`
      - `advance()` (line 211) — moves position forward, skips `Comment` tokens
      - `expect()` (line 220) — advance and verify token kind, else return `ParseError`
      - `skip_newlines()` (line 243) — consume all consecutive `Newline` tokens
      - ###### 2.6.1.1.1 Lookahead Limitation
        - Parser uses single-token lookahead only (`peek()`)
        - No `peek_nth(n)` function — some grammar rules may need 2-token lookahead
        - Example: distinguishing `let x: Int = 5` vs `let x = 5` requires seeing past identifier to `:` or `=`
        - Currently handled by trying `:` first, falling back if not found
        - **2.6.1.1.1.1** Adding `peek_nth(n)` Support
          - Implement `fn peek_nth(&self, n: usize) -> Option<&Token>` — returns token at `pos + n`
          - Must skip `Comment` tokens in the lookahead (same as `advance()` does)
          - **2.6.1.1.1.1.1** Use Cases Requiring 2+ Lookahead
            - `let x: Int` vs `let x =` — need to see past `x` to `:` or `=`
            - `foo::bar` (path) vs `foo:` (label) — need to see past `foo` to `::` or `:`
            - `f(x)` (call) vs `f (x + y)` (application with parens) — check whitespace before `(`
            - **2.6.1.1.1.1.1.1** Performance Impact
              - `peek_nth` is O(n) with comment skipping — acceptable for n ≤ 3
              - Alternative: pre-filter comments from token stream before parsing
              - Pre-filtering would make `pos` indices invalid for error reporting — keep lazy skip
              - **2.6.1.1.1.1.1.1.1** Benchmark
                - Measure parse time with/without `peek_nth` on helios-framework (800+ LOC files)
                - Expected overhead: < 1% (lookahead used only in ambiguous positions)
                - **2.6.1.1.1.1.1.1.1.1** Flamegraph Micro-Benchmarking Suite
    - ##### 2.6.1.2 Error Recovery Mechanism
      - `record_error()` (line 114) — pushes error, increments count, checks limit
      - `synchronize()` (line 124-144) — panic-mode recovery: advance tokens until sync point
      - `is_sync_token()` (line 146-173) — sync points: `Fn`, `Struct`, `Enum`, `Trait`, `Impl`, `Import`, `Const`, `Type`, `Pub`, `Extern`, `Comptime`, `Macro`, `Dedent`, or EOF
      - `suggest_hint()` (line 175-201) — common typo suggestions
      - ###### 2.6.1.2.1 Recovery Quality
        - After synchronization, parsing continues from the next valid item
        - Multiple errors can be reported in a single pass
        - `parse_module()` catches errors per-item and continues
        - `parse_block()` catches errors per-statement and continues
  - #### 2.6.2 Item Parsing (Top-Level Constructs)
    - `parse_item()` (line 316-358) dispatches on token kind:
      - `Module` → `parse_module_decl()`
      - `Struct` → `parse_struct()`
      - `Fn` → `parse_function()`
      - `Trait` → `parse_trait()`
      - `Impl` → `parse_impl()`
      - `Import` → `parse_import()`
      - `Const` → `parse_const()`
      - `Enum` → `parse_enum()`
      - `Type` → `parse_type_alias()`
      - `Extern` → `parse_extern()`
      - `Comptime` → `parse_comptime()`
      - `Macro` → (not yet dispatched — potential gap)
      - `Hash` → parse attribute `#[...]`, then retry item parsing
    - ##### 2.6.2.1 Struct Parsing (`parse_struct`, lines 395-434)
      - Expects: `struct Name:` + INDENT + field declarations + DEDENT
      - Each field: `name: Type` via `parse_field()`
      - Methods: `fn name(...)` inside the struct block via `parse_method()`
      - **Gap:** No generic parameter parsing `struct Foo<T>:` — the generic `<T>` is not handled
      - ###### 2.6.2.1.1 Generic Struct Parsing Fix
        - After parsing struct name, check for `<` token
        - Parse comma-separated type parameters: `<T, U, V: Trait>`
        - Store in `StructDef.generic_params: Vec<GenericParam>`
        - Requires adding `GenericParam` to AST
        - **2.6.2.1.1.1** GenericParam AST Node
          - `struct GenericParam { name: String, bounds: Vec<TypeBound> }`
          - `enum TypeBound { Trait(String), Lifetime(String) }`
          - Example: `<T: Display + Clone>` → `GenericParam { name: "T", bounds: [Trait("Display"), Trait("Clone")] }`
          - **2.6.2.1.1.1.1** Struct Generic Instantiation
            - When `Foo<Int>` is used, monomorphizer creates `Foo__Int` with `T` replaced by `Int`
            - All field types in the struct that reference `T` are substituted
            - **2.6.2.1.1.1.1.1** Nested Generics
              - `Foo<Vec<Int>>` → T = Vec<Int> → mangled name `Foo__Vec_Int`
              - Recursive substitution: `field: HashMap<T, Vec<T>>` becomes `HashMap<Int, Vec<Int>>`
              - **2.6.2.1.1.1.1.1.1** Depth Limit for Nested Generics
                - Max generic nesting depth: 8 levels (e.g., `Vec<Vec<Vec<...>>>` up to 8 deep)
                - Beyond 8: emit E200 "generic type nesting too deep"
                - Prevents infinite expansion from recursive type definitions
                - **2.6.2.1.1.1.1.1.1.1** Semantic Cycle Breaker Node Cache
    - ##### 2.6.2.2 Function Parsing (`parse_fn_inner`, lines 540-577)
      - Expects: `fn name(params) -> ReturnType:` + block
      - Parameters parsed via `parse_params()` (lines 579-624)
      - Return type: optional `->` + type annotation
      - Body: `parse_block()` for the colon-indented body
      - Attributes: passed through from `parse_item()`
      - **Supports:** `&self`, `&mut self` as first parameter (for methods)
      - ###### 2.6.2.2.1 Parameter Parsing Detail
        - Each param: `name: Type` or `name` (type inferred)
        - Ownership modifiers: `own T`, `&T`, `&mut T`, `shared T`
        - Default values: NOT supported yet — need `name: Type = default_expr`
        - Variadic args: NOT supported yet — need `*args` or `args: ...T`
        - **2.6.2.2.1.1** Variadic Call Site Expansion
          - **2.6.2.2.1.1.1** Slice Synthesis for Spread Arguments
            - **2.6.2.2.1.1.1.1** Heap Elision for Small Variadics
              - **2.6.2.2.1.1.1.1.1** Allocation Bypass Optimization
    - ##### 2.6.2.3 Import Parsing (`parse_import`, lines 1482-1500)
      - Handles: `import path::to::module`
      - Path uses `::` separator, parsed via consuming `Identifier` + `ColonColon` pairs
      - Supports: `import module::{item1, item2}` (selective import with curlies)
      - **Gap:** No `as` alias (`import foo::bar as baz`)
      - **Gap:** No wildcard (`import foo::*`)
    - ##### 2.6.2.4 Enum Parsing (`parse_enum`, lines 1514-1564)
      - Expects: `enum Name:` + INDENT + variant declarations + DEDENT
      - Each variant: `VariantName` or `VariantName(Type1, Type2)` (tuple variant) or `VariantName:` + fields (struct variant)
      - **Used by:** `experience.omni::Experience` enum, `capability.omni::CapabilityResult` enum
  - #### 2.6.3 Statement Parsing (`parse_statement`, lines 776-838)
    - Dispatches on token kind:
      - `Let` → `parse_let()` — `let name [: Type] = expr`
      - `Var` → `parse_var()` — `var name [: Type] = expr` (mutable)
      - `Return` → `parse_return()` — `return [expr]`
      - `If` → `parse_if()` — `if expr:` block `[elif expr: block]* [else: block]`
      - `For` → `parse_for()` — `for name in expr:` block
      - `While` → `parse_while()` — `while expr:` block
      - `Loop` → `parse_loop()` — `loop:` block (infinite loop)
      - `Match` → `parse_match()` — `match expr:` + indent + case arms + dedent
      - `Break` → break statement
      - `Continue` → continue statement
      - `Defer` → `parse_defer()` — defer statement for cleanup
      - `Yield` → `parse_yield()` — yield for generators/coroutines
      - `Spawn` → `parse_spawn()` — async task spawn
      - `Select` → `parse_select()` — channel select
      - `Pass` → no-op statement (placeholder)
      - `Unsafe` → unsafe block
      - Default → expression statement (expression followed by newline)
    - ##### 2.6.3.1 Match Parsing (`parse_match`, lines 1000-1028)
      - Parses: `match expr:` + INDENT + pattern-case arms + DEDENT
      - Each arm: `pattern: block` or `pattern => expr`
      - `parse_pattern()` (lines 1030-1080) handles:
        - Literal patterns: `42`, `"hello"`, `true`
        - Variable binding: `x` (catches all and binds)
        - Wildcard: `_`
        - Enum variant: `VariantName(binding)` or `VariantName { field: binding }`
        - Tuple: `(a, b, c)`
      - ###### 2.6.3.1.1 Missing Pattern Features
        - No guard clauses: `Some(x) if x > 0 =>` — need `guard: Option<Expression>` in MatchArm
          - **2.6.3.1.1.1** Guard Clause Implementation
            - In `parse_match_arm()`: after pattern, check for `if` token
            - If found: parse expression as guard condition
            - Store in `MatchArm { pattern, guard: Option<Expression>, body }`
            - **2.6.3.1.1.1.1** Guard Bytecode Emission
              - After pattern match succeeds, compile guard expression
              - Emit `JumpIfNot(next_arm)` — if guard false, try next arm
              - **2.6.3.1.1.1.1.1** Guard with Variable Bindings
                - Guard can reference pattern variables: `Some(x) if x > 0`
                - Pattern variables must be in scope during guard evaluation
                - Compile: bind `x` to local slot, then evaluate `x > 0`
                - **2.6.3.1.1.1.1.1.1** Guard Side Effects
                  - Guards MUST be pure expressions (no mutation, no function calls with side effects)
                  - Enforce: guard expression cannot contain `=`, `+=`, method calls that take `&mut self`
                  - This is checked during semantic analysis, not parsing
                  - **2.6.3.1.1.1.1.1.1.1** Purity Analyzer Pass
                    - **2.6.3.1.1.1.1.1.1.1.1** Effect Typing Invariant Assurance
                      - **2.6.3.1.1.1.1.1.1.1.1.1** I/O Function Call Ban in Guards
                        - **2.6.3.1.1.1.1.1.1.1.1.1.1** Compile-Time Rejection Trigger
        - No range patterns: `1..5 =>`
          - **2.6.3.1.1.2** Range Pattern Compilation
            - `1..5` → emit `LoadLocal(scrutinee)` → `Push(1)` → `Ge` → `LoadLocal(scrutinee)` → `Push(5)` → `Lt` → `And` → `JumpIfNot(next_arm)`
            - Inclusive range `1..=5`: use `Le` instead of `Lt`
            - **2.6.3.1.1.2.1** Range Pattern AST Node
              - `Pattern::Range { start: Expression, end: Expression, inclusive: bool }`
              - Parser: consume start literal, `..` or `..=`, end literal
              - **2.6.3.1.1.2.1.1** Half-Open Range Semantics
                - **2.6.3.1.1.2.1.1.1** Sentinel Maximum Integer Bound Inference
        - No or-patterns: `Red | Blue =>`
          - **2.6.3.1.1.3** Or-Pattern Implementation
            - `Pattern::Or(Vec<Pattern>)` — any sub-pattern matches
            - Compile: for each sub-pattern, emit match check; if any succeeds, jump to arm body
            - **2.6.3.1.1.3.1** Variable Binding Consistency
              - All branches of or-pattern must bind the same variables with the same types
              - `Some(x) | None` is INVALID (x not bound in None branch)
              - `Ok(x) | Err(x)` is valid if both x have same type
              - **2.6.3.1.1.3.1.1** Set Intersection for Variable Scopes
                - **2.6.3.1.1.3.1.1.1** Type Unification Across Disjoint Paths
        - No nested patterns: `Some(Some(x)) =>`
          - **2.6.3.1.1.4** Nested Pattern Compilation
            - Recursive: match outer pattern first, then match inner pattern on extracted value
            - `Some(Some(x))`: check outer is Some, extract inner, check inner is Some, extract x
            - **2.6.3.1.1.4.1** Bytecode for Nested Patterns
              - `LoadLocal(scrutinee)` → check tag == Some → `JumpIfNot(next)` → extract inner value → check tag == Some → `JumpIfNot(next)` → extract x → `StoreLocal(x_slot)`
              - **2.6.3.1.1.4.1.1** Depth-First Deconstruction Emission
                - **2.6.3.1.1.4.1.1.1** Register Pressure Mitigation Strategy
  - #### 2.6.4 Expression Parsing (Precedence Climbing)
    - `parse_expression()` (line 1082) calls `parse_binary(0)` which implements Pratt/precedence climbing
    - ##### 2.6.4.1 Operator Precedence (lowest to highest)
      1. `||` / `or` — precedence 1
      2. `&&` / `and` — precedence 2
      3. `==`, `!=` — precedence 3
      4. `<`, `>`, `<=`, `>=` — precedence 4
      5. `+`, `-` — precedence 5
      6. `*`, `/`, `%` — precedence 6
      7. Unary `!`/`not`, `-` — via `parse_unary()`
      8. Postfix `.`, `()`, `[]` — via `parse_postfix()`
      9. Primary: literals, identifiers, parenthesized expressions — via `parse_primary()`
    - ##### 2.6.4.2 Primary Expressions (`parse_primary`, lines 1216-1417)
      - This is the largest function (200 lines) and handles:
        - `IntLiteral`, `FloatLiteral`, `StringLiteral`, `True`, `False`, `None_`
        - `Identifier` — variable reference or function call start
        - `LeftParen` — grouped expression or tuple
        - `LeftBracket` — array literal `[1, 2, 3]`
        - `LeftBrace` — map/dict literal `{ "key": value }`
        - `Pipe` — closure `|params| body`
        - `If` — if-expression (inline ternary)
        - `Match` — match-expression
        - `SelfValue` — `self` reference
        - `Fn` — anonymous function
        - `Ampersand` — borrow `&expr` or `&mut expr`
        - `Not` — boolean negation `!expr`
        - ###### 2.6.4.2.1 Closure Parsing
          - Syntax: `|x, y| x + y` or `|x: Int| -> Int: x * 2`
          - Parsed as: consume `|`, parse params, consume `|`, parse body expression
          - **Gap:** No multi-statement closures `|x| { stmt1; stmt2; lastexpr }`
          - AST node: `Expression::Lambda { params, body, return_type }`
          - **2.6.4.2.1.1** Multi-Statement Closure Implementation
            - After `|params|`, if next token is `Colon` + `Indent`, parse as block body
            - If next token is expression start, parse as single-expression body
            - **2.6.4.2.1.1.1** Closure Type Inference
              - Parameter types: inferred from call site context where possible
              - Return type: inferred from body's last expression type
              - `|x| x + 1` used as `map(|x| x + 1)` — infer x: Int from map's type parameter
              - **2.6.4.2.1.1.1.1** Closure as Function Argument
                - When closure is passed to a generic function: unify closure param types with function's type params
                - Example: `fn map<T, U>(f: Fn(T) -> U)` called with `|x| x + 1` → T=Int, U=Int
                - **2.6.4.2.1.1.1.1.1** Trait-Based Closure Types
                  - `Fn(Args) -> Ret` for immutable closures
                  - `FnMut(Args) -> Ret` for closures that capture mutably
                  - `FnOnce(Args) -> Ret` for closures that consume captured values
                  - Classification determined by how closure uses captured variables
                  - **2.6.4.2.1.1.1.1.1.1** Environment Pointer Typestate Tracking
                    - **2.6.4.2.1.1.1.1.1.1.1** Call-Site Alias Analysis
                      - **2.6.4.2.1.1.1.1.1.1.1.1** Auto-Deref for Context References
                        - **2.6.4.2.1.1.1.1.1.1.1.1.1** Function Pointer Decay Fast Path
        - ###### 2.6.4.2.2 Method Call Chain
          - `parse_postfix()` handles: `expr.field`, `expr.method(args)`, `expr[index]`, `expr(args)`
          - Chains: `obj.method1().method2().field` — left-to-right associative via loop in `parse_postfix()`
          - **Critical for HELIOS:** Every `.omni` file uses method chains extensively
          - **2.6.4.2.2.1** Postfix Parsing Loop Internals
            - Loop: while next token is `.`, `(`, `[`, or `?`:
              - `.` + Identifier: field access or method call
              - `.` + Identifier + `(`: method call with args
              - `(`: direct function call (calling a callable expression)
              - `[`: index access (array, map, string)
              - `?`: try/error propagation operator
            - **2.6.4.2.2.1.1** Try Operator `?` Implementation
              - `expr?` desugars to: `match expr { Ok(v) => v, Err(e) => return Err(e) }`
              - Parser emits `Expression::Try(Box<Expression>)`
              - **2.6.4.2.2.1.1.1** Semantic Check for Try
                - Expression type must be `Result<T, E>`
                - Enclosing function must return `Result<_, E>` (same error type)
                - If not: emit E150 "cannot use ? operator: function does not return Result"
                - **2.6.4.2.2.1.1.1.1** Error Type Inference
                  - If function returns `Result<T, E>` and `?` is used on `Result<U, F>`, check `F` can convert to `E`
                  - Via `From<F> for E` trait implementation
                  - If no conversion exists: emit E151 "incompatible error types"
                  - **2.6.4.2.2.1.1.1.1.1** Auto-Box Trait Object Fallback Mechanism
                    - **2.6.4.2.2.1.1.1.1.1.1** Downcast Recovery Compatibility
                      - **2.6.4.2.2.1.1.1.1.1.1.1** V-Table Dispatch for Shared Errors
                        - **2.6.4.2.2.1.1.1.1.1.1.1.1** Runtime Type ID Type-Erased Wrapper

### 2.7 Semantic Analysis Deep Audit

- [ ] **Audit `compiler/src/semantic/mod.rs`** (2345 LOC, 92 outline items) and sub-modules
  - #### 2.7.1 SemanticAnalyzer core
    - Walks the parsed AST and produces a `TypedModule` with resolved types
    - Maintains: `scopes: Vec<Scope>`, `registered_traits: HashMap<String, TraitInfo>`, `registered_structs`, `monomorphized_functions`
    - ##### 2.7.1.1 Type System
      - Primitive types: `U8`, `U16`, `U32`, `U64`, `Usize`, `I8`, `I16`, `I32`, `I64`, `Isize`, `F32`, `F64`, `Bool`, `Str`
      - Compound types: `Array(Box<Type>, Option<Box<Expression>>)`, `Vec(Box<Type>)`, `HashMap(Box<Type>, Box<Type>)`, `Option(Box<Type>)`, `Result(Box<Type>, Box<Type>)`, `Tuple(Vec<Type>)`, `Reference(Box<Type>)`, `MutReference(Box<Type>)`, `Shared(Box<Type>)`
      - User-defined: `Named(String)`, `Generic(String, Vec<Type>)`
      - Special: `Infer` (type to be determined), `Never` (bottom type), `Void` (unit)
      - ###### 2.7.1.1.1 Type Equality (`types_equal`, lines 92-180)
        - Structural equality check, handling compound types recursively
        - `Array` equality checks both element type AND size expression (using `Debug` format comparison for size — fragile)
        - `Named` types compared by string name only — no fully-qualified name resolution
        - **Issue:** Two structs named "Config" in different modules would be considered equal
        - **2.7.1.1.1.1** Fully-Qualified Name Resolution Fix
          - Replace `Named(String)` with `Named(QualifiedName)` where `QualifiedName = Vec<String>` (module path + name)
          - Example: `helios::config::Config` vs `std::config::Config` — different types
          - **2.7.1.1.1.1.1** QualifiedName Construction
            - During semantic analysis, resolve each `Named("Config")` to its module path
            - Maintain `current_module_path: Vec<String>` in SemanticAnalyzer
            - On import: record fully-qualified path for each imported symbol
            - **2.7.1.1.1.1.1.1** Name Resolution Algorithm
              - Step 1: Check local scope for `Config` → use local definition if found
              - Step 2: Check imported symbols → use imported FQN if found
              - Step 3: Check wildcard imports → enumerate all wildcard-imported modules
              - Step 4: If still not found → emit E101 "undefined type 'Config'"
              - **2.7.1.1.1.1.1.1.1** Ambiguity Resolution
                - If `Config` found in multiple wildcard imports → emit E102 "ambiguous type 'Config'"
                - Suggest: "use `helios::config::Config` or `std::config::Config` to disambiguate"
                - **2.7.1.1.1.1.1.1.1.1** IDE Auto-Import Quick Fix Integration
          - **2.7.1.1.1.1.2** Array Size Expression Equality Fix
            - Replace `Debug` format comparison with structural AST comparison
            - `fn expr_equal(a: &Expression, b: &Expression) -> bool` — deep structural compare
            - For constant-folded sizes: compare evaluated values instead of AST nodes
            - **2.7.1.1.1.1.2.1** Const-Eval Normalization Passes
              - **2.7.1.1.1.1.2.1.1** Deterministic Compilation Hashing
    - ##### 2.7.1.2 Symbol Table
      - `Scope` struct contains: `symbols: HashMap<String, Symbol>`, `parent: Option<usize>`
      - `Symbol` contains: `name`, `sym_type: Type`, `mutable: bool`, `borrow_state: BorrowState`, `initialized: bool`, `lifetime: Option<Lifetime>`
      - Scope chain: inner scope → outer scope lookup via `parent` index
  - #### 2.7.2 Type Inference (`type_inference.rs`, 77KB)
    - Constraint-based Hindley-Milner type inference engine
    - ##### 2.7.2.1 Core Algorithm
      - Phase 1: Walk AST, generate `TypeConstraint` equations (`Equals(T1, T2)`, `TraitBound(T, "Display")`, `Subtype(T1, T2)`)
      - Phase 2: Unification — solve constraint equations using union-find
      - Phase 3: Substitution — replace `Infer` types with resolved concrete types
    - ##### 2.7.2.2 Known Limitations
      - No higher-kinded types (`T<U>` where `T` is generic and parametric)
      - No type class/trait inference for operators (hardcoded `Add`, `Sub`, etc.)
      - Return type inference: works for simple cases but may fail for recursive functions
  - #### 2.7.3 Borrow Checker (`borrow_check.rs`, 56KB)
    - Rust-style ownership and borrowing analysis
    - ##### 2.7.3.1 Borrow States
      - `Owned` — variable owns its value
      - `Moved` — value has been moved, accessing it is an error
      - `BorrowedShared(count)` — N shared borrows active
      - `BorrowedMut` — exactly one mutable borrow active
      - `PartiallyMoved(fields)` — some struct fields have been moved
    - ##### 2.7.3.2 Enforcement Rules
      - Cannot use after move (unless type is `Copy`)
      - Cannot have `&mut` and `&` borrows simultaneously
      - Cannot have two `&mut` borrows simultaneously
      - Borrow must not outlive the borrowed value
      - ###### 2.7.3.2.1 Copy vs Move Semantics
        - Copy types: `Int`, `Float`, `Bool`, `Char` (primitives)
        - Move types: `String`, `Vec`, `HashMap`, user-defined structs (unless `#[derive(Copy)]`)
        - **Issue:** No `#[derive(Copy)]` attribute handling in parser/semantic yet
        - **2.7.3.2.1.1** Derive(Copy) Implementation Plan
          - Parser: `#[derive(Copy)]` parsed as `Attribute { name: "derive", args: ["Copy"] }` on struct
          - Semantic: when struct has `derive(Copy)`, mark all its types as Copy
          - **2.7.3.2.1.1.1** Copy Validity Check
            - A struct can only derive `Copy` if ALL its fields are also `Copy`
            - If field type is `String` (non-Copy): emit E204 "cannot derive Copy: field 'name' is not Copy"
            - Recursive: if field type is another struct, check if that struct is also Copy
            - **2.7.3.2.1.1.1.1** Built-in Copy Type Registry
              - `COPY_TYPES: HashSet<&str>` containing: `Int`, `Float`, `Bool`, `Char`, `U8`..`U64`, `I8`..`I64`, `F32`, `F64`, `Usize`, `Isize`
              - Extend with user structs that have `#[derive(Copy)]` and pass validation
              - Query: `is_copy(ty: &Type) -> bool` checks registry + field recursion
              - **2.7.3.2.1.1.1.1.1** Copy Semantics in Assignment
                - For Copy types: `let y = x` creates a bitwise copy, `x` remains valid
                - For Move types: `let y = x` moves ownership, `x` becomes `BorrowState::Moved`
                - Accessing moved variable: emit E104 "use of moved value 'x'"
                - **2.7.3.2.1.1.1.1.1.1** Liveness Flow Control Traversal
  - #### 2.7.4 Monomorphization (`monomorphization.rs`, 16KB)
    - Instantiates generic functions with concrete type arguments
    - `MonomorphizedFunc { name, original_name, type_args, body }`
    - ##### 2.7.4.1 Monomorphization Process
      1. When a generic function `foo<T>()` is called with `foo::<Int>()`, create a specialized copy `foo__Int`
      2. Replace all uses of `T` with `Int` in the function body
      3. Type-check the specialized copy
      4. Generate bytecode for the specialized copy
      - ###### 2.7.4.1.1 Monomorphization Explosion Guard
        - No depth/breadth limit currently — could generate infinite copies with recursive generics
        - Add: `max_mono_depth: usize = 64`, `max_mono_count: usize = 10000`
        - **2.7.4.1.1.1** Deduplication Cache
          - Before creating a new specialization, check `monomorphization_cache: HashMap<(String, Vec<Type>), usize>`
          - Key: (original function name, concrete type args)
          - If already specialized: reuse existing function index instead of creating duplicate
          - **2.7.4.1.1.1.1** Cache Key Normalization
            - Type args must be normalized before cache lookup (e.g., `Named(["std", "Int"])` vs `Named(["Int"])` should hash the same)
            - Implement `Type::canonical()` that resolves aliases and FQNs
            - **2.7.4.1.1.1.1.1** Recursive Monomorphization Detection
              - `foo<T>()` calls `foo<Vec<T>>()` — each call creates a new specialization with deeper type
              - Track recursion depth: if `foo` is already being monomorphized, increment depth counter
              - If depth > `max_mono_depth`: emit E300 "recursive generic instantiation too deep"
              - **2.7.4.1.1.1.1.1.1** Example: Safe vs Unsafe Recursion
                - Safe: `foo<Int>()` calls `bar<Int>()` calls `baz<Int>()` — depth 1 each
                - Unsafe: `foo<T>()` calls `foo<Vec<T>>()` — generates `foo__Vec_Int`, `foo__Vec_Vec_Int`, etc.
                - Detection: track call graph during monomorphization, detect cycles with growing type args
                - **2.7.4.1.1.1.1.1.1.1** Call Graph Strongly Connected Components Analysis

### 2.8 Module Loading System

- [ ] **Check current module resolution** in the parser and compiler driver
  - Look at `compiler/src/main.rs::compile()` (lines 156-240) — how does it handle `import` statements?
  - Look at `compiler/src/parser/` for import node support.
  - **Spec §96 requires:** A module system with `mod`, `pub`, `use`, visibility rules, and search paths.
  - #### 2.8.1 Current Import Resolution
    - `parse_import()` produces `Item::Import(ImportDecl { path, items })` in the AST
    - `ImportDecl` has: `path: Vec<String>` (e.g., `["helios", "knowledge"]`), `items: Vec<String>` (selective imports)
    - **Problem:** The compiler driver (`main.rs::compile()`) does NOT recursively resolve imports — it compiles a single file
    - ##### 2.8.1.1 Multi-File Compilation Implementation
      1. Parse entry file's imports → collect all `ImportDecl` paths
      2. Resolve each path to a filesystem location:
         - `helios::knowledge` → `helios/knowledge.omni` relative to project root or `--search-path`
         - `core::io` → `omni-lang/core/io.omni`
         - `std::collections` → `omni-lang/std/collections.omni`
      3. Parse and type-check each imported module (recursively)
      4. Merge exported symbols into the importing module's scope
      5. Detect circular imports (DAG check)
      - ###### 2.8.1.1.1 Module Resolution Algorithm
        ```rust
        fn resolve_import(path: &[String], search_paths: &[PathBuf]) -> Option<PathBuf> {
            let relative = path.join("/") + ".omni";
            for sp in search_paths {
                let candidate = sp.join(&relative);
                if candidate.exists() { return Some(candidate); }
            }
            None
        }
        ```
        - **2.8.1.1.1.1** File IO During Resolution
          - `resolve_import` must probe the filesystem
          - Optimize: cache `exists()` checks in a `HashMap<PathBuf, bool>`
          - **2.8.1.1.1.1.1** Source Text Caching
            - Once resolved, read file content into memory
            - `source_files: HashMap<PathBuf, String>` shared across threads
            - **2.8.1.1.1.1.1.1** Source ID Assignment
              - Each loaded file gets a unique `SourceId: u32`
              - `Span` tokens use `SourceId` instead of cloning `PathBuf`
              - **2.8.1.1.1.1.1.1.1** O(1) Source Code Lookup
                - Global `SOURCE_MAP: RwLock<Vec<String>>` where index is `SourceId`
                - Allows diagnostic renderer to grab source lines instantly
                - **2.8.1.1.1.1.1.1.1.1** Memory-Mapped File Streaming Strategy
      - ###### 2.8.1.1.2 Default Search Paths
        - `[project_root]/` — for `helios::` imports
        - `[project_root]/omni-lang/core/` — for `core::` imports
        - `[project_root]/omni-lang/std/` — for `std::` imports
        - `[project_root]/helios-framework/` — for `helios::` framework imports
        - Configurable via `--search-path` CLI flag or `omni.toml [build] search_paths`
        - **2.8.1.1.2.1** Search Path Precedence
          - Local relative paths (`import .utils` - NOT spec compliant yet, but useful) checked first
          - Then CLI `--search-path` overrides
          - Then `omni.toml` search paths
          - Finally standard library / core paths
          - **2.8.1.1.2.1.1** Core Module Resolution bypass
            - Modules starting with `core::` skip filesystem if compiled into binary
            - **2.8.1.1.2.1.1.1** Embedded Standard Library
              - Use `include_str!()` to bake `core/lib.omni`, `core/math.omni` into compiler
              - If `core` is imported, return embedded string instead of reading file
              - **2.8.1.1.2.1.1.1.1** Virtual Filesystem Integration
                - `resolve_import` checks virtual FS first before real disk
                - Useful for REPL, online playground, and self-contained distribution
                - **2.8.1.1.2.1.1.1.1.1** Layered Overlay VFS Architecture
    - ##### 2.8.1.2 Symbol Export Rules
      - Items prefixed with `pub` are exported from a module
      - Items without `pub` are module-private
      - `import module::{item1, item2}` imports only named items
      - `import module` imports the module as a namespace (need `module.item` access)
      - **2.8.1.2.1** Cross-Module Compilation Order
        - Resolve all imports to build a directed graph of module dependencies
        - Topological sort the DAG: leaves compiled first
        - **2.8.1.2.1.1** Circular Dependency Detection
          - Tarjan's strongly connected components algorithm
          - If cycle detected (A imports B, B imports A), emit E301 "circular import"
          - **2.8.1.2.1.1.1** Breaking Import Cycles
            - Suggest user uses dynamic dispatch or extracts common items to module C
            - No "forward declaration" supported in Omni
            - **2.8.1.2.1.1.1.1** Compile-Time Cyclic Dependency Resolution
              - **2.8.1.2.1.1.1.1.1** Build Matrix Reordering Strategy
              - Cyclic data structures (A contains Option<B>, B contains Option<A>) are allowed if in same file
              - Cross-file cyclic data structures: currently rejected to simplify compiler architecture

### 2.9 Self-Hosting (Deferred — Not Blocking Deployment)

- [ ] **Document current state** of `codegen/self_hosting.rs` (30KB)
  - Read the file and document what's implemented vs. stubbed.
  - Create `docs/self-hosting-status.md` with findings.
  - **This is NOT blocking for initial deployment** — the Rust-compiled `omnc.exe` will be shipped.
  - **Long-term:** Add `omnic.omni` that implements the compiler in Omni itself.

### 2.10 Main Entry Point (`main.rs`) Deep Analysis

- [ ] **Audit `compiler/src/main.rs`** (285 LOC)
  - #### 2.10.1 CLI Commands (clap-based)
    - `omnc run <source>` — compile and execute
    - `omnc compile <source> [-o output]` — compile to .ovc bytecode
    - `omnc check <source>` — type-check only (no codegen)
    - `omnc build` — build entire project from `omni.toml`
    - `omnc test [file]` — run test functions
    - `omnc fmt [file]` — format source code
    - `omnc doc [file]` — generate documentation
    - `omnc lsp` — start language server
    - `omnc repl` — interactive REPL
    - ##### 2.10.1.1 Compile Pipeline in `compile()` (lines 156-240)
      1. Read source from file
      2. `tokenize(&source)` — lexing
      3. `Parser::new(tokens).parse_module()` — parsing
      4. Log AST items if `--verbose`
      5. `SemanticAnalyzer::new().analyze(&module)` — type checking
      6. Handle type errors: `is_hard_type_error()` filters soft vs hard
      7. `BytecodeCompiler::new().compile_module(&module)` — code generation
      8. If `run` command: `OmniVM::new().execute(&bytecode_module)` — execution
      9. If `compile` command: `module.serialize()` and write to output file
      - ###### 2.10.1.1.1 Target Selection
        - `--target ovm` — OVM bytecode (default for deployment)
        - `--target native` — LLVM native code (requires `llvm` feature flag)
        - `--target hybrid` — both OVM and native
        - Feature flags controlled by `Cargo.toml [features]`
        - **2.10.1.1.1.1** Native Target Status
          - LLVM backend is currently experimental or unmaintained
          - `compiler/src/codegen/mod.rs` stubbed but lacks full LLVM IR generation
          - **2.10.1.1.1.1.1** Missing Native Features
            - GC integration with LLVM metadata
            - Exception handling (Stack unwinding)
            - Cross-platform calling conventions
            - **2.10.1.1.1.1.1.1** Deployment Decision
              - Do not block on Native LLVM target
              - Force `--target ovm` for HeliOS deployment
              - OVM interpreter is fast enough for framework orchestrator
              - **2.10.1.1.1.1.1.1.1** Future LLVM Integration
                - Requires Inkwell crate dependency
                - Will need a dedicated LLVM pass manager configuration for optimizations

### Verification — Section 2

```powershell
# 1. Tools workspace builds
cd omni-lang\tools; cargo build --workspace
# Expected: 4 binaries in target/debug/

# 2. Compiler tests pass
cd omni-lang\compiler; cargo test
# Expected: 360+ tests pass (per status report)

# 3. Integration tests pass
cd omni-lang\compiler; cargo test --test compile_examples
# Expected: All .omni files in examples/ compile successfully

# 4. Compile helios-framework
omnc compile helios-framework/main.omni
# Expected: OVM bytecode output, no errors

# 5. Lexer-specific tests
cd omni-lang\compiler; cargo test lexer::
# Expected: 10+ lexer tests pass, including new edge case tests

# 6. Parser-specific tests
cd omni-lang\compiler; cargo test parser::
# Expected: All parse tests pass, including error recovery tests

# 7. API visibility check
cd omni-lang\compiler; cargo doc --no-deps
# Expected: Documentation generated for all pub items
```

---

## 3. OVM Runtime Consolidation

**Goal:** A single, hardened virtual machine that can execute compiled HELIOS bytecode with proper error handling, GC, and plugin isolation.

### 3.1 Clarify Canonical Runtime Location

The current plan incorrectly references `ovm/src/allocator.rs`, `ovm/src/interpreter.rs`, etc. — these files **do not exist**. The actual OVM runtime is:

| Planned Path (WRONG) | Actual Path | Size |
|---|---|---|
| `ovm/src/allocator.rs` | Does not exist | — |
| `ovm/src/interpreter.rs` | `compiler/src/runtime/interpreter.rs` | 135KB |
| `ovm/src/executor.rs` | `compiler/src/runtime/vm.rs` `OmniVM::execute()` | 67KB |
| `ovm/src/natives.rs` | `compiler/src/runtime/native.rs` | 8KB |
| `ovm/src/plugin.rs` | Does not exist | — |
| `ovm/src/error.rs` | Does not exist (errors are ad-hoc `String`) | — |

- [ ] **Decision: Keep runtime in `compiler/src/runtime/` or extract to `omni-lang/ovm/`**
  - **Recommended:** Keep in `compiler/src/runtime/` for now (avoid churn), but create a proper `OvmError` type.
  - **If extracting:** Move `vm.rs`, `bytecode.rs`, `bytecode_compiler.rs`, `interpreter.rs`, `native.rs` to a new `omni-lang/ovm/src/` crate and add it as a dependency to the compiler.
  - **3.1.1** Extraction Migration Path
    - Step 1: Create `omni-lang/ovm` Cargo workspace member
    - Step 2: Move files and update imports to `use ovm::*`
    - Step 3: Extract `OvmError` types
    - **3.1.1.1** Workspace Configuration
      - `omni-lang/Cargo.toml` must list `ovm` in `members` array
      - `compiler/Cargo.toml` must add `ovm = { path = "../ovm" }` to dependencies
      - **3.1.1.1.1** Feature Flag Pass-through
        - Standard library `no_std` support capability via `default-features = false`
        - **3.1.1.1.1.1** OVM Standalone Build
          - Allows building the VM without the compiler (smaller binary)
          - Useful for edge device deployment where only `.ovc` execution is needed
          - **3.1.1.1.1.1.1** Standalone Size Target
            - Without LLVM/Logos/Ariadne, `ovm.exe` standalone target: < 2MB stripped
            - **3.1.1.1.1.1.1.1** Build Script for Standalone
              - Add `bin` target in `ovm/Cargo.toml` that only invokes `vm::execute` on `.ovc` files
              - **3.1.1.1.1.1.1.1.1** Link-Time Optimization Configuration
                - **3.1.1.1.1.1.1.1.1.1** Thin LTO Profile for Size Reduction

### 3.2 Create Proper Error Types

- [ ] **Create `compiler/src/runtime/error.rs`** with:
  ```rust
  use thiserror::Error;
  
  #[derive(Error, Debug)]
  pub enum OvmError {
      #[error("stack underflow at instruction {instruction}")]
      StackUnderflow { instruction: usize },
      
      #[error("undefined variable: {name}")]
      UndefinedVariable { name: String },
      
      #[error("type mismatch: expected {expected}, got {actual}")]
      TypeMismatch { expected: String, actual: String },
      
      #[error("division by zero")]
      DivisionByZero,
      
      #[error("out of memory: requested {requested} bytes")]
      OutOfMemory { requested: usize },
      
      #[error("heap index out of bounds: {index}")]
      HeapOutOfBounds { index: usize },
      
      #[error("permission denied: capability '{capability}' not granted")]
      PermissionDenied { capability: String },
      
      #[error("plugin error: {message}")]
      PluginError { message: String },
      
      #[error("function not found: {name}")]
      FunctionNotFound { name: String },
      
      #[error("invalid opcode at position {position}")]
      InvalidOpcode { position: usize },
      
      #[error("maximum call depth exceeded ({depth})")]
      StackOverflow { depth: usize },
      
      #[error("IO error: {0}")]
      Io(#[from] std::io::Error),
  }
  ```
- [ ] **Add `mod error;` to `compiler/src/runtime/mod.rs`** (5.7KB)
- [ ] **Refactor `vm.rs`** to use `OvmError` instead of `Result<(), String>`:
  - `OmniVM::execute()` currently returns `Result<(), String>` (line 347)
  - Change to `Result<(), OvmError>`
  - Replace every `.ok_or("...")` and `Err(format!(...))` with typed errors
  - `OmniVM::pop()` (line 304): change `Err("Stack underflow".into())` to `Err(OvmError::StackUnderflow { instruction: self.ip })`
  - **This is ~50 replacements** across the 605-line `execute()` function
  - **3.2.1** Structured Error Context
    - Add `ip` (instruction pointer) to every error variant dynamically
    - Helps debugger pinpoint exactly which bytecode failed
    - **3.2.1.1** Call Stack Reification
      - On error, `vm` should capture the current `call_frames`
      - Append formatted stack trace to the error output
      - **3.2.1.1.1** Stack Trace Format
        - `at function_name (module_name)`
        - Need reverse lookup: IP → function name using metadata
        - **3.2.1.1.1.1** Metadata Retention
          - Output `.ovc` file must include a debug section mapping IPs to source spans and function names
          - **3.2.1.1.1.1.1** `.ovc` Debug Section Layout
            - Append chunk at EOF: `[magic][length][zlib_compressed_json_debug_info]`
            - **3.2.1.1.1.1.1.1** Stripping Debug Info
              - `omnc compile --release` skips writing debug section to minimize bytecode size
              - **3.2.1.1.1.1.1.1.1** DWARF Mapping Equivalent Schema
                - **3.2.1.1.1.1.1.1.1.1** Bytecode Address to Source Line Resolution Table

### 3.3 Audit and Harden `vm.rs`

- [ ] **Find all `panic!`, `unwrap()`, `expect()` in runtime files**
  - Run: `Select-String -Path "omni-lang\compiler\src\runtime\*.rs" -Pattern "panic!|\.unwrap\(\)|\.expect\(" | Measure-Object`
  - For each occurrence:
    - If it's in test code (`#[cfg(test)]`): leave it
    - If it's in production code: replace with `Result` + `?` operator
  - **Priority files:**
    - `vm.rs` (1790 LOC) — the GC collect (`gc_collect`, line 244-295) and heap operations
    - `interpreter.rs` (135KB) — the tree-walk interpreter, check for panics in match arms
    - `native.rs` (8KB) — native function calls
  - **Verification:** `cargo clippy -- -D warnings` produces zero warnings in runtime modules.
  - **3.3.1** Unwrap Eradication Strategy
    - Phase 1: Convert all `unwrap()` to `expect("reason")` if truly unreachable
    - Phase 2: Convert `expect()` to proper `Result` propagation if failure is possible based on user input
    - **3.3.1.1** Handling truly impossible states
      - Example: accessing local slot that bytecode compiler proved exists
      - Use `unreachable!("Compiler bug: local slot {} missing", idx)` instead of `unwrap()`
      - **3.3.1.1.1** Internal Consistency Checks
        - In debug builds only: `debug_assert!(idx < self.locals.len())`
        - Evaporates in release mode, ensuring zero performance penalty
        - **3.3.1.1.1.1** Fuzzer Integration
          - Setup `cargo-fuzz` to feed random/malformed bytecode to OVM
          - Any panic/unreachable hit by the fuzzer is a security vulnerability (sandbox escape)
          - **3.3.1.1.1.1.1** Fuzz Target Harness
            - `fuzz_target!(|data: &[u8]| { let _ = OmniVM::new().execute_bytes(data); });`
            - **3.3.1.1.1.1.1.1** Continuous Fuzzing
              - Run `libFuzzer` in a daily CI job for 1 hour to detect regressions
              - **3.3.1.1.1.1.1.1.1** Structure-Aware Bytecode Mutation
                - **3.3.1.1.1.1.1.1.1.1** Opcodes Dictionary Corpus Seeding

### 3.4 GC Safety

- [ ] **Verify tri-color mark-and-sweep in `vm.rs`** is safe:
  - `gc_collect_roots()` (line 214-242): Collects all HeapRef indices from stack, locals, globals
  - `gc_collect()` (line 244-295): Mark phase → sweep phase
  - **Check:** Does it handle circular references? (Arrays containing HeapRefs pointing to themselves)
  - **Check:** Is `alloc()` (line 193-212) safe when GC threshold is reached during an allocation?
  - #### 3.4.1 GC Algorithm Detailed Walkthrough
    - **Phase 1: Root Collection** (`gc_collect_roots`)
      - Scans `self.stack` — every `VmValue::HeapRef(idx)` → add `idx` to roots
      - Scans all `call_frames[].locals[]` — every HeapRef → add to roots
      - Scans `self.globals` HashMap values — every HeapRef → add to roots
      - ##### 3.4.1.1 Root Collection Gaps
        - Does NOT scan `self.output` (PrintLn results) — HeapRefs in output strings are lost after print, which is correct
        - Does NOT scan temporary operands being computed mid-instruction — if GC triggers between a `Pop` and a `Push`, the popped value may be collected prematurely
        - ###### 3.4.1.1.1 Mid-Instruction GC Safety Fix
          - GC should only trigger between complete instruction executions, never mid-instruction
          - Current code: `alloc()` calls `gc_collect()` during allocation, which could happen mid-instruction
          - Fix: Buffer allocations during instruction execution, trigger GC only between instructions
          - Alternative: Pin values during complex instructions using a temporary root set
          - **3.4.1.1.1.1** Allocation Buffering Implementation
            - Add `pending_allocations: usize` to VM state
            - `alloc()` increments counter. If > threshold, flag `needs_gc = true`
            - **3.4.1.1.1.1.1** Instruction Loop Check
              - `execute()` loop: `if self.needs_gc { self.gc_collect(); self.needs_gc = false; }` before fetching next opcode
              - Ensures all temporary Rust variables are dropped, leaving only VM stack/locals as roots
              - **3.4.1.1.1.1.1.1** Out of Memory Fallback
                - If single instruction allocates more than available heap without triggering GC
                - Need emergency `gc_collect()` with temporary pinning array for mid-instruction references
                - **3.4.1.1.1.1.1.1.1** Pinned Roots Array
                  - `self.pinned_roots: Vec<HeapRef>`
                  - Complex opcodes push intermediate refs to pinned array, run alloc, pop array
                  - **3.4.1.1.1.1.1.1.1.1** Pin RAII Guard
                    - Use `struct PinGuard<'a> { vm: &'a mut VM, refs: Vec<HeapRef> }`
                    - Drops refs from pin array automatically on scope exit
                    - **3.4.1.1.1.1.1.1.1.1.1** Borrow Checker Exemption Hacks
                      - **3.4.1.1.1.1.1.1.1.1.1.1** PhantomData Marker for Variance
    - **Phase 2: Mark** (inside `gc_collect`)
      - Uses tri-color algorithm: White (unmarked), Gray (discovered but children not visited), Black (fully visited)
      - All roots start as Gray, everything else starts as White
      - While Gray set is non-empty: pick a Gray object, mark it Black, add all its HeapRef children to Gray
      - ##### 3.4.1.2 HeapCell Reference Traversal
        - `HeapCell::references()` (lines 118-149) extracts HeapRef indices from nested structures
        - `HeapCell::Array(items)` → iterate `items`, collect HeapRef indices
        - `HeapCell::Map(pairs)` → iterate keys and values
        - `HeapCell::Struct(name, fields)` → iterate field values
        - `HeapCell::HeapString(s)` → no references (leaf node)
        - ###### 3.4.1.2.1 Circular Reference Handling
          - Tri-color algorithm naturally handles cycles: once an object is Black, re-encountering it is a no-op
          - Verified: `gc_collect` uses `marked: bool` flag in `GcHeader` — once marked, skip
          - **Safe for:** A→B→A cycles, self-referencing arrays, deeply nested maps
          - **3.4.1.2.1.1** Directed Graph Traversal
            - **3.4.1.2.1.1.1** Cycle Detection Instrumentation
              - **3.4.1.2.1.1.1.1** Deep Path Heuristics
                - **3.4.1.2.1.1.1.1.1** Recursion Depth Overflow Prevention
                  - **3.4.1.2.1.1.1.1.1.1** Iterative Traversal Stack Frame Allocation
    - **Phase 3: Sweep**
      - Iterate all heap entries; if `marked == false` (still White), free the cell
      - Reset all remaining `marked` flags to `false` for next cycle
      - Adjust `gc_threshold` upward if heap grew significantly (adaptive threshold)
      - ##### 3.4.1.3 Adaptive Threshold Strategy
        - Current: `gc_threshold = 256` (fixed)
        - Improvement needed: `gc_threshold = max(256, live_count * 2)` — double the live set size
        - This prevents excessive GC on programs with large stable heaps
        - **3.4.1.3.1** Heap Sizing Constants
          - `MIN_HEAP_CAPACITY = 1024` objects
          - `GROWTH_FACTOR = 2.0`
          - `MAX_HEAP_CAPACITY = 10_000_000` (configurable via env var `OMNI_MAX_HEAP`)
          - **3.4.1.3.1.1** OOM Enforcement
            - If `live_count * 2 > MAX_HEAP_CAPACITY` after full GC sweep cycle
            - Return `OvmError::OutOfMemory`
            - **3.4.1.3.1.1.1** Memory Pressure Heuristic
              - Track bytes allocated, not just object count (arrays are larger than ints)
              - Add `self.bytes_allocated` updated on every `alloc()`
              - **3.4.1.3.1.1.1.1** Object Size Calculation
                - `HeapCell` enum size is constant, but `Array` and `Map` heap-allocate Vecs/HashMaps
                - Total size = `size_of::<HeapCell>() + internal_capacity_bytes()`
                - **3.4.1.3.1.1.1.1.1** Native Resource Tracking
                  - Some heap cells wrap native handles (files, network)
                  - Add `weight` to object tracking to represent non-memory constrained resources
                  - **3.4.1.3.1.1.1.1.1.1** File Descriptor Exhaustion Monitoring
                    - **3.4.1.3.1.1.1.1.1.1.1** EBADF Signal Interception Handlers
  - **Test:** Add `compiler/tests/gc_stress.rs`:
    ```rust
    #[test]
    fn gc_handles_circular_refs() {
        // Create array A containing HeapRef to array B, 
        // and array B containing HeapRef to array A
        // Trigger GC, verify no crash and both are collected when unreachable
    }
    
    #[test]
    fn gc_under_allocation_pressure() {
        // Set GC threshold very low (e.g., 10 objects)
        // Allocate 10000 objects in a loop, discarding references
        // Verify VM doesn't OOM and GC reclaims memory
    }
    ```

### 3.5 Bytecode Compiler Pipeline Deep Analysis

- [ ] **Audit `compiler/src/runtime/bytecode_compiler.rs`** (1253 LOC, 54 items)
  - #### 3.5.1 BytecodeCompiler Architecture
    - `BytecodeCompiler` struct: `instructions: Vec<OpCode>`, `scopes: Vec<Scope>`, `next_local: usize`, `functions: Vec<CompiledFunction>`
    - ##### 3.5.1.1 Scope Management
      - `push_scope()` — creates new lexical scope with base = current `next_local`
      - `pop_scope()` — restores `next_local` to scope base (locals are stack-allocated)
      - `declare_local(name)` — maps name → slot index, increments `next_local`
      - `resolve_local(name)` — walks scope chain innermost-first, returns slot index
      - ###### 3.5.1.1.1 Closure Variable Capture
        - **Not implemented** — closures cannot capture variables from enclosing scopes
        - Requires adding `UpValue` concept: `struct UpValue { name: String, stack_slot: usize, is_local: bool }`
        - Bytecode opcodes needed: `LoadUpValue(idx)`, `StoreUpValue(idx)`, `CloseUpValue(idx)`
        - **3.5.1.1.1.1** Upvalue Resolution Algorithm
          - When variable referenced, check current locals first
          - If not found, check enclosing function scopes (not just blocks)
          - If found in enclosing function, add to function's `upvalues` list
          - **3.5.1.1.1.1.1** Upvalue Lifecycle
            - Created on stack: `Value::UpValue(Rc<RefCell<Value>>)` wrapping local slot
            - When scope exits, `CloseUpValue` moves value from stack to heap
            - Closure object holds pointers to these UpValues
            - **3.5.1.1.1.1.1.1** UpValue GC Integration
              - `GC` must traverse Closure objects to Mark their captured UpValues
              - `UpValue` itself must be a GC-managed `HeapCell::UpValue`
              - **3.5.1.1.1.1.1.1.1** Mutable Capture Semantics
                - If closure mutates upvalue, the change is visible to the enclosing scope
                - This requires upvalues to be heap-allocated immediately if they are mutated, or on escape
                - **3.5.1.1.1.1.1.1.1.1** Escape Analysis
                  - Optimization: Prove closure never leaves scope (e.g., passed to `map`), allocate upvalues on stack
                  - Fallback: allocate all captured variables on heap (slower but safe)
    - ##### 3.5.1.2 Statement Compilation (`compile_statement`, lines 238-555)
      - `Let/Var` → compile RHS expression, emit `StoreLocal(slot)` for declared local
      - `If/Elif/Else` → compile condition, emit `JumpIfNot(placeholder)`, compile body, patch jump to end
      - `For` → compile iterator, emit loop start label, compile body, emit `Jump(loop_start)`
      - `While` → loop start label, compile condition, `JumpIfNot(end)`, compile body, `Jump(start)`
      - `Return` → compile return expression, emit `Return`
      - `Match` → compile scrutinee, for each arm: emit pattern check + `JumpIfNot(next_arm)`, compile body, `Jump(end)`
      - ###### 3.5.1.2.1 Missing Statement Compilation
        - `Defer` — not compiled (emits no instructions)
        - `Spawn` — not compiled (concurrency not supported in bytecode VM)
        - `Select` — not compiled (channel select not supported)
        - `Yield` — not compiled (generator/coroutine not supported)
        - `Unsafe` — compiles body normally (no safety boundary enforcement)
        - **3.5.1.2.1.1** Defer Compilation Implementation
          - Maintain list of deferred blocks in current scope
          - Before emitting `Return` or scope exit jumps, emit all deferred blocks in LIFO order
          - **3.5.1.2.1.1.1** Defer and Errors
            - If block throws error `?`, defers must still run
            - Requires `try-catch` VM mechanics: push defer block to exception handler stack
            - **3.5.1.2.1.1.1.1** VM Exception Tables
              - Map IP ranges to catch blocks/cleanup routines
              - On error, VM unwinds stack, finding matching exception table entry
              - Runs cleanup bytecode, then re-raises error
              - **3.5.1.2.1.1.1.1.1** Unwinding Bytecode Generation
                - Bytecode compiler emits `SetExceptionHandler(target)` at scope start
                - Emits `ClearExceptionHandler` on normal exit
                - **3.5.1.2.1.1.1.1.1.1** Nested Handlers
                  - Exception handler stack is managed per-callframe
                  - Inner handler masks outer handler until cleared
    - ##### 3.5.1.3 Expression Compilation (`compile_expression`, lines 561-889)
      - Binary ops → compile LHS, compile RHS, emit `Add`/`Sub`/`Mul`/`Div` etc.
      - Unary ops → compile operand, emit `Neg`/`Not`
      - Literals → emit `Push(Value::Int(n))` / `Push(Value::String(s))` etc.
      - Variable access → `resolve_local(name)` → `LoadLocal(slot)` or `LoadGlobal(name)` if not local
      - Method call `obj.method(args)` → compile obj, compile args, emit `CallNamed(method, argc)`
      - ###### 3.5.1.3.1 Method Call Dispatch Issue
        - Current: method calls emit `CallNamed(method_name, argc)` — but this looks up the method as a top-level function
        - **Problem:** struct methods like `knowledge.query()` won't be found as top-level functions
        - **Fix needed:** Emit `LoadField("query")` to get method reference, then `Call(argc)` — requires `Call(n)` implementation (Gap 1 in §3.6.2)
        - Alternative: Mangle method names as `StructName__method_name` and register them as top-level functions
        - **3.5.1.3.1.1** Name Mangling Pipeline
          - Extract receiver type during semantic analysis: `knowledge_store: KnowledgeStore`
          - Mangle method lookup: `KnowledgeStore__query`
          - Emit `CallNamed("KnowledgeStore__query", argc)`
          - **3.5.1.3.1.1.1** Bytecode Receiver Injection
            - Method calls implicitly pass `self` as first arg
            - Source: `obj.method(arg1)` → Bytecode: `Push(obj)`, `Push(arg1)`, `CallNamed("Type__method", 2)`
            - **3.5.1.3.1.1.1.1** Dynamic Dispatch (Traits)
              - If receiver matches `dyn Trait`, mangling fails (concrete type unknown at compile time)
              - **3.5.1.3.1.1.1.1.1** VTable Implementation
                - Emit new opcode `CallMethod("query", argc)`
                - VM looks up `_vtable` hidden field on the object
                - Grabs method pointer from vtable, invokes
                - **3.5.1.3.1.1.1.1.1.1** Performance Penalty
                  - Dynamic dispatch is ~3x slower than static call
                  - Devirtualization optimization pass in bytecode compiler can convert to static if type is provably exact

### 3.6 OVM Instruction Set — Complete Opcode Map & Execution Semantics

The VM in `compiler/src/runtime/vm.rs` currently handles **36 opcodes**. Below is the exhaustive map, current behaviour, and what's **missing**.

#### 3.6.1 Implemented Opcodes (working correctly)

| Opcode | Tag | Args | Behaviour (vm.rs) |
|--------|-----|------|----|
| `Nop` | 0x00 | — | No-op |
| `Push(Value)` | 0x01 | value | Push literal (Null/Int/Float/Bool/String) |
| `Pop` | 0x02 | — | Pop and discard top |
| `Dup` | 0x03 | — | Clone top and push |
| `Swap` | 0x04 | — | Swap top two |
| `Add` | 0x10 | — | Int+Int, Float+Float, mixed promote to Float, String+String concat |
| `Sub` | 0x11 | — | Same numeric rules, no string |
| `Mul` | 0x12 | — | Same numeric rules |
| `Div` | 0x13 | — | Checks division-by-zero for both Int and Float |
| `Mod` | 0x14 | — | Checks modulo-by-zero |
| `Neg` | 0x15 | — | Negate Int or Float |
| `Eq` | 0x20 | — | Structural equality via `PartialEq` on `VmValue` |
| `Ne` | 0x21 | — | Negated `Eq` |
| `Lt/Le/Gt/Ge` | 0x22–0x25 | — | Int, Float, mixed (promote), String (lexicographic) |
| `And` | 0x30 | — | **Only** `Bool && Bool` — returns error on non-bool inputs |
| `Or` | 0x31 | — | **Only** `Bool \|\| Bool` |
| `Not` | 0x32 | — | **Only** `Bool` |
| `Concat` | 0x33 | — | String + String only (Add also does this) |
| `LoadLocal(n)` | 0x40 | slot idx | Range-checked access to `frame.locals[n]` |
| `StoreLocal(n)` | 0x41 | slot idx | Auto-grows locals vec if `n >= locals.len()` |
| `LoadGlobal(name)` | 0x42 | string | Returns `Null` if undefined (no error) |
| `StoreGlobal(name)` | 0x43 | string | Inserts/overwrites in `globals` HashMap |
| `LoadField(name)` | 0x44 | string | Works on `VmValue::Struct` only |
| `StoreField(name)` | 0x45 | string | Pushes modified struct back (no mutation-in-place) |
| `Jump(addr)` | 0x50 | instr idx | Sets `frame.ip = addr` |
| `JumpIf(addr)` | 0x51 | instr idx | Pops, jumps if `is_truthy` |
| `JumpIfNot(addr)` | 0x52 | instr idx | Pops, jumps if not truthy |
| `CallNamed(name, n)` | 0x61 | string, argc | Looks up function in `module.functions` by name, pops `n` args, pushes new CallFrame |
| `Return` | 0x62 | — | Pops return value, pops call frame, pushes return value onto caller's stack |
| `NewStruct(name, n)` | 0x70 | string, field count | Pops `n` values, creates struct with auto-named fields `field_0`, `field_1`, etc. |
| `NewArray(n)` | 0x71 | element count | Pops `n` values in reverse, creates array |
| `NewMap(n)` | 0x72 | pair count | Pops `2n` values (key, val), creates map |
| `Index` | 0x73 | — | Array[Int], Map[Any], String[Int] indexing |
| `Print` / `PrintLn` | 0x80/0x81 | — | Pops top, formats via `Display`, appends to `vm.output` AND prints to stdout |
| `Len` | 0x82 | — | String, Array, Map length |
| `TypeOf` | 0x83 | — | Returns type name as String |
| `Assert` | 0x84 | — | Pops, returns `Err` if not truthy |
| `Halt` | 0xFF | — | Breaks execution loop |

#### 3.6.2 CRITICAL GAPS in the Opcode Set

##### Gap 1: `Call(n)` — UNIMPLEMENTED
- **File:** `vm.rs` line 753-758
- **Current code:** `return Err("Call(n) with function-on-stack is not yet supported; use CallNamed")`
- **Impact:** Cannot call closures, function pointers, or dynamically-dispatched functions
- **Required implementation:**
  ```rust
  OpCode::Call(n_args) => {
      // Top of stack must be a callable value
      let callable = self.pop()?;
      match callable {
          VmValue::Int(func_idx) => {
              // Treat as function index into module.functions
              let idx = func_idx as usize;
              if idx >= module.functions.len() {
                  return Err(OvmError::FunctionNotFound { 
                      name: format!("<function@{}>", idx) 
                  });
              }
              let mut args = Vec::with_capacity(n_args);
              for _ in 0..n_args {
                  args.push(self.pop()?);
              }
              args.reverse();
              let target_fn = &module.functions[idx];
              let mut locals = args;
              while locals.len() < target_fn.locals_count {
                  locals.push(VmValue::Null);
              }
              self.call_stack.push(CallFrame {
                  function_index: idx,
                  ip: 0,
                  base_slot: self.stack.len(),
                  locals,
              });
          }
          _ => return Err(OvmError::TypeMismatch {
              expected: "callable".to_string(),
              actual: format!("{:?}", callable),
          }),
      }
  }
  ```
  - **3.6.2.1.1** Callable Types in Bytecode
    - Int variants represent `module.functions` indices (top-level `fn`)
    - Closures need a new `HeapCell::Closure { func_idx: usize, upvalues: Vec<HeapRef> }`
    - **3.6.2.1.1.1** Closure Invocation
      - If `callable` is `VmValue::HeapRef(idx)` pointing to `Closure`
      - Read `func_idx`, setup `CallFrame` exactly like an `Int` call
      - Also bind the `upvalues` array to the frame for `LoadUpValue` access
      - **3.6.2.1.1.1.1** Stack Frame Upvalue Padding
        - Add `captured_upvalues: Vec<HeapRef>` to `CallFrame` struct
        - **3.6.2.1.1.1.1.1** Type Verification on Call
          - Bytecode compiler statically checks argument counts
          - Runtime `Call(n)` should still verify `n_args == target_fn.params_count`
          - **3.6.2.1.1.1.1.1.1** Default Argument Bypass
            - If `target_fn` has defaults, `n_args` might be < `params_count`
            - `Call(n)` must pad missing args by fetching defaults from module metadata

##### Gap 2: `Import(path)` — No-Op Placeholder
- **File:** `vm.rs` line 940-942
- **Current code:** `// Placeholder – imports are resolved at compile time`
- **Impact:** Multi-module programs cannot load dependencies at runtime. The bytecode compiler emits `Import` opcodes, but the VM silently ignores them.
- **Required:** Either (a) resolve all imports at compile time and remove Import opcode emission, or (b) implement runtime module loading:
  ```rust
  OpCode::Import(ref path) => {
      // Resolve module path to .ovm file
      let module_path = resolve_module_path(path, &module.name)?;
      let bytes = std::fs::read(&module_path)
          .map_err(|e| OvmError::Io(e))?;
      let imported = OvmModule::deserialize(&bytes)
          .map_err(|e| OvmError::PluginError { 
              message: format!("Failed to load module '{}': {}", path, e) 
          })?;
      // Execute the module's top-level code (if any) and
      // merge its functions into a module registry
      self.loaded_modules.insert(path.clone(), imported);
  }
  ```
  - **This requires adding** `loaded_modules: HashMap<String, OvmModule>` to `OmniVM` struct
  - And modifying `CallNamed` to also search loaded modules, not just the current one
  - **3.6.2.2.1** Cross-Module Function Resolution
    - `CallNamed("math::sqrt", 1)` → split on `::`
    - Module: `math`, Function: `sqrt`
    - **3.6.2.2.1.1** Module Registry Architecture
      - `OmniVM` holds `Registry { modules: HashMap<String, OvmModule> }`
      - **3.6.2.2.1.1.1** Lazy vs Eager Loading
        - Bytecode compiler emits `Import("math")` at top of file
        - VM eagerly loads `.ovc` on hitting `Import`
        - **3.6.2.2.1.1.1.1** Caching Reused Modules
          - `if self.loaded_modules.contains_key(path) { return Ok(()); }`
          - Prevents diamond-dependency redundant loads
          - **3.6.2.2.1.1.1.1.1** Dependency Cycle Handling in VM
            - Requires `loading_modules: HashSet<String>` to detect `Import` loops during execution
            - **3.6.2.2.1.1.1.1.1.1** Runtime Cyclical Execution Aborts

##### Gap 3: `NewStruct` uses auto-generated field names
- **File:** `vm.rs` line 818: `fields.push((format!("field_{}", i), val));`
- **Impact:** All structs lose their original field names — they become `field_0`, `field_1`, etc.
- **Fix:** The bytecode compiler (`bytecode_compiler.rs`) must emit field names alongside `NewStruct`:
  - Change `OpCode::NewStruct(String, usize)` to `OpCode::NewStruct(String, Vec<String>)` where the Vec contains field names
  - Or add a `NewStructNamed(String, Vec<String>)` opcode
  - This requires updating `bytecode.rs` serialization/deserialization, the bytecode compiler emission, and the VM execution
  - **3.6.2.3.1** Opcode Memory Layout
    - `NewStructNamed` payload: struct name length, struct name bytes, number of fields, field names encoded back-to-back
    - **3.6.2.3.1.1** Decoder Implementation
      - `bincode` or `rmp-serde` efficiently handles `Vec<String>` in custom opcode serializers
      - **3.6.2.3.1.1.1** Struct Memory Representation Optimization
        - Currently `HeapCell::Struct` uses `Vec<(String, VmValue)>` (Slow dictionary approach)
        - Better: `HashMap<String, usize>` shared struct definition, instance holds `Vec<VmValue>`
        - **3.6.2.3.1.1.1.1** Struct Def Registry
          - Top-level `module.struct_defs: HashMap<String, Vec<String>>`
          - Instances just use `OpCode::NewStruct(type_name, count)` and link to definition
          - **3.6.2.3.1.1.1.1.1** Fast Field Access
            - `OpCode::LoadField("query")` is slow string comparison
            - Upgrade to `OpCode::LoadFieldIndex(3)` using compile-time resolved offsets
            - **3.6.2.3.1.1.1.1.1.1** Struct Field Offset Verification in Bytecode Compiler

##### Gap 4: No native function call mechanism in bytecode VM
- **File:** `vm.rs` — the entire file has NO reference to `NativeManager`
- **Impact:** Bytecode-compiled programs cannot call `io::print`, `sys::time_now`, `net::http_get`, etc.
- **Fix:** Add a `CallNative(module_name, func_name, arg_count)` opcode:
  ```rust
  // In bytecode.rs OpCode enum:
  CallNative(String, String, usize),  // tag 0x63
  
  // In vm.rs OmniVM struct:
  native_manager: NativeManager,
  
  // In vm.rs execute():
  OpCode::CallNative(ref module, ref func, n_args) => {
      let mut args = Vec::with_capacity(n_args);
      for _ in 0..n_args {
          args.push(self.pop()?);
      }
      args.reverse();
      // Convert VmValue -> RuntimeValue for NativeManager
      let native_args: Vec<RuntimeValue> = args.iter()
          .map(|v| vm_value_to_runtime_value(v))
          .collect();
      let result = self.native_manager.call(module, func, &native_args)?;
      self.push(runtime_value_to_vm_value(result));
  }
  ```
  - **3.6.2.4.1** Value Conversion Overhead
    - `VmValue` (enum) ↔ `RuntimeValue` (enum) mapping requires allocations for HeapRefs (Strings/Arrays)
    - **3.6.2.4.1.1** Zero-Copy ABI Design
      - Redefine `NativeManager` to accept `&mut OmniVM` directly
      - `fn native_print(vm: &mut OmniVM, args: &[VmValue]) -> Result<VmValue, OvmError>`
      - Eliminates `RuntimeValue` entirely, avoiding O(N) copy on every call
      - **3.6.2.4.1.1.1** Safety Boundary
        - Native functions must not panic or memory-corrupt the VM
        - They return `Result` which is bubbled up to `OpCode::CallNative` match arm
        - **3.6.2.4.1.1.1.1** Async Native Calls
          - `http_get` blocks the VM thread
          - Need `OpCode::CallNativeAsync` returning a Future/Promise handle to VM
          - **3.6.2.4.1.1.1.1.1** Polling Infrastructure
            - Yield control to Tokio runtime when performing async native bindings
            - **3.6.2.4.1.1.1.1.1.1** Await Task Waker Context Propagation

### 3.7 The Two-Runtime Unification Problem

#### 3.7.1 Current State
The project has **two completely separate execution engines** that do not share types:

| Property | Tree-Walk Interpreter | Bytecode VM |
|----------|----------------------|-------------|
| **File** | `compiler/src/runtime/interpreter.rs` (135KB) | `compiler/src/runtime/vm.rs` (67KB) |
| **Value type** | `RuntimeValue` (with NativePtr, Vector, Module variants) | `VmValue` (with HeapRef, no native types) |
| **Native calls** | Yes — via `NativeManager` (io, sys, net, math) | **NO** — no native function support |
| **GC** | None (reference counting implicit in Rust ownership) | Tri-color mark-and-sweep |
| **Module loading** | Yes — resolves `import` AST nodes | No — `OpCode::Import` is a no-op |
| **Used by** | `omnc run <file>.omni` (interprets source directly) | `omnc compile` → `.ovm` → `OmniVM::execute()` |

#### 3.7.2 Required Resolution
- [ ] **Short-term (deployment-critical):** Add `NativeManager` integration to `vm.rs` so both paths can call native functions.
  - Convert between `VmValue` and `RuntimeValue` at the FFI boundary
  - Add conversion functions:
    ```rust
    fn vm_to_runtime(v: &VmValue) -> RuntimeValue {
        match v {
            VmValue::Null => RuntimeValue::Null,
            VmValue::Int(i) => RuntimeValue::Integer(*i),
            VmValue::Float(f) => RuntimeValue::Float(*f),
            VmValue::Bool(b) => RuntimeValue::Boolean(*b),
            VmValue::String(s) => RuntimeValue::String(s.clone()),
            VmValue::Array(a) => RuntimeValue::Array(
                a.iter().map(vm_to_runtime).collect()
            ),
            _ => RuntimeValue::Null,
        }
    }
    fn runtime_to_vm(v: RuntimeValue) -> VmValue {
        match v {
            RuntimeValue::Null => VmValue::Null,
            RuntimeValue::Integer(i) => VmValue::Int(i),
            RuntimeValue::Float(f) => VmValue::Float(f),
            RuntimeValue::Boolean(b) => VmValue::Bool(b),
            RuntimeValue::String(s) => VmValue::String(s),
            RuntimeValue::Array(a) => VmValue::Array(
                a.into_iter().map(runtime_to_vm).collect()
            ),
            RuntimeValue::NativePtr(p) => VmValue::Int(p as i64),
            _ => VmValue::Null,
        }
    }
    ```
    - **3.7.2.1.1** NativePtr FFI Safety
      - Passing `NativePtr` as `i64` allows Omni scripts to forge pointers and crash the VM
      - **3.7.2.1.1.1** Pointer Capability Sealing
        - Encode pointers with an authenticated tag (MAC) using a VM session key
        - `sealed_ptr = { ptr: i64, mac: u64 }`
        - Or keep pointers in a `HashMap<u64, *mut c_void>` handle table, hand out abstract IDs
        - **3.7.2.1.1.1.1** Handle Table Implementation
          - `VM.handles: SlotMap<DefaultKey, Box<dyn Any>>`
          - Scripts only see numeric IDs
          - **3.7.2.1.1.1.1.1** Handle GC Integration
            - **3.7.2.1.1.1.1.1.1** Tombstone Marker for Stale Pointer References
              - **3.7.2.1.1.1.1.1.1.1** Safe Memory Recycling via Generational Counters
            - Handles must implement a `Trace` trait to keep underlying references alive
            - Handles must be freed when no `VmValue` references its ID
            - **3.7.2.1.1.1.1.1.1** Handle Finalizers
              - Files/Sockets backed by handles need explicit `close()` upon GC sweep
              - `Box<dyn Any>` should be `Box<dyn ManagedResource>` with an `on_collect()` hook
- [ ] **Medium-term:** Deprecate the tree-walk interpreter and use only the bytecode VM path for deployment. Keep the interpreter for debugging and REPL mode.
  - **3.7.2.2.1** REPL Mode Unification
    - The REPL requires executing partial ASTs while maintaining global state
    - **3.7.2.2.1.1** Bytecode REPL Design
      - REPL input is parsed, compiled to an anonymous function, executed, and its top-level bindings merged into VM globals
      - **3.7.2.2.1.1.1** Continuous Compilation
        - `compiler` must cleanly deserialize just the newly compiled segment into the running VM
        - **3.7.2.2.1.1.1.1** Global State Persistance
          - `LoadGlobal` and `StoreGlobal` semantics must perfectly match across partial evaluations
          - **3.7.2.2.1.1.1.1.1** Type Environment Preservation
            - The SemanticAnalyzer must keep its `scopes` and `registered_structs` alive across REPL inputs

### 3.8 Native Function Bindings — Full Gap Analysis

#### 3.8.1 Currently Implemented (in `native.rs`, 199 LOC)
| Module | Function | Implementation |
|--------|----------|---------------|
| `io` | `print(val)` | `print!("{:?}", val)` |
| `io` | `println(val)` | `println!("{:?}", val)` |
| `io` | `stdin_read_line()` | `std::io::stdin().read_line()` |
| `io` | `file_open(path)` → NativePtr | `std::fs::File::open` → handle |
| `io` | `file_create(path)` → NativePtr | `std::fs::File::create` → handle |
| `io` | `file_write(handle, data)` → Int | `file.write_all(data.as_bytes())` |
| `io` | `file_read_to_string(handle)` → String | `file.read_to_string()` |
| `sys` | `time_now()` → Int | `SystemTime::now()` epoch seconds |
| `sys` | `sleep(ms)` | `std::thread::sleep(Duration::from_millis)` |
| `sys` | `os_name()` → String | `std::env::consts::OS` |
| `sys` | `num_cpus()` → Int | `num_cpus::get()` |
| `net` | `http_get(url)` → String | `reqwest::blocking::get()` |
| `net` | `tcp_connect(addr)` → NativePtr | `TcpStream::connect` → handle |
| `net` | `tcp_write(handle, data)` → Int | `stream.write_all(data.as_bytes())` |
| `math` | `tensor_create(size)` → Vector | `ndarray::Array1::<f32>::zeros(size)` |
| `math` | `tensor_matmul(a, b)` → Vector | **BUG: does element-wise add, not matmul** |

#### 3.8.2 Missing Native Functions (required for deployment)
Each entry shows the Rust implementation needed in `native.rs`:

**File System (add to `io` module)**
```rust
("io", "file_close") => {
    let handle = self.get_handle_arg(args, 0)?;
    self.files.remove(&handle);
    Ok(RuntimeValue::Null)
},
("io", "file_exists") => {
    let path = self.get_string_arg(args, 0)?;
    Ok(RuntimeValue::Boolean(std::path::Path::new(path).exists()))
},
("io", "file_delete") => {
    let path = self.get_string_arg(args, 0)?;
    std::fs::remove_file(path).map_err(|e| e.to_string())?;
    Ok(RuntimeValue::Null)
},
("io", "dir_create") => {
    let path = self.get_string_arg(args, 0)?;
    std::fs::create_dir_all(path).map_err(|e| e.to_string())?;
    Ok(RuntimeValue::Null)
},
("io", "dir_list") => {
    let path = self.get_string_arg(args, 0)?;
    let entries: Vec<RuntimeValue> = std::fs::read_dir(path)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .map(|e| RuntimeValue::String(e.file_name().to_string_lossy().to_string()))
        .collect();
    Ok(RuntimeValue::Array(entries))
},
("io", "file_rename") => {
    let from = self.get_string_arg(args, 0)?;
    let to = self.get_string_arg(args, 1)?;
    std::fs::rename(from, to).map_err(|e| e.to_string())?;
    Ok(RuntimeValue::Null)
},
```
- **3.8.2.1** File IO Security Sandbox
  - Native filesystem access can escape the Omni working directory if unguarded
  - **3.8.2.1.1** Path Sandbox Normalization
    - Before calling `std::fs` functions, sanitize the logical path
    - **3.8.2.1.1.1** `canonicalize` Constraints
      - `path.canonicalize()` resolves symlinks; fail if resulting absolute path is outside `allowed_root`
      - **3.8.2.1.1.1.1** Symlink Jailbreak Prevention
        - If a user creates a symlink pointing to `C:\Windows`, `file_open` must reject it
        - **3.8.2.1.1.1.1.1** Capability Based Access Control
          - Instead of root paths, scripts operate on opaque Capability tokens
          - **3.8.2.1.1.1.1.1.1** Tokenized IO API
            - `io::dir_open(token)` returns a directory handle
            - `io::file_open_at(dir_handle, "filename")` ensures access is relative to capability

```rust
// Add to NativeManager struct:
named_pipes: HashMap<usize, /* platform-specific pipe type */>,

// Windows implementation:
#[cfg(windows)]
("ipc", "pipe_create") => {
    let name = self.get_string_arg(args, 0)?;
    use windows_sys::Win32::System::Pipes::*;
    use windows_sys::Win32::Foundation::*;
    let pipe_name: Vec<u16> = format!(r"\\.\pipe\{}", name)
        .encode_utf16().chain(std::iter::once(0)).collect();
    let handle = unsafe {
        CreateNamedPipeW(
            pipe_name.as_ptr(),
            PIPE_ACCESS_DUPLEX,
            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
            1, // max instances
            65536, // out buffer
            65536, // in buffer
            0,     // timeout
            std::ptr::null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err("Failed to create named pipe".to_string());
    }
    let h = self.alloc_handle();
    // store handle...
    Ok(RuntimeValue::NativePtr(h))
},
("ipc", "pipe_connect") => {
    let name = self.get_string_arg(args, 0)?;
    // CreateFile to connect to existing pipe
    // ...
},
("ipc", "pipe_read") => {
    // ReadFile on pipe handle
},
("ipc", "pipe_write") => {
    // WriteFile on pipe handle
},
```
- **Dependency:** Add `windows-sys = { version = "0.52", features = ["Win32_System_Pipes", "Win32_Foundation", "Win32_Storage_FileSystem"] }` to `compiler/Cargo.toml`
- **3.8.2.2** IPC Protocol Security
  - Named pipes must enforce access control lists (ACLs) to prevent unprivileged processes from connecting
  - **3.8.2.2.1** Security Descriptor Initialization
    - Create a `SECURITY_ATTRIBUTES` struct allowing only `LocalSystem` and `Administrators`
    - **3.8.2.2.1.1** Impersonation Levels
      - `PIPE_REJECT_REMOTE_CLIENTS` flag prevents network-based IPC attacks
      - **3.8.2.2.1.1.1** Client Handshake
        - Upon connection, server requests client PID via `GetNamedPipeClientProcessId`
        - **3.8.2.2.1.1.1.1** PID Verification
          - Verify client executable path matches signed Helios binaries using `QueryFullProcessImageNameW`
          - **3.8.2.2.1.1.1.1.1** Signature Validation
            - Use `WinVerifyTrust` on the client executable before accepting the first frame

**JSON Parsing (native needed for performance)**
```rust
("json", "parse") => {
    let text = self.get_string_arg(args, 0)?;
    let value: serde_json::Value = serde_json::from_str(text)
        .map_err(|e| e.to_string())?;
    Ok(json_to_runtime_value(&value))
},
("json", "stringify") => {
    let val = &args[0];
    let json = runtime_value_to_json(val);
    Ok(RuntimeValue::String(serde_json::to_string(&json).unwrap()))
},
```
- **3.8.2.3** Built-in JSON Overhead limits
  - Omni's `Map` is ordered whereas JSON is unordered.
  - Conversion mapping `RuntimeValue::Map` to `serde_json::Map` is O(N).
  - **3.8.2.3.1** Depth Exceeded Mitigation
    - Limit `parse` nesting to 64 levels to prevent stack overflow internally
    - **3.8.2.3.1.1** `json_to_runtime_value` implementation
      - Recursive function: must track `depth` and return `RuntimeError("json too deep")`
      - **3.8.2.3.1.1.1** Number Precision
        - JSON spec doesn't differentiate int/float
        - Omni uses i64/f64. Map `Value::Number` by checking `.is_i64()` first
        - **3.8.2.3.1.1.1.1** Array Parsing Overheads
          - Preallocate vectors using `.as_array().unwrap().len()`
          - **3.8.2.3.1.1.1.1.1** Native Array Buffer Binding
            - Ideally, map large `[f64]` JSON arrays directly into `HeapCell::Vector` for ML payloads

**Cryptography (required for OmniCrypt §3)**
```rust
// Dependency: blake3 = "1.5", chacha20poly1305 = "0.10"
("crypto", "blake3_hash") => {
    let data = self.get_string_arg(args, 0)?;
    let hash = blake3::hash(data.as_bytes());
    Ok(RuntimeValue::String(hash.to_hex().to_string()))
},
```
- **3.8.2.4** Constant-Time Validation
  - `String` comparisons in Omni bytecode are not constant-time
  - **3.8.2.4.1** `crypto::subtle_eq` binding
    - Provide `subtle::ConstantTimeEq` wrapped native function
    - **3.8.2.4.1.1** MAC Verification
      - Requires constant time compares for AES-GCM auth tags
      - **3.8.2.4.1.1.1** RNG Seeding
        - Provide `crypto::random_bytes` backed by `OsRng`
        - **3.8.2.4.1.1.1.1** Post-Quantum Support (Dilithium / ML-KEM)
          - Binding liboqs or rust-pq primitives required for §92
          - **3.8.2.4.1.1.1.1.1** Native Key Ring Handles
            - Private keys should stay in Rust memory (`NativePtr`), never serialized to Omni `String`

**HTTP Client (required for Agent capabilities)**
```rust
("net", "http_request") => {
    // Requires mapping reqwest::Client
}
```
- **3.8.2.5** HTTP TLS Trust Roots
  - `reqwest` default uses `native-tls` or `rustls`
  - **3.8.2.5.1** Certificate Pinning
    - Must provide `native` argument to supply a pinned CA cert explicitly
    - **3.8.2.5.1.1** Request Timeouts
      - Hardcode `ClientBuilder::timeout` to prevent hung agent threads
      - **3.8.2.5.1.1.1** Proxy Environments
        - Read `HTTP_PROXY` env automatically on initialization
        - **3.8.2.5.1.1.1.1** Server Name Indication (SNI) Validation
          - Prevent Domain Fronting via Host Header mismatch checks
          - **3.8.2.5.1.1.1.1.1** HTTP/3 (QUIC) Enablement
            - `reqwest` requires a feature flag for `h3`, which Helios needs for §35 Federation Multipath
    let data = self.get_string_arg(args, 0)?;
    let hash = blake3::hash(data.as_bytes());
    Ok(RuntimeValue::String(hash.to_hex().to_string()))
},
("crypto", "aead_encrypt") => {
    // XChaCha20-Poly1305 AEAD encryption
},
("crypto", "aead_decrypt") => {
    // XChaCha20-Poly1305 AEAD decryption
},
```

**Threading**
```rust
// Note: std::thread is already imported in native.rs
("thread", "spawn") => {
    // This requires VM-level thread support — each thread needs its own
    // OmniVM instance or a shared-nothing message-passing model
    // For deployment: use Tokio tasks via tokio::spawn
},
```

**MessagePack Serialization (for IPC wire format)**
```rust
// Dependency: rmp-serde = "1.1"
("msgpack", "serialize") => {
    let val = &args[0];
    let bytes = rmp_serde::to_vec(&runtime_value_to_msgpack(val))
        .map_err(|e| e.to_string())?;
    Ok(RuntimeValue::Array(bytes.into_iter().map(|b| RuntimeValue::Integer(b as i64)).collect()))
},
("msgpack", "deserialize") => {
    // byte array -> RuntimeValue
},
```

#### 3.8.3 Bug Fix: `tensor_matmul`
- **File:** `native.rs` line 161-171
- **Current behavior:** Does element-wise **add** (`a + b`), NOT matrix multiplication
- **Comment in code admits:** `"For this demo, we'll do element-wise add just to prove op works"`
- **Fix:** Either implement proper matmul using ndarray `dot()`, or mark the function as `tensor_add` and add a separate proper matmul

### Verification — Section 3

```powershell
# 1. No panics in production runtime code
Select-String -Path "omni-lang\compiler\src\runtime\*.rs" -Pattern "panic!\(|\.unwrap\(\)|\.expect\(" -Exclude "*tests*"
# Expected: Zero matches (or only in test helpers)

# 2. All runtime tests pass
cd omni-lang\compiler; cargo test runtime
# Expected: 100+ tests pass

# 3. Clippy clean
cd omni-lang\compiler; cargo clippy -- -D warnings
# Expected: Zero warnings
```

---

## 4. Knowledge Store Hardening

**Goal:** Transform the JSON-based knowledge store into a robust, binary-format persistent store with proper indexing, crash recovery, and verification — matching the spec.

### 4.1 Current State Assessment

The current `KnowledgeStore` in `helios-framework/helios/knowledge.omni` (798 LOC):
- Stores facts as `HashMap<u64, InformationUnit>`
- Has indices: `by_subject`, `by_predicate`, `by_source`, `by_accuracy`, `word_index`
- **Serializes to JSON** via `serialize::to_json()` and writes to `knowledge.json`
- Has `query()`, `verify()`, `find_similar()`, `find_causes_for()`, `search()`, `flush()`
- **Missing vs. spec:**
  - No binary `.omk` page format (spec §4)
  - No Write-Ahead Log for crash recovery (spec §7.4/§38.3)
  - No MVCC versioning (spec §38)
  - No B+ tree page structure (spec §7.2)
  - No OmniPack compression (spec §2)
  - No AES/OmniCrypt encryption (spec §3)
  - No Bloom filter for negative lookups (spec §102)
  - No confidence breakdown scoring (simplified to single `u8`)

### 4.2 Phased Upgrade Path

**Phase A — Immediately (deployment-critical):**

- [ ] **Add atomic writes to prevent corruption**
  - Currently `flush()` writes directly to `knowledge.json` — if crash occurs mid-write, data is lost.
  - **Fix in `knowledge.omni::flush()`:**
    ```omni
    fn flush(&mut self):
        if !self.dirty:
            return
        
        let data_path = format("{}/knowledge.json", self.storage_path)
        let tmp_path = format("{}/knowledge.json.tmp", self.storage_path)
        let data = serialize::to_json(self)
        
        # Write to temp file first
        match fs::write_string(&tmp_path, &data):
            Ok(_):
                # Atomic rename (on Windows: ReplaceFile is used)
                fs::rename(&tmp_path, &data_path)
                self.dirty = false
            Err(e):
                io::eprintln("Knowledge store flush failed: {}", e)
    ```
  - **Also add periodic auto-flush** (every 60 seconds or every 10 writes)

- [ ] **Add proper confidence scoring**
  - Currently `InformationUnit` has a simple `confidence: u8`
  - Spec §1 requires `ConfidenceBreakdown` with `source_reliability`, `corroboration_score`, `recency_weight`, `verification_weight`
  - **Check** `helios-framework/helios/knowledge.omni` for `confidence_breakdown` usage (the `verify()` function references `confidence_breakdown.final_score`)
  - If `ConfidenceBreakdown` struct exists but is incomplete, flesh it out per spec §1.4

- [ ] **Add backup/checkpoint support**
  - Create `fn create_checkpoint(&self, label: &str)` that copies `knowledge.json` to `checkpoints/<timestamp>-<label>.json`
  - Add `fn restore_checkpoint(&mut self, path: &str)` that loads a checkpoint

**Phase B — Post-deployment (roadmap):**

- [ ] Binary `.omk` page format with B+ tree indexing
- [ ] Write-Ahead Log (WAL) for crash recovery
- [ ] MVCC versioning with vacuum
- [ ] OmniPack compression
- [ ] OmniCrypt-256 encryption at rest

### 4.3 Knowledge Store — Internal Architecture Deep Dive

#### 4.3.1 Data Model (knowledge.omni, 798 LOC)

```
KnowledgeStore
├── facts: HashMap<u64, InformationUnit>     ← Primary storage
├── by_subject: HashMap<String, Vec<u64>>    ← Subject index
├── by_predicate: HashMap<String, Vec<u64>>  ← Predicate index
├── by_source: HashMap<String, Vec<u64>>     ← Source index
├── by_accuracy: HashMap<String, Vec<u64>>   ← Accuracy level index
├── word_index: HashMap<String, Vec<u64>>    ← Full-text word index
├── storage_path: String                     ← Path to knowledge.json
└── dirty: bool                              ← Write-pending flag
```
- **4.3.1.1** In-Memory Footprint Scaling
  - `HashMap<String, Vec<u64>>` for indices duplicates strings heavily
  - **4.3.1.1.1** String Interning
    - Instead of `String` keys, map values to integer IDs via `SymbolTable`
    - **4.3.1.1.1.1** `SymbolTable` Eviction
      - When an `InformationUnit` is deleted, its symbols should be decremented
      - **4.3.1.1.1.1.1** Reference Counting Index
        - `SymbolTable.refs: HashMap<u32, u32>` tracks active uses of a String
        - **4.3.1.1.1.1.1.1** Zero-Ref Cleanup
          - During `flush()`, remove symbols with 0 refs to prevent memory leaks over time
          - **4.3.1.1.1.1.1.1.1** Memory Fragmentation Recovery
            - Provide a `#knowledge defrag` native command to recreate the HashMap and eliminate fragmented backing arrays

#### 4.3.2 InformationUnit Structure (from knowledge.omni)

```omni
struct InformationUnit:
    id: u64
    subject: String              # e.g., "France"
    predicate: String            # e.g., "capital_of"
    content: String              # e.g., "The capital of France is Paris"
    source: Source               # Who provided this info
    confidence: u8               # 0-100 overall score
    confidence_breakdown: ConfidenceBreakdown
    accuracy: Accuracy           # Verified/Unverified/Disputed
    created_at: u64
    updated_at: u64
    related_to: Vec<u64>         # Cross-references
    history: UpdateHistory       # Previous versions
```

#### 4.3.3 Query Flow

```
query(Query { subject, constraints })
    │
    ├── Look up by_subject[subject] → candidate IDs
    │
    ├── For each candidate:
    │   ├── matches_query(unit, query) → checks constraints
    │   │   ├── Constraint::Predicate(pred) → unit.predicate matches?
    │   │   ├── Constraint::MinConfidence(min) → unit.confidence >= min?
    │   │   └── Constraint::Source(type) → unit.source.source_type() matches?
    │   │
    │   └── word_overlap(query.subject, unit.content) → relevance score
    │
    ├── Sort by relevance (word overlap count)
    │
    └── Return Vec<InformationUnit>
```
- **4.3.3.1** Relevance Sorting Performance
  - `word_overlap` calculates intersection of two HashSets per candidate
  - **4.3.3.1.1** Big-O Complexity
    - O(C * N) where C is candidates, N is query words
    - **4.3.3.1.1.1** Pre-computed TF-IDF
      - Instead of naive intersection, use Term Frequency-Inverse Document Frequency
      - **4.3.3.1.1.1.1** IDF Tracking
        - Maintain `word_frequencies: HashMap<u32, u32>` updated continuously on `insert()`
        - **4.3.3.1.1.1.1.1** BM25 Algorithm Optimization
          - Calculate BM25 score during insertion and cache it on the `InformationUnit`
          - **4.3.3.1.1.1.1.1.1** Query-time Fast Path
            - Since BM25 is pre-calculated per term, candidate scoring becomes O(N) additions rather than HashSet intersections

#### 4.3.4 Verification Flow (confidence scoring)

The `verify()` function (lines 232-258) updates confidence:
```
verify(id, verified_by, notes, independent_sources)
    │
    ├── Get existing InformationUnit
    ├── Remove from indices (will re-add with updated data)
    ├── Create ConfidenceBreakdown with:
    │   ├── source_reliability: from source type
    │   ├── corroboration_score: based on independent_sources count
    │   ├── recency_weight: temporal decay
    │   ├── verification_weight: based on who verified
    │   └── final_score: weighted average of above  
    ├── Update unit.confidence = breakdown.final_score
    └── Re-store (triggers re-indexing)
```
- **4.3.4.1** Temporal Decay Function
  - `recency_weight` must decrease as `(current_time - updated_at)` increases
  - **4.3.4.1.1** Decay Constants
    - Half-life of knowledge set to 180 days by default
    - **4.3.4.1.1.1** Exponential Decay Formula
      - `weight = base_confidence * e^(-λt)` where `λ = ln(2)/half_life`
      - **4.3.4.1.1.1.1** Periodic Recalculation Overhead
        - Iterating 1,000,000 facts to recalculate decay daily is too slow
        - **4.3.4.1.1.1.1.1** Deferred Decay Tracking
          - Only calculate current confident at `query()` time using the timestamp
          - **4.3.4.1.1.1.1.1.1** Decay-Aware Indexing
            - B+Tree indices on confidence must account for decay, requiring an append-only time-series index instead of an in-place mutation index

#### 4.3.5 Atomic Write Fix Detail (deployment-critical)

**Current flush() is NOT crash-safe:**
```omni
fn flush(&mut self):
    let data = serialize::to_json(self)
    fs::write_string(&data_path, &data)  # ← If crash here, data lost
```

**Required safe implementation details:**
```omni
fn flush(&mut self):
    if !self.dirty:
        return
    
    let data_path = format("{}/knowledge.json", self.storage_path)
    let tmp_path = format("{}/knowledge.json.tmp", self.storage_path)
    let bak_path = format("{}/knowledge.json.bak", self.storage_path)
    let data = serialize::to_json(self)
    
    # Step 1: Write to temp file
    match fs::write_string(&tmp_path, &data):
        Ok(_):
            # Step 2: Rename current -> backup (for rollback)
            if fs::exists(&data_path):
                fs::rename(&data_path, &bak_path)
            # Step 3: Rename temp -> current (atomic on most FSes)
            match fs::rename(&tmp_path, &data_path):
                Ok(_):
                    self.dirty = false
                    # Step 4: Delete backup
                    fs::delete(&bak_path).ok()
                Err(e):
                    # Rollback: restore backup
                    if fs::exists(&bak_path):
                        fs::rename(&bak_path, &data_path)
                    io::eprintln("CRITICAL: Knowledge flush failed: {}", e)
        Err(e):
            io::eprintln("Knowledge flush failed: {}", e)
```
- **4.3.5.1** OS Buffering and Sync
  - `fs::rename` changes the directory entry, but the file data might still be in OS RAM buffer
  - If power fails immediately after `rename`, the file might be 0 bytes
  - **4.3.5.1.1** `fsync` Requirement
    - Wait for disk controller confirmation before renaming
    - **4.3.5.1.1.1** Native Rust Mapping
      - Need new native binding: `io::file_sync(handle)` mapping to `File::sync_all()`
      - **4.3.5.1.1.1.1** Atomic Write Sequence
        - `write_string()` -> `file_open()` -> `file_sync()` -> `file_close()` -> `rename()`
        - **4.3.5.1.1.1.1.1** Directory Syncing
          - POSIX requires `fsync()` on the parent directory as well to persist the `rename()` operation
          - **4.3.5.1.1.1.1.1.1** Windows vs POSIX Differences
            - `ReplaceFileW` on Windows handles metadata sync gracefully, whereas Linux needs explicit `open(".", O_RDONLY)` and `fsync()`

### 4.4 Verification — Section 4

```
# 1. Flush-then-crash safety
- Write 1000 facts, flush
- Simulate crash (kill process mid-operation)
- Relaunch and verify all 1000 facts are intact

# 2. Confidence scoring
- Store a fact with explicit sub-scores
- Verify confidence_breakdown.final_score matches expected calculation

# 3. Checkpoint
- Create checkpoint, modify 10 facts, restore checkpoint
- Verify the 10 modifications are reverted
```

---

## 5. HELIOS Framework Completion

**Goal:** Implement the cognitive reasoning pipeline described in the spec, transforming the current stub into a functional multi-layer reasoning engine.

### 5.1 Current State of Cognitive Pipeline

The `helios-framework/helios/cognitive.omni` (10KB) is a high-level structural definition. The spec requires:

| Layer | Spec Section | Current State | Required |
|-------|-------------|---------------|----------|
| L0 Reflex | §9.1 | `helios-framework/brain/reflex/` (1 file) | Direct lookup in knowledge store |
| L1 RETE | §9.3 | Not implemented | Forward-chaining rule engine |
| L2 Backward Chain | §9.5 | Not implemented | Goal-driven proof search |
| L3 Analogy | §9.6 | Not implemented | Structure-mapping inference |
| L4 Deep Thought | §9.9 | `helios-framework/brain/deep_thought/` (1 file) | Pattern learning from experience |
- **5.1.0.1** Layered Execution Architecture
  - The cognitive engine queries layers sequentially (L0 -> L1 -> L2...)
  - **5.1.0.1.1** L0 Reflex System
    - Triggers instantly on exact string matches or simple regex patterns
    - **5.1.0.1.1.1** Pre-compiled Regex Cache
      - `Lazy<HashMap<String, Regex>>` to prevent compilation overhead on every input
      - **5.1.0.1.1.1.1** Early Exit Hook
        - L0 returning a `HighConfidence` answer terminates the pipeline immediately
        - **5.1.0.1.1.1.1.1** Priority Overrides
          - Safety rules (e.g., "stop", "cancel") bypass all reasoning and execute directly
  - **5.1.0.1.2** L1 RETE Forward Chaining
    - Maintains a stateful network of facts and rules
    - **5.1.0.1.2.1** RETE Node Types
      - *Alpha Nodes* evaluate conditions on single facts (e.g., `is_A(x)`)
      - *Beta Nodes* join facts across variables (e.g., `is_A(x) AND matches(x, y)`)
      - **5.1.0.1.2.1.1** Beta Memory Token Unification
        - Tokens passing Alpha nodes are buffered in Beta memory if the other side of the join is empty
        - **5.1.0.1.2.1.1.1** Partial Match Persistence
          - When new facts arrive, they only traverse the network downwards, never resetting state
          - **5.1.0.1.2.1.1.1.1** Rule Firing Queue
            - Terminal nodes insert `Activation` structs into an agenda sorted by rule salience
  - **5.1.0.1.3** L2 Backward Chaining (Prolog-style)
    - Starts with a goal `?x : capital_of(France, ?x)` and searches for proofs
    - **5.1.0.1.3.1** SLD Resolution Algorithm
      - Select literal, locate matching rule heads, substitute variables, push new sub-goals
      - **5.1.0.1.3.1.1** Occurs Check
        - Prevents infinite recursion during unification (e.g., `X = f(X)`)
        - **5.1.0.1.3.1.1.1** Trail/Undo Stack
          - When a branch fails, bindings made during unification must be rolled back
          - **5.1.0.1.3.1.1.1.1** Choice Point Backtracking
            - Store the program counter and stack state before venturing down a rule branch
  - **5.1.0.1.4** L3 Structure-Mapping Analogy
    - Finds isomorphisms between disconnected knowledge graphs
    - **5.1.0.1.4.1** Relational Graph Extraction
      - Convert `InformationUnit` sets into `Node -> Edge -> Node` semantic networks
      - **5.1.0.1.4.1.1** Graph Edit Distance (GED)
        - Heuristic matching to find sub-graphs with minimal substitution costs
        - **5.1.0.1.4.1.1.1** Systematicity Principle
          - Prefer mappings that carry higher-order relations (causes, implies) over simple attributes
          - **5.1.0.1.4.1.1.1.1** Candidate Inference Generation
            - If Base has `A -> B` and Target maps `A -> A'`, infer `A' -> B'` in Target
  - **5.1.0.1.5** L4 Deep Thought (Unsupervised Learning)
    - Runs in background idle time, searching for frequent sub-patterns in the Knowledge Store
    - **5.1.0.1.5.1** Apriori / FP-Growth Algorithms
      - Mine frequent itemsets from fact co-occurrences
      - **5.1.0.1.5.1.1** Concept Formation
        - If `{has_wings, has_feathers}` co-occur 95% of the time, propose new latent concept `BirdType`
        - **5.1.0.1.5.1.1.1** Hypothesis Generation
          - Emit speculative rules into L1 RETE with generic low confidence
          - **5.1.0.1.5.1.1.1.1** Hypothesis Validation Loop
            - If new facts violate the hypothesis, lower its confidence. If it drops below 10%, garbage collect it.

#### 5.1.1 Full Data Flow and Gaps

The cognitive system currently implements a 6-stage pipeline. Here is the **complete data flow** with types at each boundary and identified gaps:

```
USER INPUT (text: String, session_id: u64)
    │
    ▼
┌─────────────────────────────────────────────┐
│  1. CLASSIFY INTENT  (classify_intent)       │
│  Input: UserInput                            │
│  Logic: Pattern-match against word lists:    │
│    - ends with "?" → Question(text)          │
│    - starts with greeting word → Greeting    │
│    - starts with command word → Command      │
│    - default → Statement(text)               │
│  Output: Intent enum variant                 │
│                                              │
│  ⚠ GAPS:                                     │
│  - No entity extraction                      │
│  - No coreference resolution                 │
│  - No multi-sentence intent detection        │
│  - "ask" is classified as Statement, not     │
│    Question (it doesn't start with a wh-word)│
└──────────────┬──────────────────────────────┘
- **5.1.1.1.1** Intent Extraction Architecture
  - Basic regex checks are insufficient; needs full semantic parsing
  - **5.1.1.1.1.1** Dependency Parsing
    - Integrate a lightweight NLP parser or rely entirely on L0/L1 rules to categorize
    - **5.1.1.1.1.1.1** Coreference Resolution
      - "What is the capital of France? Is it nice?" -> "it" = "France"
      - **5.1.1.1.1.1.1.1** Discourse Context Buffer
        - Maintain `active_entities: Vec<MemoryItem>` in the session state
        - **5.1.1.1.1.1.1.1.1** Entity Decay
          - Entities exit the active buffer after 3 turns if not referenced
          - **5.1.1.1.1.1.1.1.1.1** Ambiguity Resolution Prompting
            - If "it" maps to two equally salient entities, pause pipeline and ask user
               │
               ▼
┌─────────────────────────────────────────────┐
│  2. MEMORY STORE  (mem::store)               │
│  Input: input.text as MemoryItem             │
│  Level: Working (short-term)                 │
│  Storage: mem::MemorySystem.store()          │
│                                              │
│  ⚠ GAPS:                                     │
│  - MemoryItem uses generic content String,   │
│    not structured InformationUnit            │
│  - Working memory is never queried for       │
│    conversation context                      │
└──────────────┬──────────────────────────────┘
- **5.1.1.2.1** Working Memory Structuring
  - Working memory must use `InformationUnit` to allow RETE network joins
  - **5.1.1.2.1.1** Ephemeral Fact Tagging
    - Add `is_ephemeral: bool` to `InformationUnit`
    - **5.1.1.2.1.1.1** TTL (Time-To-Live) Policies
      - Ephemeral facts are purged from memory after `session` ends
      - **5.1.1.2.1.1.1.1** Consolidation Trigger
        - If an ephemeral fact is recalled >3 times, promote it to long-term memory
        - **5.1.1.2.1.1.1.1.1** Promotion Conflict Resolution
          - If newly promoted fact contradicts long-term fact, schedule L4 Deep Thought reconciliation
          - **5.1.1.2.1.1.1.1.1.1** User-Verified Promotion
            - Facts explicitly affirmed by the user bypass TTL and persist immediately
               │
               ▼
┌─────────────────────────────────────────────┐
│  3. THINK  (think)                           │
│  Input: Context (session_id, history, topic) │
│  Step 3a: Recall relevant memories           │
│    - Keywords from active_topic              │
│    - memory.recall(RecallQuery) → Vec<Mem>   │
│    - Creates ThoughtStep with evidence        │
│  Step 3b: Logical reasoning (if topic set)   │
│    - re::ReasoningEngine.reason(Query)       │
│    - Query has (topic, predicate="about")    │
│    - Returns conclusion with explanation     │
│  Aggregate: avg confidence of all steps      │
- **5.1.1.3.1** Reasoning Engine Limitations
  - Current implementation only checks if a predicate exactly matches
  - **5.1.1.3.1.1** Multi-Step Deduction
    - Need backward chaining (L2) to construct paths: A->B, B->C, therefore A->C
    - **5.1.1.3.1.1.1** Depth-Bounded Search
      - Limit proof search depth to 5 to prevent combinatorial explosion
      - **5.1.1.3.1.1.1.1** Heuristic Path Pruning
        - Order sub-goals by selectivity (most restrictive clauses first)
        - **5.1.1.3.1.1.1.1.1** Confidence Attenuation
          - Each step in the inference chain multiplies confidence (0.9 * 0.9 = 0.81)
          - **5.1.1.3.1.1.1.1.1.1** Truth Thresholding
            - Terminate search branch if accumulated confidence drops below 0.5
│  Output: ThoughtChain { steps, conclusion,   │
│           total_confidence }                 │
│                                              │
│  ⚠ GAPS:                                     │
│  - Does NOT use KnowledgeStore at all!       │
│    (cognitive.omni doesn't import knowledge) │
│  - ReasoningEngine exists but depends on     │
│    brain::reasoning_engine which may be stub │
│  - No RETE forward chaining                  │
│  - No backward chaining / goal search        │
│  - No analogy detection                      │
│  - Only 2 thought steps maximum              │
└──────────────┬──────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────┐
│  4. RESPOND  (respond)                       │
│  Input: ThoughtChain                         │
│  Logic:                                      │
│    if confidence > 0.7: return conclusion     │
│    if confidence > 0.3: "I believe: ... not  │
│                          fully certain"       │
│    else: "I don't have enough information"   │
│  Output: Response { text, confidence,        │
│           sources=[], thought_process }      │
│                                              │
│  ⚠ GAPS:                                     │
│  - sources is always empty Vec               │
│  - No personality modulation applied         │
│  - No multi-sentence response generation     │
│  - Confidence thresholds are hardcoded       │
└──────────────┬──────────────────────────────┘
- **5.1.1.4.1** Natural Language Generation (NLG)
  - Raw logical conclusions (`capital_of(France, Paris)`) must be translated to English
  - **5.1.1.4.1.1** Template-Based Generation
    - Map predicates to sentence templates: `capital_of(X, Y) -> "The capital of {X} is {Y}."`
    - **5.1.1.4.1.1.1** Variable Binding Extraction
      - Extract AST variables from the winning `ThoughtChain` proof tree
      - **5.1.1.4.1.1.1.1** Discourse Marker Injection
        - If multiple steps, inject "Furthermore, ", "However, ", or "Therefore, "
        - **5.1.1.4.1.1.1.1.1** Persona Overrides
          - If agent `Persona` is "Analytical", favor "Given X, Y follows." over "I think Y!"
          - **5.1.1.4.1.1.1.1.1.1** Source Attribution Stringifying
            - Always append "[Source: X]" if the fact's `ConfidenceBreakdown` demands citation
               │
               ▼
┌─────────────────────────────────────────────┐
│  5. SAFETY CHECK  (safety.is_deceptive)      │
│  Input: Response                             │
│  Logic: Calls asimov::SafetyFramework        │
│  If deceptive: prepends "Note: this is my    │
│                best understanding. "          │
│  ⚠ depends on safety::asimov module          │
└──────────────┬──────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────┐
│  6. REFLECT  (reflect)                       │
│  Input: Interaction { input, response,       │
│          context, duration_ms }              │
│  Step 6a: Store as episodic memory           │
│    - "Q: {input} A: {response}" string       │
│    - Level: Episodic (long-term)             │
│  Step 6b: Feed to learning engine            │
│    - learn::Interaction { id, input, output, │
│      feedback=None, timestamp, context={} }  │
│    - learning.learn_from_interaction()       │
│  Step 6c: Periodic consolidation             │
│    - Every 10 turns: memory.consolidate()    │
│    - Moves Working → Long-term              │
│                                              │
│  ⚠ GAPS:                                     │
│  - Does NOT record to ExperienceLog          │
│  - Feedback field is always None             │
│  - Learning engine may be no-op stub         │
│  - No self-evaluation of response quality    │
└──────────────────────────────────────────────┘
```
- **5.1.1.6.1** Continuous Learning Loop
  - Reflection occurs post-response to update internal metrics
  - **5.1.1.6.1.1** Auto-Correction Mechanics
    - If user replies "No, that's wrong", the `ThoughtChain` used must be penalized
    - **5.1.1.6.1.1.1** Credit Assignment Problem
      - Did the rule fail, or was the base fact wrong? Both must have confidence decremented
      - **5.1.1.6.1.1.1.1** Unlearning Trigger
        - If base fact confidence drops below 10%, call `KnowledgeStore::delete()`
        - **5.1.1.6.1.1.1.1.1** Cascade Deletions
          - Any derived facts in the `history` graph must also be invalidated recursively
          - **5.1.1.6.1.1.1.1.1.1** Re-evaluation Event
            - Emit `RetractionEvent`, causing RETE to pull tokens back up the network

#### 5.1.2 What Must Be Connected

The cognitive pipeline in `cognitive.omni` does NOT connect to:
1. **KnowledgeStore** (`knowledge.omni`) — the think() phase queries `MemorySystem` and `ReasoningEngine` but never queries the `KnowledgeStore` where facts are stored via "remember" commands
2. **ExperienceLog** (`experience.omni`) — reflect() stores to `MemorySystem` and `LearningEngine` but never records `Experience::HeliosResponse` events
3. **CapabilityRegistry** (`capability.omni`) — if the intent is `Command`, capabilities should be dispatched

**Required integration code for `cognitive.omni`:**
```omni
import helios::knowledge::KnowledgeStore
import helios::experience::{ExperienceLog, Experience}
import helios::capability::CapabilityRegistry

struct HeliosCognitive:
    # ... existing fields ...
    knowledge: &mut KnowledgeStore      # ADD
    experience: &mut ExperienceLog      # ADD
```
- **5.1.2.1** Cross-Module Borrowing
  - Rust/Omni lifetimes will complain if `HeliosCognitive` holds `&mut` to Singletons
  - **5.1.2.1.1** Service Registry Pattern
    - Pass a `Context` bundle using `Rc<RefCell<ServiceRegistry>>` to the pipeline
    - **5.1.2.1.1.1** Re-entrant Queries
      - If an executing `Capability` needs to query `KnowledgeStore`, it must not cause a `RefCell` panic
      - **5.1.2.1.1.1.1** Read-Only Snapshot Isolation
        - Supply an immutable snapshot of `KnowledgeStore` indices to capabilities
        - **5.1.2.1.1.1.1.1** Dirty Read Overrides
          - Allow explicitly marked rules to read uncommitted transactions from Working Memory
          - **5.1.2.1.1.1.1.1.1** Sandboxed Capability Execution
            - Capabilities run in an isolated OmniVM context and cannot pollute global `ExperienceLog` without passing a strict `SafetyValidator` node first
    capabilities: &CapabilityRegistry   # ADD

fn think(&mut self, context: Context) -> ThoughtChain:
    # ... existing memory recall ...
    
    # NEW: Query KnowledgeStore for topic-specific facts
    if let Some(ref topic) = context.active_topic:
        let facts = self.knowledge.query(Query { subject: topic.clone(), ..default })
        if !facts.is_empty():
            steps.push(ThoughtStep {
                description: format("Found {} facts about '{}'", facts.len(), topic),
                evidence: facts.iter().map(|f| f.content.clone()).collect(),
                confidence: facts[0].confidence as f64 / 100.0,
            })

fn process_input(&mut self, input: UserInput) -> Response:
    # ... existing pipeline ...
    
    # NEW: Record to ExperienceLog
    self.experience.record(Experience::UserInput {
        input: input.clone(),
        timestamp: time::now(),
        session: input.session_id,
    })
    
    # ... after generating response ...
    
    # NEW: Handle commands via capabilities
    if let Intent::Command(action) = &intent:
        let params = extract_params(&action)
        let result = self.capabilities.execute(&action, &params, &ctx)
        # Use capability result instead of reasoning result
    
    # NEW: Record response
    self.experience.record(Experience::HeliosResponse {
        response: response.clone(),
        in_response_to: input_id,
        timestamp: time::now(),
    })
```

### 5.2 Implementation Tasks

- [ ] **L0 Reflex Layer** — Fast knowledge retrieval
  - **File:** `helios-framework/helios/cognitive.omni` — expand existing stub
  - The current `process_input()` in `runtime.omni` already does simple keyword dispatch (remember, ask, verify, etc.)
  - **Add:** Timeout enforcement — L0 must respond within 1ms (spec §9.1)
  - **Add:** Direct index lookup in `KnowledgeStore.by_subject` before falling through to deeper layers
  - **Test:** Query "What is France's capital?" when knowledge contains `(France, capital_of, Paris)` → instant response
  - **5.2.1.1** Reflex L0 Indexing Strategy
    - Map direct subject+predicate queries to `O(1)` memory lookups
    - **5.2.1.1.1** Compound Keys
      - Maintain `HashMap<(u32_sub, u32_pred), Vec<u64_fact>>` for instant O(1) multi-constraint resolution
      - **5.2.1.1.1.1** Key Encoding
        - `(subject_id << 32) | predicate_id` for cache-friendly single `u64` map keys
        - **5.2.1.1.1.1.1** L0 Timeout Enforcement Mechanisms
          - Run `process_input` wrapped in a loop checking `sys::time_now()`, yielding if `>1ms`
          - **5.2.1.1.1.1.1.1** Watchdog Cancellation
            - OVM needs an `OpCode::CheckTimeout` injected at loop boundaries by the bytecode compiler
            - **5.2.1.1.1.1.1.1.1** Interruption Recovery
              - If L0 yields, working memory buffers are cleaned up, control passes to L1 asynchronously

- [ ] **L1 RETE Forward Chaining**
  - **File:** Create `helios-framework/brain/rete.omni`
  - **Implementation:**
    ```omni
    struct ReteNetwork:
        alpha_nodes: Vec<AlphaNode>  # Pattern matchers on individual facts
        beta_nodes: Vec<BetaNode>    # Join nodes combining multiple patterns
        productions: Vec<Production> # Rules that fire when all conditions match
        working_memory: Vec<WorkingMemoryElement>
    
    struct Production:
        name: String
        conditions: Vec<Condition>   # LHS patterns
        action: Action               # RHS: assert new fact, retract, or side-effect
        salience: i32                # Priority (higher fires first)
    ```
  - **5.2.2.1** Rete Node Internal Layout
    - `AlphaNode` only checks constants (e.g. `subject == "France"`)
    - `BetaNode` checks variable consistency across joins (e.g. `fact1.object == fact2.subject`)
    - **5.2.2.1.1** Memory Management inside RETE
      - `AlphaMemory` and `BetaMemory` store references (`u64` Fact IDs), not duplicated facts
      - **5.2.2.1.1.1** Token Propagation
        - An `insert_fact` call pushes a token down the Alpha network, broadcasting to attached Beta nodes
        - **5.2.2.1.1.1.1** Join Evaluation
          - Beta Left Activation (new token from higher Beta): scan Right Memory
          - Beta Right Activation (new token from Alpha): scan Left Memory
          - **5.2.2.1.1.1.1.1** Join Optimization
            - Maintain `HashMap<Value, Vec<Token>>` on Beta inputs hashed by the join variable
            - **5.2.2.1.1.1.1.1.1** Salience Ordering Agenda
              - Fired rules go to `ConflictSet`, implemented as a `BTreeMap<i32, Vec<Activation>>` to always pop highest salience first
    
    fn add_production(&mut self, prod: Production)
    fn add_fact(&mut self, fact: InformationUnit) -> Vec<FiredRule>
    fn retract_fact(&mut self, fact_id: u64) -> Vec<FiredRule>
    fn run_cycle(&mut self) -> Vec<FiredRule>  # Fire all pending rules
    ```
  - **Spec reference:** §9.3 specifies incremental RETE with differential update
  - **Test:** Define rule "IF (X, is_a, Country) AND (X, has_capital, Y) THEN assert (Y, is_capital_of, X)" — add facts, verify derived fact appears

- [ ] **L2 Backward Chaining**
  - **File:** Create `helios-framework/brain/backward_chain.omni`
  - **Implementation:** Goal-driven proof search with depth-bounded DFS
  - Integrates with L1 productions as potential rules
  - **Test:** Query "Is Paris a capital?" when only facts are "(France, has_capital, Paris)" and rule "IF has_capital THEN is_capital_of" → proves goal
  - **5.2.3.1** SLD Resolution Engine
    - Implement a Warren Abstract Machine (WAM) inspired unification loop
    - **5.2.3.1.1** Variable Binding Environments
      - Arrays of `Option<Value>` indexed by a sequentially assigned variable ID during query parsing
      - **5.2.3.1.1.1** Trail Stack Management
        - When a variable is bound, push its ID to the `Trail`
        - **5.2.3.1.1.1.1** Choice Point Creation
          - When multiple rules match a goal, push a `ChoicePoint` containing the current `Trail` depth and environment state
          - **5.2.3.1.1.1.1.1** Backtracking Routine
            - On failure, pop `ChoicePoint`, unbind variables on `Trail` down to the saved depth, and try the next rule alternative
            - **5.2.3.1.1.1.1.1.1** Cut Operator (`!`)
              - Implement Prolog-style cut to explicitly discard `ChoicePoints` and prune the search tree to prevent infinite loops

- [ ] **Cognitive Cortex Integration**
  - **File:** Update `helios-framework/helios/cognitive.omni` to orchestrate L0→L1→L2 with budget enforcement
  - Each layer has a time budget; if exceeded, fall through to next layer or return partial answer
  - **Cross-reference:** §101 Deadline Scheduling
  - **5.2.4.1** Deadline Scheduling Orchestration
    - Provide `context.deadline_ms` to `think()`
    - **5.2.4.1.1** Preemption Checks
      - Inside RETE run cycles and L2 DFS loops, check `sys::time_now() > deadline`
      - **5.2.4.1.1.1** Partial Thought Yielding
        - If L2 times out, return the `ThoughtChain` constructed so far, marked as `Incomplete`
        - **5.2.4.1.1.1.1** Confidence Penalty
          - Apply a 0.5x multiplier to the `total_confidence` of `Incomplete` thought chains
          - **5.2.4.1.1.1.1.1** Graceful Degradation Response
            - If confidence < 0.3 due to timeout, trigger response: "I ran out of time analyzing this, but my initial thought is..."
            - **5.2.4.1.1.1.1.1.1** Background Continuation
              - Move the aborted `ThoughtChain` state machine to L4 Deep Thought to finish evaluating asynchronously

### 5.3 Experience Log Enhancement

- [ ] **Audit `helios-framework/helios/experience.omni`** (13KB)
  - Check if it implements the full `ExperienceRecord` taxonomy from spec §11.2
  - **Missing items per spec:** `QueryReceived`, `ResponseGenerated`, `KnowledgeModified`, `PluginInvoked`, `SelfModificationProposed`, `WebContentFetched`, `CapabilityExecuted`
  - **Add provenance chain tracking** using `AcquisitionHop` sequences (spec §1, §11)

#### 5.3.1 Complete Event Types (experience.omni, 388 LOC)

| Event Variant | Fields | When Recorded |
|---------------|--------|---------------|
| `UserInput` | `input`, `timestamp`, `session` | Every user message |
| `HeliosResponse` | `response`, `in_response_to`, `timestamp` | Every response |
| `SystemEvent` | `event` (String), `timestamp` | Startup, shutdown, config change |
| `LearningEvent` | `what_learned`, `source`, `information_ids`, `timestamp` | New fact stored |
| `ErrorEvent` | `error`, `context`, `timestamp` | Any error |
| `UserFeedback` | `feedback_type`, `about` (exp ID), `details`, `timestamp` | User rates response |
| `CapabilityUsed` | `capability_name`, `parameters`, `result`, `success`, `timestamp` | Any capability execution |
| `SelfModification` | `modification_type`, `details`, `approved_by_user`, `timestamp` | Self-growth proposals |

#### 5.3.2 Indices

- `by_session: HashMap<u64, Vec<usize>>` — session ID → experience indices
- `by_type: HashMap<ExperienceType, Vec<usize>>` — event type → indices
- `by_time: Vec<(u64, usize)>` — sorted by timestamp, enables binary search range queries
- **5.3.2.1** Semantic Vector Indexing
  - To support L4 deep thought, experiences need an embedding index (e.g. `Vec<f32>`)
  - **5.3.2.1.1** Local Embedding Generation
    - Use `Candle` or `ort` to run a quantized MiniLM model on `input` text
    - **5.3.2.1.1.1** HNSW Graph Implementation
      - Store embeddings in a Hierarchical Navigable Small World graph for fast Approximate Nearest Neighbors
      - **5.3.2.1.1.1.1** Disk-Backed HNSW
        - `mmap` the graph array to prevent massive RAM overhead upon startup
        - **5.3.2.1.1.1.1.1** Background Embedding Queue
          - Don't block the `UserInput` event loop; push strings to an MPSC channel for background embedding
          - **5.3.2.1.1.1.1.1.1** Batch Interference
            - Group strings into batches of 16 for SIMD/GPU acceleration during embedding generation

#### 5.3.3 Missing: Provenance Chain Tracking

Per spec §1/§11, each experience should record an `AcquisitionHop` chain:
```omni
struct AcquisitionHop:
    source_type: String     # "user_statement", "web_fetch", "derived"
    source_id: String       # Session ID or URL
    timestamp: u64
    transformation: String  # "verbatim", "summarized", "inferred"
```
This must be added to `Experience::LearningEvent` and `Experience::HeliosResponse`.
- **5.3.3.1** Provenance Serialization Overhead
  - String-based hops duplicate IDs heavily
  - **5.3.3.1.1** Compressed Hop Chains
    - Store `AcquisitionHop` as a struct of three `u64` (Type Enum, ID Hash, Timestamp)
    - **5.3.3.1.1.1** Source ID Hashing
      - Use `FxHash` of the URL or User ID to fit in 64 bits
      - **5.3.3.1.1.1.1** Hash Collision Fallback
        - Maintain a separate `HopDictionary: HashMap<u64, String>` for full URL retrieval
        - **5.3.3.1.1.1.1.1** Provenance Querying
          - Provide a `explain(fact_id)` function that walks the hop chain backwards
          - **5.3.3.1.1.1.1.1.1** Chain Visualization API
            - Export chains in Graphviz DOT format for the developer CLI

### 5.4 Capability System

- [ ] **Audit `helios-framework/helios/capability.omni`** (29KB — large)
  - This is the most substantial framework file — review what capabilities are registered
  - **Check for:** file system access, calculation, web search, grep/find, and knowledge operations
  - **Missing per spec:** Plugin-granted capabilities, capability audit logging, per-capability permission model
  - **Add:** Capability execution audit trail (log every capability use to experience log)
  - **5.4.0.1** Capability Permission Model
    - Capabilities like `file_delete` or `run_omni_code` are highly dangerous
    - **5.4.0.1.1** Capability Tiers
      - Enum: `Safe` (Read-only, sandboxed), `Privileged` (Write access, network), `Root` (System state)
      - **5.4.0.1.1.1** Authorization Interceptor
        - Before `dispatch_executor`, check if `context.session_tier >= capability.tier`
        - **5.4.0.1.1.1.1** User Confirmation Prompts
          - If a `Safe` session requests a `Privileged` capability, emit a UI confirmation request
          - **5.4.0.1.1.1.1.1** Audit Log Enforceability
            - Write to `ExperienceLog::CapabilityUsed` *before* execution starts, marking state `Pending`, then update to `Success/Fail`
            - **5.4.0.1.1.1.1.1.1** Capability Quotas
              - Prevent DoS by limiting `web_fetch` to 100/hour per session using a leaky bucket algorithm

#### 5.4.1 Full Call Path

```
User types "find all .omni files in helios-framework"
    ↓
runtime.omni::process_input(raw_input)
    ↓
input.omni::parse_input(raw) → Input::FindFiles { directory, pattern }  
    ↓
runtime.omni::handle_find_files()
    ↓  (current: direct fs call)
    ↓  (should be: capability dispatch)
capability.omni::execute("search_files", {
    "directory": "helios-framework",
    "pattern": "*.omni",
    "recursive": "true"
}, context)
    ↓
dispatch_executor("search_files", params, context)
    ↓
execute_search_files(params)
    ↓
collect_matching_files("helios-framework", "*.omni", true, &mut matches)
    ↓
fs::read_dir → recursive directory walk → glob match
    ↓
CapabilityResult::Success(Value::List(matched_paths))
```

#### 5.4.2 Missing Executors (need implementation)

| Executor | Category | What It Should Do |
|----------|----------|-------------------|
| `web_fetch` | web | HTTP GET on URL, return text content. Must add to `register_builtins()` |
| `store_knowledge` | knowledge | Call `KnowledgeStore.store()` |
| `query_knowledge` | knowledge | Call `KnowledgeStore.query()` |
| `verify_fact` | knowledge | Call `KnowledgeStore.verify()` |
| `recall_experience` | experience | Call `ExperienceLog.query()` |
| `run_omni_code` | code | Compile & execute Omni snippet in sandbox |
| `system_status` | system | Return memory, uptime, fact count, etc. |
- **5.4.2.1** `run_omni_code` Sandboxing Implementation
  - Calling OVM inside OVM requires a sub-runtime
  - **5.4.2.1.1** Nested OmniVM Instances
    - Spawn a fresh `OmniVM` struct with empty `globals`
    - **5.4.2.1.1.1** Capability Stripping
      - The nested VM must be given a `NativeManager` with NO file/net capabilities injected
      - **5.4.2.1.1.1.1** Instruction Budget
        - Inject `OpCode::CheckTimeout` or instruction counter to kill infinite loops using `max_instructions: 1_000_000`
        - **5.4.2.1.1.1.1.1** Memory Quotas
          - Pass an `allocator_limit: 16MB` to the nested VM's heap
          - **5.4.2.1.1.1.1.1.1** Result Serialization
            - The nested VM returns `VmValue`, which must be safely copied out to the parent VM via stringification or deep copy to prevent Cross-VM heap pointers

### 5.5 RETE Algorithm — Full Pseudocode

This must be implemented in `helios-framework/brain/rete.omni`.

#### 5.5.1 Data Structures

```omni
struct ReteNetwork:
    alpha_memories: Vec<AlphaMemory>    # One per unique pattern
    beta_memories: Vec<BetaMemory>      # One per join combination
    productions: Vec<Production>         # Rules with LHS conditions + RHS actions
    agenda: Vec<Activation>             # Pending rule firings, sorted by salience
    working_memory: Vec<WME>            # All current facts (Working Memory Elements)

struct AlphaMemory:
    pattern: Pattern                    # e.g., (?x, is_a, Country)
    matching_wmes: Vec<usize>           # Indices into working_memory
- **5.5.1.1** Alpha Node Optimization
  - Linear scan of `matching_wmes` per token is too slow
  - **5.5.1.1.1** Hashed Structural Matching
    - `AlphaMemory` must contain a hash table indexing facts by the variable positions
    - **5.5.1.1.1.1** Variable Mask Indexing
      - E.g., for `(?x, is_a, Country)`, maintain `HashMap<Value, Vec<usize>>` where key is the `subject`
      - **5.5.1.1.1.1.1** Fact Retraction Efficiency
        - Removing a fact requires finding it in `matching_wmes`
        - **5.5.1.1.1.1.1.1** Doubly Linked Fact Chains
          - Facts in working memory maintain a `Vec<*mut AlphaMemory>` back-pointer
          - **5.5.1.1.1.1.1.1.1** Safe Rust Graph Mapping
            - Use `petgraph` or `slab` allocator indices instead of raw pointers to maintain safety guarantees

struct BetaMemory:
    parent: Option<usize>              # Parent beta node index (or None for root)
    children: Vec<usize>               # Child beta node indices
    tokens: Vec<Token>                  # Partial match tuples

struct Token:
    bindings: HashMap<String, String>  # Variable bindings: ?x → "France"
    wme_indices: Vec<usize>            # Which WMEs matched

struct Production:
    name: String
    conditions: Vec<Pattern>            # LHS: conjunction of patterns
    action: Action                      # RHS: what to do when fired
    salience: i32                       # Priority (higher fires first)

struct Pattern:
    subject: PatternElement             # Literal string or Variable("?x")
    predicate: PatternElement
    object: PatternElement

enum PatternElement:
    Literal(String)
    Variable(String)                    # Starts with "?"
    Wildcard                            # Matches anything

enum Action:
    AssertFact(String, String, String)  # Assert new (subject, predicate, object)
    RetractFact(u64)                    # Remove fact by ID
    CallCapability(String, HashMap<String, String>)  # Invoke capability
    ModifyFact(u64, String, String)     # Change a fact's field
```
- **5.5.1.2** Rule Syntax Extraction
  - Translating Omni macro/DSL strings into `Pattern` and `Action` nodes
  - **5.5.1.2.1** Unification Variable Types
    - `String` bindings are slow; map `?x` to `[usize; N]` arrays where N is max variables per rule
    - **5.5.1.2.1.1** Array-Backed Tokens
      - `Token` payload becomes `{ wme_refs: [usize; 4], bindings: [ValueId; 4] }` avoiding `HashMap` overheads
      - **5.5.1.2.1.1.1** Memory Alignment
        - Token struct must fit in 64 bytes (L1 Cache Line) for rapid propagation
        - **5.5.1.2.1.1.1.1** Batch Propagation
          - Push blocks of 64 `Token`s to Beta Nodes simultaneously to leverage outer-loop SIMD where possible
          - **5.5.1.2.1.1.1.1.1** Early Termination
            - If Right Memory is empty, skip Left Activation loop entirely

#### 5.5.2 Add-Fact Algorithm (Incremental Update)

```
fn add_fact(wme: WME):
    working_memory.push(wme)
    let wme_idx = working_memory.len() - 1
    
    # Alpha filtering: check every alpha memory
    for alpha_mem in &mut alpha_memories:
        if alpha_mem.pattern.matches(&wme):
            alpha_mem.matching_wmes.push(wme_idx)
            
            # Beta propagation: join with existing tokens
            for beta_idx in alpha_mem.connected_betas:
                let beta = &mut beta_memories[beta_idx]
                
                # Try joining new WME with each existing token
                for token in &beta.parent_tokens():
                    if let Some(new_bindings) = try_join(token, &wme, &alpha_mem.pattern):
                        let new_token = Token {
                            bindings: merge(token.bindings, new_bindings),
                            wme_indices: token.wme_indices.with(wme_idx),
                        }
                        beta.tokens.push(new_token)
                        
                        # If this beta is the last in a production's LHS,
                        # create an activation
                        for prod in productions_using(beta_idx):
                            if all_conditions_satisfied(prod, &new_token):
                                agenda.push(Activation { 
                                    production: prod, 
                                    token: new_token.clone() 
                                })
    
    # Sort agenda by salience
    agenda.sort_by(|a, b| b.production.salience.cmp(&a.production.salience))

fn fire_cycle() -> Vec<FiredResult>:
    let mut results = Vec::new()
    while let Some(activation) = agenda.pop():
        let result = execute_action(activation.production.action, activation.token.bindings)
        results.push(result)
        
        # If the action asserted new facts, those go through add_fact too
        if let Action::AssertFact(s, p, o) = &activation.production.action:
            let new_wme = WME::new(substitute(s, bindings), substitute(p, bindings), substitute(o, bindings))
            add_fact(new_wme)  # Recursive! New fact may trigger more rules
    results
```
- **5.5.2.1** Infinite Loop Detection
  - A rule `A -> B` and `B -> A` causes `add_fact` to recurse infinitely until stack overflow
  - **5.5.2.1.1** Fact Deduplication
    - Intercept `AssertFact` before entering the network. Check if `working_memory` already contains this exact `(S,P,O)`
    - **5.5.2.1.1.1** The `Refraction` Principle
      - A specific `Production` should never fire twice on the exact same set of `Token` WME indices
      - **5.5.2.1.1.1.1** Fired-Sets Tracking
        - Add `fired_tokens: HashSet<u64>` (hash of `Token` indices) to `Production`
        - **5.5.2.1.1.1.1.1** Retraction Garbage Collection
          - When a fact is retracted, `Token`s containing it are destroyed. The `fired_tokens` set must also be purged of those hashes
          - **5.5.2.1.1.1.1.1.1** Token Reference Counting
            - Retraction must traverse down the Beta network: find tokens containing the retracted WME, delete them, and delete child tokens recursively

#### 5.5.3 Example Rule: Capital Derivation
```omni
let rule = Production {
    name: "derive_capital",
    conditions: vec![
        Pattern { subject: Variable("?country"), predicate: Literal("has_capital"), object: Variable("?city") }
    ],
    action: Action::AssertFact(
        Variable("?city"), "is_capital_of", Variable("?country")
    ),
    salience: 10,
}
```

When you add fact `(France, has_capital, Paris)`:
1. Alpha memory for pattern `(?, has_capital, ?)` matches
2. Beta token created with `{?country: France, ?city: Paris}`
3. All conditions satisfied → activation queued
4. Fire → assert `(Paris, is_capital_of, France)` into working memory
5. New fact may trigger further rules
- **5.5.3.1** Data-Driven Execution Trace
  - The RETE network eliminates polling; `is_capital_of` is asserted instantly upon addition of the premise
  - **5.5.3.1.1** Truth Maintenance System (TMS)
    - If `(France, has_capital, Paris)` is later retracted (e.g., user says "No, the capital is Lyon")
    - **5.5.3.1.1.1** Logical Dependencies
      - Derived fact `(Paris, is_capital_of, France)` must be automatically retracted
      - **5.5.3.1.1.1.1** Justification Graph
        - Fact metadata must store `derived_from: Option<Vec<u64>>`
        - **5.5.3.1.1.1.1.1** Cascading Retraction
          - `retract_fact` removes the base fact, checks the Justification Graph, and calls `retract_fact` on all dependents
          - **5.5.3.1.1.1.1.1.1** Multiple Support Handing
            - If `A` is derived from `B` AND derived from `C` independently, retracting `B` removes one support link, but `A` stays alive until `C` is also retracted

### Verification — Section 5

```
# 1. Cognitive layers respond correctly
helios> remember France: The capital is Paris
helios> ask What is the capital of France?
# Expected: "Paris" — via L0 direct lookup

# 2. RETE rule fires
# After adding a production rule and matching facts,
# verify derived facts appear in knowledge store

# 3. Experience log records all operations
# After a session, dump experience log and verify
# QueryReceived, ResponseGenerated, KnowledgeModified events present
```

---

## 6. Service Layer & IPC

**Goal:** Replace the current HTTP API with a named-pipe IPC service matching the spec, while keeping the HTTP API as a secondary interface.

### 6.1 Current Service Architecture

The service currently uses:
- `helios-framework/helios/api.omni` (6KB) — HTTP server on port 8765
- `helios-framework/helios/service.omni` — `HeliosService` struct with `run_api()` calling `api::run_server()`
- `helios-framework/main.omni` — `--service` flag triggers `svc.run_api()`
- **6.1.0.1** Dual-Protocol Listener Design
  - The service must eventually listen on both HTTP (dev) and Named Pipes (prod)
  - **6.1.0.1.1** Tokio Async Runtime
    - Rust's `tokio::select!` macro will poll both listeners concurrently
    - **6.1.0.1.1.1** Connection Multiplexing
      - Abstract incoming byte streams into a unified `RequestStream` trait
      - **6.1.0.1.1.1.1** Protocol Sniffing
        - Determine if incoming bytes are HTTP headers or length-prefixed binary IPC frames
        - **6.1.0.1.1.1.1.1** HTTP/1.1 Upgrade Header
          - Support upgrading HTTP connections to WebSockets for streaming Omni replies
          - **6.1.0.1.1.1.1.1.1** Ping/Pong Keepalive
            - Inject heartbeat frames every 15s to detect dead UI client connections

#### 6.1.1 Service State Transitions (service.omni)

```
         ┌──────────┐
    ┌───→│  Created  │
    │    └─────┬─────┘
    │          │ start()
    │          ▼
    │    ┌──────────┐
    │    │  Starting │──→ init runtime, load knowledge, start IPC
    │    └─────┬─────┘
    │          │ ready
    │          ▼
    │    ┌──────────┐
    │    │  Running  │◄──→ handle_request() loop
    │    └─────┬─────┘    │
    │          │ stop()   │ health_check() every 30s
    │          ▼          │   └→ memory, disk, facts count
    │    ┌──────────┐    │
    │    │ Stopping  │    │
    │    └─────┬─────┘    │
    │          │ flush()  │
    │          ▼          │
    │    ┌──────────┐    │
    └────│  Stopped  │    │
         └──────────┘    │
                         │
              error ─────┘
              ▼
         ┌──────────┐
         │  Error    │──→ log, attempt recovery or stay stopped
         └──────────┘
```

#### 6.1.2 Request Handling Loop (service.omni lines 108-168)

```omni
fn handle_request(&mut self, request: ServiceRequest) -> ServiceResponse:
    match request.request_type:
        "query":
            let result = self.runtime.process_input(&request.payload)
            ServiceResponse::success(result)
        
        "learn":
            let subject = request.payload.get("subject")
            let content = request.payload.get("content")
            self.runtime.knowledge.store(InformationUnit::new(subject, content))
            ServiceResponse::success("Stored")
        
        "status":
            let status = self.get_status()
            ServiceResponse::success(status)
        
        "health":
            ServiceResponse::success(self.health_check())
        
        _:
            ServiceResponse::error("Unknown request type")
```
- **6.1.2.1** Asynchronous Request Routing
  - The single `handle_request` function blocks the event loop
  - **6.1.2.1.1** Omni VM Thread Pooling
    - Spin up a thread pool for `process_input` to avoid stalling `status` loops
    - **6.1.2.1.1.1** Actor Model Mailboxing
      - Route `process_input` requests to a dedicated `CognitiveActor` via MPSC channels
      - **6.1.2.1.1.1.1** Queue Backpressure
        - If channel length > 100, return HTTP 429 Too Many Requests
        - **6.1.2.1.1.1.1.1** Load Shedding Prioritization
          - `status` and `health` requests bypass the queue and hit a lightweight system actor directly
          - **6.1.2.1.1.1.1.1.1** Priority Inversion Avoidance
            - Rust's standard MPSC is FIFO; replace with Crossbeam's priority queue or `flume` channels

#### 6.1.3 Config Loading Pipeline

**Current Config Sources**
1. `config/default.toml` (project root) — 1.9KB
2. `helios-framework/config/default.toml` — 2.2KB
3. `helios-framework/config/loader.omni` — 12KB
4. `helios-framework/config/default.omni` — 2KB

**Config Load Order (loader.omni)**
1. Load defaults from default.omni (hardcoded fallbacks)
2. Load config/default.toml (file-based overrides)
3. Check environment variables (HELIOS_* prefix)
4. Apply command-line arguments

**Critical Config Fields**
```toml
[helios]
data_dir = "~/.helios"           # Knowledge, experience, plugin storage
log_level = "info"
max_memory_mb = 512
- **6.1.3.1** Configuration Immutability Core
  - To implement the `[service]` rules dynamically
  - **6.1.3.1.1** Omni Sandbox Resource Tuning
    - The `max_memory_mb` limit must be explicitly parsed and passed to the `OmniVM` context bounds
    - **6.1.3.1.1.1** Environment Variable Override Precedence
      - Environment variable `HELIOS_MAX_MEMORY_MB` must shadow `config/default.toml` values mapping one-to-one
      - **6.1.3.1.1.1.1** CLI Arg Absolute Override
        - Starting with `--max-memory 1024` overrides the environment variable
        - **6.1.3.1.1.1.1.1** Dynamic Reloading SIGHUP
          - POSIX compliance requires config reloading on `SIGHUP` without restarting the process
          - **6.1.3.1.1.1.1.1.1** Rust `ArcSwap` Integration
            - Store the parsed config in `ArcSwap<Config>` so all threads instantly see changes atomically

[service]
http_port = 8765
ipc_pipe_name = "HeliosService"
auto_flush_interval_s = 60

[knowledge]
storage_format = "json"          # To be upgraded to "omk"
max_facts = 1000000
enable_compression = false

[gui]
theme = "dark"
font_size = 14
```

### 6.2 Implement Named Pipe IPC (Spec §12.5)

- [ ] **Create `helios-framework/helios/ipc.omni`**
  - Expose a raw byte stream interface over named pipes (Windows: `\\.\pipe\HeliosService`, POSIX: `/tmp/helios.sock`)
  - Must define the IPC frame format:
    - `u32` payload length (Little Endian)
    - `u8` message type (0x01=Query, 0x02=Response, 0x03=Event, 0x04=Error)
    - `u64` request ID
    - `[u8]` protobuf or JSON payload
  - **6.2.1.1** OS-Specific Socket Bindings
    - Create `ipc_windows.omni` and `ipc_posix.omni` abstraction layers
    - **6.2.1.1.1** Windows Named Pipe Security
      - Default pipes are accessible by any user unless secured
      - **6.2.1.1.1.1** ACL (Access Control List) Enforcement
        - Construct a `SECURITY_DESCRIPTOR` restricting access to the current `SID`
        - **6.2.1.1.1.1.1** Overlapped I/O
          - Pipe binding must use `FILE_FLAG_OVERLAPPED` for async `ConnectNamedPipe`
          - **6.2.1.1.1.1.1.1** Impersonation Hardening
            - Prevent clients from impersonating the service using `SECURITY_SQOS_PRESENT | SECURITY_IDENTIFICATION`
            - **6.2.1.1.1.1.1.1.1** Connection Limits
              - Prevent pipe exhaustion by limiting `PIPE_UNLIMITED_INSTANCES` to `MAX_IPC_CLIENTS = 16`
  - **Wire format:** 4-byte LE length prefix + MessagePack body
  - **Pipe name:** `\\.\pipe\HeliosService-<user-sid>` (Windows)
  - **Implementation sketch:**
    ```omni
    import core::io::named_pipe  # Must exist in std
    import core::serialize::msgpack
    
    struct IpcServer:
        pipe_name: String
        runtime: &mut Helios
    
    fn listen(&mut self):
        let pipe = named_pipe::create(&self.pipe_name)
        loop:
            let client = pipe.accept()
            # Read 4-byte length prefix
            let len_bytes = client.read_exact(4)
            let msg_len = u32::from_le_bytes(len_bytes)
            let msg_bytes = client.read_exact(msg_len as usize)
            let request: ServiceMessage = msgpack::deserialize(&msg_bytes)
            let response = self.handle(request)
            let resp_bytes = msgpack::serialize(&response)
            client.write(&(resp_bytes.len() as u32).to_le_bytes())
            client.write(&resp_bytes)
    ```
- **6.2.2.1** Streaming Large Payloads
  - `read_exact(msg_len)` will OOM if the client sends a 4GB payload length
  - **6.2.2.1.1** Chunked Message Reassembly
    - Read in 64KB chunks into a pre-allocated ring buffer
    - **6.2.2.1.1.1** Maximum Frame Size (MFS) Enforcement
      - Reject any frame where `msg_len > 16MB` immediately
      - **6.2.2.1.1.1.1** Zero-Copy Deserialization
        - Pass the ring buffer directly to `rmp_serde` without allocating a contiguous `Vec<u8>`
        - **6.2.2.1.1.1.1.1** Async Protocol State Machine
          - Replace the synchronous `loop` with `tokio_util::codec::LengthDelimitedCodec`
          - **6.2.2.1.1.1.1.1.1** Frame Header Abstraction
            - Codec must yield `(u64_id, u8_type, BytesMut_payload)` tuples to the router
  - **ServiceMessage enum** (from spec §12.5):
    ```omni
    enum ServiceMessage:
        Query(QueryRequest)
        LearnFact(LearnFactRequest)
        VerifyFact(VerifyFactRequest)
        GetStatus(StatusRequest)
        GetKnowledge(KnowledgeQuery)
        ApproveProposal(u64)
        RejectProposal(u64)
        TriggerCompaction
        CreateCheckpoint(String)
    ```

- [ ] **Check if `core::io` or `std::io` supports named pipes**
  - Look at `omni-lang/core/system.omni` (4KB) and `omni-lang/std/io.omni` (23KB)
  - **If missing:** Named pipe support must be added as a native function in `compiler/src/runtime/native.rs` or `compiler/src/runtime/os.rs` (9KB — already has OS-level functions)
  - **Windows API:** Use `CreateNamedPipeW`, `ConnectNamedPipe`, `ReadFile`, `WriteFile` via FFI
  - **6.2.3.1** Native FFI Bindings for Omni
    - `native.rs` must expose `io::pipe_create`, `io::pipe_accept`, `io::pipe_read`
    - **6.2.3.1.1** Handle Translation
      - OS `HANDLE` (Windows) and `fd` (POSIX) must be boxed in an opaque `RuntimeValue::Handle(usize)` to prevent Omni scripts from forging arbitrary FDs
      - **6.2.3.1.1.1** Asynchronous Yielding
        - `pipe_accept` cannot block the OS thread running the OVM evaluator
        - **6.2.3.1.1.1.1** Waker Integration
          - `pipe_accept` native function must return an `OpCode::Yield(FutureId)` instructing the OVM to park the green thread
          - **6.2.3.1.1.1.1.1** Event Loop Awakening
            - When `tokio` resolves the pipe connection, place the OVM green thread back on the `Ready` queue
            - **6.2.3.1.1.1.1.1.1** Drop Semantics
              - If the Omni script is killed, the `Handle(usize)` must trigger `CloseHandle` via a RAII drop guard in the Rust `NativeManager`

- [ ] **Check if MessagePack serialization exists**
  - Look at `omni-lang/std/serde.omni` (14KB) for serialization support
  - Look at `omni-lang/core/json.omni` (10KB) for deserialization patterns
  - **If missing:** Either implement MessagePack in Omni or add a native function backed by the `rmp-serde` Rust crate

- [ ] **Update `service.omni` to support both HTTP and IPC**
  - Add `--ipc` flag to `main.omni` alongside `--service`
  - When `--service` is used, start both HTTP API and IPC pipe listener
  - The GUI will connect via IPC; external tools via HTTP

#### 6.2.1 IPC Wire Protocol — Byte-Level Specification

**Frame Format**
```
┌────────────────┬────────────────────────────────────┐
│ Length (4 bytes)│ MessagePack Body (N bytes)          │
│ Little-Endian  │                                    │
│ u32            │                                    │
└────────────────┴────────────────────────────────────┘
```

**Message Types**
```
Request = {
    "id": u64,                    // Unique request ID
    "type": String,               // "query", "learn", "verify", "status", ...
    "payload": Map<String, Any>   // Type-specific data
}

Response = {
    "id": u64,                    // Echoes request ID
    "status": String,             // "ok", "error", "needs_approval"
    "payload": Any                // Type-specific response data
}
```

**Example: Query Request/Response**

Client sends:
```msgpack
{
    "id": 1,
    "type": "query",
    "payload": {
        "text": "What is the capital of France?",
        "session_id": 42
    }
}
```
Frame bytes: `[0x2F, 0x00, 0x00, 0x00, <47 bytes of msgpack>]`

Server responds:
```msgpack
{
    "id": 1,
    "status": "ok",
    "payload": {
        "text": "Paris",
        "confidence": 0.95,
        "sources": ["user_statement:session-12"],
        "thought_process": ["Found 1 facts about 'France'", "Direct knowledge match"]
    }
}
```
- **6.2.4.1** Structured Payload Typedefs
  - The `payload` field must strictly map to predefined Rust structs using `serde(tag)` for reliable deserialization
  - **6.2.4.1.1** Complex Variable Passing
    - E.g., `QueryRequest` needs an Omni AST context for variable substitutions injected from the GUI
    - **6.2.4.1.1.1** `Value` Enum Serialization
      - Omni `VmValue::List` and `VmValue::Object` must translate seamlessly to MessagePack arrays/maps
      - **6.2.4.1.1.1.1** Omni Object Preservation
        - Omni objects contain methods which cannot be serialized over IPC
        - **6.2.4.1.1.1.1.1** Method Stripping
          - Serialize only the data fields of an object (`InstanceData`), ignoring the `Class` vtable
          - **6.2.4.1.1.1.1.1.1** Rehydration Errors
             - Document that GUI clients cannot invoke Omni methods on received objects; they are pure Data Transfer Objects (DTOs)

### 6.3 Windows Service Registration

- [ ] **Create `tools/service_bridge.rs`** (Rust binary)
  - Uses `windows-service` crate for SCM integration
  - Commands: `install`, `uninstall`, `start`, `stop`
  - The service executable is the compiled `helios-framework/main.omni` (run via `omnc run --service`)
  - **See plan §3.2 for existing code sketch** — it's correct but paths need updating
  - **6.3.1.1** SCM State Control Callbacks
    - Register a service control handler catching `SERVICE_CONTROL_STOP` and `SERVICE_CONTROL_SHUTDOWN`
    - **6.3.1.1.1** Safe Delegation
      - SCM signals must be passed to the main `HeliosService` thread via an atomic `CancellationToken`
      - **6.3.1.1.1.1** Hard Kill Timers
        - SCM gives services exactly ~20 seconds to shut down. If token isn't respected, process is killed
        - **6.3.1.1.1.1.1** Recovery Actions
          - Configure the Windows SCM to restart the service on the first two crashes, but leave it stopped on the third
          - **6.3.1.1.1.1.1.1** Event Tracing for Windows (ETW)
            - Log service crashes into the Windows Event Viewer (`Application` log) under source `HeliosService`
            - **6.3.1.1.1.1.1.1.1** Provider Registration
              - Requires a `.man` manifest file compiled with `mc.exe` during the cargo build script

- [ ] **Create `scripts/service_install.ps1`**
  - Wrapper that calls `service_bridge.exe install` / `uninstall`
  - Must check for admin privileges first

### Verification — Section 6

```powershell
# 1. IPC pipe exists after service start
$pipe = Get-ChildItem \\.\pipe\ | Where-Object { $_.Name -match "HeliosService" }
# Expected: Pipe found

# 2. Send query via pipe
# Use a test client that connects, sends Query("ping"), reads response
# Expected: Response containing "pong" or status info

# 3. HTTP API still works
Invoke-RestMethod -Uri http://localhost:8765/status
# Expected: JSON status response
```

---

## 7. WinUI 3 Desktop GUI

**Goal:** A native Windows desktop application communicating with the HELIOS service.

### 7.1 Project Structure

- [ ] **Create `gui/WinUI3/` solution**
  ```
  gui/WinUI3/
  ├── HeliosGui.sln
  ├── HeliosGui/
  │   ├── App.xaml + App.xaml.cs
  │   ├── MainWindow.xaml + MainWindow.xaml.cs
  │   ├── Views/
  │   │   ├── ConversationPage.xaml     ← Main chat interface
  │   │   ├── KnowledgeBrowserPage.xaml ← Browse stored facts
  │   │   ├── CognitiveTracePage.xaml   ← View reasoning traces
  │   │   ├── SettingsPage.xaml         ← Config, themes, plugins
  │   │   └── LearningPage.xaml         ← Web learning queue
  │   ├── ViewModels/
  │   │   ├── ConversationViewModel.cs
  │   │   ├── KnowledgeViewModel.cs
  │   │   ├── CognitiveTraceViewModel.cs
  │   │   └── SettingsViewModel.cs
  │   ├── Services/
  │   │   ├── HeliosClient.cs           ← IPC client (named pipe + MessagePack)
  │   │   ├── NotificationService.cs
  │   │   └── ThemeService.cs
  │   ├── Themes/
  │   │   ├── LightTheme.xaml
  │   │   └── DarkTheme.xaml
  │   └── Resources/ (icons, fonts)
  └── HeliosGui.Tests/                  ← WinAppDriver tests
  ```

- [ ] **NuGet dependencies:** `Microsoft.WindowsAppSDK`, `MessagePack`, `System.IO.Pipelines`, `CommunityToolkit.Mvvm`
  - **7.1.0.1** Dependency Injection Container Setup
    - WinUI 3 does not come with a built-in DI framework like ASP.NET Core
    - **7.1.0.1.1** `Microsoft.Extensions.Hosting` Integration
      - Wrap the `App.xaml.cs` lifecycle in an `IHost` builder to manage service singletons
      - **7.1.0.1.1.1** ViewModel Registration
        - Register `ConversationViewModel` as `AddTransient` to allow multi-window chat sessions
        - **7.1.0.1.1.1.1** Service Locators vs Constructor Injection
          - Views must resolve ViewModels via `App.GetService<T>()` in their code-behind constructors
          - **7.1.0.1.1.1.1.1** Design-Time Data Support
            - If `Windows.ApplicationModel.DesignMode.DesignModeEnabled` is true, inject mock services instead
            - **7.1.0.1.1.1.1.1.1** Mock IPC Client
              - The mock IPC client returns hardcoded MessagePack byte arrays to populate XAML designer previews without running the Rust backend

### 7.2 IPC Client and ViewModels

- [ ] **Implement `HeliosClient.cs`** — Named Pipe IPC Client
  - Connect to `\\.\pipe\HeliosService-<user-sid>`
  - Send 4-byte LE length + MessagePack body
  - Handle reconnection, timeouts, serialization errors

```csharp
public class HeliosClient : IDisposable
{
    private NamedPipeClientStream _pipe;
    private readonly string _pipeName;
    private ulong _nextRequestId = 1;
    
    public async Task ConnectAsync(CancellationToken ct = default)
    {
        _pipe = new NamedPipeClientStream(".", _pipeName, 
            PipeDirection.InOut, PipeOptions.Asynchronous);
        await _pipe.ConnectAsync(5000, ct);
    }
    
    public async Task<ResponsePayload> QueryAsync(string text, ulong sessionId)
    {
        var request = new {
            id = _nextRequestId++,
            type = "query",
            payload = new { text, session_id = sessionId }
        };
        return await SendReceiveAsync(request);
    }
    
    private async Task<ResponsePayload> SendReceiveAsync(object request)
    {
        // Serialize to MessagePack
        byte[] body = MessagePackSerializer.Serialize(request);
        
        // Write 4-byte LE length prefix
        byte[] lenBytes = BitConverter.GetBytes((uint)body.Length);
        await _pipe.WriteAsync(lenBytes);
        await _pipe.WriteAsync(body);
        await _pipe.FlushAsync();
        
        // Read 4-byte LE length prefix
        byte[] respLenBytes = new byte[4];
        await _pipe.ReadExactlyAsync(respLenBytes);
        int respLen = BitConverter.ToInt32(respLenBytes);
        
        // Read response body
        byte[] respBody = new byte[respLen];
        await _pipe.ReadExactlyAsync(respBody);
        
        return MessagePackSerializer.Deserialize<ResponsePayload>(respBody);
    }
}
```
- **7.2.0.1** C# Async Synchronization Deadlocks
  - `.ConfigureAwait(false)` must be used on all `await` calls inside `SendReceiveAsync`
  - **7.2.0.1.1** UI Thread Blocking
    - If `QueryAsync` is `await`ed on the UI thread without `.ConfigureAwait(false)` in the library code, the WinUI dispatcher can deadlock
    - **7.2.0.1.1.1** `SynchronizationContext` Capturing
      - WinUI 3 components capture the `DispatcherQueueSynchronizationContext` implicitly
      - **7.2.0.1.1.1.1** IPC Read Timeout Handling
        - Pass a `CancellationToken` linked to a 30-second `CancellationTokenSource` to `ReadExactlyAsync`
        - **7.2.0.1.1.1.1.1** Pipe Broken Exceptions
          - Catch `IOException` (error code 109: ERROR_BROKEN_PIPE) when the Rust service crashes
          - **7.2.0.1.1.1.1.1.1** Automatic Reconnection Logic
            - On disconnect, transition `HeliosClient` to `Reconnecting` state, and retry `ConnectAsync` using exponential backoff (1s, 2s, 4s...)

- [ ] **Implement ViewModel Binding Pattern**

```csharp
public class ConversationViewModel : ObservableObject
{
    private readonly HeliosClient _client;
    
    [ObservableProperty]
    private ObservableCollection<MessageItem> _messages = new();
    
    [ObservableProperty]
    private string _inputText = "";
    
    [ObservableProperty]
    private bool _isConnected;
    
    [RelayCommand]
    private async Task SendMessageAsync()
    {
        if (string.IsNullOrWhiteSpace(InputText)) return;
        
        // Add user message to UI immediately
        Messages.Add(new MessageItem("user", InputText));
        var userText = InputText;
        InputText = "";
        
        try
        {
            // Send to service via IPC
            var response = await _client.QueryAsync(userText, _sessionId);
            
            // Add response to UI
            Messages.Add(new MessageItem("helios", response.Text) 
            {
                Confidence = response.Confidence,
                Sources = response.Sources,
                ThoughtProcess = response.ThoughtProcess
            });
        }
            Messages.Add(new MessageItem("error", $"Error: {ex.Message}"));
        }
    }
}
```
- **7.2.1.1** MVVM Memory Leak Prevention
  - `ObservableCollection` event handlers easily cause memory leaks in C# desktop apps
  - **7.2.1.1.1** Weak Event Patterns
    - The WinUI `ListView` subscribing to `CollectionChanged` keeps the ViewModel alive
    - **7.2.1.1.1.1** `INotifyPropertyChanged` Source Generators
      - `[ObservableProperty]` generates optimal IL that avoids boxing overhead during property updates
      - **7.2.1.1.1.1.1** Dispatcher Marshalling
        - Property updates triggered from background IPC threads *must* be marshalled to the UI thread
        - **7.2.1.1.1.1.1.1** `DispatcherQueue` Invocation
          - Wrap background mutations in `_dispatcherQueue.TryEnqueue(() => { Messages.Add(...) })`
          - **7.2.1.1.1.1.1.1.1** Batch Update Throttling
            - If receiving streaming tokens, batch `TryEnqueue` calls every 16ms (60 FPS) to prevent UI thread starvation and freezing

### 7.3 Accessibility & Theming (Spec §12.3)

- [ ] **WCAG 2.2 compliance:**
  - All controls have `AutomationProperties.Name`
  - Minimum 4.5:1 contrast ratio
  - Full keyboard navigation with `TabIndex`
  - **7.3.1.1** UI Automation (UIA) Tree Integration
    - Screen readers (Narrator/NVDA) rely on building an off-screen accessibility tree
    - **7.3.1.1.1** Custom `AutomationPeer` Implementations
      - Ensure the chat message history `ListView` announces new messages dynamically
      - **7.3.1.1.1.1** Live Regions (`LiveSetting`)
        - Set `AutomationProperties.LiveSetting = Assertive` on the notification banner area
        - **7.3.1.1.1.1.1** Tab Order Trapping Prevention
          - Ensure the user can `Shift+Tab` backwards out of the chat input `TextBox` without getting captured by internal autocomplete popups
          - **7.3.1.1.1.1.1.1** Focus Retrospection
            - When a dialog closes, `FocusManager.TryFocusAsync` must return focus to the exact button that launched it
            - **7.3.1.1.1.1.1.1.1** High Contrast Mode Reactivity
              - Bind control borders to `SystemControlHighlightBaseHighBrush` so they remain visible when High Contrast throws away `Acrylic` brushes
- [ ] **Dark mode** as first-class design token system
  - System-preference detection via `Application.Current.RequestedTheme`
  - Toggle in Settings page
  - **7.3.2.1** Resource Dictionary Switching
    - `App.xaml` must dynamically swap merged dictionaries when the theme changes
    - **7.3.2.1.1** `ThemeResource` vs `StaticResource`
      - UI brushes must exclusively use `ThemeResource` bindings to automatically react to OS-level theme toggles without requiring a restart
      - **7.3.2.1.1.1** Titlebar Customization
        - `#1E1E1E` dark mode requires extending the view into the title bar using `AppWindow.TitleBar`
        - **7.3.2.1.1.1.1** Mica / Acrylic Materials
          - Apply `MicaBackdrop` to `MainWindow` on Windows 11 dynamically, falling back to solid colors on Windows 10
          - **7.3.2.1.1.1.1.1** Fallback Rendering Policies
            - Detect `CompositionCapabilities.AreEffectsSupported()` to disable animations and blurs to save battery on low-end laptops
            - **7.3.2.1.1.1.1.1.1** Token Hierarchy
              - Define aliased tokens: `BrandColor -> PrimaryBrush -> ChatBubbleBackground` to centralize theme palette definitions

### 7.4 UI Tests

- [ ] **WinAppDriver tests** under `HeliosGui.Tests/`
  - Launch app, enter query, verify response appears
  - Switch themes, verify contrast
  - Navigate all pages via keyboard
  - **7.4.1.1** Native Desktop Automation
    - WinAppDriver (based on Appium/WebDriver) interacts with the UIA tree, not visual coordinates
    - **7.4.1.1.1** Element Locators
      - Use `AutomationId` exclusively for finding elements (e.g., `driver.FindElementByAccessibilityId("ChatInputBox")`)
      - **7.4.1.1.1.1** Explicit Waits
        - Await the appearance of `ResponseBubble` elements before querying text, as IPC delays are non-deterministic
        - **7.4.1.1.1.1.1** Page Object Model (POM)
          - Abstract specific screens into C# classes (`ConversationPageObj.cs`) to keep test files DRY
          - **7.4.1.1.1.1.1.1** Stubbing the Backend
            - GUI CI tests must use the `MockIpcClient` to avoid requiring the full Rust backend container to boot
            - **7.4.1.1.1.1.1.1.1** Mock Trace Validation
              - The test runner must verify that the mock client received exactly the `MessagePack` bytes expected from the ViewModel state transitions

### Verification — Section 7

```
# 1. GUI launches and connects to service
Launch HeliosGui.exe → Main window appears → Status bar shows "Connected"

# 2. Query works
Type "What do you know?" → Submit → Response appears in conversation panel

# 3. Accessibility
Run Accessibility Insights → Zero critical failures
```

---

## 8. Plugin Subsystem

**Goal:** Plugins compiled to OVM bytecode, loaded with manifest validation, sandboxed with capability checks.

### 8.1 Plugin Manifest & Loading

- [ ] **Create plugin infrastructure** (does NOT exist in current code):
  - `omni-lang/compiler/src/runtime/plugin.rs` — Manifest parsing, loading, isolation
  - `load_manifest(path)` → reads JSON, validates checksum with BLAKE3, returns `Result<Manifest, OvmError>`

**Manifest format** (`manifest.json`):
```json
{
  "name": "file_ingester",
  "version": "0.1.0",
  "entry": "plugin.ovc",
  "checksum": "<BLAKE3 hash>",
  "permissions": ["ReadFile", "WriteKnowledge"]
}
```
- **8.1.1.1** Cryptographic Identity Enforcement
  - Ensure plugin authors cannot impersonate core system plugins
  - **8.1.1.1.1** Package Signing
    - Include `"signature": "<Ed25519 hex>"` based on the developer's private key
    - **8.1.1.1.1.1** Signature Verification
      - `plugin::load_manifest` calls `ed25519_dalek::Verifier` on `checksum` bytes using the developer's public key
      - **8.1.1.1.1.1.1** Checksum Computation Architecture
        - BLAKE3 is used for its single-threaded 3 GB/s throughput to prevent plugin load lag
        - **8.1.1.1.1.1.1.1** TOCTOU Vectors
          - Mmap the `plugin.ovc` into RAM completely *before* hashing, to prevent a malicious process swapping the file *after* validation
          - **8.1.1.1.1.1.1.1.1** Sandboxed Environment Spawning
            - Re-verify checksum inside the `OmniVM` host struct construction to prevent internal state hijacking

### 8.2 Capability Enforcement & Sandboxing Mechanics

- [ ] **Implement Permission Model**
  - Parse permissions from manifest into a `PermissionSet` struct: `{ read_file, write_file, network, knowledge_read, knowledge_write, system }`
  - Assign this struct to the sandboxed `OmniVM` instance upon creation.

**Implementation in `vm.rs`:**
```rust
// Add to OmniVM struct:
pub permissions: PermissionSet,

#[derive(Default)]
pub struct PermissionSet {
    pub read_file: bool,
    pub write_file: bool,
    pub network: bool,
    pub knowledge_read: bool,
    pub knowledge_write: bool,
    pub system: bool,
}

// In the CallNative handler:
fn check_permission(&self, module: &str, func: &str) -> Result<(), OvmError> {
    let required = match (module, func) {
        ("io", "file_open") | ("io", "file_read_to_string") => "ReadFile",
        ("io", "file_create") | ("io", "file_write") | ("io", "file_delete") => "WriteFile",
        ("net", _) => "Network",
        ("sys", _) => "System",
        _ => return Ok(()),
    };
    if !self.permissions.has(required) {
        return Err(OvmError::PermissionDenied { capability: required.to_string() });
    }
    Ok(())
}
```

- [ ] **Create `check_capability!` macro** in `compiler/src/runtime/`
  - Invokes `check_permission` before every native function that accesses resources.
  - Return `OvmError::PermissionDenied` if not granted.
  - **8.2.1.1** Macro Code Injection Strategy
    - Wrap the body of `fn call_native` in `NativeManager` rather than copying to 50 locations
    - **8.2.1.1.1** Fine-Grained Auditing
      - The macro must also call `self.experience.record(CapabilityUsed { ... })`
      - **8.2.1.1.1.1** Resource Quotas Mapping
        - Besides permissions, implement `struct AllocLimits { max_fds, max_ram_mb, max_cpu_ms }`
        - **8.2.1.1.1.1.1** Instruction Budget Decrementing
          - Provide the `OmniVM` loop a `--max_instructions N` parameter; decrement every tick
          - **8.2.1.1.1.1.1.1** Hard Interruption Returns
            - On `InstructionsExhausted`, `vm.run()` returns `Err(OvmError::ResourceExhausted)`
            - **8.2.1.1.1.1.1.1.1** Plugin Isolation Restart
              - The main service process catches this error, unloads the plugin VM instance completely, and garbage collects it, refusing to load it again until service restart

### 8.3 Sample Plugins

- [ ] **Create `plugins/examples/file_ingester/`** — Reads a directory, inserts facts
- [ ] **Create `plugins/examples/math_rule/`** — Declares a production rule

### Verification — Section 8

```
# 1. Plugin with wrong checksum → rejected
Tamper with plugin.ovc content → load_manifest → OvmError::ChecksumMismatch

# 2. Plugin without WriteKnowledge permission → denied
Load plugin with only ReadFile permission → attempt write → PermissionDenied

# 3. Valid plugin executes successfully
Load file_ingester with correct permissions → facts appear in knowledge store
```

---

## 9. Standard Library Completion

**Goal:** Ensure all 33 std modules and 11 core modules are functional and compile.

### 9.1 Audit Each Module

- [ ] **Core modules** (`omni-lang/core/`):
  | Module | Size | Status Check |
  |--------|------|-------------|
  | `lib.omni` | 6KB | Verify exports |
  | `math.omni` | 6KB | Test basic ops |
  | `json.omni` | 10KB | Test parse/stringify |
  | `logging.omni` | 3KB | Test log levels |
  | `networking.omni` | 9KB | Test TCP client |
  | `system.omni` | 4KB | Test os(), args() |
  | `threading.omni` | 11KB | Test spawn/join |
  | `toml.omni` | 12KB | Test parse |
  | `http.omni` | 5KB | Test GET request |
  | `cuda.omni` | 5KB | GPU ops (optional) |
  | `voice.omni` | 13KB | Audio (deferred) |
  - #### 9.1.1 Core Module Dependency Graph
    - `lib.omni` imports nothing — this is the root
    - `math.omni` imports `lib.omni` — basic numeric operations
    - `json.omni` imports `lib.omni`, `collections` — JSON parse/stringify
    - `system.omni` imports `lib.omni` — OS-level queries
    - `http.omni` imports `networking.omni`, `json.omni` — HTTP over TCP
    - ##### 9.1.1.1 Native Function Dependencies per Core Module
      - `math.omni` → native `math::sin`, `math::cos`, `math::sqrt`, `math::pow`, `math::abs`
      - `system.omni` → native `sys::os_name`, `sys::num_cpus`, `sys::time_now`, `sys::sleep`
      - `json.omni` → native `json::parse`, `json::stringify` (requires serde_json in native.rs)
      - `networking.omni` → native `net::tcp_connect`, `net::tcp_write`, `net::http_get`
      - `threading.omni` → native `thread::spawn`, `thread::join` (NOT YET IN native.rs)
      - ###### 9.1.1.1.1 Missing Native Math Functions
        - `native.rs` currently has only `tensor_create` and `tensor_matmul` in math module
        - Missing: `math::sin`, `math::cos`, `math::tan`, `math::sqrt`, `math::pow`, `math::abs`, `math::floor`, `math::ceil`, `math::round`, `math::log`, `math::exp`
        - Implementation: `("math", "sin") => Ok(RuntimeValue::Float((args[0].as_float()?).sin()))`
        - **9.1.1.1.1.1** Float Precision Guarantees
          - All native math ops must use `f64` (IEEE 754 double precision)
          - **9.1.1.1.1.1.1** NaN / Infinity Handling
            - `math::sqrt(-1.0)` must return `RuntimeValue::Float(f64::NAN)`, not panic
            - **9.1.1.1.1.1.1.1** Omni Error Mapping
              - Expose `math::is_nan(x)` and `math::is_infinite(x)` as pure Omni builtins
              - **9.1.1.1.1.1.1.1.1** Deterministic PRNG
                - `math::random()` must use `ChaCha20Rng` seeded from `OsRng` for reproducible test builds
                - **9.1.1.1.1.1.1.1.1.1** Thread-Local RNG State
                  - Each OVM thread needs its own PRNG stream to prevent cross-thread correlation
      - ###### 9.1.1.1.2 String Module Native Requirements
        - `string::split(s, delimiter)` → can be Omni-native (no FFI needed)
        - `string::trim(s)` → Omni-native
        - `string::to_upper(s)` / `string::to_lower(s)` → Omni-native
        - `string::regex_match(s, pattern)` → requires native `regex::is_match` backed by `regex` crate

- [ ] **Std modules** (`omni-lang/std/`) — 33 files, 690KB total:
  - **Critical for deployment:** `core.omni`, `io.omni`, `fs.omni`, `string.omni`, `collections.omni`, `mem.omni`, `serde.omni`, `time.omni`, `sys.omni`
  - **Important:** `net.omni`, `crypto.omni`, `json.omni`, `regex.omni`, `thread.omni`, `async.omni`
  - **Deferred:** `tensor.omni`, `python.omni`, `image.omni`, `cuda`-related, `dist.omni`
  - **Test files:** `tests.omni` (14KB), `tests_comprehensive.omni` (16KB), `benchmarks.omni` (11KB)
  - #### 9.1.2 Deployment-Critical Std Module Checklist
    - ##### 9.1.2.1 io.omni (23KB) — File and Stream I/O
      - Must expose: `open(path, mode)`, `read_all(handle)`, `write(handle, data)`, `close(handle)`, `readline(handle)`
      - Must expose: `stdin`, `stdout`, `stderr` stream objects
      - Native backing: maps to `io::file_open`, `io::file_read_to_string`, `io::file_write`, `io::file_close` in `native.rs`
      - ###### 9.1.2.1.1 Buffered I/O
        - Wrap native file handles in `BufferedReader` / `BufferedWriter` for performance
        - Buffer size: 8KB default, configurable
        - Flush on `close()`, `flush()`, or when buffer full
        - **9.1.2.1.1.1** Ring Buffer Memory Layout
          - `struct BufferedReader { handle: u64, buf: [u8; 8192], pos: usize, limit: usize }`
          - **9.1.2.1.1.1.1** Read-Ahead Strategy
            - When `pos == limit`, issue a single native `io::file_read(handle, &mut buf)` for up to 8KB
            - **9.1.2.1.1.1.1.1** EOF Detection
              - Native `file_read` returns 0 bytes: set a `is_eof: bool` flag. Subsequent `readline()` returns `None`
              - **9.1.2.1.1.1.1.1.1** Encoding Support Plan
                - Default: UTF-8. Future: specify `encoding: "utf-16le"` for Windows text file compatibility
                - **9.1.2.1.1.1.1.1.1.1** BOM (Byte Order Mark) Stripping
                  - If first 3 bytes are `[0xEF, 0xBB, 0xBF]`, skip them before decoding as UTF-8
    - ##### 9.1.2.2 collections.omni — Data Structures
      - Must expose: `Vec<T>`, `HashMap<K, V>`, `HashSet<T>`, `LinkedList<T>`, `Stack<T>`, `Queue<T>`
      - All collections backed by Omni structs with native array/map primitives
      - ###### 9.1.2.2.1 HashMap Implementation Strategy
        - Option A: Backed by native Rust `HashMap` via FFI (fastest, less pure)
        - Option B: Implemented in pure Omni using open-addressing hash table (slower, self-contained)
        - **Decision for deployment:** Use Option A (native backing) for core collections
        - **9.1.2.2.1.1** Native HashMap FFI API
          - `map::create() -> handle`, `map::insert(handle, key, value)`, `map::get(handle, key) -> Option<value>`
          - **9.1.2.2.1.1.1** Key Hashing Contract
            - Omni keys must implement `fn hash() -> u64`. The native layer receives only `u64` hashes, not raw keys
            - **9.1.2.2.1.1.1.1** Collision Resolution
              - Rust's `HashMap` uses Robin Hood probing; this is transparent to Omni but requires stable `Hash` + `Eq` semantics
              - **9.1.2.2.1.1.1.1.1** Iterator Protocol
                - `map::keys(handle) -> Vec<key>` must clone keys out of the native map to prevent dangling references
                - **9.1.2.2.1.1.1.1.1.1** Capacity Pre-Allocation
                  - Expose `map::with_capacity(n)` to prevent excessive rehashing during bulk `store()` operations
    - ##### 9.1.2.3 serde.omni (14KB) — Serialization Framework
      - Must expose: `to_json(value)`, `from_json(text)`, `to_msgpack(value)`, `from_msgpack(bytes)`
      - JSON: backed by native `json::parse` / `json::stringify`
      - MessagePack: backed by native `msgpack::serialize` / `msgpack::deserialize` (requires rmp-serde)
      - ###### 9.1.2.3.1 Value ↔ Native Conversion
        - Omni `Value` type must map to `RuntimeValue` at the boundary
        - `Value::Map(entries)` → `RuntimeValue::Map(pairs)` → `serde_json::Value::Object`
        - `Value::Array(items)` → `RuntimeValue::Array(items)` → `serde_json::Value::Array`
        - **9.1.2.3.1.1** Deep Recursive Conversion
          - Nested Omni objects must be recursively walked: `Value::Object { fields }` → `serde_json::Map` per-field
          - **9.1.2.3.1.1.1** Circular Reference Safety
            - Track visited `ObjectId`s in a `HashSet` during conversion, emit `serde_json::Value::Null` if cycle detected
            - **9.1.2.3.1.1.1.1** Performance Budget
              - Target < 1ms for a 1000-field JSON stringify operation. Use `serde_json::to_writer` streaming to avoid full in-memory cloning
              - **9.1.2.3.1.1.1.1.1** Binary Serialization Alternatives
                - For IPC and Knowledge Store, `bincode` or `postcard` would be 5-10x faster than JSON
                - **9.1.2.3.1.1.1.1.1.1** Format Negotiation
                  - `serde.omni` must expose a `Serializer` trait, allowing users to plug in custom formats without modifying core code

### 9.2 Add Missing Spec Modules to Std

Per spec §14.1 (updated in this conversation), these modules must exist:

- [ ] `std/sync/crdt.omni` — CRDT primitives (§88)
  - #### 9.2.1 CRDT Stub Design
    - Define: `GCounter`, `PNCounter`, `GSet`, `ORSet`, `LWWRegister`
    - Each type has `merge(other)` method for conflict-free merging
    - Initial stub: define structs + method signatures, body returns `unimplemented()`
- [ ] `std/concurrency/stm.omni` — Software Transactional Memory (§89)
- [ ] `std/sec/pola.omni` — Capability-based security (§90)
- [ ] `std/typing/liquid.omni` — Liquid type refinements (§91)
- [ ] `std/crypto/pqc.omni` — Post-quantum crypto (§92)
- [ ] `std/net/quic.omni` — QUIC transport (§35)
- [ ] `std/net/mdns.omni` — mDNS discovery (§35.4)
- [ ] `std/collections/counting_bloom.omni` — Bloom filter (§102)
  - #### 9.2.2 Bloom Filter Implementation
    - `struct BloomFilter { bits: Vec<bool>, hash_count: usize, size: usize }`
    - `fn insert(&mut self, key: &str)` — hash `key` with `hash_count` different seeds, set bits
    - `fn may_contain(&self, key: &str) -> bool` — check all hash positions
    - `fn false_positive_rate(&self) -> f64` — estimate FP rate from current fill ratio
    - ##### 9.2.2.1 Hash Function Selection
      - Use BLAKE3 truncated to different lengths as N hash functions
      - `hash_i(key) = blake3(i || key) mod bit_array_size`
      - Native backing: `crypto::blake3_hash` from `native.rs`
      - **9.2.2.1.1** Optimal Bit Array Sizing
        - For 1% FP rate with N elements: `m = -N * ln(0.01) / (ln(2)^2)` bits
        - **9.2.2.1.1.1** Counting Bloom Extension
          - Replace `Vec<bool>` with `Vec<u8>` counters to support `remove()` operations
          - **9.2.2.1.1.1.1** Counter Overflow Protection
            - If any counter reaches 255, freeze the filter and rebuild with a larger `m`
            - **9.2.2.1.1.1.1.1** Streaming Insert Mode
              - Allow `insert_batch(keys: &[&str])` to hash all keys in a single BLAKE3 context
              - **9.2.2.1.1.1.1.1.1** Persistence Format
                - Serialize the bit array as a raw `Vec<u8>` blob prepended with `m`, `k`, and `n` header fields
- [ ] `std/alloc/arena.omni` — Arena allocator (§7.3)
- [ ] `std/alloc/slab.omni` — Slab allocator (§7.3)

**Note:** These are spec-required modules. For initial deployment, stub implementations that define the types and interfaces are acceptable; full implementations come later.

### Verification — Section 9

```powershell
# Compile every std module individually
Get-ChildItem omni-lang\std\*.omni | ForEach-Object {
    omnc compile $_.FullName
    if ($LASTEXITCODE -ne 0) { Write-Error "FAIL: $_" }
}
# Expected: All compile without errors

# Compile every core module individually
Get-ChildItem omni-lang\core\*.omni | ForEach-Object {
    omnc compile $_.FullName
    if ($LASTEXITCODE -ne 0) { Write-Error "FAIL: $_" }
}
# Expected: All compile without errors

# Run std test suite
omnc test omni-lang\std\tests.omni
omnc test omni-lang\std\tests_comprehensive.omni
# Expected: All tests pass
```

---

## 10. Testing Infrastructure

**Goal:** Comprehensive test coverage across all components.

### 10.1 Test Matrix

| Component | Test Type | Location | Runner |
|-----------|-----------|----------|--------|
| Compiler | Unit (inline) | `compiler/src/*/tests` | `cargo test` |
| Compiler | Integration | `compiler/tests/*.rs` (create) | `cargo test` |
| VM/Runtime | Unit | `compiler/src/runtime/tests.rs` | `cargo test` |
| Framework | Omni tests | `helios-framework/tests/*.omni` (create) | `omnc test` |
| Std Library | Omni tests | `omni-lang/std/tests*.omni` | `omnc test` |
| IPC/Service | PowerShell | `tests/service/*.ps1` (create) | Pester |
| GUI | C# + WinAppDriver | `gui/WinUI3/HeliosGui.Tests/` | dotnet test |
| Deployment | PowerShell | `tests/deployment/smoke.ps1` | Pester |
- **10.1.1** Test Configuration Management
  - All test runners must share a common `TEST_DATA_DIR` pointing to `tests/fixtures/`
  - **10.1.1.1** Test Isolation
    - Each test creates a temporary `data_dir` using `tempfile::TempDir` to prevent cross-test pollution
    - **10.1.1.1.1** Deterministic Timestamps
      - Inject a mock `sys::time_now()` returning fixed values during test runs to make snapshot-based assertions stable
      - **10.1.1.1.1.1** Test Coverage Targets
        - Compiler: 80% line coverage. Runtime: 90% branch coverage. Framework: 70% line coverage
        - **10.1.1.1.1.1.1** Coverage Tooling
          - Use `cargo-llvm-cov` for Rust code and a custom Omni coverage counter injected by the compiler (`--coverage` flag)
          - **10.1.1.1.1.1.1.1** Mutation Testing
            - Run `cargo-mutants` on `compiler/src/runtime/vm.rs` to verify the test suite detects seeded bugs
            - **10.1.1.1.1.1.1.1.1** Mutation Score Threshold
              - Require Mutation Score Indicator (MSI) ≥ 70% before merging critical runtime PRs

### 10.2 Create Missing Test Files

- [ ] Create `helios-framework/tests/` directory
- [ ] Add `helios-framework/tests/knowledge_test.omni` — Store, query, verify, flush tests
  - #### 10.2.1 knowledge_test.omni Detailed Scenarios
    - ##### 10.2.1.1 Store and Retrieve
      - Create `KnowledgeStore::new()`, store 5 facts with distinct subjects
      - Query each by subject → verify exact content returned
      - Query non-existent subject → verify empty result
    - ##### 10.2.1.2 Update and History
      - Store a fact, then update it with new content
      - Verify `history` field contains the original version
      - Verify only the updated version appears in query results
    - ##### 10.2.1.3 Verification and Confidence
      - Store a fact with default confidence
      - Call `verify(id, "admin", "confirmed correct", 3)`
      - Check: `confidence_breakdown.final_score` > initial confidence
      - Check: `confidence_breakdown.corroboration_score` reflects 3 independent sources
    - ##### 10.2.1.4 Flush and Reload
      - Store 10 facts, call `flush()`
      - Verify `knowledge.json` exists on disk
      - Create new `KnowledgeStore::load(path)`, verify all 10 facts present
      - ###### 10.2.1.4.1 Corruption Resistance
        - Write invalid JSON to `knowledge.json.tmp`
        - Verify `flush()` with temp+rename pattern doesn't corrupt the main file
    - ##### 10.2.1.5 Word Index Search
      - Store fact with content "The quick brown fox jumps over the lazy dog"
      - `search("brown fox")` → should find the fact
      - `search("purple cat")` → should return None
      - **10.2.1.5.1** Fuzzy Search Testing
        - `search("brwon fox")` (typo) should return empty unless Levenshtein fuzzy matching is enabled
        - **10.2.1.5.1.1** Tokenization Boundary Cases
          - Punctuation-adjacent words: `"hello, world"` should tokenize to `["hello", "world"]`
          - **10.2.1.5.1.1.1** Unicode Normalization
            - `search("é")` must match `"\u0065\u0301"` (NFC vs NFD) using Unicode canonical equivalence
            - **10.2.1.5.1.1.1.1** Case Folding
              - `search("BERLIN")` must find facts containing `"Berlin"` or `"berlin"`
              - **10.2.1.5.1.1.1.1.1** Stop Word Filtering
                - `search("the")` should return zero results if `"the"` is in the stop-word list
                - **10.2.1.5.1.1.1.1.1.1** Multi-Language Support Plan
                  - Future: provide per-language tokenizers (`en`, `de`, `ja`) with language-specific stop-word lists
- [ ] Add `helios-framework/tests/service_test.omni` — Service lifecycle, request/response
  - #### 10.2.2 service_test.omni Detailed Scenarios
    - Test `HeliosService::new()` → state is `Created`
    - Test `start()` → state transitions to `Running`
    - Test `handle_request({ type: "status" })` → returns service info
    - Test `handle_request({ type: "health" })` → returns health check data
    - Test `handle_request({ type: "unknown" })` → returns error response
    - Test `stop()` → state transitions to `Stopped`, knowledge flushed
- [ ] Add `helios-framework/tests/capability_test.omni` — Capability registration, execution, audit
  - #### 10.2.3 capability_test.omni Detailed Scenarios
    - Test `CapabilityRegistry::new()` has 7+ builtin capabilities registered
    - Test `execute("read_file", { path: "test.txt" }, ctx)` → returns file content
    - Test `execute("nonexistent", {}, ctx)` → returns Error("not found")
    - Test disabled capability → returns Error("disabled")
    - Test capability requiring confirmation → returns NeedsConfirmation
    - **10.2.3.1** Permission Escalation Testing
      - Execute a `WriteFile` capability from a session with `ReadOnly` tier → must fail with `PermissionDenied`
      - **10.2.3.1.1** Audit Trail Verification
        - After capability execution, query `ExperienceLog` for `CapabilityUsed` event → must exist with correct `parameters`
        - **10.2.3.1.1.1** Sandboxed Execution Timeout
          - Execute a capability that loops forever → verify `OvmError::ResourceExhausted` is raised within the instruction budget
          - **10.2.3.1.1.1.1** Concurrent Capability Execution
            - Execute 10 capabilities in parallel via `thread::spawn` → verify no data races in `NativeManager` file handle table
            - **10.2.3.1.1.1.1.1** Side-Effect Rollback
              - If a capability fails mid-execution, any partial writes to the KnowledgeStore must be rolled back
              - **10.2.3.1.1.1.1.1.1** Hermetic Test Infrastructure
                - Test fixtures must be self-contained zip archives unpacked to `tempdir` to prevent real file system pollution
- [ ] Create `tests/service/ipc_test.ps1` — Start service, send pipe query, verify response
  - #### 10.2.4 ipc_test.ps1 Implementation
    ```powershell
    Describe "HELIOS IPC" {
        BeforeAll {
            # Start service in background
            $proc = Start-Process -FilePath "omnc" -ArgumentList "run helios-framework\main.omni --service" -PassThru
            Start-Sleep -Seconds 3
        }
        
        It "Pipe exists" {
            Test-Path "\\.\pipe\HeliosService" | Should -BeTrue
        }
        
        It "Query returns response" {
            # Connect to pipe, send query, verify response
            $pipe = New-Object System.IO.Pipes.NamedPipeClientStream(".", "HeliosService", [System.IO.Pipes.PipeDirection]::InOut)
            $pipe.Connect(5000)
            # ... send and read ...
            $response.status | Should -Be "ok"
        }
        
        AfterAll {
            Stop-Process $proc -Force
        }
    }
    ```
- [ ] Create `tests/deployment/smoke.ps1` — Full install → start → query → stop → uninstall

### 10.3 Test Harness Script

- [ ] **Create `scripts/run-all-tests.ps1`**:
  ```powershell
  # Phase 1: Rust tests
  Push-Location omni-lang\compiler
  cargo test --all 2>&1 | Tee-Object test-compiler.log
  Pop-Location
  
  # Phase 2: Tools tests
  Push-Location omni-lang\tools
  cargo test --workspace 2>&1 | Tee-Object test-tools.log
  Pop-Location
  
  # Phase 3: Omni tests
  Get-ChildItem omni-lang\tests\*.omni |
      ForEach-Object { omnc test $_.FullName }
  
  # Phase 4: Framework tests
  Get-ChildItem helios-framework\tests\*.omni |
      ForEach-Object { omnc test $_.FullName }
  
  # Phase 5: Service integration
  Invoke-Pester tests\service\
  
  # Phase 6: Deployment smoke
  Invoke-Pester tests\deployment\
  ```
  - #### 10.3.1 Test Report Generation
    - Each phase writes to a log file in `test-results/`
    - At end of script, aggregate results:
      ```powershell
      $summary = @{
          CompilerTests = (Select-String -Path test-compiler.log -Pattern "test result").Line
          ToolsTests = (Select-String -Path test-tools.log -Pattern "test result").Line
          OmniTests = $omniTestCount
          FrameworkTests = $frameworkTestCount
          ServiceTests = $pesterResult.PassedCount
          DeploymentTests = $deployResult.PassedCount
      }
      $summary | ConvertTo-Json | Out-File test-results/summary.json
      ```
    - ##### 10.3.1.1 Exit Code Convention
      - Script exits with 0 if ALL phases pass
      - Script exits with 1 if ANY phase has failures
      - Each phase's failure count is recorded independently

### Verification — Section 10

```
# Run full test suite
.\scripts\run-all-tests.ps1
# Expected: All phases pass, zero failures

# Verify test report generated
Test-Path test-results/summary.json
# Expected: True

# Check test count
(Get-Content test-results/summary.json | ConvertFrom-Json).CompilerTests
# Expected: "test result: ok. 360+ passed; 0 failed; ..."
```

---

## 11. Build, Package & Deploy

**Goal:** Produce a distributable package containing all binaries, configs, and assets.

### 11.1 Build Pipeline

- [ ] **Update `build_and_deploy.ps1`** to produce all artifacts:
  ```powershell
  # Step 1: Build compiler
  Push-Location omni-lang\compiler
  cargo build --release
  Copy-Item target\release\omnc.exe ..\..\dist\bin\
  Pop-Location
  
  # Step 2: Build tools
  Push-Location omni-lang\tools
  cargo build --release --workspace
  Copy-Item target\release\opm.exe ..\..\dist\bin\
  Pop-Location
  
  # Step 3: Compile framework to bytecode
  .\dist\bin\omnc.exe compile helios-framework\main.omni -o dist\helios\main.ovc
  
  # Step 4: Build service bridge (Windows service wrapper)
  # cargo build --release -p service-bridge
  
  # Step 5: Build GUI
  # dotnet build gui\WinUI3\HeliosGui.sln --configuration Release
  
  # Step 6: Assemble dist/
  Copy-Item config\default.toml dist\helios\config\
  Copy-Item helios-framework\helios\*.omni dist\helios\framework\ (if runtime-loading)
  ```
  - #### 11.1.1 Build Step Dependencies
    - Step 1 has no dependencies — build first
    - Step 2 depends on Step 1 (tools import `omni_compiler` crate)
    - Step 3 depends on Step 1 (needs `omnc.exe` to compile framework)
    - Step 4 depends on nothing (standalone Rust binary)
    - Step 5 depends on nothing (standalone .NET build)
    - Step 6 depends on Steps 1-5 (assembles all outputs)
    - ##### 11.1.1.1 Parallel Build Optimization
      - Steps 1+4+5 can run in parallel (no inter-dependencies)
      - Step 2 runs after Step 1
      - Step 3 runs after Step 1
      - Step 6 runs last
      - ###### 11.1.1.1.1 Build Time Targets
        - Clean build: < 5 minutes on 8-core machine
        - Incremental build: < 30 seconds
        - Release build: < 10 minutes (with LTO optimization)
  - #### 11.1.2 Release Build Flags
    - `cargo build --release` enables LTO, strip symbols, optimize size
    - Add to `Cargo.toml`:
      ```toml
      [profile.release]
      lto = true
      codegen-units = 1
      strip = true
      opt-level = "z"  # optimize for size — deployment binary should be < 10MB
      ```
    - ##### 11.1.2.1 Binary Size Targets
      - `omnc.exe`: < 10MB (current: ~15MB without strip/LTO)
      - `opm.exe`: < 5MB
      - `helios-service.exe`: < 3MB
      - Total `dist/bin/`: < 25MB
      - **11.1.2.1.1** UPX Compression (Optional)
        - Post-LTO, UPX can halve binary sizes but increases startup time by ~50ms
        - **11.1.2.1.1.1** Code Signing
          - Sign all `.exe` files with an Authenticode certificate to prevent Windows SmartScreen warnings
          - **11.1.2.1.1.1.1** Timestamp Server
            - Use `http://timestamp.digicert.com` to ensure signatures remain valid after certificate expiry
            - **11.1.2.1.1.1.1.1** CI Secret Management
              - Store the `.pfx` signing certificate as a GitHub Actions secret: `SIGNING_CERT_BASE64`
              - **11.1.2.1.1.1.1.1.1** Sign Verification Script
                - `Get-AuthenticodeSignature dist\bin\omnc.exe | Select Status` must return `Valid`
                - **11.1.2.1.1.1.1.1.1.1** Reproducible Builds
                  - Set `SOURCE_DATE_EPOCH` environment variable and pinned `Cargo.lock` so binary hashes are deterministic across CI runs

### 11.2 Distribution Layout

```
dist/helios-1.0.0/
├── bin/
│   ├── omnc.exe          ← Compiler
│   ├── opm.exe           ← Package manager
│   ├── helios-service.exe ← Service bridge (Rust)
│   └── HeliosGui.exe     ← WinUI3 GUI
├── lib/
│   ├── main.ovc          ← Compiled framework bytecode
│   └── std/              ← Standard library (compiled)
├── config/
│   └── default.toml      ← Default configuration
├── plugins/              ← Plugin directory (empty initially)
├── data/                 ← Knowledge store location
└── scripts/
    ├── install.ps1
    └── uninstall.ps1
```

### 11.3 Installer

- [ ] **MSI via WiX Toolset** (or simpler: ZIP + install script)
  - #### 11.3.1 WiX Installer Structure
    - `installer/Product.wxs` — Main installer definition
    - `installer/Components.wxs` — File groups (binaries, configs, libs)
    - `installer/CustomActions.wxs` — Service registration, PATH update
    - ##### 11.3.1.1 Install Actions Sequence
      1. Copy files to `%ProgramFiles%\HELIOS\`
      2. Add `%ProgramFiles%\HELIOS\bin` to system PATH
      3. Register `HeliosService` as Windows service via `sc.exe create`
      4. Create Start Menu shortcut for `HeliosGui.exe`
      5. Register `.omni` file association → `omnc.exe`
      6. Create `%LOCALAPPDATA%\HELIOS\` for user data (knowledge, config)
      - ###### 11.3.1.1.1 Uninstall Actions
        - Stop `HeliosService` if running
        - Remove service registration
        - Remove PATH entry
        - Remove Start Menu shortcut
        - Prompt: "Delete user data?" → if yes, remove `%LOCALAPPDATA%\HELIOS\`
        - **11.3.1.1.1.1** Upgrade-In-Place Strategy
          - Detect existing `%ProgramFiles%\HELIOS\` and offer in-place upgrade vs clean install
          - **11.3.1.1.1.1.1** Data Migration
            - Compare `version` fields in `config/default.toml` old vs new
            - **11.3.1.1.1.1.1.1** Schema Versioning
              - `knowledge.json` header contains `"schema_version": 1`. If upgrader sees version 1, apply transform to version 2 format
              - **11.3.1.1.1.1.1.1.1** Rollback Snapshot
                - Before any data migration, create `data/backup-<timestamp>.zip` of the entire `data/` directory
                - **11.3.1.1.1.1.1.1.1.1** Silent Install Mode
                  - `msiexec /i helios.msi /qn ACCEPTEULA=1` for automated CI/CD deployment to test servers
- [ ] Register Windows Service during install
- [ ] Create Start Menu shortcut for GUI
- [ ] Register file associations for `.omni` files

### Verification — Section 11

```powershell
# Build produces complete dist
.\build_and_deploy.ps1
Test-Path dist\bin\omnc.exe       # True
Test-Path dist\bin\opm.exe        # True
Test-Path dist\helios\config\default.toml  # True
(Get-ChildItem dist -Recurse | Measure-Object).Count -gt 10  # True

# Check binary sizes
(Get-Item dist\bin\omnc.exe).Length / 1MB -lt 15  # True

# Verify dist is self-contained (no Rust/cargo required)
# Copy dist/ to a clean Windows 10 VM and run:
.\dist\bin\omnc.exe --version
# Expected: omnc 0.1.0
```

---

## 12. CI/CD Pipeline

### 12.1 GitHub Actions Workflow

- [ ] **Create `.github/workflows/ci.yml`**:
  ```yaml
  name: CI
  on: [push, pull_request]
  jobs:
    build-test:
      runs-on: windows-latest
      steps:
        - uses: actions/checkout@v4
        - uses: dtolnay/rust-toolchain@stable
        - uses: Swatinem/rust-cache@v2
        - run: cargo fmt --manifest-path omni-lang/compiler/Cargo.toml -- --check
        - run: cargo clippy --manifest-path omni-lang/compiler/Cargo.toml -- -D warnings
        - run: cargo test --manifest-path omni-lang/compiler/Cargo.toml
        - run: cargo build --manifest-path omni-lang/tools/Cargo.toml --workspace
        - run: powershell -File scripts/run-all-tests.ps1
  ```
  - #### 12.1.1 CI Job Breakdown
    - ##### 12.1.1.1 Lint Job (fast, runs first)
      - `cargo fmt -- --check` — formatting compliance
      - `cargo clippy -- -D warnings` — static analysis
      - Expected runtime: < 2 minutes
    - ##### 12.1.1.2 Build + Test Job
      - `cargo test` — compile and run all Rust tests
      - `cargo build --workspace` — build all tools
      - Expected runtime: < 10 minutes with caching
    - ##### 12.1.1.3 Integration Test Job (depends on Build)
      - `powershell -File scripts/run-all-tests.ps1`
      - Starts HELIOS service, runs IPC tests, stops service
      - Expected runtime: < 5 minutes
      - ###### 12.1.1.3.1 Test Artifact Upload
        - Upload `test-results/summary.json` as CI artifact
        - Upload `test-compiler.log` and `test-tools.log` for debugging
        - On failure: upload `target/debug/` build logs
        - **12.1.1.3.1.1** Flaky Test Handling
          - Auto-retry failed tests up to 2 times using `cargo-retry` before marking the job as failed
          - **12.1.1.3.1.1.1** Build Time Monitoring
            - Record per-step durations as CI annotations to detect performance regressions
            - **12.1.1.3.1.1.1.1** Cache Invalidation Strategy
              - Use `Swatinem/rust-cache@v2` with `key: ${{ hashFiles('**/Cargo.lock') }}` to maximize cache hits
              - **12.1.1.3.1.1.1.1.1** Scheduled Nightly Builds
                - `cron: '0 2 * * *'` runs full integration + fuzzing suite nightly without blocking PRs
                - **12.1.1.3.1.1.1.1.1.1** Security Scanning
                  - Run `cargo audit` and `cargo deny check` nightly to detect vulnerable dependencies

### 12.2 Quality Gates

- [ ] `cargo clippy -- -D warnings` — zero warnings
- [ ] No `panic!()` or `unwrap()` in production code (except in tests)
- [ ] All tests pass before merge
  - #### 12.2.1 Quality Gate Enforcement
    - Branch protection rule: require CI pass before merge to `main`
    - ##### 12.2.1.1 Panic/Unwrap Detection Script
      ```powershell
      # Run as CI step or pre-commit hook
      $violations = Select-String -Path "omni-lang\compiler\src\**\*.rs" `
          -Pattern "panic!\(|\.unwrap\(\)|\.expect\(" -Recurse |
          Where-Object { $_.Line -notmatch "#\[cfg\(test\)\]" -and $_.Path -notmatch "tests" }
      if ($violations.Count -gt 0) {
          Write-Error "Found $($violations.Count) panic/unwrap in production code"
          $violations | ForEach-Object { Write-Error $_.ToString() }
          exit 1
      }
      ```

### 12.3 Release Pipeline

- [ ] **Create `.github/workflows/release.yml`**:
  - Triggered on tag push `v*`
  - Builds release binaries
  - Creates GitHub Release with `dist/helios-*.zip` attached
  - #### 12.3.1 Release Artifact Checklist
    - `helios-1.0.0-windows-x64.zip` contains complete `dist/` layout
    - SHA256 checksum file: `helios-1.0.0-windows-x64.zip.sha256`
    - Release notes auto-generated from `CHANGELOG.md`

---

## 13. Documentation

- [ ] **`docs/BUILDING.md`** — Step-by-step build instructions
  - #### 13.1.1 BUILDING.md Structure
    - Prerequisites: Rust 1.75+, Windows 10 SDK, .NET 8+
    - ##### 13.1.1.1 Quick Start
      ```powershell
      git clone https://github.com/user/helios.git
      cd helios
      cargo build --manifest-path omni-lang/compiler/Cargo.toml --release
      cargo build --manifest-path omni-lang/tools/Cargo.toml --workspace --release
      ```
    - ##### 13.1.1.2 Troubleshooting Common Issues
      - `linker error: link.exe not found` → install Visual Studio Build Tools
      - `windows-sys compilation error` → update Rust to 1.75+ for Win32 API support
      - `tokio feature error` → ensure `features = ["full"]` in Cargo.toml
      - **13.1.1.2.1** Build Diagnostics Script
        - Create `scripts/doctor.ps1` checking: Rust version, Windows SDK path, .NET SDK version, disk space
        - **13.1.1.2.1.1** Version Pinning
          - Document exact minimum versions: `rustc 1.75.0`, `cargo 1.75.0`, `dotnet 8.0.100`
          - **13.1.1.2.1.1.1** Offline Build Support
            - Vendor all Cargo dependencies using `cargo vendor` and commit to `vendor/` for fully offline builds
            - **13.1.1.2.1.1.1.1** Developer Container
              - Provide a `devcontainer.json` for GitHub Codespaces/VS Code Remote Containers with all tools pre-installed
              - **13.1.1.2.1.1.1.1.1** Documentation Site Generation
                - Use `mdBook` to compile all `docs/*.md` files into a static HTML site deployable to GitHub Pages
                - **13.1.1.2.1.1.1.1.1.1** API Reference Auto-Generation
                  - `cargo doc --no-deps` for Rust code. Custom `omnc doc` command for Omni modules generating DocComment HTML
- [ ] **`docs/DEPLOYMENT.md`** — Install, configure, and run HELIOS
  - #### 13.1.2 DEPLOYMENT.md Structure
    - Installation from ZIP
    - Configuration: `default.toml` fields with descriptions
    - Starting the service: `helios-service.exe start`
    - GUI launch: `HeliosGui.exe`
    - ##### 13.1.2.1 Configuration Reference
      - Every `default.toml` key documented with type, default value, and description
      - Environment variable overrides: `HELIOS_DATA_DIR`, `HELIOS_LOG_LEVEL`, `HELIOS_HTTP_PORT`
- [ ] **`docs/ARCHITECTURE.md`** — High-level system architecture diagram
  - #### 13.1.3 ARCHITECTURE.md Content
    - Mermaid diagram showing: Compiler → OVM bytecode → Service (IPC) → GUI
    - Component diagram: Lexer → Parser → Semantic → CodeGen → VM
    - Data flow: User Input → Cognitive Pipeline → Knowledge Store → Response
- [ ] **`CHANGELOG.md`** — Version history and changes
- [ ] Update all `README.md` files in each directory
  - `omni-lang/README.md` — Omni language overview and build instructions
  - `helios-framework/README.md` — Framework architecture and module descriptions
  - `docs/README.md` — Documentation index

---

## 14. Deployment Verification Checklist & End-to-End Procedures

The system is deployment-ready when ALL of these pass:

- [ ] `cargo test` in `omni-lang/compiler` — 360+ tests pass
- [ ] `cargo build --workspace` in `omni-lang/tools` — All 4 tools build
- [ ] `omnc compile helios-framework/main.omni` — Compiles without errors
- [ ] `helios-service.exe install && Start-Service HeliosService` — Service starts
- [ ] Named pipe `\\.\pipe\HeliosService-*` exists while service runs
- [ ] GUI launches, connects, and can submit a query
- [ ] `helios> remember France: The capital is Paris` → fact stored
- [ ] `helios> ask What is the capital of France?` → "Paris" returned
- [ ] Plugin with incorrect checksum is rejected
- [ ] Service stops cleanly on `Stop-Service HeliosService`
- [ ] `dist/helios-1.0.0.zip` is < 100MB and contains all required files
- [ ] Clean install on a fresh Windows 10 machine works without Rust installed

### 14.1 Subsystem Acceptance Tests

#### Test 1: Full Cognitive Round-Trip
```
1. Start HELIOS runtime
2. Send: "remember France: The capital is Paris"
   Expected: KnowledgeStore.store() called, fact stored, ID returned
3. Send: "ask What is the capital of France?"
   Expected flow:
     a. classify_intent → Question("capital of france")
     b. think → knowledge.query({subject: "France"}) → finds fact
     c. respond → "Paris" (confidence > 0.7)
     d. reflect → ExperienceLog records UserInput + HeliosResponse
4. Verify: ExperienceLog has ≥ 4 entries (2 inputs, 2 responses)
5. Verify: KnowledgeStore.facts.len() == 1
```

#### Test 2: RETE Rule Firing
```
1. Add production rule:
   IF (?x, is_a, Country) AND (?x, has_capital, ?y) THEN assert (?y, is_capital_of, ?x)
2. Assert: (France, is_a, Country)
   Expected: Alpha match on condition 1, no complete match yet
3. Assert: (France, has_capital, Paris)  
   Expected: Alpha match on condition 2, beta join succeeds
   → Activation created, fired
   → New fact (Paris, is_capital_of, France) asserted
4. Query: knowledge.query({subject: "Paris", predicate: "is_capital_of"})
   Expected: Returns InformationUnit with object "France"
```

#### Test 3: IPC Named Pipe
```powershell
# Terminal 1: Start service
omnc run helios-framework/main.omni --service

# Terminal 2: Test pipe
$pipe = New-Object System.IO.Pipes.NamedPipeClientStream(".", "HeliosService", [System.IO.Pipes.PipeDirection]::InOut)
$pipe.Connect(5000)
# Send query message...
# Read response...
# Verify response contains expected data
```

#### Test 4: Plugin Sandboxing
```
1. Create plugin with only ReadFile permission
2. Plugin code attempts: io::file_create("evil.txt")
3. Expected: PermissionDenied error
4. Plugin code attempts: io::file_open("readme.txt")
5. Expected: Success (ReadFile is granted)
```

#### Test 5: Knowledge Crash Recovery
```
1. Store 100 facts, flush
2. Store 50 more facts (dirty=true, NOT flushed)
3. Kill process (simulate crash)
4. Restart, load knowledge store
5. Expected: Exactly 100 facts present (the 50 unflushed are lost, but no corruption)
6. Verify: knowledge.json is valid JSON (not truncated)
```
- **14.1.1** Performance Acceptance Criteria
  - L0 Reflex lookup: < 1ms latency
  - RETE rule firing: < 10ms for a 10-condition production on 10,000 facts
  - IPC round-trip (query+response): < 50ms
  - **14.1.1.1** Stress Testing
    - Insert 100,000 facts in a loop, verify no memory leaks using Windows Performance Monitor (`\Process(omnc)\Private Bytes`)
    - **14.1.1.1.1** Soak Test Duration
      - Run the service for 24 hours with random queries every 100ms and verify: memory stable, no response timeouts
      - **14.1.1.1.1.1** Profiling Artifacts
        - Capture `perf` flamegraphs during soak test and attach to the release ticket
        - **14.1.1.1.1.1.1** Regression Baselines
          - Store benchmark results in `benchmarks/baseline.json` and compare in CI using `cargo-criterion`
          - **14.1.1.1.1.1.1.1** Deployment Sign-Off
            - Require at least one human (project lead) to manually execute the full 14.1 acceptance test suite and sign off in the release issue before publishing
            - **14.1.1.1.1.1.1.1.1** Post-Deployment Monitoring
              - After first production deployment: monitor Windows Event Viewer for HeliosService errors for 48 hours

---

## Implementation Priority Order

| Priority | Section | Why |
|----------|---------|-----|
| **P0** | §2 Compiler Stabilization | Everything depends on a working compiler |
| **P0** | §3 OVM Runtime Consolidation | Framework can't run without a reliable VM |
| **P1** | §4 Knowledge Store Hardening | Data integrity is deployment-critical |
| **P1** | §5 Framework Completion | Core product functionality |
| **P1** | §6 Service Layer | Backend for GUI |
| **P2** | §10 Testing Infrastructure | Catches regressions |
| **P2** | §7 WinUI3 GUI | User-facing frontend |
| **P2** | §8 Plugin Subsystem | Extensibility |
| **P3** | §9 Std Library Completion | Full spec compliance |
| **P3** | §11 Build & Package | Distribution |
| **P3** | §12 CI/CD | Automation |
| **P3** | §13 Documentation | User support |

---

*End of Deployment Plan*
