# Stage F Effect Progress
## Async effect inference and phase-3 baseline alignment slice

**Date:** April 13, 2026  
**Plan source:** `docs/OMNI_FORWARD_COMPLETION_PLAN.md`  
**Scope:** Stage F follow-up after the Stage B baseline re-verification and Stage C/D hardening work

---

## Completed in this slice

### 1. Async callables now validate with an implicit `Async` effect

The effect validator previously treated async functions and async lambdas as invalid unless their declared effect row already explicitly contained `Async`.

That behavior did not match the current compiler pipeline, where effects are already inferred for unannotated code.

This slice aligns the validator with that inference model:

- async callables implicitly carry `Async` during effect validation,
- non-async callables still reject an `Async` effect row,
- non-async bodies that perform async work still error correctly,
- regression coverage now verifies that an async callable with an otherwise pure declared row is accepted.

### 2. Phase-3 async baseline now compiles end to end again

The `phase3_async_await_basic.omni` gate had regressed for two separate reasons:

- the lightweight Stage 1 parser treated `int` as an unknown custom type instead of the legacy integer alias used by that sample,
- the main compiler rejected async callables without an explicitly declared `Async` effect.

Both gaps are now closed for the verified baseline path:

- Stage 1 accepts the sample again,
- `omnc` accepts the sample again,
- the portable baseline is back to green with current binaries.

---

## Verification

The following checks passed after the effect/alignment implementation:

```bash
cd omni-lang && cargo test -p omnc --all-targets --quiet
cd omni-lang && cargo build -p omnc --release
cd omni-lang && cargo build -p omni_stage1 --release
cd omni-lang && cargo test -p omni_stage1 --all-targets --quiet
cd omni-lang && cargo test -p omni-fmt -p omni-lsp -p omni-dap -p opm -p ovm-runner --all-targets --quiet
cd omni-lang && cargo run --quiet --bin omnc -- examples/phase3_async_await_basic.omni -o $TEMP/phase3_async_test.ovm
```

Portable baseline status remains **GREEN**.

---

## What Stage F still does not prove

This slice materially improves Stage F alignment, but it does **not** complete the Stage F exit criteria from the forward plan.

Still remaining:

- user-defined effects,
- effect handlers,
- effect polymorphism beyond the current validation scaffolding,
- deeper async/cancellation integration,
- broader diagnostics for effect failures across more real programs.
