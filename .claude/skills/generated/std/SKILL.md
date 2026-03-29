---
name: std
description: "Skill for the Std area of Helios. 28 symbols across 2 files."
---

# Std

28 symbols | 2 files | Cohesion: 80%

## When to Use

- Working with code in `omni-lang/`
- Understanding how new, next_u64, generate_bytes work
- Modifying std-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `omni-lang/std/crypto_audit.rs` | new, next_u64, generate_bytes, test_crypto_random_generation, test_crypto_random_bytes (+20) |
| `omni-lang/std/benchmarks_comprehensive.rs` | new, run, run_once |

## Entry Points

Start here when exploring this area:

- **`new`** (Function) — `omni-lang/std/crypto_audit.rs:71`
- **`next_u64`** (Function) — `omni-lang/std/crypto_audit.rs:320`
- **`generate_bytes`** (Function) — `omni-lang/std/crypto_audit.rs:336`
- **`from_bytes`** (Function) — `omni-lang/std/crypto_audit.rs:48`
- **`encrypt`** (Function) — `omni-lang/std/crypto_audit.rs:76`

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `new` | Function | `omni-lang/std/crypto_audit.rs` | 71 |
| `next_u64` | Function | `omni-lang/std/crypto_audit.rs` | 320 |
| `generate_bytes` | Function | `omni-lang/std/crypto_audit.rs` | 336 |
| `from_bytes` | Function | `omni-lang/std/crypto_audit.rs` | 48 |
| `encrypt` | Function | `omni-lang/std/crypto_audit.rs` | 76 |
| `decrypt` | Function | `omni-lang/std/crypto_audit.rs` | 101 |
| `finalize` | Function | `omni-lang/std/crypto_audit.rs` | 156 |
| `sign` | Function | `omni-lang/std/crypto_audit.rs` | 267 |
| `verify` | Function | `omni-lang/std/crypto_audit.rs` | 293 |
| `run` | Function | `omni-lang/std/benchmarks_comprehensive.rs` | 47 |
| `run_once` | Function | `omni-lang/std/benchmarks_comprehensive.rs` | 69 |
| `generate_random` | Function | `omni-lang/std/crypto_audit.rs` | 52 |
| `test_crypto_random_generation` | Function | `omni-lang/std/crypto_audit.rs` | 514 |
| `test_crypto_random_bytes` | Function | `omni-lang/std/crypto_audit.rs` | 528 |
| `test_crypto_random_deterministic_seed` | Function | `omni-lang/std/crypto_audit.rs` | 539 |
| `test_crypto_random_edge_cases` | Function | `omni-lang/std/crypto_audit.rs` | 550 |
| `test_aes_gcm_encrypt_decrypt` | Function | `omni-lang/std/crypto_audit.rs` | 367 |
| `test_aes_gcm_authentication` | Function | `omni-lang/std/crypto_audit.rs` | 384 |
| `test_aes_gcm_empty_plaintext` | Function | `omni-lang/std/crypto_audit.rs` | 405 |
| `process_block` | Function | `omni-lang/std/crypto_audit.rs` | 183 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Test_aes_gcm_encrypt_decrypt → Process_block` | cross_community | 4 |
| `Test_aes_gcm_authentication → Process_block` | cross_community | 4 |
| `Test_aes_gcm_empty_plaintext → Process_block` | cross_community | 4 |

## How to Explore

1. `gitnexus_context({name: "new"})` — see callers and callees
2. `gitnexus_query({query: "std"})` — find related execution flows
3. Read key files listed above for implementation details
