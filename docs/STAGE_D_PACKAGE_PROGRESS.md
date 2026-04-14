# Stage D Package Progress
## Package graph and lockfile hardening slice

**Date:** April 13, 2026  
**Plan source:** `docs/OMNI_FORWARD_COMPLETION_PLAN.md`  
**Scope:** Stage D follow-up after the Stage C ownership slice

---

## Completed in this slice

### 1. `opm install` now resolves local path dependencies transitively

Before this slice, `opm install` wrote `omni.lock` entries only for direct dependencies listed in the root manifest.

That behavior is now improved for the currently supported local package flow:

- direct local `path` dependencies are loaded from their own `omni.toml`,
- nested local `path` dependencies are resolved recursively,
- transitive local packages are written into `omni.lock`,
- dependency edges are preserved in each locked package entry.

### 2. Lockfile output is now deterministic for the supported install path

The install pipeline now emits packages and dependency edges in sorted order.

That means:

- lockfile package ordering is stable,
- dependency lists within lockfile entries are stable,
- repeated installs produce a predictable structure for the same input graph.

### 3. Transitive local dependency conflicts now fail explicitly

Before this slice, conflicting transitive local packages could be silently flattened away because the simplified install path did not actually resolve the graph.

That gap is now covered:

- if two transitive local dependencies resolve to the same package name with different versions or sources,
- `opm install` fails with an explicit conflict error instead of producing a misleading lockfile.

### 4. `build.omni` cfg directives now flow into the compiler invocation

Before this slice, `opm build` executed `build.omni` and logged directives, but did not pass cfg directives to `omnc`.

That is now wired for the current build path:

- `opm build` now invokes `omnc` on project `main.omni` with a concrete output path,
- `cargo:rustc-cfg=...` directives from `build.omni` are exported as `OMNI_CFG_FLAGS`,
- `omnc` consumes `OMNI_CFG_FLAGS` during `#[cfg(...)]` filtering,
- link library and search-path directives are now exported (`OMNI_LINK_LIBS`, `OMNI_LINK_PATHS`) for downstream linker integration.

---

## Verification

The following checks passed after the Stage D implementation:

```bash
cd omni-lang && cargo test -p opm --all-targets --quiet
cd omni-lang && cargo test -p omni-fmt -p omni-lsp -p omni-dap -p opm -p ovm-runner --all-targets --quiet
cd omni-lang && cargo test -p omnc --all-targets --quiet
```

New automated coverage includes:

- transitive local path dependency lockfile generation,
- deterministic package ordering in the written lockfile,
- explicit detection of conflicting transitive local packages,
- build output path layout checks,
- build-script directive environment propagation checks.

---

## What Stage D still does not prove

This slice improves Stage D materially, but it does **not** complete the full Stage D exit criteria.

Still remaining:

- stronger workspace discovery and member selection semantics,
- full registry-backed transitive dependency resolution,
- lockfile fidelity for non-local dependency graphs,
- full linker/package consumption of `OMNI_LINK_LIBS`/`OMNI_LINK_PATHS`,
- broader compiler/package integration for multi-package Omni builds,
- deterministic workspace behavior across more real project layouts.
