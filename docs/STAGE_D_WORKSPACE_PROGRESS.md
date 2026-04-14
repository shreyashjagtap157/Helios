# Stage D Workspace Progress
## Workspace member-selection hardening slice

**Date:** April 13, 2026  
**Plan source:** `docs/OMNI_FORWARD_COMPLETION_PLAN.md`  
**Scope:** Stage D follow-up after the package graph / lockfile slice

---

## Completed in this slice

### 1. Workspace discovery now respects `exclude` and `default-members`

The workspace loader in `opm` previously collected members without a meaningful distinction between:

- all discovered workspace members,
- excluded paths,
- and the subset that should be treated as default members.

That behavior is now hardened for the currently supported local workspace flow:

- explicit workspace members are expanded deterministically,
- simple `/*` member patterns are expanded into matching child packages,
- `exclude` entries are removed from the discovered workspace set,
- `default-members` are tracked separately from the full member list,
- workspace paths are sorted and deduplicated for stable behavior.

### 2. Missing explicit workspace members now fail early

Before this slice, a missing workspace member path could be silently ignored depending on how the workspace was discovered.

That gap is now closed:

- explicitly listed members must exist,
- missing explicit members produce an error instead of silently shrinking the workspace,
- regression tests cover both the success and failure paths.

---

## Verification

The following checks passed after the workspace discovery implementation:

```bash
cd omni-lang && cargo test -p opm --all-targets --quiet
cd omni-lang && cargo test -p omni-fmt -p omni-lsp -p omni-dap -p opm -p ovm-runner --all-targets --quiet
```

New automated coverage includes:

- workspace member expansion with `exclude`,
- `default-members` preservation,
- deterministic workspace member ordering,
- missing explicit member detection.

---

## What Stage D still does not prove

This slice materially improves Stage D workspace behavior, but it does **not** complete the full Stage D exit criteria.

Still remaining:

- full registry-backed dependency resolution,
- broader multi-package compiler integration,
- build script integration beyond the current runner path,
- lockfile fidelity for non-local dependency graphs,
- wider reproducibility checks across more complex workspace layouts.
