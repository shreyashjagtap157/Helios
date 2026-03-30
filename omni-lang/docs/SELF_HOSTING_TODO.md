# Omni Self-Hosting TODO List

**Last Updated:** 2026-03-30

---

## Honest Assessment

**Current Status: NOT SELF-HOSTING**

The goal is to make Omni compile itself WITHOUT Rust. This document tracks progress toward that goal.

---

## The Reality

| Component | Status | Notes |
|-----------|--------|-------|
| Rust compiler (omnc) | ✅ Works | Can compile simple Omni programs |
| Self-hosted source | ✅ Exists | ~13,000 lines in `omni-lang/omni/compiler/` |
| omnc → self-hosted compiler | ❌ FAILS | 56 errors - too complex for current omnc |
| Bootstrap Stage 1 | ❌ Placeholder | Just copies Stage 0 |
| Bootstrap Stage 2 | ❌ Placeholder | Just copies Stage 1 |
| **True self-hosting** | ❌ NOT ACHIEVED | Must compile without Rust |

---

## What Works (Simple Programs)

```omni
✅ let x = 5
✅ let z = x + y
✅ if x > 5: ... else: ...
✅ fn main(): ...
✅ struct Point: x int, y int
✅ enum Color: Red, Green, Blue
✅ for i in range: ...
✅ while cond: ...
✅ match x: ...
✅ println("hello")
```

---

## What Doesn't Work

### Critical Blockers

1. **Self-hosted compiler cannot be compiled**
   - 56 errors when omnc tries to compile `omni/main.omni`
   - Complex generic constraints unsupported
   - Pattern matching edge cases fail
   - Trait bounds incomplete

2. **Bootstrap pipeline is non-functional**
   - Stage 0: Rust omnc (works)
   - Stage 1: Placeholder (copies Stage 0)
   - Stage 2: Placeholder (copies Stage 1)

3. **No standalone binary emission**
   - `omnc --emit native` doesn't produce working .exe
   - Only runtime execution works

---

## Path to True Self-Hosting

### Phase 1: Fix omnc to Compile Self-Hosted Source

- [ ] Fix the 56 compilation errors
- [ ] Simplify self-hosted source if needed
- [ ] Test: `omnc --run omni/main.omni` works

### Phase 2: Working Bootstrap Pipeline

- [ ] Stage 0: Rust omnc compiles self-hosted source → binary
- [ ] Stage 1: Use Stage 0 binary to compile self-hosted source
- [ ] Stage 2: Use Stage 1 binary to compile again
- [ ] Verify: Stage 1 and Stage 2 output is bit-identical

### Phase 3: Remove Rust Dependency

- [ ] OVM implementation in Omni (not Rust)
- [ ] Standalone build system in Omni
- [ ] Final: Omni compiles itself without any Rust

---

## Recent Fixes

- **2026-03-30**: Fixed single quote parsing issue (replaced `don't` with `do not`)
- **Result**: Parse error fixed, but 56 new errors surfaced

---

## Contributing

This is the most critical work on the project. See [ISSUES.md](../ISSUES.md) for specific tasks.

The path forward:
1. Fix omnc to handle more language features
2. Simplify self-hosted source to what omnc CAN compile
3. Build the bootstrap pipeline
4. Eventually replace Rust entirely
