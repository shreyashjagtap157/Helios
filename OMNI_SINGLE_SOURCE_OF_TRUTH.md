# OMNI Unified Specification And Execution Truth

Updated: 2026-04-02
Owner: Omni maintainers
Status: Authoritative root-level spec, roadmap, and implementation truth

## 1. Purpose

This is the single authoritative root document that merges:
1. Previous Omni requirements and implementation baselines.
2. The newer Omni v2.0 requirements (effect system, structured concurrency, generational references, linear types, MLIR/tensor direction, diagnostics and tooling upgrades).
3. Current repository implementation state and verified gaps.

All other root historical status files are archival notices only.

## 2. Canonical Omni Definition (Merged)

Omni is a layered, deterministic, systems-to-platform language with:
1. Static-first typing and explicit controlled escape hatches.
2. Ownership-first memory safety with advanced resource semantics.
3. Type-and-effect aware execution model.
4. Hybrid concurrency with structured lifetime rules.
5. Capability-centered security boundaries.
6. A self-hosting bootstrap trajectory with reproducibility gates.

HELIOS is the primary platform target and validation layer for Omni capabilities.

## 3. Normative v2.0 Requirement Baseline (Merged)

### 3.1 Language And Type System

1. Static typing by default with bidirectional inference.
2. Optional dynamic zones only through explicit boundaries.
3. Null-free safe core (`Option<T>` for absence).
4. `Result<T, E>` primary failure channel with typed propagation.
5. Generics, trait constraints, pattern matching, compile-time computation.
6. Public APIs require explicit type/effect contracts once effect pipeline is active.

### 3.2 Memory And Ownership

1. Ownership and borrowing are canonical memory rules.
2. Polonius-class precision is target model for borrow analysis.
3. Field projection borrowing, generational references, and linear-resource semantics are adopted design goals.
4. Unsafe is explicit, auditable, and narrow.
5. GC compatibility is an optional layer, never default core behavior.

### 3.3 Effects And Concurrency

1. Effects are explicit in signatures (or inferred internally): e.g. `/ io + async + throw<E>`.
2. Structured concurrency is default design direction.
3. Async cancellation must be explicit and composable.
4. Concurrency stack includes threads, async, channels, and actor patterns.

### 3.4 Compilation And Runtime

1. Pipeline target: source -> tokens -> CST/AST -> semantic/effect resolution -> MIR -> borrow/effect checks -> LIR -> codegen.
2. Backends: fast development path and optimized release path; MLIR path planned for accelerator-oriented phases.
3. Deterministic and reproducible artifacts are required for self-hosting closure claims.

### 3.5 Security And Capability Model

1. Capability-based access is mandatory for sensitive operations.
2. Effects and capabilities must align as implementation matures.
3. FFI remains explicitly unsafe with progressive sandbox hardening goals.

### 3.6 Tooling And DX

1. Unified CLI workflow (`build`, `check`, `test`, `fmt`, `lint`, `doc`, `fix`, etc.).
2. LSP and diagnostics are first-class quality gates.
3. Diagnostics quality standard: actionable spans, error codes, and fix guidance.

## 4. Current Repository Truth (Merged, Evidence-Based)

### 4.1 Verified

1. Compiler/runtime codebases are substantial and active.
2. Stage1/Stage2 deterministic parity evidence exists in prior logs.
3. Parser compatibility has improved with recent multiline/import/pattern fixes.
4. Existing tests for targeted parser regressions are passing.

### 4.2 Not Yet Verified / Not Complete

1. Full Stage3 reproducible self-hosting closure remains blocked.
2. Runtime fidelity and strict semantic parity are incomplete.
3. Several v2.0 features remain partially implemented or design-only.

## 5. Requirements Traceability Matrix (Merged)

| Requirement Area | Target | Current | Priority |
|---|---|---|---|
| Type-and-effect signatures | Full parse + semantic checks | Partial parse support | P0 |
| Ownership + advanced borrow precision | Enforced | Partial | P0 |
| Structured concurrency | Enforced model | Partial | P1 |
| Capability/effect alignment | Enforced | Partial | P1 |
| Self-hosting Stage3 closure | Required for completion | Blocked | P0 |
| Tooling quality gates | Production-grade | Partial-to-high | P1 |
| Tensor/MLIR acceleration path | Phased target | Early/partial | P2 |

## 6. Immediate Execution Plan (Start Implementing, Not Just Planning)

### P0 (Current Active)

1. Parser and frontend acceptance for v2 signature constructs.
2. Runtime compatibility closure for Stage1-bytecode -> Stage3 path.
3. Self-hosting reproducibility gate automation.

### P1

1. Effect-aware semantic validation.
2. Structured concurrency enforcement surfaces.
3. Capability/effect consistency checks.

### P2

1. Performance and warning debt reduction.
2. Expanded tensor/accelerator groundwork.

## 7. Implementation Work Started In This Iteration

This iteration starts v2 implementation directly in the compiler frontend by adding parser support for function effect clauses (e.g. `fn f() -> T / io + async`).

Scope of this started work:
1. Accept effect clause syntax in normal and extern function signatures.
2. Add parser regression tests for effect clauses.
3. Preserve existing behavior while enabling the new syntax surface.

## 8. Definition Of Done (Unchanged Gate Truth)

Omni can be declared complete only when all are true:
1. Stage1 bytecode execution produces valid Stage3 artifact reproducibly.
2. Stage1 == Stage2 == Stage3 parity holds under clean-run commands.
3. CI enforces equivalent closure gates and passes.
4. Core v2 requirement surfaces (effect parsing/checking, ownership safety, capability boundaries) are implemented and verified.
5. This file is updated with evidence and closure references.

## 9. Documentation Policy

1. This file is the only authoritative root-level detailed status/specification source.
2. Other root historical reports stay archived only.
3. Detailed per-subsystem evidence belongs in code, tests, and Memory subtree logs.
