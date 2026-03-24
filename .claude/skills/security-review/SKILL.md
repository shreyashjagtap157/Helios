---
name: security-review
description: "Use when reviewing security posture, trust boundaries, secrets handling, input validation, or vulnerability risk in Helios/Omni code and docs."
---

# Security Review Skill

## Goal

Provide a structured security assessment for changes or modules with concrete, prioritized findings.

## Review Dimensions

1. Input validation and parsing safety
2. Trust boundary and permission model
3. Secrets handling and sensitive-data exposure
4. Dependency/tooling risk indicators
5. Unsafe operations and escalation paths
6. Logging/telemetry leakage risk

## Procedure

1. Identify attack surface for the scoped change.
2. Trace data flow from input to critical operations.
3. Evaluate boundary checks and failure handling.
4. Flag risky assumptions and missing controls.
5. Recommend precise mitigations ranked by impact.

## Output Format

- **Risk Summary**
- **Findings** (Critical/High/Medium/Low)
- **Evidence Paths**
- **Mitigations**
- **Verification Steps**

## Constraints

- Avoid speculative claims without evidence.
- Mark uncertain items as hypotheses requiring confirmation.
