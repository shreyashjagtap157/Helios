# OMNI PROGRAMMING LANGUAGE
## Exhaustive Specification, Design Rationale & Implementation Guide
**Version:** 2.0 | **Bootstrap:** Rust | **License:** Apache 2.0
**Last Revised:** 2026-04 — Incorporates improvements from research into Rust 2024 Edition roadmap and pain points, algebraic effect systems (Koka/Eff/Unison), Vale's generational references and linear types, Mojo/MLIR AI acceleration, Zig's comptime build model, Swift/Kotlin structured concurrency, and state-of-the-art compiler diagnostics (Elm/Rust standards).

---

## TABLE OF CONTENTS

1. [What Omni Is](#1-what-omni-is)
2. [How Every Decision Was Made — The Full Rationale](#2-how-every-decision-was-made)
3. [Language Philosophy & Design Principles](#3-language-philosophy--design-principles)
4. [Type System](#4-type-system)
5. [Memory Model & Ownership](#5-memory-model--ownership)
6. [Effect System — A Unified Model for Side Effects](#6-effect-system)
7. [Concurrency & Execution Model](#7-concurrency--execution-model)
8. [Syntax & Surface Design](#8-syntax--surface-design)
9. [Module, Package & Visibility System](#9-module-package--visibility-system)
10. [Error Handling & Failure Model](#10-error-handling--failure-model)
11. [Standard Library Architecture](#11-standard-library-architecture)
12. [Compilation Model & IR Design](#12-compilation-model--ir-design)
13. [Runtime Architecture](#13-runtime-architecture)
14. [Tooling & Developer Experience](#14-tooling--developer-experience)
15. [Testing, Diagnostics & Validation](#15-testing-diagnostics--validation)
16. [Security, Safety & Capability System](#16-security-safety--capability-system)
17. [Interoperability & FFI](#17-interoperability--ffi)
18. [Bootstrap Strategy & Self-Hosting Roadmap](#18-bootstrap-strategy--self-hosting-roadmap)
19. [Phased Implementation Plan](#19-phased-implementation-plan)
20. [HELIOS Framework (Platform Layer)](#20-helios-framework-platform-layer)
21. [Current State & What Remains](#21-current-state--what-remains)
22. [Improvements Added in v2.0 — Research Basis](#22-improvements-added-in-v20)

---

## 1. WHAT OMNI IS

Omni is a **multi-level, hybrid programming language platform** designed for advanced developers, combining strong safety and performance guarantees with flexible abstraction layers. It enables controlled transitions between low-level and high-level programming through explicit execution modes, prioritizes deterministic correctness, and serves as the foundational language for building complex platform-level systems such as HELIOS.

This definition emerged through an exhaustive requirements interview covering over 160 design decisions, then refined in v2.0 through systematic research into the current state of language design — examining Rust's 2024 roadmap improvements and documented pain points, algebraic effect systems (Koka, Eff, Unison), Vale's linear types and generational references, Mojo's MLIR-based AI acceleration, Zig's comptime build model, structured concurrency from Swift and Kotlin, and best-in-class compiler diagnostic systems.

### 1.1 What Omni Is Not

Omni is explicitly not a beginner scripting language. It is not a loose general-purpose language with no enforced boundaries. It is not designed primarily for UI-heavy frontend development. It is not a toy compiler project. It is not a research-only language that will never ship. It is a structured platform language with a strict, deterministic core and carefully designed extensible outer layers.

### 1.2 The One-Sentence Definition

> **Omni is a layered, multi-paradigm, multi-runtime programming language platform with a structured ecosystem, deterministic core, algebraic effect system, and controlled extensibility, serving as the foundation for building advanced systems including the HELIOS cognitive platform.**

### 1.3 Primary Domains

Systems programming, backend services, AI and data infrastructure, and platform/framework construction. Secondary: CLI tooling and libraries. UI-heavy frontend is the acknowledged weak area — possible but not optimized.

---

## 2. HOW EVERY DECISION WAS MADE

### 2.1 Why "Everything, But Structured"

The initial design intent was to do everything. The resolution was organizing goals into layers. This pattern — layered architecture with strict defaults and controlled escalation — is the backbone of every design decision. The v2.0 research confirmed this approach is correct: every successful multi-domain language (Rust, Swift, Kotlin, Mojo) converged on layering as the answer to the "do everything" problem.

### 2.2 Why Not Choose One Memory Model

Omni's use cases span from low-level systems code where GC is unacceptable overhead, to high-level application code where manual memory management is unnecessary pain. The decision: ownership as the core model, with GC-compatible layer in higher-level modes. v2.0 adds generational references (from Vale) as an ergonomic bridge for graph-like and cyclic data structures, eliminating the `Rc<RefCell<T>>` ceremony that is one of Rust's most documented frustrations.

### 2.3 Why Rust as the Bootstrap Language

Omni's design philosophy maps almost exactly onto Rust's design space. Using Rust means the development team works in a mental model aligned with what Omni itself will become. v2.0 reconfirms this: the Rust 2024 ecosystem (Cranelift, Salsa, tower-lsp, rowan, polonius-engine) provides a mature, well-maintained foundation for every layer of the bootstrap project.

### 2.4 Why Determinism Is the Sacred Principle

No undefined behavior, deterministic correctness by default. v2.0 adds: determinism must extend to the effect system — all effects (IO, async, exceptions) must be trackable at the type level, so programs can reason about their side effects as precisely as they reason about their types.

### 2.5 Why an Effect System Was Added in v2.0

Research into Koka (Microsoft Research), Eff, and Unison reveals that an algebraic effect system cleanly unifies many language mechanisms that would otherwise require independent design: exceptions, async/await, generators, cancellation, probabilistic programming, and other control-flow abstractions. Koka has proven this approach is practically usable — not just theoretically interesting. Omni adopts a staged effect system: built-in effects first, user-defined effects later.

### 2.6 Why Structured Concurrency Was Strengthened

Research into Kotlin's coroutine model and Swift's structured concurrency manifesto shows that the most common concurrency bugs come from tasks that escape their intended scope. Structured concurrency (child tasks cannot outlive their parent scope) prevents this class of bugs by construction. Kotlin enforces parent-child relationships through its Job hierarchy, ensuring that child coroutines cannot outlive their parents, which prevents common resource leaks. Omni adopts structured concurrency as a hard constraint enforced at the type system level.

### 2.7 Why Borrow Checker Ergonomics Were Explicitly Addressed

The Rust language team's own 2024 roadmap explicitly acknowledges borrow checker ergonomics as the primary usability problem with ownership-based languages. Specific friction points documented by the community: where clause repetition, graph/cyclic data structures requiring `Rc<RefCell<T>>`, inability to partially borrow struct fields independently, and self-referential structure difficulty. Omni addresses all of these proactively: implied bounds, field projection support, Polonius from day one, generational references.

### 2.8 Why Diagnostic Quality Is a Language Feature

Research comparing compiler diagnostics across Rust, Elm, Go, Java, TypeScript, Kotlin, and Python confirms Elm and Rust lead the field. Crucially: good diagnostics require both UX design and architectural discipline — the compiler must preserve enough information at each pass to produce helpful messages. Omni treats diagnostic quality as a first-class design constraint, not an afterthought.

---

## 3. LANGUAGE PHILOSOPHY & DESIGN PRINCIPLES

### 3.1 Core Philosophy

Six foundational principles, ordered by priority:

**1. Deterministic Correctness First.** No undefined behavior. No silent failures. Every execution explainable by the language rules. All side effects trackable through the type-and-effect system.

**2. Layered Complexity, Not Flat Complexity.** A beginner in high-level mode is not exposed to ownership, effect annotations, or capability concepts unless they descend. An expert has full access to memory layout, unsafe operations, and raw hardware interfaces.

**3. Safe Defaults, Controlled Escape Hatches.** The default for everything is the safest available option. Unsafe, mutable, nondeterministic, or privileged behavior requires explicit opt-in. Always visible in code review, never implicit.

**4. Effects Are Visible in Types.** Every function that performs IO, launches a task, allocates heap memory, or has any side effect declares those effects in its type. Pure functions are pure by construction.

**5. Everything Extensible, But Not Everywhere Enabled.** The macro system, plugin system, metaprogramming, and runtime code generation all exist, but none are enabled everywhere by default.

**6. Phased Implementation From Structure to Platform.** The language is built in phases: correct structural core → functional core → enrichment → expansion → platform-level maturity.

### 3.2 Design Priority Ordering

When priorities conflict:
1. Safety and correctness (never sacrificed)
2. Performance (never sacrificed without an explicit, visible trade-off)
3. Developer productivity
4. Flexibility and extensibility (layered, never at the cost of 1 or 2)
5. Ecosystem breadth (important in later phases; irrelevant in early phases)

### 3.3 The "No Footguns at Default Reach" Principle (v2.0 Addition)

Features that are commonly misused or cause subtle bugs must be placed behind explicit opt-in at whatever abstraction level they exist. This principle was validated by examining Rust's documented pain points: shared mutable state, implicit drop-as-cancellation in async, implicit `Rc<RefCell<T>>` patterns. Every convenience in Omni must be explicitly invoked.

---

## 4. TYPE SYSTEM

### 4.1 Typing Style

Omni is statically typed by default. The typing model is a **type-and-effect system**: every expression has both a type and an effect set. A function with no side effects has the `pure` effect (empty set). A function reading from IO has the `io` effect. These appear in signatures and are checked by the compiler.

Layers:
- **Default**: static typing, effect inference
- **Strict**: effects must be explicitly annotated; for library APIs and critical systems
- **Dynamic zones**: explicitly marked modules or blocks where types resolve at runtime

### 4.2 Type Inference

Bidirectional type checking (not purely H-M top-down). More powerful than classic Hindley-Milner, produces better error messages, aligns with the research direction of Koka and recent functional languages. Effects are inferred in non-public code. Public API boundaries require explicit type and effect annotations.

Verbosity control: `@verbose_types` annotation forces explicit type expansion on a binding for debugging. `--types=minimal` flag suppresses inferred-type annotations in error messages.

### 4.3 Null Handling

Null does not exist in safe Omni code. Absence is `Option<T>`. Null pointers accessible in unsafe mode only. The `Option<T>` API includes a rich combinator set (`map`, `flat_map`, `or_else`, `filter`, `zip`, `unzip`, `transpose`) and participates in `?` propagation identically to `Result<T, E>`.

### 4.4 Error Handling in the Type System

`Result<T, E>` is the core error representation. **v2.0 additions**:

**Error set types** — inspired by Zig's error unions: a finite named set of possible error variants, exhaustively matchable, no heap allocation:
```omni
error set ParseErrors:
    InvalidSyntax(span: Span, message: String)
    UnexpectedEof(position: usize)
    InvalidEncoding

fn parse(input: &str) -> Result<Ast, ParseErrors>
```

**Typed error context chains** — the `|>` context operator wraps errors with context without erasing the underlying type:
```omni
let config = parse_config(path)? |> "while loading application config"
```

**Implicit error set widening** — when `?` propagates across a function boundary, error types are automatically widened when the relationship is a subset or conversion.

### 4.5 Generics

Monomorphization by default. **v2.0 additions**:

**Implied bounds** — when a struct is defined with a bound (`struct Cache<K: Hash>`) and a method is written on that struct, the bound is implied and does not need repeating in every method signature. This eliminates the where-clause copy-paste identified as one of Rust's ergonomic burdens.

**Variadic generics** — functions and types parameterized by arbitrary-length type tuples:
```omni
fn map_tuple<..Ts, ..Us>(t: (..Ts), f: (..Ts) -> (..Us)) -> (..Us)
```

**Limited specialization** — trait implementations can provide specialized versions for specific concrete types, with the general implementation as fallback.

### 4.6 Traits

Traits are the primary polymorphism mechanism. **v2.0 additions**:

**Trait upcasting** — `dyn SubTrait` coerces to `dyn SuperTrait` when `SubTrait: SuperTrait`. This was only recently stabilized in Rust 2024; Omni includes it from the start.

**Negative bounds** — `where T: !Copy` specifies that a type does not implement a trait. Enables API designs impossible with only positive bounds.

**Custom diagnostic attributes** — traits annotate with custom error messages when trait bounds are not satisfied:
```omni
@[diagnostic::on_unimplemented(
    message = "Type `{Self}` cannot be safely shared across threads",
    label = "add `Send + Sync` bounds or use `Mutex<T>`"
)]
trait ThreadSafe: Send + Sync
```

**Async traits (native)** — traits can contain `async fn` methods without boxing overhead. The compiler generates a concrete associated future type for each implementation.

### 4.7 Pattern Matching

Exhaustive. Expressive. Available in all expression positions. **v2.0 additions**:

**Or-patterns at all positions** — `(x | y) if cond` works at all nesting levels.

**Deconstructing function parameters**:
```omni
fn process((x, y): (i32, i32)) -> i32:
    x + y

fn handle(Point { lat, lon }: &Point) -> String:
    format("({lat}, {lon})")
```

**`let`-chains** — multiple pattern bindings chained with `and`:
```omni
if let Some(user) = get_user() and let Some(profile) = user.profile():
    display(profile)
```

### 4.8 Algebraic Data Types / Enums

Rich enums with payloads. Exhaustive matching mandatory. **v2.0 additions**:

**Sealed enums** — external crates cannot add new variants, enabling exhaustive matching without wildcards.

**Enum methods with field access** — direct method dispatch on enum variants without unwrapping.

### 4.9 Compile-Time Computation

`comptime` for compile-time evaluation of pure functions. **v2.0 additions**:

**Comptime string operations** — compile-time string manipulation for code generation and format-string validation.

**Comptime type reflection** — `comptime typeof(T)` returns structural type information as a comptime-evaluable value.

**Comptime budget annotations** — `@comptime_limit(ops: 1000000)` caps operations to prevent compilation-halting infinite loops.

### 4.10 Reflection

**Compile-time reflection (primary)**: Enumerate fields, methods, trait implementations, and type metadata at compile time with zero runtime cost.

**Limited runtime reflection (secondary, explicit import)**: Type names, debug formatting, dynamic dispatch only. `use std::reflect` required. Never Java-style full reflection.

---

## 5. MEMORY MODEL & OWNERSHIP

### 5.1 The Core Model: Ownership

Every value has exactly one owner. When the owner goes out of scope, the value is deterministically dropped. Ownership is transferred by moving. After a move, the original binding is inaccessible.

### 5.2 Borrowing

**Shared borrows (`&T`)**: Multiple can coexist; value cannot be mutated while any shared borrow is active.

**Exclusive borrows (`&mut T`)**: Exactly one can exist; no shared borrows coexist with it.

The borrow checker uses the **Polonius algorithm** from day one — more precise than NLL, eliminates false positives, adopted from the start to avoid shipping the weaker algorithm and then having users learn its workarounds. Implementation uses the `polonius-engine` crate (Apache 2.0) adapted for Omni's MIR.

### 5.3 Field Projections (v2.0)

The borrow checker tracks borrows at field granularity within a struct. Different fields can be independently borrowed simultaneously. This eliminates the pattern of splitting structs into sub-structs purely to enable independent field borrowing:

```omni
struct State:
    name: String
    config: Config
    cache: HashMap<String, Value>

fn update(state: &mut State):
    let name_ref = &state.name       -- shared borrow of `name` field
    let cache_ref = &mut state.cache -- exclusive borrow of `cache` field
    -- Both valid because they borrow different fields
```

### 5.4 Generational References (v2.0)

`Gen<T>` pairs a pointer with a generation counter. When an object is freed, its generation increments. Dereferencing a `Gen<T>` checks that the stored generation matches the current generation — O(1), detects use-after-free without lifetime tracking. This is memory-safe in safe code, no `unsafe` required:

```omni
struct Graph:
    nodes: Arena<Node>

struct Node:
    value: i32
    neighbors: Vec<Gen<Node>>  -- safe cyclic references

fn add_edge(graph: &mut Graph, from: Gen<Node>, to: Gen<Node>) -> Result<()>:
    let from_node = graph.nodes.get_mut(from)?  -- Returns Err if `from` was freed
    from_node.neighbors.push(to)
    Ok(())
```

Generational references are the recommended approach for graph algorithms, entity component systems, and any data structure where ownership cycles are structurally necessary. Runtime cost: a single integer comparison per dereference, eliminable by the region borrow checker in hot paths.

### 5.5 Linear Types (v2.0)

Rust uses **affine types** (used at most once). Omni additionally supports **linear types** (must be used exactly once — cannot be silently dropped):

```omni
linear struct DatabaseTransaction:
    connection_id: u64

impl DatabaseTransaction:
    fn commit(self) -> Result<(), DbError>
    fn rollback(self) -> Result<(), DbError>

-- Dropping a DatabaseTransaction without calling commit() or rollback() is a compile error.
```

Linear types model resources that must be explicitly released. They compose with ownership: a linear type is an owned type that additionally enforces usage. Use cases: file handles, network connections, transaction objects, cryptographic keys.

### 5.6 Inout Parameters (v2.0)

Syntactic sugar for the move-in/move-out pattern:
```omni
fn normalize(inout v: Vec<i32>):
    v = v.iter().map(|x| x * 2).collect()

let mut v = vec![1, 2, 3]
normalize(inout v)  -- v is updated; no rebinding needed
```

Compiles to move-in/move-out at the MIR level. Zero runtime overhead. Pure ergonomic improvement.

### 5.7 Arena Allocation

First-class arena allocators for bulk allocation and bulk deallocation:
```omni
let arena = Arena::new()
let node1 = arena.alloc(Node { value: 1 })
let node2 = arena.alloc(Node { value: 2 })
-- All freed when `arena` drops
```

The borrow checker enforces that arena-allocated object references don't outlive the arena.

### 5.8 Safe/Unsafe Boundary

`unsafe` blocks and `unsafe fn` enable raw pointer dereference, external unsafe function calls, accessing unsafe statics, and manual memory management. `@safe_wrapper` attribute signals that a function internally uses unsafe but guarantees safe-code contract to callers.

### 5.9 GC Compatibility Layer

Module-level `@gc_mode` annotation enables garbage collection for that module's allocations. GC-mode and ownership-mode objects have explicitly typed crossing points. GC uses a tracing collector with conservative stack scanning, tunable pause targets, and a write barrier integrated with the ownership system.

---

## 6. EFFECT SYSTEM

### 6.1 What an Effect System Is and Why Omni Has One

An effect system tracks what side effects a function may perform in addition to its types. A function's complete signature is `(Input) -> Output / Effects`. The compiler checks that callers handle every effect in a called function's signature.

Koka (Microsoft Research, actively developed 2025) demonstrates that algebraic effects and handlers let you define advanced control abstractions like async/await as a user library in a typed and composable way. This eliminates the need to design async, generators, exceptions, and cancellation separately. ICFP 2024 proceedings confirm algebraic effect handlers are both theoretically sound and practically implementable.

### 6.2 Built-In Effect Kinds

| Effect | Meaning |
|---|---|
| `io` | Reads/writes external state (filesystem, network, environment) |
| `async` | May suspend and resume (requires an executor) |
| `throw<E>` | May raise an exception of type `E` |
| `panic` | May panic (programming logic error) |
| `alloc` | May allocate heap memory |
| `rand` | May read from a random source |
| `time` | May read the current time |
| `log` | May produce log output |
| `pure` | No effects (empty effect set) |

A pure function has no effects and is safe to memoize, parallelize, or evaluate at compile time. Impure functions explicitly declare their effects.

### 6.3 Effect Inference

Effects are inferred by default in non-public code. The compiler propagates effects upward through the call graph automatically. Explicit effect annotations are required only when: defining a public API boundary, restricting a higher-order function to specific effects, or overriding an inferred effect for documentation:

```omni
-- Effect inferred (application code)
fn read_config(path: &str) -> Config:
    let text = std::fs::read_to_string(path)
    parse(text)

-- Effect explicit (public API)
pub fn fetch_user(id: u64) -> Result<User, ApiError> / io + throw<ApiError>:
    -- ...
```

### 6.4 Effect Handlers

Effects can be user-defined and handled with effect handlers:

```omni
-- Define a custom effect
effect Logging:
    fn log(msg: &str) -> ()

-- Use the effect
fn process_data(data: &[u8]) -> Result<Output> / Logging:
    Logging::log("Starting processing")
    let result = parse(data)
    Logging::log("Parsing complete")
    result

-- Install a handler
fn main():
    handle Logging:
        fn log(msg: &str):
            eprintln!("[INFO] {msg}")
    in:
        process_data(read_input())
```

Effect handlers compose: multiple effects can be handled at different call stack levels.

### 6.5 Async as an Effect (v2.0)

`async fn` is syntactic sugar for a function with the `async` effect. The executor is an effect handler, not a fixed language primitive. This means:
- Async and non-async code compose naturally through the effect system
- Different executors can be selected at different call sites
- Async cancellation is explicit via `CancelToken` (not implicit future-drop)

```omni
-- Explicit cancellation
async fn fetch_with_timeout(url: &str, timeout: Duration) -> Result<Response> / async + io:
    let (cancel_token, cancel_source) = CancelToken::new()
    let timer_task = spawn async:
        sleep(timeout).await
        cancel_source.cancel()
    let result = fetch(url).with_cancel(cancel_token).await?
    timer_task.abort()
    result
```

### 6.6 Generators as an Effect (v2.0)

```omni
fn fibonacci() -> Gen<i64>:
    let (a, b) = (0i64, 1i64)
    loop:
        yield a
        (a, b) = (b, a + b)

for n in fibonacci().take(10):
    println(n)
```

Generator functions compile to state machines with no heap allocation.

### 6.7 Effect Polymorphism

Functions can be polymorphic over effects, preserving the caller's effects automatically:
```omni
fn map<T, U, e>(items: &[T], f: (T) -> U / e) -> Vec<U> / e:
    items.iter().map(f).collect()

-- Caller's `io` effect propagates through map
fn process_files(paths: &[str]) -> Vec<Content> / io:
    map(paths, |p| std::fs::read_to_string(p))
```

---

## 7. CONCURRENCY & EXECUTION MODEL

### 7.1 The Hybrid Concurrency Model

Threads, structured async (via the effect system), message-passing via channels, and the actor model. Safe-code default prefers message-passing and ownership-based sharing over shared mutable state.

### 7.2 Structured Concurrency (v2.0, Hard Constraint)

No concurrent task can outlive the scope that created it. This is enforced by the type system:

```omni
async fn fetch_all(urls: &[str]) -> Vec<Result<Response>> / async:
    spawn_scope |scope|:
        urls.iter()
            .map(|url| scope.spawn(async: fetch(url).await))
            .collect::<JoinAll<_>>()
            .await
    -- scope dropped here; all tasks guaranteed complete
```

`spawn_scope` cancels all tasks and propagates errors if the scope exits early. It can never return before all tasks complete. Eliminates the most common class of async resource leaks.

**Unstructured concurrency** via `spawn_global` requires an explicit `GlobalSpawnCap` capability to prevent accidental use.

### 7.3 Explicit Async Cancellation (v2.0)

Async Rust's implicit cancellation (future-dropped = cancelled) is one of its most documented sources of subtle bugs. Omni makes cancellation explicit:

```omni
async fn download(url: &str, cancel: &CancelToken) -> Result<Bytes> / async + io:
    let response = http::get(url).await?
    cancel.check()?   -- explicit cancellation point
    let body = response.body().await?
    Ok(body)
```

### 7.4 Threads

OS threads via the standard library. `Send`/`Sync` marker traits enforced by the compiler. Compile-time data race prevention for all safe code.

### 7.5 Actors

Actor model with typed message channels and supervision trees. Actors are isolated: internal state never directly accessible from outside. Messages handled sequentially within an actor.

### 7.6 Determinism

Deterministic execution by default in development mode (fixed scheduler order, reproducible). Non-deterministic inputs (IO, time, randomness) are modeled as effects, so programs that depend on them declare their non-determinism explicitly in their types.

### 7.7 Execution Modes

- **Development**: Deterministic scheduler, full debug info, verbose diagnostics, replay debugging
- **Standard**: Balanced optimization, reasonable compile times
- **Release**: Maximum optimization, profile-guided, nondeterminism allowed where performance requires

---

## 8. SYNTAX & SURFACE DESIGN

### 8.1 Block Structure

Indentation-based blocks by default. Brace-delimited blocks in advanced modes. The layout engine handles indented continuations consistently, documented with a complete grammar rather than heuristics.

### 8.2 Semicolons

Newline-first syntax. A newline terminates a statement unless the next line's first token is a continuation token (binary operator, `.`, `?`, `)`, `]`, `}`). Semicolons used only where the parser requires disambiguation.

### 8.3 Expression Orientation

Expression-oriented by default. `if`, `match`, `loop`, `block`, and `try` all produce values. The `discard` keyword explicitly drops a non-`()` expression result.

### 8.4 Effect Annotations in Signatures (v2.0)

Effect annotations appear after the return type, separated by `/`:
```omni
fn read_line() -> Result<String, IoError> / io
async fn fetch(url: &str) -> Result<Response> / async + io
fn pure_compute(x: i32, y: i32) -> i32          -- inferred pure; no annotation needed
```

### 8.5 Naming Conventions

Enforced by formatter and linter, not compiler semantics. `PascalCase` for types, traits, effects. `snake_case` for values, functions, modules. `SCREAMING_SNAKE_CASE` for constants.

### 8.6 Comments and Documentation

```omni
-- line comment
--- multi-line comment ---
/// doc comment (attached to following item)
```

Doc comments: Markdown, executable `omni`-code examples, internationalization of doc text supported.

### 8.7 Annotations and Attributes

`@[attribute]` block form or `@attribute` inline. Custom diagnostic attributes (§4.6) for traits. `@[repr(c)]` for C ABI compatibility. `@safe_wrapper` for unsafe internals with safe external contract.

### 8.8 Operator Overloading

Via trait implementations, standard operator set. The `Try` trait is user-definable, enabling `?` propagation for custom result-like types beyond `Result` and `Option`.

### 8.9 Async Closures (v2.0)

First-class async closures implementing `AsyncFn`, `AsyncFnMut`, `AsyncFnOnce`:
```omni
let fetch_and_process = async |url: &str| -> Result<Data>:
    let response = http::get(url).await?
    process(response.body().await?)
```

### 8.10 String Interpolation

`f"Hello {name}!"` — inline interpolation of any expression implementing `Display`. `d"..."` uses `Debug` formatting. The interpolated expression is evaluated eagerly and type-checked.

### 8.11 Macro System

**Declarative macros**: Pattern-based, hygienic, operating on token streams. No special permissions.

**Procedural macros**: Compiled separately, run as sandboxed processes. Defined API surface; cannot access internal compiler structures.

**Comptime code generation (v2.0)**: Comptime expressions that produce code directly, for cases where generated code is determined by type-level computation.

---

## 9. MODULE, PACKAGE & VISIBILITY SYSTEM

### 9.1 Module Hierarchy

Files → Modules → Packages → Workspaces. A workspace groups packages with a shared lockfile.

### 9.2 Visibility Levels

| Modifier | Meaning |
|---|---|
| (none) | Private to current module |
| `pub(mod)` | Current module and children |
| `pub(pkg)` | Within the package |
| `pub` | Publicly accessible |
| `pub(cap: X)` | Requires capability X |
| `pub(friend: other::module)` | Specific named module via capability grant |

### 9.3 Import System

Explicit imports. Glob imports warned in strict mode. Scoped imports that expire at block end:
```omni
fn process():
    use std::collections::{HashMap, HashSet} in:
        let map: HashMap<str, i32> = HashMap::new()
```

### 9.4 Package Manifest (`omni.toml`)

TOML format. Declares name, version, edition, dependencies, build targets, features, and **capability declarations** (v2.0 — visible to users before installation):
```toml
[capabilities]
network = ["read"]
filesystem = ["read", "write", "/tmp"]
subprocess = false
```

### 9.5 Dependency Resolution

PubGrub algorithm with lockfiles. v2.0 addition: **automatic API compatibility checking** — publishing a new version that breaks the public API without incrementing the major version is rejected by the registry.

### 9.6 Build System

Built-in extensible build system. v2.0 addition: **comptime build scripts** — build logic written in Omni using comptime functions, evaluated at build time (inspired by Zig's comptime build system):

```omni
-- build.omni (build script, evaluated at build time)
comptime fn configure_build(config: &BuildConfig):
    if config.target.is_wasm():
        config.add_feature("no_threads")
        config.set_opt_level(OptLevel::Size)
    config.add_link_lib("ssl")
```

---

## 10. ERROR HANDLING & FAILURE MODEL

### 10.1 Errors as Values

`Result<T, E>` is the fundamental error mechanism. Errors are values. The compiler warns when a `Result` is ignored. The compiler also warns when a `Result` is propagated through `?` across a function boundary that doesn't declare the `throw<E>` effect, making effect tracking consistent with error propagation.

### 10.2 Error Types

Structured, typed, rich. Each error type implements the `Error` trait. Error set types (§4.4) for finite, exhaustively matchable errors. Typed context chains via `|>`. Machine-readable error codes (E####) on all user-facing errors.

### 10.3 Propagation Operator

The `?` operator is polymorphic via the `Try` trait. Works on `Result<T,E>`, `Option<T>`, error sets, and any type implementing `Try`.

### 10.4 Panics

Panics represent logic invariant violations. In development mode: always unwind with full stack trace and debugger attachment option. Structured metadata (location, message, optional structured payload) capturable by a panic hook.

### 10.5 Resource Cleanup

Resources dropped in reverse initialization order at scope exit, in both normal and panic unwinds. Drops are synchronous. **Async drop (v2.0)**: types can implement async destruction for resources requiring async cleanup (e.g., network connection with protocol shutdown). Async drops run in a cleanup executor before the parent scope proceeds.

---

## 11. STANDARD LIBRARY ARCHITECTURE

### 11.1 Layered Design

`std::core` (no OS, no heap), `std::alloc` (heap, no OS), `std` (full OS). All IO functions in `std` declare the `io` effect.

### 11.2 Core Traits

`Copy`, `Clone`, `Drop`, `Eq`, `PartialEq`, `Ord`, `PartialOrd`, `Hash`, `Display`, `Debug`, `Default`, `Iterator`, `From`, `Into`, `TryFrom`, `TryInto`, `Error`, `Send`, `Sync`, `Try` (v2.0 — extensible `?` propagation).

### 11.3 Collections

`Vec<T>`, `HashMap<K,V>`, `HashSet<T>`, `BTreeMap<K,V>`, `BTreeSet<T>`, `VecDeque<T>`. v2.0 additions: `Arena<T>` — arena allocator. `Gen<T>` — generational reference. `SlotMap<T>` — dense mapping with stable handles (common in game engines and graph libraries).

### 11.4 IO Model

`Read`/`Write` traits. Async `AsyncRead`/`AsyncWrite`. All IO functions are capability-gated: accessing the filesystem requires a `FilesystemCap` token. IO is modeled as an effect.

### 11.5 Serialization

Pluggable system with backends for JSON, TOML, YAML, CBOR, MessagePack, and custom binary formats. Compile-time format validation ensures all fields of a `Serialize` type are serializable.

### 11.6 AI and Tensor Support (v2.0 — Required by HELIOS)

Informed by Mojo's MLIR-based approach for portable hardware acceleration:

```omni
use std::tensor::{Tensor, Shape, DType}
use std::simd::{f32x8, auto_vectorize}

-- Statically-typed tensor with compile-time shape verification
fn dot_product(a: Tensor<f32, Shape<N>>, b: Tensor<f32, Shape<N>>) -> f32:
    (a * b).sum()

-- Explicit SIMD for performance-critical paths
@[auto_vectorize]
fn scale(data: &mut [f32], factor: f32):
    for chunk in data.chunks_exact_mut(8):
        let v = f32x8::from_slice(chunk)
        (v * f32x8::splat(factor)).copy_to_slice(chunk)
```

The tensor module provides `Tensor<T, Shape>` with compile-time shape checking, SIMD dispatch for auto-vectorization, and a hardware abstraction layer for GPU dispatch (via MLIR, future phase). This is HELIOS's foundation for inference and embedding computation.

### 11.7 Time and Scheduling

Full time system: monotonic clock, wall clock, timers, scheduled tasks. Time-reading functions declare the `time` effect.

### 11.8 Cryptography

Safe high-level primitives (AEAD encryption, authenticated key exchange, password hashing) with correct defaults. Lower-level API for experts. Cryptographic key access is capability-gated.

---

## 12. COMPILATION MODEL & IR DESIGN

### 12.1 Compiler Pipeline

```
Source Text
    ↓ [Lexer + Indentation Layout Engine]
Token Stream (INDENT/DEDENT synthetic tokens)
    ↓ [Parallel Parser — Recursive Descent + Pratt, independent files parsed concurrently]
CST (Concrete Syntax Tree, lossless — all whitespace and comments preserved)
    ↓ [AST Lowering]
AST (Abstract Syntax Tree, structured)
    ↓ [Effect Resolution]
Effect-annotated AST (all effects inferred or verified)
    ↓ [Name Resolution — two-pass]
Resolved AST (all names bound to DefIds)
    ↓ [Type Inference — bidirectional]
Type+Effect-annotated AST
    ↓ [Type + Effect Checking]
Verified AST (type-correct, effect-correct, trait bounds satisfied)
    ↓ [MIR Lowering]
MIR (Mid-level IR — CFG, ownership explicit, drops inserted)
    ↓ [Borrow Checker — Polonius algorithm]
Verified MIR (memory safety proved)
    ↓ [MIR Optimization]
Optimized MIR
    ↓ [LIR Lowering]
LIR (Low-level IR — target-closer, no ownership concepts)
    ↓ [Codegen — Cranelift (dev) / LLVM (release) / MLIR (AI targets)]
Target-specific output
    ↓ [Linker]
Final binary or library
```

### 12.2 The IR Stack

**CST**: Lossless. Rowan-based. Used by formatter and incremental parser. Preserves all source information.

**AST**: Structural. Spans preserved. Arena-allocated.

**Effect-annotated AST (v2.0)**: A separate pass resolves and verifies all effect annotations before type inference. Type errors and effect errors are diagnosed independently for cleaner messages.

**MIR**: Control-flow-graph based. Ownership, borrows, drops explicit. **Polonius algorithm** for borrow checking — more precise than NLL, fewer false positives, adopted from day one.

**LIR**: Target-specific. Ownership resolved; all drops inserted as explicit calls.

### 12.3 Borrow Checker: Polonius Algorithm

Polonius treats the borrow checker as a Datalog program. It is complete (never rejects correct programs that NLL would reject due to precision limits) and maintains identical soundness guarantees. The `polonius-engine` crate (Apache 2.0) is adapted for Omni's MIR representation.

### 12.4 Backend Targets

**Development default**: Cranelift — fast compile times, correct native code. **Release default**: LLVM via `inkwell` — maximum optimization. **AI/hardware targets (v2.0, Phase 13)**: MLIR — GPU dispatch, TPU targeting, hardware-specific accelerator backends.

### 12.5 Incremental Compilation

Salsa-inspired query-based model. Every compiler pass is a query; queries cache results and invalidate only affected downstream queries when inputs change. Sub-second LSP response times for large projects.

### 12.6 Parallel Front End (v2.0)

The lexer and parser pipeline is parallelized: independent files are parsed on separate threads simultaneously. Safe because parsing is purely functional. Reduces build times proportionally to available CPU cores on large projects.

### 12.7 Automatic Applied Fixes (v2.0)

Every compiler diagnostic for which an unambiguous fix exists emits a machine-applicable fix operation. `omni fix` reads these and applies them automatically. This follows the `cargo fix` model from Rust's edition migration system.

### 12.8 Relink Without Rebuild (v2.0)

When only a library implementation changes without changing the ABI, the binary relinks without recompiling dependent crates. Requires a stable internal ABI for Omni library types (introduced Phase 6, strengthened Phase 12).

---

## 13. RUNTIME ARCHITECTURE

### 13.1 Default: AOT-First

Native AOT compilation produces standalone binaries. Correct, fast, deterministic. No runtime dependency except the system C library.

### 13.2 Modular Runtime

Modules: core, memory, async, threading, capability, plugin, tensor (v2.0). A minimal embedded program links only the core runtime.

### 13.3 Async Executor: Structured Concurrency Enforced

Work-stealing multi-threaded executor. The executor enforces the parent-task/child-task lifetime relationship at runtime. Attempting to spawn an unstructured task requires `spawn_global` and `GlobalSpawnCap`.

### 13.4 JIT Strategy

Profile-guided JIT (Phase 9), tiered adaptive JIT (Phase 10). JIT selectively replaces AOT-compiled hot paths with optimized JIT-compiled versions.

### 13.5 MLIR Integration (v2.0, Phase 13)

MLIR-compiled functions run through MLIR's LLVM backend or directly on GPU/TPU targets. Accessed through the tensor API (§11.6), not direct MLIR APIs. Provides the hardware portability that Mojo demonstrated with its MLIR foundation.

### 13.6 Binary Strategy

Static by default. Dynamic linking available as explicit option. Relink-without-rebuild for ABI-compatible library updates.

---

## 14. TOOLING & DEVELOPER EXPERIENCE

### 14.1 Official Toolchain

Unified `omni` CLI: `new`, `build`, `run`, `check`, `test`, `bench`, `fmt`, `lint`, `doc`, `clean`, `add`, `remove`, `update`, `publish`, `profile`, `debug`, `fix` (v2.0), `verify`, `semver-check` (v2.0), `migrate --edition` (v2.0).

### 14.2 Formatter (`omni fmt`)

Idempotent, deterministic. CST-based (preserves comments). Default style; minimal configuration. In strict mode: sorts imports, aligns doc comments, applies structural normalization. `--check` mode for CI.

### 14.3 Language Server (`omni-lsp`)

LSP-compliant. Query-based incremental compiler powers sub-second response times. v2.0 additions:

**Enhanced inlay hints**: Inferred types, effect annotations, and field types displayed inline (configurable verbosity).

**Effect explorer**: Hover over any function call to see its complete effect set including transitively inferred effects.

**Borrow checker visualization**: A dedicated view showing the borrow region graph for a function — which borrows are live at each point in the control flow.

**Semantic highlighting**: Effect-annotated expressions, unsafe blocks, linear types, and generational references highlighted distinctly.

### 14.4 Compiler Diagnostics Quality

Every diagnostic meets the **Elm-inspired standard** (Elm and Rust have the two best diagnostic systems as of current research):
- A stable error code (E####)
- A primary span at the exact source location
- A clear, non-jargon message
- Secondary spans with related context
- A help note suggesting how to fix the problem
- A machine-applicable fix when unambiguous

v2.0 additions:
- **Diagnostic translations**: Error messages translatable via external translation files; internationalization-ready from the start
- **Contextual "Did you mean?" suggestions**: Levenshtein distance for undefined identifiers, covering types, functions, traits, and module paths
- **Effect error messages**: When an effect is used without a handler, the error explains which effect is missing, where it originates in the call chain, and how to add the appropriate handler
- **JSON error output**: Machine-readable diagnostics for tooling integration

### 14.5 Debugger

DAP-compliant. DWARF debug info in development builds. v2.0 addition: **Replay debugging** — development builds record execution traces and replay them with perfect determinism. Since non-deterministic inputs are declared as effects (§6), the effect handler infrastructure intercepts and records them, enabling perfect replay.

### 14.6 Documentation Generator

Generates HTML from doc comments. Executable examples as tests (`omni doc --test`). v2.0 additions:
- **Effect documentation**: Functions' effect sets displayed prominently; searchable by effect
- **Versioned docs**: Registry hosts docs for all published versions with API diff-view

---

## 15. TESTING, DIAGNOSTICS & VALIDATION

### 15.1 Test Framework

`@test`, `@test_should_panic`, `@test_ignore`. Parallel execution. JUnit XML output.

v2.0 addition: `@effect_test` — tests run in a controlled effect environment where effects like `io` and `time` are replaced by mocks:
```omni
@effect_test
fn test_data_processing():
    with MockIo::file("/data.csv", "1,2,3"):
        let result = process_data()
        assert_eq(result.sum, 6)
```

### 15.2 Property-Based Testing

Built-in shrinking property-based framework. v2.0: Effect-aware property tests that can model stateful properties by controlling effect handlers.

### 15.3 Contract Annotations (v2.0)

Lightweight contracts, checked at compile time (when statically provable) or at runtime in debug builds:

```omni
@requires(n > 0, "n must be positive")
@ensures(result > 0, "result must be positive")
fn compute_positive(n: i32) -> i32:
    n * n

@invariant(self.len <= self.capacity, "length cannot exceed capacity")
struct Buffer:
    data: Vec<u8>
    capacity: usize
    len: usize
```

Contracts are: zero-cost when statically proved, checked at runtime in debug mode, stripped in release (configurable), and visible in generated documentation as formal preconditions/postconditions.

### 15.4 Fuzzing

Official fuzzing via `cargo-fuzz`/libfuzzer. Coverage-aware. v2.0: Fuzz corpus version-controlled alongside tests. CI runs fuzzing for a fixed time budget on every PR touching parsing, serialization, or security-sensitive paths.

### 15.5 Benchmarking

Statistical framework with regression tracking. v2.0 additions: `@assert_alloc_count(max: 0)` verifies no heap allocations; `@assert_compile_time_only` verifies full compile-time evaluation.

---

## 16. SECURITY, SAFETY & CAPABILITY SYSTEM

### 16.1 Security as a Layered System

Language (memory safety, no UB), types (capability types, effect annotations), runtime (capability tokens, sandbox enforcement), tooling (package signing, verified builds, audit logging), ecosystem (transparency log, revocable capabilities).

### 16.2 Capability-Based Access Control

Capabilities are unforgeable tokens granting access to specific resources. Created only by the runtime at program startup. Passed explicitly. Delegatable. Revocable.

v2.0: **Capability-effect alignment** — capabilities and effects are unified. Having the `io` capability token enables the `io` effect. The capability system and the effect system are two faces of the same mechanism.

```omni
fn write_log(cap: &IoCapability, msg: &str) / io:
    std::fs::write("/var/log/app.log", msg)?
```

### 16.3 Sandboxing

Sandboxed execution for plugins and untrusted code. Capability violations produce `CapabilityError`.

v2.0: **Fearless FFI sandboxing** — FFI calls run in an isolated execution context using stack switching. Memory corruption in an FFI call cannot propagate to Omni's managed memory because the two execution stacks are separate. This eliminates heap corruption from C code spreading into Omni code.

### 16.4 Package Security

All published packages signed. Transparency log. `omni verify` checks all packages. CLI permission flags for runtime capability grants.

v2.0: **Supply chain verification** — the package manager verifies that a package's published binary matches what building from source would produce (reproducible build verification, same model as the self-hosting bootstrap trust chain).

### 16.5 Memory Safety Layers

Safe code: complete memory safety guaranteed by the compiler. Generational references: memory-safe for cyclic data without `unsafe`. Linear types: resource safety enforced by type system. `unsafe` blocks: explicit risk declaration, auditable. GC mode: safety guaranteed by the collector.

---

## 17. INTEROPERABILITY & FFI

### 17.1 C FFI with Fearless FFI Sandboxing (v2.0)

All C FFI calls are `unsafe`. With `--fearless-ffi` build flag, FFI calls run in an isolated stack context preventing memory corruption spread:

```omni
@extern_c
fn strlen(s: *const u8) -> usize

pub fn str_len(s: &str) -> usize:
    unsafe: strlen(s.as_ptr())
```

### 17.2 Bindgen Tool

`omni bindgen` reads C headers and generates safe Omni wrappers. v2.0: Generates ownership annotations (which functions take vs. borrow pointers) based on naming convention heuristics and manual override annotations.

### 17.3 Phased Interoperability

1. C FFI with Fearless FFI (Phase 3)
2. WebAssembly export (Phase 4)
3. Python binding auto-generation (Phase 6)
4. JVM via JNI (Phase 8)
5. MLIR dialect for GPU/hardware (Phase 13)

### 17.4 ABI Stability (v2.0)

Omni defines a stable C-compatible ABI for exported types (`@[repr(c)]`) and a separate versioned "Omni ABI" for Omni-to-Omni interoperability. ABI compatibility checking is built into the package manager.

---

## 18. BOOTSTRAP STRATEGY & SELF-HOSTING ROADMAP

### 18.1 The Bootstrap Language: Rust

Rust is correct: philosophy alignment, mature tooling (`polonius-engine`, `cranelift`, `tower-lsp`, `rowan`), type safety catches bootstrap bugs at compile time. The Rust 2024 edition provides additional ergonomic improvements (async closures, precise capturing, improved lifetime elision) that make bootstrap development faster.

### 18.2 Multi-Stage Bootstrap Pipeline

Stage 0: Rust-written compiler (trusted baseline). Stage 1: Rust compiles partial Omni compiler (Omni-written). Stage 2: Stage 1 compiles full Omni compiler. Stage 3: Stage 1 == Stage 2 binary verification. Stage 4: Rust bootstrap retired to validation-only role.

### 18.3 Trust Model

Reproducible build verification. Any deviation between Stage 1 and Stage 2 outputs is a critical bug blocking release. CI verifies on every compiler commit.

### 18.4 Module-by-Module Migration

Migration order: Lexer → Parser → AST → Name Resolver → Type Inference → Type Checker → Effect Resolver → MIR Lowering → Borrow Checker (Polonius) → Optimizer → Codegen → Standard Library.

Both Rust and Omni implementations must produce identical outputs on all test inputs during transition. Output diff must be zero.

---

## 19. PHASED IMPLEMENTATION PLAN

The core principle: each phase produces a working, testable system. No phase adds complexity on top of an incomplete foundation.

### Phase 0: Project Foundation
Create the governance, repository, and CI scaffolding. All contributors can clone, build, and run tests in one command. No language features yet.

**Key deliverables**: Cargo workspace (all crates, even stubs), CI (fmt, clippy, test, docs, security), devcontainer, CONTRIBUTING, ROADMAP, ADR-0001 (Rust bootstrap), ADR-0002 (workspace structure).

**Acceptance criteria**: `cargo build --workspace` green. CI green on push. New contributor productive in 30 minutes.

---

### Phase 1: Language Core Skeleton
Programs can be parsed into a typed, richly annotated AST with useful diagnostics.

**Key deliverables**: Lexer (full token set, INDENT/DEDENT layout engine, string interpolation scaffolding, fuzz target), Parser (recursive descent + Pratt, panic-mode recovery, UI test harness, parallel multi-file), Diagnostics system (stable error codes, JSON output, machine-applicable fix encoding, "Did you mean?" foundation), CLI with `parse`, `fmt` stub, `fix` stub.

**Acceptance criteria**: `omni parse hello.omni` prints valid AST. Invalid syntax produces useful diagnostics. Formatter round-trips. Fuzz target runs 60 seconds without panics. JSON error output parseable. 20+ UI tests pass.

---

### Phase 2: Semantic Core and Type Checking
The compiler understands meaning. Name resolution, bidirectional type inference, effect inference, and basic type checking enforced. Programs execute via a minimal interpreter.

**Key deliverables**: Name resolver (two-pass, scope tree, DefId system, use declarations, implied bounds), Type system (bidirectional inference, effect set representation, basic effect inference), Type checker (unification, trait bound checking, basic effect handler checking), Basic effect kinds (io, async, panic, pure — built-in only), Minimal interpreter for testing, Integrated pipeline.

**Acceptance criteria**: Hello world, fizzbuzz, recursive fibonacci execute. Type errors produce diagnostics with spans. Basic effects inferred correctly. "Did you mean?" suggestions appear. 30+ UI tests pass.

---

### Phase 3: Ownership, Borrowing, and Safety Core
Core memory safety enforced using Polonius. Generational references and linear types available.

**Key deliverables**: MIR definition and AST→MIR lowering, CFG construction and liveness analysis, Polonius-based borrow checker (via `polonius-engine` adapted for Omni MIR), Field projection support, Generational references (`Gen<T>`) and arena allocator (`Arena<T>`), Linear type annotations and usage enforcement, `inout` parameter desugaring, Drop insertion, `unsafe` tracking, Fearless FFI sandbox skeleton.

**Acceptance criteria**: Use-after-move caught with diagnostics. Conflicting borrows caught. Field projections enable independent field borrows. Generational references catch use-after-free in debug mode. Linear types prevent dropped-without-use and double-use. 40+ UI tests pass. All Phase 2 programs still compile.

---

### Phase 4: Modules, Packages, and Build System
Multi-file, multi-package projects compile reproducibly. Comptime build scripts work.

**Key deliverables**: Hierarchical module system (file modules, inline modules, all visibility levels), `omni.toml` with capability declarations, `omni.lock` lockfile, PubGrub resolver, Build graph and incremental compilation, Monorepo workspace, Comptime build scripts (`build.omni`).

**Acceptance criteria**: 3-package project compiles. Lockfile deterministic. Comptime build script conditionally configures a build. Module privacy enforced. Capability declarations visible.

---

### Phase 5: Standard Library Core
Vec, HashMap, String, Result, Option, IO, tensor foundation — implemented, tested, documented.

**Key deliverables**: All core traits (including `Try`), Collections (Vec, HashMap, HashSet, String, Arena, Gen, SlotMap), IO traits with capability-gating and `io` effect annotation, Option and Result with `Try` integration, Error set types, Math primitives, Tensor module foundation (Tensor<T, Shape>, SIMD dispatch stubs).

**Acceptance criteria**: All stdlib types tested. `omni doc --test` passes. No `unwrap()` in library code. Tensor module compiles and runs basic operations. All IO functions correctly declare the `io` effect.

---

### Phase 6: Tooling and Developer Experience
Full development workflow from CLI alone.

**Key deliverables**: `omni-fmt` (CST-based, idempotent, effect annotation formatting), `omni-lsp` (diagnostics, go-to-def, hover with effect info, completion, inlay hints, borrow visualization, semantic highlighting), Test runner (parallel, JUnit, doc tests, `@effect_test`), Full CLI (`fix`, `verify`, `semver-check`), VS Code extension, `omni doc` with effect documentation and versioned docs.

**Acceptance criteria**: All CLI commands work. Formatter idempotent (property test in CI). LSP provides completions, go-to-def, and effect hover. `omni fix` applies at least 10 common automatically-fixable errors. Effect documentation visible in `omni doc` output.

---

### Phase 7: Advanced Type System
Generics, traits, pattern matching, macros, comptime, variadic generics, specialization.

**Key deliverables**: Generic functions/structs/enums with monomorphization and implied bounds, Trait definitions with async traits, trait upcasting, negative bounds, custom diagnostic attributes, Exhaustive pattern matching with usefulness algorithm, or-patterns, deconstructing parameters, let-chains, `comptime` with budget annotations and type reflection, Declarative and procedural macros (sandboxed), Variadic generics (basic form).

**Acceptance criteria**: Generic containers work with all element types. Async traits work without boxing. Trait upcasting works. Non-exhaustive matches rejected. Custom diagnostic attributes produce custom messages. Variadic tuples work in basic cases. 60+ UI tests pass.

---

### Phase 8: Effect System (Full Implementation)
Algebraic effects as a first-class language feature. User-defined effects. Effect polymorphism. Structured concurrency enforced.

**Key deliverables**: Full effect handler syntax and semantics, User-defined effect kinds, Effect polymorphism in generics, Structured concurrency (`spawn_scope`, enforced lifetime), Explicit cancellation tokens, Async closures (`AsyncFn` traits), Generator effects (`Gen<T>` as lazy sequence), Async drop.

**Acceptance criteria**: Custom effects can be defined and handled. Effect polymorphism works. Structured concurrency enforces task lifetime. Unstructured spawn requires `GlobalSpawnCap`. Async closures work in higher-order functions. Generators produce lazily. Async drop works. 40+ effect and concurrency tests pass.

---

### Phase 9: Concurrency Runtime and Tensor Acceleration
Production-grade concurrent/parallel execution with tensor acceleration.

**Key deliverables**: Work-stealing structured concurrency executor, Replay debugging infrastructure, Actor model with supervisor trees, Typed channels (MPSC, bounded, broadcast), Deterministic execution mode, SIMD dispatch in tensor module (auto-vectorization), SlotMap and Arena performance optimization.

**Acceptance criteria**: Concurrent programs execute correctly. Replay debugging works for simple concurrent programs. Actor ping-pong works. SIMD-accelerated tensor operations measurably faster than scalar equivalents. Deterministic mode produces identical output for same inputs.

---

### Phase 10: Security, Sandboxing, and Fearless FFI
Untrusted code cannot exceed granted capabilities. FFI is sandboxed. Package supply chain is verified.

**Key deliverables**: Full capability type system with effect-capability alignment, Fearless FFI sandboxing (isolated stack), Sandboxed plugin execution with revocable capabilities, Package signing, verification, supply chain verification, CLI permission flags, Audit logging.

**Acceptance criteria**: Plugin without `--allow-fs` cannot read files. FFI memory corruption does not spread to Omni memory (verified by test). Package verification catches tampered packages. Supply chain verification works.

---

### Phase 11: Interoperability Expansion
C FFI mature, WebAssembly working, Python bindings generating.

**Key deliverables**: C FFI with `omni bindgen` (ownership annotations), WebAssembly backend, Python binding auto-generation (`omni bindgen --python`), ABI stability documentation and versioning, ABI compatibility checks in package manager.

**Acceptance criteria**: C interop tests pass on Linux and macOS. WebAssembly output runs in Node.js and browser. Python bindings work for a simple type hierarchy. ABI compatibility checks catch breaking ABI changes.

---

### Phase 12: Self-Hosting Migration
Omni compiler progressively replaces Rust implementation.

**Key deliverables**: Module-by-module rewrite in Omni, Dual-compiler CI validation, Bootstrap trust verification (Stage 1 == Stage 2), Standard library migration.

**Acceptance criteria**: Omni compiler passes all test suites when compiled by itself. Stage 1 == Stage 2 binary comparison passes. Rust bootstrap retained as fallback only.

---

### Phase 13: Platform Maturity and MLIR Integration (v2.0 Addition)
Stabilize Omni as a production-grade platform. Full MLIR integration for AI acceleration.

**Key deliverables**: Edition system with `omni migrate --edition`, RFC process, Performance regression monitoring in CI, Long-term compatibility policy, MLIR backend (GPU dispatch through tensor API), Hardware abstraction layer for AI accelerators.

**Acceptance criteria**: Edition migration works on real code. CI catches >5% performance regressions. GPU tensor operations produce correct results. MLIR compilation pipeline executes on at least one GPU target.

---

## 20. HELIOS FRAMEWORK (PLATFORM LAYER)

### 20.1 What HELIOS Is

HELIOS is the first major platform built on Omni. An **advanced cognitive platform** for building AI-backed systems with structured knowledge management, capability-based security, multi-modal input processing, and autonomous reasoning support. HELIOS is not an AI model; it is a platform for orchestrating knowledge, reasoning, and execution in a controlled, auditable, and extensible way.

### 20.2 Relationship to Omni

HELIOS depends on and validates: the capability system (§16), async runtime with structured concurrency (§7), the effect system (§6), the tensor module (§11.6), and the plugin system (§16.3). HELIOS development begins in earnest after Phase 7 and scales as Phases 8-10 complete.

### 20.3 Seven Non-Negotiable Requirements

1. **Provenance-preserving knowledge storage** — every entry carries source, timestamp, confidence, author permanently.
2. **Immutable historical record** — updates create versions; deletions are soft; contradictions are explicit records.
3. **Structured confidence model** — every entry has a confidence score; decays for time-sensitive facts; influences retrieval ranking.
4. **Capability-gated access** — all HELIOS capabilities gated by the Omni capability system; no unrestricted access.
5. **Explainable reasoning** — all conclusions traceable to stored knowledge and reasoning steps; chains stored not just outputs.
6. **Layered plugin architecture** — HELIOS is extensible by signed, sandboxed, capability-declared plugins.
7. **Offline-first, local-primary operation** — operates without network access; cloud sync is optional.

### 20.4 HELIOS and the Omni Effect System

HELIOS defines its own effects aligned with its knowledge model:
```omni
effect KnowledgeStore:
    fn query(q: &Query) -> Vec<KnowledgeEntry> / KnowledgeStore
    fn insert(e: KnowledgeEntry) -> KnowledgeId / KnowledgeStore + io
    fn update(id: KnowledgeId, delta: Delta) -> Result<(), ConflictError> / KnowledgeStore + io

effect ReasoningEngine:
    fn infer(context: &Context) -> Vec<Hypothesis> / ReasoningEngine + KnowledgeStore
```

This makes HELIOS functions' dependencies on knowledge storage and reasoning explicit in their type signatures, enabling testing with mock effect handlers and deterministic replay of knowledge operations.

---

## 21. CURRENT STATE & WHAT REMAINS

### 21.1 What Has Been Built

Based on examination of the repository (`github.com/shreyashjagtap157/Helios`):

**Partial (structural work exists, needs completion)**: Project structure and Cargo workspace, bootstrap scaffolding and build scripts, partial stdlib fragments (`std/iter.omni`, logging), LSP and VS Code extension (architecturally correct, premature without a working compiler), mini-compiler demonstrating intent (not a complete pipeline), HELIOS runtime experiments and capability system scaffolding.

**Missing (critical path blockers)**: Complete, production-grade lexer; complete parser with error recovery and INDENT/DEDENT handling; semantic analysis (name resolution, type inference, type checking); MIR representation and borrow checker; IR-to-codegen pipeline producing executable output; package manager with working dependency resolution; verified end-to-end pipeline: source file → binary → execution.

### 21.2 The Honest Assessment

The repository has over-invested in future layers before the core compiler pipeline is stable. The LSP was built before the language server has a stable language to serve. The HELIOS capability system was designed before the Omni type system that will enforce it exists. This demonstrates clear vision of the end state, but requires deliberate refocusing on the foundation.

### 21.3 Recommended Immediate Focus (Priority Order)

**The first and only goal: a vertical slice that works end-to-end.**

1. Complete lexer — tokenize with INDENT/DEDENT. The literal foundation.
2. Complete parser — typed AST from token stream with error recovery and UI tests.
3. Name resolution — bind names to DefIds; report undefined names.
4. Type inference and checking — bidirectional; basic generics; not exhaustive, just correct for a small subset.
5. Minimal MIR and codegen — native code for: variable bindings, arithmetic, if/else, loops, function calls, basic structs. No generics, no traits, no effects.
6. Wire the pipeline — `omni build hello.omni` produces a binary. The binary runs. The binary prints "Hello, World!". Everything else is commentary until this works.

The existing LSP, HELIOS scaffolding, and advanced runtime experiments should be frozen (not deleted) until the core compiler pipeline is complete.

---

## 22. IMPROVEMENTS ADDED IN V2.0 — RESEARCH BASIS

Each improvement is documented with the specific research finding that motivated it.

### 22.1 Algebraic Effect System

**Added**: A first-class effect system where functions declare side effects in their type signatures. User-defined effects via effect handlers. Async, generators, exceptions as effects.

**Research**: Koka (Microsoft Research, actively developed 2025) demonstrates that algebraic effects and handlers let you define advanced control abstractions like async/await as a user library in a typed and composable way. ICFP 2024 proceedings confirm algebraic effect handlers are both theoretically sound and practically implementable.

**Feasibility**: Koka is a production language. The core effect system builds directly on the type inference engine. Effect inference eliminates most annotation burden.

### 22.2 Structured Concurrency as a Hard Constraint

**Added**: `spawn_scope` enforces child tasks cannot outlive creating scope. Unstructured `spawn_global` requires explicit capability.

**Research**: Kotlin enforces parent-child relationships through its Job hierarchy, ensuring that child coroutines cannot outlive their parents, which prevents common resource leaks. Swift's structured concurrency manifesto identified the same problem. Both adopted structured concurrency as the default model.

**Feasibility**: Implementable as a library layer on top of the async executor. `spawn_scope` is equivalent to Kotlin's `coroutineScope` and Swift's `withTaskGroup`. No exotic runtime support required.

### 22.3 Polonius Borrow Checker from Day One

**Added**: Using the Polonius algorithm instead of NLL as the borrow checker.

**Research**: The Rust language design team's 2024 roadmap states: "Non-lexical lifetimes were a big stride forward, but the Polonius project promises to improve the borrow check's precision even more." Polonius eliminates false positives that NLL produces for legitimate programs. Omni avoids shipping the weaker algorithm first.

**Feasibility**: The `polonius-engine` crate (Apache 2.0) is actively maintained. Integration requires adapting Omni's MIR to match Polonius's input format.

### 22.4 Generational References

**Added**: `Gen<T>` generational reference and `Arena<T>` arena allocator as first-class safe alternatives to `Rc<RefCell<T>>` for cyclic data.

**Research**: Vale demonstrates that generational references occupy a sweet spot because they allow objects to be linear yet allow shared mutability in a way that doesn't artificially extend the lifetime of the object. The Rust community identifies Rc<RefCell<T>> as a primary ergonomic pain point for graph-like and cyclic data structures.

**Feasibility**: The `generational-arena` crate demonstrates this is implementable today. Runtime cost is a single integer comparison per dereference.

### 22.5 Linear Types

**Added**: `linear` type modifier requiring types to be used exactly once.

**Research**: Linear types can take care of not just memory safety but resource safety (open files, network connections) generally. The Austral language demonstrates practical linear type usability. Affine types (Rust's ownership) allow discard; linear types do not.

**Feasibility**: Linear type tracking is a straightforward extension of the existing affine type system. The compiler tracks which linear bindings are consumed; scope exit without consuming a linear binding is a compile error.

### 22.6 Field Projections

**Added**: Borrow checker tracks borrows at field granularity within a struct.

**Research**: The Rust language team has been actively working on field projections since 2025, recognizing that the inability to independently borrow struct fields is a significant ergonomic limitation. The Polonius algorithm naturally extends to field-level tracking.

**Feasibility**: Part of the ongoing Polonius work in the Rust project. Implementable from the start since Omni's borrow checker is built fresh with Polonius.

### 22.7 Inout Parameters

**Added**: `inout` parameter syntax for the move-in/move-out pattern.

**Research**: The "move out and move back" pattern produces verbose, unintuitive code. Swift uses `inout` for exactly this purpose. Omni adopts the same solution.

**Feasibility**: Pure syntactic sugar that desugars to existing ownership semantics at the MIR level. Zero runtime cost. No new MIR operations required.

### 22.8 Implied Bounds

**Added**: Struct-level generic bounds are implied in method signatures.

**Research**: The Rust 2024 roadmap explicitly identifies implied bounds as something that "promises to remove a lot of copy-and-pasting of where clauses." This is a documented ergonomic burden in the Rust community.

**Feasibility**: Being implemented in Rust today. The design is well-understood; Omni adopts the final design directly.

### 22.9 Explicit Async Cancellation

**Added**: `CancelToken` and `with_cancel()` for explicit cancellation.

**Research**: Implicit cancellation (future-dropped = cancelled) is one of async Rust's most documented footguns. Swift's `Task.cancel()` and Kotlin's `Job.cancel()` demonstrate explicit cancellation with structured propagation.

**Feasibility**: Implementable as a library abstraction using the effect system. `CancelToken` is a capability-like token passed through the async call chain.

### 22.10 Variadic Generics

**Added**: Variadic generic parameters (`..Ts`) for arbitrary-length type tuples.

**Research**: The Rust 2024 roadmap identifies "variadic tuples and variadic generics" as addressing a common pain point of implementing traits for specific tuple arities.

**Feasibility**: Complex but can be introduced incrementally. Basic form (variadic tuples, variadic function arguments) covers most use cases. Full power is a Phase 7-8 deliverable.

### 22.11 Tensor and SIMD Standard Library

**Added**: `std::tensor` and `std::simd` modules.

**Research**: Mojo builds on MLIR to target CPUs, GPUs, TPUs, ASICs, and other accelerators directly — demonstrating that a systems language can provide portable hardware acceleration as a first-class feature. HELIOS requires native tensor support to avoid the Python/C++ split that has plagued AI tooling.

**Feasibility**: Initial CPU implementation with SIMD dispatch is feasible with existing Rust SIMD libraries as reference. GPU support via MLIR is Phase 13.

### 22.12 Comptime Build Scripts

**Added**: Build logic written in Omni using `comptime` functions, evaluated at build time.

**Research**: Zig's build system is written in Zig itself using comptime, enabling build logic to be expressed in the full language without a separate DSL. Zig is notable for using comptime which lets you run code at compile time instead of at runtime for metaprogramming without a separate macro system.

**Feasibility**: Executable once `comptime` evaluation is implemented (Phase 2-3). The build system calls the comptime evaluator on `build.omni` exactly like any other comptime function.

### 22.13 Fearless FFI Sandboxing

**Added**: Isolated execution context for FFI calls using stack switching.

**Research**: Vale's "Fearless FFI" design uses stack switching to isolate FFI calls, preventing memory corruption in C code from propagating to Vale's managed memory. The core insight: FFI memory corruption is only dangerous if C and Omni share a heap.

**Feasibility**: Stack switching via `sigaltstack` on Unix-like systems and fibers on Windows. Minimal per-FFI-call overhead (two context switches). Initial implementation can use process isolation; native stack switching added later.

### 22.14 Diagnostic Improvements

**Added**: JSON error output, machine-applicable fixes, custom diagnostic attributes for traits, "Did you mean?" suggestions, internationalization support.

**Research**: Research comparing compiler diagnostics across languages finds that Rust and Elm lead. Key characteristics: making it easy to get into the language, explaining errors clearly. Rust 1.78 introduced `#[diagnostic::on_unimplemented]`, allowing library authors to provide custom error messages.

**Feasibility**: All four improvements are straightforward. JSON output requires serializing the diagnostic structure. Machine-applicable fixes are encoded in the same diagnostic structure. Custom diagnostic attributes are simple trait metadata. "Did you mean?" uses Levenshtein edit distance on identifier names.

### 22.15 Replay Debugging

**Added**: Development builds record execution traces for perfect replay.

**Research**: Replay debugging (Mozilla's `rr`, Microsoft's Time Travel Debugging) eliminates Heisenbug problems where bugs disappear when debugging tools are attached. Omni's effect system makes this feasible: all non-deterministic inputs (IO, time, randomness) are declared as effects and can be intercepted and recorded.

**Feasibility**: Since non-deterministic inputs are declared as effects, the effect handler infrastructure intercepts and records them. Replay installs a handler returning recorded results instead of real system calls. Storage cost is O(size of IO operations).

---

## APPENDIX A: DESIGN DECISION REGISTRY (v2.0 — 40 Decisions)

| ID | Decision | Chosen | Rationale |
|---|---|---|---|
| D001 | Language category | Hybrid multi-level platform | Spans systems/application/AI domains |
| D002 | Primary priority | Safety + Performance | Cannot be sacrificed; productivity secondary |
| D003 | Abstraction levels | Multi-level with explicit transitions | All audiences without chaos |
| D004 | Sacred principle | Deterministic correctness + no UB + effects visible in types | Root cause of most expensive bugs |
| D005 | Primary audience | Advanced developers + Framework authors | Beginner access via modes, not primary design |
| D006 | Opinionation level | Moderately opinionated | Prevents chaos without killing flexibility |
| D007 | Typing model | Static + effect annotations; bidirectional inference | Safety + expressiveness |
| D008 | Null handling | Option types; null in restricted zones only | Eliminates NPEs in safe code |
| D009 | Error model | Result types + error sets + typed context chains | Explicit, traceable, composable |
| D010 | Memory core | Ownership-based + generational refs for cyclic data | Deterministic, GC-free, safe by construction |
| D011 | GC compatibility | Optional layer in higher-level modes | Some domains benefit without contaminating core |
| D012 | Unsafe code | Restricted to blocks/functions + Fearless FFI | Necessary for systems; must be auditable |
| D013 | Concurrency model | Hybrid: threads + structured async + actors + channels | Different workloads need different models |
| D014 | Shared mutable state | Only in unsafe mode | Prevents data races statically |
| D015 | Execution default | AOT native | Best performance; other modes explicit |
| D016 | Determinism | Core deterministic; nondeterminism is explicit opt-in | Reproducibility and debuggability |
| D017 | Block syntax | Indentation default, braces in advanced modes | Clean default; power user escape hatch |
| D018 | Semicolons | Newline-first; semicolons disambiguate only | Cleaner code |
| D019 | Expression orientation | Expression-oriented by default | More compositional and concise |
| D020 | Operator overloading | Via traits; standard set; `Try` trait extensible | Type-safe; no unreadable custom operators |
| D021 | Macro system | Two-tier: declarative + sandboxed procedural + comptime codegen | Power without chaos |
| D022 | Module visibility | Layered: private/mod/pkg/pub/capability/friend | Fine-grained encapsulation |
| D023 | Dependency resolution | PubGrub + lockfiles + API compatibility checks | Correct, good errors, reproducible |
| D024 | Package distribution | Multi-target + capability declarations in manifest | Maximum flexibility, security transparency |
| D025 | Language versioning | Edition-based | Controlled evolution |
| D026 | Standard library | Layered + tensor module for AI | Scales from embedded to AI platform |
| D027 | Bootstrap language | Rust | Philosophy alignment; mature tooling |
| D028 | Self-hosting strategy | Multi-stage bootstrap pipeline | Safe, verified, controlled migration |
| D029 | MVP scope | Systems-core first (vertical slice) | Ship something real |
| D030 | Security model | Layered: language + runtime + tooling + ecosystem | Cannot be bolted on later |
| D031 | Effect system | Algebraic effects (built-in + user-defined) | Unifies async, exceptions, generators |
| D032 | Structured concurrency | Hard constraint: children cannot outlive scope | Eliminates resource leak class of bugs |
| D033 | Borrow checker algorithm | Polonius from day one | More precise; no false positives from NLL |
| D034 | Generational references | First-class in stdlib | Safe, ergonomic cyclic data |
| D035 | Linear types | Compiler-enforced `linear` modifier | Resource safety beyond affine types |
| D036 | Field projections | Supported in borrow checker | Eliminates struct-splitting workarounds |
| D037 | Inout parameters | Syntactic sugar for move-in/move-out | Ergonomic ownership transfer |
| D038 | Implied bounds | Struct bounds implied in method signatures | Eliminates where-clause copy-paste |
| D039 | Async cancellation | Explicit CancelToken | Prevents implicit-drop cancellation bugs |
| D040 | Tensor/SIMD stdlib + MLIR | Built-in with MLIR for GPU (future) | HELIOS dependency; AI-first platform |

---

## APPENDIX B: TECHNOLOGY STACK (v2.0)

| Component | Technology | Rationale |
|---|---|---|
| Bootstrap language | Rust (2024 edition) | Philosophy alignment, safety, tooling |
| Parsing | Custom recursive descent + Pratt | Control, error recovery, parallel |
| CST (for formatter) | Rowan | Used by rust-analyzer, lossless |
| Union-find (type inference) | `ena` crate | Used by rustc, proven |
| Borrow checker | `polonius-engine` crate + Omni MIR adapter | More precise than NLL; Apache 2.0 |
| Codegen (development) | Cranelift | Fast, Rust-native, correct |
| Codegen (release) | LLVM via `inkwell` | Maximum optimization |
| Codegen (AI targets, Phase 13) | MLIR | GPU and hardware portability |
| SIMD dispatch | Std SIMD intrinsics via Cranelift/LLVM | Auto-vectorization foundation |
| LSP framework | `tower-lsp` | Mature, async |
| CLI argument parsing | `clap` (derive) | Ergonomic |
| Test runner | `cargo nextest` | Parallel, fast |
| Dependency resolution | PubGrub algorithm | Correct, complete, good errors |
| Manifest format | TOML + `serde` | Ecosystem standard |
| Async executor | Custom work-stealing (structured) | Structured concurrency enforcement |
| Fuzzing | `cargo-fuzz` / libfuzzer | Industry standard |
| Security audit | `cargo-audit` | RUSTSEC advisory database |
| Incremental compilation | Salsa-inspired query model | Fast LSP + incremental builds |
| Replay debugging | Trace recording via effect interceptors | Deterministic debugging |
| Generational references | `generational-arena` crate as reference | Proven approach |
| Fearless FFI | `sigaltstack` / Windows fibers | Stack isolation for FFI safety |
| API compatibility | `cargo-semver-checks` as reference | Supply chain and ABI safety |
