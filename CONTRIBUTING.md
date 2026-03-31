# Contributing to Helios + Omni

Thank you for your interest in contributing! This guide covers setup, code style, testing, and the PR process.

---

## Quick Start

```bash
# 1. Fork the repo on GitHub

# 2. Clone your fork
git clone https://github.com/YOUR_USERNAME/Helios.git
cd Helios

# 3. Add upstream
git remote add upstream https://github.com/shreyashjagtap157/Helios.git

# 4. Build the compiler
cd omni-lang/compiler
cargo build

# 5. Run tests
cargo test

# 6. Create a feature branch
git checkout dev
git checkout -b feature/your-feature-name
```

---

## Prerequisites

- **Rust 1.75+** (edition 2021) — [Install](https://rustup.rs/)
- **Git**
- Optional: LLVM 17 (for native codegen), CUDA/OpenCL/Vulkan (for GPU backend)

---

## Branch Strategy

- **`main`** — Stable. All releases come from here.
- **`dev`** — Active development. All PRs target this branch.
- **`feature/*`** — Your work. Branch from `dev`.

Always branch from `dev`:
```bash
git checkout dev
git pull upstream dev
git checkout -b feature/your-feature-name
```

---

## Code Style

### Rust

```bash
# Format
cargo fmt

# Lint
cargo clippy -- -D warnings
```

- `snake_case` for functions/variables
- `PascalCase` for types/traits
- `SCREAMING_SNAKE_CASE` for constants
- Doc comments for all public APIs

### Omni

```bash
# Format
cargo run --bin omni-fmt -- file.omni
```

- 4-space indentation
- `snake_case` for functions/variables
- `PascalCase` for types/structs
- `///` doc comments for public functions

---

## Testing

```bash
# All tests
cargo test

# Lib tests only
cargo test --lib

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture
```

**Rules:**
- All new functionality must have tests
- All tests must pass before merging
- Run `cargo clippy -- -D warnings` before submitting

---

## Pull Request Process

### Before You Submit

1. **Branch from `dev`**
2. **Run tests:** `cargo test`
3. **Run clippy:** `cargo clippy -- -D warnings`
4. **Run format check:** `cargo fmt --check`
5. **Write/update tests** for your changes
6. **Update docs** if user-facing

### PR Template

Your PR should include:
- **What** — one-line summary
- **Why** — link to issue (Closes #123)
- **How tested** — commands run and results
- **Checklist** — tests pass, clippy clean, docs updated

### After Approval

1. Squash-merge to `dev`
2. Delete your branch
3. Periodic merges from `dev` to `main`

---

## Commit Messages

Use conventional commits:
```
feat(parser): add support for new syntax
fix(type-check): resolve inference issue with generics
test(integration): add test for new feature
docs(readme): update installation instructions
refactor(semantic): extract type checking logic
```

---

## What to Work On

### Project-Wide Issues
- [Critical Project Issues](../ISSUES.md#critical-project-issues) — blockers affecting all components

### Omni Compiler Issues
- [Compiler-Specific Issues](omni-lang/ISSUES.md) — lexer, parser, type inference, codegen
  - [Self-Hosting Blockers](omni-lang/ISSUES.md#-critical-self-hosting-blockers)
  - [Good First Issues](omni-lang/ISSUES.md#good-first-issues)
  - [Compiler Errors](omni-lang/ISSUES.md#compiler-error-reference)

---

## License

By contributing, you agree your contributions are licensed under Apache 2.0.

```
Copyright 2024 Shreyash Jagtap
Licensed under the Apache License, Version 2.0
```
