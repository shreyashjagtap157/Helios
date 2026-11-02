# Contributing to Helios

Helios is currently a personal concept and specification project. No implementation has started.

## Current contribution scope

Useful contributions at this stage include:
- Research summaries with authoritative sources.
- Architecture alternatives and trade-off analysis.
- Requirement clarifications.
- Non-training reasoning prototypes proposed for later evaluation.
- Hardware and OS security analysis.
- Threat models and safety requirements.
- Reproducible benchmark designs.

Do not create implementation scaffolding and describe it as a working Helios subsystem.

## Evidence standards

Research contributions must record sources, versions, dates, assumptions and limitations. Technical claims must distinguish documented facts from proposals.

When implementation begins, every change must identify requirements, tests, results, failure cases, security implications and known limitations.

## Non-training dependency review

Before proposing a library, SDK, plugin, operating-system service or external service, review [docs/NON_TRAINING_COMPLIANCE.md](docs/NON_TRAINING_COMPLIANCE.md). The review must cover transitive dependencies, optional features, build scripts, model assets, network destinations and offline behavior. Unknown behavior is a reason to defer or reject a dependency, not an assumption of compliance.

## Scope discipline

Do not introduce model training, model inference, pretrained models, learned embeddings, model weights or model-based services. Do not add privileged device-control paths without an explicit capability and safety design.

The master specification is authoritative. Update the decision and status registers when an approved project decision or verified state changes.
