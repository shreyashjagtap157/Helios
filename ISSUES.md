# Helios + Omni Issues

> **Note:** This repository contains two projects:
> - **`omni-lang/`** - The Omni programming language (active development)
> - **`helios-framework/`** - The Helios cognitive framework (separate project)
>
> **For the exhaustive Omni-specific issue tracker, see:** [omni-lang/ISSUES.md](omni-lang/ISSUES.md)
>
> **This file tracks cross-cutting concerns and provides a high-level project overview.**
>
> **Audit Date:** 2026-03-30 — Full byte-by-byte code audit performed on all 170+ source files (~85,000+ lines)

---

## Project Structure

```
Helios/
├── omni-lang/                # Omni Programming Language (ACTIVE)
│   ├── compiler/            # Rust-based compiler (93 files, ~35,000L)
│   │   ├── src/lexer/      # Tokenization (836L, Logos-based)
│   │   ├── src/parser/     # AST generation (2,851L, recursive descent)
│   │   ├── src/semantic/   # Type inference, borrow checking (18 files, ~10,000L)
│   │   ├── src/ir/         # SSA-based IR (1,714L)
│   │   ├── src/codegen/    # Code generation (33 files, ~22,000L)
│   │   ├── src/runtime/    # Bytecode VM/interpreter (13 files, ~14,000L)
│   │   ├── src/optimizer/  # 12+ optimization passes (2,500L)
│   │   ├── src/brain/      # Adaptive reasoning (800L)
│   │   ├── src/safety/     # Safety analysis (470L)
│   │   └── src/language_features/ # Extended features (950L)
│   ├── std/                 # Standard library (37 files, ~20,770L)
│   ├── core/                # Core library for Helios integration (12 files, ~3,254L)
│   ├── omni/                # Self-hosted compiler source (34 files, ~23,000L)
│   ├── ovm/                 # OVM Virtual Machine — Rust (1,415L) + C (873L)
│   ├── tools/               # Developer tooling (20 files, ~8,294L)
│   │   ├── omni-lsp/       # Language Server Protocol
│   │   ├── omni-dap/       # Debug Adapter Protocol
│   │   ├── omni-fmt/       # Code formatter
│   │   ├── opm/            # Package manager
│   │   └── vscode-omni/    # VS Code extension
│   ├── examples/            # Example programs (15 files, all working)
│   ├── tests/               # Integration tests (8 files)
│   ├── docs/                # Documentation (grammar, status, plans)
│   ├── build/               # Build artifacts (48 .ovm files)
│   └── ISSUES.md            # Exhaustive Omni-specific issues
│
└── helios-framework/        # Helios Cognitive Framework (SEPARATE)
    └── ...                  # Not part of current work
```

---

## Omni Language — Specification Compliance Overview

A full audit was performed comparing the implementation against the Omni specification. The specification describes a phased, multi-paradigm, self-hosting programming language. Here is the high-level compliance status:

### Phase Completion

| Phase | Spec Description | Compliance | Status |
|---|---|---|---|
| **Phase 1: Structural Foundation** | Core syntax, parsing, basic IR | **100%** | ✅ Complete |
| **Phase 2: Core Functionality** | Type system, memory management, stdlib, pipeline | **95%** | ✅ Complete (minor gaps in closures, module resolution) |
| **Phase 3: Enrichment** | Advanced types, concurrency, comptime, tooling | **65%** | 🔄 In Progress |
| **Phase 4: Optimization** | High-perf execution, cross-platform, ecosystem | **50%** | 🔄 In Progress |
| **Phase 5: Self-Hosting** | Compiler in Omni, bootstrap verification | **30%** | 🔴 Blocked |

### Specification Feature Compliance

| Specification Feature | Status | Notes |
|---|---|---|
| Multi-paradigm (procedural, OOP, functional, data-oriented) | ✅ | All four paradigms supported |
| Expression-oriented | ✅ | If-expr, match-expr, block-expr all work |
| Indentation-based syntax | ✅ | Fully working |
| Brace-based syntax | 🔴 | **NOT IMPLEMENTED** — spec says both, only indent works |
| Static typing (default) | ✅ | HM type inference (2,299L) |
| Optional dynamic typing | 🔴 | **NOT IMPLEMENTED** |
| Advanced type inference | ✅ | Constraint-based HM solver |
| Generics | ⚠️ | Basic generics work, GATs/complex where clauses missing |
| Compile-time computation | ⚠️ | `comptime` parsed, limited execution |
| Ownership-based memory | ✅ | Full borrow checker (1,455L, 11 error types) |
| Garbage collection | ✅ | Tri-color incremental mark-and-sweep in OVM |
| Manual memory management | ⚠️ | Declared (unsafe pointers), not runtime-tested |
| Multiple memory models coexist | ⚠️ | MSR resolver exists, runtime switching untested |
| Async/await | ⚠️ | Parsed and declared, thread intrinsics unimplemented (O-082) |
| Parallel execution | ⚠️ | Declared in stdlib, thread intrinsics unimplemented |
| Distributed systems | ⚠️ | Declared in stdlib (NCCL/MPI/Raft), simulated not real |
| AOT compilation (bytecode) | ✅ | Working — omnc → .ovm → OVM |
| AOT compilation (native) | 🔴 | Infrastructure exists, end-to-end **NOT WORKING** |
| JIT compilation | ⚠️ | Framework exists (1,749L), not integrated into pipeline |
| Modular pipeline | ✅ | Resolver-based strategy selection |
| Package management | ✅ | opm with init/add/remove/build/run |
| Build system | ✅ | Build tool in self-hosted compiler |
| Debugging tools | ✅ | DAP adapter with breakpoints, stepping, variables |
| Profiling tools | ✅ | Runtime profiler with PGO, CPU detection |
| IDE support | ✅ | VS Code extension with LSP, formatting, debug |
| Self-hosting source | ✅ | 23,000L of Omni compiler in Omni |
| Self-hosting verification | 🔴 | Stages 1-2 **NOT IMPLEMENTED** |
| Apache 2.0 license | ✅ | Present |
| Contribution guidelines | ✅ | CONTRIBUTING.md present |

### Quantitative Summary

| Metric | Value |
|---|---|
| Total source files | 170+ |
| Total lines of code | ~85,000+ |
| Cargo tests passing | 1,019 (0 failures) |
| Example programs working | 15/15 |
| Standard library modules | 37 (30 with real implementation, 5 stubs) |
| Self-hosted compiler files | 34 (~23,000 lines) |
| Codegen backends | 5 (OVM bytecode, LLVM, native, JIT, GPU) |
| Optimization passes | 12+ |
| Target architectures | x86-64, ARM64, RISC-V, WASM (emitters exist) |
| Developer tools | 5 (LSP, DAP, formatter, package manager, VS Code) |

---

## Cross-Cutting Issues

### CC-001: Self-Hosting Not Achieved

**Status:** 🔴 CRITICAL

**Description:** Omni cannot compile itself without Rust. The self-hosted compiler source exists (34 files, 23,000 lines) and mirrors the Rust compiler's full pipeline (lexer → parser → semantic → IR → optimizer → codegen → linker). Stage 0 works (Rust compiles Omni to bytecode). Stages 1-2 (self-hosted compiles self-hosted, bit-identical verification) are not implemented.

**Blocking Factors:**
1. `omni/compiler_minimal.omni` is a 15-line stub, not a real compiler
2. Self-hosted source uses syntax constructs the Rust parser may not handle
3. Monomorphization must specialize generic functions end-to-end
4. Bootstrap verification code (SHA-256) exists in `bootstrap.omni` but cannot be exercised

**See:** [omni-lang/ISSUES.md](omni-lang/ISSUES.md) → SH-001

---

### CC-002: Standalone Runtime Not Achieved

**Status:** 🔴 CRITICAL

**Description:** The specification requires Omni to be "standalone: not dependent on external runtimes." The compiler currently requires Rust/Cargo. Two paths to standalone exist:
1. **C OVM** (`ovm/ovm.c`, 873L) — independent C implementation with no Rust dependency
2. **Native binary emission** — codegen infrastructure exists but doesn't produce working executables

**See:** [omni-lang/ISSUES.md](omni-lang/ISSUES.md) → SH-002, SH-003

---

### CC-003: Concurrency Not Runtime-Verified

**Status:** 🔴 HIGH

**Description:** The specification lists concurrency as "a first-class concern." The Omni implementation declares a full concurrency stack:
- `std/thread.omni` (660L) — threads, mutexes, rwlocks, channels, atomics
- `std/async.omni` (544L) — futures, executor, thread pool, timers
- `std/coroutines.omni` (390L) — coroutines, generators

However, all 27 thread intrinsics declared in `std/thread.omni` are **NOT IMPLEMENTED** in the OVM runtime (O-082). This means all concurrency primitives are non-functional at runtime.

**See:** [omni-lang/ISSUES.md](omni-lang/ISSUES.md) → HP-006

---

### CC-004: Brace-Delimited Syntax Not Supported

**Status:** 🔴 HIGH

**Description:** The specification says Omni supports "both indentation-based and brace-based styles." The parser only handles indentation-based (colon + indent/dedent). 7 stdlib files have WARNING headers about unparseable brace-delimited extern blocks.

**See:** [omni-lang/ISSUES.md](omni-lang/ISSUES.md) → HP-004

---

### CC-005: Conditional Compilation Not Implemented

**Status:** 🔴 HIGH

**Description:** 9 stdlib files use `#[cfg(unix)]` / `#[cfg(windows)]` for platform-specific code, but the compiler does not process these attributes. One file uses non-standard `@cfg()` syntax.

**See:** [omni-lang/ISSUES.md](omni-lang/ISSUES.md) → HP-005

---

### CC-006: Native Binary Emission Not Working

**Status:** 🔴 HIGH

**Description:** `omnc --emit native` does not produce working executables. The codegen infrastructure exists:
- Native codegen: x86-64, ARM64, RISC-V, WASM emitters (2,459L + 1,051L)
- Linker: ELF64, PE/COFF, Mach-O emission (1,600L + 1,291L)
- DWARF debug info emitter (424L)

But the end-to-end pipeline (IR → native codegen → linking → executable) is not connected.

**See:** [omni-lang/ISSUES.md](omni-lang/ISSUES.md) → SH-002

---

### CC-007: Technical Debt — Duplicate Systems

**Status:** ⚠️ MEDIUM

**Description:** Several subsystems have duplicate implementations that should be consolidated:
1. **Two type inference engines**: `semantic/inference.rs` (328L) and `semantic/type_inference.rs` (2,299L)
2. **Two SemanticError types**: in `semantic/mod.rs` and `semantic/error_recovery.rs`
3. **Three VM/interpreter implementations**: `runtime/interpreter.rs` (3,817L), `runtime/vm.rs` (1,792L), `ovm/src/main.rs` (1,415L)
4. **Blanket `#![allow(dead_code)]`** in `lib.rs` masking unused code across 32+ files

**See:** [omni-lang/ISSUES.md](omni-lang/ISSUES.md) → MP-008, MP-009, GFI-002

---

### CC-008: Tooling Lacks Tests

**Status:** ⚠️ MEDIUM

**Description:** All four developer tools (opm, omni-lsp, omni-dap, omni-fmt) totaling 7,645 lines have zero automated tests.

**See:** [omni-lang/ISSUES.md](omni-lang/ISSUES.md) → LP-004 through LP-007

---

### CC-009: Helios Framework Scope

**Status:** 📋 SEPARATE PROJECT

**Description:** The Helios framework (`helios-framework/`) is maintained separately from the Omni language. The `omni-lang/core/` directory provides integration modules (CUDA, HTTP, JSON, math, networking, threading, TOML, voice processing).

---

## Priority Matrix

### Must Fix (Blocks Specification Compliance)

| Issue | Priority | Effort | Impact |
|---|---|---|---|
| SH-001: Bootstrap Stages 1-2 | CRITICAL | Hard | Self-hosting impossible |
| SH-002: Native binary emission | CRITICAL | Hard | Standalone impossible |
| SH-003: Standalone runtime | CRITICAL | Medium | Runtime independence |
| HP-004: Brace-delimited syntax | HIGH | Medium | 7 stdlib files unparseable |
| HP-005: `#[cfg]` support | HIGH | Medium | 9 stdlib files have dead platform code |
| HP-006: Thread intrinsics | HIGH | Medium | All concurrency non-functional |

### Should Fix (Improves Quality)

| Issue | Priority | Effort | Impact |
|---|---|---|---|
| MP-001: Closure type inference | MEDIUM | Medium | Functional programming incomplete |
| MP-002: Complex patterns | MEDIUM | Medium | Pattern matching incomplete |
| MP-003: Advanced generics | MEDIUM | Medium | Generic programming incomplete |
| MP-008: Duplicate type systems | MEDIUM | Easy | Code clarity |
| MP-009: Duplicate VMs | MEDIUM | Medium | Maintainability |
| CC-008: Tooling tests | MEDIUM | Easy | Reliability |

### Nice to Have (Enrichment)

| Issue | Priority | Effort | Impact |
|---|---|---|---|
| LP-012: JIT integration | LOW | Medium | Performance |
| LP-013: Dynamic typing | LOW | Hard | Specification compliance |
| LP-014: Iterator trait | LOW | Medium | Stdlib completeness |
| LP-002: GPU backend | LOW | Hard | Domain coverage |
| LP-003: LLVM backend | LOW | Medium | Native performance |

---

## Contributing

For Omni language issues, see:
- [omni-lang/ISSUES.md](omni-lang/ISSUES.md) — Exhaustive issue tracker with 40+ tracked items
- [CONTRIBUTING.md](CONTRIBUTING.md) — Contribution guidelines

### Quick Start for Contributors

**Good First Issues (Easy):**
- GFI-001: Fix clippy warnings (240 remaining)
- GFI-002: Remove blanket `#![allow(dead_code)]` and fix actual dead code
- GFI-005: Standardize `Vec` → `Vector` in self-hosted compiler
- GFI-006: Standardize `var` → `let mut` in self-hosted compiler
- HW-008: Add test suites for opm/omni-lsp/omni-dap/omni-fmt

**Medium Issues:**
- HW-003: Closure type inference
- HW-004: Advanced pattern matching
- HW-005: Brace-delimited block parsing
- HW-006: `#[cfg]` conditional compilation
- HW-007: Thread intrinsics in OVM

**Hard Issues (Core Contributors):**
- HW-001: Bootstrap Stages 1-2
- HW-002: Native binary emission end-to-end

---

**Last Updated:** 2026-03-30

**Audit Method:** Full byte-by-byte code review of all 170+ source files (~85,000+ lines) by automated analysis agents
