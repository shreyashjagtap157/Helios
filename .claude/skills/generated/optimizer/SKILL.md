---
name: optimizer
description: "Skill for the Optimizer area of Helios. 115 symbols across 5 files."
---

# Optimizer

115 symbols | 5 files | Cohesion: 89%

## When to Use

- Working with code in `omni-lang/`
- Understanding how fold_expr, simplify_expr, inline_functions work
- Modifying optimizer-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `omni-lang/compiler/src/optimizer/constant_folding.rs` | fold_expr, bool_lit, str_lit, bin, unary (+32) |
| `omni-lang/compiler/src/optimizer/simplify.rs` | simplify_expr, simplify_unary, ident, bool_lit, str_lit (+24) |
| `omni-lang/compiler/src/optimizer/inlining.rs` | find_inline_candidates, build_call_graph, collect_callees_in_block, collect_callees_in_stmt, collect_callees_in_expr (+21) |
| `omni-lang/compiler/src/optimizer/dead_code.rs` | eliminate_dead_code, test_remove_after_return, test_remove_unused_function, test_keep_called_function, test_remove_unused_import (+15) |
| `omni-lang/compiler/src/optimizer/mod.rs` | optimize, test_o0_no_change, test_optimize_runs_without_panic |

## Entry Points

Start here when exploring this area:

- **`fold_expr`** (Function) — `omni-lang/compiler/src/optimizer/constant_folding.rs:228`
- **`simplify_expr`** (Function) — `omni-lang/compiler/src/optimizer/simplify.rs:89`
- **`inline_functions`** (Function) — `omni-lang/compiler/src/optimizer/inlining.rs:16`
- **`optimize`** (Function) — `omni-lang/compiler/src/optimizer/mod.rs:30`
- **`fold_constants`** (Function) — `omni-lang/compiler/src/optimizer/constant_folding.rs:10`

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `fold_expr` | Function | `omni-lang/compiler/src/optimizer/constant_folding.rs` | 228 |
| `simplify_expr` | Function | `omni-lang/compiler/src/optimizer/simplify.rs` | 89 |
| `inline_functions` | Function | `omni-lang/compiler/src/optimizer/inlining.rs` | 16 |
| `optimize` | Function | `omni-lang/compiler/src/optimizer/mod.rs` | 30 |
| `fold_constants` | Function | `omni-lang/compiler/src/optimizer/constant_folding.rs` | 10 |
| `eliminate_dead_code` | Function | `omni-lang/compiler/src/optimizer/dead_code.rs` | 9 |
| `simplify_expressions` | Function | `omni-lang/compiler/src/optimizer/simplify.rs` | 8 |
| `bool_lit` | Function | `omni-lang/compiler/src/optimizer/constant_folding.rs` | 351 |
| `str_lit` | Function | `omni-lang/compiler/src/optimizer/constant_folding.rs` | 356 |
| `bin` | Function | `omni-lang/compiler/src/optimizer/constant_folding.rs` | 361 |
| `unary` | Function | `omni-lang/compiler/src/optimizer/constant_folding.rs` | 366 |
| `test_fold_int_add` | Function | `omni-lang/compiler/src/optimizer/constant_folding.rs` | 375 |
| `test_fold_int_sub` | Function | `omni-lang/compiler/src/optimizer/constant_folding.rs` | 381 |
| `test_fold_int_mul` | Function | `omni-lang/compiler/src/optimizer/constant_folding.rs` | 387 |
| `test_fold_int_div` | Function | `omni-lang/compiler/src/optimizer/constant_folding.rs` | 393 |
| `test_fold_int_div_by_zero_no_fold` | Function | `omni-lang/compiler/src/optimizer/constant_folding.rs` | 399 |
| `test_fold_int_mod` | Function | `omni-lang/compiler/src/optimizer/constant_folding.rs` | 405 |
| `test_fold_float_add` | Function | `omni-lang/compiler/src/optimizer/constant_folding.rs` | 411 |
| `test_fold_string_concat` | Function | `omni-lang/compiler/src/optimizer/constant_folding.rs` | 420 |
| `test_fold_bool_and` | Function | `omni-lang/compiler/src/optimizer/constant_folding.rs` | 429 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Test_and_true_identity → Is_int_zero` | cross_community | 4 |
| `Test_and_true_identity → Is_empty_string` | cross_community | 4 |
| `Test_and_true_identity → Exprs_equal` | cross_community | 4 |
| `Test_and_true_identity → Is_int_one` | cross_community | 4 |
| `Test_or_false_identity → Is_int_zero` | cross_community | 4 |
| `Test_or_false_identity → Is_empty_string` | cross_community | 4 |
| `Test_or_false_identity → Exprs_equal` | cross_community | 4 |
| `Test_or_false_identity → Is_int_one` | cross_community | 4 |
| `Test_and_false_short_circuit → Is_int_zero` | cross_community | 4 |
| `Test_and_false_short_circuit → Is_empty_string` | cross_community | 4 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Brain | 1 calls |

## How to Explore

1. `gitnexus_context({name: "fold_expr"})` — see callers and callees
2. `gitnexus_query({query: "optimizer"})` — find related execution flows
3. Read key files listed above for implementation details
