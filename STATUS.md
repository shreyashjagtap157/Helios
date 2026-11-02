# Helios Project Status

**As of:** 2026-11-02  
**Overall status:** Concept and specification only  
**Implementation:** Not started

## Verified facts

- The intended project is a personal cognitive computing framework.
- It has a strict non-training requirement.
- Pretrained models are excluded, even for inference-only use.
- It is intended to maintain explicit knowledge and memory, reason, plan and evaluate its own conclusions.
- It is intended to interact with hardware for user-authorized requests.
- A wider operating-system ecosystem is a long-term goal.
- The project should begin with existing languages and an existing host operating system.
- No implementation has started.

## Not implemented

- No cognitive runtime.
- No reasoning engine.
- No knowledge base.
- No persistent memory subsystem.
- No natural-language parser.
- No planning engine.
- No device inventory or hardware broker.
- No hardware drivers or device adapters.
- No operating system or distribution.
- No application or plugin runtime.
- No conformance suite or validated benchmarks.

## Current approved constraints

- No training of any model.
- No fine-tuning or pretrained-model inference.
- No learned embeddings or model weights.
- Explicit knowledge acquisition and revision are permitted.
- Reasoning and privileged hardware execution must be separated.
- User authority and independent action policy are mandatory.
- No custom kernel until a demonstrated need justifies it.

## Repository governance findings

- main remains protected and still requires the old status contexts `check`, `test`, `clippy`, and `fmt`; GitHub has not yet persisted the replacement with `docs`.
- The new `docs` workflow has passed on both `main` and `dev` for the same commit. The GitHub Actions job succeeded on Ubuntu 24.04; branch protection has not yet been switched to require `docs`.
- The runner is pinned to ubuntu-24.04 to avoid an unplanned hosted-image migration.
- Added `.github/workflows/docs.yml`, which publishes a stable `docs` job on pushes to main/dev and pull requests targeting main.
- The workflow uses a read-only GITHUB_TOKEN, a full-SHA-pinned official checkout action, and a dependency-free Ruby validator.
- Earlier hosted run [41467896240](https://github.com/shreyashjagtap157/Helios/actions/runs/41467896240) for commit `47b3b57323d71d29cb4a819e49ba29e94ddfea1e` passed on the initial runner configuration; it emitted an `ubuntu-latest` migration notice, which motivated explicit runner pinning.
- Latest hosted run on `main`: [41469285815](https://github.com/shreyashjagtap157/Helios/actions/runs/41469285815), commit `1f54a2cc24c55944fec68180029262e96522dc79`; status and conclusion `success`, with the `docs` validator step successful and runner label `ubuntu-24.04`.
- Latest hosted run on `dev`: [41469294548](https://github.com/shreyashjagtap157/Helios/actions/runs/41469294548), same commit; status and conclusion `success`, with the `docs` validator step successful and runner label `ubuntu-24.04`.
- Attempted to replace the old required contexts with `docs`, but GitHub required a fresh sudo-mode passkey/MFA check to save. The browser showed the updated form, but the server-side readback confirms the change was not persisted.
- `main` therefore remains protected with the old required contexts `check`, `test`, `clippy`, and `fmt`. The replacement is still pending user-completed GitHub reauthentication.
- dev currently points to the same commit as main but is not protected. main is the repository default branch.

## Current work phase

Research and architecture refinement. The next practical work is to define a small target use case, select a bounded reasoning domain, evaluate candidate technologies and design a mock hardware-action vertical slice.

## Evidence policy

No progress percentage is reported. A project phase changes only when its exit criteria have reproducible evidence.
