---
name: codegen
description: "Skill for the Codegen area of Helios. 672 symbols across 34 files."
---

# Codegen

672 symbols | 34 files | Cohesion: 83%

## When to Use

- Working with code in `omni-lang/`
- Understanding how compile, optimize, x86_64_linux work
- Modifying codegen-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `omni-lang/compiler/src/codegen/linker.rs` | host, for_host, set_entry_point, set_pie, add_text (+61) |
| `omni-lang/compiler/src/codegen/gpu_dispatch.rs` | copy_to_device, copy_device_to_device, synchronize_stream, new, try_new (+52) |
| `omni-lang/compiler/src/codegen/native_extended.rs` | emit_word, label, r_type, i_type, s_type (+50) |
| `omni-lang/compiler/src/codegen/jit.rs` | record_branch, record_loop_iteration, branch_probability, request_osr, record_loop_back_edge (+39) |
| `omni-lang/compiler/src/codegen/optimizer.rs` | optimize, tail_call_optimization_pass, loop_unrolling_pass, rename_for_unroll, new (+36) |
| `omni-lang/compiler/src/codegen/cognitive.rs` | new, register_heuristic, enqueue, next_task, complete (+35) |
| `omni-lang/compiler/src/codegen/comprehensive_tests.rs` | make_function, make_module, simple_add_func, multi_op_func, test_ir_function_display (+33) |
| `omni-lang/compiler/src/codegen/gpu_binary.rs` | new, alloc_id, emit_spirv_binary, generate_spirv_assembly, assemble_spirv (+31) |
| `omni-lang/compiler/src/codegen/jit_complete.rs` | new, add_method, lookup, get_vtable, dispatch (+30) |
| `omni-lang/compiler/src/codegen/native_codegen.rs` | x86_64_linux, aarch64_linux, wasm32_wasi, new, add_vreg (+29) |

## Entry Points

Start here when exploring this area:

- **`compile`** (Function) — `omni-lang/compiler/src/codegen/optimizing_jit.rs:30`
- **`optimize`** (Function) — `omni-lang/compiler/src/codegen/optimizer.rs:126`
- **`x86_64_linux`** (Function) — `omni-lang/compiler/src/codegen/native_codegen.rs:115`
- **`aarch64_linux`** (Function) — `omni-lang/compiler/src/codegen/native_codegen.rs:135`
- **`wasm32_wasi`** (Function) — `omni-lang/compiler/src/codegen/native_codegen.rs:145`

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `compile` | Function | `omni-lang/compiler/src/codegen/optimizing_jit.rs` | 30 |
| `optimize` | Function | `omni-lang/compiler/src/codegen/optimizer.rs` | 126 |
| `x86_64_linux` | Function | `omni-lang/compiler/src/codegen/native_codegen.rs` | 115 |
| `aarch64_linux` | Function | `omni-lang/compiler/src/codegen/native_codegen.rs` | 135 |
| `wasm32_wasi` | Function | `omni-lang/compiler/src/codegen/native_codegen.rs` | 145 |
| `new` | Function | `omni-lang/compiler/src/codegen/native_codegen.rs` | 339 |
| `add_vreg` | Function | `omni-lang/compiler/src/codegen/native_codegen.rs` | 373 |
| `allocate` | Function | `omni-lang/compiler/src/codegen/native_codegen.rs` | 387 |
| `add_type` | Function | `omni-lang/compiler/src/codegen/native_codegen.rs` | 1389 |
| `build` | Function | `omni-lang/compiler/src/codegen/native_codegen.rs` | 1552 |
| `add_text` | Function | `omni-lang/compiler/src/codegen/native_codegen.rs` | 1664 |
| `add_function_symbol` | Function | `omni-lang/compiler/src/codegen/native_codegen.rs` | 1697 |
| `set_opt_level` | Function | `omni-lang/compiler/src/codegen/native_codegen.rs` | 1834 |
| `compile_module` | Function | `omni-lang/compiler/src/codegen/native_codegen.rs` | 1839 |
| `label` | Function | `omni-lang/compiler/src/codegen/native_extended.rs` | 544 |
| `add` | Function | `omni-lang/compiler/src/codegen/native_extended.rs` | 586 |
| `sub` | Function | `omni-lang/compiler/src/codegen/native_extended.rs` | 591 |
| `addi` | Function | `omni-lang/compiler/src/codegen/native_extended.rs` | 596 |
| `and` | Function | `omni-lang/compiler/src/codegen/native_extended.rs` | 601 |
| `or` | Function | `omni-lang/compiler/src/codegen/native_extended.rs` | 606 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Dispatch_multi_gpu → Try_new` | cross_community | 8 |
| `Dispatch_multi_gpu → Software_fallback` | cross_community | 8 |
| `Run_async → Get_mut` | cross_community | 7 |
| `Run → Get_mut` | cross_community | 7 |
| `Dispatch_multi_gpu → Probe_opencl` | cross_community | 7 |
| `Dispatch_multi_gpu → Probe_vulkan` | cross_community | 7 |
| `Compile → Alloc_vreg` | cross_community | 6 |
| `Dispatch_multi_gpu → Active_device` | cross_community | 6 |
| `Test_native_all_output_formats → Alloc_vreg` | cross_community | 6 |
| `Compile_kernel → Probe_cuda` | cross_community | 6 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Brain | 6 calls |

## How to Explore

1. `gitnexus_context({name: "compile"})` — see callers and callees
2. `gitnexus_query({query: "codegen"})` — find related execution flows
3. Read key files listed above for implementation details
