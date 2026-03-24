---
name: cluster-60
description: "Skill for the Cluster_60 area of Helios. 11 symbols across 1 files."
---

# Cluster_60

11 symbols | 1 files | Cohesion: 61%

## When to Use

- Working with code in `omni-lang/`
- Understanding how evaluate work
- Modifying cluster_60-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `omni-lang/tools/omni-dap/src/main_v0.rs` | evaluate, handle_initialize, handle_set_exception_breakpoints, handle_variables, handle_evaluate (+6) |

## Entry Points

Start here when exploring this area:

- **`evaluate`** (Function) — `omni-lang/tools/omni-dap/src/main_v0.rs:208`

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `evaluate` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 208 |
| `handle_initialize` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 357 |
| `handle_set_exception_breakpoints` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 626 |
| `handle_variables` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 758 |
| `handle_evaluate` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 813 |
| `handle_threads` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 861 |
| `handle_set_variable` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 962 |
| `handle_source` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 979 |
| `handle_loaded_sources` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 996 |
| `handle_terminate` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 1046 |
| `success_response` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 1067 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Handle_evaluate → Bucket` | cross_community | 5 |
| `Handle_evaluate → New` | cross_community | 4 |
| `Handle_set_variable → Bucket` | cross_community | 4 |
| `Handle_set_function_breakpoints → DapMessage` | cross_community | 3 |
| `Handle_set_function_breakpoints → Next_seq` | cross_community | 3 |
| `Handle_stack_trace → DapMessage` | cross_community | 3 |
| `Handle_stack_trace → Next_seq` | cross_community | 3 |
| `Handle_read_memory → DapMessage` | cross_community | 3 |
| `Handle_read_memory → Next_seq` | cross_community | 3 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Brain | 4 calls |
| Cluster_58 | 2 calls |
| Runtime | 1 calls |

## How to Explore

1. `gitnexus_context({name: "evaluate"})` — see callers and callees
2. `gitnexus_query({query: "cluster_60"})` — find related execution flows
3. Read key files listed above for implementation details
