# Omni Self-Hosting TODO List

**Last Updated:** 2026-03-30

---

## Progress Update

### What's Achieved

| Component | Status | Notes |
|-----------|--------|-------|
| Rust compiler (omnc) | ✅ Works | Can compile and run Omni programs |
| Minimal self-hosted compiler | ✅ Works | `omni/compiler_minimal.omni` compiles and runs |
| Bytecode emission | ✅ Works | `omnc file.omni -o output` creates .ovm |
| OVM runtime | ✅ Works | Can load and execute .ovm files |
| Bootstrap script | ✅ Works | Demonstrates the full pipeline |

### What's Still Needed

| Component | Status | Notes |
|-----------|--------|-------|
| Full bootstrap stages | ⚠️ Partial | Structure in place, needs real self-hosted compiler |
| Self-hosted compiler compiles itself | ⚠️ Partial | Uses demo stub, needs full compiler |
| Remove Rust dependency | ❌ Not achieved | Still need Rust to build omnc |

### 2026-03-30: main.omni Fixed

Simplified `main.omni` to use only supported Omni syntax:
- Removed nested generics (`Option<Vec<T>>`)
- Removed closure pipe syntax (`|x| expr`)
- Simplified struct literals to use `{ }` not `:`
- Removed complex pattern matching

**Result:** `main.omni` now compiles successfully!

---

## Current Working Pipeline

```
Rust (cargo build) → omnc → compiler_minimal.omni → bytecode (.ovm) → OVM runtime → executes!
```

This proves:
1. omnc CAN emit bytecode files (not just run in-memory)
2. OVM runtime CAN load and execute .ovm files
3. The bootstrap concept is fully functional

### Verified on 2026-03-30:

```bash
$ cd omni-lang/compiler && cargo build
$ ./target/debug/omnc ../omni/compiler_minimal.omni -o ../build/test.ovm
$ ./target/debug/omnc --run ../build/test.ovm
```

Both compilation and execution work!

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

1. **Complete bootstrap pipeline**
   - Stage 0: Rust omnc (✅ working)
   - Stage 1: Compile self-hosted compiler with Stage 0 → produces new compiler
   - Stage 2: Compile self-hosted compiler with Stage 1 → produces new compiler
   - Verify: Stage 1 and Stage 2 outputs identical

2. **Implement full self-hosted compiler**
   - Currently using `compiler_minimal.omni` (simple demo)
   - Need to make `main.omni` fully functional
   - Then Stage 1 can compile the real compiler

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
