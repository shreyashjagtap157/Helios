# Helios + Omni Issues

> **For compiler/language-specific issues, see:** [omni-lang/ISSUES.md](omni-lang/ISSUES.md)

---

## Executive Summary

**Helios** is a cognitive framework. **Omni** is a systems programming language that compiles itself.

### Project Status

| Component | Status |
|-----------|--------|
| Omni Compiler (Rust-based) | ✅ Working |
| Bytecode Emission | ✅ Working |
| OVM Runtime | ✅ Working |
| Self-Hosted Compiler | ✅ Working (minimal) |
| Bootstrap Pipeline | ⚠️ Partial |
| True Self-Hosting | ❌ Not Achieved |

---

## Critical Project Issues

These are **cross-cutting concerns** affecting the entire Helios + Omni project.

### CP-001: True Self-Hosting Not Achieved

**Status:** 🔴 CRITICAL  
**Component:** All

Omni cannot yet compile itself without Rust.

**What's needed:**
1. Fix type inference for arithmetic in function returns
2. Implement bootstrap stages 1 and 2
3. Verify bit-identical output (proof of correct bootstrap)

**See:** [omni-lang/ISSUES.md](omni-lang/ISSUES.md) → SH-001, SH-002

---

### CP-002: Limited Type Inference

**Status:** 🔴 CRITICAL  
**Component:** Compiler, Semantic Analysis

Type inference fails on:
- Function returns with arithmetic (`return a + b`)
- Parameters without explicit types
- Complex expressions

**See:** [omni-lang/ISSUES.md](omni-lang/ISSUES.md) → HP-001, HP-002, HP-003

---

### CP-003: No Native Binary Emission

**Status:** 🔴 CRITICAL  
**Component:** Codegen

`omnc --emit native` doesn't produce working executables.

**See:** [omni-lang/ISSUES.md](omni-lang/ISSUES.md) → SH-003

---

## Architecture Decisions

### Monorepo Structure

```
Helios/
├── ISSUES.md              # This file - project-wide issues
├── CONTRIBUTING.md         # Contribution guidelines
├── omni-lang/             # Omni Programming Language
│   ├── ISSUES.md         # Compiler/language-specific issues
│   ├── compiler/         # Rust-based compiler
│   ├── omni/             # Self-hosted compiler source
│   ├── std/              # Standard library
│   └── examples/         # Example programs
└── ...                   # Other Helios components
```

---

## Roadmap

### Phase 1: Working Compiler ✅
- [x] Basic lexer and parser
- [x] Type inference
- [x] Bytecode emission
- [x] OVM runtime

### Phase 2: Self-Hosting (Current) ⚠️
- [x] Bootstrap Stage 0 (Rust compiles Omni)
- [ ] Bootstrap Stage 1 (Omni compiles itself)
- [ ] Bootstrap Stage 2 (Verify bit-identical)
- [ ] Remove Rust dependency

### Phase 3: Production Ready
- [ ] Full type inference
- [ ] Native binary emission
- [ ] Complete standard library
- [ ] GPU backend

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup and development guidelines.

For specific tasks:
- **Compiler/Language issues:** [omni-lang/ISSUES.md](omni-lang/ISSUES.md)
- **Beginner-friendly tasks:** Look for `good first issue` labels

---

## Quick Links

- [Omni Compiler Issues](omni-lang/ISSUES.md)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [README.md](README.md)

---

**Last updated:** 2026-03-30
