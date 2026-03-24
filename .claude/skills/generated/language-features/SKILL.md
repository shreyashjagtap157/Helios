---
name: language-features
description: "Skill for the Language_features area of Helios. 42 symbols across 6 files."
---

# Language_features

42 symbols | 6 files | Cohesion: 95%

## When to Use

- Working with code in `omni-lang/`
- Understanding how select_best_strategy, multi_step_reasoning, new work
- Modifying language_features-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `omni-lang/compiler/src/language_features/default_params.rs` | new, with_default, add_param, resolve_call, test_function_signature_defaults (+12) |
| `omni-lang/tests/comprehensive_test_suite.rs` | test_default_parameters_resolution, test_named_arguments_ordering, test_full_workflow_language_and_ml, test_variadic_type_validation, test_lazy_static_thread_safety (+1) |
| `omni-lang/compiler/src/language_features/variadics.rs` | new, add_param, set_variadic, test_variadic_function_creation, test_variadic_argument_validation (+1) |
| `helios-framework/brain/adaptive_reasoning.rs` | success_rate, select_best_strategy, multi_step_reasoning, test_strategy_selection, test_multi_step_reasoning |
| `omni-lang/compiler/src/language_features/lazy_static.rs` | new, get_or_init, test_lazy_static_initialization, test_lazy_static_thread_safety |
| `omni-lang/compiler/src/language_features/operator_overloading.rs` | method_name, register, new, test_operator_registry |

## Entry Points

Start here when exploring this area:

- **`select_best_strategy`** (Function) — `helios-framework/brain/adaptive_reasoning.rs:91`
- **`multi_step_reasoning`** (Function) — `helios-framework/brain/adaptive_reasoning.rs:127`
- **`new`** (Function) — `omni-lang/compiler/src/language_features/default_params.rs:19`
- **`with_default`** (Function) — `omni-lang/compiler/src/language_features/default_params.rs:27`
- **`add_param`** (Function) — `omni-lang/compiler/src/language_features/default_params.rs:82`

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `select_best_strategy` | Function | `helios-framework/brain/adaptive_reasoning.rs` | 91 |
| `multi_step_reasoning` | Function | `helios-framework/brain/adaptive_reasoning.rs` | 127 |
| `new` | Function | `omni-lang/compiler/src/language_features/default_params.rs` | 19 |
| `with_default` | Function | `omni-lang/compiler/src/language_features/default_params.rs` | 27 |
| `add_param` | Function | `omni-lang/compiler/src/language_features/default_params.rs` | 82 |
| `resolve_call` | Function | `omni-lang/compiler/src/language_features/default_params.rs` | 125 |
| `new` | Function | `omni-lang/compiler/src/language_features/variadics.rs` | 37 |
| `add_param` | Function | `omni-lang/compiler/src/language_features/variadics.rs` | 48 |
| `set_variadic` | Function | `omni-lang/compiler/src/language_features/variadics.rs` | 54 |
| `new` | Function | `omni-lang/compiler/src/language_features/lazy_static.rs` | 18 |
| `get_or_init` | Function | `omni-lang/compiler/src/language_features/lazy_static.rs` | 27 |
| `has_default` | Function | `omni-lang/compiler/src/language_features/default_params.rs` | 35 |
| `required_params` | Function | `omni-lang/compiler/src/language_features/default_params.rs` | 88 |
| `total_params` | Function | `omni-lang/compiler/src/language_features/default_params.rs` | 93 |
| `validate` | Function | `omni-lang/compiler/src/language_features/default_params.rs` | 98 |
| `is_valid_call_count` | Function | `omni-lang/compiler/src/language_features/default_params.rs` | 175 |
| `named` | Function | `omni-lang/compiler/src/language_features/default_params.rs` | 52 |
| `named_arg` | Function | `omni-lang/compiler/src/language_features/default_params.rs` | 201 |
| `build` | Function | `omni-lang/compiler/src/language_features/default_params.rs` | 206 |
| `method_name` | Function | `omni-lang/compiler/src/language_features/operator_overloading.rs` | 88 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Brain | 1 calls |
| Codegen | 1 calls |

## How to Explore

1. `gitnexus_context({name: "select_best_strategy"})` — see callers and callees
2. `gitnexus_query({query: "language_features"})` — find related execution flows
3. Read key files listed above for implementation details
