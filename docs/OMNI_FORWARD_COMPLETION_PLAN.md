# OMNI PROJECT — FORWARD COMPLETION PLAN
## Exhaustive audit synthesis and how to proceed

**Date:** April 13, 2026  
**Compared against:** `docs/Omni_Complete_Specification.md`  
**Baseline used:** `docs/EXHAUSTIVE_AUDIT_REPORT.md`, `omni-lang/docs/IMPLEMENTATION_STATUS.md`, `PHASE_PLAN.md`, `omni-lang/COMPLETION_TODO.md`, and current repository structure

---

## 1. Purpose

This file is a **new planning artifact**. It does **not** replace `docs/EXHAUSTIVE_AUDIT_REPORT.md`.

Its purpose is to:
1. Synthesize the older audit against the current specification and visible project status files.
2. Identify what is likely still true, what is likely stale, and what must be re-verified.
3. Define the correct order to finish the Omni project without further architectural drift.

---

## 2. Executive Summary

Omni is **materially real**, not aspirational. The repository appears to contain a functioning Rust bootstrap compiler with:
- lexer,
- parser,
- semantic pipeline,
- MIR/IR work,
- OVM-oriented code generation,
- example execution,
- significant automated test coverage.

However, compared to the v2.0 specification, Omni is **not yet complete in the layers that determine real specification compliance**:

1. The **effect system** is not complete at v2.0 scope.
2. The **module/package/build system** is not complete enough for full language maturity.
3. The **ownership/borrow/linearity story** needs stronger validation and coverage.
4. The **stdlib is substantial but not specification-complete**.
5. The **tooling layer exists but is ahead of verified compiler maturity**.
6. **Self-hosting progress exists**, but should not yet be treated as final proof.
7. **HELIOS is still premature** relative to unfinished Omni foundations.

### Core conclusion

The project should not proceed by trying to “finish everything” in parallel.

The correct strategy is:

1. Freeze architectural drift.
2. Re-verify the compiler baseline.
3. Complete missing core layers in dependency order.
4. Harden package/build/stdlib/tooling integration.
5. Resume self-hosting and platform maturity only after core truth is stable.
6. Treat HELIOS as a downstream consumer, not as evidence of Omni maturity.

---

## 3. What from the previous audit still looks directionally correct

The old audit still appears broadly correct in these ways:

- The compiler core is real and materially functional.
- The project is far beyond a toy language implementation.
- The workspace/crate structure is substantial.
- OVM-based execution exists.
- The stdlib is already significant.
- Self-hosting code exists but is not yet conclusive proof of bootstrap maturity.
- Package/build work exists but is partial.
- Tooling exists but is not fully aligned with final language maturity.
- HELIOS is present too early relative to the state of Omni core.
- The main problem is not lack of code; it is **misordered investment and incomplete integration**.

---

## 4. What likely needs re-verification because the old audit may be stale

The following areas should be treated as “needs current verification” rather than final truth:

### 4.1 Borrow checker and field projection claims
Different documents imply different levels of completion for:
- field projections,
- Polonius precision,
- linear enforcement,
- borrow regression coverage.

This likely means:
- some implementation exists,
- but full specification-grade validation is still missing.

### 4.2 Effect system maturity
There are signs of:
- effect syntax,
- effect types,
- validation passes,
- async/effect infrastructure.

But the v2.0 specification requires much more:
- user-defined effects,
- handlers,
- effect polymorphism,
- structured integration with async/cancellation/generators.

This area is likely easy to overstate.

### 4.3 Package/build maturity
The specification requires a serious package/build model:
- manifests,
- dependency resolution,
- lockfiles,
- build scripts,
- workspace behavior,
- reproducibility.

Current evidence suggests this is still incomplete.

### 4.4 Self-hosting status
One status document suggests strong bootstrap progress, while the older audit is more conservative. The likely truth is:
- self-hosting artifacts and experiments are meaningful,
- some fixpoint/bootstrap gates may work,
- but specification-grade self-hosting trust is not yet complete.

### 4.5 Tooling status
There is a difference between:
- a command existing,
- a command working,
- a command being spec-grade.

This especially affects:
- `omni fix`,
- `omni doc`,
- LSP claims,
- diagnostics JSON,
- machine-applicable fixes.

---

## 5. Synthesis by major domain

| Domain | Current best reading | Planning status |
|---|---|---|
| Foundation / governance | Strong | Good enough |
| Lexer/parser core | Strong | Keep and harden |
| Semantic core | Real but incomplete at v2.0 depth | Continue |
| Ownership / borrow safety | Partially strong, incompletely proven | High priority |
| Modules / packages / build | Partial | Critical gap |
| Stdlib | Substantial but incomplete | High priority |
| Tooling | Partial | Secondary after core/package |
| Effects | Partial and under-complete | Critical gap |
| Runtime / concurrency | Partial | Later core milestone |
| Security / capabilities | Partial | Medium priority |
| Interop | Partial | Defer until core stable |
| Self-hosting | Meaningful but not final | Important, but not first |
| HELIOS | Premature | Freeze as downstream consumer |

---

## 6. Real state by implementation phase

### Phase 0 — Foundation
Appears sufficiently complete:
- workspace,
- contributor structure,
- repository organization,
- build/test capability.

**Plan:** no redesign needed; maintain.

### Phase 1 — Language core skeleton
Appears materially complete:
- lexer,
- parser,
- AST,
- diagnostics baseline.

**Plan:** harden rather than rebuild.

### Phase 2 — Semantic core
Appears materially real:
- resolution,
- type inference,
- checking.

**Plan:** keep extending; do not re-architect unless forced by evidence.

### Phase 3 — Ownership / borrowing / safety
Meaningful implementation exists, but this phase still needs stronger proof:
- precision,
- coverage,
- field sensitivity,
- linearity,
- diagnostics.

**Plan:** make this one of the first finish targets.

### Phase 4 — Modules / packages / build
This is likely the most underestimated missing layer.

Without it, Omni cannot honestly claim:
- scalable multi-file development,
- reproducible workspaces,
- serious package semantics,
- reliable build-time integration.

**Plan:** elevate this to top-tier priority.

### Phase 5 — Stdlib
Substantial but not finished:
- core/collections/string/io/tensor work exists,
- alloc split and platform-sensitive areas remain incomplete,
- cfg-sensitive support appears unfinished.

**Plan:** complete alongside package/build/compiler support.

### Phase 6+ — Tooling, advanced types, effects, runtime, security, interop, self-hosting, maturity
These phases contain real work, but are not yet completion-grade relative to the specification.

**Plan:** advance only in the correct dependency order.

---

## 7. Core strategic problems blocking completion

## 7.1 Diluted focus
The repository mixes:
- core compiler work,
- runtime experiments,
- multiple backend ambitions,
- self-hosting,
- tooling,
- HELIOS platform work.

**Effect:** progress looks larger than verified completion really is.

## 7.2 “Exists” is being mistaken for “implemented”
A feature is not complete just because:
- syntax exists,
- a file exists,
- a module exists,
- a type exists,
- a command exists.

Completion must mean:
1. parser support,
2. semantic enforcement,
3. lowering/IR support if needed,
4. runtime/codegen support if needed,
5. diagnostics,
6. regression tests.

## 7.3 Phase 4 is underweighted
Package/build maturity is a structural dependency for:
- stdlib maturity,
- tooling maturity,
- package security,
- self-hosting trust,
- real project usability.

## 7.4 Effect claims exceed current proof
The v2.0 specification makes effects central to Omni’s identity, but current evidence suggests only partial fulfillment.

## 7.5 HELIOS is too early
HELIOS is useful as:
- a design target,
- a stress consumer,
- a future product layer.

It is not yet useful as proof that Omni core is mature.

---

## 8. Recommended completion order

## Stage A — Freeze scope and define truth rules
**Goal:** stop further drift.

Actions:
1. Freeze major HELIOS/platform expansion.
2. Freeze speculative backend expansion unless on the critical path.
3. Adopt one status taxonomy:
   - Implemented
   - Partially Implemented
   - Not Implemented
   - Blocked
   - Premature
   - Experimental
4. Declare one rule: nothing is “implemented” without end-to-end verification.

**Exit criteria:**
- contributors share one definition of completion,
- critical path is explicit,
- non-critical work is gated.

---

## Stage B — Re-verify the compiler baseline
**Goal:** make current compiler truth indisputable.

Actions:
1. Re-verify:
   - workspace build,
   - compiler build,
   - parser tests,
   - semantic tests,
   - borrow tests,
   - example compilation/execution,
   - stage1/bootstrap checks.
2. Produce one fresh verification matrix for:
   - lexer,
   - parser,
   - AST,
   - resolver,
   - inference,
   - MIR,
   - codegen,
   - runtime.

**Exit criteria:**
- one trusted current-state baseline exists,
- contradictory status claims are resolved.

---

## Stage C — Finish ownership / borrow / linearity
**Goal:** make Omni’s safety story credible at specification level.

Priority work:
1. Field projection correctness and regression coverage.
2. Polonius precision validation across loops/control flow/complex borrows.
3. Linear type enforcement end-to-end.
4. Clarify generational reference expectations vs. borrow-check integration.
5. Improve ownership diagnostics.

**Exit criteria:**
- broad borrow regression suite,
- field-sensitive behavior verified,
- linear types enforced, not merely parsed,
- ownership diagnostics are trustworthy.

---

## Stage D — Finish modules / packages / build system
**Goal:** make Omni scalable and reproducible.

Priority work:
1. Cross-file import resolution hardening.
2. Package graph semantics.
3. Lockfile fidelity.
4. Transitive dependency correctness.
5. Build script integration.
6. Manifest capability declarations.
7. Deterministic workspace behavior.

**Exit criteria:**
- multi-package sample compiles,
- imports and visibility are reliable,
- lockfile behavior is stable,
- build scripts affect builds in controlled/tested ways.

---

## Stage E — Finish stdlib core
**Goal:** turn the stdlib from substantial into specification-reliable.

Priority work:
1. Separate `std::alloc` properly.
2. Harden `std::io`.
3. Align std APIs with effect/capability expectations.
4. Validate `Option`, `Result`, and `Try` ergonomics.
5. Fill missing auxiliary modules.
6. Add platform/cfg-sensitive tests.
7. Decide explicit milestone treatment for `std::tensor` and `std::simd`.

**Exit criteria:**
- core stdlib modules compile and test cleanly,
- cfg/platform-sensitive behavior is sane,
- docs and implementation align.

---

## Stage F — Complete the minimum viable v2.0 effect system
**Goal:** satisfy the most identity-critical v2.0 promise.

Priority work:
1. User-defined effects.
2. Effect handlers.
3. Effect polymorphism.
4. Stronger async/effect interaction model.
5. Better effect diagnostics.
6. Explicitly defer generators/async drop if necessary under named milestones instead of implicit incompleteness.

**Exit criteria:**
- user effects work end-to-end,
- handlers work end-to-end,
- effect-polymorphic examples compile,
- diagnostics clearly explain effect failures.

---

## Stage G — Tooling alignment
**Goal:** make the developer experience honestly match the language core.

Priority work:
1. `omni fmt` maturity and idempotence proof.
2. LSP/compiler integration.
3. `omni doc` credibility.
4. `omni fix` only after diagnostic/fix infrastructure is actually real.
5. Bring CLI commands into a single verified matrix.

**Exit criteria:**
- formatter stable,
- go-to-definition/hover work on real samples,
- docs output is useful,
- fix system exists only if truly machine-applicable.

---

## Stage H — Security, interop, and runtime hardening
**Goal:** mature outer layers once compiler/package/core semantics are stable.

Priority work:
1. Capability/effect alignment.
2. Package verification/signing roadmap.
3. FFI sandbox hardening.
4. WebAssembly/Python/C interop stabilization.
5. Runtime determinism and structured concurrency enforcement.

**Exit criteria:**
- language-level promises and runtime/tooling promises align,
- interop surfaces are real and testable.

---

## Stage I — Self-hosting proof
**Goal:** complete self-hosting on top of a credible language core.

Priority work:
1. Reduce syntax/semantic mismatches between stage0 and self-hosted compiler.
2. Verify stage transitions and fixpoints.
3. Add reproducibility and output-diff checks.
4. Ensure the self-hosted compiler reflects actual language semantics, not a divergent subset.

**Exit criteria:**
- stage pipeline is reproducible,
- fixpoint verification is automated,
- self-hosting is honest and repeatable.

---

## Stage J — HELIOS resumption and Phase 13 maturity
**Goal:** only after Omni is truly ready.

Priority work:
1. Resume HELIOS expansion only once Omni phases 3–8 are credibly mature.
2. Gate MLIR/tensor/platform work behind explicit acceptance criteria.
3. Treat HELIOS as a proof consumer of Omni, not a replacement for Omni maturity.

---

## 9. Immediate next 90-day plan

## Days 1–15
1. Re-baseline actual current compiler truth.
2. Resolve contradictions between existing status documents.
3. Freeze non-critical platform drift.
4. Establish one completion taxonomy.

## Days 16–35
1. Finish borrow-check/field-projection/linearity hardening.
2. Expand regression coverage for ownership and safety.
3. Improve ownership diagnostics where weak.

## Days 36–60
1. Finish modules/package/build essentials.
2. Prove multi-file/multi-package correctness.
3. Stabilize lockfile/build-script behavior.

## Days 61–90
1. Finish stdlib core gaps.
2. Advance the minimum viable full effect system.
3. Align tooling to newly hardened compiler/package reality.

---

## 10. Practical milestone ladder

### Milestone 1 — Trusted baseline
- all core compiler tests re-verified,
- one unified truth matrix exists.

### Milestone 2 — Safety credibility
- borrow checker precision and linearity hardened,
- ownership diagnostics strong.

### Milestone 3 — Package credibility
- multi-file, multi-package builds work reproducibly.

### Milestone 4 — Stdlib credibility
- core stdlib behaves consistently across targeted environments.

### Milestone 5 — Effect credibility
- user-defined effects and handlers work end-to-end.

### Milestone 6 — Tooling credibility
- formatter, LSP, docs, and CLI are aligned with actual language behavior.

### Milestone 7 — Self-hosting credibility
- reproducible, automated, honest bootstrap proof.

### Milestone 8 — Platform credibility
- HELIOS and advanced platform layers resume on top of a stable Omni foundation.

---

## 11. What should explicitly be deprioritized for now

Until Stages A–F are complete or nearly complete, the following should be deprioritized:

- major HELIOS feature expansion,
- speculative MLIR/GPU/platform work,
- broad interoperability expansion beyond what is needed for core validation,
- tooling polish disconnected from compiler truth,
- any feature whose primary evidence is “there is a file/stub/module.”

---

## 12. Final recommendation

The project should move forward under a single principle:

> **Finish the truth-bearing layers first: safety, packages, stdlib, effects, then tooling, then bootstrap proof, then platform maturity.**

If Omni follows that order, the existing body of work becomes a major advantage.

If Omni continues mixing:
- foundation work,
- speculative platform work,
- premature HELIOS expansion,
- and unverified completion claims,

then even strong existing implementation work will continue to underperform its potential.

The correct path is not to start over.  
The correct path is to **re-focus, re-verify, and complete in dependency order**.
