# Helios + Omni

**A systems programming language with ownership semantics, and the cognitive framework built on top of it.**

[![CI](https://github.com/shreyashjagtap157/Helios/actions/workflows/ci.yml/badge.svg)](https://github.com/shreyashjagtap157/Helios/actions/workflows/ci.yml)
[![Tests](https://img.shields.io/badge/tests-1%2C019%20passing-brightgreen)](#testing)
[![Rust](https://img.shields.io/badge/rust-2021-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-Apache%202.0-green)](LICENSE)

---

## The Problem

Existing systems languages force a choice: **performance** (C/C++) or **safety** (Go, Java). Neither was designed with AI workloads, cognitive reasoning, or evidence-aware computation as first-class concerns.

**Omni** aims to be a language where memory safety comes from the type system (ownership + borrowing, no GC), AI and reasoning primitives are native, and the compiler can eventually compile itself.

**Helios** is the proof — a cognitive framework built on Omni for knowledge storage, adaptive reasoning, and evidence-based workflows.

---

## Quick Start

**Prerequisites:** [Rust 1.75+](https://rustup.rs/) (edition 2021)

```bash
git clone https://github.com/shreyashjagtap157/Helios.git
cd Helios/omni-lang/compiler

# Build
cargo build

# Run a program
cargo run --bin omnc -- --run ../examples/hello.omni

# Run tests
cargo test

# Lint
cargo clippy
```

**Hello World** (`hello.omni`):
```omni
module hello

fn main():
    let msg = own String::from("Hello, World!")
    println(msg)
```

```bash
cargo run --bin omnc -- --run hello.omni
# Output: Hello, World!
```

---

## Project Status

| Component | Status | Details |
|-----------|--------|---------|
| **Compiler (omnc)** | Working | Full pipeline: lexer → parser → semantic → IR → codegen/runtime |
| **Tests** | 1,019 passing | 547 integration + 472 unit, 0 failures |
| **Clippy** | ~109 warnings | Mostly unused vars, dead code, naming conventions |
| **Format** | Clean (1 diff) | token_dump.rs import order |
| **OVM Bytecode** | Working | Default runtime target; interpreter + bytecode VM |
| **LLVM Native** | Feature-gated | `--features llvm` (requires LLVM 17) |
| **GPU Backend** | Feature-gated | `--features gpu` (CUDA/OpenCL/Vulkan) |
| **Standard Library** | 37 modules | 21,617 lines — crypto, math, networking, I/O, async, etc. |
| **Self-Hosted Compiler** | In Progress | 34 files, 28,433 lines of Omni source |
| **Bootstrap Pipeline** | Placeholder | Stage 0 works (Rust); Stages 1-2 are copies |
| **Tools** | Implemented | LSP (2,305 lines), DAP (1,717), formatter (417), package manager (2,765), VS Code extension |
| **Helios Framework** | Scaffolding | Core modules exist, not fully functional |
| **Examples** | Partial | hello.omni and minimal.omni work; 3/5 tutorials fail |

---

## Repository Layout

```
Helios/
├── omni-lang/
│   ├── compiler/          The omnc compiler (Rust, 110 .rs files)
│   │   └── src/
│   │       ├── main.rs         CLI entry point
│   │       ├── lib.rs          Library root
│   │       ├── lexer/          Logos-based tokenizer
│   │       ├── parser/         Recursive-descent parser + AST
│   │       ├── semantic/       Type inference, borrow checking, traits
│   │       ├── ir/             Intermediate representation
│   │       ├── optimizer/      Constant folding, DCE, inlining
│   │       ├── codegen/        OVM, LLVM, native, GPU, JIT, MLIR
│   │       ├── runtime/        Interpreter, bytecode VM, hot-swap
│   │       ├── brain/          Adaptive reasoning modules
│   │       └── language_features/  Default params, variadics, overloading
│   ├── std/               Standard library (37 .omni modules, 21,617 lines)
│   ├── core/              Core library modules
│   ├── omni/              Self-hosted compiler source (34 .omni files, 28,433 lines)
│   ├── tools/
│   │   ├── omni-fmt/      Source formatter (417 lines, 3 tests)
│   │   ├── omni-lsp/      Language Server (2,305 lines)
│   │   ├── omni-dap/      Debug Adapter (1,717 lines)
│   │   ├── opm/           Package manager (2,765 lines)
│   │   └── vscode-omni/   VS Code extension (TypeScript)
│   ├── tests/             Integration tests
│   ├── examples/          Example .omni programs
│   └── ovm/               Standalone OVM runner
│
├── helios-framework/      Helios cognitive framework
│   ├── helios/            Core runtime surfaces
│   ├── brain/             Cognitive modules
│   ├── training/          Training pipeline
│   ├── app/               User-facing app / GUI
│   ├── safety/            Governance and action control
│   ├── biometrics/        Identity verification
│   ├── config/            Configuration management
│   └── kernel/            Hot-swap infrastructure
│
├── .github/
│   ├── workflows/ci.yml           GitHub Actions CI
│   ├── ISSUE_TEMPLATE/            Bug, feature, question templates
│   ├── PULL_REQUEST_TEMPLATE.md   PR checklist
│   └── labels.yml                 Label definitions
│
├── README.md              This file
├── LICENSE                Apache 2.0
├── CONTRIBUTING.md        How to contribute
├── CODE_OF_CONDUCT.md     Contributor Covenant v2.0
└── ISSUES.md              Known issues, good first issues, help wanted
```

---

## Omni Language

### Design Principles

- **Memory safety without GC** — ownership system with `own`, `shared`, `&`, `&mut`
- **Indentation-based syntax** — no braces or semicolons
- **Trait-based generics** — zero-cost abstractions via monomorphization
- **Built-in concurrency** — `async`/`await`, threads, channels
- **Multiple execution modes** — script, hosted, bare_metal
- **Multiple backends** — interpreter, OVM bytecode, LLVM native (optional)

### Example: Ownership

```omni
fn ownership_demo():
    let owned = own String::from("data")      // Unique ownership
    let shared = shared owned                  // Shared (read-only) reference
    let borrow = &owned                        // Immutable borrow
    let mut_borrow = &mut owned                // Mutable borrow (exclusive)
```

### Example: Traits and Generics

```omni
trait Printable:
    fn display(&self) -> String

struct Point:
    x: f64
    y: f64

impl Printable for Point:
    fn display(&self) -> String:
        return format("({}, {})", self.x, self.y)

fn print_item<T: Printable>(item: &T):
    println(item.display())
```

### Example: Pattern Matching

```omni
enum Shape:
    Circle(radius: f64)
    Rectangle(width: f64, height: f64)

fn area(shape: &Shape) -> f64:
    match shape:
        Shape::Circle(r):
            return 3.14159 * r * r
        Shape::Rectangle(w, h):
            return w * h
```

### Example: Async

```omni
async fn fetch_data(url: &str) -> Result<String, Error>:
    let response = await http::get(url)?
    return await response.text()

async fn main():
    let data = await fetch_data("https://api.example.com/data")
    println("Got: {}", data)
```

### Compiler Pipeline

```
Source (.omni) → Lexer (Logos, INDENT/DEDENT) → Parser (recursive-descent, AST)
    → Semantic (type inference, borrow checking, trait resolution)
    → IR (with optimization: constant folding, DCE, inlining)
    → Codegen (OVM default | LLVM opt | GPU opt)
    → Runtime (interpreter + bytecode VM)
```

### Standard Library (37 modules)

| Domain | Modules | Size |
|--------|---------|------|
| **Core** | core, error, option, result | ~1,800 lines |
| **Collections** | array, map, set, queue, stack, linked_list | ~3,000 lines |
| **I/O** | io, file | ~2,200 lines |
| **String** | string | ~730 lines |
| **Networking** | net, http | ~1,600 lines |
| **Crypto** | crypto | ~1,900 lines |
| **Math** | math | ~1,300 lines |
| **Async** | async, sync | ~1,500 lines |
| **Serialization** | json, yaml | ~1,500 lines |
| **Other** | env, args, security, performance, test, time, process, ... | ~6,000 lines |

### Building

```bash
cd omni-lang/compiler

# Debug
cargo build

# Release
cargo build --release

# With LLVM
cargo build --features llvm

# With GPU
cargo build --features gpu
```

### Running Programs

```bash
# Run
cargo run --bin omnc -- --run program.omni

# Check syntax only
cargo run --bin omnc -- --check program.omni

# Verbose output
cargo run --bin omnc -- --run --verbose program.omni
```

---

## Helios Framework

Helios is a cognitive framework built on Omni. It provides knowledge storage, evidence-aware workflows, adaptive reasoning, and multi-mode operation (CLI, REPL, API, service).

**Status:** Scaffolding — core module structure exists but is not fully functional.

**Modules:**
- `helios/` — Core runtime surfaces (API, service, knowledge, IO)
- `brain/` — Adaptive reasoning, knowledge graph, memory, pattern recognition
- `training/` — Ingestion, pruning, checkpoints, optimization
- `app/` — User-facing app and GUI hooks
- `safety/` — Governance and action control
- `biometrics/` — Identity verification
- `config/` — Configuration management
- `kernel/` — Hot-swap infrastructure

---

## Self-Hosting

Omni aims to be self-hosting: the compiler will eventually be written in and compiled by Omni.

| Component | Status |
|-----------|--------|
| Self-hosted source | 34 files, 28,433 lines (real code, not stubs) |
| Pipeline coverage | Full — lexer, parser, semantic, IR, codegen, linker |
| Stage 0 | Working — Rust bootstrap compiler |
| Stage 1 | PLACEHOLDER — copies Stage 0 |
| Stage 2 | PLACEHOLDER — copies Stage 1 |

**Blocking issues (O-100 through O-106):**
- Monomorphization must specialize generic functions
- IR must preserve actual types (not hardcode I64)
- Codegen must emit binaries / OVM bytecode files
- Linker must produce standalone executables
- Parse error in self-hosted main.omni
- Compilation timeout on compiler/main.omni

---

## Testing

| Type | Count | Status |
|------|-------|--------|
| **Lib tests** | 547 | All passing |
| **Bin tests** | 472 | All passing |
| **Total** | 1,019 | 0 failures |

```bash
# All tests
cargo test

# Lib only
cargo test --lib

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture
```

**Clippy:** ~109 unique warnings (unused vars, dead code, naming).

**Example status:**
| Program | Result |
|---------|--------|
| hello.omni | Works (warnings) |
| minimal.omni | Works clean |
| integration_test.omni | 2 test failures, crashes on boolean logic |
| tutorial_01 | FAILS — unsupported RangeInclusive |
| tutorial_03 | FAILS — undefined variable: math |
| tutorial_04 | FAILS — cannot iterate HashMap |
| tutorial_05 | FAILS — undefined variable: messages |

---

## Contributing

First time? See [CONTRIBUTING.md](CONTRIBUTING.md) for setup, code style, and PR process.

- [Good First Issues](ISSUES.md#good-first-issues) — well-scoped beginner tasks
- [Help Wanted](ISSUES.md#help-wanted) — tasks that need community help

**Branch strategy:** `main` (stable) ← `dev` (active) ← `feature/*`

**PR checklist:**
1. Fork → branch from `dev` → make changes → `cargo test` + `cargo clippy` → PR to `dev`

---

## License

Apache License 2.0 — see [LICENSE](LICENSE).

```
Copyright 2024 Shreyash Jagtap
```

---

**Last updated:** 2026-03-29
