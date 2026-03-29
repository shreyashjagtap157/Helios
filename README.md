# Helios + Omni

A monorepo containing the **Omni programming language** and the **Helios cognitive framework**.

[![Omni Compiler](https://img.shields.io/badge/omnc-v2.0.0-blue)](omni-lang/compiler/)
[![Tests](https://img.shields.io/badge/tests-1417%20passing-brightgreen)](#testing)
[![Rust Edition](https://img.shields.io/badge/rust-2021-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-proprietary-lightgrey)](#license)

---

## Table of Contents

- [What Is This](#what-is-this)
- [Project Status](#project-status)
- [Repository Layout](#repository-layout)
- [Omni Language](#omni-language)
  - [Language Overview](#language-overview)
  - [Compiler Architecture](#compiler-architecture)
  - [Standard Library](#standard-library)
  - [Module Modes](#module-modes)
  - [Tools](#tools)
  - [Building the Compiler](#building-the-compiler)
  - [Running Omni Programs](#running-omni-programs)
  - [Project Structure](#omni-project-structure)
- [Helios Framework](#helios-framework)
  - [Framework Overview](#framework-overview)
  - [Architecture](#helios-architecture)
  - [Key Modules](#key-modules)
  - [Running Helios](#running-helios)
- [Self-Hosting](#self-hosting)
- [Testing](#testing)
- [Development Workflow](#development-workflow)
- [Documentation](#documentation)
- [License](#license)

---

## What Is This

This repository contains two tightly-coupled systems:

**Omni** (`omni-lang/`) — A systems/application programming language written in Rust. Features ownership semantics, trait-based generics, an interpreter, a bytecode VM, optional LLVM native codegen, and a self-hosting bootstrap.

**Helios** (`helios-framework/`) — A cognitive framework built on Omni. Provides exact knowledge storage, evidence-aware workflows, capability orchestration, persistent memory, and multi-mode operation (CLI, REPL, API, service).

Omni provides the language and compiler substrate. Helios is the framework built on top of it.

---

## Project Status

| Component | Status | Details |
|-----------|--------|---------|
| **Compiler (omnc)** | Operational | Full pipeline: lexer -> parser -> semantic -> IR -> codegen/runtime |
| **Tests** | 1,417 passing | 746 lib tests + 671 bin tests, 0 warnings |
| **Clippy** | Clean | 0 warnings on default feature set |
| **OVM Bytecode** | Operational | Default execution target; tree-walking interpreter + bytecode VM |
| **LLVM Native** | Feature-gated | Optional backend via `--features llvm` (requires LLVM 17) |
| **GPU Backend** | Feature-gated | CUDA/OpenCL/Vulkan support via `--features gpu` |
| **Standard Library** | 30+ modules | Core, collections, I/O, networking, crypto, async, math, etc. |
| **Self-Hosted Compiler** | In Progress | Source in Omni exists; binary emission is the critical path |
| **Bootstrap Pipeline** | Partial | Stage 0 functional; Stages 1-2 in development |
| **Omni Tools** | Implemented | LSP, DAP, formatter, package manager, VS Code extension |
| **Helios Framework** | In Development | Core runtime, brain modules, training pipeline scaffolding |
| **Documentation** | Partial | Language reference, BNF grammar, status reports exist |

---

## Repository Layout

```
Helios/
├── omni-lang/              Omni language: compiler, stdlib, tools, tests, self-hosting
│   ├── compiler/           The omnc compiler (Rust)
│   ├── std/                Standard library (.omni source, 30+ modules)
│   ├── core/               Core library modules (math, io, json, http, etc.)
│   ├── omni/               Self-hosted Omni compiler source
│   ├── tools/              Ecosystem tools (LSP, DAP, formatter, package manager)
│   ├── tests/              Integration tests (.omni + .rs)
│   ├── examples/           Example programs
│   ├── ovm/                Omni Virtual Machine (standalone runner)
│   └── docs/               Language grammar, internal docs
│
├── helios-framework/       Helios cognitive framework
│   ├── helios/             Core runtime surfaces (API, service, knowledge, IO)
│   ├── brain/              Cognitive modules (reasoning, memory, learning)
│   ├── training/           Ingestion, pruning, checkpoints, optimization
│   ├── app/                User-facing app and GUI hooks
│   ├── safety/             Governance and action control
│   ├── biometrics/         Identity verification
│   ├── config/             Configuration defaults and loaders
│   └── kernel/             Hot-swap infrastructure
│
├── helios-wrapper/         Rust binary wrapping the Helios framework
├── docs/                   Project-level documentation and references
├── diagnostics/            Build and clippy logs
├── scripts/                PowerShell utility scripts
├── config/                 Root-level configuration assets
├── examples/               Root-level example programs
├── build/                  Build artifacts (gitignored)
│
├── README.md               This file
├── AGENTS.md               AI assistant operating conventions
├── CLAUDE.md               Claude-specific integration config
├── CHANGELOG.md            Release changelog
└── CONTRIBUTING.md         Contribution guidelines
```

---

## Omni Language

### Language Overview

Omni is a systems programming language designed for safety, performance, and AI workloads.

**Core design principles:**

- **Memory safety without garbage collection** — ownership system with `own`, `shared`, `&`, and `&mut` semantics
- **Readable syntax** — indentation-based blocks (Python-like), no braces or semicolons
- **Trait-based generics** — zero-cost abstractions through monomorphization
- **Built-in concurrency** — `async`/`await`, threads, channels, executor runtime
- **Multiple execution modes** — script, hosted, bare_metal
- **Multiple backends** — interpreter, OVM bytecode VM, LLVM native (optional)

**Example:**

```omni
module hello

fn main():
    let owned = own String::from("Hello, World!")
    println(owned)
```

**Ownership modes:**

```omni
fn ownership_demo():
    let owned = own String::from("data")      // Unique ownership
    let shared = shared owned                   // Shared (read-only) reference
    let borrow = &owned                         // Immutable borrow
    let mut_borrow = &mut owned                 // Mutable borrow (exclusive)
```

**Traits and generics:**

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

**Pattern matching:**

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

**Async/await:**

```omni
async fn fetch_data(url: &str) -> Result<String, Error>:
    let response = await http::get(url)?
    return await response.text()

async fn main():
    let data = await fetch_data("https://api.example.com/data")
    println("Got: {}", data)
```

### Compiler Architecture

The `omnc` compiler processes source code through a multi-stage pipeline:

```
Source (.omni)
    |
    v
+---------+   Logos-based tokenizer with indentation tracking
|  Lexer   |   Produces token stream with INDENT/DEDENT tokens
+----+----+
     |
     v
+---------+   Recursive-descent parser with error recovery
|  Parser  |   Produces typed AST (Module -> Items -> Expressions)
+----+----+
     |
     v
+------------------+   Name resolution, type checking,
| Semantic Analysis |   trait bound verification, monomorphization
+--------+---------+
         |
         v
+----------------+   Hindley-Milner with constraint
| Type Inference  |   generation and unification
+--------+-------+
         |
         v
+---------------+   Ownership tracking, lifetime analysis,
| Borrow Checker |   move semantics validation
+-------+-------+
        |
        v
+--------------+   Constant folding, dead code elimination,
| Optimization  |   inlining, algebraic simplification
+------+-------+
       |
       v
+
