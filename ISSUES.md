# Helios + Omni Issues & Tasks

> **New contributor?** Start with a [Good First Issue](#good-first-issues).  
> **Experienced?** See [Help Wanted](#help-wanted).

---

## Executive Summary (2026-03-30)

### What's Working ✅

| Component | Status | Notes |
|-----------|--------|-------|
| Rust-based Compiler (omnc) | ✅ Working | Compiles and runs Omni source code |
| Bytecode Emission | ✅ Working | `omnc file.omni -o output.ovm` |
| OVM Runtime | ✅ Working | `omnc --run file.ovm` executes bytecode |
| Self-Hosted Compiler | ✅ Working | `compiler_minimal.omni` compiles and runs |
| Bootstrap Pipeline | ✅ Working | `bootstrap.sh` demonstrates self-hosting |
| Installation Scripts | ✅ Working | `install.sh`, `install.ps1`, `uninstall.sh` |
| Examples | ✅ Working | All 15 examples compile successfully |
| Cargo Tests | ✅ Working | 547+ tests passing |

### What's Broken/Limited ❌

| Component | Status | Notes |
|-----------|--------|-------|
| Type Inference | ⚠️ Limited | Fails on complex expressions, arithmetic in returns |
| Function Parameters | ⚠️ Limited | Without explicit types, causes inference errors |
| Full Self-Hosting | ❌ Not Achieved | Still need Rust to build omnc |
| Native Binary Emission | ❌ Not Working | `--emit native` doesn't produce working binaries |
| Bootstrap Stages 1-2 | ❌ Not Implemented | Only Stage 0 works |
| Complex Language Features | ❌ Limited | Closures, advanced patterns, complex generics |

---

## CRITICAL: Self-Hosting Blockers

These issues prevent Omni from compiling itself without Rust.

### SH-001: Type Inference Fails on Function Return Expressions

**Status:** 🔴 OPEN  
**Priority:** CRITICAL  
**Component:** compiler, semantic analysis

**Description:**
The type inference system fails when functions return arithmetic expressions. This causes runtime errors like:
```
Error: Type mismatch for integer operation
```

**Root Cause:**
The type inference system creates constraints for expressions but fails to properly unify the return type with the function's declared return type. Specifically:
1. When a function returns `a + b` (arithmetic), the type inference doesn't properly infer `Int`
2. The constraint solver doesn't properly propagate types from binary operations to return types
3. Runtime receives incorrect type information, causing execution to fail

**Affected Code Patterns:**
```omni
fn add(a: int, b: int) -> int:
    return a + b  // FAILS - type inference doesn't resolve correctly
```

**Workaround (Current):**
```omni
fn add(a: int, b: int) -> int:
    return a  // WORKS - no complex expression
```

**What Needs to Happen:**
1. Fix type inference for binary operations in return statements
2. Ensure constraint solver properly unifies return types
3. Add tests for all arithmetic operations in function returns

**Test Case:**
```omni
fn add(a: int, b: int) -> int:
    return a + b

fn main():
    let x = add(1, 2)  # Should work
    println(x)
```

---

### SH-002: Bootstrap Stages 1-2 Not Implemented

**Status:** 🔴 OPEN  
**Priority:** CRITICAL  
**Component:** bootstrap

**Description:**
The bootstrap pipeline only has Stage 0 working. True self-hosting requires:
- Stage 0: Rust omnc compiles self-hosted source → bytecode
- Stage 1: Self-hosted compiler compiles itself → new bytecode
- Stage 2: Stage 1 output compiles itself → verify bit-identical output

**Current Status:**
```
Stage 0: ✅ Working - Rust omnc compiles compiler_minimal.omni
Stage 1: ❌ Not implemented - needs self-hosted compiler that compiles real code
Stage 2: ❌ Not implemented - needs Stage 1
```

**What Needs to Happen:**
1. Enhance `compiler_minimal.omni` to be a real compiler (lex/parse/codegen)
2. Implement Stage 1: Self-hosted compiler compiles itself
3. Implement Stage 2: Stage 1 compiles Stage 1
4. Verify bit-identical output (proves bootstrap is correct)

---

### SH-003: No Native Binary Emission

**Status:** 🔴 OPEN  
**Priority:** CRITICAL  
**Component:** codegen

**Description:**
The `omnc --emit native` option doesn't produce working native executables. Only the bytecode runtime execution works.

**What Needs to Happen:**
1. Fix codegen to produce valid native binaries
2. Implement proper OVM → native compilation
3. Test on Windows, Linux, macOS

---

## HIGH PRIORITY: Type System Issues

### HP-001: Type Inference for Function Parameters Without Explicit Types

**Status:** 🔴 OPEN  
**Priority:** HIGH  
**Component:** semantic analysis, type inference

**Description:**
When function parameters don't have explicit type annotations, the type inference produces errors:
```
warning: type inference: Function expects 1 arguments but 2 were provided – function call
```

**Root Cause:**
The type inference creates a fresh type variable for parameters but doesn't properly constrain it when the function is called with concrete arguments.

**Affected Code:**
```omni
fn add(a, b):  # No type annotations
    return a + b

fn main():
    let x = add(1, 2)  # Fails
```

**Workaround:**
```omni
fn add(a: int, b: int) -> int:  # Explicit types
    return a + b
```

**What Needs to Happen:**
1. Fix parameter type inference to work without explicit annotations
2. Ensure constraints are properly generated from call sites
3. Test with various parameter counts and types

---

### HP-002: Borrow Checker Errors on Simple Assignments

**Status:** 🔴 OPEN  
**Priority:** HIGH  
**Component:** borrow checker

**Description:**
The borrow checker produces errors on simple variable assignments:
```
warning[E006]: borrow check: use of moved value `b`: moved at stmt 1, used at stmt 1
```

**Affected Code:**
```omni
fn add(a: int, b: int) -> int:
    let result = a + b  # May fail
    return result
```

**What Needs to Happen:**
1. Fix borrow checker to understand simple arithmetic doesn't need borrowing
2. Implement Copy semantics for primitive types
3. Add proper move/copy semantics for all types

---

### HP-003: Variable Scope Issues in Loops

**Status:** 🔴 OPEN  
**Priority:** HIGH  
**Component:** semantic analysis

**Description:**
Loop variables are incorrectly flagged as having ownership issues:
```
warning[E006]: borrow check: value `i` moved inside loop at stmt 2 (would be used again in next iteration)
```

**Affected Code:**
```omni
fn main():
    for i in 0..5:
        println(i)  # Fails
```

**What Needs to Happen:**
1. Fix loop variable semantics
2. Ensure loop variables are properly scoped
3. Implement proper iteration semantics

---

## MEDIUM PRIORITY: Parser Limitations

### MP-001: Closures Not Fully Supported

**Status:** 🔴 OPEN  
**Priority:** MEDIUM  
**Component:** parser, lexer

**Description:**
Closure syntax `|x| expr` is recognized by the lexer but causes type inference errors.

**Affected Code:**
```omni
fn main():
    let add = |a: int, b: int| a + b
    let x = add(1, 2)
```

**Error:**
```
warning: type inference: Expected numeric type (Int or Float) but found _ in left operand of Add
```

**What Needs to Happen:**
1. Fix type inference for closure parameters
2. Implement closure codegen
3. Add tests for closures

---

### MP-002: Complex Pattern Matching Not Supported

**Status:** 🔴 OPEN  
**Priority:** MEDIUM  
**Component:** parser

**Description:**
Advanced pattern matching features like match expressions with complex patterns fail.

**What Needs to Happen:**
1. Implement full match expression support
2. Add pattern matching for enums, tuples, structs

---

### MP-003: Advanced Generics Not Supported

**Status:** 🔴 OPEN  
**Priority:** MEDIUM  
**Component:** parser, semantic

**Description:**
While basic nested generics (`Vec<Option<T>>`) now work after SH-003 fix, more complex generic patterns fail.

**What Needs to Happen:**
1. Implement generic trait bounds
2. Implement where clauses
3. Implement generic associated types

---

### MP-004: Struct Field Inference Issues

**Status:** 🔴 OPEN  
**Priority:** MEDIUM  
**Component:** semantic analysis

**Description:**
Struct field access causes type inference errors:
```
warning: type inference: Undefined variable 'Point'
```

**Affected Code:**
```omni
struct Point:
    x: int
    y: int

fn main():
    let p = Point { x: 1, y: 2 }
    println(p.x)  # May fail
```

**What Needs to Happen:**
1. Fix struct type resolution
2. Implement proper field access type inference

---

### MP-005: Module Resolution Issues

**Status:** 🔴 OPEN  
**Priority:** MEDIUM  
**Component:** resolver

**Description:**
Imports don't resolve correctly:
```
warning: unresolved import 'core/logging' (as 'log'): file not found
```

**What Needs to Happen:**
1. Fix module resolution system
2. Implement proper import paths
3. Add stdlib module files

---

## LOW PRIORITY: Missing Features

### LP-001: Standard Library Incomplete

**Status:** 🔴 OPEN  
**Priority:** LOW  
**Component:** stdlib

**Description:**
The stdlib modules are incomplete or missing. Many examples fail because stdlib functions don't exist.

**What Needs to Happen:**
1. Implement core::io, core::system, core::memory
2. Implement collections (Vector, HashMap, etc.)
3. Implement networking, async modules

---

### LP-002: No GPU Backend

**Status:** 🔴 OPEN  
**Priority:** LOW  
**Component:** codegen

**Description:**
GPU compilation via LLVM/CUDA/OpenCL is not implemented.

**What Needs to Happen:**
1. Implement GPU codegen
2. Add CUDA backend
3. Add OpenCL backend

---

### LP-003: No LLVM Backend

**Status:** 🔴 OPEN  
**Priority:** LOW  
**Component:** codegen

**Description:**
Native compilation via LLVM is not fully implemented.

**What Needs to Happen:**
1. Complete LLVM backend implementation
2. Add cross-compilation support
3. Test on multiple platforms

---

### LP-004: No Package Manager (opm)

**Status:** 🔴 OPEN  
**Priority:** LOW  
**Component:** tooling

**Description:**
The opm package manager exists but has no tests.

**What Needs to Happen:**
1. Add tests for manifest parsing
2. Add tests for dependency resolution
3. Add integration tests

---

### LP-005: No LSP Tests

**Status:** 🔴 OPEN  
**Priority:** LOW  
**Component:** tooling

**Description:**
The omni-lsp has no tests.

**What Needs to Happen:**
1. Add document sync tests
2. Add completion tests
3. Add hover tests
4. Add diagnostics tests

---

## Good First Issues

### GFI-001: Fix Unnecessary Parentheses

**Status:** 🔴 OPEN  
**Labels:** good first issue, component: compiler  
**Difficulty:** Easy  

Clippy flags unnecessary parentheses. Fix them.

```
cargo clippy --lib 2>&1 | grep "unnecessary parentheses"
```

---

### GFI-002: Fix ~109 Clippy Warnings

**Status:** 🔴 OPEN  
**Labels:** good first issue, component: compiler  
**Difficulty:** Easy-Medium  

Warning categories:
- ~40 unused variables
- ~10 redundant closures
- ~19 first-element access issues
- ~12 div_ceil reimplementations
- ~8 dead code items
- ~5 complex type aliases
- ~4 or_insert_with → or_default
- ~4 unreachable patterns

---

### GFI-003: Add Default Implementations for Brain Types

**Status:** 🔴 OPEN  
**Labels:** good first issue, component: helios  
**Difficulty:** Easy  

Files: `omni-lang/compiler/src/brain/`

`AdaptiveReasoner`, `KnowledgeGraph`, `MemorySystem`, `PatternRecognizer` need `impl Default`.

---

### GFI-004: Fix tutorial Examples

**Status:** 🔴 OPEN  
**Labels:** good first issue, area: examples  
**Difficulty:** Medium  

Tutorial examples need to be fixed to work with the current compiler:
- tutorial_01_basics.omni - Simplified, but could have more features
- tutorial_02_ownership.omni - Only has placeholder code
- tutorial_03_structs_traits.omni - Only has placeholder code

---

## Help Wanted Issues

### HW-001: Fix Boolean Logic Crash

**Status:** 🔴 OPEN  
**Labels:** help wanted, bug, area: runtime  
**Difficulty:** Hard  

`integration_test.omni` crashes on boolean logic operations.

---

### HW-002: Fix Array/Stack Operations

**Status:** 🔴 OPEN  
**Labels:** help wanted, bug, area: runtime  
**Difficulty:** Hard  

Array and stack operations fail in tests.

---

### HW-003: Implement HashMap Iteration

**Status:** 🔴 OPEN  
**Labels:** help wanted, feature, component: stdlib  
**Difficulty:** Hard  

"Cannot iterate over Map" error.

---

## Known Compiler Errors (Complete List)

### Parser Errors

| Error Code | Message | Status | Workaround |
|------------|---------|--------|------------|
| E001 | Unexpected token | Fixed | Use simpler syntax |
| E002 | Invalid syntax (Indent) | Fixed | Use `{}` instead of `:` for structs |
| E009 | Too many errors | Open | Simplify code |

### Type Inference Errors

| Error | Message | Status | Workaround |
|-------|---------|--------|------------|
| W001 | Function expects N arguments but M were provided | Open | Use explicit parameter types |
| W002 | Expected numeric type but found String | Open | Avoid arithmetic with inferred types |
| W003 | Undefined variable | Open | Declare variables explicitly |
| W004 | Type mismatch | Open | Use explicit types |
| W005 | Borrow check failure | Open | Avoid move semantics |

### Runtime Errors

| Error | Message | Status | Workaround |
|-------|---------|--------|------------|
| R001 | Type mismatch for integer operation | Open | Use simpler return expressions |
| R002 | Undefined function | Open | N/A - missing implementation |
| R003 | Stack underflow | Open | N/A - runtime issue |

---

## Testing Status

### Cargo Tests
```
547 tests passing
```

### Example Tests
| Example | Compiles | Runs | Notes |
|---------|----------|------|-------|
| minimal | ✅ | ✅ | Works perfectly |
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
| integration_test | ✅ | ⚠️ | Simplified, minimal features |
| interpreter_test | ✅ | ⚠️ | Simplified, minimal features |

---

## Label Reference

| Label | Meaning |
|-------|---------|
| `good first issue` | Beginner-friendly, well-scoped |
| `help wanted` | Needs community help |
| `bug` | Something is broken |
| `enhancement` | Improvement to existing feature |
| `feature` | New capability |
| `priority: critical` | Blocks core functionality |
| `priority: high` | Should fix soon |
| `priority: medium` | Important but not urgent |
| `priority: low` | Nice to have |
| `component: compiler` | Omni compiler |
| `component: stdlib` | Standard library |
| `component: helios` | Helios framework |
| `component: tooling` | LSP, DAP, formatter, opm |

---

**Last updated:** 2026-03-30

**Next steps:**
1. Fix SH-001: Type inference for arithmetic in returns (CRITICAL)
2. Implement SH-002: Bootstrap stages 1-2 (CRITICAL)
3. Fix HP-001 through HP-003: Type system issues (HIGH)
4. Complete MP-* issues (MEDIUM)
5. Complete LP-* features (LOW)
