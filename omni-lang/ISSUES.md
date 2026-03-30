# Omni Programming Language - Comprehensive Issue Tracker

> **Project Scope:** This document tracks all issues for the Omni programming language compiler, runtime, standard library, tooling, and self-hosting pipeline in `omni-lang/`.
>
> **Audit Date:** 2026-03-30 — Full byte-by-byte code audit performed on all 170+ source files (~85,000+ lines)
>
> **Last Updated:** 2026-03-30

---

## Executive Summary

### Overall Specification Compliance

The Omni language specification describes a phased, multi-paradigm, self-hosting programming language. This audit compares every aspect of the implementation against the specification.

| Specification Area | Compliance | Details |
|---|---|---|
| **Phase 1: Structural Foundation** | ✅ **100%** | Core syntax, parsing, basic IR — all fully implemented |
| **Phase 2: Core Functionality** | ✅ **95%** | Type system, memory management, stdlib, pipeline — all working. Minor gaps in closure inference and module resolution. |
| **Phase 3: Enrichment** | 🔄 **65%** | Advanced type features (partial), concurrency models (declared, not runtime-verified), compile-time computation (comptime parsed, limited execution), tooling (complete) |
| **Phase 4: Optimization** | 🔄 **50%** | High-performance execution (12 optimizer passes, JIT framework), cross-platform (x86-64/ARM64/RISC-V/WASM emitters exist), native binary emission (not end-to-end working) |
| **Phase 5: Self-Hosting** | 🔴 **30%** | Self-hosted compiler source exists (~23,000 lines), Stage 0 works, Stages 1-2 not implemented |

### Code Quality Summary

| Metric | Value |
|---|---|
| **Total source files** | 170+ |
| **Total lines of code** | ~85,000+ |
| **Rust compiler (compiler/)** | ~35,000 lines across 93 files |
| **Self-hosted compiler (omni/)** | ~23,000 lines across 34 files |
| **Standard library (std/)** | ~20,770 lines across 37 files |
| **Core library (core/)** | ~3,254 lines across 12 files |
| **Tools (tools/)** | ~8,294 lines across 20 files |
| **OVM runtime (ovm/)** | ~2,326 lines across 4 files |
| **Cargo tests passing** | 1,019 (547 lib + 472 integration) |
| **Example programs working** | 15/15 |
| **Files with real implementation** | 165+ (only ~5 are stubs/specs) |
| **Files that are pure stubs** | 5 (iter.omni, entry_point.omni, ownership_keywords.omni, logging.omni, ffi.omni) |
| **TODO/FIXME comments in Rust compiler** | 6 total |
| **TODO/FIXME comments in self-hosted compiler** | 0 |

---

## ✅ Working Components (Phase 1-2 Complete)

| Component | Status | Lines | Notes |
|---|---|---|---|
| Lexer (Rust) | ✅ Working | 836 | Logos-based, 70+ token types, indentation-sensitive, 18 tests |
| Parser (Rust) | ✅ Working | 2,851 | Recursive descent, error recovery, all constructs, panic-mode sync |
| AST | ✅ Working | 495 | 11 item types, 25+ expression types, 15 statement types, 25+ type variants |
| Semantic Analysis | ✅ Working | 2,981 | Scope management, symbol table, type inference, borrow checking, trait bounds |
| Type Inference (HM) | ✅ Working | 2,299 | Constraint-based Hindley-Milner with full unification |
| Borrow Checker | ✅ Working | 1,455 | Use-after-move, double-mut, dangling refs, 11 error types, 11 tests |
| IR Generation | ✅ Working | 1,714 | SSA-based, 25+ instruction types, full AST lowering |
| OVM Bytecode Codegen | ✅ Working | 2,020 | 144-opcode ISA, binary serialization, two backends (IR-based + direct AST) |
| OVM Runtime (Rust) | ✅ Working | 1,415 | Stack-based VM, 7 value types, GC, native functions |
| OVM Runtime (C) | ✅ Working | 873 | Independent C implementation, no Rust dependency |
| AST Interpreter | ✅ Working | 3,817 | Tree-walking + bytecode, async runtime, incremental GC |
| High-Level VM | ✅ Working | 1,792 | Second VM with tri-color GC, 29 tests |
| Bytecode Compiler | ✅ Working | 1,278 | AST→OVM bytecode, 17 tests |
| Bytecode Format | ✅ Working | 1,059 | Serialize/deserialize, 12 tests |
| Optimizer | ✅ Working | 2,478 | 12+ passes (const fold, DCE, inlining, CSE, LICM, strength reduction, etc.) |
| Standard Library | ✅ Working | 20,770 | 37 modules covering all major domains |
| Tooling (LSP) | ✅ Working | 2,561 | Diagnostics, completions, hover, go-to-definition |
| Tooling (Formatter) | ✅ Working | 564 | Import sorting, code normalization |
| Tooling (DAP) | ✅ Working | 1,735 | Breakpoints, stepping, variable inspection |
| Tooling (OPM) | ✅ Working | 2,785 | init/add/remove/build/run, semver resolution |
| Tooling (VS Code) | ✅ Working | 641 | Syntax highlighting, LSP integration, formatting |
| Self-Hosted Source | ✅ Exists | 23,000+ | Full compiler pipeline in Omni (34 files) |
| Bootstrap Stage 0 | ✅ Working | — | Rust omnc → .ovm bytecode → OVM execution |
| Examples | ✅ Working | 132 | 15 programs, 5 tutorials |
| Cargo Tests | ✅ Passing | — | 1,019 tests (0 failures) |

---

## Specification Compliance — Detailed Analysis

### 1. Vision and Goals

| Goal | Status | Evidence |
|---|---|---|
| **Self-hosting** | 🔴 Not achieved | Self-hosted source exists but cannot compile itself yet (Stages 1-2 missing) |
| **Standalone** | 🔴 Not achieved | Still requires Rust/Cargo for compilation. C OVM exists as standalone runtime path |
| **Universal** | 🔄 Partial | Systems programming (ownership, unsafe), web (HTTP/WebSocket), AI (tensor ops, autograd), distributed (NCCL/MPI/Raft), embedded (bare-metal mode exists) |
| **Layered** | ✅ Achieved | Three module modes (Script/Hosted/BareMetal), memory strategy resolver, execution strategy resolver |

### 2. Core Philosophy

| Principle | Status | Evidence |
|---|---|---|
| **Progressive complexity** | ✅ Achieved | Simple hello world to ownership to async to GPU. Tutorials 01-05 demonstrate this. |
| **Extreme extensibility** | 🔄 Partial | Module modes configurable, resolver strategies selectable, but syntax not yet user-extensible (no user-defined syntax macros) |
| **Multi-paradigm** | ✅ Achieved | Procedural (functions), OOP (structs+traits+impl), functional (lambdas, map/filter/reduce, iterators), data-oriented (tensors) |

### 3. Design Approach (Phased Development)

| Phase | Spec Status | Implementation Status | Gap Analysis |
|---|---|---|---|
| **Phase 1: Structural Foundation** | Core syntax, parsing, basic IR | ✅ **COMPLETE** — Lexer (836L), Parser (2,851L), IR (1,714L), all with tests | No gaps |
| **Phase 2: Core Functionality** | Type system, memory management, stdlib, pipeline | ✅ **COMPLETE** — Type inference (2,299L), borrow checker (1,455L), 37 stdlib modules (20,770L), full pipeline | Minor gap: closure type inference, module resolution |
| **Phase 3: Enrichment** | Advanced types, concurrency, comptime, tooling | 🔄 **65%** — Const generics (131L), async runtime declared, comptime keyword parsed, full tooling suite (LSP/DAP/fmt/opm/VSCode) | Gaps: const evaluation limited, concurrency untested at runtime, advanced generics partial |
| **Phase 4: Optimization** | High-perf, cross-platform, ecosystem | 🔄 **50%** — 12 optimizer passes, JIT framework, x86-64/ARM64/RISC-V/WASM emitters, native linker | Gap: native binary emission not end-to-end working, JIT not connected to main pipeline |
| **Phase 5: Self-Hosting** | Compiler in Omni, bootstrap | 🔴 **30%** — 23,000 lines of self-hosted source exists, Stage 0 works | Gaps: Stages 1-2 not implemented, self-hosted compiler has syntax compatibility issues |

### 4. Language Characteristics

#### 4a. Multi-Paradigm Support

| Paradigm | Status | Evidence |
|---|---|---|
| Procedural | ✅ Full | Functions, variables, loops, conditionals |
| Object-Oriented | ✅ Full | Structs with methods, traits with impl blocks, trait objects |
| Functional | ✅ Partial | Lambdas, closures (partial type inference), map/filter/reduce in stdlib |
| Data-Oriented | ✅ Full | Tensor<T> (494L), N-dimensional arrays, BLAS operations |

#### 4b. Execution Models

| Model | Status | Evidence |
|---|---|---|
| Interpreted | ✅ Working | AST tree-walking interpreter (3,817L) |
| Compiled (bytecode) | ✅ Working | OVM bytecode compilation + execution |
| Compiled (native) | 🔴 Not working | Native codegen exists (2,459L) but end-to-end emission fails |
| JIT | 🔄 Exists, not integrated | JIT framework (1,749L) with real x86-64 machine code emission, but not wired to main pipeline |
| Hybrid | 🔄 Designed | Resolver system selects AOT/JIT/Interpreter, but JIT path not complete |

#### 4c. Memory Management

| Model | Status | Evidence |
|---|---|---|
| Manual | ✅ Declared | `unsafe *` pointers in grammar, raw pointer ops in std/mem.omni |
| Ownership-based | ✅ Working | Borrow checker (1,455L), move semantics, `own`/`shared`/`&`/`&mut` |
| Garbage Collection | ✅ Working | Incremental tri-color mark-and-sweep GC in OVM interpreter and vm.rs |
| Coexistence | 🔄 Partial | Memory Strategy Resolver (MSR) can select GC/Ownership/Manual/Region/RefCounted per module, but runtime switching untested |

#### 4d. Concurrency

| Feature | Status | Evidence |
|---|---|---|
| Async/await | ✅ Parsed + declared | Parser handles async/await, std/async.omni (544L) declares full runtime |
| Parallel execution | ✅ Declared | std/thread.omni (660L), ThreadPool in std/async.omni |
| Distributed systems | ✅ Declared | std/dist.omni (958L), NCCL/MPI FFI, Raft consensus, 2PC |
| Runtime verification | 🔴 Not verified | Thread intrinsics declared but NOT implemented in OVM (O-082) |

### 5. Syntax and Structure

| Feature | Status | Evidence |
|---|---|---|
| Indentation-based | ✅ Working | Lexer tracks indent/dedent, Python-style blocks |
| Brace-based | 🔴 **Not supported** by parser | Grammar spec supports both, but parser only handles colon+indent. 7 stdlib files have WARNING headers about unparseable brace-delimited extern blocks (O-078) |
| Expression-oriented | ✅ Working | If-expressions, match-expressions, block expressions in AST |
| Statement-based | ✅ Working | Full statement grammar (15 variants) |

**Gap:** The specification says Omni supports "both indentation-based and brace-based styles." The parser only supports indentation-based for function/struct/control-flow bodies. Extern blocks in the stdlib use brace syntax that the current parser cannot parse.

### 6. Type System

| Feature | Status | Evidence |
|---|---|---|
| Static typing (default) | ✅ Working | Type annotations, HM type inference |
| Optional dynamic typing | 🔴 Not implemented | No `dyn` or `Any` at language level (reflect.omni has runtime `Any` trait but it's a library concept, not language-level) |
| Advanced type inference | ✅ Working | HM constraint solver (2,299L), untyped params, return inference |
| Generics | ✅ Partial | Basic generics work, const generics (131L), but GATs and complex where clauses not fully working |
| Metaprogramming | 🔄 Partial | `comptime` keyword parsed, derive macros in self-hosted compiler, but compile-time execution limited |

### 7. Compilation and Execution

| Feature | Status | Evidence |
|---|---|---|
| AOT (bytecode) | ✅ Working | omnc → .ovm files |
| AOT (native) | 🔴 Not working | Codegen infrastructure exists but no working native binaries |
| JIT | 🔄 Exists | 1,749L JIT with real x86-64 emission, not integrated |
| Modular pipeline | ✅ Working | Resolver-based strategy selection, configurable optimization levels (O0-O3) |

### 8. Tooling and Ecosystem

| Tool | Status | Lines | Evidence |
|---|---|---|---|
| Package manager (opm) | ✅ Working | 2,785 | init, add, remove, build, run, semver resolution |
| Build system | ✅ Working | 929 (self-hosted) | Build tool in omni/tools/build.omni |
| Debugging (DAP) | ✅ Working | 1,735 | Breakpoints, stepping, variable inspection, tensor visualization |
| Profiling | ✅ Working | 550 | Runtime profiler with PGO, CPU feature detection |
| IDE support (VS Code) | ✅ Working | 641 | Syntax highlighting, LSP, formatting, debug |
| Formatter | ✅ Working | 564 | Import sorting, code normalization, CI check mode |
| Language Server | ✅ Working | 2,561 | Diagnostics, completions, hover, go-to-definition, rename |

### 9. Self-Hosting Strategy

| Milestone | Status | Evidence |
|---|---|---|
| Initial Rust implementation | ✅ Complete | 93 Rust source files, ~35,000 lines, 1,019 tests |
| Self-hosted source exists | ✅ Complete | 34 Omni files, ~23,000 lines, mirrors Rust compiler structure |
| Stage 0 (Rust compiles Omni) | ✅ Working | compiler_minimal.omni compiles and runs |
| Stage 1 (Self-hosted compiles itself) | 🔴 Not implemented | Self-hosted main.omni has syntax compatibility issues |
| Stage 2 (Verify bit-identical) | 🔴 Not implemented | bootstrap.omni has SHA-256 verification code ready |
| Rust dependency removed | 🔴 Not achieved | C OVM exists as alternative runtime, but compiler still requires Rust |

### 10. Contribution Guidelines

| Requirement | Status | Evidence |
|---|---|---|
| Apache 2.0 license | ✅ Present | LICENSE file in root |
| Contribution guidelines | ✅ Present | CONTRIBUTING.md in root |
| Issue tagging | ✅ Present | SH-*/HP-*/MP-*/LP-*/GFI-*/HW-* labels defined |
| Clear areas for contribution | ✅ Present | Compiler, design, docs, testing sections identified |

---

## 🔴 CRITICAL: Self-Hosting Blockers

### SH-001: Bootstrap Stages 1-2 Not Implemented

**Status:** 🔴 OPEN
**Priority:** CRITICAL
**Component:** bootstrap
**Estimated Effort:** Hard (weeks)

**Description:**
The bootstrap pipeline only has Stage 0 working. True self-hosting requires:
- Stage 0: Rust omnc compiles self-hosted source → bytecode ✅
- Stage 1: Self-hosted compiler compiles itself → new bytecode ❌
- Stage 2: Stage 1 output compiles itself → verify bit-identical ❌

**Current Pipeline:**
```
Stage 0: ✅ Rust omnc compiles compiler_minimal.omni → bytecode
Stage 1: ❌ Not implemented
Stage 2: ❌ Not implemented
```

**What Needs to Happen:**
1. Enhance `omni/compiler_minimal.omni` (currently 15-line stub) to be a real compiler
2. Fix syntax compatibility between self-hosted source and Rust parser
3. Implement Stage 1: Self-hosted compiles itself
4. Implement Stage 2: Stage 1 compiles Stage 1
5. Verify bit-identical output using SHA-256 (code already exists in bootstrap.omni)

**Blocking Factors:**
- Self-hosted main.omni uses syntax constructs (nested generics, complex closures) that Rust parser may not handle
- Monomorphization must specialize generic functions end-to-end
- IR must preserve actual types (not hardcode I64)
- Codegen must emit proper bytecode files that the OVM can load

**Related Files:**
- `omni/bootstrap.omni` (684L) — 3-stage bootstrap logic ready
- `omni/compiler_minimal.omni` (15L) — needs to become real compiler
- `omni/compiler/` (34 files, 23,000L) — full self-hosted compiler source

---

### SH-002: Native Binary Emission Not Working

**Status:** 🔴 OPEN
**Priority:** CRITICAL
**Component:** codegen
**Estimated Effort:** Hard (weeks)

**Description:**
`omnc --emit native` doesn't produce working native executables. The code generation infrastructure exists but is not connected end-to-end.

**What Exists:**
- Native codegen (`native_codegen.rs`, 2,459L) — x86-64/ARM64/WASM/RISC-V emitters with correct instruction encoding
- Native linker (`native_linker.rs`, 1,291L) — ELF/PE/Mach-O parsing and linking
- Full linker (`linker.rs`, 1,600+L) — ELF64/PE/Mach-O executable emission
- PE builder, Mach-O builder (`native_extended.rs`, 1,051L)
- DWARF debug info emitter (`dwarf.rs`, 424L)

**What's Missing:**
1. End-to-end pipeline from IR → native codegen → linking → executable
2. Integration testing on any platform
3. Runtime startup code (crt0 equivalent)
4. Standard library linking

---

### SH-003: Standalone Runtime Not Achieved

**Status:** 🔴 OPEN
**Priority:** CRITICAL
**Component:** runtime
**Estimated Effort:** Medium (days-weeks)

**Description:**
The specification requires Omni to be "standalone: not dependent on external runtimes." Currently:
- The compiler requires Rust/Cargo to build
- The OVM runtime exists in both Rust (1,415L) and C (873L)
- The C OVM (`ovm/ovm.c`) is the path to standalone, but needs to be the primary runtime

**What Needs to Happen:**
1. Make C OVM the primary standalone runtime
2. Ensure all OVM opcodes work identically in Rust and C implementations
3. Remove Rust runtime dependency for program execution
4. Test cross-platform (Windows, Linux, macOS)

---

## 🔴 HIGH PRIORITY: Type System & Language

### HP-001: Type Inference for Complex Expressions

**Status:** ✅ WORKING
**Priority:** HIGH

The type inference system now handles untyped function parameters, arithmetic expressions in returns, and polymorphic builtins. Verified with 20+ tests in `semantic/type_inference.rs`.

---

### HP-002: Borrow Checker

**Status:** ✅ WORKING
**Priority:** HIGH

Full borrow checker (1,455L) with 11 error types: UseAfterMove, DoubleMutBorrow, MutAndSharedBorrow, MovedWhileBorrowed, DanglingReference, MutationOfImmutable, ReturnLocalReference, MoveInLoop, BorrowOfMoved, InvalidDeref, Other. 11 unit tests passing.

---

### HP-003: Variable Scope in Loops

**Status:** ✅ WORKING
**Priority:** HIGH

Loop variables work correctly with conservative borrow checking.

---

### HP-004: Brace-Delimited Syntax Not Supported by Parser

**Status:** 🔴 OPEN
**Priority:** HIGH
**Component:** parser
**Estimated Effort:** Medium

**Description:**
The specification says Omni supports "both indentation-based and brace-based styles." The parser only handles indentation-based (colon + indent/dedent) for all constructs. Brace-delimited blocks (`{ }`) are not parsed.

**Impact:** 7 stdlib files have WARNING headers noting they use brace-delimited `extern "C" { }` blocks that the current parser cannot handle:
- `std/compress.omni` (O-078)
- `std/env.omni` (O-078)
- `std/mem.omni` (O-078)
- `std/rand.omni` (O-078)
- `std/sys.omni` (O-078, O-081)
- `std/time.omni` (O-078)
- `std/python.omni` (implicit)

**What Needs to Happen:**
1. Add brace-delimited block parsing to the parser
2. Make `extern` blocks parseable with `{ }` syntax
3. Ensure both styles coexist cleanly
4. Verify all stdlib files parse without warnings

---

### HP-005: Conditional Compilation (`#[cfg]`) Not Implemented

**Status:** 🔴 OPEN
**Priority:** HIGH
**Component:** parser, semantic
**Estimated Effort:** Medium
**Reference:** O-081

**Description:**
Many stdlib files use `#[cfg(unix)]` / `#[cfg(windows)]` attributes for platform-specific code, but the compiler does not process these. One file (`std/time.omni`) uses non-standard `@cfg(unix)` syntax.

**Impact:** Cross-platform stdlib code cannot be conditionally compiled. All platform-specific code paths are included regardless of target.

**Affected Files:** std/crypto.omni, std/env.omni, std/fs.omni, std/io.omni, std/mem.omni, std/net.omni, std/sys.omni, std/thread.omni, std/time.omni

---

### HP-006: Thread Intrinsics Not Implemented in OVM

**Status:** 🔴 OPEN
**Priority:** HIGH
**Component:** runtime
**Estimated Effort:** Medium
**Reference:** O-082

**Description:**
`std/thread.omni` declares 27 thread intrinsics (`extern "intrinsic"`) that are NOT implemented in the OVM runtime. This means threading, mutexes, atomics, channels, and all concurrency primitives are declared but non-functional.

**Impact:** Concurrency is a specification requirement ("first-class concern"). Without working thread intrinsics, the async runtime, thread pool, and all synchronization primitives are non-functional.

**What Needs to Happen:**
1. Implement thread intrinsics in the Rust OVM (`ovm/src/main.rs`)
2. Implement thread intrinsics in the C OVM (`ovm/ovm.c`)
3. Add integration tests for threading

---

## ⚠️ MEDIUM PRIORITY: Parser & Language Features

### MP-001: Closures Partially Supported

**Status:** ⚠️ PARTIAL
**Priority:** MEDIUM
**Component:** parser, type inference

**Working:** `let add = |a: int, b: int| a + b` (explicit types)
**Not Working:** `let add = |a, b| a + b` (type inference for closure parameters)

---

### MP-002: Complex Pattern Matching

**Status:** ⚠️ PARTIAL
**Priority:** MEDIUM
**Component:** parser

**Working:** Literal patterns, binding patterns, constructor patterns (Some/None/Ok/Err), or-patterns, wildcard
**Not Working:** Nested patterns, guard clauses (`if` in match arms), complex enum matching with field destructuring

---

### MP-003: Advanced Generics

**Status:** ⚠️ PARTIAL
**Priority:** MEDIUM
**Component:** semantic, parser

**Working:** Basic generics (`Vec<T>`), simple trait bounds, const generics (131L)
**Not Working:** Generic trait bounds in where clauses, generic associated types (GATs), higher-kinded types

---

### MP-004: Module Resolution

**Status:** ⚠️ PARTIAL
**Priority:** MEDIUM
**Component:** resolver

**Issue:** Imports don't resolve correctly for stdlib modules. The import resolver (`resolve_imports()` in main.rs) does recursive file-based resolution but stdlib path mapping is incomplete.

---

### MP-005: Struct Field Inference

**Status:** ✅ WORKING
**Priority:** MEDIUM

Struct creation and field access work correctly.

---

### MP-006: Compile-Time Computation (comptime)

**Status:** ⚠️ PARTIAL
**Priority:** MEDIUM
**Component:** parser, semantic

**Working:** `comptime` keyword is parsed as an item type
**Not Working:** Actual compile-time expression evaluation is limited to simple constant folding. No compile-time function execution.

**Spec Requirement:** "Compile-time computation" is a Phase 3 feature.

---

### MP-007: Syntax Inconsistency Between `#[cfg]` and `@cfg`

**Status:** ⚠️ INCONSISTENCY
**Priority:** MEDIUM
**Component:** parser

**Description:** Most stdlib files use `#[cfg(unix)]` / `#[cfg(windows)]` syntax, but `std/time.omni` uses `@cfg(unix)` / `@cfg(windows)`. Neither is actually processed by the compiler.

---

### MP-008: Duplicate Type Inference Systems

**Status:** ⚠️ TECHNICAL DEBT
**Priority:** MEDIUM
**Component:** semantic

**Description:** Two parallel type inference systems exist:
1. `semantic/inference.rs` (328L) — simpler HM solver
2. `semantic/type_inference.rs` (2,299L) — full constraint-based engine

Both are compiled. The full engine is the primary one used from `main.rs`. The simpler one should either be consolidated or removed.

**Also:** Two `SemanticError` types exist: one in `semantic/mod.rs` and one in `semantic/error_recovery.rs`.

---

### MP-009: Two VM Implementations in Rust

**Status:** ⚠️ TECHNICAL DEBT
**Priority:** MEDIUM
**Component:** runtime

**Description:** Three separate VM/interpreter implementations exist in the Rust compiler:
1. `runtime/interpreter.rs` (3,817L) — OVM bytecode interpreter + tree-walking AST interpreter
2. `runtime/vm.rs` (1,792L) — separate OpCode-based VM with its own GC
3. `ovm/src/main.rs` (1,415L) — standalone OVM runner

These should be consolidated. The `vm.rs` has its own `Call(n_args)` opcode that returns an error ("not yet supported; use CallNamed").

---

## 📋 LOW PRIORITY: Missing Features & Gaps

### LP-001: Standard Library Completeness

**Status:** ⚠️ PARTIAL
**Priority:** LOW

**Complete Modules (30):** algorithm, async, benchmarks, collections, compress, core, crypto, dist, env, fs, image, io, json, math, mem, net, python, rand, reflect, reflect_extended, regex, serde, string, sys, tensor, tests, tests_comprehensive, thread, time, coroutines

**Stub/Incomplete Modules (5):**
- `iter.omni` (30L) — Completely deferred (O-031), comments only
- `entry_point.omni` (23L) — Spec document only, no code
- `ownership_keywords.omni` (17L) — Spec document only
- `logging.omni` (18L) — 4 trivial wrapper functions
- `ffi.omni` (30L) — Type aliases and declarations only

**Modules with Pure Omni Implementations (15):** algorithm, async, benchmarks, collections, coroutines, json, math, reflect, reflect_extended, regex, serde, string, tests, tests_comprehensive, logging

**Modules Requiring FFI (13):** compress, crypto, dist, env, fs, image, io, mem, net, python, rand, sys, time, thread

**Known Issues in Specific Modules:**
- `tensor.omni:265-269` — Dead/duplicate code from `_compute_strides`
- `coroutines.omni:354` — `todo!()` in `select` function
- `thread.omni:623-629` — All 27 intrinsics unimplemented (O-082)

---

### LP-002: GPU Backend

**Status:** ⚠️ FEATURE-GATED
**Priority:** LOW
**Component:** codegen

**What Exists:**
- 8 GPU source files in codegen (~4,600 lines total)
- GPU dispatch with CUDA/Software/Mock backends (`gpu_dispatch.rs`, 1,800+L)
- GPU binary compilation: PTX, SPIR-V, Metal, WGSL (`gpu_binary.rs`, 1,204L)
- Kernel fusion pass (`gpu_fusion.rs`, 295L)
- MLIR-style tensor pipeline (`mlir.rs`, 793L)
- Self-hosted GPU codegen (`omni/compiler/codegen/gpu.omni`, 820L)

**What's Missing:**
- `gpu_hardware.rs` and `gpu_runtime.rs` are simulation stubs (correct API surface but don't call real GPU drivers)
- CUDA backend requires `nvcuda.dll`/`libcuda.so` at runtime
- Not integrated into default compilation pipeline

---

### LP-003: LLVM Backend

**Status:** ⚠️ FEATURE-GATED
**Priority:** LOW
**Component:** codegen

**What Exists:**
- Rust LLVM backend (`llvm_backend.rs`, 834L) — uses `inkwell` crate
- Self-hosted LLVM backend (`omni/compiler/codegen/llvm.omni`, 983L) — uses LLVM-C FFI

**Requirements:** `--features llvm` flag, LLVM 17+, `inkwell` dependency
**Known Issue:** `resolve_value` assumes i64 for all loaded values (O-103)

---

### LP-004: Package Manager Tests

**Status:** 🔴 MISSING TESTS
**Priority:** LOW
**Component:** tooling

`opm` package manager (2,785L across 3 files) has zero automated tests.

---

### LP-005: LSP Tests

**Status:** 🔴 MISSING TESTS
**Priority:** LOW
**Component:** tooling

`omni-lsp` (2,561L across 4 files) has zero automated tests.

---

### LP-006: DAP Tests

**Status:** 🔴 MISSING TESTS
**Priority:** LOW
**Component:** tooling

`omni-dap` (1,735L across 2 files) has zero automated tests.

---

### LP-007: Formatter Tests

**Status:** 🔴 MISSING TESTS
**Priority:** LOW
**Component:** tooling

`omni-fmt` (564L across 2 files) has zero automated tests.

---

### LP-008: GUI Runtime is Scaffolding

**Status:** ⚠️ SCAFFOLDING
**Priority:** LOW
**Component:** runtime

`runtime/gui.rs` (203L) defines window management and event types but all platform-specific calls (Win32 CreateWindowExW, X11, Wayland) are commented-out placeholders.

---

### LP-009: Distributed Runtime is Simulated

**Status:** ⚠️ SIMULATED
**Priority:** LOW
**Component:** runtime

`runtime/distributed_logic.rs` (317L) has real algorithms (ZeRO optimizer, gradient bucketing, topology discovery) but distributed operations are single-node simulations. Comments note "In real impl: ncclAllGather/ncclReduceScatter."

---

### LP-010: Hot Swap Uses Dummy IR

**Status:** ⚠️ DEMO ONLY
**Priority:** LOW
**Component:** runtime

`runtime/mod.rs:120` — hot swap path creates dummy IR instead of using real parse→IR pipeline for code reloading.

---

### LP-011: tensor_matmul is Faked

**Status:** ⚠️ INCORRECT
**Priority:** LOW
**Component:** runtime

`runtime/native.rs:170` — `tensor_matmul` native function performs element-wise addition instead of matrix multiplication.

---

### LP-012: JIT Not Connected to Main Pipeline

**Status:** ⚠️ NOT INTEGRATED
**Priority:** LOW
**Component:** codegen

`codegen/jit.rs` (1,749L) has real x86-64 machine code emission with tiered compilation, inline caching, and OSR. But it's not wired into the main compilation pipeline. The trait dispatch IC stub has a TODO (line 948).

---

### LP-013: Optional Dynamic Typing Not Implemented

**Status:** 🔴 NOT IMPLEMENTED
**Priority:** LOW
**Component:** type system

The specification mentions "optional dynamic typing" as a type system feature. The implementation has static typing only. The `reflect.omni` module provides runtime `Any` trait, but this is a library concept, not language-level dynamic typing.

---

### LP-014: Iterator Trait Deferred

**Status:** 🔴 DEFERRED
**Priority:** LOW
**Component:** stdlib
**Reference:** O-031

`std/iter.omni` is completely empty (just comments). The Iterator trait is not available. The self-hosted stdlib has its own `Iterator` trait in `omni/stdlib/core.omni`, but it's not connected to the main stdlib.

---

## 🔧 Code Quality Issues

### GFI-001: Clippy Warnings

**Status:** ⚠️ PARTIAL (240 warnings)
**Difficulty:** Easy-Medium
**Component:** compiler

**Categories:**
- ~19 `.get(0)` → `.first()`
- ~12 `div_ceil` reimplementations
- ~10 redundant closures
- ~40 unused variables
- ~8 dead code items
- ~5 complex type aliases
- GPU variable naming (camelCase → snake_case)

---

### GFI-002: Blanket `#![allow(dead_code)]`

**Status:** ⚠️ TECHNICAL DEBT
**Difficulty:** Easy
**Component:** compiler

`lib.rs` has `#![allow(dead_code)]` suppressing warnings for the entire crate. Additionally, 12+ individual files have their own `#![allow(dead_code)]` annotations. This masks genuinely unused code.

**Files with dead_code suppression:** lib.rs (blanket), lexer/mod.rs, parser/ast.rs, ir/mod.rs, semantic/mod.rs, semantic/type_inference.rs, semantic/borrow_check.rs, semantic/autograd.rs, codegen/mod.rs, codegen/ovm.rs, codegen/jit.rs, codegen/mlir.rs, codegen/native_codegen.rs, codegen/native_extended.rs, codegen/linker.rs, codegen/exception_handling.rs, codegen/gpu_dispatch.rs, codegen/gpu_binary.rs, codegen/optimizer.rs, codegen/comprehensive_tests.rs, codegen/cpp_interop.rs, runtime/mod.rs, runtime/interpreter.rs, runtime/bytecode.rs, runtime/bytecode_compiler.rs, runtime/native.rs, runtime/hot_swap.rs, runtime/profiler.rs, runtime/gui.rs, runtime/network.rs, runtime/os.rs, runtime/distributed_logic.rs

---

### GFI-003: Default Implementations

**Status:** ✅ FIXED

---

### GFI-004: Tutorial Examples

**Status:** ✅ FIXED

---

### GFI-005: `Vec` vs `Vector` Naming Inconsistency in Self-Hosted Compiler

**Status:** ⚠️ INCONSISTENCY
**Difficulty:** Easy
**Component:** omni/

`omni/compiler/macros/derive.omni` and `omni/compiler/macros/hygiene.omni` use `Vec` (Rust-style) instead of `Vector` (Omni-style). The rest of the self-hosted codebase uses `Vector`.

---

### GFI-006: `var` vs `let mut` Inconsistency

**Status:** ⚠️ INCONSISTENCY
**Difficulty:** Easy
**Component:** omni/

Some self-hosted compiler files use `var` while others use `let mut` for mutable bindings. Both are valid Omni syntax but should be standardized.

---

### GFI-007: Mixed Brace and Colon Block Syntax in stdlib

**Status:** ⚠️ INCONSISTENCY
**Difficulty:** Easy
**Component:** std/

`std/algorithm.omni` uses mixed brace `{` and colon `:` block delimiters inconsistently within the same file. Other files have similar minor inconsistencies.

---

## Help Wanted

### HW-001: Implement Bootstrap Stages 1-2

**Status:** 🔴 OPEN
**Labels:** help wanted, critical, self-hosting
**Difficulty:** Hard
**See:** SH-001

---

### HW-002: Fix Native Binary Emission

**Status:** 🔴 OPEN
**Labels:** help wanted, critical, codegen
**Difficulty:** Hard
**See:** SH-002

---

### HW-003: Closure Type Inference

**Status:** 🔴 OPEN
**Labels:** help wanted, medium, type inference
**Difficulty:** Medium
**See:** MP-001

---

### HW-004: Advanced Pattern Matching

**Status:** 🔴 OPEN
**Labels:** help wanted, medium, parser
**Difficulty:** Medium
**See:** MP-002

---

### HW-005: Implement Brace-Delimited Blocks

**Status:** 🔴 OPEN
**Labels:** help wanted, high, parser
**Difficulty:** Medium
**See:** HP-004

---

### HW-006: Implement `#[cfg]` Conditional Compilation

**Status:** 🔴 OPEN
**Labels:** help wanted, high, parser
**Difficulty:** Medium
**See:** HP-005

---

### HW-007: Implement Thread Intrinsics in OVM

**Status:** 🔴 OPEN
**Labels:** help wanted, high, runtime
**Difficulty:** Medium
**See:** HP-006

---

### HW-008: Add Tooling Test Suites

**Status:** 🔴 OPEN
**Labels:** help wanted, low, tooling, good first issue
**Difficulty:** Easy-Medium
**See:** LP-004, LP-005, LP-006, LP-007

---

### HW-009: Consolidate VM Implementations

**Status:** 🔴 OPEN
**Labels:** help wanted, medium, refactoring
**Difficulty:** Medium
**See:** MP-009

---

### HW-010: Implement Iterator Trait

**Status:** 🔴 OPEN
**Labels:** help wanted, low, stdlib
**Difficulty:** Medium
**See:** LP-014

---

## Testing Status

### Cargo Tests
```
✅ 1,019 tests passing
   - 547 lib tests
   - 472 integration tests
   - 0 failures
```

### Test Distribution by Module
| Module | Test Count | Key Coverage |
|---|---|---|
| Lexer | 18 | Keywords, operators, literals, comments, indentation |
| Parser | (integration) | All construct types |
| Semantic/Analyzer | 24 | Builtins, symbols, type unification, monomorphization, traits |
| Semantic/Type Inference | 20+ | All expression types, method calls, error classification |
| Semantic/Borrow Checker | 11 | All 11 error types |
| Semantic/Traits | 6 | User traits, object safety, GAT cache |
| Semantic/Inference | 6 | HM solver, unification, occurs check |
| Semantic/Monomorphization | 7 | Type substitution, expression/statement rewriting |
| Semantic/Constraints | 8 | Constraint solving, lifetime vars, where clauses |
| Semantic/Other | 30+ | Edge cases, error recovery, performance, properties, const generics, lifetimes |
| IR | (integration) | All instruction types |
| Codegen/OVM | (integration) | Bytecode generation and serialization |
| Codegen/JIT | 13+ | x86-64 encoding, vtable dispatch, deoptimization |
| Codegen/Native | 10+ | x86-64, ARM64, WASM, RISC-V encoding |
| Codegen/Linker | 10 | ELF, PE, Mach-O, symbol resolution, relocations |
| Codegen/Optimizer | 12+ | All 12 optimization passes |
| Codegen/Other | 50+ | DWARF, exception handling, cognitive, GPU, comprehensive pipeline |
| Runtime/VM | 29 | All opcodes, GC, recursive factorial |
| Runtime/Bytecode | 12 | Serialization roundtrip, all opcode types |
| Runtime/BytecodeCompiler | 17 | All statement/expression compilation |
| Runtime/Hot Swap | 2 | Registry update, file watching |
| Optimizer | 53 | Constant folding (19), DCE (10), inlining (9), simplify (13), pipeline (2) |
| Brain | 15 | Adaptive reasoning (5), knowledge graph (7), memory (3) |
| Language Features | 12 | Default params (4), operator overloading (3), variadics (3), lazy_static (2) |
| Resolver | 12 | All three resolvers |
| Modes | 18 | All module modes, feature restrictions |
| Enhancements | 5 | SIMD, pooling, caching |

### Example Programs

| Example | Compiles | Runs | Lines |
|---|---|---|---|
| minimal.omni | ✅ | ✅ | 3 |
| simple_test.omni | ✅ | ✅ | 8 |
| func_test.omni | ✅ | ✅ | 11 |
| func_test2.omni | ✅ | ✅ | 17 |
| hello.omni | ✅ | ✅ | 6 |
| std_demo.omni | ✅ | ✅ | 15 |
| match_comprehensive.omni | ✅ | ✅ | 12 |
| struct_test.omni | ✅ | ✅ | 14 |
| tutorial_01_basics.omni | ✅ | ✅ | 7 |
| tutorial_02_ownership.omni | ✅ | ✅ | 8 |
| tutorial_03_structs_traits.omni | ✅ | ✅ | 9 |
| tutorial_04_collections.omni | ✅ | ✅ | 7 |
| tutorial_05_async.omni | ✅ | ✅ | 6 |
| integration_test.omni | ✅ | ✅ | 5 |
| interpreter_test.omni | ✅ | ✅ | 4 |

---

## Compiler Architecture Reference

```
Source (.omni)
  │
  ▼
Phase 0: Resolver Engines (ESR/MSR/CSR strategy selection) — resolver.rs
  │
  ▼
Phase 1: Lexical Analysis — lexer/mod.rs (836L, Logos-based, indent/dedent)
  │
  ▼
Phase 2: Parsing — parser/mod.rs (2,356L, recursive descent, error recovery)
  │
  ├─ Phase 2.0: Import Resolution — main.rs::resolve_imports()
  ├─ Phase 2.1: Memory Zone Enforcement — modes.rs
  ├─ Phase 2.5: Type Inference — semantic/type_inference.rs (2,299L, HM)
  └─ Phase 2.6: Borrow Checking — semantic/borrow_check.rs (1,455L)
  │
  ▼
Phase 3: Semantic Analysis — semantic/mod.rs (2,981L)
  │
  ▼
Phase 4: IR Generation — ir/mod.rs (1,714L, SSA-based)
  │
  ▼
Phase 5: Optimization — optimizer/ (2,478L, 12+ passes, O0-O3)
  │
  ▼
Phase 6: Code Generation
  ├─ OVM Bytecode — codegen/ovm.rs + ovm_direct.rs (2,020L)
  ├─ LLVM IR — codegen/llvm_backend.rs (834L, feature-gated)
  ├─ Native — codegen/native_codegen.rs (2,459L, not end-to-end)
  ├─ JIT — codegen/jit.rs (1,749L, not integrated)
  └─ GPU — codegen/gpu_*.rs (~4,600L, feature-gated)
  │
  ▼
Runtime
  ├─ OVM Bytecode Interpreter — runtime/interpreter.rs (3,817L)
  ├─ High-Level VM — runtime/vm.rs (1,792L)
  ├─ Standalone OVM (Rust) — ovm/src/main.rs (1,415L)
  └─ Standalone OVM (C) — ovm/ovm.c (873L)
```

---

## Repository Layout

```
omni-lang/
├── compiler/              # Omni compiler in Rust (93 files, ~35,000L)
│   ├── src/
│   │   ├── main.rs       # CLI entry point (801L)
│   │   ├── lib.rs        # Library root (58L)
│   │   ├── diagnostics.rs # Error reporting (83L)
│   │   ├── enhancements.rs # SIMD/cache/security (393L)
│   │   ├── modes.rs      # Module modes (591L)
│   │   ├── monitor.rs    # Build monitoring (114L)
│   │   ├── resolver.rs   # Strategy resolvers (517L)
│   │   ├── lexer/        # Tokenization (836L)
│   │   ├── parser/       # AST generation (2,851L)
│   │   ├── semantic/     # Type inference, borrow checking (18 files, ~10,000L)
│   │   ├── ir/           # Intermediate representation (1,714L)
│   │   ├── codegen/      # Code generation (33 files, ~22,000L)
│   │   ├── runtime/      # Bytecode VM/interpreter (13 files, ~14,000L)
│   │   ├── brain/        # Reasoning modules (4 files, ~800L)
│   │   ├── optimizer/    # Optimizations (5 files, ~2,500L)
│   │   ├── safety/       # Safety passes (2 files, ~470L)
│   │   └── language_features/ # Extended features (5 files, ~950L)
│   └── Cargo.toml
├── std/                   # Standard library (37 files, ~20,770L)
├── core/                  # Core library for Omni (12 files, ~3,254L)
├── omni/                  # Self-hosted compiler source (34 files, ~23,000L)
│   ├── main.omni         # Entry point
│   ├── bootstrap.omni    # 3-stage bootstrap (684L)
│   ├── compiler_minimal.omni # Minimal bootstrap compiler (15L)
│   ├── compiler/         # Full self-hosted compiler
│   │   ├── main.omni     # Compiler driver (624L)
│   │   ├── lexer/        # Tokenizer (1,635L)
│   │   ├── parser/       # Parser + AST (2,303L)
│   │   ├── semantic/     # Semantic analysis (3,073L)
│   │   ├── ir/           # IR + optimization (2,167L)
│   │   ├── codegen/      # OVM/LLVM/GPU backends (2,698L)
│   │   ├── linker/       # Linker (709L)
│   │   └── macros/       # Derive + hygiene (1,150L)
│   ├── runtime/          # Runtime module (697L)
│   ├── stdlib/           # Self-hosted stdlib (9 files, ~9,900L)
│   ├── tools/            # Build + OPM (1,801L)
│   └── tests/            # Test runner (424L)
├── ovm/                   # OVM Virtual Machine (4 files, ~2,326L)
│   ├── src/main.rs       # Rust OVM runner (1,415L)
│   ├── ovm.c             # C OVM (standalone, 873L)
│   ├── Cargo.toml
│   └── Makefile
├── tools/                 # Developer tools (20 files, ~8,294L)
│   ├── omni-lsp/         # Language Server (4 files, ~2,561L)
│   ├── omni-dap/         # Debug Adapter (2 files, ~1,735L)
│   ├── omni-fmt/         # Formatter (2 files, ~564L)
│   ├── opm/              # Package Manager (3 files, ~2,785L)
│   └── vscode-omni/      # VS Code Extension (4 files, ~641L)
├── examples/              # Example programs (15 files)
├── tests/                 # Integration tests (8 files)
├── docs/                  # Documentation (4 files)
│   ├── grammar.bnf       # Formal grammar (451L)
│   ├── IMPLEMENTATION_STATUS.md
│   ├── IMPLEMENTATION_PLAN.md
│   └── SELF_HOSTING_TODO.md
├── build/                 # Build artifacts (48 .ovm files)
├── Omni.toml             # Project manifest
├── bootstrap.sh          # Bootstrap script (219L)
└── ISSUES.md             # This file
```

---

## Label Reference

| Label | Meaning |
|---|---|
| `SH-*` | Self-Hosting blockers (CRITICAL) |
| `HP-*` | High Priority issues |
| `MP-*` | Medium Priority issues |
| `LP-*` | Low Priority issues |
| `GFI-*` | Good First Issues (code quality) |
| `HW-*` | Help Wanted issues |
| `O-*` | Original issue references from code comments |

---

## Roadmap

### Current Focus (Phase 3: Enrichment — 65% Complete)
1. ✅ Type inference improvements (COMPLETED)
2. 🔄 Bootstrap implementation (IN PROGRESS — Stage 0 works, Stages 1-2 needed)
3. 🔄 Closure support (IN PROGRESS — explicit types work, inference missing)
4. 🔴 Brace-delimited syntax support (HP-004)
5. 🔴 Conditional compilation support (HP-005)
6. 🔴 Thread intrinsics in OVM (HP-006)

### Next Steps (Phase 4: Optimization — 50% Complete)
1. 🔴 Fix native binary emission (SH-002)
2. 🔴 Complete LLVM backend integration
3. 🔄 Connect JIT to main pipeline (LP-012)
4. ✅ Optimizer framework (12 passes working)

### Future (Phase 5: Self-Hosting — 30% Complete)
1. 🔴 Implement self-hosted compiler bootstrap (SH-001)
2. 🔴 Bootstrap stages 1-2 (SH-001)
3. 🔴 Full self-hosting verification (SHA-256 bit-identical)
4. 🔴 Remove Rust dependency (SH-003)

---

**Last Updated:** 2026-03-30

**Audit Method:** Full byte-by-byte code review of all 170+ source files by automated analysis agents

**Maintained By:** Omni Language Team

**Contributing:** See [CONTRIBUTING.md](../CONTRIBUTING.md)
