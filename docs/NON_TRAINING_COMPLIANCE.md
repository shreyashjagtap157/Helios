# Helios Non-Training and Perception Compliance Policy

**Status:** Normative implementation policy derived from HEL-NTR-001 through HEL-NTR-010 and HEL-PER-001 through HEL-PER-008.  
**Project phase:** Design only. No implementation or automated compliance gate currently exists.  
**Authority:** HELIOS_MASTER_SPEC.md is the canonical requirements source.

## 1. Governing rule

Helios never trains, pretrains, fine-tunes, distills, fits, updates or continually adapts a neural/learned model.

Frozen pretrained-model inference is permitted only inside the approved Neural Perception Boundary (NPB), solely to convert supplied sensory/unstructured input into untrusted, typed candidate observations. It is not permitted for reasoning, intent interpretation, dialogue orchestration, planning, decisions, memory, knowledge revision, tool selection, policy evaluation or action authorization.

The initial NPB profile is local-only and network-denied. Remote perception requires a separate decision and verifiable immutability, provenance, isolation and data-handling evidence.

## 2. Absolute prohibitions

- Training, pretraining, fine-tuning, distillation, fitting or adaptation of model parameters.
- Automatic updates to weights or inference-affecting artifacts.
- Neural models for cognitive reasoning, planning, intent interpretation, decisions, memory or policy.
- Learned embeddings, latent-vector retrieval, learned rerankers or learned indexes in the symbolic core.
- NPB access to memory, credentials, tools, network clients or privileged interfaces.
- Perception output setting trust, epistemic status, persistence, authority or authorization.
- Unapproved/remote model fallback when perception is unavailable.
- Training datasets, optimizer state, model-update pipelines or unapproved model artifacts in supported builds.
- Hidden inference through transitive dependencies, optional features, plugins, SDKs, OS services or hosted endpoints.

## 3. NPB model admission

An approved deployment is pinned by publisher, immutable model ID/version, SHA-256 digests for weights and processor/tokenizer assets, preprocessing and inference-configuration digests, runtime version/build digest, backend, hardware profile, precision/quantization and sandbox profile.

Admission also requires proof that the process cannot access cognitive memory, symbolic rules, executive state, tools, credentials, arbitrary filesystem paths, network or action broker. The model's only role is perception. Changes require a new admission record and regression suite.

Frozen weights do not guarantee identical output across drivers, kernels, runtimes, backends or numerical paths. Perception repeatability is measured per pinned execution profile. Core determinism is scoped to identical validated observations and symbolic configuration.

## 4. Observation contract and trust

The v1 wire format is UTF-8 JSON validated using JSON Schema Draft 2020-12. Raw input bytes and the canonical observation envelope use SHA-256. The envelope is canonicalized using RFC 8785 JCS after removing the integrity object.

A hash establishes byte integrity/content identity, not source authenticity or truth. Source attestation is evaluated separately. Epistemic admission is a symbolic operation and cannot be replaced by a model score.

Perception strings are inert data. The schema rejects undeclared fields; the gateway additionally enforces cross-field references, payload bounds and allowlisted model manifests. No memory pointer, command, tool call, executable markup or policy field is accepted.

## 5. Dependency admission

Reviewers record identity/version/publisher/license; direct/transitive dependencies and enabled features; training/fitting/inference and model-download behavior; NPB-only role; artifact digests; runtime network/filesystem/IPC behavior; sandbox controls and negative tests; offline behavior; build hooks; and accept/reject/defer decision with evidence.

The symbolic core dependency set contains no neural inference facility. The NPB set may contain only the exact reviewed perception runtime and immutable artifacts. If relevant behavior cannot be established, the dependency is rejected or deferred.

## 6. Build and release evidence

Each supported build/release should produce an SBOM, enabled-feature manifest, NPB model manifest, scans for training data/optimizer state/unapproved weights/undeclared checkpoints/learned cognitive indexes, network/IPC review, runtime isolation tests, offline tests with NPB disabled, schema/adversarial tests and human-reviewed dependency records.

Automated scans are defense-in-depth, not proof by themselves.

## 7. Memory policy compliance

All cognitive memory is task/session ephemeral by default. Durable episodic, semantic, procedural, prospective, autobiographical or preference retention requires explicit scoped authorization. The NPB cannot write memory or choose retention.

Safety/action audit records are separate under their own access and retention policy and are not implicitly loaded into cognitive context. Expiration/deletion tests cover caches and derived indexes as well as primary task records.

## 8. Change control

A change to perception role, model/network profile, memory default, observation envelope or retention behavior requires a dated decision, master-spec update, dependency/safety impact assessment and acceptance-test update.

## 9. Current verification state

The repository contains specifications and project-control records only. There is no NPB, cognitive runtime, dependency manifest, model package, SBOM, memory subsystem or automated compliance gate. This policy defines future evidence; it does not claim implementation conformance.