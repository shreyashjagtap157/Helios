# API Conventions

## Design Principles

- Prefer explicit contracts and predictable behavior.
- Keep interfaces stable where possible.
- Surface errors with actionable context.

## Compatibility

- Avoid breaking changes unless explicitly required.
- If breaking change is unavoidable, document migration path.
- Keep naming and parameter semantics consistent.

## Validation

- Validate call paths touched by API changes.
- Update docs/examples when signatures or behavior change.

## Governance

- Align API behavior with authoritative project docs.
- Avoid hidden side effects in API boundaries.
