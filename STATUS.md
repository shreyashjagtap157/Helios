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

- main is protected and currently requires status contexts named check, test, clippy, and fmt.
- The current default-branch tree has no .github/workflows/ directory. Therefore this repository currently defines no GitHub Actions workflows that produce those contexts.
- Whether an external integration publishes any of those status contexts has not been established. The required checks and actual CI design must be reconciled before treating pull-request validation as operational.
- dev currently points to the same commit as main but is not protected. main is the repository default branch.

## Current work phase

Research and architecture refinement. The next practical work is to define a small target use case, select a bounded reasoning domain, evaluate candidate technologies and design a mock hardware-action vertical slice.

## Evidence policy

No progress percentage is reported. A project phase changes only when its exit criteria have reproducible evidence.
