---
name: deploy
description: "Use when preparing release readiness, rollout plans, deployment checklists, post-deploy verification, and rollback strategy for Helios/Omni changes."
---

# Deploy Skill

## Goal

Drive reliable, staged, and observable deployment decisions with explicit readiness gates.

## Phases

1. **Scope Freeze**: confirm exact release contents and dependencies.
2. **Build & Verify**: validate build artifacts and critical checks.
3. **Readiness Gate**: ensure docs/config/migration readiness.
4. **Staged Rollout**: dev → staging → production progression.
5. **Post-Deploy Verification**: health indicators + functional checks.
6. **Rollback Preparedness**: clear rollback trigger and procedure.

## Output

- Readiness decision
- Required preconditions
- Rollout sequence
- Monitoring checklist
- Rollback conditions and steps

## Constraints

- Never mark production-ready without explicit verification evidence.
- Escalate unknowns and blockers clearly.
