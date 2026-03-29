# HELIOS + Omni Monorepo

Comprehensive repository guide for the HELIOS cognitive framework and the Omni programming language/toolchain.

---

## 1) What This Repository Is

This repository contains two tightly-coupled main systems:

- **Omni Language (`omni-lang/`)**  
  A systems/application programming language and toolchain with ownership semantics, trait-based abstractions, interpreter + VM execution, native/codegen paths, and self-hosting bootstrap work.

- **HELIOS Framework (`helios-framework/`)**  
  A broader cognitive framework ecosystem centered on exact knowledge storage, evidence-aware workflows, capability orchestration, and persistent assistant/runtime surfaces.

In short: **Omni provides the language/compiler substrate; HELIOS is the framework built on top of it.**

---

## 2) Current Project State (High-Level)

Based on current repository status docs and test runs:

- Omni compiler core pipeline is operational (lexer → parser → semantic → IR → codegen/runtime).
- Compiler test suites pass in current baseline runs.
- Self-hosting is in progress; binary emission and bootstrap stage completion remain the critical path.
- HELIOS framework structure is present and being normalized around exact-knowledge contracts and unified runtime/service/API/app surfaces.
- Obsidian vault tracking exists in `Memory/` and is intended to be continuously maintained.

Primary status references:

- `docs/IMPLEMENTATION_STATUS_REPORT.md`
- `docs/HELIOS & Omni Language — Comprehensive.md`
- `omni-lang/docs/ISSUES.md`
- `Memory/Welcome.md`

---

## 3) Repository Layout

Top-level directories:

- `omni-lang/` — Omni language, compiler, stdlib, tooling, tests, self-hosting sources
- `helios-framework/` — HELIOS framework modules and runtime surfaces
- `docs/` — long-form architecture, plans, status, roadmap, and references
- `Memory/` — Obsidian-compatible project memory, link graph, and implementation tracking notes
- `examples/` — example workflows (repository-level)
- `config/` — configuration assets
- `scripts/` — utility scripts and automation
- `build/` — build artifacts/utilities
- `legacy/` — legacy items retained for migration/reference

Important root files:

- `ISSUES.md` — repository-level issue context
- `AGENTS.md`, `CLAUDE.md` — assistant/tooling operating conventions
- `build_and_deploy.ps1` — deployment/build orchestration entrypoint

---

## 4) Omni Language Deep Overview

### 4.1 Core goals

- memory safety and explicit ownership semantics
- performant compiled/runtime execution options
- modern language ergonomics with strong static analysis
- eventual self-hosting bootstrap chain

### 4.2 Key Omni areas

- `omni-lang/compiler/`  
  Rust compiler implementation (`omnc`) and runtime infrastructure.

- `omni-lang/std/` and `omni-lang/core/`  
  Omni standard/core library modules.

- `omni-lang/omni/`  
  Self-hosted Omni compiler/toolchain sources written in Omni.

- `omni-lang/tools/`  
  Ecosystem tools such as LSP/DAP/formatter/package management.

- `omni-lang/tests/` + `omni-lang/examples/`  
  Integration and scenario coverage for language/runtime behavior.

### 4.3 Omni known priority work

- Complete standalone binary emission in self-hosted path
- Finish bootstrap stage progression and verification
- Close interpreter parity gaps (`std::math`, `std::thread` related work)
- Continue warning and maintainability cleanup across compiler modules

Canonical issue tracker:

- `omni-lang/docs/ISSUES.md`

Omni-specific references:

- `omni-lang/README.md`
- `docs/README_omni-lang.md`
- `docs/OMNI_LANGUAGE_REFERENCE.md`

---

## 5) HELIOS Framework Deep Overview

### 5.1 Core goals

- exact knowledge and provenance-aware operation
- capability-driven architecture (auditable operations)
- persistent memory and experience handling
- converged CLI/service/API/app behavior over a common runtime

### 5.2 Key HELIOS areas

- `helios-framework/helios/` — core runtime surfaces (API, service, knowledge, IO)
- `helios-framework/brain/` — cognitive modules, memory, reasoning, learning workflows
- `helios-framework/training/` — corpus, pruning, checkpoints, optimization utilities
- `helios-framework/app/` — user-facing app and integration surface
- `helios-framework/safety/` — governance/action-control logic
- `helios-framework/config/` — defaults and config loading
- `helios-framework/biometrics/` — identity-verification surfaces

HELIOS-specific references:

- `helios-framework/README.md`
- `docs/README_helios-framework.md`

---

## 6) Documentation and Knowledge System

### 6.1 Repository docs (`docs/`)

Use `docs/` for canonical long-form technical documents:

- architecture and comprehensive plans
- implementation status reports
- roadmap and verification reports
- language/reference material

### 6.2 Obsidian memory (`Memory/`)

`Memory/` is the project memory system designed for continuous tracking.

Entry points:

- `Memory/Welcome.md`
- `Memory/Projects/Projects Dashboard.md`

Project tracking:

- `Memory/Projects/Omni/*`
- `Memory/Projects/Helios/*`

Folder/file link graph:

- `Memory/Projects/Omni/Folder-File Links Index.md`
- `Memory/Projects/Helios/Folder-File Links Index.md`

These notes are intended to provide exhaustive, navigable coverage of project folders/files and progress context.

---

## 7) Build, Test, and Validation

### 7.1 Omni compiler baseline

From `omni-lang/compiler/`:

```powershell
cargo build
cargo test
```

Targeted examples:

```powershell
cargo test lexer
cargo test parser
cargo test semantic
cargo test optimizer
```

### 7.2 General guidance

- Run targeted tests for the subsystem you change.
- Run full compiler tests before finalizing major Omni changes.
- Keep issue tracker and Memory notes synchronized after meaningful updates.

---

## 8) Development Workflow (Recommended)

1. **Identify scope**  
   Start from issue tracker and status docs.

2. **Assess impact**  
   Locate affected modules/files and expected test coverage.

3. **Implement in small batches**  
   Prefer cohesive, subsystem-local commits.

4. **Validate**  
   Run relevant tests, then broader suite as needed.

5. **Update docs/memory**  
   Reflect changes in:
   - `omni-lang/docs/ISSUES.md`
   - `Memory/` project notes/indexes
   - `docs/` if architecture/status materially changed

---

## 9) Contributor Conventions

- Keep changes scoped and traceable.
- Do not silently break vault/documentation links.
- Prefer explicit status updates over implicit assumptions.
- Preserve consistency between:
  - code reality,
  - issue tracker,
  - status docs,
  - Obsidian memory notes.

---

## 10) Quick Navigation

- Root status:
  - `docs/IMPLEMENTATION_STATUS_REPORT.md`
  - `docs/COMPREHENSIVE_LONG_TERM_ROADMAP.md`

- Omni:
  - `omni-lang/README.md`
  - `omni-lang/docs/ISSUES.md`
  - `docs/README_omni-lang.md`

- HELIOS:
  - `helios-framework/README.md`
  - `docs/README_helios-framework.md`

- Obsidian memory:
  - `Memory/Welcome.md`
  - `Memory/Projects/Projects Dashboard.md`
  - `Memory/Projects/Omni/Folder-File Links Index.md`
  - `Memory/Projects/Helios/Folder-File Links Index.md`

---

## 11) Intended End State

This monorepo is aimed at:

- a robust Omni toolchain capable of full self-hosting,
- a fully integrated HELIOS framework running on Omni-native foundations,
- complete traceability of implementation and status through both canonical docs and Obsidian memory.

When updating this repository, treat documentation and memory synchronization as part of the implementation definition-of-done.
