# Omni Programming Language — Exhaustive Technical Audit

**Date:** 2026-04-10  
**Auditor:** Principal Language Engineer  
**Repository:** `d:\Project\Helios\omni-lang`

---

## PHASE 1: FULL PROJECT INGESTION

### 1.1 Repository Structure Mapping — File Inventory

| Path | Language | LOC (approx) | Apparent Purpose | Status |
|------|----------|------|------------------|--------|
| [compiler/src/main.rs](file:///d:/Project/Helios/omni-lang/compiler/src/main.rs) | Rust | 1241 | CLI entry + compilation pipeline | Active |
| [compiler/src/lib.rs](file:///d:/Project/Helios/omni-lang/compiler/src/lib.rs) | Rust | ~50 | Library root (re-exports) | Active |
| [compiler/src/lexer.rs](file:///d:/Project/Helios/omni-lang/compiler/src/lexer.rs) | Rust | 357 | OLD hand-written lexer (no INDENT/DEDENT) | Abandoned |
| [compiler/src/lexer/mod.rs](file:///d:/Project/Helios/omni-lang/compiler/src/lexer/mod.rs) | Rust | 839 | NEW Logos-based lexer with INDENT/DEDENT | Active |
| [compiler/src/parser.rs](file:///d:/Project/Helios/omni-lang/compiler/src/parser.rs) | Rust | ~520 | OLD recursive descent parser | Abandoned |
| [compiler/src/parser/mod.rs](file:///d:/Project/Helios/omni-lang/compiler/src/parser/mod.rs) | Rust | 3691 | NEW recursive descent parser with error recovery | Active |
| [compiler/src/parser/ast.rs](file:///d:/Project/Helios/omni-lang/compiler/src/parser/ast.rs) | Rust | 507 | AST node definitions (current) | Active |
| [compiler/src/ast.rs](file:///d:/Project/Helios/omni-lang/compiler/src/ast.rs) | Rust | ~70 | OLD AST definitions (Program, FunctionDef, etc.) | Abandoned |
| [compiler/src/ir.rs](file:///d:/Project/Helios/omni-lang/compiler/src/ir.rs) | Rust | 284 | OLD SSA IR generator (uses OLD AST types) | Abandoned |
| `compiler/src/ir/mod.rs` | Rust | ~300 | NEW IR module (bridges to codegen) | Active |
| [compiler/src/codegen.rs](file:///d:/Project/Helios/omni-lang/compiler/src/codegen.rs) | Rust | 303 | OLD LLVM codegen (string-based, uses OLD types) | Abandoned |
| [compiler/src/codegen/mod.rs](file:///d:/Project/Helios/omni-lang/compiler/src/codegen/mod.rs) | Rust | ~470 | NEW codegen dispatcher (OVM/LLVM/Native) | Active |
| [compiler/src/codegen/ovm.rs](file:///d:/Project/Helios/omni-lang/compiler/src/codegen/ovm.rs) | Rust | ~1100 | OVM bytecode emitter from typed AST | Active |
| [compiler/src/codegen/ovm_direct.rs](file:///d:/Project/Helios/omni-lang/compiler/src/codegen/ovm_direct.rs) | Rust | ~900 | Direct typed-AST→OVM codegen (bypasses IR) | Active |
| [compiler/src/codegen/native_codegen.rs](file:///d:/Project/Helios/omni-lang/compiler/src/codegen/native_codegen.rs) | Rust | ~2300 | x86-64 machine code emitter | Prototype |
| [compiler/src/codegen/native_linker.rs](file:///d:/Project/Helios/omni-lang/compiler/src/codegen/native_linker.rs) | Rust | ~1200 | ELF/PE/Mach-O linker | Prototype |
| [compiler/src/codegen/llvm_backend.rs](file:///d:/Project/Helios/omni-lang/compiler/src/codegen/llvm_backend.rs) | Rust | ~900 | LLVM backend (inkwell) | Stub |
| [compiler/src/codegen/jit.rs](file:///d:/Project/Helios/omni-lang/compiler/src/codegen/jit.rs) | Rust | ~1800 | JIT compiler | Prototype |
| [compiler/src/codegen/mlir.rs](file:///d:/Project/Helios/omni-lang/compiler/src/codegen/mlir.rs) | Rust | ~800 | MLIR integration | Stub |
| [compiler/src/codegen/optimizer.rs](file:///d:/Project/Helios/omni-lang/compiler/src/codegen/optimizer.rs) | Rust | ~2800 | IR optimizer (constant folding, DCE, etc.) | Prototype |
| [compiler/src/codegen/gpu_dispatch.rs](file:///d:/Project/Helios/omni-lang/compiler/src/codegen/gpu_dispatch.rs) | Rust | ~2100 | GPU dispatch | Prototype |
| [compiler/src/codegen/gpu_binary.rs](file:///d:/Project/Helios/omni-lang/compiler/src/codegen/gpu_binary.rs) | Rust | ~1300 | GPU binary | Prototype |
| [compiler/src/codegen/self_hosting.rs](file:///d:/Project/Helios/omni-lang/compiler/src/codegen/self_hosting.rs) | Rust | ~800 | Self-hosting codegen utilities | Prototype |
| [compiler/src/codegen/linker.rs](file:///d:/Project/Helios/omni-lang/compiler/src/codegen/linker.rs) | Rust | ~2300 | Extended linker | Prototype |
| [compiler/src/codegen/python_interop.rs](file:///d:/Project/Helios/omni-lang/compiler/src/codegen/python_interop.rs) | Rust | ~200 | Python interop | Stub |
| [compiler/src/codegen/cpp_interop.rs](file:///d:/Project/Helios/omni-lang/compiler/src/codegen/cpp_interop.rs) | Rust | ~70 | C++ interop | Stub |
| [compiler/src/codegen/dwarf.rs](file:///d:/Project/Helios/omni-lang/compiler/src/codegen/dwarf.rs) | Rust | ~400 | DWARF debug info emitter | Stub |
| [compiler/src/semantics.rs](file:///d:/Project/Helios/omni-lang/compiler/src/semantics.rs) | Rust | ~280 | OLD semantic checker (TypeChecker) | Abandoned |
| [compiler/src/semantic/mod.rs](file:///d:/Project/Helios/omni-lang/compiler/src/semantic/mod.rs) | Rust | 3091 | NEW semantic analyzer (Analyzer, typed AST) | Active |
| [compiler/src/semantic/type_inference.rs](file:///d:/Project/Helios/omni-lang/compiler/src/semantic/type_inference.rs) | Rust | ~2500 | Type inference engine (H-M based) | Active |
| [compiler/src/semantic/borrow_check.rs](file:///d:/Project/Helios/omni-lang/compiler/src/semantic/borrow_check.rs) | Rust | ~1700 | Borrow checker | Active |
| [compiler/src/semantic/effects.rs](file:///d:/Project/Helios/omni-lang/compiler/src/semantic/effects.rs) | Rust | ~600 | Effect system stubs | Stub |
| [compiler/src/semantic/traits.rs](file:///d:/Project/Helios/omni-lang/compiler/src/semantic/traits.rs) | Rust | ~450 | Trait resolver | Active |
| [compiler/src/semantic/constraints.rs](file:///d:/Project/Helios/omni-lang/compiler/src/semantic/constraints.rs) | Rust | ~500 | Constraint solver | Active |
| [compiler/src/semantic/monomorphization.rs](file:///d:/Project/Helios/omni-lang/compiler/src/semantic/monomorphization.rs) | Rust | ~450 | Generic monomorphization | Stub |
| [compiler/src/semantic/lifetimes.rs](file:///d:/Project/Helios/omni-lang/compiler/src/semantic/lifetimes.rs) | Rust | ~140 | Lifetime inference | Stub |
| [compiler/src/semantic/autograd.rs](file:///d:/Project/Helios/omni-lang/compiler/src/semantic/autograd.rs) | Rust | ~340 | Autodiff for AI | Premature |
| [compiler/src/semantic/properties.rs](file:///d:/Project/Helios/omni-lang/compiler/src/semantic/properties.rs) | Rust | ~380 | Sealed classes/properties | Active |
| [compiler/src/semantic/inference.rs](file:///d:/Project/Helios/omni-lang/compiler/src/semantic/inference.rs) | Rust | ~320 | Additional inference logic | Active |
| [compiler/src/semantic/edge_cases.rs](file:///d:/Project/Helios/omni-lang/compiler/src/semantic/edge_cases.rs) | Rust | ~270 | Edge case handling | Active |
| [compiler/src/semantic/error_recovery.rs](file:///d:/Project/Helios/omni-lang/compiler/src/semantic/error_recovery.rs) | Rust | ~300 | Semantic error recovery | Active |
| [compiler/src/semantic/tests.rs](file:///d:/Project/Helios/omni-lang/compiler/src/semantic/tests.rs) | Rust | ~300 | Unit tests for semantic | Active |
| [compiler/src/resolver.rs](file:///d:/Project/Helios/omni-lang/compiler/src/resolver.rs) | Rust | ~500 | Execution/memory/concurrency resolver | Active |
| [compiler/src/modes.rs](file:///d:/Project/Helios/omni-lang/compiler/src/modes.rs) | Rust | ~530 | Module mode checker (script/hosted/bare_metal) | Active |
| [compiler/src/monitor.rs](file:///d:/Project/Helios/omni-lang/compiler/src/monitor.rs) | Rust | ~100 | Runtime monitor (heartbeat, counters) | Active |
| [compiler/src/diagnostics.rs](file:///d:/Project/Helios/omni-lang/compiler/src/diagnostics.rs) | Rust | ~55 | Diagnostic types | Stub |
| [compiler/src/manifest.rs](file:///d:/Project/Helios/omni-lang/compiler/src/manifest.rs) | Rust | ~170 | omni.toml manifest parsing | Active |
| [compiler/src/enhancements.rs](file:///d:/Project/Helios/omni-lang/compiler/src/enhancements.rs) | Rust | ~350 | Misc enhancements | Active |
| [compiler/Cargo.toml](file:///d:/Project/Helios/omni-lang/compiler/Cargo.toml) | TOML | 92 | Monolithic crate with 25+ dependencies | Active |
| [ovm/src/main.rs](file:///d:/Project/Helios/omni-lang/ovm/src/main.rs) | Rust | 1464 | OVM bytecode virtual machine | Active |
| [ovm/ovm.c](file:///d:/Project/Helios/omni-lang/ovm/ovm.c) | C | ~2000 | C implementation of OVM | Prototype |
| [stage1-compiler/src/main.rs](file:///d:/Project/Helios/omni-lang/stage1-compiler/src/main.rs) | Rust | 134 | Minimal compiler using OLD files | Active |
| `std/*.omni` | Omni | ~37 files | Standard library (all spec-level stubs) | Stub |
| `omni/compiler/**` | Omni | ~15 files | Self-hosting compiler in Omni | Prototype |
| `omni/stdlib/**` | Omni | ~15 files | Self-hosting stdlib in Omni | Prototype |
| `tools/omni-lsp/` | Rust | ~3 files | LSP server (basic) | Stub |
| `tools/omni-fmt/` | Rust | ~2 files | Formatter | Stub |
| `tools/omni-dap/` | Rust | ~2 files | DAP server | Stub |
| `tools/opm/` | Rust | ~3 files | Package manager | Stub |
| `tools/vscode-omni/` | TypeScript | ~5 files | VS Code extension | Active |
| `examples/*.omni` | Omni | ~65 files | Example programs + OVM bytecode | Active |
| `diagnostics/` | Log | 624 files | Monitor stall logs (all from one session) | Abandoned |
| `build/` | Mixed | ~50 files | Build outputs, stage0 exe, empty stage1/2 | Active |

### 1.2 Codebase Summary

The repository contains **two parallel compiler implementations** that share no code:

1. **OLD system** ([lexer.rs](file:///d:/Project/Helios/omni-lang/compiler/src/lexer.rs), [parser.rs](file:///d:/Project/Helios/omni-lang/compiler/src/parser.rs), [ast.rs](file:///d:/Project/Helios/omni-lang/compiler/src/ast.rs), [ir.rs](file:///d:/Project/Helios/omni-lang/compiler/src/ir.rs), [codegen.rs](file:///d:/Project/Helios/omni-lang/compiler/src/codegen.rs), [semantics.rs](file:///d:/Project/Helios/omni-lang/compiler/src/semantics.rs)): A brace-delimited, C-like language. Used only by `stage1-compiler/`. Types include `Program`, `FunctionDef`, `VariableDecl`, `Type::String`, `Type::Void`, `BinaryOp::Add/Subtract/Multiply` — completely different from the current AST.

2. **NEW system** ([lexer/mod.rs](file:///d:/Project/Helios/omni-lang/compiler/src/lexer/mod.rs), [parser/mod.rs](file:///d:/Project/Helios/omni-lang/compiler/src/parser/mod.rs), [parser/ast.rs](file:///d:/Project/Helios/omni-lang/compiler/src/parser/ast.rs), [semantic/mod.rs](file:///d:/Project/Helios/omni-lang/compiler/src/semantic/mod.rs), [codegen/mod.rs](file:///d:/Project/Helios/omni-lang/compiler/src/codegen/mod.rs) + 28 submodules): Indentation-based, Python-like syntax. Types include [Module](file:///d:/Project/Helios/omni-lang/compiler/src/parser/ast.rs#22-25), [Function](file:///d:/Project/Helios/omni-lang/compiler/src/parser/ast.rs#124-132), [StructDef](file:///d:/Project/Helios/omni-lang/compiler/src/parser/ast.rs#107-114), `Type::Str`, `BinaryOp::Add/Sub/Mul` — the actual Omni language. Used by [main.rs](file:///d:/Project/Helios/omni-lang/ovm/src/main.rs).

> [!CAUTION]
> The [ir.rs](file:///d:/Project/Helios/omni-lang/compiler/src/ir.rs) and [codegen.rs](file:///d:/Project/Helios/omni-lang/compiler/src/codegen.rs) files reference types from [ast.rs](file:///d:/Project/Helios/omni-lang/compiler/src/ast.rs) (OLD system), but the active pipeline in [main.rs](file:///d:/Project/Helios/omni-lang/ovm/src/main.rs) uses [parser/ast.rs](file:///d:/Project/Helios/omni-lang/compiler/src/parser/ast.rs) (NEW system). These are **incompatible**. The active OVM codegen path ([codegen/ovm_direct.rs](file:///d:/Project/Helios/omni-lang/compiler/src/codegen/ovm_direct.rs)) bypasses the OLD IR entirely.

### 1.3 Build and Compilation Status

- **`cargo build --workspace`**: Root workspace now exists at `omni-lang/Cargo.toml`; legacy per-crate lockfiles remain, but builds can be driven from the repository root.
- **Parallel OLD/NEW modules**: Both exist in `compiler/src/` simultaneously. The [main.rs](file:///d:/Project/Helios/omni-lang/ovm/src/main.rs) uses `#[path = ...]` attributes to import the NEW modules while the OLD files coexist.
- **CI**: No CI configuration found (no `.github/workflows/`, no `.gitlab-ci.yml`, no `Cargo.workspace` at root).
- **`omnc-stage0.exe`**: A 3.8MB compiled binary exists in `build/`.
- **`omnc-stage1.ovm` / `omnc-stage2.ovm`**: Both are **empty files** (0 bytes).

### 1.4 Test Coverage

- **Lexer tests**: 15 tests in `compiler/src/lexer/mod.rs` (keywords, operators, literals, comments, indentation).
- **Parser tests**: None found in `parser/mod.rs`.
- **Semantic tests**: `compiler/src/semantic/tests.rs` (~300 LOC), `comprehensive_tests.rs` (~30 LOC stub).
- **Stage1 tests**: 4 unit tests in `stage1-compiler/src/main.rs`.
- **Integration tests**: `tests/comprehensive_test_suite.rs` (~400 LOC), `tests/integration/` (exists).
- **Example programs**: 65 `.omni` and `.ovm` files with varying complexity.
- **Zero test coverage**: Parser (0 tests), IR generation (0 tests), codegen (0 tests), OVM (0 tests), tools (0 tests).

---

## PHASE 3: REQUIREMENT-BY-REQUIREMENT AUDIT (Selected Critical Items)

### REQ-02-01: Hybrid Type System with Explicit Modes
**Specification:** Static by default with optional dynamic zones  
**Status:** ⚠️ Partially Implemented  
**Evidence:** `parser/ast.rs:175-227` defines primitive types (U8-F64, Bool, Str), Named, Generic, Function types. `Type::Any` acts as a wildcard. No dynamic zone mechanism.  
**What exists:** Static type annotations, type inference via `semantic/type_inference.rs` (H-M based, ~2500 LOC).  
**What is missing:** Dynamic typing zones, explicit mode switching, bidirectional inference.  
**Immediate action:** Type system works for basic scenarios but lacks mode-driven behavior.

### REQ-02-03: No Null in Safe Code / Option Types
**Specification:** No null in safe code; Option types  
**Status:** ⚠️ Partially Implemented  
**Evidence:** `parser/ast.rs:445-446` has `Expression::None` and `Expression::Some(Box<Expression>)`. `Literal::Null` exists at line 461. The type `Type::Nullable(Box<Type>)` exists. The semantic analyzer uses `Type::Any` for builtins which bypasses null safety.  
**What is missing:** Compiler does not enforce null-safety. `Literal::Null` is freely usable.

### REQ-03-01: Ownership-Based Memory Model
**Specification:** Single ownership, move semantics, `&T` / `&mut T`  
**Status:** ⚠️ Partially Implemented  
**Evidence:** `parser/ast.rs:290-297` defines `Ownership` enum (Owned/Borrow/BorrowMut/Shared/RawPointer). `semantic/borrow_check.rs` (60K, ~1700 LOC) implements borrow checking. `Expression::Borrow` and `Expression::Deref` exist.  
**What is missing:** No Polonius algorithm. Borrow checker is AST-walk based (not MIR-based). No field projections. No generational references.  
**Quality:** The borrow checker is an overlay on the semantic analyzer, not a proper dataflow analysis.

### REQ-04: Effect System
**Specification:** Built-in effects (io, async, throw, panic, alloc, rand, time, log, pure), effect inference, handlers  
**Status:** 🟡 Stub/Placeholder  
**Evidence:** `semantic/effects.rs` exists (20K, ~600 LOC) but is not integrated into the compilation pipeline. `main.rs` never calls any effect-related code. No effect annotations in the parser.  
**What is missing:** Everything. No `/ effects` syntax, no effect inference, no handlers, no capability alignment.

### REQ-05: Concurrency Model
**Specification:** Structured concurrency, actors, channels, spawn_scope  
**Status:** 🟡 Stub/Placeholder  
**Evidence:** `parser/ast.rs:351-354` has `Statement::Spawn` and `Statement::Select`. The lexer recognizes `spawn`, `select`, `yield`, `async`, `await`. The semantic analyzer can lower these to typed AST variants. The OVM VM does not implement any concurrency.  
**What is missing:** No structured concurrency enforcement. No `spawn_scope`. No actors. No channels. No `Send`/`Sync` checking.

### REQ-06-01: Indentation-Based Blocks
**Specification:** Indentation-based by default; braces in advanced modes  
**Status:** ✅ Implemented  
**Evidence:** `lexer/mod.rs:425-465` handles INDENT/DEDENT synthesis using an indent stack. Parser handles both colon+indent and brace-delimited blocks.

### REQ-06-05: Comment Syntax (`--`)
**Specification:** `--` for line comments  
**Status:** ❌ Not Implemented  
**Evidence:** `lexer/mod.rs:299` uses `//` for line comments, not `--`. The specification says `--` but the implementation uses `//` (C-style) and `#` (Python-style hash comments for non-attribute use).

### REQ-06-09: String Interpolation (`f"..."`, `d"..."`)
**Specification:** `f"Hello {name}!"` for Display, `d"..."` for Debug  
**Status:** ❌ Not Implemented  
**Evidence:** `lexer/mod.rs:201` recognizes only standard `"..."` strings. No `f"..."` or `d"..."` prefix handling.

### REQ-07: Module / Package / Visibility System
**Specification:** Layered visibility (pub, pub(mod), pub(pkg), pub(cap:X), pub(friend:module))  
**Status:** ⚠️ Partially Implemented  
**Evidence:** `parser/mod.rs:526-530` treats `pub` as a decorator attribute (`@pub`), not a proper visibility modifier. No `pub(mod)`, `pub(pkg)`, etc. Module declarations exist. Import resolution works (lines 621-892 in `main.rs`).  
**What is missing:** All layered visibility, capability-based module boundaries, `omni.toml` capability declarations.

### REQ-08: Error Handling
**Specification:** Result types, `?` operator, error sets, typed error context chains  
**Status:** ⚠️ Partially Implemented  
**Evidence:** `parser/ast.rs:447-448` has `Expression::Ok(Box<Expression>)` and `Expression::Err(Box<Expression>)`. The lexer recognizes `try`, `catch`, `finally`, `?`. The parser handles `?` for try-operator.  
**What is missing:** Error sets, `|>` context chains, structured error types.

### REQ-10: Compilation Model
**Specification:** CST→AST→MIR→Borrow-checked MIR→LIR→Codegen, Polonius borrow checker, Salsa incremental  
**Status:** ❌ Not Implemented (to spec)  
**Evidence:** Pipeline is Source→Tokens→AST→TypedAST→OVM bytecode. No CST (the parser produces a lossy AST). No MIR. No LIR. No Salsa. No Polonius. The borrow checker operates on the raw AST, not a control-flow graph.  
**What exists:** A working end-to-end pipeline for the OVM target that produces executable bytecode.

### REQ-12: Tooling
**Specification:** Rich CLI (build, test, bench, fmt, lint, doc, clean, add, remove, update, publish, profile, debug, fix, verify, semver-check, migrate)  
**Status:** 🟡 Stub/Placeholder  
**Evidence:** `tools/opm/` has `main.rs` (~200 LOC) with basic `init`, `add`, `build`, `run` commands. `tools/omni-lsp/` has `server.rs` and `main.rs`. `tools/omni-fmt/` has `formatter.rs`. All are minimal stubs.  
**What exists:** CLI flags in `omnc` for `--run`, `--emit-ast`, `--emit-ir`, `--emit-tokens`, `--target`, `--monitor`.  
**What is missing:** `omni test`, `omni bench`, `omni doc`, `omni lint`, `omni fix`, `omni clean`, `omni verify`, `omni semver-check`.

---

## PHASE 4: COMPONENT-LEVEL ANALYSIS

### COMP-01: Lexer
- **Exists:** Yes, two versions. The active one (`lexer/mod.rs`, 839 LOC) uses Logos for tokenization.
- **INDENT/DEDENT:** ✅ Implemented via indent stack (`lines 425-465`).
- **String interpolation (`f"..."`):** ❌ Not implemented.
- **Token kinds:** ~80 token kinds including all basic keywords, operators, delimiters, numeric literal variants (hex, binary, octal, float with exponent).
- **Error recovery:** Returns `LexerError` on unexpected character. No recovery — aborts on first error.
- **Fuzz target:** ❌ None.
- **Arena-allocated:** ❌ No. Uses `Vec<char>` for input.
- **Tests:** 15 unit tests covering keywords, operators, literals, comments, indentation.
- **Missing keywords for spec:** No `inout`, `linear`, `pure`, `effect`, `gen`, `arena`. No `--` comment syntax.

### COMP-02: Parser
- **Exists:** Yes, 3691 LOC recursive descent with error recovery.
- **Pratt for expressions:** ⚠️ Uses a precedence table (`BinaryOp::precedence()`) but not a proper Pratt parser. Expression parsing is recursive descent with manual precedence climbing.
- **Syntax forms:** Supports: functions, structs, enums, traits, impl blocks, modules, imports, extern blocks, const/static, comptime, macros, match/if/for/while/loop, closures, async/await, spawn/select/yield.
- **Missing syntax:** No effect annotations (`/ effects`), no `inout` parameters, no `linear` modifier, no `let`-chains, no deconstructing parameters, no async closures.
- **Error recovery:** ✅ Panic-mode recovery with synchronization sets. 50-error limit. "Did you mean?" suggestions.
- **CST vs AST:** Produces a lossy AST (not a lossless CST). No Rowan.
- **UI tests:** ❌ No `.omni` + `.ast` + `.stderr` triplet tests.
- **Parser tests:** ❌ Zero parser-specific unit tests.

### COMP-03: AST
- **Node hierarchy:** Comprehensive — Module, 12 Item variants, Statement (15 variants), Expression (25+ variants), Type (27 variants), Pattern (5 variants).
- **Arena-allocated:** ❌ No. Uses `Box`, `Vec`, `String` heap allocations.
- **Visitor trait:** ❌ No visitor or fold trait.
- **Pretty-printer:** ❌ No. Uses `Debug` derive.
- **Spans on nodes:** ❌ No spans. Only tokens have spans. Once parsed to AST, location information is lost from expressions and statements.

### COMP-04: Name Resolution
- **Exists:** ⚠️ Partially. Import resolution exists in `main.rs:621-892`. The semantic `Analyzer` has scope-based symbol lookup with `define_symbol`/`resolve_symbol`.
- **Two-pass:** ❌ No. Single-pass, which means forward references may fail.
- **DefIds:** ❌ No. Symbols are looked up by string name.
- **"Did you mean?":** ✅ In the parser (for keyword typos). ❌ Not in name resolution.
- **Visibility enforcement:** ❌ No. `pub` is stored as an attribute string, not enforced.

### COMP-05: Type Inference and Checking
- **Exists:** Yes. `semantic/type_inference.rs` (~2500 LOC).
- **Algorithm:** Claims Hindley-Milner. Actually a constraint-based system with unification.
- **Generics:** ⚠️ Partial. Generic syntax parses. Monomorphization exists as a stub (`monomorphization.rs`, ~450 LOC).
- **Trait bounds:** ⚠️ Partial. `traits.rs` (~450 LOC) has trait resolution.
- **Effect sets:** ❌ Not in type checking.
- **Hard/soft error classification:** ✅ `is_hard_type_error()` classifies errors.

### COMP-06: Effect System
- **Exists:** 🟡 Stub only. `semantic/effects.rs` (20K, ~600 LOC) defines types but is never called from the pipeline.
- **Built-in effects:** Defined in the file but never used.
- **Effect inference:** ❌ Not implemented.
- **User-defined effects:** ❌ Not implemented.
- **Async as effect:** ❌ Not implemented.

### COMP-07: MIR
- **Exists:** ❌ No MIR exists. The `ir.rs` file is the OLD system's SSA IR, and the `ir/mod.rs` is a thin wrapper. Neither is a proper MIR with basic blocks, places, rvalues, drops.
- **AST→MIR lowering:** ❌ Does not exist.

### COMP-08: Borrow Checker
- **Exists:** Yes. `semantic/borrow_check.rs` (60K, ~1700 LOC).
- **Algorithm:** AST-walk based ownership tracking. **Not Polonius**, **not NLL**, **not MIR-based**.
- **Use-after-move:** ⚠️ Checks exist in the `BorrowChecker::check_module()` method but operate as warnings, not errors.
- **Conflicting borrows:** ⚠️ Tracked via `BorrowState` enum (Owned/Moved/BorrowedShared/BorrowedMut).
- **Field projections:** ❌ Not implemented.
- **Generational references:** ❌ Not implemented.
- **Linear types:** ❌ Not implemented.

### COMP-09: Code Generation
- **Exists:** Yes, multiple backends.
- **OVM bytecode:** ✅ Works end-to-end. `codegen/ovm_direct.rs` generates OVM bytecode from typed AST.
- **Native (x86-64):** 🟡 Prototype. `native_codegen.rs` (85K) emits raw machine code. Quality unclear.
- **LLVM:** 🟡 Stub. `llvm_backend.rs` (31K) requires `inkwell` and LLVM 17 installed.
- **DWARF:** 🟡 Stub. `dwarf.rs` exists but `--debug-info` warns "not yet implemented".
- **Has a complete program been compiled and run?** ✅ Yes, via OVM target. `examples/hello.omni` → `hello.ovm` → OVM runner → "Hello, World!".

### COMP-10: Runtime / OVM
- **Exists:** Yes. `ovm/src/main.rs` (1464 LOC) is a working stack-based bytecode VM.
- **Features:** Arithmetic (i64/f64), comparisons, control flow (jump/branch/call/return), local/global variables, arrays, objects (hashmaps), string operations, builtin functions (println, len, sqrt, file I/O). Stack overflow protection (10k frame limit). Step limit (5M).
- **Async executor:** ❌ None.
- **Structured concurrency:** ❌ None.
- **Sandboxing:** ❌ None.
- **Modularity:** Monolithic single file.

### COMP-11: Standard Library
- **Status:** All 37 `.omni` files in `std/` are **specification-level stubs** — they define types, traits, and function signatures but contain no actual implementation bodies. They cannot be compiled by the current compiler.
- **`std::core`**: `core.omni` (16K) defines `Option`, `Result`, `Copy`, `Clone`, `Drop`, `Display`, `Debug`, `Iterator`, `From`, `Into`.
- **`std::tensor`**: `tensor.omni` (16K) defines `Tensor<T, Shape>`.
- **`std::crypto`**: `crypto.omni` (71K) defines SHA, AES, ChaCha, TLS.
- **Arena, Gen, SlotMap:** ❌ Not present.
- **All core traits:** Defined in `core.omni` but not implemented by the compiler.

### COMP-12: Package Manager
- **`opm`:** `tools/opm/src/main.rs` (~200 LOC) with basic CLI commands.
- **`omni.toml` parsing:** ✅ `compiler/src/manifest.rs` (170 LOC) parses basic manifest.
- **Lockfile:** ❌ Not implemented.
- **PubGrub resolver:** ❌ Not implemented.
- **Workspace support:** ❌ Not implemented.
- **Build scripts:** ❌ Not implemented.

### COMP-13: Tooling
- **`omni-lsp`:** Exists in `tools/omni-lsp/` with basic server structure. No semantic understanding.
- **`omni-fmt`:** Exists in `tools/omni-fmt/` with `formatter.rs`. Basic formatting.
- **`omni-dap`:** Exists in `tools/omni-dap/`. Stub.
- **VS Code extension:** `tools/vscode-omni/` with syntax highlighting and extension.ts.

### COMP-14: Diagnostics
- **Structured type:** ⚠️ `ParseError` has error codes (E001-E009). `SemanticError` has variant-based errors.
- **Stable error codes:** ⚠️ Parser has E001-E009. Semantic uses descriptive variant names, not stable codes.
- **Primary spans:** ❌ AST nodes lack spans. Only tokens have line/column.
- **Machine-applicable fixes:** ❌ Not implemented.
- **JSON output:** ❌ Not implemented.
- **"Did you mean?":** ✅ In parser for common typos.
- **Internationalization:** ❌ Not implemented.

### COMP-15: Security and Capability System
- **Capability tokens:** ❌ Not implemented.
- **Runtime enforcement:** ❌ Not implemented.
- **Sandboxed plugins:** ❌ Not implemented.
- **Fearless FFI:** ❌ Not implemented.
- **Package signing:** ❌ Not implemented.

---

## PHASE 5: CROSS-CUTTING CONCERNS

### CCO-01: Code Quality
- **`#![allow(dead_code)]`**: Present in `parser/ast.rs:15`, `semantic/mod.rs:15`, `lexer/mod.rs:15`, `stage1-compiler/src/main.rs:1`. Hides real problems.
- **`todo!()` macros**: Not found in hot paths. Some TODO comments exist.
- **`unwrap()`/`expect()` in library code**: Present in `main.rs:282` (`self.advance().unwrap()`).
- **Rust clippy**: Not determinable without running.

### CCO-02: Architectural Integrity
- **Parallel implementations:** ✅ **CONFIRMED**. Two complete, incompatible compiler front-ends coexist:
  - OLD: `lexer.rs` + `parser.rs` + `ast.rs` + `ir.rs` + `codegen.rs` + `semantics.rs`
  - NEW: `lexer/mod.rs` + `parser/mod.rs` + `parser/ast.rs` + `semantic/mod.rs` + `codegen/mod.rs`
  - `stage1-compiler` references the OLD files via `#[path = ...]`.
- **Module boundary violations:** `codegen.rs` and `ir.rs` import from `ast.rs` (OLD), while `main.rs` uses `parser::ast` (NEW). They define different types with the same names.
- **Heavy runtime dependencies in compiler crate:** `tokio`, `reqwest`, `ndarray`, `rand`, `chrono`, `uuid`, `sysinfo`, `libloading` are all in the compiler's `Cargo.toml` — these belong in a runtime crate, not the compiler.

### CCO-03: Self-Hosting Integrity
- **Is the Omni compiler in `omni/compiler/` real?** ⚠️ It's a collection of `.omni` source files that define a lexer, parser, AST, codegen, etc. in Omni syntax. However, none of these files can actually be compiled by either the OLD or NEW Rust compiler because they use syntax features the parsers don't fully support.
- **`omnc-stage1.ovm` / `omnc-stage2.ovm`**: Both are **empty files** (0 bytes). Self-hosting has not been achieved.
- **Bootstrap claim:** The `docs/IMPLEMENTATION_STATUS.md` claims "Phase 3: 75%" for self-hosting. This is misleading — the source exists but none of it compiles.

### CCO-04: HELIOS vs Omni Coupling
- **HELIOS code:** Not found in the Omni compiler codebase. The top-level `d:\Project\Helios` directory contains both `omni-lang/` and presumably other Helios projects, but no Helios code is mixed into `omni-lang/`.
- **Assessment:** Clean separation. ✅

### CCO-05: Documentation vs Reality Gap

> [!WARNING]
> **Major documentation-reality gaps:**

| Claim (README/docs) | Reality |
|---|---|
| "1019 tests passing" (badge) | ~20 actual tests across the codebase |
| "Phase 2: Core Functionality" | Phase 1 (basic end-to-end) is barely complete |
| "Self-hosted compiler (minimal)" ✅ | Self-hosted compiler source exists but cannot compile |
| "Type system with inference" ✅ | Basic inference works; advanced features missing |
| "Ownership and borrowing" ✅ | Warnings only; no enforcement; AST-walk only |
| `build-passing` badge | No CI exists |
| `omni-lsp` described as real tool | Stub with no semantic understanding |
| `opm` described with full commands | Most commands are stubs |

---

## PHASE 6: PHASE-BY-PHASE STATUS

### Phase 0: Project Foundation
**Status:** Started (60%)  
**What exists:** Repository structure, Cargo.toml, README, license, basic CI setup (no CI actually configured).  
**What's missing:** No CI, no contributing guide with enforced standards, no ADR folder, no issue templates.  
**Completion:** 60%

### Phase 1: Language Core Skeleton (Lexer + Parser + AST)
**Status:** Started (70%)  
**What exists:** Logos-based lexer with INDENT/DEDENT (✅), recursive-descent parser with error recovery (✅), comprehensive AST (✅).  
**What's missing:** No spans on AST nodes, no CST (lossless representation), no `--` comment syntax, no `f"..."` strings, no effect annotation syntax, no `inout`/`linear` syntax, zero parser tests.  
**Completion:** 70%

### Phase 2: Semantic Core and Type Checking
**Status:** Started (40%)  
**What exists:** Type inference engine, borrow checker (warnings only), trait definitions, constraint solver, builtin function registration.  
**What's missing:** No MIR, no proper dataflow analysis, no Polonius, no effect checking, no exhaustiveness checking for match (exists as error variant but not implemented).  
**Completion:** 40%

### Phase 3: Ownership, Borrowing, and Safety Core
**Status:** Started (20%)  
**What exists:** `BorrowChecker` struct, `BorrowState` tracking, basic move detection.  
**What's missing:** Not MIR-based. No field projections. No generational references. No linear types. No arena allocation. Borrow checker emits warnings, not errors.  
**Completion:** 20%

### Phase 4: Modules, Packages, and Build System
**Status:** Started (25%)  
**What exists:** Import resolution in `main.rs`, manifest parsing.  
**What's missing:** No package resolution, no lockfile, no workspace support, no build graph, no incremental compilation.  
**Completion:** 25%

### Phase 5: Standard Library Core
**Status:** Started (10%)  
**What exists:** 37 `.omni` stub files with type/trait/function signatures.  
**What's missing:** All implementation bodies. None of these files can compile.  
**Completion:** 10%

### Phase 6: Tooling and Developer Experience
**Status:** Started (15%)  
**What exists:** CLI flags, basic VSCode extension with syntax highlighting, stub LSP/formatter/DAP.  
**What's missing:** Working LSP, working formatter, working test runner, working package manager, documentation generator.  
**Completion:** 15%

### Phases 7-13: Advanced Features
**Status:** Not Started (0%)  
**Premature work present:**
- `codegen/gpu_dispatch.rs` (75K) — Phase 9/13 GPU work
- `codegen/mlir.rs` (28K) — Phase 13 MLIR work
- `codegen/jit.rs` (65K) — Phase 11 JIT work
- `codegen/python_interop.rs` — Phase 11 interop
- `semantic/autograd.rs` — Not in any phase (HELIOS-level feature)
- `semantic/effects.rs` — Phase 8 effect system

> [!CAUTION]
> **~430K of code in `codegen/` belongs to Phases 7-13** while Phase 1-3 foundations are incomplete. This represents significant misdirected engineering investment.

---

## PHASE 8: CRITICAL PATH ANALYSIS

### Milestone 1: "Hello, World!" compiles and runs

**Status: ✅ ACHIEVED** (via OVM bytecode target)

```
hello.omni → lexer → parser → semantic analyzer → ovm_direct codegen → hello.ovm → OVM runner → stdout
```

### Milestone 2: Variables, arithmetic, control flow

**Status: ✅ ACHIEVED** (basic level via OVM)

### Milestone 3: Functions, structs, pattern matching

**Status: ⚠️ PARTIALLY** — Functions compile. Structs parse but codegen support is limited. Pattern matching parses but OVM codegen support is partial.

### Milestone 4: Borrow checker rejects use-after-move

**Status: ❌ NOT ACHIEVED** — Borrow checker emits warnings, not errors. Code compiles even with ownership violations.

### Milestone 5: Type checker rejects type mismatch

**Status: ⚠️ PARTIALLY** — Hard type errors abort compilation. But `Type::Any` on builtins means most function calls pass type checking regardless.

### Milestone 6: `omni test` runs a `@test`

**Status: ❌ NOT ACHIEVED** — No `omni test` command exists. `@test` attribute parsing exists but no test runner.

### Milestone 7: Two-file cross-import compiles

**Status: ✅ ACHIEVED** — Import resolution in `main.rs:621-892` resolves and inlines imported modules.

### Milestone 8: LSP go-to-definition

**Status: ❌ NOT ACHIEVED** — LSP exists but has no semantic understanding.

---

## PHASE 9: RISK ASSESSMENT

### RISK-1: Architectural Fragmentation
**Description:** Two parallel compiler front-ends (OLD/NEW) create confusion, duplicate maintenance, and impossible module boundaries.  
**Likelihood:** High (already occurring)  
**Impact:** Critical  
**Mitigation:** Delete the OLD files (`lexer.rs`, `parser.rs`, `ast.rs`, `ir.rs`, `codegen.rs`, `semantics.rs`). Rewrite `stage1-compiler` to use the NEW modules.

### RISK-2: Premature Phase 7-13 Investment
**Description:** ~430K of code in codegen/ (GPU, JIT, MLIR, Python, native linker) was written before Phase 1-3 foundations are stable.  
**Likelihood:** High (already occurred)  
**Impact:** High — this code will need to be rewritten when the IR changes.  
**Mitigation:** Freeze Phase 7-13 work. Focus exclusively on Phases 1-6.

### RISK-3: No AST Spans → Unusable Diagnostics
**Description:** AST nodes have no source spans. Diagnostic messages cannot point to specific source locations in expressions/statements.  
**Likelihood:** High  
**Impact:** High — blocks useful error messages, LSP, and IDE integration.  
**Mitigation:** Add `Span` field to every AST node and `TypedExpr`.

### RISK-4: Borrow Checker Not MIR-Based
**Description:** The borrow checker operates on the AST, not a control-flow graph. This makes it impossible to correctly handle conditional moves, loops, or non-lexical lifetimes.  
**Likelihood:** High  
**Impact:** Critical — the borrow checker will need to be completely rewritten.  
**Mitigation:** Design and implement a MIR before investing further in the borrow checker.

### RISK-5: Self-Hosting Claims Are Misleading
**Description:** Documentation claims 75% progress on self-hosting, but stage1.ovm and stage2.ovm are empty files.  
**Likelihood:** High  
**Impact:** Medium — erodes trust in project status assessments.  
**Mitigation:** Update documentation to accurately reflect status: "Self-hosted source exists but does not compile."

### RISK-6: Monolithic Compiler Crate
**Description:** The compiler is a single Cargo crate with 25+ dependencies including `tokio`, `reqwest`, `ndarray`, `sysinfo`. Compile times are enormous and runtime deps are mixed with compiler deps.  
**Likelihood:** High  
**Impact:** Medium  
**Mitigation:** Split into workspace crates: `omni-lexer`, `omni-parser`, `omni-semantic`, `omni-codegen`, `omni-cli`.

### RISK-7: No CI/CD
**Description:** No continuous integration. No automated testing. No build verification.  
**Likelihood:** High  
**Impact:** High — regressions go undetected.  
**Mitigation:** Add GitHub Actions CI with `cargo test`, `cargo clippy`, `cargo fmt --check`.

### RISK-8: Zero Parser Tests
**Description:** The parser (3691 LOC, the largest and most critical component) has zero tests.  
**Likelihood:** High  
**Impact:** Critical — any parser change can silently break the language.  
**Mitigation:** Write snapshot tests for every syntactic form.

### RISK-9: `Type::Any` Undermines Type Safety
**Description:** All builtin functions use `Type::Any` parameters, which match any type. This means `println(42)`, `println("hello")`, and `println(some_struct)` all pass type checking without any actual type resolution.  
**Likelihood:** High  
**Impact:** Medium — false sense of type safety.  
**Mitigation:** Implement proper overloading or trait-based built-in dispatch.

### RISK-10: 624 Stall Logs Indicate Systematic Issue
**Description:** The `diagnostics/` directory contains 624 monitor stall logs from a single compilation session. This indicates the compiler has infinite-loop or performance issues that required the stall detector.  
**Likelihood:** High (evidence in repo)  
**Impact:** Medium — indicates parser performance issues with large files.  
**Mitigation:** Investigate and fix the parser loops. Delete stale diagnostics.

---

## PHASE 10: ACTIONABLE REMEDIATION PLAN

### Immediate (Days 1-30): Foundation Repair

- [ ] **TASK: Delete OLD compiler files**
  - Files: `compiler/src/lexer.rs`, `parser.rs`, `ast.rs`, `ir.rs`, `codegen.rs`, `semantics.rs`
  - Depends on: Nothing
  - Hours: 2
  - Check: `stage1-compiler` either removed or rewritten to use new modules

- [ ] **TASK: Add `Span` to all AST nodes**
  - File: `compiler/src/parser/ast.rs`
  - Depends on: Nothing
  - Hours: 8
  - Check: Every `Expression`, `Statement`, `Item`, `Type` has a `span: Span` field

- [ ] **TASK: Write parser snapshot tests (50+ tests)**
  - File: `compiler/src/parser/tests.rs` (new)
  - Depends on: Nothing
  - Hours: 16
  - Check: Every syntactic form (fn, struct, enum, trait, impl, match, if, for, while, import, etc.) has at least one test

- [ ] **TASK: Add GitHub Actions CI**
  - File: `.github/workflows/ci.yml`
  - Depends on: Nothing
  - Hours: 2
  - Check: CI runs `cargo test`, `cargo clippy`, `cargo fmt --check` on every push

- [ ] **TASK: Make borrow checker errors fatal**
  - File: `compiler/src/main.rs:532-537`
  - Depends on: Nothing
  - Hours: 1
  - Check: Borrow violations abort compilation

- [ ] **TASK: Delete 624 stall logs**
  - Dir: `diagnostics/`
  - Hours: 0.5
  - Check: Directory is clean

- [ ] **TASK: Remove runtime deps from compiler Cargo.toml**
  - File: `compiler/Cargo.toml` — remove `tokio`, `reqwest`, `ndarray`, `rand`, `chrono`, `uuid`, `sysinfo`, `libloading`
  - Depends on: Audit which codegen modules use these
  - Hours: 4
  - Check: `cargo build` succeeds with minimal deps

### Short-term (Days 31-60): Type System and Safety

- [ ] **TASK: Implement proper MIR**
  - New crate: `compiler/src/mir/mod.rs`
  - Depends on: AST spans
  - Hours: 40
  - Check: MIR has basic blocks, places, rvalues, terminators, drop flags

- [ ] **TASK: Move borrow checker to MIR**
  - File: `compiler/src/semantic/borrow_check.rs` → rewrite
  - Depends on: MIR
  - Hours: 40
  - Check: Borrow checker catches use-after-move, conflicting borrows via dataflow

- [ ] **TASK: Remove `Type::Any` from builtins**
  - File: `compiler/src/semantic/mod.rs:597-715`
  - Depends on: Proper overloading or Display trait
  - Hours: 8
  - Check: `println` requires a type that implements Display

- [ ] **TASK: Implement `f"..."` string interpolation**
  - File: `compiler/src/lexer/mod.rs`
  - Depends on: Nothing
  - Hours: 8
  - Check: `f"Hello {name}!"` lexes and parses correctly

### Medium-term (Days 61-90): Modules, Stdlib, Tooling

- [ ] **TASK: Split compiler into workspace crates**
  - Files: New `Cargo.toml` at root, `crates/omni-lexer/`, `crates/omni-parser/`, etc.
  - Depends on: Nothing (can be done incrementally)
  - Hours: 16
  - Check: `cargo build --workspace` succeeds

- [ ] **TASK: Implement basic `omni test` runner**
  - File: `compiler/src/main.rs` (add `test` subcommand)
  - Depends on: AST attribute parsing
  - Hours: 16
  - Check: `omnc --test file.omni` finds and runs `@test` functions

- [ ] **TASK: Implement core stdlib (`Option`, `Result`, `Iterator`)**
  - Files: `std/core.omni`, with actual implementations
  - Depends on: Compiler supports trait methods
  - Hours: 24
  - Check: `Option::map()`, `Result::unwrap()`, `Iterator::next()` compile and run

---

## PHASE 11: SUMMARY SCORECARD

### OVERALL PROJECT HEALTH SCORECARD

| Dimension | Score | Justification |
|-----------|-------|---------------|
| Architecture Coherence | 3/10 | Two parallel front-ends, monolithic crate, Phase 7-13 code before Phase 1-3 done |
| Specification Compliance | 2/10 | Most spec requirements (effect system, capabilities, MIR, Polonius, structured concurrency) are not implemented |
| Phase 0 Completion | 60% | |
| Phase 1 Completion | 70% | Lexer and parser exist, lacking spans, tests, some syntax |
| Phase 2 Completion | 40% | Type inference works basically, no MIR |
| Phase 3 Completion | 20% | Borrow checker is AST-walk, warnings only |
| Phase 4 Completion | 25% | Basic import resolution only |
| Phase 5 Completion | 10% | All stubs |
| Phase 6 Completion | 15% | CLI flags only, tools are stubs |
| Phases 7-13 Completion (premature) | 5% | ~430K of premature code exists |
| Code Quality | 4/10 | `#![allow(dead_code)]`, no CI, parallel implementations, runtime deps in compiler |
| Test Coverage | 2/10 | ~20 tests for ~15000 LOC of active compiler code |
| Documentation Quality | 3/10 | README is misleading vs reality |
| Diagnostic Quality | 3/10 | Parser has error codes; no spans on AST; no machine-applicable fixes |
| Self-Hosting Legitimacy | 1/10 | Stage1/Stage2 OVM files are 0 bytes |
| Critical Path Clarity | 5/10 | Pipeline works for basic OVM target but foundation for spec compliance is absent |

### Top 3 Strengths
1. **Working end-to-end pipeline:** Source → Tokens → AST → TypedAST → OVM bytecode → VM execution works for basic programs.
2. **Solid lexer with INDENT/DEDENT:** The Logos-based lexer with indentation tracking is well-implemented and tested.
3. **Parser error recovery:** The parser's panic-mode recovery, "did you mean?" suggestions, and error limit are production-quality patterns.

### Top 3 Critical Problems
1. **Two parallel compiler implementations** coexist with incompatible types. Delete the OLD files immediately.
2. **~430K of premature Phase 7-13 code** (GPU, JIT, MLIR, native linker) was written without MIR or proper IR infrastructure. This will all need rewriting.
3. **No AST spans + no MIR = no path to correct borrow checking, proper diagnostics, or LSP** — the two most foundational architectural decisions (spans and MIR) are both missing.

### Time Estimates
| Milestone | Estimate |
|-----------|----------|
| "Hello World" running end-to-end | ✅ Already achieved |
| Borrow checker correctly rejects use-after-move | 8-10 weeks (requires MIR first) |
| Phase 6 complete (usable language) | 6-8 months focused |
| Phase 12 (self-hosting) | 3-4 years realistic |
