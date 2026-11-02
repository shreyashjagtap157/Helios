# Helios Security and Safety Policy

**Status:** Design requirements only. No security implementation currently exists.

## Core rules

1. Default deny for hardware, operating-system and sensitive-data operations.
2. Least privilege and narrowly scoped, revocable capabilities.
3. The cognitive reasoning process cannot grant itself authority.
4. Privileged operations are performed by a separate trusted service.
5. External documents, websites, messages, files, plugins and device responses are untrusted data.
6. Imported content cannot override user policy or authorization requirements.
7. Consequential operations require explicit authorization under a defined policy.
8. Physical actions require risk classification, safety interlocks and human override.
9. Unknown device state, failed authorization or uncertain preconditions block consequential actions.
10. Every attempted action and its result must be recorded.

## Device action controls

A device adapter must define its identity, operations, parameter bounds, preconditions, expected effects, timeouts, failure behavior and safety class.

No natural-language request may directly become an unrestricted shell command, memory operation, register write or privileged system call.

The action broker must independently enforce permissions, device allowlists, operation limits, capability expiry, timeouts and rate limits. It must support emergency stop and explicit revocation.

## Plugin controls

Plugins must declare identity, version, interfaces, requested permissions, network/filesystem/device access and resource limits. Plugins must be isolated and must not alter the trusted policy boundary.

## Data protection

Sensitive data should be local by default, access-controlled, backed up according to user policy and deleted through explicit retention rules. Secrets must not appear in logs. Network connections must authenticate endpoints where supported.

## Failure handling

A command sent is not a verified result. A timeout is not success. Partial execution must be represented explicitly. Unknown hardware state must be treated as unsafe for consequential operations.

## Vulnerability handling

Security issues should be reported privately to the project owner until a formal disclosure process is established. Do not publish secrets, device credentials or exploit details in public issue reports.
