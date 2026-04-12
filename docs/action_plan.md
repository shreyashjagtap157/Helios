# Omni Compiler: Exhaustive Remediation & Architectural Strategy

**Date:** 2026-04-10
**Status:** CRITICAL PATH ALIGNMENT

Based on the exhaustive technical audit, the Omni compiler has severe architectural fractures—most notably the lack of AST spans, the absence of a proper Mid-level IR (MIR), and the existence of a massive volume of premature Phase 7-13 code (GPU, JIT) built on a non-viable foundation. Furthermore, two parallel parser/AST systems coexist, causing friction.

To achieve the stated goals of memory safety enforcement, specification compliance, and self-hosting, the project must undergo a strict, phased remediation. This report details the exhaustive step-by-step strategy.

---

## Triage Phase: Codebase Pruning & Stabilization (Weeks 1-2)

Before any architectural forward-progress can occur, the project must be ruthlessly pruned of technical debt that is masking the true state of the compiler.

### 1. Extirpate the "OLD" Compiler System
**Problem:** The `compiler/src/` directory contains two completely different parsers and AST definitions. `stage1-compiler/` imports the "OLD" files, while [main.rs](file:///d:/Project/Helios/omni-lang/ovm/src/main.rs) imports the "NEW" files.
**Action:**
- Delete [compiler/src/lexer.rs](file:///d:/Project/Helios/omni-lang/compiler/src/lexer.rs), [compiler/src/parser.rs](file:///d:/Project/Helios/omni-lang/compiler/src/parser.rs), [compiler/src/ast.rs](file:///d:/Project/Helios/omni-lang/compiler/src/ast.rs), [compiler/src/ir.rs](file:///d:/Project/Helios/omni-lang/compiler/src/ir.rs), [compiler/src/codegen.rs](file:///d:/Project/Helios/omni-lang/compiler/src/codegen.rs), and [compiler/src/semantics.rs](file:///d:/Project/Helios/omni-lang/compiler/src/semantics.rs).
- Update [stage1-compiler/src/main.rs](file:///d:/Project/Helios/omni-lang/stage1-compiler/src/main.rs) to route through [compiler/src/lexer/mod.rs](file:///d:/Project/Helios/omni-lang/compiler/src/lexer/mod.rs) and the new pipeline, or temporarily disable `stage1-compiler` entirely until the `compiler/` crate is stabilized.
- Delete the 624 diagnostic stall logs in `diagnostics/` to clean the workspace.

### 2. Freeze and Isolate Premature Backend Code
**Problem:** ~430,000 lines of code across `codegen/` and [semantic/autograd.rs](file:///d:/Project/Helios/omni-lang/compiler/src/semantic/autograd.rs) represent GPU, MLIR, JIT, and advanced optimization passes built around a fundamentally flawed IR that lacks ownership and control-flow semantics.
**Action:**
- Move all files related to GPU, JIT, native x86, and MLIR into a `compiler/src/archive_experimental/` folder and remove them from the module tree ([mod.rs](file:///d:/Project/Helios/omni-lang/compiler/src/lexer/mod.rs)).
- The ONLY allowed code generation target for the next 6 months is the **OVM bytecode emitter** ([codegen/ovm.rs](file:///d:/Project/Helios/omni-lang/compiler/src/codegen/ovm.rs) / [ovm_direct.rs](file:///d:/Project/Helios/omni-lang/compiler/src/codegen/ovm_direct.rs)).

### 3. Establish Project Strictness
**Problem:** `#![allow(dead_code)]` hides hundreds of structural issues. Compiler crate has heavy runtime dependencies (tokio, reqwest). Zero Parser tests.
**Action:**
- Remove `#![allow(dead_code)]` from all [mod.rs](file:///d:/Project/Helios/omni-lang/compiler/src/lexer/mod.rs) files. Fix the resulting warnings or explicitly `_` prefix unused fields that are genuinely planned for the immediate next phase.
- Remove `tokio`, `sysinfo`, `reqwest`, etc., from [compiler/Cargo.toml](file:///d:/Project/Helios/omni-lang/compiler/Cargo.toml). The compiler itself does not need a runtime executor.
- Set up a GitHub Action running `cargo test`, `cargo clippy -- -D warnings`, and `cargo fmt`.
- Add 50+ snapshot tests for the parser to guarantee syntax stability.

---

## Phase 1: Foundational Information Recovery (Weeks 3-4)

The language cannot have an LSP, "Did you mean?" semantics, proper error reporting, or a borrow checker without knowing exactly where code came from.

### 1. Implement Source Spans Everywhere
**Problem:** Currently, when the [Lexer](file:///d:/Project/Helios/omni-lang/compiler/src/lexer/mod.rs#339-343) produces tokens, they have line/column info. But once folded into the `AST`, all location information evaporates.
**Action:**
- Introduce a `Span { start: usize, end: usize }` struct.
- Introduce `Spanned<T> { node: T, span: Span }`.
- Rewrite [parser/ast.rs](file:///d:/Project/Helios/omni-lang/compiler/src/parser/ast.rs) so that EVERY node `Item`, `Statement`, `Expression`, and [Type](file:///d:/Project/Helios/omni-lang/compiler/src/semantic/mod.rs#394-399) holds a `Span`.
- Update the Logos lexer integration to thread byte offsets to the AST nodes.
- Update `SemanticError` to utilize these spans to emit rich, rustc-style error messages with context pointers.

### 2. Lossless Syntax Tree (Optional but Recommended)
**Action:** Consider migrating from the recursive-descent lossy AST to the `rowan` crate (Red-Green Trees). This preserves whitespace and comments, automatically enabling `omni-fmt` and `omni-lsp` to manipulate code safely.

---

## Phase 2: The Missing Link — MIR (Weeks 5-8)

This is the single most critical architectural failure in Omni. You cannot write a borrow checker on a raw AST properly, because ASTs do not flatten control flow. 

### 1. Design Omni-MIR
**Problem:** The current `ir/mod.rs` is a naïve SSA generator. The borrow checker currently (flawedly) walks the AST directly.
**Action:**
Build a Mid-level Intermediate Representation explicitly designed for ownership tracking.
- **Basic Blocks:** Linear sequences of statements ending in a terminator (Goto, SwitchInt, Return, Drop).
- **Locals and Places:** Replace complex expressions with memory locations (e.g., `_1`, `_2.field[0]`).
- **Rvalues and Constants:** Assignments only occur from Rvalues to Places.
- **Explicit Drops:** Control flow branches must explicitly generate `Drop(_X)` terminators when variables go out of scope.

### 2. Lowering AST to MIR
**Action:**
- Write `compiler/src/mir/lower.rs`. 
- Desugar [while](file:///d:/Project/Helios/omni-lang/compiler/src/parser/mod.rs#1836-1844), [for](file:///d:/Project/Helios/omni-lang/compiler/src/parser/mod.rs#1808-1835), [match](file:///d:/Project/Helios/omni-lang/compiler/src/parser/mod.rs#1845-1924), and [if](file:///d:/Project/Helios/omni-lang/compiler/src/semantic/mod.rs#1589-1646) loops into pure Goto networks of basic blocks.
- Thread the lifetimes inferred during semantic analysis into the MIR places.

---

## Phase 3: The True Borrow Checker (Weeks 9-12)

With MIR in place, Omni can enforce memory safety.

### 1. Dataflow Analysis
**Action:**
- Implement Gen/Kill sets on the MIR basic blocks.
- Calculate liveness, initialization states, and uninitialized data paths.

### 2. Erase the AST Borrow Checker & Rewrite on MIR
**Problem:** [semantic/borrow_check.rs](file:///d:/Project/Helios/omni-lang/compiler/src/semantic/borrow_check.rs) emits warnings and is fundamentally incapable of resolving conditionally initialized variables (e.g., assigned in an [if](file:///d:/Project/Helios/omni-lang/compiler/src/semantic/mod.rs#1589-1646) but not the [else](file:///d:/Project/Helios/omni-lang/compiler/src/parser/mod.rs#3649-3663)).
**Action:**
- Delete the existing [semantic/borrow_check.rs](file:///d:/Project/Helios/omni-lang/compiler/src/semantic/borrow_check.rs).
- Implement Non-Lexical Lifetimes (NLL) tracing over the MIR.
- Enforce strict Ownership checks: Use-after-move, Double-mut-borrow, Mut-alias-shared-borrow MUST return `Err` and explicitly abort compilation. Warnings for memory safety violations are unacceptable.

---

## Phase 4: Validating the Vertical Slice (Weeks 13-16)

The language needs to be proven via dogfooding.

### 1. Standard Library Implementation
**Problem:** The `std/` directory contains 37 [.omni](file:///d:/Project/Helios/omni-lang/std/io.omni) files that are purely signatures.
**Action:**
- Write the actual Omni bodies for [core.omni](file:///d:/Project/Helios/omni-lang/std/core.omni) (Option, Result).
- Write [string.omni](file:///d:/Project/Helios/omni-lang/std/string.omni) and [collections.omni](file:///d:/Project/Helios/omni-lang/std/collections.omni) bodies utilizing the newly hardened borrow checker to ensure they are provably safe.

### 2. Unblock Self-Hosting
**Problem:** The self-hosted compiler files (`omni/compiler/`) use syntax the Rust compiler doesn't understand, and the [stage1.ovm](file:///d:/Project/Helios/omni-lang/build/omnc-stage1.ovm) files are empty.
**Action:**
- Reconcile the Rust parser ([compiler/src/parser/mod.rs](file:///d:/Project/Helios/omni-lang/compiler/src/parser/mod.rs)) to perfectly match the syntax used in the `omni/compiler/` codebase (or vice versa).
- Once they match, compile `omni/compiler/` using the Rust `omnc`, outputting a 100% valid [stage1.ovm](file:///d:/Project/Helios/omni-lang/build/omnc-stage1.ovm) file.
- Verify `ovm-runner stage1.ovm` behaves identically to the Rust compiler.

### 3. Implement The Effect System (REQ-04)
**Action:** 
- Only after basic semantics are rock-solid, resurrect the [semantic/effects.rs](file:///d:/Project/Helios/omni-lang/compiler/src/semantic/effects.rs) stub.
- Add syntactic support for `/ effect` capabilities.
- Integrate capability checking into the Semantic Analyzer.

---

## Phase 5: Re-enabling the Backends (Month 5+)

Only after MIR, Borrow Checking, and Self-Hosting (OVM) work flawlessly, do we resurrect the advanced backends.

### 1. MIR Optimization Pipeline
**Action:** 
- Write optimization passes against the MIR, *not* the AST. (Constant Propagation, Dead Code Elimination, Inlining).

### 2. LLVM / Native Backend Resurrection
**Action:**
- Rewrite [codegen/llvm_backend.rs](file:///d:/Project/Helios/omni-lang/compiler/src/codegen/llvm_backend.rs) to consume MIR rather than the typed AST.
- MIR maps trivially perfectly to LLVM IR (Basic Blocks -> Basic Blocks).

### 3. GPU/JIT/MLIR Re-integration
**Action:**
- Bring the 430K lines of experimental code back from `archive_experimental/`, porting them generator-by-generator to consume the stabilized MIR.

---

## Summary of Architectural Constraints moving forward:
1. **Never compile advanced backends on top of an unstable frontend.**
2. **Memory safety is binary.** Borrow checker errors must fail the build, never warn.
3. **Control flow requires MIR.** Do not write complex analyses on ASTs.
4. **Source of Truth:** Spans must be tracked from token generation down through MIR generation to ensure errors are legible to humans.
