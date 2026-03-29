---
name: cluster-41
description: "Skill for the Cluster_41 area of Helios. 13 symbols across 1 files."
---

# Cluster_41

13 symbols | 1 files | Cohesion: 84%

## When to Use

- Working with code in `omni-lang/`
- Understanding how new, find_definition, find_references work
- Modifying cluster_41-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `omni-lang/tools/omni-lsp/src/main_v0.rs` | new, find_definition, find_references, get_completions, get_hover (+8) |

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `new` | Function | `omni-lang/tools/omni-lsp/src/main_v0.rs` | 105 |
| `find_definition` | Function | `omni-lang/tools/omni-lsp/src/main_v0.rs` | 360 |
| `find_references` | Function | `omni-lang/tools/omni-lsp/src/main_v0.rs` | 376 |
| `get_completions` | Function | `omni-lang/tools/omni-lsp/src/main_v0.rs` | 430 |
| `get_hover` | Function | `omni-lang/tools/omni-lsp/src/main_v0.rs` | 532 |
| `word_at_position` | Function | `omni-lang/tools/omni-lsp/src/main_v0.rs` | 587 |
| `format_document` | Function | `omni-lang/tools/omni-lsp/src/main_v0.rs` | 614 |
| `rename_symbol` | Function | `omni-lang/tools/omni-lsp/src/main_v0.rs` | 669 |
| `get_signature_help` | Function | `omni-lang/tools/omni-lsp/src/main_v0.rs` | 707 |
| `extract_params_from_signature` | Function | `omni-lang/tools/omni-lsp/src/main_v0.rs` | 786 |
| `get_code_actions` | Function | `omni-lang/tools/omni-lsp/src/main_v0.rs` | 801 |
| `main` | Function | `omni-lang/tools/omni-lsp/src/main_v0.rs` | 903 |
| `handle_message` | Function | `omni-lang/tools/omni-lsp/src/main_v0.rs` | 951 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Handle_message → Bucket` | cross_community | 4 |
| `Handle_message → RpcResponse` | cross_community | 3 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Runtime | 3 calls |
| Brain | 2 calls |
| Cluster_42 | 1 calls |
| Codegen | 1 calls |

## How to Explore

1. `gitnexus_context({name: "new"})` — see callers and callees
2. `gitnexus_query({query: "cluster_41"})` — find related execution flows
3. Read key files listed above for implementation details
