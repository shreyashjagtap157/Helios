# Helios

**A personal, non-training cognitive computing framework with a long-term hardware and operating-system ecosystem vision.**

Helios is intended to acquire and organize explicit knowledge, reason over it, maintain memory, plan tasks, evaluate its conclusions and carry out user-authorized actions through controlled software and hardware interfaces.

Its defining constraint is strict: Helios must not depend on training any model, fine-tuning a model, pretrained neural models, learned embeddings or model inference services. Its cognitive mechanisms are to be built from explicit representations, algorithms, symbolic reasoning, search, planning, constraint solving, persistent knowledge and auditable procedures.

## Project phase

**Concept and specification only**

No Helios implementation has started. No subsystem, operating system, cognitive engine, device broker or hardware integration is implemented. This repository currently holds the design baseline and project-control records only.

## Start here

1. **[HELIOS_MASTER_SPEC.md](HELIOS_MASTER_SPEC.md)** — canonical project definition, non-training constraint, cognitive architecture, knowledge and memory model, hardware interaction, operating-system direction, security, requirements and open decisions.
2. **[ROADMAP.md](ROADMAP.md)** — dependency-ordered research and implementation milestones.
3. **[DECISIONS.md](DECISIONS.md)** — approved decisions, proposed decisions and unresolved questions.
4. **[STATUS.md](STATUS.md)** — verified project state.
5. **[docs/NON_TRAINING_COMPLIANCE.md](docs/NON_TRAINING_COMPLIANCE.md)** — dependency admission rules and future verification evidence for the non-training constraint.
6. **[docs/RESEARCH_REGISTER.md](docs/RESEARCH_REGISTER.md)** — research areas and authoritative starting sources.

## Repository validation

Run `ruby scripts/validate_repository.rb` from a local clone. This dependency-free check validates required project files, removal of legacy project artifacts, GitHub issue-form YAML structure, the repository workflow's safety properties, local Markdown link targets, and the master requirement register. It validates repository governance and documentation only; it does not test a Helios runtime or establish that any planned capability has been implemented.

## Core objectives

- Explicit knowledge acquisition with provenance and version history.
- Multiple transparent reasoning methods rather than one opaque model.
- Working, episodic, semantic, procedural and prospective memory.
- Planning, problem solving and metacognitive uncertainty management.
- A strict separation between reasoning and privileged action execution.
- Safe, authorized interaction with devices and hardware.
- A local-first framework that can eventually form the center of an operating environment.

## Development principles

- No hidden model dependencies.
- No claims of implementation without source and test evidence.
- Use established languages and tools; select them per subsystem after evaluation.
- Integrate with an existing operating system first.
- Defer kernel development until there is a demonstrated need.
- Keep user authority, least privilege, safe failure and auditability central.

## License

See [LICENSE](LICENSE).
