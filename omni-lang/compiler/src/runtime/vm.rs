// Copyright 2024 Shreyash Jagtap
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(dead_code)]
//! OVM Bytecode Virtual Machine
//!
//! A stack-based virtual machine that executes OVM bytecode produced by the
//! bytecode compiler. Supports arithmetic, control flow, function calls,
//! composite types (arrays, maps, structs), and built-in operations.

use std::collections::HashMap;
use std::fmt;

use super::bytecode::{CompiledFunction, OpCode, OvmModule, Value};

// ---------------------------------------------------------------------------
// VmValue – extended runtime values
// ---------------------------------------------------------------------------

/// Runtime value representation with support for composite types.
#[derive(Debug, Clone, PartialEq)]
pub enum VmValue {
    Null,
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Array(Vec<VmValue>),
    Map(Vec<(VmValue, VmValue)>),
    Struct(String, Vec<(String, VmValue)>),
    /// Reference to a heap-allocated object managed by the GC.
    HeapRef(usize),
}

impl fmt::Display for VmValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VmValue::Null => write!(f, "null"),
            VmValue::Int(i) => write!(f, "{}", i),
            VmValue::Float(v) => {
                if v.fract() == 0.0 && v.is_finite() {
                    write!(f, "{:.1}", v)
                } else {
                    write!(f, "{}", v)
                }
            }
            VmValue::Bool(b) => write!(f, "{}", b),
            VmValue::String(s) => write!(f, "{}", s),
            VmValue::Array(elems) => {
                write!(f, "[")?;
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", e)?;
                }
                write!(f, "]")
            }
            VmValue::Map(pairs) => {
                write!(f, "{{")?;
                for (i, (k, v)) in pairs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
            VmValue::Struct(name, fields) => {
                write!(f, "{} {{", name)?;
                for (i, (fname, fval)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", fname, fval)?;
                }
                write!(f, "}}")
            }
            VmValue::HeapRef(idx) => write!(f, "<heap@{}>", idx),
        }
    }
}

// ---------------------------------------------------------------------------
// CallFrame – function execution context
// ---------------------------------------------------------------------------

/// Tracks the execution context of a single function invocation.
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// Index into `OvmModule::functions`
    pub function_index: usize,
    /// Instruction pointer (index into the function's instruction stream)
    pub ip: usize,
    /// Base offset into the VM stack for this frame
    pub base_slot: usize,
    /// Local variable slots
    pub locals: Vec<VmValue>,
}

// ---------------------------------------------------------------------------
// GC Infrastructure — Tri-color Mark-and-Sweep
// ---------------------------------------------------------------------------

/// Header for each heap-allocated object.
#[derive(Debug, Clone)]
pub struct GcHeader {
    pub mark: bool,
}

/// Heap-allocated object managed by the GC.
#[derive(Debug, Clone)]
pub enum HeapCell {
    Array(Vec<VmValue>),
    Map(Vec<(VmValue, VmValue)>),
    Struct(String, Vec<(String, VmValue)>),
    HeapString(String),
}

impl HeapCell {
    /// Iterate over all VmValue references contained in this heap cell.
    fn references(&self) -> Vec<usize> {
        let mut refs = Vec::new();
        match self {
            HeapCell::Array(items) => {
                for item in items {
                    if let VmValue::HeapRef(idx) = item {
                        refs.push(*idx);
                    }
                }
            }
            HeapCell::Map(pairs) => {
                for (k, v) in pairs {
                    if let VmValue::HeapRef(idx) = k {
                        refs.push(*idx);
                    }
                    if let VmValue::HeapRef(idx) = v {
                        refs.push(*idx);
                    }
                }
            }
            HeapCell::Struct(_, fields) => {
                for (_, v) in fields {
                    if let VmValue::HeapRef(idx) = v {
                        refs.push(*idx);
                    }
                }
            }
            HeapCell::HeapString(_) => {}
        }
        refs
    }
}

// ---------------------------------------------------------------------------
// OmniVM – the virtual machine
// ---------------------------------------------------------------------------

/// Stack-based virtual machine for executing OVM bytecode.
pub struct OmniVM {
    /// Operand stack
    pub stack: Vec<VmValue>,
    /// Global variables
    pub globals: HashMap<String, VmValue>,
    /// Call stack (one frame per active function invocation)
    pub call_stack: Vec<CallFrame>,
    /// Captured output from Print / PrintLn (useful for testing)
    pub output: Vec<String>,
    /// GC heap: each slot is either occupied (header + cell) or free (None)
    pub heap: Vec<Option<(GcHeader, HeapCell)>>,
    /// Free-list of reclaimed heap indices
    pub free_list: Vec<usize>,
    /// Number of allocations since last GC cycle
    pub allocations: usize,
    /// Allocation count threshold that triggers a GC cycle
    pub gc_threshold: usize,
}

impl OmniVM {
    /// Create a new, empty VM.
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            globals: HashMap::new(),
            call_stack: Vec::new(),
            output: Vec::new(),
            heap: Vec::new(),
            free_list: Vec::new(),
            allocations: 0,
            gc_threshold: 256,
        }
    }

    // -- GC methods ---------------------------------------------------------

    /// Allocate a new heap cell, returning its index.
    /// Triggers GC if allocation count exceeds threshold.
    pub fn alloc(&mut self, cell: HeapCell) -> usize {
        self.allocations += 1;

        // Trigger GC when threshold exceeded
        if self.allocations > self.gc_threshold {
            self.gc_collect();
        }

        let header = GcHeader { mark: false };
        if let Some(idx) = self.free_list.pop() {
            self.heap[idx] = Some((header, cell));
            idx
        } else {
            let idx = self.heap.len();
            self.heap.push(Some((header, cell)));
            idx
        }
    }

    /// Collect all HeapRef indices reachable from the stack, locals, and globals.
    fn gc_collect_roots(&self) -> Vec<usize> {
        let mut roots = Vec::new();

        // Scan operand stack
        for val in &self.stack {
            if let VmValue::HeapRef(idx) = val {
                roots.push(*idx);
            }
        }

        // Scan globals
        for val in self.globals.values() {
            if let VmValue::HeapRef(idx) = val {
                roots.push(*idx);
            }
        }

        // Scan call-frame locals
        for frame in &self.call_stack {
            for val in &frame.locals {
                if let VmValue::HeapRef(idx) = val {
                    roots.push(*idx);
                }
            }
        }

        roots
    }

    /// Run a full mark-and-sweep garbage collection cycle.
    pub fn gc_collect(&mut self) {
        // ── Mark phase (tri-color: gray worklist → black when done) ──
        let mut worklist: Vec<usize> = self.gc_collect_roots();

        // Clear all marks first
        for slot in self.heap.iter_mut() {
            if let Some((header, _)) = slot {
                header.mark = false;
            }
        }

        // Process worklist
        while let Some(idx) = worklist.pop() {
            if idx >= self.heap.len() {
                continue;
            }
            if let Some((ref mut header, ref cell)) = self.heap[idx] {
                if header.mark {
                    continue; // already visited (black)
                }
                header.mark = true;
                // Discover children (gray → add to worklist)
                for child_idx in cell.references() {
                    if child_idx < self.heap.len() {
                        if let Some((ref child_header, _)) = self.heap[child_idx] {
                            if !child_header.mark {
                                worklist.push(child_idx);
                            }
                        }
                    }
                }
            }
        }

        // ── Sweep phase ──
        for idx in 0..self.heap.len() {
            let should_free = if let Some((ref header, _)) = self.heap[idx] {
                !header.mark
            } else {
                false
            };
            if should_free {
                self.heap[idx] = None;
                self.free_list.push(idx);
            }
        }

        // Reset allocation counter, double threshold
        self.allocations = 0;
        self.gc_threshold = self.gc_threshold.saturating_mul(2).max(256);
    }

    /// Dereference a HeapRef to get a clone of the heap cell.
    pub fn heap_get(&self, idx: usize) -> Option<&HeapCell> {
        self.heap
            .get(idx)
            .and_then(|slot| slot.as_ref().map(|(_, cell)| cell))
    }

    // -- Helper methods -----------------------------------------------------

    /// Pop a value from the operand stack.
    fn pop(&mut self) -> Result<VmValue, String> {
        self.stack
            .pop()
            .ok_or_else(|| "Stack underflow".to_string())
    }

    /// Push a value onto the operand stack.
    fn push(&mut self, val: VmValue) {
        self.stack.push(val);
    }

    /// Peek at the top of the operand stack without removing it.
    fn peek(&self) -> Result<&VmValue, String> {
        self.stack
            .last()
            .ok_or_else(|| "Stack underflow (peek)".to_string())
    }

    /// Determine whether a value is truthy.
    fn is_truthy(val: &VmValue) -> bool {
        match val {
            VmValue::Null => false,
            VmValue::Bool(b) => *b,
            VmValue::Int(i) => *i != 0,
            VmValue::Float(f) => *f != 0.0,
            VmValue::String(s) => !s.is_empty(),
            VmValue::Array(a) => !a.is_empty(),
            VmValue::Map(m) => !m.is_empty(),
            VmValue::Struct(_, _) => true,
            VmValue::HeapRef(_) => true,
        }
    }

    /// Convert a bytecode `Value` into a runtime `VmValue`.
    fn value_to_vm(val: &Value) -> VmValue {
        match val {
            Value::Null => VmValue::Null,
            Value::Int(i) => VmValue::Int(*i),
            Value::Float(f) => VmValue::Float(*f),
            Value::Bool(b) => VmValue::Bool(*b),
            Value::String(s) => VmValue::String(s.clone()),
        }
    }

    // -- Main execution loop ------------------------------------------------

    /// Execute an OVM module to completion.
    ///
    /// Finds the entry point (via `module.entry_point` or by searching for a
    /// function named `"main"`), pushes the initial call frame, and runs the
    /// fetch-decode-execute loop until a `Halt` instruction is reached or the
    /// last frame's instructions are exhausted.
    pub fn execute(&mut self, module: &OvmModule) -> Result<(), String> {
        // Locate entry point
        let entry_idx = if let Some(idx) = module.entry_point {
            idx
        } else {
            module
                .functions
                .iter()
                .position(|f| f.name == "main")
                .ok_or_else(|| {
                    "No entry point: no `entry_point` set and no `main` function found".to_string()
                })?
        };

        if entry_idx >= module.functions.len() {
            return Err(format!(
                "Entry point index {} out of range (module has {} functions)",
                entry_idx,
                module.functions.len()
            ));
        }

        // Push initial call frame
        let entry_fn = &module.functions[entry_idx];
        let locals = vec![VmValue::Null; entry_fn.locals_count];
        self.call_stack.push(CallFrame {
            function_index: entry_idx,
            ip: 0,
            base_slot: 0,
            locals,
        });

        // Fetch-decode-execute loop
        loop {
            if self.call_stack.is_empty() {
                break;
            }

            let frame = self.call_stack.last().unwrap();
            let func = &module.functions[frame.function_index];

            // Check if we've run past the end of the current function
            if frame.ip >= func.instructions.len() {
                // Implicit return from the function
                self.call_stack.pop();
                if self.call_stack.is_empty() {
                    break;
                }
                continue;
            }

            // Fetch instruction (clone to avoid borrow issues)
            let instruction = func.instructions[frame.ip].clone();

            // Advance IP *before* execution so jumps can override it
            self.call_stack.last_mut().unwrap().ip += 1;

            // Decode & execute
            match instruction {
                OpCode::Nop => { /* do nothing */ }

                OpCode::Push(ref v) => {
                    let vm_val = Self::value_to_vm(v);
                    self.push(vm_val);
                }

                OpCode::Pop => {
                    self.pop()?;
                }

                OpCode::Dup => {
                    let val = self.peek()?.clone();
                    self.push(val);
                }

                OpCode::Swap => {
                    let a = self.pop()?;
                    let b = self.pop()?;
                    self.push(a);
                    self.push(b);
                }

                // -- Arithmetic -------------------------------------------------
                OpCode::Add => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = match (&a, &b) {
                        (VmValue::Int(x), VmValue::Int(y)) => VmValue::Int(x + y),
                        (VmValue::Float(x), VmValue::Float(y)) => VmValue::Float(x + y),
                        (VmValue::Int(x), VmValue::Float(y)) => VmValue::Float(*x as f64 + y),
                        (VmValue::Float(x), VmValue::Int(y)) => VmValue::Float(x + *y as f64),
                        (VmValue::String(x), VmValue::String(y)) => {
                            VmValue::String(format!("{}{}", x, y))
                        }
                        _ => return Err(format!("Cannot add {:?} and {:?}", a, b)),
                    };
                    self.push(result);
                }

                OpCode::Sub => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = match (&a, &b) {
                        (VmValue::Int(x), VmValue::Int(y)) => VmValue::Int(x - y),
                        (VmValue::Float(x), VmValue::Float(y)) => VmValue::Float(x - y),
                        (VmValue::Int(x), VmValue::Float(y)) => VmValue::Float(*x as f64 - y),
                        (VmValue::Float(x), VmValue::Int(y)) => VmValue::Float(x - *y as f64),
                        _ => return Err(format!("Cannot subtract {:?} from {:?}", b, a)),
                    };
                    self.push(result);
                }

                OpCode::Mul => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = match (&a, &b) {
                        (VmValue::Int(x), VmValue::Int(y)) => VmValue::Int(x * y),
                        (VmValue::Float(x), VmValue::Float(y)) => VmValue::Float(x * y),
                        (VmValue::Int(x), VmValue::Float(y)) => VmValue::Float(*x as f64 * y),
                        (VmValue::Float(x), VmValue::Int(y)) => VmValue::Float(x * *y as f64),
                        _ => return Err(format!("Cannot multiply {:?} and {:?}", a, b)),
                    };
                    self.push(result);
                }

                OpCode::Div => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = match (&a, &b) {
                        (VmValue::Int(_), VmValue::Int(0)) => {
                            return Err("Division by zero".to_string())
                        }
                        (VmValue::Int(x), VmValue::Int(y)) => VmValue::Int(x / y),
                        (VmValue::Float(x), VmValue::Float(y)) => {
                            if *y == 0.0 {
                                return Err("Division by zero".to_string());
                            }
                            VmValue::Float(x / y)
                        }
                        (VmValue::Int(x), VmValue::Float(y)) => {
                            if *y == 0.0 {
                                return Err("Division by zero".to_string());
                            }
                            VmValue::Float(*x as f64 / y)
                        }
                        (VmValue::Float(x), VmValue::Int(y)) => {
                            if *y == 0 {
                                return Err("Division by zero".to_string());
                            }
                            VmValue::Float(x / *y as f64)
                        }
                        _ => return Err(format!("Cannot divide {:?} by {:?}", a, b)),
                    };
                    self.push(result);
                }

                OpCode::Mod => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = match (&a, &b) {
                        (VmValue::Int(_), VmValue::Int(0)) => {
                            return Err("Modulo by zero".to_string())
                        }
                        (VmValue::Int(x), VmValue::Int(y)) => VmValue::Int(x % y),
                        (VmValue::Float(x), VmValue::Float(y)) => VmValue::Float(x % y),
                        (VmValue::Int(x), VmValue::Float(y)) => VmValue::Float(*x as f64 % y),
                        (VmValue::Float(x), VmValue::Int(y)) => VmValue::Float(x % *y as f64),
                        _ => return Err(format!("Cannot modulo {:?} by {:?}", a, b)),
                    };
                    self.push(result);
                }

                OpCode::Neg => {
                    let val = self.pop()?;
                    let result = match val {
                        VmValue::Int(x) => VmValue::Int(-x),
                        VmValue::Float(x) => VmValue::Float(-x),
                        _ => return Err(format!("Cannot negate {:?}", val)),
                    };
                    self.push(result);
                }

                // -- Comparison -------------------------------------------------
                OpCode::Eq => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(VmValue::Bool(a == b));
                }

                OpCode::Ne => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(VmValue::Bool(a != b));
                }

                OpCode::Lt => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = match (&a, &b) {
                        (VmValue::Int(x), VmValue::Int(y)) => x < y,
                        (VmValue::Float(x), VmValue::Float(y)) => x < y,
                        (VmValue::Int(x), VmValue::Float(y)) => (*x as f64) < *y,
                        (VmValue::Float(x), VmValue::Int(y)) => *x < (*y as f64),
                        (VmValue::String(x), VmValue::String(y)) => x < y,
                        _ => return Err(format!("Cannot compare {:?} < {:?}", a, b)),
                    };
                    self.push(VmValue::Bool(result));
                }

                OpCode::Le => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = match (&a, &b) {
                        (VmValue::Int(x), VmValue::Int(y)) => x <= y,
                        (VmValue::Float(x), VmValue::Float(y)) => x <= y,
                        (VmValue::Int(x), VmValue::Float(y)) => (*x as f64) <= *y,
                        (VmValue::Float(x), VmValue::Int(y)) => *x <= (*y as f64),
                        (VmValue::String(x), VmValue::String(y)) => x <= y,
                        _ => return Err(format!("Cannot compare {:?} <= {:?}", a, b)),
                    };
                    self.push(VmValue::Bool(result));
                }

                OpCode::Gt => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = match (&a, &b) {
                        (VmValue::Int(x), VmValue::Int(y)) => x > y,
                        (VmValue::Float(x), VmValue::Float(y)) => x > y,
                        (VmValue::Int(x), VmValue::Float(y)) => (*x as f64) > *y,
                        (VmValue::Float(x), VmValue::Int(y)) => *x > (*y as f64),
                        (VmValue::String(x), VmValue::String(y)) => x > y,
                        _ => return Err(format!("Cannot compare {:?} > {:?}", a, b)),
                    };
                    self.push(VmValue::Bool(result));
                }

                OpCode::Ge => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = match (&a, &b) {
                        (VmValue::Int(x), VmValue::Int(y)) => x >= y,
                        (VmValue::Float(x), VmValue::Float(y)) => x >= y,
                        (VmValue::Int(x), VmValue::Float(y)) => (*x as f64) >= *y,
                        (VmValue::Float(x), VmValue::Int(y)) => *x >= (*y as f64),
                        (VmValue::String(x), VmValue::String(y)) => x >= y,
                        _ => return Err(format!("Cannot compare {:?} >= {:?}", a, b)),
                    };
                    self.push(VmValue::Bool(result));
                }

                // -- Logic ------------------------------------------------------
                OpCode::And => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (&a, &b) {
                        (VmValue::Bool(x), VmValue::Bool(y)) => {
                            self.push(VmValue::Bool(*x && *y));
                        }
                        _ => return Err(format!("Cannot AND {:?} and {:?}", a, b)),
                    }
                }

                OpCode::Or => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (&a, &b) {
                        (VmValue::Bool(x), VmValue::Bool(y)) => {
                            self.push(VmValue::Bool(*x || *y));
                        }
                        _ => return Err(format!("Cannot OR {:?} and {:?}", a, b)),
                    }
                }

                OpCode::Not => {
                    let val = self.pop()?;
                    match val {
                        VmValue::Bool(b) => self.push(VmValue::Bool(!b)),
                        _ => return Err(format!("Cannot NOT {:?}", val)),
                    }
                }

                // -- String -----------------------------------------------------
                OpCode::Concat => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (&a, &b) {
                        (VmValue::String(x), VmValue::String(y)) => {
                            self.push(VmValue::String(format!("{}{}", x, y)));
                        }
                        _ => return Err(format!("Cannot concat {:?} and {:?}", a, b)),
                    }
                }

                // -- Local variables --------------------------------------------
                OpCode::LoadLocal(n) => {
                    let frame = self.call_stack.last().unwrap();
                    if n >= frame.locals.len() {
                        return Err(format!(
                            "LoadLocal: index {} out of range (function has {} locals)",
                            n,
                            frame.locals.len()
                        ));
                    }
                    let val = frame.locals[n].clone();
                    self.push(val);
                }

                OpCode::StoreLocal(n) => {
                    let val = self.pop()?;
                    let frame = self.call_stack.last_mut().unwrap();
                    // Grow locals vector if needed
                    while frame.locals.len() <= n {
                        frame.locals.push(VmValue::Null);
                    }
                    frame.locals[n] = val;
                }

                // -- Global variables -------------------------------------------
                OpCode::LoadGlobal(ref name) => {
                    let val = self.globals.get(name).cloned().unwrap_or(VmValue::Null);
                    self.push(val);
                }

                OpCode::StoreGlobal(ref name) => {
                    let val = self.pop()?;
                    self.globals.insert(name.clone(), val);
                }

                // -- Fields (struct access) -------------------------------------
                OpCode::LoadField(ref field_name) => {
                    let obj = self.pop()?;
                    match obj {
                        VmValue::Struct(_, ref fields) => {
                            let val = fields
                                .iter()
                                .find(|(name, _)| name == field_name)
                                .map(|(_, v)| v.clone())
                                .ok_or_else(|| {
                                    format!("Field '{}' not found in struct", field_name)
                                })?;
                            self.push(val);
                        }
                        _ => return Err(format!("Cannot load field from {:?}", obj)),
                    }
                }

                OpCode::StoreField(ref field_name) => {
                    let val = self.pop()?;
                    let obj = self.pop()?;
                    match obj {
                        VmValue::Struct(sname, mut fields) => {
                            let mut found = false;
                            for (fname, fval) in fields.iter_mut() {
                                if fname == field_name {
                                    *fval = val.clone();
                                    found = true;
                                    break;
                                }
                            }
                            if !found {
                                fields.push((field_name.clone(), val));
                            }
                            self.push(VmValue::Struct(sname, fields));
                        }
                        _ => return Err(format!("Cannot store field on {:?}", obj)),
                    }
                }

                // -- Control flow -----------------------------------------------
                OpCode::Jump(addr) => {
                    self.call_stack.last_mut().unwrap().ip = addr;
                }

                OpCode::JumpIf(addr) => {
                    let val = self.pop()?;
                    if Self::is_truthy(&val) {
                        self.call_stack.last_mut().unwrap().ip = addr;
                    }
                }

                OpCode::JumpIfNot(addr) => {
                    let val = self.pop()?;
                    if !Self::is_truthy(&val) {
                        self.call_stack.last_mut().unwrap().ip = addr;
                    }
                }

                // -- Function calls ---------------------------------------------
                OpCode::Call(_n_args) => {
                    return Err(
                        "Call(n) with function-on-stack is not yet supported; use CallNamed"
                            .to_string(),
                    );
                }

                OpCode::CallNamed(ref name, n_args) => {
                    // Look up the function by name
                    let func_idx = module
                        .functions
                        .iter()
                        .position(|f| f.name == *name)
                        .ok_or_else(|| format!("Function '{}' not found", name))?;

                    // Pop arguments from the stack (they are pushed left-to-right,
                    // so the last argument is on top).
                    let mut args = Vec::with_capacity(n_args);
                    for _ in 0..n_args {
                        args.push(self.pop()?);
                    }
                    args.reverse();

                    // Build locals: start with args, pad with Null up to locals_count
                    let target_fn = &module.functions[func_idx];
                    let mut locals = args;
                    while locals.len() < target_fn.locals_count {
                        locals.push(VmValue::Null);
                    }

                    self.call_stack.push(CallFrame {
                        function_index: func_idx,
                        ip: 0,
                        base_slot: self.stack.len(),
                        locals,
                    });
                }

                OpCode::Return => {
                    // Pop return value (or Null if the stack is empty for this frame)
                    let return_val = if !self.stack.is_empty() {
                        self.pop()?
                    } else {
                        VmValue::Null
                    };

                    // Pop the call frame
                    self.call_stack.pop();

                    if self.call_stack.is_empty() {
                        // Return from the top-level function – push for tests
                        self.push(return_val);
                        break;
                    }

                    // Push return value onto the caller's stack
                    self.push(return_val);
                }

                // -- Composite constructors -------------------------------------
                OpCode::NewStruct(ref name, n) => {
                    let mut fields = Vec::with_capacity(n);
                    for i in (0..n).rev() {
                        let val = self.pop()?;
                        fields.push((format!("field_{}", i), val));
                    }
                    fields.reverse();
                    self.push(VmValue::Struct(name.clone(), fields));
                }

                OpCode::NewArray(n) => {
                    let mut elems = Vec::with_capacity(n);
                    for _ in 0..n {
                        elems.push(self.pop()?);
                    }
                    elems.reverse();
                    self.push(VmValue::Array(elems));
                }

                OpCode::NewMap(n) => {
                    // n = number of key-value pairs → pop 2n values
                    let mut pairs = Vec::with_capacity(n);
                    for _ in 0..n {
                        let val = self.pop()?;
                        let key = self.pop()?;
                        pairs.push((key, val));
                    }
                    pairs.reverse();
                    self.push(VmValue::Map(pairs));
                }

                OpCode::Index => {
                    let index = self.pop()?;
                    let collection = self.pop()?;
                    match (&collection, &index) {
                        (VmValue::Array(elems), VmValue::Int(i)) => {
                            let idx = *i as usize;
                            if idx >= elems.len() {
                                return Err(format!(
                                    "Array index {} out of bounds (length {})",
                                    i,
                                    elems.len()
                                ));
                            }
                            self.push(elems[idx].clone());
                        }
                        (VmValue::Map(pairs), _) => {
                            let val = pairs
                                .iter()
                                .find(|(k, _)| k == &index)
                                .map(|(_, v)| v.clone())
                                .unwrap_or(VmValue::Null);
                            self.push(val);
                        }
                        (VmValue::String(s), VmValue::Int(i)) => {
                            let idx = *i as usize;
                            match s.chars().nth(idx) {
                                Some(c) => self.push(VmValue::String(c.to_string())),
                                None => {
                                    return Err(format!(
                                        "String index {} out of bounds (length {})",
                                        i,
                                        s.len()
                                    ))
                                }
                            }
                        }
                        _ => return Err(format!("Cannot index {:?} with {:?}", collection, index)),
                    }
                }

                // -- Built-in operations ----------------------------------------
                OpCode::Print => {
                    let val = self.pop()?;
                    let text = format!("{}", val);
                    self.output.push(text.clone());
                    print!("{}", text);
                }

                OpCode::PrintLn => {
                    let val = self.pop()?;
                    let text = format!("{}", val);
                    self.output.push(text.clone());
                    println!("{}", text);
                }

                OpCode::Len => {
                    let val = self.pop()?;
                    let length = match &val {
                        VmValue::String(s) => s.len() as i64,
                        VmValue::Array(a) => a.len() as i64,
                        VmValue::Map(m) => m.len() as i64,
                        _ => return Err(format!("Cannot get length of {:?}", val)),
                    };
                    self.push(VmValue::Int(length));
                }

                OpCode::TypeOf => {
                    let val = self.pop()?;
                    let type_name = match &val {
                        VmValue::Null => "null",
                        VmValue::Int(_) => "int",
                        VmValue::Float(_) => "float",
                        VmValue::Bool(_) => "bool",
                        VmValue::String(_) => "string",
                        VmValue::Array(_) => "array",
                        VmValue::Map(_) => "map",
                        VmValue::Struct(name, _) => name.as_str(),
                        VmValue::HeapRef(_) => "heap_ref",
                    };
                    self.push(VmValue::String(type_name.to_string()));
                }

                OpCode::Assert => {
                    let val = self.pop()?;
                    if !Self::is_truthy(&val) {
                        return Err(format!("Assertion failed: value was {:?}", val));
                    }
                }

                OpCode::Import(_) => {
                    // Placeholder – imports are resolved at compile time
                }

                OpCode::Halt => {
                    break;
                }
            }
        }

        Ok(())
    }
}

// ===========================================================================
// Unit tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::bytecode::{CompiledFunction, OpCode, OvmModule, Value};

    /// Helper: create a minimal module with a single function as entry point.
    fn make_module(name: &str, instructions: Vec<OpCode>, locals_count: usize) -> OvmModule {
        let func = CompiledFunction {
            name: name.to_string(),
            arity: 0,
            locals_count,
            instructions,
        };
        OvmModule {
            name: "test".to_string(),
            functions: vec![func],
            globals: vec![],
            entry_point: Some(0),
            constants: vec![],
        }
    }

    // -- 1. Push / Pop -------------------------------------------------------

    #[test]
    fn test_push_pop() {
        let module = make_module(
            "main",
            vec![
                OpCode::Push(Value::Int(42)),
                OpCode::Push(Value::String("hello".into())),
                OpCode::Pop,
                OpCode::Halt,
            ],
            0,
        );
        let mut vm = OmniVM::new();
        vm.execute(&module).unwrap();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack[0], VmValue::Int(42));
    }

    // -- 2. Integer arithmetic -----------------------------------------------

    #[test]
    fn test_arithmetic_int() {
        let module = make_module(
            "main",
            vec![
                // (10 + 3) => 13
                OpCode::Push(Value::Int(10)),
                OpCode::Push(Value::Int(3)),
                OpCode::Add,
                OpCode::StoreGlobal("add_result".into()),
                // (10 - 3) => 7
                OpCode::Push(Value::Int(10)),
                OpCode::Push(Value::Int(3)),
                OpCode::Sub,
                OpCode::StoreGlobal("sub_result".into()),
                // (4 * 5) => 20
                OpCode::Push(Value::Int(4)),
                OpCode::Push(Value::Int(5)),
                OpCode::Mul,
                OpCode::StoreGlobal("mul_result".into()),
                // (20 / 4) => 5
                OpCode::Push(Value::Int(20)),
                OpCode::Push(Value::Int(4)),
                OpCode::Div,
                OpCode::StoreGlobal("div_result".into()),
                // (17 % 5) => 2
                OpCode::Push(Value::Int(17)),
                OpCode::Push(Value::Int(5)),
                OpCode::Mod,
                OpCode::StoreGlobal("mod_result".into()),
                // neg(-7) => 7
                OpCode::Push(Value::Int(-7)),
                OpCode::Neg,
                OpCode::StoreGlobal("neg_result".into()),
                OpCode::Halt,
            ],
            0,
        );
        let mut vm = OmniVM::new();
        vm.execute(&module).unwrap();
        assert_eq!(vm.globals["add_result"], VmValue::Int(13));
        assert_eq!(vm.globals["sub_result"], VmValue::Int(7));
        assert_eq!(vm.globals["mul_result"], VmValue::Int(20));
        assert_eq!(vm.globals["div_result"], VmValue::Int(5));
        assert_eq!(vm.globals["mod_result"], VmValue::Int(2));
        assert_eq!(vm.globals["neg_result"], VmValue::Int(7));
    }

    // -- 3. Float arithmetic -------------------------------------------------

    #[test]
    fn test_arithmetic_float() {
        let module = make_module(
            "main",
            vec![
                OpCode::Push(Value::Float(3.5)),
                OpCode::Push(Value::Float(1.5)),
                OpCode::Add,
                OpCode::StoreGlobal("result".into()),
                OpCode::Push(Value::Float(10.0)),
                OpCode::Push(Value::Float(3.0)),
                OpCode::Div,
                OpCode::StoreGlobal("div_result".into()),
                OpCode::Halt,
            ],
            0,
        );
        let mut vm = OmniVM::new();
        vm.execute(&module).unwrap();
        assert_eq!(vm.globals["result"], VmValue::Float(5.0));
        if let VmValue::Float(v) = vm.globals["div_result"] {
            assert!((v - 3.3333333333333335).abs() < 1e-10);
        } else {
            panic!("Expected Float");
        }
    }

    // -- 4. Comparison -------------------------------------------------------

    #[test]
    fn test_comparison() {
        let module = make_module(
            "main",
            vec![
                OpCode::Push(Value::Int(5)),
                OpCode::Push(Value::Int(5)),
                OpCode::Eq,
                OpCode::StoreGlobal("eq".into()),
                OpCode::Push(Value::Int(3)),
                OpCode::Push(Value::Int(5)),
                OpCode::Lt,
                OpCode::StoreGlobal("lt".into()),
                OpCode::Push(Value::Int(5)),
                OpCode::Push(Value::Int(3)),
                OpCode::Gt,
                OpCode::StoreGlobal("gt".into()),
                OpCode::Push(Value::Int(1)),
                OpCode::Push(Value::Int(2)),
                OpCode::Ne,
                OpCode::StoreGlobal("ne".into()),
                OpCode::Halt,
            ],
            0,
        );
        let mut vm = OmniVM::new();
        vm.execute(&module).unwrap();
        assert_eq!(vm.globals["eq"], VmValue::Bool(true));
        assert_eq!(vm.globals["lt"], VmValue::Bool(true));
        assert_eq!(vm.globals["gt"], VmValue::Bool(true));
        assert_eq!(vm.globals["ne"], VmValue::Bool(true));
    }

    // -- 5. Logic ------------------------------------------------------------

    #[test]
    fn test_logic() {
        let module = make_module(
            "main",
            vec![
                OpCode::Push(Value::Bool(true)),
                OpCode::Push(Value::Bool(false)),
                OpCode::And,
                OpCode::StoreGlobal("and_result".into()),
                OpCode::Push(Value::Bool(true)),
                OpCode::Push(Value::Bool(false)),
                OpCode::Or,
                OpCode::StoreGlobal("or_result".into()),
                OpCode::Push(Value::Bool(false)),
                OpCode::Not,
                OpCode::StoreGlobal("not_result".into()),
                OpCode::Halt,
            ],
            0,
        );
        let mut vm = OmniVM::new();
        vm.execute(&module).unwrap();
        assert_eq!(vm.globals["and_result"], VmValue::Bool(false));
        assert_eq!(vm.globals["or_result"], VmValue::Bool(true));
        assert_eq!(vm.globals["not_result"], VmValue::Bool(true));
    }

    // -- 6. String concatenation ---------------------------------------------

    #[test]
    fn test_string_concat() {
        let module = make_module(
            "main",
            vec![
                // Via Concat opcode
                OpCode::Push(Value::String("Hello, ".into())),
                OpCode::Push(Value::String("World!".into())),
                OpCode::Concat,
                OpCode::StoreGlobal("concat".into()),
                // Via Add (string + string)
                OpCode::Push(Value::String("foo".into())),
                OpCode::Push(Value::String("bar".into())),
                OpCode::Add,
                OpCode::StoreGlobal("add_concat".into()),
                OpCode::Halt,
            ],
            0,
        );
        let mut vm = OmniVM::new();
        vm.execute(&module).unwrap();
        assert_eq!(
            vm.globals["concat"],
            VmValue::String("Hello, World!".into())
        );
        assert_eq!(vm.globals["add_concat"], VmValue::String("foobar".into()));
    }

    // -- 7. Local variables --------------------------------------------------

    #[test]
    fn test_local_variables() {
        let module = make_module(
            "main",
            vec![
                OpCode::Push(Value::Int(99)),
                OpCode::StoreLocal(0),
                OpCode::Push(Value::Int(1)),
                OpCode::StoreLocal(1),
                OpCode::LoadLocal(0),
                OpCode::LoadLocal(1),
                OpCode::Add,
                OpCode::StoreGlobal("sum".into()),
                OpCode::Halt,
            ],
            2,
        );
        let mut vm = OmniVM::new();
        vm.execute(&module).unwrap();
        assert_eq!(vm.globals["sum"], VmValue::Int(100));
    }

    // -- 8. Global variables -------------------------------------------------

    #[test]
    fn test_global_variables() {
        let module = make_module(
            "main",
            vec![
                OpCode::Push(Value::String("omni".into())),
                OpCode::StoreGlobal("lang".into()),
                OpCode::LoadGlobal("lang".into()),
                OpCode::StoreGlobal("copy".into()),
                // Loading an undefined global yields Null
                OpCode::LoadGlobal("undefined_var".into()),
                OpCode::StoreGlobal("undef".into()),
                OpCode::Halt,
            ],
            0,
        );
        let mut vm = OmniVM::new();
        vm.execute(&module).unwrap();
        assert_eq!(vm.globals["lang"], VmValue::String("omni".into()));
        assert_eq!(vm.globals["copy"], VmValue::String("omni".into()));
        assert_eq!(vm.globals["undef"], VmValue::Null);
    }

    // -- 9. Unconditional jump -----------------------------------------------

    #[test]
    fn test_jump() {
        // Jump over a store, so the global should NOT be set
        let module = make_module(
            "main",
            vec![
                /* 0 */ OpCode::Jump(3),
                /* 1 */ OpCode::Push(Value::Int(1)),
                /* 2 */ OpCode::StoreGlobal("skipped".into()),
                /* 3 */ OpCode::Push(Value::Int(2)),
                /* 4 */ OpCode::StoreGlobal("reached".into()),
                /* 5 */ OpCode::Halt,
            ],
            0,
        );
        let mut vm = OmniVM::new();
        vm.execute(&module).unwrap();
        assert!(!vm.globals.contains_key("skipped"));
        assert_eq!(vm.globals["reached"], VmValue::Int(2));
    }

    // -- 10. Conditional jump ------------------------------------------------

    #[test]
    fn test_conditional_jump() {
        // if true → jump to 4 (store "yes"), else fall through to store "no"
        let module = make_module(
            "main",
            vec![
                /* 0 */ OpCode::Push(Value::Bool(true)),
                /* 1 */ OpCode::JumpIf(4),
                /* 2 */ OpCode::Push(Value::String("no".into())),
                /* 3 */ OpCode::StoreGlobal("result".into()),
                /* 4 */ OpCode::Push(Value::String("yes".into())),
                /* 5 */ OpCode::StoreGlobal("result".into()),
                /* 6 */ OpCode::Halt,
            ],
            0,
        );
        let mut vm = OmniVM::new();
        vm.execute(&module).unwrap();
        assert_eq!(vm.globals["result"], VmValue::String("yes".into()));
    }

    // -- 11. Function call (CallNamed) ---------------------------------------

    #[test]
    fn test_function_call() {
        // main calls "double(21)" which returns 42
        let main_fn = CompiledFunction {
            name: "main".to_string(),
            arity: 0,
            locals_count: 0,
            instructions: vec![
                OpCode::Push(Value::Int(21)),
                OpCode::CallNamed("double".into(), 1),
                OpCode::StoreGlobal("result".into()),
                OpCode::Halt,
            ],
        };
        let double_fn = CompiledFunction {
            name: "double".to_string(),
            arity: 1,
            locals_count: 1,
            instructions: vec![
                OpCode::LoadLocal(0),
                OpCode::Push(Value::Int(2)),
                OpCode::Mul,
                OpCode::Return,
            ],
        };
        let module = OvmModule {
            name: "test".to_string(),
            functions: vec![main_fn, double_fn],
            globals: vec![],
            entry_point: Some(0),
            constants: vec![],
        };
        let mut vm = OmniVM::new();
        vm.execute(&module).unwrap();
        assert_eq!(vm.globals["result"], VmValue::Int(42));
    }

    // -- 12. Return value ----------------------------------------------------

    #[test]
    fn test_return_value() {
        // main calls "make_greeting()" → returns "hi"
        let main_fn = CompiledFunction {
            name: "main".to_string(),
            arity: 0,
            locals_count: 0,
            instructions: vec![
                OpCode::CallNamed("make_greeting".into(), 0),
                OpCode::StoreGlobal("greeting".into()),
                OpCode::Halt,
            ],
        };
        let greet_fn = CompiledFunction {
            name: "make_greeting".to_string(),
            arity: 0,
            locals_count: 0,
            instructions: vec![OpCode::Push(Value::String("hi".into())), OpCode::Return],
        };
        let module = OvmModule {
            name: "test".to_string(),
            functions: vec![main_fn, greet_fn],
            globals: vec![],
            entry_point: Some(0),
            constants: vec![],
        };
        let mut vm = OmniVM::new();
        vm.execute(&module).unwrap();
        assert_eq!(vm.globals["greeting"], VmValue::String("hi".into()));
    }

    // -- 13. Array creation --------------------------------------------------

    #[test]
    fn test_array_creation() {
        let module = make_module(
            "main",
            vec![
                OpCode::Push(Value::Int(10)),
                OpCode::Push(Value::Int(20)),
                OpCode::Push(Value::Int(30)),
                OpCode::NewArray(3),
                OpCode::StoreGlobal("arr".into()),
                // Index into the array
                OpCode::LoadGlobal("arr".into()),
                OpCode::Push(Value::Int(1)),
                OpCode::Index,
                OpCode::StoreGlobal("elem".into()),
                OpCode::Halt,
            ],
            0,
        );
        let mut vm = OmniVM::new();
        vm.execute(&module).unwrap();
        assert_eq!(
            vm.globals["arr"],
            VmValue::Array(vec![VmValue::Int(10), VmValue::Int(20), VmValue::Int(30)])
        );
        assert_eq!(vm.globals["elem"], VmValue::Int(20));
    }

    // -- 14. PrintLn ---------------------------------------------------------

    #[test]
    fn test_println() {
        let module = make_module(
            "main",
            vec![
                OpCode::Push(Value::String("Hello, VM!".into())),
                OpCode::PrintLn,
                OpCode::Push(Value::Int(42)),
                OpCode::PrintLn,
                OpCode::Halt,
            ],
            0,
        );
        let mut vm = OmniVM::new();
        vm.execute(&module).unwrap();
        assert_eq!(vm.output.len(), 2);
        assert_eq!(vm.output[0], "Hello, VM!");
        assert_eq!(vm.output[1], "42");
    }

    // -- 15. Halt ------------------------------------------------------------

    #[test]
    fn test_halt() {
        // Instructions after Halt should never execute
        let module = make_module(
            "main",
            vec![
                OpCode::Push(Value::Int(1)),
                OpCode::StoreGlobal("before".into()),
                OpCode::Halt,
                OpCode::Push(Value::Int(2)),
                OpCode::StoreGlobal("after".into()),
            ],
            0,
        );
        let mut vm = OmniVM::new();
        vm.execute(&module).unwrap();
        assert_eq!(vm.globals["before"], VmValue::Int(1));
        assert!(!vm.globals.contains_key("after"));
    }

    // -- 16. Factorial (recursive) -------------------------------------------

    #[test]
    fn test_factorial_recursive() {
        // factorial(n):
        //   0: LoadLocal 0           -- load n
        //   1: Push(Int(1))
        //   2: Le                    -- n <= 1?
        //   3: JumpIfNot(5)          -- if not, go to recursive case
        //   4: Push(Int(1))          -- base case: push 1
        //      Return
        //   5: (skipped by jump from 3)
        //   6: LoadLocal 0           -- push n
        //   7: LoadLocal 0           -- push n
        //   8: Push(Int(1))
        //   9: Sub                   -- n - 1
        //  10: CallNamed("factorial", 1)
        //  11: Mul                   -- n * factorial(n-1)
        //  12: Return

        let main_fn = CompiledFunction {
            name: "main".to_string(),
            arity: 0,
            locals_count: 0,
            instructions: vec![
                OpCode::Push(Value::Int(5)),
                OpCode::CallNamed("factorial".into(), 1),
                OpCode::StoreGlobal("result".into()),
                OpCode::Halt,
            ],
        };

        let factorial_fn = CompiledFunction {
            name: "factorial".to_string(),
            arity: 1,
            locals_count: 1,
            instructions: vec![
                /* 0 */ OpCode::LoadLocal(0),
                /* 1 */ OpCode::Push(Value::Int(1)),
                /* 2 */ OpCode::Le,
                /* 3 */ OpCode::JumpIfNot(6),
                /* 4 */ OpCode::Push(Value::Int(1)),
                /* 5 */ OpCode::Return,
                /* 6 */ OpCode::LoadLocal(0),
                /* 7 */ OpCode::LoadLocal(0),
                /* 8 */ OpCode::Push(Value::Int(1)),
                /* 9 */ OpCode::Sub,
                /* 10 */ OpCode::CallNamed("factorial".into(), 1),
                /* 11 */ OpCode::Mul,
                /* 12 */ OpCode::Return,
            ],
        };

        let module = OvmModule {
            name: "test".to_string(),
            functions: vec![main_fn, factorial_fn],
            globals: vec![],
            entry_point: Some(0),
            constants: vec![],
        };

        let mut vm = OmniVM::new();
        vm.execute(&module).unwrap();
        // 5! = 120
        assert_eq!(vm.globals["result"], VmValue::Int(120));
    }

    // -- 17. Assert ----------------------------------------------------------

    #[test]
    fn test_assert() {
        // Assert with a truthy value should pass
        let module_ok = make_module(
            "main",
            vec![
                OpCode::Push(Value::Bool(true)),
                OpCode::Assert,
                OpCode::Halt,
            ],
            0,
        );
        let mut vm = OmniVM::new();
        vm.execute(&module_ok).unwrap();

        // Assert with a falsy value should fail
        let module_fail = make_module(
            "main",
            vec![
                OpCode::Push(Value::Bool(false)),
                OpCode::Assert,
                OpCode::Halt,
            ],
            0,
        );
        let mut vm2 = OmniVM::new();
        let result = vm2.execute(&module_fail);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Assertion failed"));
    }

    // -- 18. Division by zero ------------------------------------------------

    #[test]
    fn test_division_by_zero() {
        let module = make_module(
            "main",
            vec![
                OpCode::Push(Value::Int(10)),
                OpCode::Push(Value::Int(0)),
                OpCode::Div,
                OpCode::Halt,
            ],
            0,
        );
        let mut vm = OmniVM::new();
        let result = vm.execute(&module);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Division by zero"));
    }

    // -- 19. Stack underflow -------------------------------------------------

    #[test]
    fn test_stack_underflow() {
        let module = make_module(
            "main",
            vec![
                OpCode::Pop, // nothing on the stack
                OpCode::Halt,
            ],
            0,
        );
        let mut vm = OmniVM::new();
        let result = vm.execute(&module);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Stack underflow"));
    }

    // -- 20. Dup and Swap ----------------------------------------------------

    #[test]
    fn test_dup_and_swap() {
        let module = make_module(
            "main",
            vec![
                OpCode::Push(Value::Int(1)),
                OpCode::Dup,
                OpCode::Add, // 1 + 1 = 2
                OpCode::StoreGlobal("dup_result".into()),
                OpCode::Push(Value::Int(10)),
                OpCode::Push(Value::Int(20)),
                OpCode::Swap,
                OpCode::StoreGlobal("swap_top".into()), // was bottom (10)
                OpCode::StoreGlobal("swap_bot".into()), // was top (20)
                OpCode::Halt,
            ],
            0,
        );
        let mut vm = OmniVM::new();
        vm.execute(&module).unwrap();
        assert_eq!(vm.globals["dup_result"], VmValue::Int(2));
        assert_eq!(vm.globals["swap_top"], VmValue::Int(10));
        assert_eq!(vm.globals["swap_bot"], VmValue::Int(20));
    }

    // -- 21. Len and TypeOf --------------------------------------------------

    #[test]
    fn test_len_and_typeof() {
        let module = make_module(
            "main",
            vec![
                // Len of an array
                OpCode::Push(Value::Int(1)),
                OpCode::Push(Value::Int(2)),
                OpCode::Push(Value::Int(3)),
                OpCode::NewArray(3),
                OpCode::Len,
                OpCode::StoreGlobal("arr_len".into()),
                // TypeOf
                OpCode::Push(Value::Int(42)),
                OpCode::TypeOf,
                OpCode::StoreGlobal("type_int".into()),
                OpCode::Push(Value::String("hi".into())),
                OpCode::TypeOf,
                OpCode::StoreGlobal("type_str".into()),
                OpCode::Halt,
            ],
            0,
        );
        let mut vm = OmniVM::new();
        vm.execute(&module).unwrap();
        assert_eq!(vm.globals["arr_len"], VmValue::Int(3));
        assert_eq!(vm.globals["type_int"], VmValue::String("int".into()));
        assert_eq!(vm.globals["type_str"], VmValue::String("string".into()));
    }

    // -- 22. Map creation and indexing ---------------------------------------

    #[test]
    fn test_map_creation() {
        let module = make_module(
            "main",
            vec![
                // Create a map: {"a": 1, "b": 2}
                OpCode::Push(Value::String("a".into())),
                OpCode::Push(Value::Int(1)),
                OpCode::Push(Value::String("b".into())),
                OpCode::Push(Value::Int(2)),
                OpCode::NewMap(2),
                OpCode::StoreGlobal("map".into()),
                // Index into map
                OpCode::LoadGlobal("map".into()),
                OpCode::Push(Value::String("b".into())),
                OpCode::Index,
                OpCode::StoreGlobal("val".into()),
                OpCode::Halt,
            ],
            0,
        );
        let mut vm = OmniVM::new();
        vm.execute(&module).unwrap();
        assert_eq!(vm.globals["val"], VmValue::Int(2));
    }

    // -- 23. JumpIfNot -------------------------------------------------------

    #[test]
    fn test_jump_if_not() {
        let module = make_module(
            "main",
            vec![
                /* 0 */ OpCode::Push(Value::Bool(false)),
                /* 1 */ OpCode::JumpIfNot(4),
                /* 2 */ OpCode::Push(Value::String("yes".into())),
                /* 3 */ OpCode::StoreGlobal("result".into()),
                /* 4 */ OpCode::Push(Value::String("no".into())),
                /* 5 */ OpCode::StoreGlobal("result".into()),
                /* 6 */ OpCode::Halt,
            ],
            0,
        );
        let mut vm = OmniVM::new();
        vm.execute(&module).unwrap();
        assert_eq!(vm.globals["result"], VmValue::String("no".into()));
    }

    // -- 24. GC: collect unreachable -----------------------------------------

    #[test]
    fn test_gc_collects_unreachable() {
        let mut vm = OmniVM::new();
        // Allocate 2000 objects in a loop (simulate via direct alloc)
        for _ in 0..2000 {
            let _idx = vm.alloc(HeapCell::Array(vec![]));
            // do NOT keep a reference — object is immediately unreachable
        }
        vm.gc_collect();
        let live = vm.heap.iter().filter(|c| c.is_some()).count();
        assert_eq!(live, 0, "all objects should be collected");
    }

    // -- 25. GC: keep reachable objects alive --------------------------------

    #[test]
    fn test_gc_keeps_reachable() {
        let mut vm = OmniVM::new();
        // Allocate an object and keep it on the stack
        let idx = vm.alloc(HeapCell::HeapString("keep me".to_string()));
        vm.stack.push(VmValue::HeapRef(idx));

        // Allocate unreachable objects
        for _ in 0..500 {
            let _dead = vm.alloc(HeapCell::Array(vec![]));
        }

        vm.gc_collect();

        // The reachable object should survive
        assert!(vm.heap[idx].is_some());
        // All others should be collected
        let live = vm.heap.iter().filter(|c| c.is_some()).count();
        assert_eq!(live, 1, "only the referenced object should survive");
    }

    // -- 26. GC: transitive references preserved -----------------------------

    #[test]
    fn test_gc_transitive_references() {
        let mut vm = OmniVM::new();

        // Create a chain: root -> child -> grandchild
        let grandchild = vm.alloc(HeapCell::HeapString("grandchild".to_string()));
        let child = vm.alloc(HeapCell::Array(vec![VmValue::HeapRef(grandchild)]));
        let root = vm.alloc(HeapCell::Struct(
            "Node".to_string(),
            vec![("child".to_string(), VmValue::HeapRef(child))],
        ));

        // Only the root is on the stack
        vm.stack.push(VmValue::HeapRef(root));

        // Allocate garbage
        for _ in 0..200 {
            let _dead = vm.alloc(HeapCell::Array(vec![]));
        }

        vm.gc_collect();

        // All three should survive (transitive mark)
        assert!(vm.heap[root].is_some(), "root should survive");
        assert!(vm.heap[child].is_some(), "child should survive");
        assert!(vm.heap[grandchild].is_some(), "grandchild should survive");

        // Only 3 live objects
        let live = vm.heap.iter().filter(|c| c.is_some()).count();
        assert_eq!(live, 3);
    }

    // -- 27. GC: free-list reuse ---------------------------------------------

    #[test]
    fn test_gc_free_list_reuse() {
        let mut vm = OmniVM::new();
        vm.gc_threshold = usize::MAX; // Disable auto-GC

        // Allocate and immediately discard
        let idx0 = vm.alloc(HeapCell::Array(vec![]));
        let idx1 = vm.alloc(HeapCell::Array(vec![]));

        vm.gc_collect();
        assert_eq!(vm.free_list.len(), 2);

        // Next allocation should reuse a free slot
        let reused = vm.alloc(HeapCell::HeapString("reused".to_string()));
        assert!(
            reused == idx0 || reused == idx1,
            "should reuse a freed slot"
        );
    }

    // -- 28. GC: globals as roots --------------------------------------------

    #[test]
    fn test_gc_globals_as_roots() {
        let mut vm = OmniVM::new();
        let idx = vm.alloc(HeapCell::HeapString("global obj".to_string()));
        vm.globals
            .insert("my_global".to_string(), VmValue::HeapRef(idx));

        // Allocate garbage
        for _ in 0..100 {
            let _dead = vm.alloc(HeapCell::Array(vec![]));
        }

        vm.gc_collect();

        assert!(
            vm.heap[idx].is_some(),
            "object referenced from globals should survive"
        );
        let live = vm.heap.iter().filter(|c| c.is_some()).count();
        assert_eq!(live, 1);
    }

    // -- 29. GC: threshold doubles after collection --------------------------

    #[test]
    fn test_gc_threshold_doubles() {
        let mut vm = OmniVM::new();
        let initial = vm.gc_threshold;
        vm.gc_collect();
        assert_eq!(vm.gc_threshold, initial * 2);
        vm.gc_collect();
        assert_eq!(vm.gc_threshold, initial * 4);
    }
}
