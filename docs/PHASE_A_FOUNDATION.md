# Helios Phase A — Symbolic Foundation

**Status:** Proposed and specified; not implemented.  
**Authority:** HELIOS_MASTER_SPEC.md.

## Goal

Prove a small deterministic, non-neural symbolic core that accepts typed facts, records provenance, evaluates one bounded rule set, records derivations, preserves conflicting evidence and expires task-scoped state. Phase A must operate with the Neural Perception Boundary disabled.

## Scope

- Typed identifiers, entities, claims, relations and source records.
- Task-local evidence ledger and provenance.
- One rule-evaluation path with deterministic input ordering and canonical output.
- Explicit claim states and derivation links.
- Contradiction representation without overwriting evidence.
- Task-scoped memory allocation, TTL and cleanup.
- Repeatable fixture replay.

## Deferred

Do not integrate perception models, embeddings, vector retrieval, SMT, ASP, general planners, durable episodic/semantic memory, automatic consolidation, hardware actions or OS components in this phase.

## Technology spike

Rust is a candidate, not yet selected. Compare a minimal Rust implementation and one rule-evaluation option against a suitable alternative. In particular, compare Soufflé with a Rust-native Datalog/rule engine using the same typed-fact/provenance fixture. Record versions, licenses, dependency graphs, maintenance, offline behavior and measured results before choosing. Select one path only; do not add multiple solver families in Phase A.

## Acceptance tests

| ID | Test | Expected result |
|---|---|---|
| A-001 | Replay the same fixture 100 times | Byte-identical canonical result and trace |
| A-002 | Change fact input order | Same canonical result |
| A-003 | Submit duplicate claim | Idempotent or explicitly reported |
| A-004 | Add opposing evidence | Both claims remain represented |
| A-005 | Retract a premise | Dependent conclusions become stale |
| A-006 | Exceed a resource budget | Bounded stop, never a false proof |
| A-007 | End a task | Temporary state and derived caches expire |
| A-008 | Restart without a persistence grant | No cognitive memory is restored |
| A-009 | Disable NPB | Symbolic tests still pass |
| A-010 | Supply malformed or unknown fields | Reject before inference |

## Deliverables and exit gate

Deliver a reviewed technology selection, minimal workspace, typed record definitions, one deterministic rule engine, provenance/derivation traces, fixtures and tests. Do not create the supported workspace until the selection spike is complete. A directory or code skeleton is not acceptance evidence.

Phase A passes only when every test above is runnable and its result is recorded. Phase B may then prototype NPB integration against the versioned PerceptionObservation contract.
