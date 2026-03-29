---
name: ir
description: "Skill for the Ir area of Helios. 12 symbols across 1 files."
---

# Ir

12 symbols | 1 files | Cohesion: 76%

## When to Use

- Working with code in `omni-lang/`
- Understanding how new, generate work
- Modifying ir-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `omni-lang/compiler/src/ir/mod.rs` | new, generate, calculate_struct_size, calculate_enum_size, type_size (+7) |

## Entry Points

Start here when exploring this area:

- **`new`** (Function) — `omni-lang/compiler/src/ir/mod.rs:638`
- **`generate`** (Function) — `omni-lang/compiler/src/ir/mod.rs:680`

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `new` | Function | `omni-lang/compiler/src/ir/mod.rs` | 638 |
| `generate` | Function | `omni-lang/compiler/src/ir/mod.rs` | 680 |
| `calculate_struct_size` | Function | `omni-lang/compiler/src/ir/mod.rs` | 797 |
| `calculate_enum_size` | Function | `omni-lang/compiler/src/ir/mod.rs` | 801 |
| `type_size` | Function | `omni-lang/compiler/src/ir/mod.rs` | 811 |
| `gen_function` | Function | `omni-lang/compiler/src/ir/mod.rs` | 834 |
| `fresh_temp` | Function | `omni-lang/compiler/src/ir/mod.rs` | 658 |
| `fresh_block` | Function | `omni-lang/compiler/src/ir/mod.rs` | 664 |
| `intern_string` | Function | `omni-lang/compiler/src/ir/mod.rs` | 670 |
| `gen_statement` | Function | `omni-lang/compiler/src/ir/mod.rs` | 873 |
| `gen_expr` | Function | `omni-lang/compiler/src/ir/mod.rs` | 1242 |
| `convert_type` | Function | `omni-lang/compiler/src/ir/mod.rs` | 1510 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Generate → Type_size` | intra_community | 3 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Brain | 1 calls |

## How to Explore

1. `gitnexus_context({name: "new"})` — see callers and callees
2. `gitnexus_query({query: "ir"})` — find related execution flows
3. Read key files listed above for implementation details
