---
name: tests
description: "Skill for the Tests area of Helios. 20 symbols across 3 files."
---

# Tests

20 symbols | 3 files | Cohesion: 65%

## When to Use

- Working with code in `omni-lang/`
- Understanding how close_document, format_document, get_diagnostics work
- Modifying tests-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `omni-lang/tools/omni-lsp/src/server.rs` | close_document, format_document, get_diagnostics, format_omni_code, new (+9) |
| `omni-lang/tests/comprehensive_test_suite.rs` | test_lsp_document_operations, test_full_development_workflow, test_lsp_code_completion, test_lsp_hover_and_navigation, test_operator_overloading_registry |
| `omni-lang/compiler/src/language_features/operator_overloading.rs` | overloads_for_type |

## Entry Points

Start here when exploring this area:

- **`close_document`** (Function) — `omni-lang/tools/omni-lsp/src/server.rs:129`
- **`format_document`** (Function) — `omni-lang/tools/omni-lsp/src/server.rs:328`
- **`get_diagnostics`** (Function) — `omni-lang/tools/omni-lsp/src/server.rs:364`
- **`new`** (Function) — `omni-lang/tools/omni-lsp/src/server.rs:106`
- **`completions`** (Function) — `omni-lang/tools/omni-lsp/src/server.rs:259`

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `close_document` | Function | `omni-lang/tools/omni-lsp/src/server.rs` | 129 |
| `format_document` | Function | `omni-lang/tools/omni-lsp/src/server.rs` | 328 |
| `get_diagnostics` | Function | `omni-lang/tools/omni-lsp/src/server.rs` | 364 |
| `new` | Function | `omni-lang/tools/omni-lsp/src/server.rs` | 106 |
| `completions` | Function | `omni-lang/tools/omni-lsp/src/server.rs` | 259 |
| `find_references` | Function | `omni-lang/tools/omni-lsp/src/server.rs` | 318 |
| `rename` | Function | `omni-lang/tools/omni-lsp/src/server.rs` | 337 |
| `open_document` | Function | `omni-lang/tools/omni-lsp/src/server.rs` | 117 |
| `hover` | Function | `omni-lang/tools/omni-lsp/src/server.rs` | 222 |
| `overloads_for_type` | Function | `omni-lang/compiler/src/language_features/operator_overloading.rs` | 210 |
| `test_lsp_document_operations` | Function | `omni-lang/tests/comprehensive_test_suite.rs` | 221 |
| `test_full_development_workflow` | Function | `omni-lang/tests/comprehensive_test_suite.rs` | 313 |
| `format_omni_code` | Function | `omni-lang/tools/omni-lsp/src/server.rs` | 386 |
| `test_lsp_code_completion` | Function | `omni-lang/tests/comprehensive_test_suite.rs` | 245 |
| `test_lsp_completions` | Function | `omni-lang/tools/omni-lsp/src/server.rs` | 438 |
| `test_lsp_hover_and_navigation` | Function | `omni-lang/tests/comprehensive_test_suite.rs` | 266 |
| `test_lsp_open_document` | Function | `omni-lang/tools/omni-lsp/src/server.rs` | 417 |
| `test_lsp_hover` | Function | `omni-lang/tools/omni-lsp/src/server.rs` | 429 |
| `test_format_code` | Function | `omni-lang/tools/omni-lsp/src/server.rs` | 448 |
| `test_operator_overloading_registry` | Function | `omni-lang/tests/comprehensive_test_suite.rs` | 51 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Cluster_31 | 1 calls |
| Brain | 1 calls |

## How to Explore

1. `gitnexus_context({name: "close_document"})` — see callers and callees
2. `gitnexus_query({query: "tests"})` — find related execution flows
3. Read key files listed above for implementation details
