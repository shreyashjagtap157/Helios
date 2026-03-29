---
name: cluster-35
description: "Skill for the Cluster_35 area of Helios. 16 symbols across 1 files."
---

# Cluster_35

16 symbols | 1 files | Cohesion: 67%

## When to Use

- Working with code in `omni-lang/`
- Understanding how publish, main, print_usage work
- Modifying cluster_35-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `omni-lang/tools/opm/src/main_v0.rs` | publish, main, print_usage, init_project, test_project (+11) |

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `publish` | Function | `omni-lang/tools/opm/src/main_v0.rs` | 424 |
| `main` | Function | `omni-lang/tools/opm/src/main_v0.rs` | 583 |
| `print_usage` | Function | `omni-lang/tools/opm/src/main_v0.rs` | 630 |
| `init_project` | Function | `omni-lang/tools/opm/src/main_v0.rs` | 680 |
| `test_project` | Function | `omni-lang/tools/opm/src/main_v0.rs` | 989 |
| `bench_project` | Function | `omni-lang/tools/opm/src/main_v0.rs` | 1056 |
| `check_project` | Function | `omni-lang/tools/opm/src/main_v0.rs` | 1074 |
| `find_source_files` | Function | `omni-lang/tools/opm/src/main_v0.rs` | 1110 |
| `visit_dir` | Function | `omni-lang/tools/opm/src/main_v0.rs` | 1113 |
| `remove_dependency` | Function | `omni-lang/tools/opm/src/main_v0.rs` | 1217 |
| `publish_package` | Function | `omni-lang/tools/opm/src/main_v0.rs` | 1556 |
| `create_package_tarball` | Function | `omni-lang/tools/opm/src/main_v0.rs` | 1601 |
| `generate_docs` | Function | `omni-lang/tools/opm/src/main_v0.rs` | 1693 |
| `show_dep_tree` | Function | `omni-lang/tools/opm/src/main_v0.rs` | 1757 |
| `format_code` | Function | `omni-lang/tools/opm/src/main_v0.rs` | 1784 |
| `lint_code` | Function | `omni-lang/tools/opm/src/main_v0.rs` | 1815 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Main → Home_dir` | cross_community | 5 |
| `Main → As_str` | cross_community | 5 |
| `Main → Num_cpus` | cross_community | 3 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Cluster_33 | 7 calls |
| Cluster_36 | 4 calls |
| Brain | 3 calls |
| Cluster_34 | 2 calls |
| Codegen | 2 calls |
| Cluster_30 | 1 calls |
| Runtime | 1 calls |

## How to Explore

1. `gitnexus_context({name: "publish"})` — see callers and callees
2. `gitnexus_query({query: "cluster_35"})` — find related execution flows
3. Read key files listed above for implementation details
