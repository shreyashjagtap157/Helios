# Helios Project Status

**As of:** 2026-11-05  
**Overall status:** Concept and specification only  
**Implementation:** Not started

## Verified facts

- The intended project is a personal cognitive computing framework.
- It has a strict non-training requirement.
- Frozen pretrained-model inference is permitted only in the isolated NPB for perception; neural cognition remains prohibited.
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
- No Helios runtime conformance suite or validated capability benchmarks.

## Current approved constraints

- DEC-012: frozen foundation models are permitted only as isolated perception peripherals producing untrusted observations.
- DEC-013: cognitive memory is ephemeral by default; durable cross-task/session retention is opt-in.
- DEC-014: PerceptionObservation v1 uses JSON Schema 2020-12, RFC 8785 JCS canonicalization and SHA-256 digests.

- No training of any model.
- No training, fine-tuning or neural inference outside the NPB.
- No learned cognitive embeddings; only reviewed, digest-pinned perception weights may be admitted to the NPB deployment.
- Explicit knowledge acquisition and revision are permitted.
- Reasoning and privileged hardware execution must be separated.
- User authority and independent action policy are mandatory.
- No custom kernel until a demonstrated need justifies it.

## Repository governance findings

- `main` remains protected and still requires the four old status contexts: `check`, `test`, `clippy`, and `fmt`. GitHub has not persisted their replacement with `docs`.
- The repository-owned `.github/workflows/docs.yml` workflow publishes a stable `docs` job on pushes to `main` and `dev`, and on pull requests targeting `main`.
- The workflow grants read-only `GITHUB_TOKEN` permissions, pins `actions/checkout` to a full commit SHA, disables persisted checkout credentials, and runs a dependency-free Ruby validator on Ubuntu 24.04.
- Hosted validation of revision `1f54a2cc24c55944fec68180029262e96522dc79` passed on both branches: `main` run [41469285815](https://github.com/shreyashjagtap157/Helios/actions/runs/41469285815) and `dev` run [41469294548](https://github.com/shreyashjagtap157/Helios/actions/runs/41469294548).
- The next revision `6a82b867dcceee22fc1262da4b5eae92e863df69` also passed on both branches: `main` run [41469777104](https://github.com/shreyashjagtap157/Helios/actions/runs/41469777104) and `dev` run [41469788857](https://github.com/shreyashjagtap157/Helios/actions/runs/41469788857). Both jobs reported `success`, the validator step passed, and the runner label was `ubuntu-24.04`.
- The earlier initial workflow run [41467896240](https://github.com/shreyashjagtap157/Helios/actions/runs/41467896240) used `ubuntu-latest` and emitted a runner-migration notice. The workflow was subsequently pinned to Ubuntu 24.04.
- An attempted required-check replacement was blocked by GitHub's sudo-mode passkey/MFA challenge. The settings form showed `docs` as the sole required check, but the server-side readback confirms no change was saved.
- The available GitHub connector supports branch-protection reads but does not expose a branch-protection write action. The remaining settings change therefore requires the repository owner to complete GitHub's interactive sudo-mode verification and save it in repository settings.
- `dev` and `main` are synchronized at the current revision; `main` is protected and `dev` is not.

## Current work phase

Architecture specification and research refinement. The NPB trust boundary, PerceptionObservation v1 contract and opt-in memory policy are specified in the review branch. Phase A is specified but not implemented. Rust and a Datalog engine remain candidates pending a bounded comparison spike. No Rust workspace, runtime, model deployment or cognitive memory subsystem has been created.

## Evidence policy

No progress percentage is reported. A project phase changes only when its exit criteria have reproducible evidence.
