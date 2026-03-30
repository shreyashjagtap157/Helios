# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added
- Apache 2.0 license (`LICENSE`, `NOTICE`)
- `SECURITY.md` — vulnerability reporting policy
- `CODE_OF_CONDUCT.md` — Contributor Covenant v2.0
- `CONTRIBUTING.md` — setup, code style, PR process, commit conventions
- `ISSUES.md` — 10 good first issues, 8 help wanted, all open issues categorized
- `.github/workflows/ci.yml` — GitHub Actions CI (check, test, clippy, fmt, build)
- `.github/ISSUE_TEMPLATE/` — bug report, feature request, question templates
- `.github/PULL_REQUEST_TEMPLATE.md` — checklist-based PR template
- `.github/labels.yml` — 24 labels across priority, type, status, component, area
- SPDX license identifier in `Cargo.toml` (`Apache-2.0`)
- Apache 2.0 copyright headers in `src/main.rs` and `src/lib.rs`
- CI status badge in README

### Changed
- `README.md` — rewritten with verified numbers, problem statement, quick start
- `.gitignore` — cleaned up stale references, added force-track rules
- `Cargo.toml` — `license` field changed from `Proprietary` to `Apache-2.0`

### Removed
- `AUDIT.md` — data folded into `ISSUES.md`
- `CLAUDE.md` — internal GitNexus config, not for open source
- `config/` — root-level config, not core project
- `docs/` — duplicates of `omni-lang/docs/` and README content
- `examples/` — duplicates of `omni-lang/examples/`
- `.claude/` — Claude Code internal config
- `.vscode/` — IDE settings
- `.gitnexus/` — GitNexus cache
- `Memory/` — personal Obsidian vault

### Verified (2026-03-29)
- Tests: 1,019 passing (547 lib + 472 bin), 0 failures
- Clippy: ~109 unique warnings
- Self-hosted compiler: 34 .omni files, 28,433 lines
- Standard library: 37 modules, 21,617 lines
- `hello.omni`: works with warnings
- `minimal.omni`: works clean
