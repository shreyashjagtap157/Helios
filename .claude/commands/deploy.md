# /project:deploy

Plan and execute a deployment-oriented release workflow for Helios/Omni artifacts.

## Purpose

Prepare code, docs, and operational checks for controlled rollout.

## Inputs

- Target environment (`dev`, `staging`, `prod`)
- Release scope (modules/features)
- Rollback strategy availability

## Deployment Checklist

1. Confirm release scope and changelog summary.
2. Confirm build outputs are current and reproducible.
3. Confirm critical tests/checks for touched areas.
4. Confirm documentation and migration notes.
5. Validate runtime compatibility constraints.
6. Execute staged rollout with observation gates.
7. Verify post-deploy health signals.
8. Confirm rollback readiness.

## Output

- Release readiness verdict
- Blockers (if any)
- Rollout plan
- Monitoring/rollback notes

## Constraints

- No production deployment recommendations without explicit validation evidence.
- Flag risky assumptions clearly.
