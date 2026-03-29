---
name: semantic
description: "Skill for the Semantic area of Helios. 326 symbols across 19 files."
---

# Semantic

326 symbols | 19 files | Cohesion: 82%

## When to Use

- Working with code in `omni-lang/`
- Understanding how with_hint, lookup, push_scope work
- Modifying semantic-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `omni-lang/compiler/src/semantic/borrow_check.rs` | check_module, make_function, make_module, test_use_after_move, test_valid_sequential_usage (+54) |
| `omni-lang/compiler/src/semantic/type_inference.rs` | with_hint, lookup, push_scope, pop_scope, define (+45) |
| `omni-lang/compiler/src/semantic/mod.rs` | new, types_equal, is_lvalue, init_builtin_traits, init_primitive_impls (+43) |
| `omni-lang/compiler/src/semantic/tests.rs` | test_literal_analysis_int, test_literal_analysis_float, test_literal_analysis_bool, test_literal_analysis_string, test_binary_addition (+20) |
| `omni-lang/compiler/src/semantic/properties.rs` | get_property, to_getter_method, to_setter_method, expand_getter, expand_setter (+13) |
| `omni-lang/compiler/src/semantic/inference.rs` | new, fresh_var, bind, add_constraint, unify (+10) |
| `omni-lang/compiler/src/semantic/constraints.rs` | new, compose, add_constraint, solve, occurs_check (+10) |
| `omni-lang/compiler/src/semantic/traits.rs` | new, init_builtin_traits, register_trait, test_builtin_trait_copy, test_builtin_trait_clone (+10) |
| `omni-lang/compiler/src/semantic/performance.rs` | new, start_operation, end_operation, report, record_phase (+10) |
| `omni-lang/compiler/src/semantic/monomorphization.rs` | add, apply_to_type, test_substitute_generic_named_type, test_substitute_array_element_type, test_substitute_function_parameter_types (+8) |

## Entry Points

Start here when exploring this area:

- **`with_hint`** (Function) — `omni-lang/compiler/src/semantic/type_inference.rs:146`
- **`lookup`** (Function) — `omni-lang/compiler/src/semantic/type_inference.rs:216`
- **`push_scope`** (Function) — `omni-lang/compiler/src/semantic/type_inference.rs:275`
- **`pop_scope`** (Function) — `omni-lang/compiler/src/semantic/type_inference.rs:280`
- **`define`** (Function) — `omni-lang/compiler/src/semantic/type_inference.rs:287`

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `with_hint` | Function | `omni-lang/compiler/src/semantic/type_inference.rs` | 146 |
| `lookup` | Function | `omni-lang/compiler/src/semantic/type_inference.rs` | 216 |
| `push_scope` | Function | `omni-lang/compiler/src/semantic/type_inference.rs` | 275 |
| `pop_scope` | Function | `omni-lang/compiler/src/semantic/type_inference.rs` | 280 |
| `define` | Function | `omni-lang/compiler/src/semantic/type_inference.rs` | 287 |
| `infer_module` | Function | `omni-lang/compiler/src/semantic/type_inference.rs` | 450 |
| `infer_function` | Function | `omni-lang/compiler/src/semantic/type_inference.rs` | 555 |
| `infer_statement` | Function | `omni-lang/compiler/src/semantic/type_inference.rs` | 582 |
| `infer_expr` | Function | `omni-lang/compiler/src/semantic/type_inference.rs` | 805 |
| `generate_constraints` | Function | `omni-lang/compiler/src/semantic/type_inference.rs` | 1688 |
| `check_module` | Function | `omni-lang/compiler/src/semantic/borrow_check.rs` | 566 |
| `new` | Function | `omni-lang/compiler/src/semantic/type_inference.rs` | 139 |
| `fresh_var` | Function | `omni-lang/compiler/src/semantic/type_inference.rs` | 385 |
| `apply_substitution` | Function | `omni-lang/compiler/src/semantic/type_inference.rs` | 1678 |
| `new` | Function | `omni-lang/compiler/src/semantic/inference.rs` | 25 |
| `fresh_var` | Function | `omni-lang/compiler/src/semantic/inference.rs` | 33 |
| `bind` | Function | `omni-lang/compiler/src/semantic/inference.rs` | 74 |
| `add_constraint` | Function | `omni-lang/compiler/src/semantic/inference.rs` | 105 |
| `unify` | Function | `omni-lang/compiler/src/semantic/inference.rs` | 110 |
| `solve` | Function | `omni-lang/compiler/src/semantic/inference.rs` | 205 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Compile → As_str` | cross_community | 7 |
| `Eval_source → As_str` | cross_community | 6 |
| `Eval_source → Define` | cross_community | 6 |
| `Compile → Define` | cross_community | 6 |
| `Run → New` | cross_community | 5 |
| `Generate_constraints → As_str` | cross_community | 5 |
| `Check_statement → Loc` | cross_community | 5 |
| `Compile → Now_ms` | cross_community | 5 |
| `Infer_statement → With_hint` | intra_community | 5 |
| `Test_complete_semantic_analysis_workflow → New` | cross_community | 5 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Codegen | 9 calls |
| Parser | 3 calls |
| Brain | 1 calls |

## How to Explore

1. `gitnexus_context({name: "with_hint"})` — see callers and callees
2. `gitnexus_query({query: "semantic"})` — find related execution flows
3. Read key files listed above for implementation details
