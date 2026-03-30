# Omni Language Compiler Issues

> **Project-wide issues:** See [../ISSUES.md](../ISSUES.md)

---

## Quick Reference

| Category | Priority | Count |
|----------|----------|-------|
| Critical (Self-Hosting Blockers) | 🔴 CRITICAL | 3 |
| High Priority | 🔴 HIGH | 3 |
| Medium Priority | ⚠️ MEDIUM | 5 |
| Low Priority | 📋 LOW | 5 |

---

## 🔴 CRITICAL: Self-Hosting Blockers

These issues **block** Omni from compiling itself without Rust.

### SH-001: Type Inference Fails on Function Return Expressions

**Status:** 🔴 OPEN  
**Priority:** CRITICAL

**Error:**
```
Error: Type mismatch for integer operation
```

**Affected Code:**
```omni
fn add(a: int, b: int) -> int:
    return a + b  // FAILS
```

**Workaround:** Use simpler returns:
```omni
fn add(a: int, b: int) -> int:
    return a  // WORKS
```

**Root Cause:** Constraint solver doesn't unify return types with binary operations.

**What needs to happen:**
1. Fix type inference for binary operations
2. Ensure return type unification works correctly
3. Add regression tests

**Files to examine:**
- `compiler/src/semantic/type_inference.rs`
- `compiler/src/semantic/mod.rs`

---

### SH-002: Bootstrap Stages 1-2 Not Implemented

**Status:** 🔴 OPEN  
**Priority:** CRITICAL

**Current Pipeline:**
```
Stage 0: ✅ Rust omnc compiles compiler_minimal.omni → bytecode
Stage 1: ❌ Needs self-hosted compiler that compiles real code
Stage 2: ❌ Needs Stage 1 output
```

**What needs to happen:**
1. Enhance `omni/compiler_minimal.omni` to be a real compiler
2. Implement Stage 1: Self-hosted compiles itself
3. Implement Stage 2: Stage 1 compiles itself
4. Verify bit-identical output

---

### SH-003: No Native Binary Emission

**Status:** 🔴 OPEN  
**Priority:** CRITICAL

`omnc --emit native` doesn't produce working executables.

**What needs to happen:**
1. Fix codegen for native output
2. Implement proper binary format
3. Test on Windows, Linux, macOS

---

## 🔴 HIGH PRIORITY: Type System

### HP-001: Function Parameters Without Types Fail

**Status:** 🔴 OPEN  
**Priority:** HIGH

**Error:**
```
warning: type inference: Function expects 1 arguments but 2 were provided
```

**Affected Code:**
```omni
fn add(a, b):  // No type annotations
    return a + b
```

**Workaround:** Explicit types required:
```omni
fn add(a: int, b: int) -> int:
    return a + b
```

---

### HP-002: Borrow Checker Errors on Simple Assignments

**Status:** 🔴 OPEN  
**Priority:** HIGH

**Error:**
```
warning[E006]: borrow check: use of moved value
```

**Affected Code:**
```omni
fn add(a: int, b: int) -> int:
    let result = a + b  // May fail
    return result
```

---

### HP-003: Loop Variable Scope Issues

**Status:** 🔴 OPEN  
**Priority:** HIGH

**Error:**
```
warning[E006]: borrow check: value moved inside loop
```

**Affected Code:**
```omni
for i in 0..5:
    println(i)  // Fails
```

---

## ⚠️ MEDIUM PRIORITY: Parser Limitations

### MP-001: Closures Partially Supported

**Status:** 🔴 OPEN  
**Priority:** MEDIUM

Closure syntax is recognized but type inference fails.

```omni
let add = |a: int, b: int| a + b  // Fails at runtime
```

---

### MP-002: Complex Pattern Matching Limited

**Status:** 🔴 OPEN  
**Priority:** MEDIUM

Match expressions work for simple cases but fail for:
- Nested patterns
- Guard clauses
- Complex enum matching

---

### MP-003: Advanced Generics Not Supported

**Status:** 🔴 OPEN  
**Priority:** MEDIUM

Basic nested generics work (`Vec<Option<T>>`) but:
- Generic trait bounds fail
- Where clauses fail
- Generic associated types fail

---

### MP-004: Struct Field Inference Issues

**Status:** 🔴 OPEN  
**Priority:** MEDIUM

**Error:**
```
warning: type inference: Undefined variable 'Point'
```

**Affected Code:**
```omni
struct Point:
    x: int
    y: int

fn main():
    let p = Point { x: 1, y: 2 }  // May fail
```

---

### MP-005: Module Resolution Issues

**Status:** 🔴 OPEN  
**Priority:** MEDIUM

**Error:**
```
warning: unresolved import 'core/logging'
```

Imports don't resolve correctly for stdlib modules.

---

## 📋 LOW PRIORITY: Missing Features

### LP-001: Standard Library Incomplete

**Status:** 📋 OPEN  
**Priority:** LOW

Missing implementations:
- `core::io`
- `core::system`
- `core::memory`
- `collections::Vector`
- `collections::HashMap`

---

### LP-002: No GPU Backend

**Status:** 📋 OPEN  
**Priority:** LOW

GPU compilation via CUDA/OpenCL/Vulkan not implemented.

---

### LP-003: No LLVM Backend

**Status:** 📋 OPEN  
**Priority:** LOW

LLVM native compilation incomplete.

---

### LP-004: No Package Manager Tests

**Status:** 📋 OPEN  
**Priority:** LOW

`opm` package manager has no tests.

---

### LP-005: No LSP Tests

**Status:** 📋 OPEN  
**Priority:** LOW

`omni-lsp` has no tests.

---

## Good First Issues

### GFI-001: Fix ~109 Clippy Warnings

**Difficulty:** Easy-Medium

```bash
cd omni-lang/compiler
cargo clippy --lib 2>&1 | grep "warning"
```

Categories:
- ~40 unused variables
- ~10 redundant closures
- ~19 first-element access issues
- ~12 div_ceil reimplementations

---

### GFI-002: Fix Unnecessary Parentheses

**Difficulty:** Easy

```bash
cargo clippy --lib 2>&1 | grep "unnecessary parentheses"
```

---

### GFI-003: Add Default Implementations

**Difficulty:** Easy

Files: `compiler/src/brain/`

Need `impl Default` for:
- `AdaptiveReasoner`
- `KnowledgeGraph`
- `MemorySystem`
- `PatternRecognizer`

---

## Compiler Error Reference

### Parser Errors

| Code | Message | Status |
|------|---------|--------|
| E001 | Unexpected token | ✅ Fixed |
| E002 | Invalid syntax (Indent) | ✅ Fixed |
| E009 | Too many errors | ⚠️ Workaround needed |

### Type Inference Warnings

| Code | Message | Status |
|------|---------|--------|
| W001 | Function expects N arguments | 🔴 Open |
| W002 | Expected numeric type | 🔴 Open |
| W003 | Undefined variable | 🔴 Open |
| W004 | Type mismatch | 🔴 Open |
| W005 | Borrow check failure | 🔴 Open |

### Runtime Errors

| Code | Message | Status |
|------|---------|--------|
| R001 | Type mismatch for integer | 🔴 Open |
| R002 | Undefined function | 🔴 Open |
| R003 | Stack underflow | 🔴 Open |

---

## Testing Status

### Cargo Tests
```
547 tests passing
```

### Example Programs

| Example | Compiles | Runs | Notes |
|---------|----------|------|-------|
| minimal | ✅ | ✅ | Works |
| simple_test | ✅ | ✅ | Works |
| func_test | ✅ | ✅ | Works |
| func_test2 | ✅ | ✅ | Works |
| hello | ✅ | ✅ | Works |
| std_demo | ✅ | ✅ | Works |
| match_comprehensive | ✅ | ✅ | Works |
| struct_test | ✅ | ✅ | Works |
| tutorial_01 | ✅ | ✅ | Simplified |
| tutorial_02 | ✅ | ✅ | Placeholder |
| tutorial_03 | ✅ | ✅ | Placeholder |
| tutorial_04 | ✅ | ✅ | Placeholder |
| tutorial_05 | ✅ | ✅ | Placeholder |
| integration_test | ✅ | ⚠️ | Simplified |
| interpreter_test | ✅ | ⚠️ | Simplified |

---

## Source File Reference

```
compiler/
├── src/
│   ├── main.rs           # Entry point, CLI
│   ├── lib.rs            # Library root
│   ├── lexer/
│   │   └── mod.rs        # Tokenization
│   ├── parser/
│   │   └── mod.rs        # AST generation
│   ├── semantic/
│   │   ├── mod.rs        # Type checking
│   │   ├── type_inference.rs    # Type inference engine
│   │   └── borrow_check.rs    # Borrow checker
│   ├── codegen/
│   │   ├── mod.rs        # Code generation
│   │   └── ovm_direct.rs # Bytecode emission
│   ├── ir/
│   │   └── mod.rs        # IR representation
│   └── runtime/
│       └── mod.rs        # OVM bytecode interpreter

omni/
├── compiler_minimal.omni  # Self-hosted compiler (minimal)
└── main.omni             # Helios entry point

std/
└── *.omni               # Standard library (incomplete)

examples/
└── *.omni               # 15 example programs
```

---

## Label Reference

| Label | Meaning |
|-------|---------|
| `SH-*` | Self-Hosting blockers |
| `HP-*` | High Priority issues |
| `MP-*` | Medium Priority issues |
| `LP-*` | Low Priority issues |
| `GFI-*` | Good First Issues |
| `good first issue` | Beginner-friendly |
| `help wanted` | Needs community help |
| `bug` | Something is broken |
| `feature` | New capability |

---

**Last updated:** 2026-03-30
