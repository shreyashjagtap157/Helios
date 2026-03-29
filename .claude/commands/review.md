# /project:review

Run a full project-aware review for Helios/Omni changes.

## Purpose

Perform a structured review that checks correctness, architecture alignment, determinism constraints, safety/governance impact, and documentation completeness.

## Inputs

- Target scope (file, folder, or feature)
- Optional branch or changed-file list
- Optional priority (`quick`, `standard`, `deep`)

## Review Procedure

1. Identify changed files and map to subsystem boundaries (`omni-lang`, `helios-framework`, `docs`, `Memory`).
2. Validate implementation behavior against authoritative docs:
   - `docs/HELIOS & Omni Language — Comprehensive.md`
   - `docs/IMPLEMENTATION_STATUS_REPORT.md`
3. Check determinism and architecture constraints:
   - no hidden nondeterministic behavior in core pipeline,
   - no undocumented behavior drift.
4. Validate API contracts and compatibility impact.
5. Validate tests and diagnostics relevance.
6. Validate docs and vault note updates for user-facing changes.

## Output Format

- **Summary**: one-paragraph verdict
- **Findings**: prioritized (Critical/High/Medium/Low)
- **Evidence**: file paths + rationale
- **Recommended Fixes**: concrete actions
- **Residual Risk**: what still needs human validation

## Guardrails

- Do not auto-approve risky changes without explicit evidence.
- Do not suggest broad refactors unless required to fix root cause.
