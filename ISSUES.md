# Helios + Omni Issues & Tasks

> **New contributor?** Start with a [Good First Issue](#good-first-issues).  
> **Experienced?** See [Help Wanted](#help-wanted).

---

## Good First Issues

Well-scoped tasks for newcomers. Each touches a single area and has clear steps.

### GFI-001: Fix Unnecessary Parentheses

**Labels:** `good first issue`, `component: compiler`  
**Difficulty:** Easy  

Clippy flags unnecessary parentheses in the parser module. Remove them so the warning disappears.

```
cargo clippy --lib 2>&1 | grep "unnecessary parentheses"
```

---

### GFI-002: Add Default Implementations for Brain Types

**Labels:** `good first issue`, `component: helios`  
**Difficulty:** Easy  
**Files:** `omni-lang/compiler/src/brain/`  

`AdaptiveReasoner`, `KnowledgeGraph`, `MemorySystem`, `PatternRecognizer` need `impl Default`. Use their existing `new()` constructors.

---

### GFI-003: Collapse Collapsible If Statements

**Labels:** `good first issue`, `component: compiler`  
**Difficulty:** Easy  

```
cargo clippy --lib 2>&1 | grep "collapsible"
```

---

### GFI-004: Replace Manual Suffix Stripping with strip_suffix

**Labels:** `good first issue`, `component: compiler`  
**Difficulty:** Easy  

Replace manual string suffix operations with `.strip_suffix("...")`.

---

### GFI-005: Replace or_insert_with(Default::default) with or_default

**Labels:** `good first issue`, `component: compiler`  
**Difficulty:** Easy  

```
grep -r "or_insert_with(Default::default)" omni-lang/compiler/src/
```

---

### GFI-006: Prefix Unused Variables with Underscore

**Labels:** `good first issue`, `component: compiler`  
**Difficulty:** Easy  

Clippy reports ~40 unused variables. Prefix each with `_` to suppress the warning.

```
cargo clippy --lib 2>&1 | grep "unused variable"
```

---

### GFI-007: Remove Unused Imports

**Labels:** `good first issue`, `component: compiler`  
**Difficulty:** Easy  

```
cargo clippy --lib 2>&1 | grep "unused import"
```

---

### GFI-008: Add Doc Comments to One Stdlib Module

**Labels:** `good first issue`, `component: stdlib`, `documentation`  
**Difficulty:** Easy–Medium  
**Files:** `omni-lang/std/*.omni`  

Pick one module (e.g. `math.omni`) and add `///` doc comments to all public functions.

---

### GFI-009: Fix tutorial_01_basics.omni

**Labels:** `good first issue`, `area: examples`, `bug`  
**Difficulty:** Medium  

Fails with "Unsupported binary operation RangeInclusive". Either fix the range support or rewrite the tutorial to avoid it.

---

### GFI-010: Add CI Badge to README

**Labels:** `good first issue`, `documentation`  
**Difficulty:** Easy  
**Status:** ✅ Done  

Badge already added in README.

---

## Help Wanted

Tasks needing extra attention or broader expertise.

### HW-001: Fix All ~109 Clippy Warnings

**Labels:** `help wanted`, `component: compiler`  
**Difficulty:** Medium  

Warning categories:
- 40 unused variables
- 10 redundant closures
- 19 first-element access (`args.get(0)` instead of `args.first()`)
- 12 `div_ceil` reimplementations
- 8 dead code items (modes.rs)
- 5 complex types needing aliases
- 4 `or_insert_with` → `or_default`
- 4 unreachable patterns
- Misc: collapsible ifs, manual suffix stripping, unnecessary returns

Suggested approach: fix one module at a time.

---

### HW-002: Add Tests for omni-lsp

**Labels:** `help wanted`, `component: tooling`  
**Difficulty:** Medium  

2,305 lines, 0 tests. Add tests for document sync, completion, hover, diagnostics.

---

### HW-003: Add Tests for opm (Package Manager)

**Labels:** `help wanted`, `component: tooling`  
**Difficulty:** Medium  

2,765 lines, 0 tests. Add tests for manifest parsing, dependency resolution.

---

### HW-004: Fix Boolean Logic Crash

**Labels:** `help wanted`, `bug`, `area: runtime`  
**Difficulty:** Hard  

`integration_test.omni` crashes on boolean logic operations. Debug and fix.

```
cargo run --bin omnc -- --run ../examples/integration_test.omni --verbose
```

---

### HW-005: Fix Integration Test Failures (Stack, Array)

**Labels:** `help wanted`, `bug`, `area: runtime`  
**Difficulty:** Hard  

Two tests fail in `integration_test.omni`. Trace through the interpreter/VM and fix.

---

### HW-006: Implement HashMap Iteration

**Labels:** `help wanted`, `feature`, `component: stdlib`  
**Difficulty:** Hard  

`tutorial_04_collections.omni` fails: "Cannot iterate over Map". Design and implement `for` iteration over HashMap.

---

### HW-007: Fix undefined variable: math in tutorial_03

**Labels:** `help wanted`, `bug`, `area: semantic`  
**Difficulty:** Medium  

Tutorial uses `math` module but gets "undefined variable: math". Fix module resolution.

---

### HW-008: Expand omni-fmt Test Suite

**Labels:** `help wanted`, `component: tooling`  
**Difficulty:** Easy–Medium  

3 tests exist. Add idempotent formatting tests, edge cases, invalid input handling.

---

## Open Issues

### Critical (self-hosting blockers)

| ID | Description | Status |
|----|-------------|--------|
| C-001 | Self-hosted compiler cannot be parsed by omnc | Open |
| C-002 | Bootstrap stages 1-2 are placeholders (copies of stage0) | Open |
| C-003 | IR hardcodes I64 instead of preserving actual types | Open |
| C-004 | No binary emission — only runtime execution supported | Open |

### High Priority

| ID | Description | Status |
|----|-------------|--------|
| H-001 | Monomorphization not implemented | Open |
| H-002 | No standalone Helios binary | Open |
| H-003 | No installer for major platforms | Open |
| H-004 | Integration test has 2 failures + crash | Open |
| H-005 | 3 of 5 tutorial examples fail | Open |
| H-006 | ~109 clippy warnings | Open |
| H-007 | No CI/CD beyond basic GitHub Actions | In Progress |
| H-008 | No versioned release process | Open |

### Medium Priority

| ID | Description | Status |
|----|-------------|--------|
| M-001 | Type inference produces 100+ warnings on complex code | Open |
| M-002 | Borrow checker produces 20+ warnings on complex code | Open |
| M-003 | Only omni-fmt has tests (3). LSP/DAP/opm have 0 | Open |
| M-004 | GPU backend untested | Open |
| M-005 | LLVM backend untested | Open |
| M-006 | Helios framework is scaffolding | In Progress |

### Low Priority

| ID | Description | Status |
|----|-------------|--------|
| L-001 | More example programs needed | Open |
| L-002 | Performance benchmarks | Open |
| L-003 | Cross-platform testing (only Windows tested) | Open |

---

## Label Reference

| Label | Meaning |
|-------|---------|
| `good first issue` | Beginner-friendly, well-scoped |
| `help wanted` | Needs community help |
| `bug` | Something is broken |
| `enhancement` | Improvement to existing feature |
| `feature` | New capability |
| `priority: critical` | Blocks core functionality |
| `priority: high` | Should fix soon |
| `priority: medium` | Important but not urgent |
| `priority: low` | Nice to have |
| `component: compiler` | Omni compiler |
| `component: stdlib` | Standard library |
| `component: helios` | Helios framework |
| `component: tooling` | LSP, DAP, formatter, opm |

---

**Last updated:** 2026-03-29
