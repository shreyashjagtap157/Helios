# Testing Rules

## Test Strategy

- Start with the smallest relevant validation first.
- Expand to broader checks only after targeted confidence is achieved.
- Validate only impacted areas unless requested otherwise.

## Expectations

- Every behavior change should have an explicit verification path.
- For bug fixes, verify the failing path and nearby regressions.
- If test gaps remain, document them clearly.

## Constraints

- Do not fix unrelated failing tests by default.
- Do not claim verification without concrete check evidence.

## Reporting

- Report what was verified, what was not, and why.
- Call out residual risk when full validation is unavailable.
