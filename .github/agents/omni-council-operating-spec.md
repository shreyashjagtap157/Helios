# Omni Council Operating Spec

This file defines the shared, non-optional execution contract for the Omni council and all specialist agents.

## Purpose
- Remove role ambiguity and hidden assumptions.
- Standardize intake, execution, verification, and handoff behavior.
- Keep all work aligned with Omni self-hosting goals.

## Universal Intake Schema
Every agent response must normalize requests into:
1. Problem statement
2. Work type
3. Target phase
4. Acceptance criteria
5. Constraints and non-goals
6. Risk class
7. Required artifacts

## Universal Evidence Policy
- Claims without evidence are invalid.
- Verification status must be one of: pass, fail, not-run.
- Performance claims require baseline and post-change metrics.
- Self-hosting progress claims require reproducible checkpoint evidence.

## Universal Handoff Schema
Every delegated track must define:
1. Inputs
2. Deliverable
3. Required handoff artifact
4. Join condition
5. Conflict trigger

## Universal Merge Order
1. Contract/invariant alignment
2. Behavioral correctness
3. Performance implications
4. Verification gate status
5. Self-hosting impact

## Continuous Loop Protocol
For long-running project advancement, execute iterative cycles:
1. Check current state
2. Plan next highest-impact slice
3. Implement scoped change
4. Test and verify outcomes
5. Re-check against final requirements

Continuation rules:
- Continue until final requirement criteria are fully met or a hard blocker is reached.
- If two consecutive loops produce no meaningful delta, trigger strategy reset with architecture and verification review.

Loop reporting requirements:
- Iteration ID
- Actions taken
- Verification status
- Remaining gaps
- Next loop trigger

## Universal Memory Update Policy
- Decision-impacting work updates `Memory/20-Decisions/`.
- Implementation and verification work updates `Memory/30-Execution/`.
- Self-hosting-impacting work updates `Memory/40-Self-Hosting/`.
- Milestone focus updates `Memory/00-Index/`.

## Universal Done Criteria
A task is done only when all applicable conditions are met:
- Scope and phase explicit
- Role assignment explicit
- Artifacts produced or explicitly waived
- Evidence-backed verification status recorded
- Memory updates identified
- Next self-hosting milestone stated
