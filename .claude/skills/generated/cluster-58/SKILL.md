---
name: cluster-58
description: "Skill for the Cluster_58 area of Helios. 13 symbols across 1 files."
---

# Cluster_58

13 symbols | 1 files | Cohesion: 71%

## When to Use

- Working with code in `omni-lang/`
- Understanding how send_command, continue_execution, step_instruction work
- Modifying cluster_58-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `omni-lang/tools/omni-dap/src/main_v0.rs` | send_command, continue_execution, step_instruction, step_over, step_out (+8) |

## Entry Points

Start here when exploring this area:

- **`send_command`** (Function) — `omni-lang/tools/omni-dap/src/main_v0.rs:134`
- **`continue_execution`** (Function) — `omni-lang/tools/omni-dap/src/main_v0.rs:158`
- **`step_instruction`** (Function) — `omni-lang/tools/omni-dap/src/main_v0.rs:163`
- **`step_over`** (Function) — `omni-lang/tools/omni-dap/src/main_v0.rs:168`
- **`step_out`** (Function) — `omni-lang/tools/omni-dap/src/main_v0.rs:173`

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `send_command` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 134 |
| `continue_execution` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 158 |
| `step_instruction` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 163 |
| `step_over` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 168 |
| `step_out` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 173 |
| `handle_request` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 324 |
| `handle_config_done` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 846 |
| `handle_continue` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 873 |
| `handle_pause` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 892 |
| `handle_next` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 910 |
| `handle_step_in` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 928 |
| `handle_step_out` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 945 |
| `handle_restart` | Function | `omni-lang/tools/omni-dap/src/main_v0.rs` | 1054 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Main → Bucket` | cross_community | 7 |
| `Main → New` | cross_community | 6 |
| `Handle_set_breakpoints → Bucket` | cross_community | 5 |
| `Handle_stack_trace → Bucket` | cross_community | 5 |
| `Handle_read_memory → Bucket` | cross_community | 5 |
| `Handle_scopes → Bucket` | cross_community | 5 |
| `Handle_evaluate → Bucket` | cross_community | 5 |
| `Main → SourceMapEntry` | cross_community | 5 |
| `Main → Positional` | cross_community | 5 |
| `Handle_set_breakpoints → New` | cross_community | 4 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Runtime | 4 calls |
| Cluster_56 | 1 calls |
| Brain | 1 calls |

## How to Explore

1. `gitnexus_context({name: "send_command"})` — see callers and callees
2. `gitnexus_query({query: "cluster_58"})` — find related execution flows
3. Read key files listed above for implementation details
