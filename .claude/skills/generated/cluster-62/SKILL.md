---
name: cluster-62
description: "Skill for the Cluster_62 area of Helios. 10 symbols across 1 files."
---

# Cluster_62

10 symbols | 1 files | Cohesion: 71%

## When to Use

- Working with code in `omni-lang/`
- Understanding how new, read_message, handle_set_breakpoints work
- Modifying cluster_62-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `omni-lang/tools/omni-dap/src/main.rs` | new, read_message, handle_set_breakpoints, handle_configuration_done, handle_threads (+5) |

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `new` | Function | `omni-lang/tools/omni-dap/src/main.rs` | 49 |
| `read_message` | Function | `omni-lang/tools/omni-dap/src/main.rs` | 70 |
| `handle_set_breakpoints` | Function | `omni-lang/tools/omni-dap/src/main.rs` | 202 |
| `handle_configuration_done` | Function | `omni-lang/tools/omni-dap/src/main.rs` | 237 |
| `handle_threads` | Function | `omni-lang/tools/omni-dap/src/main.rs` | 241 |
| `handle_stack_trace` | Function | `omni-lang/tools/omni-dap/src/main.rs` | 253 |
| `handle_scopes` | Function | `omni-lang/tools/omni-dap/src/main.rs` | 272 |
| `handle_variables` | Function | `omni-lang/tools/omni-dap/src/main.rs` | 285 |
| `handle_disconnect` | Function | `omni-lang/tools/omni-dap/src/main.rs` | 340 |
| `main` | Function | `omni-lang/tools/omni-dap/src/main.rs` | 347 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Main → DapMessage` | cross_community | 4 |
| `Main → Next_seq` | cross_community | 4 |
| `Main → New` | intra_community | 3 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Cluster_63 | 6 calls |
| Runtime | 2 calls |
| Brain | 1 calls |

## How to Explore

1. `gitnexus_context({name: "new"})` — see callers and callees
2. `gitnexus_query({query: "cluster_62"})` — find related execution flows
3. Read key files listed above for implementation details
