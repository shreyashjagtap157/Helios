# Omni Programming Language

<!-- Badges -->
![Version](https://img.shields.io/badge/version-1.0.0-blue)
![Build](https://img.shields.io/badge/build-passing-brightgreen)
![License](https://img.shields.io/badge/license-proprietary-lightgrey)
![Tests](https://img.shields.io/badge/tests-674%20passing-brightgreen)
![Platform](https://img.shields.io/badge/platform-windows%20%7C%20linux%20%7C%20macos-lightgrey)

---

## Overview

**Omni** is a modern systems programming language designed for building safe, performant software with first-class support for AI workloads. It combines the safety guarantees of Rust-style ownership with Python-like readability through indentation-based syntax.

### Key Design Principles

- **Memory safety without garbage collection** — ownership system with `own`, `shared`, `&`, and `&mut` semantics
- **Expressive and readable** — indentation-based blocks, no braces or semicolons
- **Trait-based generics** — zero-cost abstractions through monomorphization
- **Built-in concurrency** — `async`/`await`, threads, channels, and an executor runtime
- **Comprehensive standard library** — networking, crypto, filesystem, collections, math, and more
- **Multiple backends** — tree-walking interpreter, OVM bytecode, and native code generation

---

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/helios-project/omni-lang.git
cd omni-lang/compiler

# Build the compiler
cargo build --release

# The compiler binary is at target/release/omnc
```

### Hello World

Create a file called `hello.omni`:

```omni
module hello

fn main():
    println("Hello, World!")
```

### Running

```bash
# Interpret directly
omnc run hello.omni

# Compile to OVM bytecode
omnc build hello.omni -o hello.ovm
omnc run hello.ovm

# Compile to native binary (requires LLVM feature)
omnc build hello.omni --target native -o hello
```

---

## Language Features

### Ownership & Borrowing

Omni enforces memory safety at compile time through an ownership system inspired by Rust, with four ownership modes:

```omni
fn ownership_demo():
    let owned = own String::from("hello")   // Unique ownership
    let shared_ref = shared owned            // Shared (read-only) reference
    let borrow = &owned                      // Immutable borrow
    let mut_borrow = &mut owned              // Mutable borrow (exclusive)
```

### Traits and Generics

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

### Async / Await

```omni
async fn fetch_data(url: &str) -> Result<String, Error>:
    let response = await http::get(url)?
    return await response.text()

async fn main():
    let data = await fetch_data("https://api.example.com/data")
    match data:
        Result::Ok(text):
            println("Got: {}", text)
        Result::Err(e):
            println("Error: {}", e)
```

### Pattern Matching

```omni
enum Shape:
    Circle(radius: f64)
    Rectangle(width: f64, height: f64)
    Triangle(a: f64, b: f64, c: f64)

fn area(shape: &Shape) -> f64:
    match shape:
        Shape::Circle(r):
            return 3.14159 * r * r
        Shape::Rectangle(w, h):
            return w * h
        Shape::Triangle(a, b, c):
            let s = (a + b + c) / 2.0
            return math::sqrt(s * (s - a) * (s - b) * (s - c))
```

### Modules and Imports

```omni
module myapp::utils

import std::collections::{HashMap, Vector}
import std::io
import std::math as m

fn compute():
    let pi = m::PI
    println("Pi is {}", pi)
```

### Error Handling

```omni
fn read_config(path: &str) -> Result<Config, Error>:
    let content = fs::read_to_string(path)?    // ? propagates errors
    let config = parse_toml(content)?
    return Result::Ok(config)

fn main():
    match read_config("config.toml"):
        Result::Ok(cfg):
            println("Loaded: {}", cfg.name)
        Result::Err(e):
            println("Failed: {}", e)
```

---

## Compiler Architecture

The `omnc` compiler processes source code through a multi-stage pipeline:

```
Source (.omni)
    │
    ▼
┌─────────┐   Logos-based tokenizer with indentation tracking
│  Lexer   │   Produces token stream with INDENT/DEDENT tokens
└────┬────┘
     ▼
┌─────────┐   Recursive-descent parser with error recovery
│  Parser  │   Produces a typed AST (Module → Items → Expressions)
└────┬────┘
     ▼
┌──────────────────┐   Name resolution, type checking,
│ Semantic Analysis │   trait bound verification
└────────┬─────────┘
         ▼
┌────────────────┐   Hindley-Milner algorithm with
│ Type Inference  │   constraint generation and unification
└────────┬───────┘
         ▼
┌───────────────┐   Ownership tracking, lifetime analysis,
│ Borrow Checker │   move semantics validation
└───────┬───────┘
        ▼
┌──────────────┐   Constant folding, dead code elimination,
│ Optimization  │   inlining, algebraic simplification
└──────┬───────┘
       ▼
┌──────────────┐   Tree-walking interpreter (default)
│  Code Gen /   │   OVM bytecode compiler
│  Interpret    │   Native via LLVM (optional)
└──────────────┘   ELF64 / PE / Mach-O linker
```

### Optimization Levels

| Level | Passes |
|-------|--------|
| `-O0` | No optimization |
| `-O1` | Constant folding + simplification |
| `-O2` | All passes including dead code elimination and inlining |
| `-O3` | Aggressive inlining with all passes |

---

## Tools

| Tool | Description |
|------|-------------|
| **omnc** | The Omni compiler — lexing, parsing, analysis, optimization, codegen |
| **omni-lsp** | Language Server Protocol server — diagnostics, completion, hover |
| **omni-dap** | Debug Adapter Protocol server — breakpoints, stepping, variables |
| **opm** | Package manager — `init`, `add`, `remove`, `install`, `build`, `run`, `publish`, `search` |
| **omni-fmt** | Code formatter |
| **vscode-omni** | VS Code extension — syntax highlighting, LSP/DAP integration |

### Package Manager (opm)

```bash
opm init myproject          # Create new project with omni.toml
opm add std::crypto         # Add a dependency
opm build                   # Build the project
opm run                     # Build and run
opm test                    # Run tests
```

Project manifest format (`omni.toml`):

```toml
[package]
name = "myproject"
version = "0.1.0"
edition = "2026"

[dependencies]
std = "1.0"
```

---

## Standard Library

| Module | Description |
|--------|-------------|
| `std::core` | Fundamental traits (`Clone`, `Copy`, `Display`, `Send`, `Sync`), `Option`, `Result` |
| `std::collections` | `Vector`, `HashMap`, `HashSet`, `VecDeque`, `BTreeMap`, `LinkedList` |
| `std::io` | File I/O, streams, buffered readers/writers, stdin/stdout |
| `std::net` | TCP/UDP sockets, HTTP client/server, WebSocket, DNS resolution |
| `std::math` | Constants, trigonometry, linear algebra, `Vector3`, `Matrix4` |
| `std::string` | String utilities — `repeat`, `pad_left`, `center`, `StringBuilder` |
| `std::thread` | Thread spawning, `Mutex`, `RwLock`, `Channel`, `Barrier`, `Condvar` |
| `std::time` | `Duration`, `Instant`, `DateTime`, formatting, `sleep` |
| `std::fs` | File/directory CRUD, path manipulation, permissions, directory walking |
| `std::crypto` | SHA-256/512, AES-GCM, ChaCha20, TLS, X.509 certificates |
| `std::async` | `Future` trait, `async`/`await`, executor, task spawning |
| `std::rand` | ChaCha20-based CSPRNG, uniform/normal distributions |
| `std::regex` | Regular expression matching |
| `std::serde` | Serialization/deserialization framework |
| `std::json` | JSON parsing and generation |
| `std::env` | Environment variables, command-line arguments |
| `std::sys` | OS information, CPU count, current directory |
| `std::compress` | Compression algorithms |
| `std::image` | Image loading and manipulation |
| `std::tensor` | Tensor operations for AI/ML workloads |
| `std::reflect` | Runtime reflection |

---

## Building from Source

### Prerequisites

- **Rust** 1.75+ (with Cargo)
- **LLVM 17** (optional, for native code generation)

### Build

```bash
cd omni-lang/compiler

# Default build (interpreter + OVM)
cargo build --release

# With LLVM native codegen
cargo build --release --features llvm

# With GPU backend support
cargo build --release --features gpu

# Run the test suite
cargo test
```

### Verify

```bash
# Run the hello world example
cargo run -- run ../examples/hello.omni
```

---

## Project Structure

```
omni-lang/
├── compiler/               # The omnc compiler (Rust)
│   └── src/
│       ├── main.rs          # CLI entry point
│       ├── lib.rs           # Library root
│       ├── lexer/           # Logos-based tokenizer
│       ├── parser/          # Recursive-descent parser + AST
│       ├── semantic/        # Type checking, borrow checker, inference
│       ├── optimizer/       # Constant folding, DCE, inlining, simplify
│       ├── codegen/         # OVM, native, LLVM, linker, JIT
│       ├── runtime/         # Tree-walking interpreter, bytecode VM
│       ├── ir/              # Intermediate representation
│       └── diagnostics.rs   # Error reporting
├── std/                     # Standard library (.omni source)
│   ├── core.omni            # Fundamental types and traits
│   ├── collections.omni     # Data structures
│   ├── io.omni              # Input/output
│   ├── net.omni             # Networking
│   ├── math.omni            # Mathematics
│   ├── thread.omni          # Concurrency primitives
│   ├── time.omni            # Time and duration
│   ├── fs.omni              # Filesystem
│   ├── crypto.omni          # Cryptography
│   ├── async.omni           # Async runtime
│   └── ...                  # Additional modules
├── tools/                   # Ecosystem tools
│   ├── omni-lsp/            # Language Server Protocol server
│   ├── omni-dap/            # Debug Adapter Protocol server
│   ├── omni-fmt/            # Code formatter
│   ├── opm/                 # Package manager
│   └── vscode-omni/         # VS Code extension
├── examples/                # Example programs
├── tests/                   # Integration tests
├── docs/                    # Documentation
│   ├── grammar.bnf          # Formal grammar specification
│   ├── language_guide.md    # Language tutorial
│   ├── compiler_internals.md # Compiler contributor guide
│   └── standard_library_reference.md # API reference
└── ovm/                     # Omni Virtual Machine specification
```

---

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes and add tests
4. Ensure all tests pass: `cd compiler && cargo test`
5. Submit a pull request

### Code Style

- Rust code follows standard `rustfmt` formatting
- Omni code uses 4-space indentation
- All public APIs must have doc comments (`///`)

### Running Tests

```bash
cd omni-lang/compiler
cargo test                    # All tests
cargo test lexer              # Lexer tests only
cargo test parser             # Parser tests only
cargo test semantic           # Semantic analysis tests
cargo test optimizer          # Optimizer tests
```

---

## License

Proprietary — HELIOS Project. All rights reserved.
