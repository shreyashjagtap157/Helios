# Helios Non-Training Compliance Policy

**Status:** Normative implementation policy derived from HEL-NTR-001 through HEL-NTR-010.  
**Project phase:** Design only. No implementation or automated compliance gate currently exists.  
**Authority:** HELIOS_MASTER_SPEC.md remains the canonical requirements source. This document defines the evidence expected to demonstrate conformance.

## 1. Governing rule

Helios must not train, fine-tune, fit, update or infer a trained or pretrained model. This applies to models created by the project, models supplied by third parties, local model runtimes, remote inference APIs, plugins, vendor SDKs and operating-system services.

The restriction is not satisfied merely because Helios does not run a training job. Inference-only use of pretrained models is also prohibited.

## 2. Prohibited capabilities and assets

A supported Helios build or workflow must not:

- Train, pretrain, fine-tune, distill or continually adapt any machine-learning model.
- Fit learned model parameters from user data or observations.
- Invoke local or remote inference of trained or pretrained models.
- Use LLMs, transformers, learned speech or vision models, learned classifiers, learned rankers or learned recommendation models.
- Create or consume learned vector embeddings or latent representations.
- Download, load, bundle or publish model weights, checkpoints or training datasets.
- Send user or device data to hosted AI/model-inference services.
- Introduce any of these capabilities through an optional dependency, plugin, SDK, system service or transitive package.

A model-related feature being disabled by default is not enough. It must be excluded from supported builds and the exclusion must be evidenced.

## 3. Permitted mechanisms

The following are compatible when their behavior is explicit and testable:

- Deterministic algorithms and fixed mathematical transforms.
- Human-authored rules, ontologies, schemas and domain models.
- Explicit fact and observation acquisition with provenance.
- Parsing, indexing and searching documents using non-learned algorithms.
- Symbolic inference, theorem proving within declared fragments, graph algorithms, search, constraint solving and planning.
- Explicit statistical calculations over supplied data when the calculation is not fitting or updating a predictive model.
- Versioned user-approved changes to knowledge, rules, procedures and preferences.
- Bounded procedural composition from an allowlist of verified primitives.

Knowledge acquisition, memory formation and correction of explicit facts are not model training. A rule or procedure change must remain inspectable, versioned, attributable and reversible.

## 4. Dependency admission gate

Before adopting a dependency, reviewers must record:

| Review item | Required evidence |
|---|---|
| Identity | Name, exact version, source repository or publisher |
| License | License identifier and compatibility decision |
| Maintenance | Release activity, support status and known security concerns |
| Dependency graph | Direct and transitive dependencies; lockfile or resolved graph |
| Optional features | Activated feature flags and optional packages |
| Model behavior | Explicit determination of training, fitting, inference, embedding and model-download capability |
| Runtime services | Network destinations, API calls, background services and telemetry behavior |
| Assets | Model files, checkpoints, datasets, learned indexes or generated artifacts |
| Build behavior | Build scripts, code generation and install/runtime hooks |
| Offline behavior | Whether core functionality remains available without external services |
| Decision | Accept, reject or defer, with reviewer and date |

If a reviewer cannot determine whether a component introduces prohibited behavior, it must be deferred or rejected. Do not infer compliance from package descriptions, popularity, licensing or the fact that a component is open source.

## 5. Build and release evidence

When implementation begins, each supported build should produce:

1. A software bill of materials covering direct and transitive dependencies.
2. A record of enabled optional features and resolved packages.
3. An artifact scan for prohibited model weights, checkpoints, learned embedding assets and training datasets.
4. A review of network destinations and integrations for model-inference services.
5. Offline tests demonstrating that the cognitive core does not require an external AI/model service.
6. A human-reviewed dependency admission record for every new or upgraded dependency.

Automated filename, package-name or string scans are useful tripwires but are not sufficient proof. They can miss renamed assets and can produce false positives. Combine them with dependency graph inspection, source review, build configuration review and runtime tests.

## 6. Change control

No exception is permitted under the current approved decisions. Any future reconsideration requires an explicit owner decision, a dated change to DECISIONS.md, revision of the master specification and an impact assessment against all HEL-NTR requirements.

A dependency that violates this policy must not be merged while a policy change is merely proposed.

## 7. Current verification state

The repository currently contains specifications and project-control documents only. There are no Helios dependency manifests, runtime binaries, model assets, SBOMs or executable compliance checks to validate. This policy defines future acceptance evidence; it does not claim that an implementation has passed it.
