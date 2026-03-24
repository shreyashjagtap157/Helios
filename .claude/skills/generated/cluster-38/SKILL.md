---
name: cluster-38
description: "Skill for the Cluster_38 area of Helios. 12 symbols across 1 files."
---

# Cluster_38

12 symbols | 1 files | Cohesion: 94%

## When to Use

- Working with code in `omni-lang/`
- Understanding how find_manifest, read_manifest, write_manifest work
- Modifying cluster_38-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `omni-lang/tools/opm/src/main.rs` | find_manifest, read_manifest, write_manifest, cmd_init, cmd_add (+7) |

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `find_manifest` | Function | `omni-lang/tools/opm/src/main.rs` | 102 |
| `read_manifest` | Function | `omni-lang/tools/opm/src/main.rs` | 115 |
| `write_manifest` | Function | `omni-lang/tools/opm/src/main.rs` | 120 |
| `cmd_init` | Function | `omni-lang/tools/opm/src/main.rs` | 128 |
| `cmd_add` | Function | `omni-lang/tools/opm/src/main.rs` | 171 |
| `cmd_remove` | Function | `omni-lang/tools/opm/src/main.rs` | 189 |
| `cmd_install` | Function | `omni-lang/tools/opm/src/main.rs` | 201 |
| `cmd_build` | Function | `omni-lang/tools/opm/src/main.rs` | 223 |
| `cmd_run` | Function | `omni-lang/tools/opm/src/main.rs` | 242 |
| `cmd_publish` | Function | `omni-lang/tools/opm/src/main.rs` | 263 |
| `cmd_search` | Function | `omni-lang/tools/opm/src/main.rs` | 274 |
| `main` | Function | `omni-lang/tools/opm/src/main.rs` | 285 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Cmd_run → Positional` | cross_community | 4 |
| `Main → OmniManifest` | intra_community | 3 |
| `Main → PackageInfo` | intra_community | 3 |
| `Main → Write_manifest` | intra_community | 3 |
| `Main → Find_manifest` | intra_community | 3 |
| `Main → Read_manifest` | intra_community | 3 |
| `Cmd_run → Find_manifest` | intra_community | 3 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Codegen | 2 calls |
| Runtime | 1 calls |

## How to Explore

1. `gitnexus_context({name: "find_manifest"})` — see callers and callees
2. `gitnexus_query({query: "cluster_38"})` — find related execution flows
3. Read key files listed above for implementation details
