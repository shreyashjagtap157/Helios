# Omni Programming Language — Reference

**Version:** 1.0 — March 2026  
**Paradigm:** Multi-paradigm (systems, application, concurrent)  
**Typing:** Static with optional dynamic (script mode)  
**Memory:** Three modes — GC, Ownership, Manual  
**Concurrency:** Async/await, threads, channels, cooperative  

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Lexical Structure](#2-lexical-structure)
3. [Types](#3-types)
4. [Variables and Mutability](#4-variables-and-mutability)
5. [Functions](#5-functions)
6. [Ownership and Borrowing](#6-ownership-and-borrowing)
7. [Structs](#7-structs)
8. [Traits](#8-traits)
9. [Enums and Pattern Matching](#9-enums-and-pattern-matching)
10. [Collections](#10-collections)
11. [Control Flow](#11-control-flow)
12. [Modules and Imports](#12-modules-and-imports)
13. [Async and Concurrency](#13-async-and-concurrency)
14. [Error Handling](#14-error-handling)
15. [Standard Library](#15-standard-library)
16. [Compiler Flags](#16-compiler-flags)
17. [Module Modes](#17-module-modes)

---

## 1. Introduction

Omni is a modern programming language designed for the Helios AI Cognitive Framework. It combines systems-level control with high-level ergonomics, supporting three memory management modes and multiple concurrency strategies within a single unified syntax.

**Design Goals:**
- Zero-cost abstractions where possible
- Memory safety through ownership, not garbage collection alone
- First-class concurrency primitives
- Self-hosting capability (compiler written in Omni)

---

## 2. Lexical Structure

### 2.1 Source Files

Omni source files use the `.omni` extension and UTF-8 encoding. The language uses **indentation-based scoping** (similar to Python) with colons marking block introducers.

### 2.2 Comments

```
// Single-line comment

/* Multi-line
   comment */
```

### 2.3 Keywords

```
fn, let, var, mut, if, else, while, for, in, loop, match, return,
break, continue, struct, enum, trait, impl, type, import, module,
pub, async, await, spawn, own, shared, unsafe, const, static,
defer, yield, select, true, false, none, some, ok, err, self, Self,
as, where, extern
```

### 2.4 Identifiers

Identifiers start with a letter or underscore, followed by letters, digits, or underscores. Unicode identifiers are supported in string contexts.

```
my_variable    _private    Vec2    MAX_SIZE
```

### 2.5 Literals

```
42              // Integer
3.14            // Float
"hello"         // String
true, false     // Boolean
null            // Null
'c'             // Character
```

---

## 3. Types

### 3.1 Primitive Types

| Type | Description | Size |
|------|-------------|------|
| `i8` - `i64` | Signed integers | 8-64 bits |
| `u8` - `u64` | Unsigned integers | 8-64 bits |
| `f32`, `f64` | Floating point | 32/64 bits |
| `bool` | Boolean | 1 byte |
| `String` | UTF-8 string | Heap |
| `char` | Unicode character | 4 bytes |
| `()` | Unit type | 0 bytes |

### 3.2 Composite Types

```
// Arrays (fixed size)
let arr: [Int; 5] = [1, 2, 3, 4, 5]

// Vectors (dynamic)
let vec: Vector<Int> = Vector::new()

// Tuples
let point: (f64, f64) = (3.0, 4.0)

// Functions
let add: fn(Int, Int) -> Int = |a, b| a + b
```

### 3.3 Generic Types

```
struct Pair<A, B>:
    first: A
    second: B

fn swap<A, B>(p: Pair<A, B>) -> Pair<B, A>:
    Pair { first: p.second, second: p.first }
```

### 3.4 Option and Result

```
let some_val: Option<Int> = Some(42)
let no_val: Option<Int> = None

let ok_val: Result<String, String> = Ok("success")
let err_val: Result<String, String> = Err("failed")
```

---

## 4. Variables and Mutability

### 4.1 Immutable Bindings

```
let x = 42
let name: String = "Omni"
let pi: f64 = 3.14159
```

### 4.2 Mutable Bindings

```
let mut counter = 0
counter += 1

var total: f64 = 0.0  // var is also mutable
total += 1.5
```

### 4.3 Constants

```
const MAX_SIZE: Int = 1024
const PI: f64 = 3.14159265358979
```

### 4.4 Type Inference

Omni infers types from context:

```
let x = 42          // Int
let y = 3.14        // f64
let s = "hello"     // String
let b = true        // Bool
let v = [1, 2, 3]   // [Int]
```

---

## 5. Functions

### 5.1 Basic Functions

```
fn add(a: Int, b: Int) -> Int:
    a + b
```

### 5.2 Functions with Blocks

```
fn fibonacci(n: Int) -> Int:
    if n <= 1:
        return n
    fibonacci(n - 1) + fibonacci(n - 2)
```

### 5.3 Closures

```
let double = |x: Int| x * 2
let add = |a, b| a + b

let numbers = [1, 2, 3, 4, 5]
let doubled = numbers.map(|x| x * 2)
```

### 5.4 Higher-Order Functions

```
fn apply_twice(f: fn(Int) -> Int, x: Int) -> Int:
    f(f(x))

let result = apply_twice(|x| x + 1, 5)  // 7
```

### 5.5 Variadic Functions

```
fn print_all(args: ...String):
    for arg in args:
        println(arg)
```

---

## 6. Ownership and Borrowing

### 6.1 Ownership

```
fn consume(s: String):
    println(s)
    // s is dropped here

let msg = String::from("hello")
consume(msg)
// msg is no longer valid
```

### 6.2 Ownership Keywords

```
let owned = own String::from("uniquely owned")
let shared_ref = shared String::from("reference counted")
```

### 6.3 Borrowing

```
fn inspect(s: &String):
    println("Length: {}", s.len())

fn modify(s: &mut String):
    s.push_str(" world")

let mut greeting = String::from("Hello")
inspect(&greeting)         // Shared borrow
modify(&mut greeting)      // Mutable borrow
```

### 6.4 Move Semantics

```
let a = String::from("hello")
let b = a                  // a is moved to b
// println(a)              // ERROR: a is no longer valid

let x: i32 = 42
let y = x                  // Copy (primitives implement Copy)
println(x)                 // OK: x is still valid
```

### 6.5 Borrow Rules

1. You can have either one mutable reference OR any number of shared references
2. References must always be valid (no dangling references)
3. The borrow checker enforces these rules at compile time

---

## 7. Structs

### 7.1 Definition

```
struct Vec2:
    x: f64
    y: f64
```

### 7.2 Construction

```
let v = Vec2 { x: 3.0, y: 4.0 }
let v2 = Vec2 { x: 1.0, y: 2.0 }
```

### 7.3 Methods (impl)

```
impl Vec2:
    fn new(x: f64, y: f64) -> Vec2:
        Vec2 { x: x, y: y }

    fn length(&self) -> f64:
        sqrt(self.x * self.x + self.y * self.y)

    fn add(&mut self, other: &Vec2):
        self.x += other.x
        self.y += other.y

    fn scale(self, factor: f64) -> Vec2:
        Vec2 { x: self.x * factor, y: self.y * factor }
```

### 7.4 Operator Overloading via Traits

```
impl Add for Vec2:
    type Output = Vec2
    fn add(self, other: Vec2) -> Vec2:
        Vec2 { x: self.x + other.x, y: self.y + other.y }
```

---

## 8. Traits

### 8.1 Definition

```
trait Drawable:
    fn draw(&self)
    fn bounds(&self) -> Rect
```

### 8.2 Implementation

```
impl Drawable for Circle:
    fn draw(&self):
        println("Drawing circle at ({}, {})", self.center.x, self.center.y)

    fn bounds(&self) -> Rect:
        Rect {
            x: self.center.x - self.radius,
            y: self.center.y - self.radius,
            width: self.radius * 2.0,
            height: self.radius * 2.0,
        }
```

### 8.3 Trait Bounds

```
fn print_drawable<T: Drawable>(item: &T):
    item.draw()

fn process<T>(item: T) where T: Drawable + Clone:
    let copy = item.clone()
    copy.draw()
```

### 8.4 Built-in Traits

- `Clone` — deep copy
- `Debug` — debug formatting
- `Display` — user-facing formatting
- `PartialEq`, `Eq` — equality
- `PartialOrd`, `Ord` — ordering
- `Hash` — hashing
- `Iterator` — iteration
- `From`, `Into` — conversions
- `Add`, `Sub`, `Mul`, `Div` — arithmetic operators

---

## 9. Enums and Pattern Matching

### 9.1 Enum Definition

```
enum Shape:
    Circle(f64)                          // radius
    Rectangle(f64, f64)                  // width, height
    Triangle(f64, f64, f64)             // sides
```

### 9.2 Option and Result

```
enum Option<T>:
    Some(T)
    None

enum Result<T, E>:
    Ok(T)
    Err(E)
```

### 9.3 Pattern Matching

```
fn area(shape: Shape) -> f64:
    match shape:
        Shape::Circle(r):
            3.14159 * r * r
        Shape::Rectangle(w, h):
            w * h
        Shape::Triangle(a, b, c):
            let s = (a + b + c) / 2.0
            sqrt(s * (s - a) * (s - b) * (s - c))
```

### 9.4 Pattern Types

```
// Wildcard
match x:
    _:
        println("anything")

// Binding
match x:
    value:
        println("captured: {}", value)

// Literal
match x:
    42:
        println("forty-two")
    0:
        println("zero")

// Constructor
match option:
    Some(val):
        println("got: {}", val)
    None:
        println("nothing")

// Path constructor
match option:
    Option::Some(val):
        println("got: {}", val)
    Option::None:
        println("nothing")
```

---

## 10. Collections

### 10.1 Arrays

```
let arr: [Int; 5] = [1, 2, 3, 4, 5]
let first = arr[0]          // 1
let len = arr.len()         // 5
```

### 10.2 Vector

```
let mut v: Vector<Int> = Vector::new()
v.push(1)
v.push(2)
v.push(3)

println("Length: {}", v.len())       // 3
println("First: {}", v.get(0).unwrap())  // 1

let last = v.pop()                   // removes and returns 3

for item in &v:
    println(item)
```

### 10.3 HashMap

```
let mut map: HashMap<String, Int> = HashMap::new()
map.insert("Alice", 95)
map.insert("Bob", 87)

match map.get("Bob"):
    Some(score):
        println("Bob: {}", score)
    None:
        println("Not found")

println("Has Alice? {}", map.contains_key("Alice"))
```

### 10.4 HashSet

```
let mut set: HashSet<String> = HashSet::new()
set.insert("hello")
set.insert("world")

println("Contains hello? {}", set.contains("hello"))
```

---

## 11. Control Flow

### 11.1 if/else

```
if x > 0:
    println("positive")
else if x < 0:
    println("negative")
else:
    println("zero")

// As expression
let label = if x > 0: "positive" else: "non-positive"
```

### 11.2 while

```
let mut i = 0
while i < 10:
    println(i)
    i += 1
```

### 11.3 for

```
for i in 0..10:
    println(i)

for item in collection:
    println(item)

for (key, value) in &map:
    println("{}: {}", key, value)
```

### 11.4 loop

```
let mut count = 0
loop:
    if count >= 10:
        break
    count += 1
```

### 11.5 break and continue

```
for i in 0..100:
    if i % 2 == 0:
        continue
    if i > 50:
        break
    println(i)
```

### 11.6 match (see Section 9)

### 11.7 return

```
fn find_first_even(numbers: &[Int]) -> Option<Int>:
    for n in numbers:
        if n % 2 == 0:
            return Some(n)
    None
```

---

## 12. Modules and Imports

### 12.1 Module Declaration

```
module tutorials::ownership
module std::collections
module my_app::models::user
```

### 12.2 Imports

```
import std::collections::{Vector, HashMap}
import std::io::{File, read_to_string}
import my_app::models::user::User

import std::math::{sqrt as square_root}
```

### 12.3 Visibility

```
pub struct PublicStruct:
    pub field: Int
    private_field: String

pub fn public_function():
    println("public")

fn private_function():
    println("private")
```

---

## 13. Async and Concurrency

### 13.1 Async Functions

```
async fn fetch_data(url: String) -> String:
    let response = await http_get(url)
    response.body()
```

### 13.2 Spawning Tasks

```
async fn main():
    let handle1 = spawn fetch_data("url1")
    let handle2 = spawn fetch_data("url2")

    let result1 = await handle1
    let result2 = await handle2
```

### 13.3 Channels

```
import std::thread::{Channel}

fn main():
    let (tx, rx) = Channel::new()

    spawn:
        tx.send("hello from task")

    let msg = rx.recv()
    println(msg)
```

### 13.4 Shared State

```
import std::thread::{Arc, Mutex}

fn main():
    let counter = Arc::new(Mutex::new(0))

    for i in 0..10:
        let c = counter.clone()
        spawn:
            let mut num = c.lock()
            *num += 1
```

---

## 14. Error Handling

### 14.1 Result Type

```
fn divide(a: f64, b: f64) -> Result<f64, String>:
    if b == 0.0:
        Err("Division by zero")
    else:
        Ok(a / b)
```

### 14.2 Unwrap

```
let result = divide(10.0, 3.0).unwrap()  // Panics on Err
let value = divide(10.0, 3.0).ok()      // Returns Option<T>
```

### 14.3 Match on Result

```
match divide(10.0, 0.0):
    Ok(val):
        println("Result: {}", val)
    Err(msg):
        println("Error: {}", msg)
```

---

## 15. Standard Library

### 15.1 Core (std::core)

- `Option`, `Some`, `None`
- `Result`, `Ok`, `Err`
- `String` — UTF-8 string type
- `Iterator` — lazy iteration
- `print()`, `println()` — console output
- `format()` — string formatting
- `type_of()` — runtime type name
- `assert()` — assertion

### 15.2 Collections (std::collections)

- `Vector<T>` — dynamic array
- `HashMap<K,V>` — hash map
- `HashSet<T>` — hash set
- `Stack<T>` — LIFO stack
- `Queue<T>` — FIFO queue
- `Deque<T>` — double-ended queue

### 15.3 Math (std::math)

- `sqrt()`, `abs()`, `pow()`
- `sin()`, `cos()`, `tan()`
- `min()`, `max()`, `clamp()`
- `floor()`, `ceil()`, `round()`

### 15.4 IO (std::io)

- `File::open()`, `File::create()`
- `read_to_string()`, `write_string()`
- `stdin()`, `stdout()`, `stderr()`

### 15.5 Thread (std::thread)

- `spawn` — create thread
- `Channel` — message passing
- `Arc` — atomic reference counting
- `Mutex` — mutual exclusion

### 15.6 Async (std::async)

- `async fn` — async function
- `await` — await future
- `spawn` — spawn task
- `block_on()` — run future synchronously

---

## 16. Compiler Flags

| Flag | Description |
|------|-------------|
| `--run` | Execute immediately (interpreter mode) |
| `--target ovm` | Code generation target |
| `--emit-ir` | Emit Omni IR |
| `--emit-llvm` | Emit LLVM IR |
| `--opt-level N` | Optimization level (0-3) |
| `-g` | Generate debug info |
| `--mode MODE` | Module mode (script/hosted/bare_metal) |
| `--deterministic` | Deterministic builds |
| `--resolver-log DIR` | Write resolver decision logs |
| `--monitor` | Enable runtime monitor |
| `--profile` | Enable PGO profiling |
| `--hardware-adaptive` | Hardware-specific optimization |

---

## 17. Module Modes

### 17.1 Script Mode

For quick scripts and prototyping. Uses interpreter, dynamic typing, limited stdlib.

```
# No special declaration needed — use --mode script
```

### 17.2 Hosted Mode (Default)

For production applications. Full stdlib, GC, optional JIT.

```
# Default mode — use --mode hosted or omit
```

### 17.3 Bare Metal Mode

For systems programming. No GC, ownership-based memory, inline asm.

```
# Use --mode bare_metal
```

| Feature | Script | Hosted | Bare Metal |
|---------|--------|--------|------------|
| GC | ✗ | ✓ | ✗ |
| JIT | ✗ | ✓ | ✗ |
| AOT | ✗ | ✓ | ✓ |
| Unsafe | ✗ | ✗ | ✓ |
| Inline Asm | ✗ | ✗ | ✓ |
| OS Threads | ✗ | ✓ | ✗ |
| Async | ✓ | ✓ | ✗ |
| Ownership | ✗ | ✓ | ✓ |
| Manual Memory | ✗ | ✗ | ✓ |
| FFI | ✗ | ✓ | ✓ |
| Dynamic Typing | ✓ | ✓ | ✗ |
| Static Typing | ✗ | ✓ | ✓ |
| Reflection | ✓ | ✓ | ✗ |

---

*Omni Language Reference — March 2026*  
*Part of the Helios AI Cognitive Framework*
