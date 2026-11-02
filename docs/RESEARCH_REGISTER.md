# Helios Research Register

This is a starting register. It is not a claim that all topics have been fully researched or that listed projects are approved dependencies.

## Systems languages and operating environments

- Linux kernel Rust documentation: https://www.kernel.org/doc/html/latest/rust/
  - Research relevance: constraints and status of Rust use inside the Linux kernel.
- Rust Embedded Working Group: https://github.com/rust-embedded
  - Research relevance: embedded and bare-metal toolchain ecosystem.
- Embedded Rust Book — no_std: https://doc.rust-lang.org/embedded-book/intro/no-std.html
  - Research relevance: distinction between hosted and no_std environments.
- Tock OS: https://tockos.org/documentation/getting-started/
  - Research relevance: a Rust-based embedded OS with isolation boundaries for applications and drivers.
- seL4 documentation: https://docs.sel4.systems/projects/sel4/
  - Research relevance: microkernel architecture and formal assurance.
- seL4 capabilities tutorial: https://docs.sel4.systems/Tutorials/capabilities.html
  - Research relevance: capability-based access control.
- seL4 hardware support: https://docs.sel4.systems/Hardware/
  - Research relevance: platform coverage and configuration-specific verification status.

## Symbolic reasoning and logic

- Soufflé Datalog documentation: https://souffle-lang.org/docs/home/
  - Research relevance: scalable rule evaluation and relational reasoning.
- Soufflé language reference: https://souffle-lang.org/datalog-language
  - Research relevance: relations, facts, rules, types and evaluation.
- Potassco clingo: https://potassco.org/clingo/
  - Research relevance: answer-set programming, grounding and solving.

## Research questions to expand

- Logic-programming semantics, recursion, negation and termination.
- Knowledge representation, ontology engineering and entity identity.
- Truth-maintenance and belief revision.
- SAT/SMT capabilities, theories, proof production and solver limits.
- Classical and hierarchical planning.
- Causal inference without learned models.
- Analogical reasoning and structural mapping.
- Natural-language grammars and semantic parsing.
- Classical computer vision and signal processing.
- Device-driver isolation, IOMMU/DMA safety and firmware trust.
- Secure system update, recovery and audit integrity.

## Source-quality rules

Prefer official documentation, standards, primary research papers and maintained repositories. For each adopted technology, record exact version, license, supported targets, dependencies, maintenance status, known limitations and whether any capability depends on a trained model. Recheck all versions before implementation.
