# /project:fix-issue

Fix an issue using a root-cause-first workflow.

## Purpose

Resolve bugs or defects in Helios/Omni while preserving subsystem boundaries and minimizing collateral changes.

## Inputs

- Issue description (symptom, expected behavior, observed behavior)
- Scope hint (optional file/module)
- Severity and urgency

## Workflow

1. Reproduce or isolate symptom path.
2. Identify likely subsystem and entry points.
3. Confirm root cause with evidence.
4. Implement the smallest sufficient fix.
5. Verify with targeted checks/tests.
6. Update related docs/notes if behavior changed.

## Expected Output

- Root cause explanation
- Files changed
- Validation performed
- Remaining risks / follow-up tasks

## Constraints

- Avoid unrelated refactors.
- Prefer deterministic and auditable logic.
- Keep APIs stable unless issue explicitly requires breaking change.
