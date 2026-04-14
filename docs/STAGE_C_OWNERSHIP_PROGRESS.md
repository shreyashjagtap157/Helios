# Stage C Ownership Progress
## Borrow and linearity hardening slice

**Date:** April 13, 2026  
**Plan source:** `docs/OMNI_FORWARD_COMPLETION_PLAN.md`  
**Scope:** Stage C follow-up after the portable Stage B baseline re-verification

---

## Completed in this slice

### 1. Linear parameter enforcement is now semantic, not just syntactic

The parser already supported `linear` function parameters, but the semantic pipeline did not enforce any linearity rules.

That gap is now closed for function parameters:

- linear parameters must be consumed exactly once,
- borrowing a linear parameter is rejected,
- consuming a linear parameter more than once is rejected,
- capturing a linear parameter in a lambda is rejected,
- regression tests cover these cases.

### 2. Field-projection borrows are now exercised by the borrow checker

The legacy borrow checker already had field-borrow data structures, but `&obj.field` and `&mut obj.field` were not actually wired into the AST walk.

That is now implemented for the verified borrow-checking path:

- field borrows are recorded distinctly from whole-value borrows,
- independent borrows of different fields are allowed,
- conflicting borrows of the same field are rejected,
- moving a value while one of its fields is borrowed is rejected,
- regression tests cover these cases.

---

## Verification

The following checks passed after the Stage C implementation:

```bash
cd omni-lang && cargo test -p omnc --all-targets --quiet
cd omni-lang && cargo test -p omni_stage1 --all-targets --quiet
cd omni-lang && cargo test -p omni-fmt -p omni-lsp -p omni-dap -p opm -p ovm-runner --all-targets --quiet
```

Portable baseline status remains **GREEN**.

Known follow-up still present:

- stage1 effect mismatch warnings for inferred `IO`
- stage1 effect mismatch warning for inferred `Async`

These remain aligned with the forward plan’s later Stage F effect-system work.

---

## What Stage C still does not prove

This slice materially improves Stage C, but does **not** yet complete Stage C exit criteria by itself.

Still remaining:

- stronger Polonius precision validation across richer control flow,
- broader ownership diagnostics,
- fuller integration between the typed semantic pipeline and the stronger ownership rules,
- wider regression coverage for field-sensitive and linear behavior beyond the cases implemented here.
