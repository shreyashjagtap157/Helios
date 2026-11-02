# Helios Decision Register

This register tracks choices that govern the project. The master specification remains authoritative for requirements.

## Approved decisions

| ID | Decision | Rationale | Status |
|---|---|---|---|
| DEC-001 | Helios is a personal cognitive framework, not a programming language. | Establishes project identity and scope. | Approved |
| DEC-002 | Helios is currently in concept/specification phase only. No implementation has started. | Corrects historical repository descriptions and prevents false progress claims. | Approved |
| DEC-003 | No model training of any kind is permitted. | Core design constraint. | Approved |
| DEC-004 | Pretrained models are also excluded, including inference-only use. | Avoids dependence on trained models, not just training infrastructure. | Approved |
| DEC-005 | Knowledge acquisition and explicit knowledge revision are permitted; model-parameter learning is not. | Preserves the ability to acquire information without training. | Approved |
| DEC-006 | Hardware interaction is a first-class target capability. | Helios must be able to perform authorized user-requested operations. | Approved |
| DEC-007 | A broader operating-system ecosystem is a long-term vision. | Hardware and system control require deeper integration over time. | Approved |
| DEC-008 | Initial work should use existing, mature programming languages and host operating systems. | Avoids depending on an unimplemented language or premature custom kernel. | Approved |
| DEC-009 | The repository is a design-only project until real implementation is explicitly started and evidenced. | Prevents scaffolding from being represented as working software. | Approved |
| DEC-010 | The non-training constraint prohibits model inference as well as training, including local and remote services. | Prevents a pretrained or hosted model from entering the system under an inference-only or optional-dependency label. | Approved |
| DEC-011 | Every dependency and supported build must undergo non-training conformance review, with automated artifact and integration checks where feasible. | Makes the non-training rule enforceable across the software supply chain rather than relying on developer intent. | Approved |

## Proposed decisions

| ID | Proposal | Reason | Required evidence |
|---|---|---|---|
| PROP-001 | Use Rust as the leading candidate for the cognitive core and security-sensitive services. | Memory safety, systems ecosystem and hardware interoperability. | Prototype, dependency, platform, license and performance evaluation. |
| PROP-002 | Use C only where existing kernel, firmware, vendor or driver interfaces require it. | Interoperability with established low-level ecosystems. | Boundary design and unsafe-code review. |
| PROP-003 | Use Python for early research and experiments, not as a mandatory privileged runtime. | Rapid iteration and solver integration. | Prototype comparison and deployment evaluation. |
| PROP-004 | Begin on an existing Linux-based host OS. | Hardware support and shorter path to a useful system. | Initial target hardware and OS integration spike. |
| PROP-005 | Use SQLite or another local transactional store for the first knowledge prototype. | Local persistence and inspectability. | Schema and query benchmark. |
| PROP-006 | Investigate Datalog, ASP and SMT as reasoning components. | Explicit rules and constraints without trained models. | Representative tasks and solver comparison. |
| PROP-007 | Use a device capability broker separate from the reasoning engine. | Privilege separation and independent action enforcement. | Threat model and mock-device prototype. |

## Open questions

| ID | Question | Why it matters |
|---|---|---|
| OPEN-001 | Which hardware is the first supported target? | Controls the initial device and driver scope. |
| OPEN-002 | Which first domain should demonstrate intelligence? | Defines initial knowledge, ontology and tests. |
| OPEN-003 | What interaction mode is essential at launch? | Determines the first parser and interface. |
| OPEN-004 | How broad must non-trained natural-language understanding be? | May be the most difficult capability constraint. |
| OPEN-005 | Which modalities are required initially? | Non-trained vision and speech are substantially harder than structured text. |
| OPEN-006 | Which existing host OS is preferred? | Determines service, device and permission integration. |
| OPEN-007 | Is a dedicated distribution sufficient, or is a custom kernel ultimately required? | Major cost and maintenance decision. |
| OPEN-008 | Which physical actions should be supported, and which should be permanently excluded? | Required for safety architecture. |
| OPEN-009 | Which user policies may pre-authorize repeated low-risk actions? | Balances convenience and authority. |
| OPEN-010 | What are acceptable response latency and resource budgets? | Drives solver and runtime choices. |

## Change control

A decision may be changed only with:
1. A recorded rationale.
2. Evidence or a clearly identified owner preference.
3. A note about affected requirements and documents.
4. Updated tests or acceptance criteria if implementation exists.
5. A dated entry preserving the prior decision.

Never silently replace an approved decision.
