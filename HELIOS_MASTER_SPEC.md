# HELIOS — MASTER PROJECT SPECIFICATION

**Document status:** Authoritative design baseline  
**Project phase:** Concept and specification only — pre-implementation  
**Intended owner:** Personal project  
**Last updated:** 2026-11-02  
**Normative authority:** This document is the sole source of truth for Helios requirements and architecture.  
**Implementation status:** No Helios implementation has begun. No code, prototype, subsystem, operating system, or hardware integration is accepted as implemented merely because an earlier repository contained a similarly named directory, archive, experiment, or design sketch.

---


## 1. Executive definition

Helios is a personal cognitive computing framework whose reasoning, planning, decisions, knowledge revision and memory governance are performed by an explicitly non-neural, deterministic symbolic core.

Frozen foundation models may be used only as quarantined perception peripherals. Their outputs are untrusted candidate observations with provenance; they are not facts, instructions, beliefs, plans, decisions or authorization. The Neural Perception Boundary (NPB) has no direct access to the symbolic executive, cognitive memory, policy engine, tools or privileged execution.

The symbolic core represents entities, claims, events, relations, rules, procedures, uncertainty and goals explicitly. It derives conclusions using declared algorithms and formal methods within published operational limits. It preserves premises, evidence, contradictions, derivations and failure states. It manages working context and temporary memory explicitly; durable cognitive retention is opt-in and separately authorized.

Helios is intended to interact with digital and physical environments through controlled software and hardware interfaces. A separate policy and action-enforcement boundary must authorize consequential operations. A reasoning conclusion or perceptual observation is never, by itself, permission to execute.

Helios is not a chatbot, an LLM wrapper, a collection of agents, a knowledge-graph front end or an autonomous shell script. Its defining intelligence mechanisms are symbolic, inspectable and reproducible. Neural inference is an optional sensory input capability, never a cognitive authority.

The project investigates how far a useful, extensible intelligence system can be built through explicit representations, symbolic reasoning, search, planning, constraint solving, controlled memory and verified action mechanisms, without training or adapting any model.

## 2. Foundational commitments

The following commitments govern all future design decisions.


### 2.1 Absolute non-training constraint and restricted perception exception

Helios must never train, pretrain, fine-tune, distill, fit, update or continually adapt neural models or other learned model parameters. This is an absolute project constraint.

Pretrained foundation-model inference is permitted only inside an approved Neural Perception Boundary (NPB), and only to transform supplied sensory or unstructured source material into typed candidate observations. This is a narrow architectural exception, not permission to use a model as a cognitive subsystem.

Neural/model-based reasoning, inference chaining, deliberation, planning, goal selection, decision-making, policy evaluation, memory management, knowledge revision, action authorization, LLM-based user-intent interpretation, dialogue orchestration, code generation, tool selection and response generation remain prohibited. Learned embeddings, latent-vector retrieval, learned rerankers and learned memory indexes are prohibited in the cognitive core. The NPB may not access cognitive memory, the executive, tools, credentials, network clients, policy services or privileged device operations.

Remote or hosted model inference is excluded from the initial supported profile. A later remote-perception proposal requires a separate decision and evidence that model version, weights, execution configuration, data handling and isolation are immutable and auditable.

Every permitted perception model is frozen as an exact reviewed deployment: model identity, publisher, version, weight digest, processor/tokenizer artifacts, preprocessing, inference configuration, runtime build, precision/quantization, backend and sandbox profile are recorded. Updates are disabled; replacing any item requires a new admission review and regression suite.

The NPB is an isolated process or equivalent enforced security domain. It receives only the input needed for the specific perception request and returns only a versioned, schema-validated observation envelope. It has no access to cognitive memory, symbolic rules, action broker, user secrets or arbitrary tools. Network access is denied in the initial profile. Model output is untrusted even when schema-valid.

A dependency that can train or infer models is not acceptable in the symbolic core merely because the relevant feature is optional or disabled. Perception-only dependencies are confined to the separately reviewed NPB dependency set. If behavior, artifacts or isolation cannot be established, they are rejected or deferred.


### 2.2 What remains permitted

Permitted mechanisms include deterministic algorithms; human-authored rules, ontologies and schemas; explicit facts and provenance-bearing observations; classical signal processing, OCR, grammar parsing and non-learned feature extraction; logical inference, theorem proving within declared fragments, search, constraint solving and planning; recording task events under scoped retention policies; and explicit, auditable knowledge revision.

Frozen foundation-model inference is permitted only within the approved NPB and solely to produce untrusted candidate observations from sensory or unstructured input. Perception is an acquisition mechanism only. A candidate becomes an operational belief or durable knowledge item only through a separate symbolic admission process that checks schema, provenance, corroboration policy, contradictions, source limitations and required user authorization.

Explicit statistical calculations are permitted where the algorithm and assumptions are known and no predictive model is fitted. These activities must not silently become model-parameter training or neural cognition.

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


### 2.7 Dependency admission and perception isolation

Every dependency, plugin, SDK, system service, external service and build tool is reviewed across direct, transitive, optional, build-time and runtime paths.

Admission establishes exact component identity/version/source/license; direct and transitive dependencies and enabled features; training/fitting/inference capability; model downloads and artifacts; runtime network destinations and telemetry; build scripts and dynamic loading; offline behavior; and whether the symbolic core remains independent of the NPB.

The symbolic core dependency set contains no neural inference facility. The NPB dependency set may contain only the exact frozen perception model and minimum reviewed runtime required to execute it. NPB admission additionally records artifact and processor digests, inference runtime/backend, precision/quantization, execution configuration, sandbox profile, input/output permissions and negative tests proving cognitive/privileged interfaces are unreachable.

A default-off feature or package-level assurance is insufficient. When implementation begins, generate an SBOM for each supported build, scan artifacts for training data/optimizer state/undeclared weights/learned indexes, inspect network destinations and test NPB isolation. Automated scans are defense-in-depth; human dependency review and runtime isolation tests remain mandatory.

The NPB is optional. If absent, unavailable or rejected, the symbolic core continues on structured inputs and reports perception unavailable rather than falling back to an unapproved model service.

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

The executive is a deterministic symbolic coordinator. It receives typed requests, determines the active goal, selects relevant knowledge and task memory, allocates bounded resources, invokes approved reasoning/planning mechanisms and decides whether to answer, ask, defer or submit an action proposal to the independent policy boundary.

It maintains explicit task state; decomposes goals using declared rules; selects task-scoped knowledge and memory; chooses reasoning from an allowlist; tracks dependencies and assumptions; enforces time, memory, search-depth and solver budgets; applies deterministic ordering/tie-breaking; and records replayable traces.

The executive never invokes neural models for reasoning, planning, intent interpretation or decisions. Privileged action enforcement remains outside the executive.


### 5.2 Input and acquisition subsystem

The acquisition subsystem accepts structured user commands, bounded grammar-based text, local documents/files, databases, approved web sources, application interfaces, APIs, telemetry, sensor readings and Helios-generated events.

Each connector declares source identity, permissions, supported formats, data handling, failure semantics and provenance quality. Original input bytes are referenced by content digest where retention policy permits; acquisition metadata is distinct from claims extracted from a source.

Unstructured sensory inputs requiring a frozen foundation model pass through the NPB and the separate observation contract. Text or strings extracted from external content remain inert data, never instructions to the executive, model runtime, policy engine or action broker. Acquisition never grants an external source or perception model authority to modify policy, permissions, durable memory or action state.

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


### 5.4 Memory system — ephemeral by default

Memory is explicit, bounded state, not an uncontrolled continuation of the conversation. All cognitive memory is temporary by default. No interaction, perceptual observation, episodic record, semantic interpretation, procedure, preference or task summary crosses a task/session boundary unless an explicit persistence operation is authorized.

Logical memory categories are sensory buffer; working memory; temporary episodic context; temporary semantic context; procedural context; prospective context; and autobiographical/project memory. Each allocation has an owner, scope, creation time, expiry rule, sensitivity label and provenance. Durable reuse of any category requires explicit promotion.

Lifecycle operations include allocate, retrieve, link, update, invalidate, expire, erase and promote. A separate trusted persistence gate identifies exact records/fields, purpose, retention period, access scope, authorizing principal and revocation/deletion path. No broad "remember everything" permission is inferred from a one-time request.

Durable semantic facts, episodic history, procedures and preferences are opt-in. A user-approved fact is stored as a separate versioned knowledge record linked to its source; it does not mutate the original observation into ground truth. Retraction of a premise invalidates dependent derived records.

Safety, security and action audit records are not cognitive memory. They are held in a separately governed audit store with its own minimization, integrity, access and retention policy. Audit retention must not become implicit cognitive recall.

Automatic cross-session recall, background consolidation, model-generated memory summaries and silent promotion are disabled by default. Expiration/deletion covers derived indexes, caches and temporary copies, subject to separately disclosed legal/safety audit retention requirements.


### 5.5 Symbolic reasoning system

All reasoning is performed by explicit, non-neural algorithms over typed symbolic representations. Foundation models are not reasoning engines and cannot contribute hidden premises, proofs, plans or decisions.

Candidate mechanisms include rule-based deduction; forward/backward chaining; Datalog; bounded first-order logic; truth maintenance and belief revision; constraint satisfaction; SAT/SMT; answer-set programming; graph algorithms and deterministic search; temporal, spatial and causal reasoning over explicit models; abductive hypothesis generation; analogical reasoning over explicit mappings; and symbolic algebra.

The portfolio is phased. The first symbolic foundation implements typed facts, provenance, a small deterministic rule set and derivation traces. Datalog is the leading candidate for the initial inference layer, subject to a recorded comparison of Soufflé and suitable Rust-native options. SMT, ASP and general planners are deferred until concrete use cases and benchmarks justify them.

Every engine declares supported semantics, determinism profile, resource bounds, failure modes and provenance. Solver timeout, resource exhaustion, unsupported theory or indeterminate result remains unknown/indeterminate; it is never reported as proof of satisfiability or unsatisfiability.


### 5.6 Metacognition and epistemic control

Helios distinguishes known from unknown; observed from inferred; proven from plausible; current from stale; consistent from contradictory; a plan from execution; and an attempted action from a verified result.

Every claim retains source, acquisition method, temporal context, epistemic state, supporting/contradicting evidence, derivation premises and rule/solver version. A model score is never automatically calibrated probability or evidence of truth.

Research four-valued evidence semantics, including Belnap-style representations, so positive support, negative support, both and neither can coexist without unrestricted explosion. This is an evidence-management choice, not a claim that the external world has four truth values.

A perception candidate is always UNTRUSTED. Promotion to an operational belief requires a separate symbolic admission rule, source policy and any required corroboration or user adjudication. No generic confidence threshold converts neural output directly into fact.

Results preserve premises, evidence, rules, intermediate derivations, unresolved assumptions and available proof artifacts. If evidence is insufficient or inconsistent, Helios abstains, asks or exposes uncertainty.

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

The interaction subsystem does not rely on a foundation model to interpret intent, conduct dialogue, generate plans or produce decisions. The initial interface uses structured commands and explicit grammar/semantic parsing over declared language subsets.

Candidate components include tokenization, hand-authored lexical resources, deterministic grammar parsing, semantic parsing into typed requests, entity resolution against approved knowledge, dialogue-state tracking, clarification and template/grammar-based response generation.

A frozen foundation model may extract perceptual content such as visible text, spoken words or scene candidates only when classified as perception and placed in the untrusted observation envelope. Converting extracted text into a command, policy change or executable action requires separate symbolic interpretation and authorization.


### 5.9 Neural Perception Boundary and multiple modalities

The Neural Perception Boundary (NPB) is the only permitted location for frozen foundation-model inference. It is a quarantined sensory peripheral, not a cognitive subsystem.

Permitted tasks are limited to extracting candidate observations from supplied sensory/unstructured inputs: image regions, object/entity candidates, audio events, speech/text segments, document structure or modality-specific measurements. It cannot summarize task history, answer general questions, infer user intent, deliberate, plan, select tools, write memory or issue instructions to another subsystem.

The NPB is isolated by OS process/container/sandbox controls appropriate to the host. Its initial profile is local-only, network-denied, credential-free and least-privilege. It receives read access only to the specific task input and write access only to a bounded output channel. It has no access to cognitive databases/caches, rule stores, action brokers, shell, arbitrary filesystem paths, user secrets or ambient IPC.

Every deployment pins model identity/publisher/version, weights digest, processor/tokenizer digest, preprocessing and inference configuration, runtime version/build digest, backend, hardware profile, precision/quantization and sandbox image/profile. Runtime updates, dynamic downloads, automatic fallback, weight replacement and configuration drift require re-admission.

The NPB returns the versioned PerceptionObservation envelope defined in docs/architecture/NEURAL_PERCEPTION_BOUNDARY.md and schemas/perception-observation.schema.json. The trusted gateway stamps untrusted epistemic status, no authority and task-ephemeral retention; the model cannot set trust, memory scope, policy or persistence.

All perceptual text is inert data. The symbolic core validates schema, bounds, identifiers, references and value types. No arbitrary pointers, executable markup, tool-call structures or undeclared properties are accepted. A valid schema proves structural conformance, not truth.

Hashes protect integrity/content identity, not source authenticity or correctness. Source identity/attestation is assessed separately from epistemic support. Perception is optional; if unavailable or rejected, Helios reports unavailable/unknown and never silently routes to a hosted/unapproved model.


## 6. The non-training learning model

Helios acquires and revises explicit knowledge without training or adapting model parameters. Frozen inference, when used, occurs only inside the NPB and ends at an untrusted observation envelope.

The permitted cycle is:
1. Receive a structured request or untrusted PerceptionObservation.
2. Preserve source identity, input digest and acquisition context for the task lifetime; do not retain raw inputs beyond the ephemeral policy unless authorized.
3. Parse candidate entities, claims, relations, events or procedures through schema validation and declared deterministic processing.
4. Check types, constraints, provenance and source credibility.
5. Compare with task-scoped knowledge and evidence.
6. Identify conflicts, novelty, uncertainty and possible consequences.
7. Derive candidate conclusions only through declared symbolic reasoning.
8. Request human review when required by policy or ambiguity.
9. Add, correct, link or retract task knowledge. Promote selected records into durable knowledge only through the opt-in persistence gate.
10. Record the reason and provenance in the task trace; retain it beyond the task only under explicit policy.
11. Make resulting task knowledge available to later reasoning/planning within the authorized task scope.

This process does not create or optimize neural weights, embeddings, latent representations or other trained parameters. It does not allow a perception model to promote its output, retain memory, update configuration or influence symbolic rules. Task-end cleanup expires temporary cognitive state unless durable retention was explicitly authorized.

Automatic rule creation is high risk. It must be bounded by a formal rule language, provenance, safety constraints, review policy, regression tests and rollback. Helios must not silently convert observed correlation into causal rule.


## 7. Knowledge integrity and evidence model

Every claim should have a stable identifier and structured record containing:
- Claim identifier, subject/predicate/object or proposition, type and domain.
- Source identifier/location, author/asserting party and acquisition method.
- Observation/publication time, ingestion time and valid-time interval.
- Raw input digest/content reference and perception model/runtime provenance, where applicable.
- Evidence and derivation references.
- Evidence state and rationale.
- Status: untrusted candidate, accepted for a declared purpose, disputed, superseded, retracted or rejected.
- Sensitivity/access classification, version/change history and retention scope/authorization reference.

A PerceptionObservation is a source observation, not a knowledge fact. Its epistemic status is UNTRUSTED_CANDIDATE and authority is NONE. The symbolic core may create a separate claim after applying explicit admission rules. The original observation remains immutable during the task; correction/adjudication creates a linked version or status event, not a rewrite of model output.

Contradictions coexist. A newer source does not automatically overwrite an older claim. The system preserves competing claims, supporting/contradicting evidence and reasons for adjudication. Use four-valued evidence semantics or another explicitly specified non-explosive approach where appropriate.

Every derived conclusion links to premises and rule/engine version. Retraction/change marks dependents stale for deterministic recomputation or review. A model-provided score is metadata, not truth, calibrated confidence or proof.

SHA-256 input/envelope digests provide integrity and content-addressing only. Source authenticity requires separately verified identity, signature or transport/device attestation. Neither integrity nor authenticity establishes proposition truth.

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

The data architecture separates temporary cognitive state, durable records and safety/audit state.

### 14.1 Task-ephemeral cognitive state

Working memory, sensory buffers, temporary episodic/semantic context, candidate perception observations, derived task facts, plans and intermediate traces are held in task-scoped storage. They carry owner, scope, creation time, expiry and sensitivity labels. Task-end cleanup erases or invalidates temporary records, caches and derived indexes.

Crash recovery may restore only the minimum volatile state required to safely abort/reconcile an interrupted action; it must not silently resume cognitive recall or execution.

### 14.2 Explicitly retained knowledge

A separate persistence gate controls any durable semantic fact, preference, episode, procedure, project record or prospective obligation. The request identifies exact records/fields, purpose, access scope, retention duration, authorizing principal and deletion/revocation mechanism.

Durable knowledge is versioned and provenance-bearing. It is not created merely because something appeared in a conversation, was perceived, inferred or useful once. Persistent semantic knowledge and episodic history are opt-in.

### 14.3 Safety and action audit store

Consequential authorization and execution records are stored separately from cognitive memory. Their retention is governed by safety, security, privacy and applicable legal requirements. Audit records are not automatically retrievable as cognitive memory; access requires a distinct audited operation.

### 14.4 Storage categories

Separate as needed: temporary raw-input buffers/content-addressed task objects; task-local observations/claims; explicitly approved durable knowledge; versioned rules/ontologies/schemas; temporary plan/execution state; user configuration/scoped permissions; device inventory/adapter metadata; separate safety/action audit records; rebuildable indexes/caches.

No persistence technology is selected yet. A database or file store must not cause retention merely because it is technically persistent. Application and storage boundaries enforce the policy; temporary caches/indexes must be task-scoped and clearable.

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

Helios is assessed by reproducible task suites, not subjective impressions. Evaluation includes:
- Symbolic-core determinism for identical canonical inputs, state, rules and execution profile.
- Logical validity of deductions and complete premise/provenance/derivation traces.
- Contradiction handling and premise retraction.
- Perception model/artifact pinning and processor/runtime configuration.
- NPB sandbox isolation, denied network/credential/tool/memory access and bounded I/O.
- Schema validation, size/reference bounds and rejection of undeclared fields.
- Adversarial multimodal inputs with no authority propagation.
- Separation of content integrity, source authenticity and epistemic support.
- Memory scope, expiration/deletion and no unapproved cross-session restoration.
- Explicit durable-persistence authorization/revocation.
- No background consolidation or model-generated memory promotion.
- Planning feasibility, deterministic tie-breaking and constraint satisfaction.
- Abstention, clarification, unknown/timeout reporting and safe action authorization.

Initial experiments include repeated replay of identical symbolic fixtures; malformed/adversarial observation injection; contradictory sensor/model evidence; model/runtime unavailability; attempted NPB access to memory/tools/network; task-end cleanup; restart without persistence authorization; premise retraction; solver resource exhaustion; and unauthorized mock-device operations.

Perception accuracy is measured independently against representative labelled data. Symbolic reasoning correctness is measured independently against fixed observations. End-to-end success must not obscure which layer produced an error. Every benchmark defines inputs, invariants, environment, resource limits, scoring and known limitations.

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
- Datalog semantics, recursion, negation, termination and provenance; compare Soufflé with Rust-native candidates.
- SAT/SMT capabilities, proof production, deterministic resource bounds and solver limitations.
- Truth-maintenance systems, paraconsistent logic and belief revision.
- Classical/hierarchical planning, PDDL and scheduling.
- Typed knowledge representation, ontology engineering, temporal databases and claim provenance.
- Ephemeral memory lifecycle, explicit persistence, secure erasure and crash-safe task cleanup.
- Frozen perception deployment: model manifests, weight/processor digests, runtime/build pinning, quantization, backend reproducibility and model supply-chain review.
- Sandboxing, process isolation, network denial, least privilege and IPC contracts for the NPB.
- Multimodal prompt-injection/adversarial-input risk and deterministic downstream validation.
- JSON Schema Draft 2020-12, I-JSON constraints, RFC 8785 canonicalization and SHA-256 integrity; evaluate Protobuf only after deterministic canonical-signing profile is defined.
- Classical computer vision, non-trained signal processing and bounded perception alternatives.
- Causal reasoning, counterfactual models, analogical reasoning and non-neural language grammars.
- Capability security, device-driver isolation, IOMMU/DMA boundaries, device identity/firmware trust, hardware-in-loop testing and privacy/deletion governance.

Prefer official specifications, standards, original research and maintained repositories. For each topic record source/date, assumptions, alternatives, limits, license, version and recommendation. Recheck before implementation. No broad implementation begins until the initial symbolic domain, observation contract, memory lifecycle, threat model and acceptance fixtures are specified.


## 19. Non-goals and exclusions

Helios will not train, fine-tune, distill or adapt any model; use a foundation model outside the approved NPB perception role; use neural models for reasoning, intent interpretation, dialogue orchestration, planning, decisions, memory, knowledge revision, tool selection or action authorization; use learned embeddings/retrieval in the symbolic core; expose memory, policy, tools, credentials, network clients or privileged execution to the NPB; automatically persist episodic or semantic cognitive memory across tasks/sessions; silently consolidate or promote temporary state; execute arbitrary natural-language shell commands with elevated privileges; control hardware without declared capabilities and authorization; silently resolve contradictory evidence; or claim formal verification/safety certification/production readiness without evidence.

## 20. Requirements register

These identifiers are stable requirement anchors. Future detailed requirements may be added without changing existing IDs.

### Project identity
- HEL-FOUND-001: Helios is a personal cognitive computing framework.
- HEL-FOUND-002: Project status must remain Concept/Design until actual implementation begins and is evidenced.
- HEL-FOUND-003: The master specification is the canonical authority.
- HEL-FOUND-004: Design and implementation must use existing, maintained programming languages and tools.


### Non-training and restricted perception
- HEL-NTR-001: No training of any neural or learned model is permitted.
- HEL-NTR-002: No pretraining, fine-tuning, distillation, online adaptation, parameter updates or model fitting is permitted.
- HEL-NTR-003: Frozen pretrained-model inference is permitted only inside the approved NPB and only for sensory/unstructured-input perception; all other model inference is prohibited.
- HEL-NTR-004: The symbolic core must not use learned embeddings, latent-vector retrieval, learned rerankers or neural classifiers; perception candidates may be produced only within the NPB.
- HEL-NTR-005: New information is acquired as explicit, provenance-bearing observations and claims.
- HEL-NTR-006: Rule/procedure changes are explicit, auditable and governed; they do not become hidden parameter learning.
- HEL-NTR-007: Neural inference is not used for cognition; the initial supported perception profile is local-only; remote inference requires separate approval and verifiable immutability.
- HEL-NTR-008: Every direct, transitive, optional, build-time and runtime dependency is reviewed; perception-only dependencies are admitted only to the isolated NPB set.
- HEL-NTR-009: Supported builds contain no training datasets, optimizer state, unapproved weights, checkpoints or learned cognitive indexes; only approved, digest-pinned perception weights may be present in the NPB.
- HEL-NTR-010: Non-training, model-pinning and NPB-isolation checks are automated where feasible and supplemented by human review.

### Neural perception boundary
- HEL-PER-001: Frozen foundation-model inference is confined to the NPB and limited to perception.
- HEL-PER-002: Every model output is an untrusted candidate observation with no authority.
- HEL-PER-003: The NPB cannot access symbolic memory, rules, executive state, tools, credentials, network clients or privileged execution.
- HEL-PER-004: Every observation is schema-validated, bounded and linked to input, source, model, processor and runtime provenance.
- HEL-PER-005: The NPB cannot set epistemic status, trust, persistence, policy or authorization fields.
- HEL-PER-006: Perception failure/unsupported input yields unavailable/unknown; no silent model fallback.
- HEL-PER-007: Hashes establish integrity, not truth or source authenticity.
- HEL-PER-008: Model artifacts, runtime, backend, precision/quantization and inference configuration are pinned and re-admitted on change.

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
- HEL-MEM-004: Cognitive memory is ephemeral by default and scoped to an explicit task/session.
- HEL-MEM-005: Cross-task/session episodic, semantic, procedural, prospective and autobiographical retention is opt-in.
- HEL-MEM-006: Durable persistence requires explicit scoped authorization identifying records, purpose, retention period and deletion path.
- HEL-MEM-007: Safety/action audit records are isolated from cognitive memory and are not automatically available for recall.
- HEL-MEM-008: Task-end expiration/deletion covers temporary records, caches and derived indexes, subject to separately governed audit retention.
- HEL-MEM-009: Automatic background consolidation, memory summarization and silent promotion are disabled by default.

### Interaction and perception
- HEL-INT-001: Initial interaction supports a precise structured command path.
- HEL-INT-002: Natural-language capabilities are bounded by implemented grammars and semantic representations.
- HEL-INT-003: Ambiguous requests trigger clarification.
- HEL-INT-004: Multimodal support is staged and evidence-based.
- HEL-INT-005: No neural model is used for language understanding, dialogue, reasoning or generation; frozen models may be used only for NPB perception.

### Symbolic core
- HEL-SYM-001: All reasoning, planning, decisions and memory governance are performed by explicit non-neural symbolic mechanisms.
- HEL-SYM-002: Identical canonical symbolic inputs, state, rules and execution profile produce reproducible outputs within the declared determinism profile.
- HEL-SYM-003: Solver timeouts, resource exhaustion and unsupported operations remain unknown/indeterminate, not proof.
- HEL-SYM-004: Every derived result preserves premises, evidence, engine/rule version and derivation trace.
- HEL-SYM-005: Contradictory evidence is represented without silent overwrite or unrestricted inference.

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
6. Which data, if any, should be promoted to durable memory after the opt-in persistence gate is available?
7. Which actions, if any, may be pre-authorized under narrow user-configured policies?
8. Which sensory modalities and local frozen perception models are justified for the first NPB profile, and what reproducibility guarantees can the selected hardware/runtime meet?
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
- No model training or neural cognitive inference is part of the intended architecture. A restricted frozen-model perception exception is specified but no model/runtime has been approved or implemented.
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
- Soufflé Datalog: https://souffle-lang.github.io/docs/home/
- Soufflé rules and stratification: https://souffle-lang.github.io/rules
- Potassco clingo: https://potassco.org/clingo/
- Z3 Guide and solver parameters: https://microsoft.github.io/z3guide/ and https://microsoft.github.io/z3guide/programming/Parameters/
- JSON Schema 2020-12: https://json-schema.org/specification
- RFC 8785 JSON Canonicalization Scheme: https://www.rfc-editor.org/rfc/rfc8785.html
- OWASP LLM01:2025 Prompt Injection: https://genai.owasp.org/llmrisk/llm01-prompt-injection/
- NIST AI 600-1 Generative AI Profile: https://doi.org/10.6028/NIST.AI.600-1
- NPB contract: docs/architecture/NEURAL_PERCEPTION_BOUNDARY.md
- PerceptionObservation schema: schemas/perception-observation.schema.json
- Phase A plan: docs/PHASE_A_FOUNDATION.md

Reference pages must be rechecked before implementation. Record actual versions, licenses, platform support and limitations. Do not infer formal guarantees for an entire system from a kernel or solver's guarantees.

---

**Authority statement:** This document governs Helios design. Roadmap, status and decision records are supporting project-control documents. If they conflict with this specification, this specification takes precedence until an approved revision changes it.
