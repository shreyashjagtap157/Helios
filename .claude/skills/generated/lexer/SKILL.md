---
name: lexer
description: "Skill for the Lexer area of Helios. 24 symbols across 2 files."
---

# Lexer

24 symbols | 2 files | Cohesion: 90%

## When to Use

- Working with code in `omni-lang/`
- Understanding how new, tokenize work
- Modifying lexer-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `omni-lang/compiler/src/lexer/mod.rs` | new, next_nonblank_indent, tokenize, test_basic_tokens, test_struct_definition (+8) |
| `omni-lang/compiler/src/codegen/comprehensive_tests.rs` | test_lexer_keywords, test_lexer_operators, test_lexer_literals, test_lexer_indentation, test_lexer_dedentation (+6) |

## Entry Points

Start here when exploring this area:

- **`new`** (Function) — `omni-lang/compiler/src/lexer/mod.rs:293`
- **`tokenize`** (Function) — `omni-lang/compiler/src/lexer/mod.rs:349`

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `new` | Function | `omni-lang/compiler/src/lexer/mod.rs` | 293 |
| `tokenize` | Function | `omni-lang/compiler/src/lexer/mod.rs` | 349 |
| `next_nonblank_indent` | Function | `omni-lang/compiler/src/lexer/mod.rs` | 329 |
| `test_basic_tokens` | Function | `omni-lang/compiler/src/lexer/mod.rs` | 490 |
| `test_struct_definition` | Function | `omni-lang/compiler/src/lexer/mod.rs` | 500 |
| `test_pass_keyword` | Function | `omni-lang/compiler/src/lexer/mod.rs` | 509 |
| `test_line_comments` | Function | `omni-lang/compiler/src/lexer/mod.rs` | 518 |
| `test_hash_is_not_comment` | Function | `omni-lang/compiler/src/lexer/mod.rs` | 528 |
| `test_attribute_tokens` | Function | `omni-lang/compiler/src/lexer/mod.rs` | 537 |
| `test_all_keywords` | Function | `omni-lang/compiler/src/lexer/mod.rs` | 548 |
| `test_operators` | Function | `omni-lang/compiler/src/lexer/mod.rs` | 575 |
| `test_string_literal` | Function | `omni-lang/compiler/src/lexer/mod.rs` | 588 |
| `test_numeric_literals` | Function | `omni-lang/compiler/src/lexer/mod.rs` | 595 |
| `test_lexer_keywords` | Function | `omni-lang/compiler/src/codegen/comprehensive_tests.rs` | 106 |
| `test_lexer_operators` | Function | `omni-lang/compiler/src/codegen/comprehensive_tests.rs` | 127 |
| `test_lexer_literals` | Function | `omni-lang/compiler/src/codegen/comprehensive_tests.rs` | 141 |
| `test_lexer_indentation` | Function | `omni-lang/compiler/src/codegen/comprehensive_tests.rs` | 156 |
| `test_lexer_dedentation` | Function | `omni-lang/compiler/src/codegen/comprehensive_tests.rs` | 163 |
| `test_lexer_type_keywords` | Function | `omni-lang/compiler/src/codegen/comprehensive_tests.rs` | 170 |
| `test_lexer_delimiters` | Function | `omni-lang/compiler/src/codegen/comprehensive_tests.rs` | 183 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Run → Next` | cross_community | 5 |
| `Run → Next_nonblank_indent` | cross_community | 5 |
| `Compile → Next` | cross_community | 3 |
| `Compile → Next_nonblank_indent` | cross_community | 3 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Runtime | 1 calls |
| Brain | 1 calls |

## How to Explore

1. `gitnexus_context({name: "new"})` — see callers and callees
2. `gitnexus_query({query: "lexer"})` — find related execution flows
3. Read key files listed above for implementation details
