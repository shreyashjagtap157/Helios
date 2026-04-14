# Phase 0 + Phase 1 Training Arc Commandbook

This commandbook provides exact shell commands and script invocations for the requested repository sanitization and foundational hardening checks.

## Preconditions

```bash
cd /path/to/Project/Helios
```

## Phase 0.1 - Repository Sanitization

Run the full sanitization script:

```bash
bash scripts/phase0_repository_sanitize.sh .
```

What this performs:
- `git fsck --full --strict`
- reflog expiration + object pruning + aggressive gc/repack
- `cargo clean` for every discovered `Cargo.toml`
- full git index rebuild (`rm .git/index` + `git reset --mixed HEAD`)
- recursive readability validation of tracked files

Acceptance check:
- Script exits successfully and prints: `phase0.1 completed: 0 unreadable tracked files`

## Phase 0.2 - HELIOS Quarantine

Extract `helios-framework/` into an external repository while preserving split history:

```bash
bash scripts/phase0_helios_quarantine.sh ../helios-framework-repo .
```

What this performs:
- `git subtree split --prefix=helios-framework`
- initializes/fetches external HELIOS repo and checks out `main`
- removes `helios-framework/` from source repo
- scrubs HELIOS references from `omni-lang/Cargo.toml`

Post-quarantine validation:

```bash
grep -Ein "helios|helios-framework" omni-lang/Cargo.toml || true
cargo build --workspace --manifest-path omni-lang/Cargo.toml
```

Acceptance check:
- no HELIOS references in `omni-lang/Cargo.toml`
- workspace build completes without HELIOS dependencies

## Phase 1 - Hardening Verification Commands

### OPM trust + orchestration checks

```bash
cargo test --manifest-path omni-lang/tools/opm/Cargo.toml
```

### Compiler borrow-check path sanity

```bash
cargo test --manifest-path omni-lang/compiler/Cargo.toml phase_3_tests::test_polonius_stress_artifact_present
```

### Stdlib cfg and alloc split sanity

```bash
grep -n "target_os" omni-lang/std/io.omni
grep -n "module std::alloc" omni-lang/std/alloc.omni
grep -n "pub use std::alloc" omni-lang/std/core.omni
```

## Optional full pass

```bash
cargo test --workspace --manifest-path omni-lang/Cargo.toml
```
