---
name: runtime
description: "Skill for the Runtime area of Helios. 244 symbols across 24 files."
---

# Runtime

244 symbols | 24 files | Cohesion: 84%

## When to Use

- Working with code in `omni-lang/`
- Understanding how new, execute, compile_module work
- Modifying runtime-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `omni-lang/compiler/src/runtime/interpreter.rs` | next_ready, run_async, execute_task, execute_bytecode, read_u16 (+45) |
| `omni-lang/compiler/src/runtime/vm.rs` | new, peek, is_truthy, value_to_vm, execute (+34) |
| `omni-lang/compiler/src/runtime/bytecode.rs` | read_u8, read_u16, read_u32, read_u64, read_i64 (+34) |
| `omni-lang/compiler/src/runtime/bytecode_compiler.rs` | module_with_main, compile, main_instrs, test_empty_main, test_let_binding (+31) |
| `omni-lang/compiler/src/runtime/profiler.rs` | start_profiling, start, new, schedule, analyze_dependencies (+15) |
| `omni-lang/compiler/src/runtime/hot_swap.rs` | new, check_for_updates, get_pending_changes, apply_pending, watch_file (+6) |
| `omni-lang/tools/omni-dap/src/main_v0.rs` | connect, handle_attach, main, send_message, load_source_maps (+5) |
| `omni-lang/compiler/src/runtime/distributed_logic.rs` | flush, new, discover, detect_gpu_count, read_sysfs_topology |
| `omni-lang/compiler/src/runtime/native.rs` | new, call, alloc_handle, get_string_arg, get_handle_arg |
| `omni-lang/compiler/src/monitor.rs` | enable, disable, snapshot, rich_snapshot |

## Entry Points

Start here when exploring this area:

- **`new`** (Function) — `omni-lang/compiler/src/runtime/vm.rs:177`
- **`execute`** (Function) — `omni-lang/compiler/src/runtime/vm.rs:352`
- **`compile_module`** (Function) — `omni-lang/compiler/src/runtime/bytecode_compiler.rs:82`
- **`compile_function`** (Function) — `omni-lang/compiler/src/runtime/bytecode_compiler.rs:123`
- **`compile_statement`** (Function) — `omni-lang/compiler/src/runtime/bytecode_compiler.rs:237`

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `new` | Function | `omni-lang/compiler/src/runtime/vm.rs` | 177 |
| `execute` | Function | `omni-lang/compiler/src/runtime/vm.rs` | 352 |
| `compile_module` | Function | `omni-lang/compiler/src/runtime/bytecode_compiler.rs` | 82 |
| `compile_function` | Function | `omni-lang/compiler/src/runtime/bytecode_compiler.rs` | 123 |
| `compile_statement` | Function | `omni-lang/compiler/src/runtime/bytecode_compiler.rs` | 237 |
| `compile_expression` | Function | `omni-lang/compiler/src/runtime/bytecode_compiler.rs` | 560 |
| `next_ready` | Function | `omni-lang/compiler/src/runtime/interpreter.rs` | 156 |
| `run_async` | Function | `omni-lang/compiler/src/runtime/interpreter.rs` | 531 |
| `deserialize` | Function | `omni-lang/compiler/src/runtime/bytecode.rs` | 723 |
| `new` | Function | `omni-lang/compiler/src/runtime/bytecode.rs` | 224 |
| `serialize` | Function | `omni-lang/compiler/src/runtime/bytecode.rs` | 680 |
| `alloc` | Function | `omni-lang/compiler/src/runtime/vm.rs` | 194 |
| `gc_collect` | Function | `omni-lang/compiler/src/runtime/vm.rs` | 244 |
| `is_truthy` | Function | `omni-lang/compiler/src/runtime/interpreter.rs` | 1765 |
| `with_parent` | Function | `omni-lang/compiler/src/runtime/interpreter.rs` | 1834 |
| `eval_block` | Function | `omni-lang/compiler/src/runtime/interpreter.rs` | 2076 |
| `eval_statement` | Function | `omni-lang/compiler/src/runtime/interpreter.rs` | 2089 |
| `eval_expr` | Function | `omni-lang/compiler/src/runtime/interpreter.rs` | 2310 |
| `enable` | Function | `omni-lang/compiler/src/monitor.rs` | 21 |
| `disable` | Function | `omni-lang/compiler/src/monitor.rs` | 26 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Run_async → Get_mut` | cross_community | 7 |
| `Compile → As_str` | cross_community | 7 |
| `Run → Get_mut` | cross_community | 7 |
| `Run → As_str` | cross_community | 7 |
| `Run → Apply_binary_op` | cross_community | 7 |
| `Main → Bucket` | cross_community | 7 |
| `Run → As_int` | cross_community | 6 |
| `Run → Peek` | cross_community | 6 |
| `Run_async → New` | cross_community | 6 |
| `Run_async → Collect_roots` | cross_community | 6 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Parser | 6 calls |
| Brain | 6 calls |
| Codegen | 6 calls |
| Semantic | 4 calls |
| Lexer | 2 calls |
| Cluster_36 | 1 calls |
| Cluster_60 | 1 calls |
| Cluster_56 | 1 calls |

## How to Explore

1. `gitnexus_context({name: "new"})` — see callers and callees
2. `gitnexus_query({query: "runtime"})` — find related execution flows
3. Read key files listed above for implementation details
