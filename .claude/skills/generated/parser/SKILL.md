---
name: parser
description: "Skill for the Parser area of Helios. 66 symbols across 4 files."
---

# Parser

66 symbols | 4 files | Cohesion: 89%

## When to Use

- Working with code in `omni-lang/`
- Understanding how new, parse_expression, precedence work
- Modifying parser-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `omni-lang/compiler/src/parser/mod.rs` | new, suggest_hint, peek, peek_kind, advance (+52) |
| `omni-lang/compiler/src/monitor.rs` | now_ms, enabled, inc_tokens, inc_items, record_parser_snapshot (+2) |
| `omni-lang/compiler/src/parser/ast.rs` | precedence |
| `omni-lang/compiler/src/semantic/type_inference.rs` | finalize_types |

## Entry Points

Start here when exploring this area:

- **`new`** (Function) — `omni-lang/compiler/src/parser/mod.rs:102`
- **`parse_expression`** (Function) — `omni-lang/compiler/src/parser/mod.rs:1276`
- **`precedence`** (Function) — `omni-lang/compiler/src/parser/ast.rs:397`
- **`enabled`** (Function) — `omni-lang/compiler/src/monitor.rs:30`
- **`inc_tokens`** (Function) — `omni-lang/compiler/src/monitor.rs:34`

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `new` | Function | `omni-lang/compiler/src/parser/mod.rs` | 102 |
| `parse_expression` | Function | `omni-lang/compiler/src/parser/mod.rs` | 1276 |
| `precedence` | Function | `omni-lang/compiler/src/parser/ast.rs` | 397 |
| `enabled` | Function | `omni-lang/compiler/src/monitor.rs` | 30 |
| `inc_tokens` | Function | `omni-lang/compiler/src/monitor.rs` | 34 |
| `inc_items` | Function | `omni-lang/compiler/src/monitor.rs` | 41 |
| `record_parser_snapshot` | Function | `omni-lang/compiler/src/monitor.rs` | 49 |
| `record_parser_error` | Function | `omni-lang/compiler/src/monitor.rs` | 62 |
| `update_heartbeat` | Function | `omni-lang/compiler/src/monitor.rs` | 72 |
| `parse_module` | Function | `omni-lang/compiler/src/parser/mod.rs` | 325 |
| `parse_with_recovery` | Function | `omni-lang/compiler/src/parser/mod.rs` | 1882 |
| `suggest_hint` | Function | `omni-lang/compiler/src/parser/mod.rs` | 204 |
| `peek` | Function | `omni-lang/compiler/src/parser/mod.rs` | 231 |
| `peek_kind` | Function | `omni-lang/compiler/src/parser/mod.rs` | 235 |
| `advance` | Function | `omni-lang/compiler/src/parser/mod.rs` | 239 |
| `expect` | Function | `omni-lang/compiler/src/parser/mod.rs` | 250 |
| `skip_newlines` | Function | `omni-lang/compiler/src/parser/mod.rs` | 273 |
| `parse_attribute` | Function | `omni-lang/compiler/src/parser/mod.rs` | 280 |
| `parse_item` | Function | `omni-lang/compiler/src/parser/mod.rs` | 412 |
| `parse_module_decl` | Function | `omni-lang/compiler/src/parser/mod.rs` | 493 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Run → Peek` | cross_community | 6 |
| `Eval_source → Enabled` | cross_community | 6 |
| `Eval_source → Now_ms` | cross_community | 6 |
| `Compile → Now_ms` | cross_community | 5 |
| `Main → Now_ms` | cross_community | 4 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Brain | 1 calls |

## How to Explore

1. `gitnexus_context({name: "new"})` — see callers and callees
2. `gitnexus_query({query: "parser"})` — find related execution flows
3. Read key files listed above for implementation details
