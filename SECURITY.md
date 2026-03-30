# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| dev (development) | Yes |
| main (stable) | Yes |

## Reporting a Vulnerability

**Do NOT open a public GitHub issue for security vulnerabilities.**

Instead, report them privately:

1. **Email:** ssjagtap2016@gmail.com
2. **Subject:** `[SECURITY] Helios/Omni — <brief description>`

### What to Include

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

### Response Timeline

- **Acknowledgment:** within 48 hours
- **Initial assessment:** within 1 week
- **Fix or mitigation:** depends on severity

### Disclosure Policy

- We follow responsible disclosure
- We will credit reporters in the fix (unless they prefer anonymity)
- We will coordinate a public disclosure timeline with the reporter

## Security Considerations

This project is a programming language compiler and runtime. Key security surfaces:

- **Compiler:** input parsing, code generation, memory safety
- **Runtime:** sandboxing, file system access, network access
- **Package manager (opm):** dependency integrity, supply chain

If you find a vulnerability in any of these areas, please report it.
