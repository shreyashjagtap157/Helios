Omni Self-Hosting Phased Plan
=================================

This file records the per-phase implementation plan, acceptance criteria, and commands
to verify progress for the Omni programming language self-hosting effort.

Phase 1 — Structural Foundation (COMPLETE)
- Goals:
  - Parser, AST, minimal IR, and basic runtime scaffolding are present.
  - Repository successfully builds with `cargo check` and compiler tests pass.
- Acceptance:
  - `cargo check` and `cargo test` pass for `omni-lang/compiler`.

Phase 2 — Core Functionality (NEXT)
- Goals:
  - Type system implemented and exercised by unit tests.
  - Minimal standard library pieces available (`io`, `core`, `collections`).
  - Compilation pipeline functioning: lex -> parse -> semantic -> IR -> OVM back-end.
- Acceptance:
  - `cargo test` passes; basic Omni stdlib code compiles under stage0 compiler.
  - `bootstrap.sh --stage1` produces `omnc_stage1.ovm` without runtime errors.
- Verification commands:
  - `cd omni-lang/compiler && cargo test --quiet`
  - `./bootstrap.sh stage1` (or use `BootstrapRunner`)

Phase 3 — Enrichment
- Goals:
  - Advanced type features (generics, trait bounds) tested.
  - Concurrency primitives and async/await support validated.
- Acceptance:
  - New test suites for async/await and generics pass.

Phase 4 — Expansion and Optimization
- Goals:
  - LLVM/native backend polishing and optimizer passes validated.
  - Performance regression tests added.
- Acceptance:
  - Native codegen smoke tests and basic microbenchmarks pass.

Phase 5 — Self-Hosting
- Goals:
  - Stage0 (Rust omnc) compiles the Omni compiler to Stage1.
  - Stage1 compiles the same source to Stage2; Stage1==Stage2 (fixpoint).
- Acceptance:
  - `BootstrapRunner::verify_fixpoint` returns OK for `omnc_stage1.ovm` vs `omnc_stage2.ovm`.

Notes and immediate next actions
- Create minimal runtime helpers needed for Stage1 (e.g., `println` bootstraps).
- Add CI jobs for Node/TS for `tools/vscode-omni` and an automated bootstrap job.
- Address top lints/warnings in `omni-lang/tools/omni-fmt` and `compiler` tests.

How to contribute
- Follow the per-phase acceptance tests above and create small, focused PRs.
