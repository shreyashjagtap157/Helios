# Helios Neural Perception Boundary (NPB)

**Status:** Normative design derived from HEL-PER-001 through HEL-PER-008.  
**Authority:** HELIOS_MASTER_SPEC.md.  
**Implementation:** Not implemented.

## Purpose

The NPB is a quarantined sensory peripheral. Its only permitted purpose is to transform a supplied sensory or unstructured input into bounded, typed candidate observations.

It does not reason, plan, decide, interpret user intent, manage memory, choose tools, authorize actions, write knowledge or generate the system's answer. It cannot determine whether its output is true. Model inference is permitted only inside this boundary; training, fine-tuning, distillation, parameter updates, learned memory/retrieval and neural cognition remain prohibited.

## Security profile

The initial supported profile is local-only and network-denied. The NPB receives read access only to the input for the current request, access only to approved immutable model artifacts, bounded compute/memory/time and a bounded output channel.

It receives no access to cognitive memory, durable knowledge, rules, task history, credentials, shell, arbitrary filesystem paths, process spawning, action broker, device-control interfaces, plugins, tools, network or generic IPC to the executive. Schema validation occurs outside the NPB trust domain. A process boundary alone is not proof; concrete OS sandbox controls and negative access tests are required.

## Frozen deployment manifest

Each admitted deployment pins publisher, immutable model ID/version, SHA-256 weights digest, processor/tokenizer digest, preprocessing/configuration digests, inference configuration digest, runtime version/build digest, backend, hardware profile, precision/quantization, kernel choices and sandbox profile. No automatic downloads, updates, aliases, fallbacks or configuration drift are allowed. A changed artifact or inference-affecting setting requires re-admission and regression testing.

Frozen weights do not guarantee bit-identical outputs across hardware, drivers, kernels, runtimes or backends. The core reproducibility guarantee is scoped to identical validated observations and symbolic configuration. Perception repeatability is measured separately for each pinned execution profile.

## PerceptionObservation v1

The wire format is UTF-8 JSON validated against JSON Schema Draft 2020-12. The normative schema is `../../schemas/perception-observation.schema.json`.

The envelope carries:
- Gateway-issued observation UUID and issuance timestamp.
- Source identity, source-authentication status, modality and capture timestamp where known.
- Raw input byte length, media type, SHA-256 digest and opaque content-addressed reference.
- Preprocessor, model, processor, runtime, backend, hardware and execution-profile provenance.
- Digest of raw model-output bytes for diagnostic correlation; raw unstructured model output is not passed to cognition.
- Zero or more bounded typed candidates: entity, attribute, text segment, spatial region, event, measurement or relation.
- Gateway-stamped epistemic status `UNTRUSTED_CANDIDATE`, authority `NONE` and retention `TASK_EPHEMERAL`.
- An envelope digest over canonical JSON, excluding the integrity object itself.

Candidate text is inert data. Candidate-local references must resolve only within the same envelope. The schema and semantic validator reject undeclared properties, arbitrary pointers/paths/URIs, executable markup, tool calls, policy/authority fields and malformed references. Model scores are untrusted model metadata, not calibrated confidence, truth or authorization.

The gateway—not the model—issues the observation ID/timestamp and stamps epistemic status, authority and retention.

## Canonicalization and integrity

Hash the exact raw input bytes with SHA-256 before model processing. After schema validation, remove the envelope's integrity object, canonicalize the remaining JSON using RFC 8785 JCS, UTF-8 encode it and calculate SHA-256. Store the result in `integrity.payload_digest_sha256`.

RFC 8785 is an Informational RFC, not an Internet Standards Track specification. Implementations must pass its test vectors and reject duplicate object keys, invalid Unicode and values outside I-JSON/JCS constraints. Do not normalize strings before hashing.

Integrity, authenticity and epistemic support are separate:
- **Integrity:** bytes match the digest.
- **Authenticity:** source identity is supported by signature, device attestation or authenticated transport, where available.
- **Epistemic support:** a claim is supported under Helios source/corroboration policy.

A hash proves neither source identity nor truth.

## Admission gateway

The trusted gateway:
1. Enforces byte, nesting, string, candidate-count and processing-time limits.
2. Validates JSON Schema and semantic cross-field/reference constraints.
3. Verifies input and envelope digests.
4. Verifies model/runtime/processor/sandbox manifests against an allowlist.
5. Attaches source attestation status separately from model claims.
6. Stamps `UNTRUSTED_CANDIDATE`, `NONE` and `TASK_EPHEMERAL`.
7. Delivers the envelope as read-only task-local input.
8. Leaves belief admission, contradiction handling and durable persistence to symbolic policies.

Semantic checks beyond schema validation are mandatory: references must resolve; normalized coordinates must be in range; rectangle bounds must be ordered; numeric values must be finite; all limits must be enforced. Malformed or unverifiable data is rejected or retained only in a separate quarantine diagnostic channel, never partially trusted.

## Failure and validation

NPB outage, timeout, unsupported input, malformed output or failed manifest/digest verification returns unavailable/unknown. There is no fallback to a hosted LLM, an unreviewed model or neural reasoning. The symbolic core remains operational with the NPB disabled.

Acceptance tests must demonstrate denied access to memory/rules/executive/tools/secrets/network/action broker; rejection of malformed and oversized envelopes; resistance to command smuggling in text segments; detection of model/runtime/processor/quantization changes; integrity failure after byte changes; no truth promotion from a valid digest; NPB outage isolation; and no policy/action change from adversarial image, document or audio content.

## References

- JSON Schema 2020-12: https://json-schema.org/specification
- RFC 8785 JCS: https://www.rfc-editor.org/rfc/rfc8785.html
- OWASP LLM01:2025 Prompt Injection: https://genai.owasp.org/llmrisk/llm01-prompt-injection/
- NIST AI 600-1 Generative AI Profile: https://doi.org/10.6028/NIST.AI.600-1
