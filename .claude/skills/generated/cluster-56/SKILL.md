---
name: cluster-56
description: "Skill for the Cluster_56 area of Helios. 12 symbols across 1 files."
---

# Cluster_56

12 symbols | 1 files | Cohesion: 65%

## When to Use

- Working with code in `omni-lang/`
- Understanding how new, get_stack_frames, get_locals work
- Modifying cluster_56-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `omni-lang/tools/omni-dap/src/main_v0.rs` | new, get_stack_frames, get_locals, read_memory, offset_to_source (+7) |

## Entry Points

Start here when exploring this area:

- **`new`** (Function) — `omni-lang/tools/omni-dap/src/main_v0.rs:119`
- **`get_stack_frames`** (Function) — `omni-lang/tools/omni-dap/src/main_v0.rs:178`
- **`get_locals`** (Function) — `omni-lang/tools/omni-dap/src/main_v0.rs:194`
- **`read_memory`** (Function) — `omni-lang/tools/omni-dap/src/main_v0.rs:218`
- **`render_heatmap`** (Function) — `omni-lang/tools/omni-dap/src/main_v0.rs:1125`

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `new` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 119 |
| `get_stack_frames` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 178 |
| `get_locals` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 194 |
| `read_memory` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 218 |
| `render_heatmap` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 1125 |
| `render_sparkline` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 1184 |
| `encode` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 1273 |
| `offset_to_source` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 315 |
| `handle_set_function_breakpoints` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 582 |
| `handle_stack_trace` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 636 |
| `handle_scopes` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 691 |
| `handle_read_memory` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 1012 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Main → New` | cross_community | 6 |
| `Handle_stack_trace → Bucket` | cross_community | 5 |
| `Handle_read_memory → Bucket` | cross_community | 5 |
| `Handle_scopes → Bucket` | cross_community | 5 |
| `Handle_set_breakpoints → New` | cross_community | 4 |
| `Handle_stack_trace → New` | cross_community | 4 |
| `Handle_read_memory → New` | cross_community | 4 |
| `Handle_scopes → New` | cross_community | 4 |
| `Handle_evaluate → New` | cross_community | 4 |
| `Handle_set_function_breakpoints → DapMessage` | cross_community | 3 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Cluster_60 | 4 calls |
| Cluster_58 | 3 calls |
| Brain | 2 calls |

## How to Explore

1. `gitnexus_context({name: "new"})` — see callers and callees
2. `gitnexus_query({query: "cluster_56"})` — find related execution flows
3. Read key files listed above for implementation details
