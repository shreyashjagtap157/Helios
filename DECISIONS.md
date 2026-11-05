# Helios Decision Register

This register tracks choices that govern the project. The master specification remains authoritative for requirements.

## Approved decisions

| ID | Decision | Rationale | Status |
|---|---|---|---|
| DEC-001 | Helios is a personal cognitive framework, not a programming language. | Establishes project identity and scope. | Approved |
| DEC-002 | Helios is currently in concept/specification phase only. No implementation has started. | Corrects historical repository descriptions and prevents false progress claims. | Approved |
| DEC-003 | No model training of any kind is permitted. | Core design constraint. | Approved |
| DEC-004 | Neural inference is excluded from cognition; the restricted perception exception is governed by DEC-012. | Prevents pretrained models from becoming reasoning or decision-making components. | Superseded by DEC-012 |
| DEC-005 | Knowledge acquisition and explicit knowledge revision are permitted; model-parameter learning is not. | Preserves the ability to acquire information without training. | Approved |
| DEC-006 | Hardware interaction is a first-class target capability. | Helios must be able to perform authorized user-requested operations. | Approved |
| DEC-007 | A broader operating-system ecosystem is a long-term vision. | Hardware and system control require deeper integration over time. | Approved |
| DEC-008 | Initial work should use existing, mature programming languages and host operating systems. | Avoids depending on an unimplemented language or premature custom kernel. | Approved |
| DEC-009 | The repository is a design-only project until real implementation is explicitly started and evidenced. | Prevents scaffolding from being represented as working software. | Approved |
| DEC-010 | Model inference is prohibited except for the isolated perception operation approved by DEC-012. | Prevents inference-only use from entering symbolic cognition or hidden optional dependencies. | Superseded by DEC-012 |
| DEC-011 | Every dependency and supported build must undergo non-training, model-pinning and isolation conformance review. | Makes the policy enforceable across the software supply chain and restricted NPB. | Approved |
| DEC-012 | Frozen foundation-model inference is permitted only inside an isolated, local-only, network-denied NPB, solely to produce untrusted candidate observations. Training, fine-tuning, neural reasoning/planning/decisions/memory, model-based intent interpretation and action authority remain prohibited. | Preserves a strictly symbolic cognitive core while permitting controlled sensory interpretation. | Approved |
| DEC-013 | Cognitive memory is ephemeral by default. Cross-task/session episodic, semantic, procedural, prospective and autobiographical retention requires explicit scoped opt-in authorization. Safety/action audit records are separate and are not implicit cognitive memory. | Prevents uncontrolled persistent cognitive flow and silent memory promotion. | Approved |
| DEC-014 | PerceptionObservation v1 uses JSON validated by JSON Schema 2020-12, RFC 8785 JCS canonicalization and SHA-256 digests for raw-input and envelope integrity. Protobuf is deferred until a canonical integrity profile is defined and tested. | Establishes a versioned, language-neutral and hashable observation contract. | Approved |

## Proposed decisions

| ID | Proposal | Reason | Required evidence |
|---|---|---|---|
| PROP-001 | Use Rust as the leading candidate for the cognitive core and security-sensitive services. | Memory safety, systems ecosystem and hardware interoperability. | Prototype, dependency, platform, license and performance evaluation. |
| PROP-002 | Use C only where existing kernel, firmware, vendor or driver interfaces require it. | Interoperability with established low-level ecosystems. | Boundary design and unsafe-code review. |
| PROP-003 | Use Python for early research and experiments, not as a mandatory privileged runtime. | Rapid iteration and solver integration. | Prototype comparison and deployment evaluation. |
| PROP-004 | Begin on an existing Linux-based host OS. | Hardware support and shorter path to a useful system. | Initial target hardware and OS integration spike. |
| PROP-005 | Use SQLite or another local transactional store for the first knowledge prototype. | Local persistence and inspectability. | Schema and query benchmark. |
| PROP-006 | Investigate Datalog, ASP and SMT as reasoning components. | Explicit rules and constraints without neural cognition. | Representative tasks and solver comparison; begin with one Datalog/rule-evaluation path only. |
| PROP-007 | Use a device capability broker separate from the reasoning engine. | Privilege separation and independent action enforcement. | Threat model and mock-device prototype. |

## Open questions

| ID | Question | Why it matters |
|---|---|---|
| OPEN-001 | Which hardware is the first supported target? | Controls the initial device and driver scope. |
| OPEN-002 | Which first domain should demonstrate intelligence? | Defines initial knowledge, ontology and tests. |
| OPEN-003 | What interaction mode is essential at launch? | Determines the first parser and interface. |
| OPEN-004 | What bounded command grammar and semantic representation should the initial symbolic interface support? | Keeps intent interpretation non-neural and testable. |
| OPEN-005 | Which modalities and locally hosted frozen perception models are justified for the first NPB profile? | Determines minimal sensory scope and pinned runtime/artifact requirements. |
| OPEN-006 | Which existing host OS is preferred? | Determines service, device and permission integration. |
| OPEN-007 | Is a dedicated distribution sufficient, or is a custom kernel ultimately required? | Major cost and maintenance decision. |
| OPEN-008 | Which physical actions should be supported, and which should be permanently excluded? | Required for safety architecture. |
| OPEN-009 | Which user policies may pre-authorize repeated low-risk actions? | Balances convenience and authority. |
| OPEN-010 | What latency/resource budgets and reproducibility levels are acceptable for the first symbolic and perception profiles? | Avoids conflating perception repeatability with symbolic determinism. |

## Change control

A decision may be changed only with:
1. A recorded rationale.
2. Evidence or a clearly identified owner preference.
3. A note about affected requirements and documents.
4. Updated tests or acceptance criteria if implementation exists.
5. A dated entry preserving the prior decision.

Never silently replace an approved decision.
