# Helios Research Register

This is a research index, not a claim that all topics have been fully investigated or that listed projects are approved dependencies.

## Symbolic reasoning and logic

- Soufflé Datalog: https://souffle-lang.github.io/docs/home/
  - Declarative relational reasoning, compiled evaluation and explicit rule semantics.
- Soufflé rules and stratification: https://souffle-lang.github.io/rules
  - Grounded variables, rule structure and stratified negation.
- Potassco clingo: https://potassco.org/clingo/
  - Answer-set programming and combinatorial reasoning; deferred beyond the first slice.
- Z3 Guide: https://microsoft.github.io/z3guide/
  - SMT solving, constraints, proof/unsat-core options and resource limits; defer until a concrete use case.
- Z3 parameters: https://microsoft.github.io/z3guide/programming/Parameters/
  - Resource limits and timeouts; preserve unknown/timeout semantics.

## Knowledge integrity and epistemic reasoning

- Belnap/paraconsistent logic: https://plato.stanford.edu/entries/logic-paraconsistent/
  - Positive support, negative support, both and neither without unrestricted explosion.
- Truth-maintenance systems and belief revision: https://www.cs.toronto.edu/~doyle/publications/ai/1983/tms.pdf
  - Justifications and dependency-directed invalidation.
- Provenance-aware databases and event sourcing.
  - Research: lineage, retraction, temporal validity, source authenticity and audit immutability.

## Perception boundary and AI security

- OWASP LLM01:2025 Prompt Injection: https://genai.owasp.org/llmrisk/llm01-prompt-injection/
  - Direct/indirect and multimodal injection, least privilege, output validation and adversarial testing.
- NIST AI 600-1 Generative AI Profile: https://doi.org/10.6028/NIST.AI.600-1
  - Model lifecycle risk identification, governance and measurement.
- JSON Schema: https://json-schema.org/specification
  - Versioned validation contract; v1 uses Draft 2020-12.
- RFC 8785 JCS: https://www.rfc-editor.org/rfc/rfc8785.html
  - Canonical representation for hashing/signing; Informational RFC, not an Internet Standards Track specification.
- NIST SHA-256 / FIPS 180-4: https://csrc.nist.gov/pubs/fips/180-4/upd1/final
  - Message digest for input and envelope integrity.

Research questions:
- Which OS process/sandbox controls provide a testable NPB boundary?
- How can model weights, processor, runtime, quantization, kernels and inference configuration be pinned?
- Which local runtimes provide reproducible output on target hardware?
- How is sensor/source attestation represented separately from content hashes?
- How are schema-valid text and multimodal content prevented from gaining instruction authority?
- What strict limits apply to input/output bytes, nesting, candidate count, time and memory?
- How is NPB compromise contained and detected?

## Memory lifecycle and persistence

Research task-scoped memory allocation, deterministic expiry, crash cleanup, cross-session leakage, explicit persistence grants, revocation/deletion, cache/index erasure, audit separation and immutable episodic evidence versus versioned semantic interpretations. No automatic model-driven consolidation or embedding retrieval.

## Systems languages and operating environments

- Rust for Linux: https://www.kernel.org/doc/html/latest/rust/
- Rust Embedded: https://github.com/rust-embedded
- Embedded Rust Book: https://doc.rust-lang.org/embedded-book/intro/no-std.html
- Tock OS: https://tockos.org/documentation/getting-started/
- seL4 docs: https://docs.sel4.systems/projects/sel4/
- seL4 capabilities: https://docs.sel4.systems/Tutorials/capabilities.html
- seL4 hardware support: https://docs.sel4.systems/Hardware/

Research: compare Rust and C boundaries; Soufflé and Rust-native Datalog; process isolation/capability systems/IOMMU/DMA/firmware trust; select host OS after a representative vertical slice.

## Planning, interaction and classical perception

Research PDDL/HTN planning, deterministic grammars/semantic parsing, classical vision/signal processing, causal models and structural analogy.

## Source-quality and adoption rules

Prefer official documentation, standards, original research and maintained repositories. Record exact versions, licenses, target support, transitive dependencies, optional features, maintenance, network behavior, resource limits and model dependence. A research reference is not adoption approval.

Keep Phase A small: one typed fact/claim model, provenance, one bounded rule-evaluation path and deterministic replay. Defer SMT, ASP, general planning and additional perception modalities until evidence justifies them.