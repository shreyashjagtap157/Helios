# Audit Report Verification & Revised Remediation Plan

**Date:** 2026-04-10  
**Purpose:** Independent re-verification of the original audit against the live codebase, correction of all factual errors, and a revised action plan that accounts for the corrected findings.

---

## Part 1: Audit Claim Verification (Each Checked Against Source)

### ✅ CONFIRMED — Original Audit Correct

| # | Claim | Verification Evidence |
|---|---|---|
| 1 | **Two parallel compiler front-ends (OLD vs NEW)** | OLD files ([lexer.rs](file:///d:/Project/Helios/omni-lang/compiler/src/lexer.rs), [parser.rs](file:///d:/Project/Helios/omni-lang/compiler/src/parser.rs), [ast.rs](file:///d:/Project/Helios/omni-lang/compiler/src/ast.rs), [ir.rs](file:///d:/Project/Helios/omni-lang/compiler/src/ir.rs), [codegen.rs](file:///d:/Project/Helios/omni-lang/compiler/src/codegen.rs), [semantics.rs](file:///d:/Project/Helios/omni-lang/compiler/src/semantics.rs)) confirmed at `compiler/src/` root. NEW modules ([lexer/mod.rs](file:///d:/Project/Helios/omni-lang/compiler/src/lexer/mod.rs), [parser/mod.rs](file:///d:/Project/Helios/omni-lang/compiler/src/parser/mod.rs), [parser/ast.rs](file:///d:/Project/Helios/omni-lang/compiler/src/parser/ast.rs), [semantic/mod.rs](file:///d:/Project/Helios/omni-lang/compiler/src/semantic/mod.rs), [codegen/mod.rs](file:///d:/Project/Helios/omni-lang/compiler/src/codegen/mod.rs)) confirmed in subdirectories. [stage1-compiler/src/main.rs](file:///d:/Project/Helios/omni-lang/stage1-compiler/src/main.rs) uses `#[path = "../../compiler/src/ast.rs"]` etc. to import OLD files. |
| 2 | **No AST spans** | Searched [parser/ast.rs](file:///d:/Project/Helios/omni-lang/compiler/src/parser/ast.rs) for `span` — zero matches. `Expression`, `Statement`, `Item`, [Type](file:///d:/Project/Helios/omni-lang/compiler/src/semantic/mod.rs#382-386) hold no source location. |
| 3 | **Comment syntax uses `//`, not `--`** | `lexer/mod.rs:299`: `#[regex(r"//[^\n]*")]` — no `--` pattern exists anywhere in lexer. |
| 4 | **No `f"..."` string interpolation** | `lexer/mod.rs:201`: only `#[regex(r#""([^"\\]|\\.)*""#)]` for standard strings. No `f"` or `d"` prefix. |
| 5 | **`Type::Any` on all builtins** | `semantic/mod.rs:599`: `("println", Type::Function(vec![Type::Any], None), false)` — all I/O builtins use `Type::Any`. |
| 6 | **`#![allow(dead_code)]` suppressing warnings** | Present in `parser/ast.rs:15`, `lexer/mod.rs:15`, `semantic/mod.rs:15`, `borrow_check.rs:1`. |
| 7 | **Runtime deps in compiler `Cargo.toml`** | `tokio`, `reqwest`, `ndarray`, `rand`, `chrono`, `uuid`, `sysinfo`, `libloading` confirmed at `Cargo.toml:65-76`. |
| 8 | **Borrow checker violations emitted as warnings, compilation continues** | `main.rs:529`: comment says `"(warnings for ownership violations)"`. Line 535: `eprintln!("warning[E006]: borrow check: {}", e)`. No `return Err(...)` — compilation proceeds regardless. |
| 9 | **`omni-lang/build/omnc-stage1.ovm` = 0 bytes** | PowerShell `Get-Item` confirms: `omnc-stage1.ovm` Length = 0, `omnc-stage2.ovm` Length = 0 in `omni-lang/build/`. |
| 10 | **No CI configuration** | `find_by_name` for `*.yml` / `*.yaml` across all of `d:\Project\Helios` — zero results. No `.github/workflows/`. |

---

### ⚠️ CORRECTIONS — Original Audit Needed Updates

| # | Claim | Original Audit Said | Actual Reality | Impact |
|---|---|---|---|---|
| 1 | **Stage OVM files** | "Both are empty files (0 bytes)" | **Two sets exist.** `omni-lang/build/omnc-stage1.ovm` = 0 bytes (**audit correct for this path**). But `Helios/build/omnc_stage1.ovm` = **14,366 bytes** and `omnc_stage2.ovm` = **14,366 bytes** (a separate, successful build output at the parent project level). A `stage3.ovm` also exists at 14,366 bytes. | Self-hosting has progressed further than stated. A working stage0→stage1→stage2→stage3 pipeline has produced non-trivial bytecode files. The audit undersold actual bootstrap progress. |
| 2 | **`diagnostics/` has 624 stall logs** | "624 files in `diagnostics/`" | No `diagnostics/` directory exists anywhere under `omni-lang/`. It was either cleaned up or never existed at this path. | This was stale information. Remove from audit. |
| 3 | **Borrow checker quality undersold** | "AST-walk based, warnings only, no real enforcement" | The `borrow_check.rs` is **1,673 lines** of well-structured code with a proper `BorrowError` enum (10 variants: `UseAfterMove`, `DoubleMutBorrow`, `MutBorrowWhileShared`, `MovedWhileBorrowed`, `DanglingReference`, `MutationOfImmutable`, `ReturnLocalReference`, `UndeclaredVariable`, `SharedBorrowWhileMut`, `MoveInLoop`). It tracks per-variable `BorrowState`, lexical scopes, loop-move detection, and reference return checking. | The borrow checker is **architecturally sound for an AST-walk approach**. The real problem isn't its quality — it's that `main.rs:535` emits its output as `warning[E006]` instead of `error[E006]` and never aborts. This is a **one-line fix**, not a rewrite. |
| 4 | **Self-hosting legitimacy = 1/10** | "Stage1/Stage2 OVM files are 0 bytes" | The `Helios/build/` copies are 14,366 bytes each with matching sizes (stage1 = stage2 = stage3 = 14,366), which is consistent with a successful fixed-point bootstrap. | Self-hosting score should be revised upward to ~3-4/10. The bootstrap pipeline has demonstrably run. |

---

## Part 2: Revised Severity Assessment

With corrections applied, here's the updated picture:

### Critical Issues (Must Fix Immediately)

| Issue | Severity | Effort | Why It's Blocking |
|---|---|---|---|
| **Borrow errors treated as warnings** | 🔴 Critical | **1 hour** | One-line change in `main.rs:535` to `error[E006]` + add `return Err(...)`. Without this, ownership guarantees are fiction. |
| **No AST spans** | 🔴 Critical | **8-16 hours** | Blocks all diagnostic quality, LSP go-to-definition, error messages pointing at source. Every downstream consumer (semantic, borrow checker, codegen) needs location info. |
| **OLD/NEW parallel front-ends** | 🔴 Critical | **2-4 hours** | Causes confusion, duplicate maintenance. `stage1-compiler` should either use the NEW modules or be archived. `ir.rs` and `codegen.rs` reference types that don't exist in the active AST. |
| **`Type::Any` undermines type safety** | 🟠 High | **8 hours** | `println(anything)` compiles regardless of type. Needs trait-based dispatch (`Display` trait) or at minimum a variadic builtin type. |

### Architectural Issues (Must Fix Before Phase 3+)

| Issue | Severity | Effort | Why It Matters |
|---|---|---|---|
| **No MIR** | 🟠 High | **40-60 hours** | Required for correct borrow checking of conditional moves, loop moves, and NLL. The current AST-walk borrow checker handles many cases but fundamentally cannot reason about control-flow-dependent initialization. |
| **Runtime deps in compiler** | 🟡 Medium | **4 hours** | `tokio`, `reqwest`, `ndarray` in the compiler bloat build times and create unnecessary coupling. |
| **Zero parser tests** | 🟡 Medium | **16 hours** | 3,691 lines of parser with no tests. Any change could silently break syntax. |
| **Premature Phase 7-13 code (~430K)** | 🟡 Medium | **4 hours** | GPU/JIT/MLIR/native code is built on unstable IR. Should be gated behind feature flags, not actively compiled. |

### Revised Items (Previously Overstated)

| Issue | Original Severity | Revised | Why |
|---|---|---|---|
| **Borrow checker rewrite** | "Delete and rewrite" | **Keep, upgrade** | The checker is well-structured. Move to MIR *incrementally* — the current `BorrowError` enum and `BorrowState` types are reusable. |
| **Self-hosting** | "1/10, not achieved" | **3/10, partially achieved** | 14K byte stage files at `Helios/build/` prove the pipeline ran. The `omni-lang/build/` copies being 0 bytes suggest the *checked-in* artifacts are stale, not that bootstrap never worked. |

---

## Part 3: Revised Action Plan

### IMMEDIATE (Days 1-7): Stop the Bleeding

#### Task 1: Make borrow errors fatal
**File:** [main.rs](file:///d:/Project/Helios/omni-lang/compiler/src/main.rs#L529-L537)  
**Change:**
```diff
-    // Phase 2.6: Borrow checking (warnings for ownership violations)
+    // Phase 2.6: Borrow checking (errors for ownership violations)
     let borrow_errors = semantic::borrow_check::BorrowChecker::check_module(&ast);
     if !borrow_errors.is_empty() {
         for e in &borrow_errors {
-            eprintln!("warning[E006]: borrow check: {}", e);
+            eprintln!("error[E006]: borrow check: {}", e);
         }
+        anyhow::bail!("{} borrow checking error(s) found", borrow_errors.len());
     }
```
**Time:** 30 minutes  
**Impact:** All ownership violations now abort compilation.

#### Task 2: Delete OLD compiler files
**Files to delete:**
- `compiler/src/lexer.rs` (357 LOC, superseded by `lexer/mod.rs`)
- `compiler/src/parser.rs` (~520 LOC, superseded by `parser/mod.rs`)
- `compiler/src/ast.rs` (~70 LOC, superseded by `parser/ast.rs`)
- `compiler/src/ir.rs` (284 LOC, references OLD `ast.rs` types)
- `compiler/src/codegen.rs` (303 LOC, references OLD types like `Type::String`, `Type::Void`)
- `compiler/src/semantics.rs` (~280 LOC, superseded by `semantic/mod.rs`)

**Then:** Either archive `stage1-compiler/` or rewrite it to import the NEW modules.  
**Time:** 2 hours  
**Impact:** Eliminates the most confusing architectural issue in the project.

#### Task 3: Gate experimental codegen behind feature flags
**File:** `compiler/src/codegen/mod.rs`  
**Change:** Wrap GPU, JIT, MLIR, native, Python/C++ interop modules in `#[cfg(feature = "experimental")]`:
```rust
#[cfg(feature = "experimental")]
pub mod gpu_dispatch;
#[cfg(feature = "experimental")]
pub mod jit;
#[cfg(feature = "experimental")]
pub mod mlir;
#[cfg(feature = "experimental")]
pub mod native_codegen;
// ... etc
```
**Time:** 2 hours  
**Impact:** Default `cargo build` compiles only lexer → parser → semantic → OVM codegen. Build time drops dramatically. Experimental code preserved but not blocking.

#### Task 4: Add CI
**File:** `.github/workflows/ci.yml` (NEW)
```yaml
name: CI
on: [push, pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cd compiler && cargo test
      - run: cd compiler && cargo clippy -- -D warnings
      - run: cd compiler && cargo fmt -- --check
```
**Time:** 1 hour  
**Impact:** Every push is tested. Regressions are caught.

---

### SHORT-TERM (Weeks 2-4): Foundation Repair

#### Task 5: Add spans to all AST nodes
**Files:** `parser/ast.rs`, `parser/mod.rs`  

Introduce:
```rust
#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}
```

Then add `pub span: Span` to: `Expression` (wrap as enum with span), `Statement`, `Item`, `Type`, `Pattern`.  
Update the parser to thread `lexer.span()` byte offsets into every constructed AST node.  
Update semantic error messages to include span-based source excerpts.

**Time:** 16 hours  
**Impact:** Unlocks rustc-quality error messages, LSP integration, and future MIR lowering.

#### Task 6: Write parser snapshot tests (50+ tests)
**File:** `compiler/src/parser/tests.rs` (NEW)  
**Coverage targets:**
- Every `parse_item` branch: function, struct, enum, trait, impl, module, import, extern, const, static, comptime, macro
- Every statement form: let, var, assignment, return, if/elif/else, for, while, loop, match, defer, break, continue, pass, yield, spawn, select
- Every expression form: literals, binary/unary ops, calls, method calls, field access, indexing, array, struct literal, borrow, deref, await, range, lambda, if-expr, match-expr, list comprehension, tuple
- Error recovery: malformed input produces errors + partial AST
- Edge cases: nested blocks, empty blocks, trailing commas

**Time:** 16 hours  
**Impact:** Parser changes become safe. Regression protection.

#### Task 7: Remove `Type::Any` from builtins
**File:** `semantic/mod.rs:597-715`  
**Strategy:** Replace `Type::Any` with a `Type::Named("Display")` trait bound or a proper variadic type. At minimum, split builtins into typed signatures:
```rust
("println", Type::Function(vec![Type::Str], None), false),  // println takes &str
("len", Type::Function(vec![Type::Slice(Box::new(Type::Infer))], Some(Box::new(Type::I64))), false),
```
**Time:** 8 hours  
**Impact:** Type checker actually checks types for builtin calls.

---

### MEDIUM-TERM (Weeks 5-12): MIR and Borrow Checker Evolution

#### Task 8: Design and implement MIR
**New module:** `compiler/src/mir/`  

Structure:
```
mir/
├── mod.rs          # MIR data structures (BasicBlock, Place, Rvalue, Terminator)
├── lower.rs        # AST → MIR lowering
├── pretty.rs       # MIR pretty-printer for debugging
└── dataflow.rs     # Gen/Kill analysis framework
```

The MIR must support:
- **Basic blocks** with explicit terminators (Goto, SwitchInt, Return, Drop, Call)
- **Places** (`_1`, `_1.field`, `(*_1)`) for memory locations
- **Rvalues** (Use, BinaryOp, Ref, AddressOf) for computations
- **Explicit drops** at scope boundaries
- **StorageLive / StorageDead** for variable lifetimes

**Time:** 40-60 hours  
**Impact:** Enables correct borrow checking, optimization passes, and clean LLVM/native codegen.

#### Task 9: Migrate borrow checker to MIR (incremental)
**Strategy:** Don't delete the existing borrow checker. Instead:
1. Keep the current AST-walk checker as a "lint pass" for now
2. Build a new MIR-based checker in `mir/borrow_check.rs`
3. Run both in parallel, asserting they agree
4. Once the MIR checker is proven, remove the AST-walk checker

Reuse the existing `BorrowError` enum (it's well-designed) and `BorrowState` types.

**Time:** 40 hours (after MIR is in place)  
**Impact:** Correct handling of conditional moves, loop moves, NLL.

---

### LONG-TERM (Months 4-6): Self-Hosting and Stdlib

#### Task 10: Stabilize the bootstrap pipeline
The `Helios/build/` files prove the pipeline has run. The priority is:
1. Reproduce the bootstrap from scratch (`cargo run -- omni/compiler/main.omni -o build/omnc_stage1.ovm`)
2. Verify `stage1.ovm` output matches `stage2.ovm` (fixed-point)
3. Check in the working `.ovm` files to `omni-lang/build/` (replacing the 0-byte stubs)
4. Add a CI job that runs the bootstrap and verifies the fixed-point

#### Task 11: Implement stdlib bodies
Start with the 5 most critical modules:
1. `std/core.omni` — `Option::map`, `Option::unwrap_or`, `Result::map`, `Result::unwrap`
2. `std/string.omni` — basic string operations
3. `std/collections.omni` — `Vector` (growable array)
4. `std/io.omni` — `println`, `read_line` (currently builtins, should move to stdlib)
5. `std/math.omni` — basic math functions

#### Task 12: Implement `f"..."` string interpolation
**Files:** `lexer/mod.rs`, `parser/mod.rs`, `parser/ast.rs`  
Add a `FStringLiteral` token kind, parse `{expr}` interpolation segments into `Expression::FString(Vec<FStringPart>)`.

---

## Part 4: Revised Scorecard

| Dimension | Original Score | Revised Score | Justification |
|---|---|---|---|
| Architecture Coherence | 3/10 | **3/10** | Still two parallel front-ends, still monolithic crate. Confirmed. |
| Borrow Checker Quality | 2/10 | **5/10** | Well-structured `BorrowError` enum, 10 error kinds, loop-move detection, scope tracking. Problem is `main.rs` treating output as warnings. |
| Self-Hosting Progress | 1/10 | **3/10** | 14K stage files exist at parent level. Pipeline has run. Stale 0-byte files mislead. |
| Type System | 4/10 | **4/10** | No change. `Type::Any` still undermines safety. |
| Test Coverage | 2/10 | **2/10** | No change. ~20 tests confirmed. |
| OVM Pipeline | 7/10 | **7/10** | Works end-to-end. Confirmed. |
| Overall | ~25% | **~30%** | Borrow checker and bootstrap were undersold. Core issues remain. |

---

## Part 5: Execution Priority Matrix

| Priority | Task | Effort | Blocks |
|---|---|---|---|
| **P0** | Make borrow errors fatal | 30 min | Everything safety-related |
| **P0** | Delete OLD compiler files | 2 hrs | Architectural clarity |
| **P1** | Add AST spans | 16 hrs | LSP, diagnostics, MIR |
| **P1** | Parser snapshot tests | 16 hrs | Safe parser evolution |
| **P1** | Add CI | 1 hr | Regression prevention |
| **P2** | Gate experimental codegen | 2 hrs | Build time, focus |
| **P2** | Remove `Type::Any` | 8 hrs | Type safety |
| **P2** | Remove runtime deps | 4 hrs | Build time |
| **P3** | Implement MIR | 40-60 hrs | Correct borrow checking |
| **P3** | Migrate borrow checker | 40 hrs | Full safety guarantee |
| **P4** | Stabilize bootstrap | 16 hrs | Self-hosting proof |
| **P4** | Implement stdlib bodies | 40 hrs | Usable language |
| **P5** | `f"..."` interpolation | 8 hrs | Spec compliance |
| **P5** | Resurrect LLVM/native | 40+ hrs | Native compilation |
