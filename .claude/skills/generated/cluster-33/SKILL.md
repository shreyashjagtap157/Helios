---
name: cluster-33
description: "Skill for the Cluster_33 area of Helios. 10 symbols across 1 files."
---

# Cluster_33

10 symbols | 1 files | Cohesion: 56%

## When to Use

- Working with code in `omni-lang/`
- Understanding how new, build_project, run_project work
- Modifying cluster_33-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `omni-lang/tools/opm/src/main_v0.rs` | new, build_project, run_project, clean_build, dir_size (+5) |

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `new` | Function | `omni-lang/tools/opm/src/main_v0.rs` | 357 |
| `build_project` | Function | `omni-lang/tools/opm/src/main_v0.rs` | 790 |
| `run_project` | Function | `omni-lang/tools/opm/src/main_v0.rs` | 945 |
| `clean_build` | Function | `omni-lang/tools/opm/src/main_v0.rs` | 1652 |
| `dir_size` | Function | `omni-lang/tools/opm/src/main_v0.rs` | 1676 |
| `read_lockfile` | Function | `omni-lang/tools/opm/src/main_v0.rs` | 1841 |
| `generate_lockfile` | Function | `omni-lang/tools/opm/src/main_v0.rs` | 1885 |
| `compute_hash` | Function | `omni-lang/tools/opm/src/main_v0.rs` | 1915 |
| `hash_path` | Function | `omni-lang/tools/opm/src/main_v0.rs` | 1919 |
| `num_cpus` | Function | `omni-lang/tools/opm/src/main_v0.rs` | 1948 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Main → Home_dir` | cross_community | 5 |
| `Main → As_str` | cross_community | 5 |
| `Resolve → Home_dir` | cross_community | 4 |
| `Resolve → As_str` | cross_community | 4 |
| `Main → Num_cpus` | cross_community | 3 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Cluster_36 | 2 calls |
| Brain | 2 calls |
| Codegen | 2 calls |
| Cluster_35 | 1 calls |

## How to Explore

1. `gitnexus_context({name: "new"})` — see callers and callees
2. `gitnexus_query({query: "cluster_33"})` — find related execution flows
3. Read key files listed above for implementation details
