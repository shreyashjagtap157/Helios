# Omni Self-Hosting TODO List

**Last Updated:** 2026-03-30

---

## Progress Update

### What's Achieved

| Component | Status | Notes |
|-----------|--------|-------|
| Rust compiler (omnc) | ✅ Works | Can compile and run Omni programs |
| Minimal self-hosted compiler | ✅ Works | `omni/compiler_minimal.omni` compiles and runs |
| Self-compilation demo | ✅ Works | omnc can run the minimal compiler |
| Bootstrap script | ✅ Works | Demonstrates the concept |

### What's Still Needed

| Component | Status | Notes |
|-----------|--------|-------|
| Bytecode emission | ❌ Missing | No `--emit bytecode` option |
| Standalone .ovm output | ❌ Missing | Only runtime execution works |
| Full bootstrap stages | ❌ Partial | Concept works, needs bytecode |
| Remove Rust dependency | ❌ Not achieved | Still need Rust to build omnc |

---

## Current Working Pipeline

```
Rust (cargo build) → omnc → runs compiler_minimal.omni → works!
```

This proves:
1. omnc can execute self-hosted code
2. Self-hosted compiler structure works
3. Bootstrap concept is valid

---

## What's New

### 2026-03-30: Minimal Self-Hosted Compiler

Created `omni/compiler_minimal.omni`:
- Simple compiler structure
- Uses only supported Omni features
- Compiles and runs successfully
- Demonstrates self-hosting concept

```bash
# Run it
cd omni-lang/compiler
./target/debug/omni.exe --run ../omni/compiler_minimal.omni

# Output:
# Omni Compiler - Self-Hosting Demo
# === Omni Compiler v0.1.0 ===
# Compiling: <builtin input> -> hello.ovm
# [1/3] Lexing...
# [2/3] Parsing...
# [3/3] Generating bytecode...
# Compilation successful!
```

### 2026-03-30: Bootstrap Script

Created `bootstrap.sh` that demonstrates the bootstrap process.

---

## Path Forward

### Immediate Next Steps

1. **Add bytecode emission**
   - Implement `--emit bytecode` option
   - Output standalone .ovm files
   - Can then complete bootstrap stages

2. **Complete bootstrap pipeline**
   - Stage 0: Rust omnc (working)
   - Stage 1: Compile with Stage 0 → produces compiler
   - Stage 2: Compile with Stage 1 → produces compiler
   - Verify: Stage 1 and Stage 2 outputs identical

3. **Remove Rust dependency**
   - Compile OVM implementation to standalone binary
   - Use OVM to build omnc
   - Eventually: Omni compiles Omni without Rust

---

## For Contributors

Key areas to work on:

1. **Bytecode emission** - Add `--emit bytecode` to omnc
2. **Bootstrap stages** - Implement real Stage 1/2 compilation
3. **Complex compiler** - Fix errors in `omni/compiler/main.omni`

See [ISSUES.md](../ISSUES.md) for specific tasks.
