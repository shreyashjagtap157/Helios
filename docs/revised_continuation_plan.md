# REVISED CONTINUATION PLAN
## Post-Audit Cross-Check: Action Plan vs Reality

**Date:** April 11, 2026  
**Based on:** `docs/action_plan.md` and `docs/audit_verification_and_revised_plan.md`

---

## EXECUTIVE SUMMARY

After cross-checking the revised action plan against the current codebase, many P0/P1/P2 tasks have been **completed** in previous sessions. The project is in better shape than the original audit suggested.

**Key Finding:** The revised action plan from the verification report is ~70% already implemented or moot.

---

## TASK COMPLETION STATUS

### ✅ COMPLETED (Previously Done)

| Task | Original Priority | Status | Evidence |
|------|-----------------|--------|----------|
| Make borrow errors fatal | P0 | ✅ DONE | `main.rs:503` - `error[E006]` + `anyhow::bail!` |
| Delete OLD compiler files | P0 | ✅ DONE | 6 files deleted (lexer.rs, parser.rs, etc.) |
| Gate experimental codegen | P1 | ✅ DONE | `codegen/mod.rs:26-62` - 18 modules behind `#[cfg(feature = "experimental")]` |
| Add CI | P1 | ✅ DONE | `.github/workflows/ci.yml` exists |
| Add AST spans | P1 | ✅ DONE | `parser/ast.rs:19-24` - Span struct |
| Add parser tests | P1 | ✅ DONE | 66+ tests in `parser/mod.rs` (418 pass, 10 fail) |
| Remove runtime deps | P2 | ✅ DONE | `Cargo.toml` cleaned |
| Remove #![allow(dead_code)] | P2 | ✅ DONE | Removed from 30+ files |
| Implement MIR | P3 | ✅ DONE | `mir/mod.rs`, `place.rs`, `rvalue.rs`, `statement.rs`, `pretty.rs`, `lower.rs` |
| f-string interpolation | P5 | ✅ DONE | FStringLiteral token + AST types |

### ⚠️ PARTIALLY COMPLETE

| Task | Priority | Status | Notes |
|------|----------|--------|-------|
| Remove Type::Any | P2 | ⚠️ MOSTLY DONE | Removed from builtin signatures, comment confirms "Removed Type::Any" |
| Stabilize bootstrap | P4 | ⚠️ VERIFIED | `build/bootstrap_status.env` confirms stage1=stage2=14366 bytes, identical hash |

### ❌ NOT COMPLETED (Still Pending)

| Task | Priority | Effort | Why Not Done |
|------|----------|--------|--------------|
| Migrate borrow checker to MIR | P3 | 40 hrs | MIR created but borrow checker still on AST |
| Implement stdlib bodies | P4 | 40 hrs | Still stubs in `core.omni` |
| Implement effect system integration | P4 | High | Types defined but not wired into compilation |
| LSP Server | P5 | High | Not started |
| Formatter | P5 | Medium | Not started |

---

## WHAT WAS ACCOMPLISHED IN PRIOR SESSIONS

Based on the session context, here's what was done:

1. **Borrow errors → fatal** - Changed from warnings to errors in `main.rs:495-507`
2. **Deleted 6 OLD compiler files** - Removed legacy lexer.rs, parser.rs, ast.rs, ir.rs, codegen.rs, semantics.rs
3. **Gated experimental codegen** - Behind `#[cfg(feature = "experimental")]`
4. **CI exists** - `.github/workflows/ci.yml` verified
5. **Span type** - Exists in `parser/ast.rs:19-24`
6. **Parser tests added** - 66+ tests (418 pass, 10 fail)
7. **Runtime deps removed** - From Cargo.toml
8. **dead_code removed** - From 30 files
9. **MIR module created** - 6 files: mod.rs, place.rs, rvalue.rs, statement.rs, pretty.rs, lower.rs
10. **Bootstrap verified** - Stage 0 produces 14,366 bytes
11. **f-string implemented** - FStringLiteral token + AST types

---

## REMAINING PRIORITY WORK

### P3: MIR-Based Borrow Checker (Still Needed)

The original plan was to migrate the borrow checker to MIR. While MIR exists, the borrow checker still operates on AST.

**Why it matters:**  
- Current NLL approach works for many cases but can't handle:
  - Conditional moves (assigned in `if` but not `else`)
  - Loop moves (value moved inside loop, used after)
  - Complex control flow paths

**Recommendation:** Keep current AST-based checker as "lint" and add parallel MIR-based checking. This was the original incremental plan.

### P4: Stdlib Bodies (High Impact)

Current state: `core.omni` has only stubs (placeholder implementations)

**Priority implementations:**
1. `std/core.omni` - Option::map, unwrap_or, Result::map, unwrap
2. `std/collections.omni` - Vector<T>
3. `std/string.omni` - String operations
4. `std/io.omni` - println, read_line

### P4: Effect System Integration (v2.0 Core)

Current state: Effect types defined in `semantic/effects.rs` but not integrated

**What's needed:**
- Parse `/ effect` syntax in function signatures
- Thread effect rows through type inference
- Implement effect inference
- Add effect handlers

### P5: Tooling (LSP, Formatter)

Not started. Requires:
- CST (lossless syntax tree) for safe manipulation
- LSP server implementation
- Formatter implementation

---

## REVISED ROADMAP

### Continue With (Based on Current Priorities)

| Phase | Task | Effort | Prerequisites |
|-------|------|--------|---------------|
| **P3** | Integrate effect system into compilation | 40 hrs | Parser effect annotation support |
| **P3** | Add effect inference to type checker | 20 hrs | Effect types wired up |
| **P4** | Implement stdlib Option/Result bodies | 16 hrs | None |
| **P4** | Implement Vector<T> in collections | 24 hrs | None |
| **P3** | Add Polonius borrow checking (or document NLL adequacy) | 40 hrs | MIR exists |

### Deprioritize (Not Blocking Core)

| Task | Why |
|------|-----|
| LSP Server | Need CST first |
| Formatter | Need CST first |
| GPU/JIT backends | Still gated behind experimental |
| Native x86 codegen | OVM is sufficient for now |

---

## SPEC COMPLIANCE GAP

The v2.0 specification requires:

| Spec Requirement | Current Status | Gap |
|------------------|----------------|-----|
| Effect system | Partial | Not integrated into compilation |
| Polonius borrow checker | NLL | Wrong algorithm per spec |
| Linear types | None | Not implemented |
| Structured concurrency | None | Not implemented |
| Full stdlib | Stubs | No actual implementations |

---

## RECOMMENDED CONTINUATION

**Next immediate priorities:**

1. **Effect system integration** - This is v2.0's flagship feature. Wire `semantic/effects.rs` into the compilation pipeline.

2. **Fix remaining 10 parser test failures** - These represent syntax gaps that should be closed.

3. **Complete stdlib Option/Result** - Implement actual body logic, not just stubs.

4. **Decide on borrow checker** - Either implement Polonius per spec, or document why NLL is sufficient for Omni's model.

**These align with:**
- `action_plan.md` Phase 4 (Effect System)
- `audit_verification_and_revised_plan.md` P4 items
- The v2.0 specification core features

---

## FILES CREATED THIS SESSION

- `docs/comprehensive_audit_v2.md` - Full audit report
- `docs/revised_continuation_plan.md` - This file

---

**Conclusion:** The project has made significant progress. The P0/P1 tasks from the revised action plan are largely complete. The remaining work focuses on:
1. Integrating the already-defined effect system
2. Completing stdlib implementations
3. Deciding on borrow checker algorithm (Polonius vs NLL)

---

## CONCRETE EXECUTION SLICE

### 1. Frontend Fidelity Closure
Target files:
- `compiler/src/lexer/mod.rs`
- `compiler/src/parser/mod.rs`
- `compiler/src/parser/ast.rs`
- `compiler/src/diagnostics.rs`

Tasks:
- Keep INDENT/DEDENT synthesis stable and add any missing token kinds required by the v2.0 surface.
- Parse effect annotations, `inout` parameters, `linear` markers, async closures, `let`-chains, deconstructing parameters, and interpolated string forms.
- Preserve source spans on every node needed for diagnostics and tooling.
- Expand parser regression coverage until new syntax is gated by tests, not by comments.

Acceptance gates:
- Parser round-trips the current bootstrap corpus.
- Syntax errors produce stable codes, spans, and actionable messages.
- The parser test suite covers the new syntax forms with no regressions.

### 2. Semantic Core Closure
Target files:
- `compiler/src/resolver.rs`
- `compiler/src/semantic/mod.rs`
- `compiler/src/semantic/type_inference.rs`
- `compiler/src/semantic/traits.rs`
- `compiler/src/semantic/effects.rs`

Tasks:
- Complete two-pass name resolution with explicit definition identities and visibility checks.
- Keep bidirectional inference as the default semantic path and propagate effect rows through non-public code.
- Finalize trait-bound checking, implied bounds, and custom diagnostic hooks.
- Make effect inference and effect annotation checking part of the normal semantic pipeline.

Acceptance gates:
- Sample programs type-check with predictable spans and error messages.
- Effect-aware code either infers or rejects correctly.
- Trait and visibility failures produce stable diagnostics instead of generic aborts.

### 3. MIR And Memory Model Closure
Target files:
- `compiler/src/mir/mod.rs`
- `compiler/src/mir/lower.rs`
- `compiler/src/semantic/borrow_check.rs`
- `compiler/src/semantic/polonius.rs`
- `compiler/src/semantic/generational.rs`
- `compiler/src/semantic/linear.rs`
- `compiler/src/memory/arena.rs`

Tasks:
- Make MIR the control-flow and ownership layer between semantics and codegen.
- Route borrow checking through the Polonius-based path and keep the AST checker only as a temporary comparison aid.
- Add field projections, generational references, linear usage enforcement, and arena allocation as first-class semantics.
- Ensure `inout` desugars to MIR-level move-in/move-out without hidden runtime cost.

Acceptance gates:
- Use-after-move, conflicting borrows, field-level borrows, and linear-resource failures are rejected deterministically.
- MIR lowering emits explicit control flow and drop points.
- Existing accepted programs remain accepted after the borrow-checker migration.

### 4. Later Gated Work
Only after the above slice is stable:
- Complete stdlib bodies and package/build tooling.
- Finish runtime/security, capability enforcement, and sandboxing.
- Resume self-hosting migration with dual-compiler verification.
- Add platform maturity work such as editioning and MLIR/GPU support.