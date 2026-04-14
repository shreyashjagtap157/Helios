# Stage B Baseline Verification
## Omni workspace re-verification and immediate findings

**Date:** April 13, 2026  
**Scope:** Stage B from `docs/OMNI_FORWARD_COMPLETION_PLAN.md`  
**Goal:** Re-verify the current compiler/workspace baseline and identify immediate blockers before deeper implementation work.

---

## 1. Verification strategy used

An initial workspace-wide command was attempted with all features enabled:

```bash
cargo test --workspace --all-targets --all-features --quiet
```

That check is **not portable** on the current machine because the compiler crate exposes an optional GPU feature set:

- `cust` (CUDA)
- `ocl` (OpenCL)
- `ash` (Vulkan)

The `cust` dependency failed during build-script discovery because no CUDA installation was available on the machine.

This means the `--all-features` command should **not** be used as the default Stage B baseline check for portable verification on developer machines without CUDA installed.

### Immediate conclusion

The first baseline issue is **environmental / verification-strategy related**, not a confirmed regression in the Omni codebase itself.

---

## 2. Portable baseline commands that were verified

### 2.1 Compiler crate baseline

Command:

```bash
cd omni-lang && cargo test -p omnc --all-targets --quiet
```

Result:

- library/binary test set: **552 passed**
- additional test set: **461 passed**
- remaining target set: **0 tests / pass**
- overall result: **PASS**

### 2.2 Stage1 compiler baseline

Command:

```bash
cd omni-lang && cargo test -p omni_stage1 --all-targets --quiet
```

Result:

- unit tests: **4 passed**
- conformance/parity test group: **1 passed**
- phase3 enrichment baseline test group: **1 passed**
- overall result: **PASS**

Observed warnings during stage1 validation:

- `main` function effect mismatch warnings involving inferred `IO`
- `fetch` function effect mismatch warning involving inferred `Async`

These did **not** fail the tests, but they are significant because they confirm that effect-system enforcement/annotation behavior remains incomplete or inconsistent with intended semantics.

### 2.3 Tools and VM baseline

Command:

```bash
cd omni-lang && cargo test -p omni-fmt -p omni-lsp -p omni-dap -p opm -p ovm-runner --all-targets --quiet
```

Result:

- test groups: **5 passed**, **3 passed**, **3 passed**, **9 passed**, **0 tests / pass**
- overall result: **PASS**

---

## 3. Current Stage B baseline status

## 3.1 Verified green components

The following workspace members were successfully re-verified on this machine without optional GPU features:

| Workspace member | Status |
|---|---|
| `omnc` | PASS |
| `omni_stage1` | PASS |
| `omni-fmt` | PASS |
| `omni-lsp` | PASS |
| `omni-dap` | PASS |
| `opm` | PASS |
| `ovm-runner` | PASS |

## 3.2 Verified blocker

| Item | Status | Notes |
|---|---|---|
| Workspace all-features baseline | BLOCKED (environmental) | Fails on `cust_raw` build script when CUDA is not installed |

---

## 4. Interpretation

### 4.1 What Stage B established

Stage B re-verification now supports the following conclusions:

1. The **current non-GPU workspace baseline is green** for the main compiler, stage1 compiler, tools, package manager, and VM runner.
2. The first failed baseline run was caused by **forcing optional GPU dependencies** in an environment without CUDA.
3. The repository already has a **portable baseline path** that can be used for normal development verification.
4. There are still **semantic warnings** in stage1 validation related to the effect system, which aligns with the forward plan’s conclusion that effects are not yet specification-complete.

### 4.2 What Stage B did not yet prove

This verification pass did **not** prove:

- full all-features support on every developer machine,
- CUDA/GPU backend readiness,
- full specification compliance,
- effect-system correctness,
- package/build-system completeness,
- self-hosting completion.

Stage B only re-established that the **core currently testable baseline is working** when verified in a portable way.

---

## 5. Recommendations from this baseline pass

## 5.1 Immediate repository/process recommendation

Adopt the following as the default portable Stage B baseline command set:

```bash
cd omni-lang && cargo test -p omnc --all-targets --quiet
cd omni-lang && cargo test -p omni_stage1 --all-targets --quiet
cd omni-lang && cargo test -p omni-fmt -p omni-lsp -p omni-dap -p opm -p ovm-runner --all-targets --quiet
```

Do **not** treat `--all-features` as the default baseline unless the environment is explicitly prepared for GPU/CUDA dependencies.

## 5.2 Next technical recommendation

The next high-value follow-up after Stage B is:

1. either formalize GPU verification as a separate environment-specific lane,
2. or gate/document GPU feature verification more clearly in CI/docs,
3. then move into Stage C:
   - ownership/borrow validation,
   - field projection regression coverage,
   - linear type enforcement checks.

## 5.3 Effect-system follow-up signal

The stage1 warnings should be preserved as an explicit tracked issue because they are evidence that:
- effect inference is active enough to emit meaningful warnings,
- but the language/compiler is still not fully aligned on declared vs inferred effects.

This supports the forward plan’s prioritization of later Stage F effect-system completion.

---

## 6. Final Stage B status

**Stage B baseline re-verification status:** **SUCCESSFUL (portable baseline)**

### Final summary
- Portable workspace baseline: **GREEN**
- Optional GPU/all-features baseline: **environment-dependent / not portable**
- Stage1 semantic pipeline: **GREEN with effect mismatch warnings**
- Ready to proceed to the next plan stage after documenting the baseline and preserving the warnings as known follow-up items.
