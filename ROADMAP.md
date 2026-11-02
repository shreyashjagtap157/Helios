# Helios Roadmap

**Status:** Proposed sequence; no implementation has started.  
**Authority:** The master specification defines requirements. This roadmap defines sequencing only.

## Guiding sequence

Do not attempt to build a cognitive engine, device-control framework, desktop environment and operating-system kernel simultaneously. First prove a small non-training cognitive loop on an existing operating system. Expand only when each dependency has a tested contract.

## Stage 0 — Project definition and research

**Goal:** Resolve enough ambiguity to create a buildable first prototype without pretending the full vision is already specified.

Deliverables:
- Approved master specification.
- Requirements traceability and decision log.
- Initial use-case catalogue.
- Strict non-training dependency policy, including transitive dependency admission, model-asset exclusion and runtime-service restrictions.
- Domain and modality scope for the first demonstration.
- Hardware target shortlist.
- Research register with source quality and conclusions.
- Risk register covering non-training language/perception limits and hardware action risks.

Exit criteria:
- The first user problem is explicit.
- The initial supported reasoning domain is bounded.
- No trained or pretrained model dependency is required.
- The host OS and candidate implementation stack have a reasoned shortlist.
- Initial security and action boundaries are specified.

## Stage 1 — Technology and architecture spikes

**Goal:** Test the riskiest design assumptions before selecting the implementation architecture.

Spikes:
- Rust core service and persistence experiment.
- Datalog/logic engine comparison.
- SAT/SMT and constraint-solving integration.
- Simple typed planning model.
- Provenance and contradiction handling.
- Grammar/structured-command interaction without neural models.
- Mock device capability broker.
- Dependency graph, SBOM and build-artifact review to verify non-training conformance.
- Host-OS permission and process-isolation design.

Exit criteria:
- Benchmark results and limitations are recorded.
- Candidate libraries and licenses are checked.
- At least one end-to-end reasoning trace is reproducible.
- No component silently depends on trained models or model-inference services.
- The supported dependency graph and build artifacts have recorded non-training review evidence.

## Stage 2 — Cognitive vertical-slice prototype

**Goal:** Prove the smallest useful Helios loop.

Target flow:
1. Accept a structured user request.
2. Retrieve or ingest explicit facts with provenance.
3. Derive a result using declared rules or a solver.
4. Explain the derivation and uncertainty.
5. Store the interaction as an episodic event.
6. Create a bounded plan.
7. Request authorization for a harmless, reversible action.
8. Run a mock adapter.
9. Verify the mock postcondition.
10. Persist the audit record and recover state after restart.

Exit criteria:
- Reproducible tests pass.
- The system can distinguish facts, assumptions, inferences and unknowns.
- It reports unsupported requests honestly.
- It has no model training or model inference dependency.
- Authorization and audit boundaries work.

## Stage 3 — Knowledge and memory foundation

**Goal:** Develop robust explicit knowledge rather than accumulating unstructured text.

Deliverables:
- Typed entity, fact, relation, event and rule schemas.
- Source and claim provenance.
- Versioning and correction.
- Contradiction representation.
- Temporal validity.
- Derived-claim dependency tracking.
- Working, episodic, semantic and procedural memory.
- Search and query interfaces.
- Backup, restore and schema migration.

Exit criteria:
- Knowledge survives restart and migration.
- Contradiction and retraction tests pass.
- Derived conclusions can be invalidated after premise changes.
- Memory lifecycle behavior is explicit and tested.

## Stage 4 — Reasoning and planning expansion

**Goal:** Add reasoning methods according to demonstrated use cases.

Candidate sequence:
- Deduction and rule execution.
- Forward and backward chaining.
- Constraint solving.
- Search and graph algorithms.
- Abductive hypothesis generation.
- Temporal reasoning.
- Causal and counterfactual models where domain knowledge is sufficient.
- Analogical reasoning over explicit structures.
- Planning with preconditions, effects, costs and safety constraints.

Exit criteria:
- Each reasoning engine has a declared supported fragment.
- Soundness/validity tests exist where possible.
- Time and memory bounds are enforced.
- Solver timeouts are not misreported as negative answers.
- Explanations link to premises and derivations.

## Stage 5 — Practical interaction and personal workflows

**Goal:** Make the framework usable for real personal tasks without overstating natural-language competence.

Deliverables:
- Structured command interface.
- Bounded natural-language grammar and semantic parser.
- Dialogue state and clarification.
- Template-based response generation.
- Task history and search.
- User preferences and explicit policy configuration.
- Local UI/CLI integration.
- Optional integrations for documents, files and approved services.

Exit criteria:
- Ambiguous requests are clarified.
- User corrections are recorded safely.
- External content cannot override system policy.
- Personal data access is auditable and revocable.

## Stage 6 — Controlled hardware integration

**Goal:** Add real device interaction in a staged, safe manner.

Sequence:
1. Device inventory and read-only discovery.
2. Mock adapters and protocol conformance.
3. Read-only telemetry.
4. Harmless, reversible low-impact operations.
5. Explicitly confirmed consequential operations.
6. Physical devices only after safety analysis and hardware-in-the-loop tests.

Deliverables:
- Typed device capability model.
- Device identity and inventory.
- Permission broker.
- Operation bounds and timeouts.
- Action preview and confirmation.
- Emergency stop and human override paths.
- Postcondition verification.
- Append-only action audit.
- Device adapter test harness.

Exit criteria:
- Unauthorized operations are rejected.
- Unknown device state blocks consequential actions.
- Failures, disconnections and partial actions are handled safely.
- Every action is recorded and verified.
- Hazardous actions remain outside supported scope unless independently justified.

## Stage 7 — Helios system environment

**Goal:** Integrate Helios deeply with an existing OS before considering kernel development.

Potential deliverables:
- Background cognitive service.
- Secure device broker service.
- User-session and permission integration.
- Local storage and recovery services.
- Application/extension lifecycle.
- System tray, command shell or desktop interface.
- Secure update and rollback.
- OS-level audit and monitoring.

Exit criteria:
- Service isolation and privilege boundaries are tested.
- Helios can operate as a useful personal environment on an existing OS.
- Driver compatibility is not lost.
- No custom kernel is required to meet the demonstrated requirements.

## Stage 8 — Operating-system ecosystem decision

**Goal:** Decide whether a dedicated distribution, microkernel-based system or custom kernel is justified.

Evaluate:
- Linux-based dedicated distribution.
- Microkernel options and user-mode driver architecture.
- Existing embedded OS for constrained device nodes.
- Custom kernel only if measurable requirements cannot be met otherwise.

Exit criteria:
- Hardware coverage is inventoried.
- Security boundary and threat model are documented.
- Driver strategy and maintenance costs are understood.
- Boot, update, recovery and compatibility plans exist.
- A kernel choice is approved on evidence, not ambition alone.

## Stage 9 — Broader ecosystem maturity

**Goal:** Extend only after the core is reliable.

Possible work:
- Plugin SDK and conformance suite.
- Additional domains and knowledge connectors.
- Additional hardware adapters.
- More capable non-trained multimodal processing.
- Distributed or multi-device operation where justified.
- Formal models and proofs for critical subsystems.
- Long-term backup, upgrade and migration governance.

## Milestone discipline

Every milestone must publish:
- Scope and exclusions.
- Requirements covered.
- Decisions made.
- Source files and artifacts.
- Tests actually run and results.
- Build/lint/CI results where applicable.
- Security and safety review.
- Known limitations.
- Residual risks.
- Next dependency.

A milestone is not complete because documentation or code exists. Completion requires evidence against its acceptance criteria.
