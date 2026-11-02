# HELIOS — MASTER PROJECT SPECIFICATION

**Document status:** Authoritative design baseline  
**Project phase:** Concept and specification only — pre-implementation  
**Intended owner:** Personal project  
**Last updated:** 2026-11-02  
**Normative authority:** This document is the sole source of truth for Helios requirements and architecture.  
**Implementation status:** No Helios implementation has begun. No code, prototype, subsystem, operating system, or hardware integration is accepted as implemented merely because an earlier repository contained a similarly named directory, archive, experiment, or design sketch.

---

## 1. Executive definition

Helios is a personal, non-training cognitive computing framework intended to provide useful intelligence through explicit computational mechanisms rather than trained machine-learning models.

Its intended capabilities include acquiring and organizing knowledge; representing entities, concepts, facts, events, relationships, rules, procedures and uncertainty; reasoning over that information; remembering prior experiences; planning; solving problems; evaluating its own conclusions; improving its explicit knowledge and procedures; and carrying out user-authorized tasks.

Helios is also intended to interact with the physical and digital environment through controlled hardware and software interfaces. The longer-term vision includes an operating-system ecosystem in which Helios can manage resources, communicate with devices, run services and applications, and safely execute actions requested by its user.

Helios is not defined as a chatbot, an LLM wrapper, a collection of agents, a knowledge-graph front end, or an autonomous shell script. Those may be interfaces or integration options, but they do not constitute the cognitive architecture.

The core ambition is to investigate how far a useful, extensible intelligence system can be built with algorithms, explicit representations, symbolic reasoning, search, planning, constraint solving, persistent knowledge and verified action mechanisms—without training any model.

## 2. Foundational commitments

The following commitments govern all future design decisions.

### 2.1 Strict non-training constraint

Helios must not depend on training anything. This is a defining design constraint, not an optimization preference.

The following are out of scope for the intended architecture:

- Training, pretraining, fine-tuning or distilling neural networks.
- Supervised, unsupervised, self-supervised or reinforcement-learning training.
- Online or continual training of model parameters.
- Downloading or relying on pretrained neural models, even when used only for inference.
- LLM, transformer, diffusion, learned speech, learned vision or learned embedding dependencies.
- Vector embeddings produced by trained models, learned rerankers, model-generated classifiers or other hidden learned components.
- Any inference of a trained or pretrained model, whether local or remote, required or optional, or invoked indirectly by a third-party component.
- External AI or model-inference APIs and hosted model services as any part of a Helios workflow.
- Training datasets, weight files, model checkpoints, learned embedding assets or model-serving infrastructure.
- Quietly introducing a trained model behind a library, plugin, operating-system service or device adapter.

This prohibition applies to third-party components, plugins, operating-system services, vendor SDKs and externally hosted services as well as components written specifically for Helios. A dependency is not acceptable merely because the project itself did not train the model. A component that can train or infer a model is not acceptable simply because its model path is optional or disabled by default; any such functionality must be excluded from the supported build and its absence demonstrated by dependency and artifact evidence. If the behavior of a dependency cannot be established, the dependency must be rejected or deferred.

### 2.2 What remains permitted

The prohibition on training does not prohibit the acquisition of new information or the addition of knowledge.

Permitted mechanisms include:

- Deterministic algorithms and hand-designed procedures.
- Human-authored rules, ontologies, schemas and domain models.
- Explicit facts and observations acquired from approved sources.
- Parsing and converting documents, messages, structured data and device observations into explicit representations.
- Logical inference, search, theorem proving within supported fragments, constraint solving and planning.
- Recording experiences and action outcomes as auditable events.
- Adding, correcting, versioning or retracting explicit knowledge.
- Human-reviewed changes to rules, procedures, preferences and domain knowledge.
- Bounded symbolic skill composition from approved primitives, subject to verification and authorization.
- Explicit statistical calculations over supplied data where the algorithm and assumptions are known and no learned model is fitted.

These activities may be described as knowledge acquisition, memory formation, knowledge revision or procedural adaptation. They must not silently turn into parameter training or model fitting.

### 2.3 No artificial capability claims

A non-training architecture will not automatically reproduce the language comprehension, perception, common-sense breadth or generalization of modern trained models. Every capability must have a defined representation, algorithm, knowledge requirement, operating boundary and test method.

Helios must prefer a correct limitation or a clarification question over invented understanding.

### 2.4 Personal-use orientation

The initial requirements are optimized for one person's use, control, privacy and ability to inspect the system. Multi-user operation, hosted service operation and commercial-scale deployment are not foundational requirements unless separately approved.

### 2.5 Local-first direction

Local execution and local persistence are the preferred defaults. Offline operation is a target capability. Network services may be optional information sources or synchronization mechanisms, but the cognitive core must not require a cloud service to function.

### 2.6 Hardware and action capability

Helios must be designed to interact with hardware and software resources when explicitly asked by its user. This includes discovering available devices, understanding their declared capabilities, planning actions, obtaining authorization, issuing bounded commands, observing results and recording what occurred.

Hardware access must be controlled by an enforcement layer outside the reasoning engine. A reasoning conclusion is not itself permission to operate a device.

### 2.7 Non-training dependency admission

The non-training constraint applies to the complete supported product, not only to code written in this repository. Before a dependency, plugin, SDK, system service, external service or build tool is admitted, its relevant behavior must be assessed across direct, transitive, optional, build-time and runtime paths.

The admission review must establish:

- Exact component identity, version, source, license and maintenance status.
- Direct and transitive dependency graph, including activated feature flags and optional components.
- Whether it can train, fine-tune, fit, update or infer any trained or pretrained model.
- Whether it downloads, loads or ships model weights, checkpoints, learned embeddings or training datasets.
- Whether it invokes remote model services or sends data to model-inference endpoints.
- Build scripts, runtime hooks, plugins and operating-system integrations that may introduce hidden behavior.
- Network destinations and whether the component remains functional offline.

A dependency must be rejected or deferred when its relevant behavior cannot be established. A default-off model feature is not sufficient evidence of compliance. A feature may be considered only when it is excluded from supported builds and its absence is verified in the resolved dependency graph and built artifacts.

When implementation begins, the project should generate a software bill of materials (SBOM) for each build or release and add automated checks for prohibited model artifacts, unexpected model-service endpoints and disallowed dependency features. Automated scans are defense-in-depth checks, not proof by themselves; dependency review and test evidence remain mandatory. No build or release may contain model weights, checkpoints, training datasets or learned embedding assets.

## 3. Project boundaries and terminology

### 3.1 Helios is the cognitive framework

Helios owns the definitions of cognition, knowledge, memory, reasoning, planning, action policy, user interaction and its own persistent state.

### 3.2 Helios is intended to grow into an ecosystem

The long-term ecosystem may contain:

1. A cognitive runtime.
2. A persistent knowledge and memory service.
3. A planning and execution service.
4. A trusted device and hardware-control broker.
5. User-facing interfaces.
6. An application and plugin environment.
7. An operating-system integration layer.
8. Potentially, a dedicated operating-system distribution or a system built on a microkernel or other kernel foundation.

These are architectural targets, not implemented components.

### 3.3 The operating-system boundary

Helios should initially be designed to run on an existing, mature operating system. It should not begin by writing a kernel, bootloader, filesystem, driver stack and desktop environment simultaneously.

The eventual ecosystem may provide a customized operating-system distribution or more deeply integrated operating environment. A kernel built from scratch is an unresolved option, not a present requirement.

### 3.4 Implementation languages

No new programming language is required for Helios. Implementation should use established, currently available languages and mature libraries. Language selection must be evidence-based and may vary by subsystem.

The architecture must not assume one language is suitable for kernel code, hardware drivers, cognitive algorithms, user interfaces, scripts and tools alike.

## 4. Intelligence model

Helios should be evaluated by what it can reliably do, not by the label assigned to it.

The intended capabilities are:

- Knowledge acquisition and organization.
- Entity and concept recognition within supported domains.
- Relationship and structure discovery using explicit algorithms.
- Deductive reasoning.
- Inductive generalization only where a bounded, explicit method exists; no learned model fitting.
- Abductive hypothesis generation.
- Causal and counterfactual reasoning where a causal model is available.
- Temporal and spatial reasoning where the required representations exist.
- Mathematical and constraint-based problem solving.
- Analogical reasoning over represented structures.
- Search and planning over explicit state/action models.
- Procedural knowledge and repeatable skill execution.
- Persistent episodic, semantic and procedural memory.
- Self-monitoring, contradiction detection and uncertainty reporting.
- Controlled execution of user-authorized actions.
- Learning in the sense of adding and revising explicit knowledge and procedures, not training models.

A capability is not complete merely because a module exists. Each must have supported inputs, outputs, semantics, failure modes, complexity limits, persistence behavior, security boundaries and acceptance tests.

## 5. Proposed cognitive architecture

The following architecture is the proposed design baseline. Internal interfaces and algorithms remain subject to research and validation.

### 5.1 Cognitive executive

The executive coordinates the cognitive cycle. It receives requests, determines the active goal, selects relevant context, allocates bounded resources, invokes reasoning and planning services, and decides whether to answer, ask, defer or execute.

Responsibilities:

- Maintain the current interaction and task state.
- Decompose user goals into explicit subgoals.
- Select relevant knowledge and memory.
- Choose an appropriate reasoning method.
- Track dependencies and completion conditions.
- Enforce time, memory, search-depth and tool-use budgets.
- Track pending questions and unresolved assumptions.
- Stop, suspend or request authorization when required.
- Keep a trace of why a particular operation was selected.

The executive must not be an unrestricted autonomous process. It is a coordinator operating under user policy and an independent action-control boundary.

### 5.2 Input and acquisition subsystem

The acquisition subsystem accepts information from explicit sources:

- User text and structured commands.
- Local documents and files.
- Databases and structured datasets.
- Approved web and network sources.
- Application interfaces and APIs.
- Device telemetry and sensor readings.
- Events generated by Helios itself.
- Explicitly configured external knowledge sources.

Each connector must declare its source identity, permissions, supported formats, data handling, failure semantics and trust level.

Acquisition must preserve raw source material when practical and separately store extracted claims. Extracted facts must never be presented as if they were directly verified when the extraction process is uncertain.

### 5.3 Knowledge system

The knowledge system stores explicit, inspectable information. It must support:

- Entities and typed identifiers.
- Concepts and taxonomies.
- Attributes and values.
- Relations and relation constraints.
- Facts and propositions.
- Rules and inference premises.
- Events and observations.
- Temporal validity and event time.
- Spatial relationships where relevant.
- Causal relationships with provenance.
- Procedures, actions and preconditions.
- Hypotheses, assumptions and unresolved claims.
- Sources, authorship and derivation.
- Confidence or evidence assessments with explicit rationale.
- Contradictions and competing claims.
- Version history, correction and retraction.
- Access policy and sensitivity labels.

A knowledge item must identify whether it is observed, imported, user-provided, assumed, inferred, proposed or externally verified. These categories must not be collapsed into one generic fact type.

### 5.4 Memory system

Memory must be more than a list of messages or a database of text.

Required memory categories:

- **Working memory:** information relevant to the active task, bounded by explicit capacity and lifetime.
- **Episodic memory:** records of events, interactions, actions, results and their context.
- **Semantic memory:** durable concepts, facts, entities and relationships.
- **Procedural memory:** explicit procedures, routines, skills and action plans.
- **Autobiographical/project memory:** durable records of Helios's own configured identity, user-approved preferences, project history and prior decisions.
- **Prospective memory:** reminders, deadlines, pending obligations and planned future actions.

Memory lifecycle operations must include creation, retrieval, linking, consolidation, archival, correction, expiration and deletion according to policy.

Memory consolidation must not silently discard evidence or rewrite history. It may create summaries or indexes, but the original event or source record should remain addressable when retention policy permits.

### 5.5 Reasoning system

Reasoning should be a portfolio of explicit mechanisms, not one opaque universal reasoner.

Candidate mechanisms:

- Rule-based deduction.
- Forward chaining.
- Backward chaining and goal-directed proof search.
- Datalog-style relational inference.
- First-order logic fragments with explicit decidability and resource limits.
- Answer-set programming for suitable combinatorial problems.
- Constraint satisfaction and optimization.
- SAT and SMT solving.
- Graph traversal and graph algorithms.
- Search algorithms, including heuristic search.
- Planning over explicit states, actions, preconditions and effects.
- Temporal reasoning.
- Spatial and geometric reasoning.
- Causal models and intervention analysis where causal structure is known.
- Abductive reasoning to generate possible explanations.
- Analogical reasoning over explicit mappings.
- Mathematical computation and symbolic algebra.
- Probabilistic calculations with declared assumptions and known limitations.

No method should be used outside its supported semantics without marking the result as heuristic or speculative.

### 5.6 Metacognition and epistemic control

Helios must track not only conclusions but also the status of its own knowledge.

It should distinguish:

- Known from unknown.
- Observed from inferred.
- Proven from plausible.
- Current from stale.
- Consistent from contradictory.
- Supported from unsupported.
- High-confidence from low-confidence.
- A completed action from an attempted action.
- A plan from an execution.
- A simulation from a real-world result.

For a conclusion, Helios should preserve the relevant premises, source references, rules or algorithms used, intermediate derivations, unresolved assumptions and any solver result or proof artifact available.

Confidence must not be an arbitrary scalar that creates an illusion of statistical calibration. Its meaning, derivation and permitted uses must be specified. When there is no principled probability model, use categorical evidence states or explicit uncertainty reasons rather than fabricated precision.

### 5.7 Planning and problem-solving system

Tasks should be represented as goals and executable plans.

A plan should identify:

- Desired final state.
- Initial state and source of that state.
- Actions and their preconditions.
- Expected effects.
- Dependencies and ordering constraints.
- Required knowledge and resources.
- Risks and safety class.
- Reversibility and compensation steps.
- Time and resource budgets.
- Verification conditions.
- Abort and recovery behavior.

Planning methods may include classical planning, hierarchical task decomposition, partial-order planning, graph search and constraint-based scheduling. Each planning engine must state which classes of problems it supports.

### 5.8 Interaction and language subsystem

Natural-language interaction without trained models is a major research challenge and must be treated honestly.

Candidate components:

- Tokenization and normalization.
- Hand-authored lexical resources.
- Grammar-based parsing for defined language subsets.
- Semantic parsing into explicit task and knowledge representations.
- Entity resolution against the knowledge base.
- Dialogue state and context tracking.
- Clarification questions for ambiguity.
- Template- and grammar-based natural-language generation.
- Structured command interfaces as a precise alternative to unrestricted natural language.

The initial system should prefer a reliable bounded language and a structured command interface over claiming arbitrary human-level language comprehension.

### 5.9 Perception and multiple modalities

The framework should be designed for multiple input modalities, but modality support must be staged.

Text, structured records, event streams and device telemetry are appropriate early targets. Image, audio and video processing without trained models will require research into deterministic signal processing, classical computer vision, hand-designed feature extraction, explicit grammars and domain-constrained algorithms.

No general-purpose visual recognition, speech recognition or image understanding capability should be claimed without measured evidence on representative data.

## 6. The non-training learning model

Helios must be capable of acquiring new knowledge without training a model.

The permitted cycle is:

1. Receive an observation, document, user correction or task outcome.
2. Preserve source and context.
3. Parse it into candidate entities, claims, relations, events or procedures.
4. Check types, constraints, provenance and source credibility.
5. Compare with existing knowledge.
6. Identify conflicts, novelty, uncertainty and possible consequences.
7. Derive candidate conclusions only through declared reasoning mechanisms.
8. Request human review when required by policy or ambiguity.
9. Add, correct, link or retract explicit knowledge.
10. Record the complete history and reason for the change.
11. Make the resulting knowledge available to later reasoning and planning.

This is explicit knowledge acquisition and revision. It does not create or optimize neural weights, embeddings, latent representations or other trained parameters.

Automatic rule creation is high risk. It must be bounded by a formal rule language, provenance, safety constraints, review policy, regression tests and rollback. Helios must not silently convert an observed correlation into a causal rule.

## 7. Knowledge integrity and evidence model

Every claim should have a stable identifier and a structured record containing, where applicable:

- Claim identifier.
- Subject, predicate and object or proposition.
- Type and domain.
- Source identifier and source location.
- Observation or publication time.
- Ingestion time.
- Valid-time interval.
- Author or asserting party.
- Acquisition method.
- Evidence references.
- Derivation references.
- Confidence/evidence category and rationale.
- Status: candidate, accepted, disputed, superseded, retracted or rejected.
- Sensitivity and access classification.
- Version and change history.

Contradictions must be representable. A newer source must not automatically overwrite an older claim. The system must be able to preserve both claims, explain the difference and ask for adjudication when evidence does not resolve the conflict.

Derived conclusions should link to their premises. If a premise is retracted or changed, dependent conclusions must be marked stale and scheduled for recomputation or review.

## 8. Hardware interaction and action execution

Hardware interaction is a first-class architectural requirement.

### 8.1 Device discovery and inventory

The system should eventually discover and maintain an inventory of devices that the host operating environment exposes.

Potential device classes include:

- CPU, memory, storage and system power interfaces.
- Display, keyboard, mouse, touch and audio devices.
- USB, serial and other peripheral interfaces.
- Network adapters and network-connected devices.
- Bluetooth devices.
- Sensors such as temperature, motion, position, pressure and environmental sensors.
- Cameras and microphones.
- Microcontrollers and development boards.
- Robotics and automation controllers.
- Laboratory, measurement and testing equipment.
- Home and building automation devices.

Actual support depends on hardware, operating-system interfaces, firmware, drivers and vendor protocols. A device appearing in an inventory does not imply that it is safe or controllable.

### 8.2 Capability-based device model

Each device adapter must declare:

- Device identity and authenticity evidence where available.
- Device class and supported operations.
- Read-only and write capabilities.
- Permitted parameter ranges.
- Preconditions and required state.
- Expected result and observable feedback.
- Maximum operation duration and resource use.
- Reversibility and compensation behavior.
- Failure and timeout behavior.
- Safety classification.
- Required user authorization.
- Driver, firmware and protocol versions.
- Audit events and diagnostic information.

The cognitive engine should request typed capabilities rather than directly access arbitrary memory, ports, registers, files or system calls.

### 8.3 Action lifecycle

Every consequential action should follow a controlled lifecycle:

1. **Interpret:** determine what the user requested and identify ambiguity.
2. **Inspect:** discover the target and current state through authorized read operations.
3. **Plan:** form a plan with explicit preconditions and expected effects.
4. **Assess:** classify risk, reversibility, authority and resource requirements.
5. **Preview:** show the intended action and expected consequences where practical.
6. **Authorize:** obtain required confirmation or verify a previously granted narrow policy.
7. **Prepare:** verify device state, capability validity and safety conditions.
8. **Execute:** issue only the approved, bounded operation.
9. **Observe:** collect independent feedback where available.
10. **Verify:** compare actual state against expected postconditions.
11. **Record:** persist request, plan, authorization, commands, results and errors.
12. **Recover:** stop, compensate or enter a safe state when execution fails or becomes uncertain.

A command being sent is not proof that an action succeeded.

### 8.4 Safety classes

At minimum, classify actions as:

- **Class 0 — Information only:** read-only queries and status inspection.
- **Class 1 — Low-impact local changes:** reversible, bounded changes with no external consequence.
- **Class 2 — Consequential digital actions:** file changes, software installation, network configuration, communications, account or resource changes.
- **Class 3 — Physical or safety-relevant actions:** movement, heating, power switching, mechanical control, robotics, laboratory equipment or other actions with physical consequences.
- **Class 4 — Prohibited or unsupported actions:** actions that violate policy, exceed device guarantees, bypass safety controls or cannot be made sufficiently safe.

Higher classes require stronger authorization, checks and independent enforcement. Class 3 should default to explicit confirmation per operation until a narrowly scoped, tested policy is approved. Class 4 must be refused.

### 8.5 Independent enforcement

The cognitive engine must not be the sole safety boundary.

A separate action broker or trusted system service must enforce:

- Least privilege.
- Device and operation allowlists.
- Capability expiry and revocation.
- Parameter and rate limits.
- Resource and time budgets.
- User authorization.
- Interlocks and device safety limits.
- Fail-closed handling when authorization or state is uncertain.
- Emergency stop and human override.
- Audit logging that the reasoning engine cannot silently alter.

Never bypass manufacturer safety controls, hardware interlocks, operating-system permissions or physical emergency mechanisms.

### 8.6 Device simulation and hardware-in-the-loop

Before controlling a real device, test the adapter in simulation or against a mock device. Then test on isolated, non-hazardous hardware. Hardware-in-the-loop testing must validate timing, fault handling, disconnection, malformed responses, repeated commands and recovery.

## 9. Operating-system ecosystem

The long-term vision is a Helios-centered operating environment. This does not require a new kernel in the first milestone.

### 9.1 Layered architecture

The intended ecosystem has these conceptual layers:

1. **Firmware and boot:** existing firmware, boot process, trusted boot configuration and hardware initialization.
2. **Kernel:** scheduling, virtual memory, process/thread isolation, interrupt management, device access control and resource primitives.
3. **Hardware abstraction:** stable device descriptors, driver boundaries and typed hardware capabilities.
4. **Core services:** storage, filesystem, networking, time, identity, logging, package management, IPC and update services.
5. **Security and policy services:** identities, permissions, capability broker, secrets and audit.
6. **Helios runtime:** cognition, knowledge, memory, reasoning, planning and action coordination.
7. **Application and plugin environment:** tools, interfaces, extensions and user workflows.
8. **User interface:** text, graphical, voice and other interfaces as their non-training feasibility is demonstrated.

### 9.2 Recommended development sequence

First integrate Helios with an existing operating system and hardware APIs. This gives access to working drivers, filesystems, networking, process isolation and device management while the cognitive architecture is developed.

Next, create a Helios system service layer with a narrow, audited hardware broker. Build a dedicated user environment or distribution only after the service architecture is useful.

Only then evaluate a custom kernel or microkernel. Any kernel decision must be justified by measurable needs for isolation, safety, security, resource control, determinism or hardware support that cannot be met adequately by the host system.

### 9.3 OS design alternatives

The following are candidates for research, not final selections:

- Linux-based integration or a dedicated distribution: strongest initial hardware and software ecosystem, fastest route to practical use.
- A microkernel-based environment: potentially useful for isolating drivers and services; requires careful driver and platform work.
- A custom kernel: maximum architectural control but very high cost, hardware coverage burden and security responsibility.
- Existing embedded operating systems for constrained device nodes, connected to Helios through explicit protocols.

seL4 is an important high-assurance reference because its documentation describes a capability-based microkernel and specifies which configurations have particular verification evidence. Its device drivers generally execute in user mode, which is relevant to the separation between kernel and hardware-control services. These properties do not make it an automatic choice for a personal desktop ecosystem.

### 9.4 OS ecosystem requirements

The future environment must address:

- Boot and recovery.
- Hardware inventory and drivers.
- Memory and process isolation.
- Filesystem and durable storage.
- Networking and secure communication.
- User identity and authentication.
- Permissions and capability delegation.
- Device access broker.
- Application and plugin lifecycle.
- Package installation and update.
- Secure configuration and secrets.
- Crash recovery and backups.
- Logs, metrics and event history.
- Accessibility and user interaction.
- Compatibility with existing software and devices.

## 10. Programming-language and implementation strategy

Language selection is intentionally open until representative prototypes have been evaluated.

Candidate direction:

| Area | Candidate language or technology | Reason to evaluate |
|---|---|---|
| Cognitive core and security-sensitive services | Rust | Strong memory-safety model, mature libraries and systems use |
| OS and driver interoperability | C plus Rust interfaces | Existing kernel and hardware ecosystems still contain substantial C surfaces |
| Early algorithm experiments | Python | Rapid prototyping, solver integration and test generation |
| User interface | TypeScript or another mature UI stack | Established desktop/web interface ecosystem |
| Persistence | SQLite initially; later evaluate embedded or service databases | Local transactional persistence and inspectable data |
| Declarative reasoning | Datalog, ASP or a constrained logic language | Explicit facts, relations and rules |
| Constraints | SAT/SMT and constraint solvers | Formalized feasibility and arithmetic constraints |
| Planning | PDDL/HTN-inspired or custom typed planning models | Explicit actions, preconditions and effects |

No technology is accepted solely because it is popular. Evaluation must cover licensing, maintenance, platform support, safety, performance, interoperability, reproducibility, offline behavior, supply-chain risk and long-term sustainability.

Rust is a serious candidate for the core and systems integration. Official Linux kernel documentation describes a constrained Rust environment using core without the standard library; the Embedded Rust ecosystem documents work across embedded Linux, RTOS and bare-metal environments. This supports investigation, not a claim that every driver or kernel subsystem is already available in Rust.

C should remain an interoperability option where vendor SDKs, kernel interfaces, firmware and existing drivers require it. Any unsafe boundary must be small, reviewed and tested.

## 11. Security, safety and privacy

Security is architectural, not an optional feature.

Required design principles:

- Default deny for unrequested hardware and system access.
- Least privilege and narrow capabilities.
- Separate cognitive reasoning from privileged execution.
- Separate device drivers from the user-facing cognitive process.
- Authenticate device and service endpoints where supported.
- Protect credentials and secrets.
- Encrypt sensitive data at rest and in transit where appropriate.
- Maintain tamper-evident or append-only audit records where feasible.
- Make permission grants visible, scoped and revocable.
- Sandbox plugins and untrusted parsers.
- Validate external data and commands.
- Prevent prompt-like content in external documents from acquiring authority.
- Treat all imported content as data, never as instructions to override user policy.
- Provide backup, restore, recovery and deletion mechanisms.
- Minimize personal data collection.
- Keep local data under the user's control.
- Define safe shutdown and emergency-stop behavior.

A reasoning system must never be permitted to grant itself new authority. Policy changes require explicit authorization through a trusted control path.

## 12. Reliability and failure semantics

Every subsystem must document its failure behavior.

Examples include:

- Missing or unavailable knowledge source.
- Contradictory claims.
- Invalid or malformed input.
- Unsupported language construction.
- Incomplete proof search.
- Solver timeout or resource exhaustion.
- Planner failure or unsatisfied goal.
- Stale sensor reading.
- Device disconnect during action.
- Action timeout or partial execution.
- Uncertain postcondition.
- Storage corruption or failed migration.
- Plugin crash or privilege violation.
- Network partition.
- Clock uncertainty.
- Power loss and restart.

Helios must distinguish failure, timeout, unsupported input, unknown state and partial completion. It must not report success where only a request or command was issued.

## 13. Observability and explainability

The project must provide inspectable records of:

- User request and interpreted intent.
- Ambiguities and assumptions.
- Retrieved facts and evidence.
- Reasoning rules and solver methods.
- Intermediate conclusions.
- Plan and dependency graph.
- Policy decision and authorization.
- Device operations and parameters.
- Actual observations and postcondition checks.
- Errors, retries, recovery and final status.

Sensitive values must be redacted according to policy. Auditability must not require exposing passwords, secrets or unrelated private content.

## 14. Data architecture and persistence

The design should separate at least:

- Raw source archive.
- Normalized knowledge records.
- Entity and relation indexes.
- Rule and ontology definitions.
- Episodic event log.
- Procedural plans and validated procedures.
- User configuration and permissions.
- Device inventory and adapter metadata.
- Task state and execution records.
- Audit log.
- Cache and derived indexes.

Data schemas require stable IDs, versions, migrations, backup/restore and integrity checks. A derived index must be rebuildable from authoritative records or explicitly classified as irreplaceable data.

A relational store is a reasonable initial baseline for transactional records. Graph and logic indexes may be layered on top. Do not select a graph database merely because the architecture contains relationships.

## 15. Plugin and extension model

Plugins may add knowledge sources, parsers, reasoning methods, planners, user interfaces, device adapters and application integrations.

Every plugin must declare:

- Identity and version.
- Publisher and signature or integrity mechanism.
- Supported interfaces and compatibility.
- Permissions and capabilities requested.
- Data it can access.
- Network and filesystem access.
- Device access, if any.
- Resource budgets.
- Failure isolation.
- Upgrade and rollback policy.
- Tests and conformance evidence.

Plugins must not receive unrestricted authority by default. A plugin cannot modify the trusted policy engine or grant itself capabilities.

## 16. Evaluation and acceptance

Helios must be assessed by reproducible task suites, not subjective impressions.

Evaluation dimensions:

- Conformance with the non-training dependency policy across direct and transitive dependencies.
- Absence of prohibited model assets and model-inference network paths in supported builds.
- Logical validity of deductions.
- Correctness and provenance of retrieved facts.
- Contradiction detection.
- Accuracy of entity resolution in declared domains.
- Planning feasibility and constraint satisfaction.
- Ability to identify missing information.
- Appropriate abstention and clarification.
- Memory retrieval and correction behavior.
- Reproducibility of reasoning traces.
- Device-command authorization correctness.
- Hardware action success and postcondition verification.
- Failure handling and recovery.
- Resource consumption and latency.
- Privacy and permission enforcement.
- Offline operation.
- Compatibility and upgrade behavior.

Each benchmark must define inputs, expected outputs or invariants, environment, resource limits, scoring and known limitations. Test fixtures must not be treated as evidence of unrestricted general intelligence.

## 17. Initial implementation philosophy

The first implementation should prove a narrow vertical slice on existing operating systems:

1. A local Helios process starts and accepts a structured request.
2. It stores explicit facts with source and timestamp.
3. It answers a bounded logical query with a traceable derivation.
4. It stores an event in episodic memory.
5. It creates a small plan over a typed action model.
6. It requests permission for a harmless, reversible action.
7. It invokes a mock device adapter.
8. It verifies the mock result and records the execution.
9. It survives restart with its knowledge and audit history intact.
10. It runs without model training, pretrained models or network AI services.

This is the first meaningful acceptance milestone. It is not a complete intelligence system and must not be described as one.

## 18. Research requirements before major implementation

Research and record evidence for:

- Symbolic AI architectures and limitations.
- Datalog, rule engines and deductive databases.
- Answer-set programming.
- SAT/SMT solving and proof-producing workflows.
- Automated planning, PDDL, hierarchical planning and scheduling.
- Ontologies, knowledge graphs and temporal databases.
- Truth-maintenance and belief-revision systems.
- Causal models, intervention and counterfactual reasoning.
- Analogical reasoning.
- Classical computer vision and non-trained signal processing.
- Natural-language parsing and generation without neural models.
- Memory architectures and event sourcing.
- Agent/action planning with independent policy enforcement.
- Capability-based security and sandboxing.
- OS architectures: monolithic, hybrid and microkernel.
- Existing kernels and operating systems with hardware coverage.
- Rust, C and other candidate languages for OS and device work.
- Driver architecture, IOMMU/DMA boundaries, device identity and firmware.
- Hardware-in-the-loop testing and safety engineering.
- Local storage, cryptographic integrity, backup and recovery.

For each topic, record sources, date, assumptions, candidate technologies, alternatives, limitations, licensing and an implementation recommendation. Prefer official documentation, original research, formal specifications and maintained repositories.

## 19. Non-goals and exclusions

Unless explicitly reconsidered, Helios will not:

- Train or fine-tune machine-learning models.
- Depend on pretrained models even for inference.
- Use learned embeddings or model-based retrieval.
- Claim arbitrary human-level comprehension or perception.
- Execute arbitrary natural-language shell commands with elevated privileges.
- Control hardware without declared capabilities and authorization.
- Treat an unverified plan as a completed action.
- Treat a source's confidence score as proof.
- Treat all graph relationships as causal.
- Silently resolve contradictory evidence.
- Start by building every layer of an operating system.
- Use AI or model-inference services, whether local or remote.
- Claim formal verification, safety certification or production readiness without evidence.

## 20. Requirements register

These identifiers are stable requirement anchors. Future detailed requirements may be added without changing existing IDs.

### Project identity
- HEL-FOUND-001: Helios is a personal cognitive computing framework.
- HEL-FOUND-002: Project status must remain Concept/Design until actual implementation begins and is evidenced.
- HEL-FOUND-003: The master specification is the canonical authority.
- HEL-FOUND-004: Design and implementation must use existing, maintained programming languages and tools.

### Non-training
- HEL-NTR-001: No training of any model.
- HEL-NTR-002: No fine-tuning, pretraining, distillation or online model adaptation.
- HEL-NTR-003: No pretrained model dependency, including inference-only use.
- HEL-NTR-004: No learned embeddings, model weights or model-generated classifiers.
- HEL-NTR-005: New information is acquired as explicit, provenance-bearing knowledge.
- HEL-NTR-006: Rule and procedure changes must be explicit, auditable and governed; they must not become hidden parameter learning.
- HEL-NTR-007: No trained or pretrained model inference service, local or remote, is used by a Helios runtime or workflow.
- HEL-NTR-008: Every direct, transitive, optional, build-time and runtime dependency is reviewed for model training, inference, learned components and model assets before admission.
- HEL-NTR-009: Supported build and release artifacts contain no training datasets, model weights, checkpoints or learned embedding assets.
- HEL-NTR-010: Non-training conformance checks are automated where feasible and supplemented by recorded human dependency review.

### Cognition
- HEL-COG-001: A cognitive executive coordinates requests, goals and subsystem operations.
- HEL-COG-002: The system supports multiple explicit reasoning mechanisms.
- HEL-COG-003: Conclusions preserve premises, evidence and derivation traces.
- HEL-COG-004: The system distinguishes fact, inference, assumption, hypothesis and unknown.
- HEL-COG-005: The system can abstain or ask for clarification.
- HEL-COG-006: The system supports bounded task decomposition and planning.
- HEL-COG-007: Metacognition identifies uncertainty, conflict and stale conclusions.

### Knowledge and memory
- HEL-KNO-001: Knowledge items have stable identities, types and provenance.
- HEL-KNO-002: Contradictory claims can coexist without silent overwrite.
- HEL-KNO-003: Temporal validity and history are represented where relevant.
- HEL-KNO-004: Derived claims link to premises and can be invalidated.
- HEL-MEM-001: Working, episodic, semantic, procedural and prospective memory are distinct.
- HEL-MEM-002: Memory changes are versioned and auditable.
- HEL-MEM-003: Consolidation does not silently destroy authoritative evidence.

### Interaction and perception
- HEL-INT-001: Initial interaction supports a precise structured command path.
- HEL-INT-002: Natural-language capabilities are bounded by implemented grammars and semantic representations.
- HEL-INT-003: Ambiguous requests trigger clarification.
- HEL-INT-004: Multimodal support is staged and evidence-based.
- HEL-INT-005: No trained speech, vision or language model is required.

### Hardware and actions
- HEL-HW-001: Hardware is accessed through declared device adapters and typed capabilities.
- HEL-HW-002: Reasoning and privileged device execution are separated.
- HEL-HW-003: Consequential actions have explicit authorization.
- HEL-HW-004: Physical actions have risk classifications, interlocks and emergency-stop paths.
- HEL-HW-005: Device operations have bounded parameters, timeouts and resource limits.
- HEL-HW-006: Action completion requires verification of postconditions.
- HEL-HW-007: Hardware actions are auditable and recoverable where possible.
- HEL-HW-008: Simulation and hardware-in-the-loop tests precede hazardous real-device execution.

### Operating environment
- HEL-OS-001: Initial development runs on an existing operating system.
- HEL-OS-002: The architecture permits a future Helios-centered operating environment.
- HEL-OS-003: Kernel development is deferred pending evidence of need.
- HEL-OS-004: Drivers, services and applications have explicit isolation boundaries.
- HEL-OS-005: The OS ecosystem includes recovery, secure update, device management and application lifecycle.

### Security and governance
- HEL-SEC-001: Default-deny and least-privilege apply to system and hardware operations.
- HEL-SEC-002: Reasoning cannot grant itself authority.
- HEL-SEC-003: Plugins are permission-declared and isolated.
- HEL-SEC-004: External content is treated as untrusted data.
- HEL-SEC-005: Audit records capture consequential operations.
- HEL-SEC-006: The user can revoke grants and stop execution.

### Reliability and evidence
- HEL-REL-001: Failure and timeout states are distinct from success.
- HEL-REL-002: Unknown device state is treated as unsafe for consequential actions.
- HEL-REL-003: Storage has backup, integrity and migration procedures.
- HEL-EVD-001: Design, implementation, tests, measurements and proofs are tracked separately.
- HEL-EVD-002: No capability is called complete without reproducible acceptance evidence.

## 21. Open decisions

These are unresolved design questions, not hidden assumptions:

1. Which hardware should be the first supported target: the personal computer only, a development board, home devices, robotics, or a defined combination?
2. What initial interaction modes are essential: structured text commands, natural-language text, GUI, voice, or a staged subset?
3. Which domains should the first knowledge and reasoning demonstration support?
4. What local operating system should host the first prototype?
5. Should the eventual operating environment be a customized existing distribution, a microkernel-based system or a custom kernel?
6. What data must persist indefinitely, and what may expire?
7. Which actions, if any, may be pre-authorized under narrow user-configured policies?
8. What level of local multimodal perception is actually required without trained models?
9. Which candidate reasoning engines should be embedded, wrapped or implemented directly?
10. What are acceptable latency and resource budgets for the first use cases?

Do not block the initial design on decisions that can be evaluated experimentally. Record each choice and its evidence in the decision log.

## 22. Definition of progress

Project state must use this vocabulary:

- **Concept:** an idea has been recorded.
- **Researching:** sources and alternatives are being examined.
- **Proposed:** a design has been written but not approved.
- **Approved:** the owner has accepted the design.
- **Specified:** requirements, interfaces and acceptance criteria are sufficiently precise.
- **Implemented:** code exists and has been inspected.
- **Tested:** specified tests have been run and results recorded.
- **Validated:** evidence shows the implemented behavior meets the intended use.
- **Released:** a version has been packaged and its installation/recovery path tested.

A document, directory, archive, interface sketch or code sample is not proof that a subsystem has been implemented.

## 23. Current project truth

As of 2026-11-02:

- Helios is a concept and design effort.
- No Helios implementation has started.
- No cognitive engine is implemented.
- No knowledge or memory engine is implemented.
- No operating system or hardware broker is implemented.
- No device adapter is implemented.
- No model training or model inference is part of the intended architecture.
- No programming-language or operating-system stack has been finally selected.
- The next work is research, architecture refinement, requirements approval and a carefully scoped prototype plan.

This section must be updated only when there is concrete evidence of a state transition.

## 24. Reference starting points

These are research entry points, not automatic endorsements:

- Rust for Linux and the Linux kernel Rust documentation: https://www.kernel.org/doc/html/latest/rust/
- Rust Embedded Working Group: https://github.com/rust-embedded
- The Embedded Rust Book, no_std: https://doc.rust-lang.org/embedded-book/intro/no-std.html
- Tock embedded operating system: https://tockos.org/documentation/getting-started/
- seL4 project documentation: https://docs.sel4.systems/projects/sel4/
- seL4 capabilities: https://docs.sel4.systems/Tutorials/capabilities.html
- seL4 supported hardware and verification status: https://docs.sel4.systems/Hardware/
- Soufflé Datalog: https://souffle-lang.org/docs/home/
- Soufflé Datalog language: https://souffle-lang.org/datalog-language
- Potassco clingo answer-set programming: https://potassco.org/clingo/

Reference pages must be rechecked before implementation. Record actual versions, licenses, platform support and limitations. Do not infer formal guarantees for an entire system from a kernel or solver's guarantees.

---

**Authority statement:** This document governs Helios design. Roadmap, status and decision records are supporting project-control documents. If they conflict with this specification, this specification takes precedence until an approved revision changes it.
