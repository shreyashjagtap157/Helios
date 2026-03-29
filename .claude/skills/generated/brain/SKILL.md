---
name: brain
description: "Skill for the Brain area of Helios. 75 symbols across 9 files."
---

# Brain

75 symbols | 9 files | Cohesion: 80%

## When to Use

- Working with code in `omni-lang/`
- Understanding how apply_strategy, as_str, analyze work
- Modifying brain-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `helios-framework/brain/integration_tests.rs` | new, select_best_strategy, reason, get_performance, test_reasoning_engine_strategy_selection (+15) |
| `omni-lang/compiler/src/brain/knowledge_graph.rs` | new, add_node, add_edge, shortest_path, has_cycle (+14) |
| `helios-framework/brain/adaptive_reasoning.rs` | apply_strategy, deductive_reasoning, inductive_reasoning, abductive_reasoning, analogical_reasoning (+9) |
| `omni-lang/compiler/src/brain/adaptive_reasoning.rs` | new, deduce, record_outcome, test_deductive_modus_ponens, test_deductive_conditional (+3) |
| `omni-lang/compiler/src/brain/memory.rs` | new, add_short_term, consolidate, query, test_memory_consolidation (+2) |
| `omni-lang/compiler/src/codegen/gpu_advanced.rs` | analyze, depends_on_thread_id, causes_bank_conflict |
| `omni-lang/tests/comprehensive_test_suite.rs` | test_contextual_reasoning_with_domain_knowledge, test_multi_step_adaptive_reasoning |
| `omni-lang/compiler/src/parser/mod.rs` | as_str |
| `omni-lang/compiler/src/codegen/cognitive.rs` | heuristic_names |

## Entry Points

Start here when exploring this area:

- **`apply_strategy`** (Function) — `helios-framework/brain/adaptive_reasoning.rs:104`
- **`as_str`** (Function) — `omni-lang/compiler/src/parser/mod.rs:38`
- **`analyze`** (Function) — `omni-lang/compiler/src/codegen/gpu_advanced.rs:11`
- **`heuristic_names`** (Function) — `omni-lang/compiler/src/codegen/cognitive.rs:295`
- **`new`** (Function) — `omni-lang/compiler/src/brain/knowledge_graph.rs:42`

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `apply_strategy` | Function | `helios-framework/brain/adaptive_reasoning.rs` | 104 |
| `as_str` | Function | `omni-lang/compiler/src/parser/mod.rs` | 38 |
| `analyze` | Function | `omni-lang/compiler/src/codegen/gpu_advanced.rs` | 11 |
| `heuristic_names` | Function | `omni-lang/compiler/src/codegen/cognitive.rs` | 295 |
| `new` | Function | `omni-lang/compiler/src/brain/knowledge_graph.rs` | 42 |
| `add_node` | Function | `omni-lang/compiler/src/brain/knowledge_graph.rs` | 51 |
| `add_edge` | Function | `omni-lang/compiler/src/brain/knowledge_graph.rs` | 56 |
| `shortest_path` | Function | `omni-lang/compiler/src/brain/knowledge_graph.rs` | 67 |
| `has_cycle` | Function | `omni-lang/compiler/src/brain/knowledge_graph.rs` | 124 |
| `new` | Function | `omni-lang/compiler/src/brain/adaptive_reasoning.rs` | 20 |
| `deduce` | Function | `omni-lang/compiler/src/brain/adaptive_reasoning.rs` | 35 |
| `record_outcome` | Function | `omni-lang/compiler/src/brain/adaptive_reasoning.rs` | 128 |
| `new` | Function | `omni-lang/compiler/src/brain/memory.rs` | 27 |
| `add_short_term` | Function | `omni-lang/compiler/src/brain/memory.rs` | 36 |
| `consolidate` | Function | `omni-lang/compiler/src/brain/memory.rs` | 45 |
| `query` | Function | `omni-lang/compiler/src/brain/memory.rs` | 76 |
| `add_fact` | Function | `omni-lang/compiler/src/brain/knowledge_graph.rs` | 164 |
| `add_rule` | Function | `omni-lang/compiler/src/brain/knowledge_graph.rs` | 176 |
| `forward_chain` | Function | `omni-lang/compiler/src/brain/knowledge_graph.rs` | 191 |
| `set_context` | Function | `helios-framework/brain/adaptive_reasoning.rs` | 502 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Compile → As_str` | cross_community | 7 |
| `Run → As_str` | cross_community | 7 |
| `Eval_source → As_str` | cross_community | 6 |
| `Main → As_str` | cross_community | 5 |
| `Generate_constraints → As_str` | cross_community | 5 |
| `Main → As_str` | cross_community | 4 |
| `Resolve → As_str` | cross_community | 4 |
| `Lower_ast_to_mlir → As_str` | cross_community | 3 |
| `Test_full_pipeline_integration → Select_best_strategy` | cross_community | 3 |
| `Test_full_pipeline_integration → Get_mut` | cross_community | 3 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Codegen | 4 calls |
| Language_features | 2 calls |
| Runtime | 1 calls |

## How to Explore

1. `gitnexus_context({name: "apply_strategy"})` — see callers and callees
2. `gitnexus_query({query: "brain"})` — find related execution flows
3. Read key files listed above for implementation details
