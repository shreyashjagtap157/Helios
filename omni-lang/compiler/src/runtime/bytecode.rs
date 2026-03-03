#![allow(dead_code)]
//! OVM Bytecode Format
//!
//! Defines the instruction set, value representation, and binary serialization
//! format for the Omni Virtual Machine (OVM). This module provides a high-level
//! bytecode representation that sits between the AST and the low-level codegen
//! OVM opcodes, suitable for interpretation and ahead-of-time compilation.

use std::fmt;
use anyhow::{Result, anyhow, Context};

// ---------------------------------------------------------------------------
// Magic & version constants
// ---------------------------------------------------------------------------

/// Magic bytes identifying an OVM binary module: "OVM\x01"
pub const OVM_MAGIC: [u8; 4] = [b'O', b'V', b'M', 0x01];

/// Current bytecode format version
pub const OVM_VERSION: u16 = 1;

// ---------------------------------------------------------------------------
// OpCode – the instruction set
// ---------------------------------------------------------------------------

/// OVM Instruction Set
///
/// Each variant corresponds to a single VM operation. Operands are encoded
/// inline in the enum so that the compiler can produce a `Vec<OpCode>` without
/// needing a separate operand stream.
#[derive(Debug, Clone, PartialEq)]
pub enum OpCode {
    // -- Stack operations --
    /// No operation
    Nop,
    /// Push a literal value onto the stack
    Push(Value),
    /// Pop (and discard) the top of the stack
    Pop,
    /// Duplicate the top value
    Dup,
    /// Swap the two topmost values
    Swap,

    // -- Arithmetic --
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neg,

    // -- Comparison --
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // -- Logic --
    And,
    Or,
    Not,

    // -- String --
    /// Pop two strings, push their concatenation
    Concat,

    // -- Variables --
    /// Load a value from local slot `n`
    LoadLocal(usize),
    /// Store top-of-stack into local slot `n`
    StoreLocal(usize),
    /// Load a global variable by name
    LoadGlobal(String),
    /// Store into a global variable by name
    StoreGlobal(String),
    /// Load a field from the struct on top of the stack
    LoadField(String),
    /// Store into a field of the struct on top of the stack
    StoreField(String),

    // -- Control flow --
    /// Unconditional jump to instruction index
    Jump(usize),
    /// Jump if top of stack is truthy
    JumpIf(usize),
    /// Jump if top of stack is falsy
    JumpIfNot(usize),

    // -- Functions --
    /// Call the function on top of stack with `n` arguments
    Call(usize),
    /// Call a named function with `n` arguments
    CallNamed(String, usize),
    /// Return from the current function
    Return,

    // -- Composite constructors --
    /// Construct a struct named `name` from `n` field values on the stack
    NewStruct(String, usize),
    /// Construct an array from `n` elements on the stack
    NewArray(usize),
    /// Construct a map from `n` key-value pairs on the stack (2n values)
    NewMap(usize),
    /// Index into an array or map (pops index, then collection)
    Index,

    // -- Built-in operations --
    Print,
    PrintLn,
    Len,
    TypeOf,
    Assert,

    // -- Module --
    /// Import a module by path
    Import(String),

    // -- Special --
    /// Halt the VM
    Halt,
}

// ---------------------------------------------------------------------------
// Display for OpCode
// ---------------------------------------------------------------------------

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpCode::Nop => write!(f, "NOP"),
            OpCode::Push(v) => write!(f, "PUSH {}", v),
            OpCode::Pop => write!(f, "POP"),
            OpCode::Dup => write!(f, "DUP"),
            OpCode::Swap => write!(f, "SWAP"),
            OpCode::Add => write!(f, "ADD"),
            OpCode::Sub => write!(f, "SUB"),
            OpCode::Mul => write!(f, "MUL"),
            OpCode::Div => write!(f, "DIV"),
            OpCode::Mod => write!(f, "MOD"),
            OpCode::Neg => write!(f, "NEG"),
            OpCode::Eq => write!(f, "EQ"),
            OpCode::Ne => write!(f, "NE"),
            OpCode::Lt => write!(f, "LT"),
            OpCode::Le => write!(f, "LE"),
            OpCode::Gt => write!(f, "GT"),
            OpCode::Ge => write!(f, "GE"),
            OpCode::And => write!(f, "AND"),
            OpCode::Or => write!(f, "OR"),
            OpCode::Not => write!(f, "NOT"),
            OpCode::Concat => write!(f, "CONCAT"),
            OpCode::LoadLocal(s) => write!(f, "LOAD_LOCAL {}", s),
            OpCode::StoreLocal(s) => write!(f, "STORE_LOCAL {}", s),
            OpCode::LoadGlobal(n) => write!(f, "LOAD_GLOBAL \"{}\"", n),
            OpCode::StoreGlobal(n) => write!(f, "STORE_GLOBAL \"{}\"", n),
            OpCode::LoadField(n) => write!(f, "LOAD_FIELD \"{}\"", n),
            OpCode::StoreField(n) => write!(f, "STORE_FIELD \"{}\"", n),
            OpCode::Jump(a) => write!(f, "JUMP {}", a),
            OpCode::JumpIf(a) => write!(f, "JUMP_IF {}", a),
            OpCode::JumpIfNot(a) => write!(f, "JUMP_IF_NOT {}", a),
            OpCode::Call(n) => write!(f, "CALL {}", n),
            OpCode::CallNamed(name, n) => write!(f, "CALL_NAMED \"{}\" {}", name, n),
            OpCode::Return => write!(f, "RETURN"),
            OpCode::NewStruct(name, n) => write!(f, "NEW_STRUCT \"{}\" {}", name, n),
            OpCode::NewArray(n) => write!(f, "NEW_ARRAY {}", n),
            OpCode::NewMap(n) => write!(f, "NEW_MAP {}", n),
            OpCode::Index => write!(f, "INDEX"),
            OpCode::Print => write!(f, "PRINT"),
            OpCode::PrintLn => write!(f, "PRINTLN"),
            OpCode::Len => write!(f, "LEN"),
            OpCode::TypeOf => write!(f, "TYPEOF"),
            OpCode::Assert => write!(f, "ASSERT"),
            OpCode::Import(p) => write!(f, "IMPORT \"{}\"", p),
            OpCode::Halt => write!(f, "HALT"),
        }
    }
}

// ---------------------------------------------------------------------------
// Value – runtime stack values
// ---------------------------------------------------------------------------

/// OVM stack value
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(v) => write!(f, "{}", v),
            Value::Bool(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "\"{}\"", s),
        }
    }
}

// ---------------------------------------------------------------------------
// CompiledFunction
// ---------------------------------------------------------------------------

/// A single compiled function inside an OVM module.
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledFunction {
    /// Function name (e.g. `"main"`)
    pub name: String,
    /// Number of parameters
    pub arity: usize,
    /// Total number of local variable slots (including parameters)
    pub locals_count: usize,
    /// The instruction stream
    pub instructions: Vec<OpCode>,
}

impl CompiledFunction {
    pub fn new(name: impl Into<String>, arity: usize) -> Self {
        Self {
            name: name.into(),
            arity,
            locals_count: arity,
            instructions: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// OvmModule – the top-level bytecode container
// ---------------------------------------------------------------------------

/// A compiled OVM module – the unit of serialisation that is written to `.ovm`
/// files.
#[derive(Debug, Clone, PartialEq)]
pub struct OvmModule {
    /// Module name
    pub name: String,
    /// All compiled functions
    pub functions: Vec<CompiledFunction>,
    /// Global variable names (in declaration order)
    pub globals: Vec<String>,
    /// Index into `functions` for the entry-point (typically `main`)
    pub entry_point: Option<usize>,
    /// Constant pool
    pub constants: Vec<Value>,
}

impl OvmModule {
    /// Create a new, empty module.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            functions: Vec::new(),
            globals: Vec::new(),
            entry_point: None,
            constants: Vec::new(),
        }
    }

    // -----------------------------------------------------------------------
    // Binary serialisation helpers
    // -----------------------------------------------------------------------

    fn write_u8(buf: &mut Vec<u8>, v: u8) {
        buf.push(v);
    }

    fn write_u16(buf: &mut Vec<u8>, v: u16) {
        buf.extend_from_slice(&v.to_le_bytes());
    }

    fn write_u32(buf: &mut Vec<u8>, v: u32) {
        buf.extend_from_slice(&v.to_le_bytes());
    }

    fn write_u64(buf: &mut Vec<u8>, v: u64) {
        buf.extend_from_slice(&v.to_le_bytes());
    }

    fn write_i64(buf: &mut Vec<u8>, v: i64) {
        buf.extend_from_slice(&v.to_le_bytes());
    }

    fn write_f64(buf: &mut Vec<u8>, v: f64) {
        buf.extend_from_slice(&v.to_le_bytes());
    }

    fn write_string(buf: &mut Vec<u8>, s: &str) {
        let bytes = s.as_bytes();
        Self::write_u32(buf, bytes.len() as u32);
        buf.extend_from_slice(bytes);
    }

    fn write_usize(buf: &mut Vec<u8>, v: usize) {
        Self::write_u64(buf, v as u64);
    }

    // -----------------------------------------------------------------------
    // Deserialisation helpers
    // -----------------------------------------------------------------------

    fn read_u8(bytes: &[u8], pos: &mut usize) -> Result<u8> {
        if *pos >= bytes.len() {
            return Err(anyhow!("unexpected end of bytecode reading u8"));
        }
        let v = bytes[*pos];
        *pos += 1;
        Ok(v)
    }

    fn read_u16(bytes: &[u8], pos: &mut usize) -> Result<u16> {
        if *pos + 2 > bytes.len() {
            return Err(anyhow!("unexpected end of bytecode reading u16"));
        }
        let v = u16::from_le_bytes([bytes[*pos], bytes[*pos + 1]]);
        *pos += 2;
        Ok(v)
    }

    fn read_u32(bytes: &[u8], pos: &mut usize) -> Result<u32> {
        if *pos + 4 > bytes.len() {
            return Err(anyhow!("unexpected end of bytecode reading u32"));
        }
        let v = u32::from_le_bytes(bytes[*pos..*pos + 4].try_into().unwrap());
        *pos += 4;
        Ok(v)
    }

    fn read_u64(bytes: &[u8], pos: &mut usize) -> Result<u64> {
        if *pos + 8 > bytes.len() {
            return Err(anyhow!("unexpected end of bytecode reading u64"));
        }
        let v = u64::from_le_bytes(bytes[*pos..*pos + 8].try_into().unwrap());
        *pos += 8;
        Ok(v)
    }

    fn read_i64(bytes: &[u8], pos: &mut usize) -> Result<i64> {
        if *pos + 8 > bytes.len() {
            return Err(anyhow!("unexpected end of bytecode reading i64"));
        }
        let v = i64::from_le_bytes(bytes[*pos..*pos + 8].try_into().unwrap());
        *pos += 8;
        Ok(v)
    }

    fn read_f64(bytes: &[u8], pos: &mut usize) -> Result<f64> {
        if *pos + 8 > bytes.len() {
            return Err(anyhow!("unexpected end of bytecode reading f64"));
        }
        let v = f64::from_le_bytes(bytes[*pos..*pos + 8].try_into().unwrap());
        *pos += 8;
        Ok(v)
    }

    fn read_string(bytes: &[u8], pos: &mut usize) -> Result<String> {
        let len = Self::read_u32(bytes, pos)? as usize;
        if *pos + len > bytes.len() {
            return Err(anyhow!("unexpected end of bytecode reading string of len {}", len));
        }
        let s = std::str::from_utf8(&bytes[*pos..*pos + len])
            .context("invalid UTF-8 in bytecode string")?
            .to_string();
        *pos += len;
        Ok(s)
    }

    fn read_usize(bytes: &[u8], pos: &mut usize) -> Result<usize> {
        Ok(Self::read_u64(bytes, pos)? as usize)
    }

    // -----------------------------------------------------------------------
    // Value serialisation
    // -----------------------------------------------------------------------

    /// Value type tags used in the binary format.
    const TAG_NULL: u8 = 0;
    const TAG_INT: u8 = 1;
    const TAG_FLOAT: u8 = 2;
    const TAG_BOOL: u8 = 3;
    const TAG_STRING: u8 = 4;

    fn write_value(buf: &mut Vec<u8>, v: &Value) {
        match v {
            Value::Null => Self::write_u8(buf, Self::TAG_NULL),
            Value::Int(i) => {
                Self::write_u8(buf, Self::TAG_INT);
                Self::write_i64(buf, *i);
            }
            Value::Float(f) => {
                Self::write_u8(buf, Self::TAG_FLOAT);
                Self::write_f64(buf, *f);
            }
            Value::Bool(b) => {
                Self::write_u8(buf, Self::TAG_BOOL);
                Self::write_u8(buf, if *b { 1 } else { 0 });
            }
            Value::String(s) => {
                Self::write_u8(buf, Self::TAG_STRING);
                Self::write_string(buf, s);
            }
        }
    }

    fn read_value(bytes: &[u8], pos: &mut usize) -> Result<Value> {
        let tag = Self::read_u8(bytes, pos)?;
        match tag {
            Self::TAG_NULL => Ok(Value::Null),
            Self::TAG_INT => Ok(Value::Int(Self::read_i64(bytes, pos)?)),
            Self::TAG_FLOAT => Ok(Value::Float(Self::read_f64(bytes, pos)?)),
            Self::TAG_BOOL => {
                let b = Self::read_u8(bytes, pos)?;
                Ok(Value::Bool(b != 0))
            }
            Self::TAG_STRING => Ok(Value::String(Self::read_string(bytes, pos)?)),
            _ => Err(anyhow!("unknown value tag 0x{:02x}", tag)),
        }
    }

    // -----------------------------------------------------------------------
    // OpCode serialisation
    // -----------------------------------------------------------------------

    // Opcode tags – one byte per opcode kind.
    const OP_NOP: u8 = 0x00;
    const OP_PUSH: u8 = 0x01;
    const OP_POP: u8 = 0x02;
    const OP_DUP: u8 = 0x03;
    const OP_SWAP: u8 = 0x04;
    const OP_ADD: u8 = 0x10;
    const OP_SUB: u8 = 0x11;
    const OP_MUL: u8 = 0x12;
    const OP_DIV: u8 = 0x13;
    const OP_MOD: u8 = 0x14;
    const OP_NEG: u8 = 0x15;
    const OP_EQ: u8 = 0x20;
    const OP_NE: u8 = 0x21;
    const OP_LT: u8 = 0x22;
    const OP_LE: u8 = 0x23;
    const OP_GT: u8 = 0x24;
    const OP_GE: u8 = 0x25;
    const OP_AND: u8 = 0x30;
    const OP_OR: u8 = 0x31;
    const OP_NOT: u8 = 0x32;
    const OP_CONCAT: u8 = 0x33;
    const OP_LOAD_LOCAL: u8 = 0x40;
    const OP_STORE_LOCAL: u8 = 0x41;
    const OP_LOAD_GLOBAL: u8 = 0x42;
    const OP_STORE_GLOBAL: u8 = 0x43;
    const OP_LOAD_FIELD: u8 = 0x44;
    const OP_STORE_FIELD: u8 = 0x45;
    const OP_JUMP: u8 = 0x50;
    const OP_JUMP_IF: u8 = 0x51;
    const OP_JUMP_IF_NOT: u8 = 0x52;
    const OP_CALL: u8 = 0x60;
    const OP_CALL_NAMED: u8 = 0x61;
    const OP_RETURN: u8 = 0x62;
    const OP_NEW_STRUCT: u8 = 0x70;
    const OP_NEW_ARRAY: u8 = 0x71;
    const OP_NEW_MAP: u8 = 0x72;
    const OP_INDEX: u8 = 0x73;
    const OP_PRINT: u8 = 0x80;
    const OP_PRINTLN: u8 = 0x81;
    const OP_LEN: u8 = 0x82;
    const OP_TYPEOF: u8 = 0x83;
    const OP_ASSERT: u8 = 0x84;
    const OP_IMPORT: u8 = 0x90;
    const OP_HALT: u8 = 0xFF;

    fn write_opcode(buf: &mut Vec<u8>, op: &OpCode) {
        match op {
            OpCode::Nop => Self::write_u8(buf, Self::OP_NOP),
            OpCode::Push(v) => {
                Self::write_u8(buf, Self::OP_PUSH);
                Self::write_value(buf, v);
            }
            OpCode::Pop => Self::write_u8(buf, Self::OP_POP),
            OpCode::Dup => Self::write_u8(buf, Self::OP_DUP),
            OpCode::Swap => Self::write_u8(buf, Self::OP_SWAP),
            OpCode::Add => Self::write_u8(buf, Self::OP_ADD),
            OpCode::Sub => Self::write_u8(buf, Self::OP_SUB),
            OpCode::Mul => Self::write_u8(buf, Self::OP_MUL),
            OpCode::Div => Self::write_u8(buf, Self::OP_DIV),
            OpCode::Mod => Self::write_u8(buf, Self::OP_MOD),
            OpCode::Neg => Self::write_u8(buf, Self::OP_NEG),
            OpCode::Eq => Self::write_u8(buf, Self::OP_EQ),
            OpCode::Ne => Self::write_u8(buf, Self::OP_NE),
            OpCode::Lt => Self::write_u8(buf, Self::OP_LT),
            OpCode::Le => Self::write_u8(buf, Self::OP_LE),
            OpCode::Gt => Self::write_u8(buf, Self::OP_GT),
            OpCode::Ge => Self::write_u8(buf, Self::OP_GE),
            OpCode::And => Self::write_u8(buf, Self::OP_AND),
            OpCode::Or => Self::write_u8(buf, Self::OP_OR),
            OpCode::Not => Self::write_u8(buf, Self::OP_NOT),
            OpCode::Concat => Self::write_u8(buf, Self::OP_CONCAT),
            OpCode::LoadLocal(s) => {
                Self::write_u8(buf, Self::OP_LOAD_LOCAL);
                Self::write_usize(buf, *s);
            }
            OpCode::StoreLocal(s) => {
                Self::write_u8(buf, Self::OP_STORE_LOCAL);
                Self::write_usize(buf, *s);
            }
            OpCode::LoadGlobal(n) => {
                Self::write_u8(buf, Self::OP_LOAD_GLOBAL);
                Self::write_string(buf, n);
            }
            OpCode::StoreGlobal(n) => {
                Self::write_u8(buf, Self::OP_STORE_GLOBAL);
                Self::write_string(buf, n);
            }
            OpCode::LoadField(n) => {
                Self::write_u8(buf, Self::OP_LOAD_FIELD);
                Self::write_string(buf, n);
            }
            OpCode::StoreField(n) => {
                Self::write_u8(buf, Self::OP_STORE_FIELD);
                Self::write_string(buf, n);
            }
            OpCode::Jump(a) => {
                Self::write_u8(buf, Self::OP_JUMP);
                Self::write_usize(buf, *a);
            }
            OpCode::JumpIf(a) => {
                Self::write_u8(buf, Self::OP_JUMP_IF);
                Self::write_usize(buf, *a);
            }
            OpCode::JumpIfNot(a) => {
                Self::write_u8(buf, Self::OP_JUMP_IF_NOT);
                Self::write_usize(buf, *a);
            }
            OpCode::Call(n) => {
                Self::write_u8(buf, Self::OP_CALL);
                Self::write_usize(buf, *n);
            }
            OpCode::CallNamed(name, n) => {
                Self::write_u8(buf, Self::OP_CALL_NAMED);
                Self::write_string(buf, name);
                Self::write_usize(buf, *n);
            }
            OpCode::Return => Self::write_u8(buf, Self::OP_RETURN),
            OpCode::NewStruct(name, n) => {
                Self::write_u8(buf, Self::OP_NEW_STRUCT);
                Self::write_string(buf, name);
                Self::write_usize(buf, *n);
            }
            OpCode::NewArray(n) => {
                Self::write_u8(buf, Self::OP_NEW_ARRAY);
                Self::write_usize(buf, *n);
            }
            OpCode::NewMap(n) => {
                Self::write_u8(buf, Self::OP_NEW_MAP);
                Self::write_usize(buf, *n);
            }
            OpCode::Index => Self::write_u8(buf, Self::OP_INDEX),
            OpCode::Print => Self::write_u8(buf, Self::OP_PRINT),
            OpCode::PrintLn => Self::write_u8(buf, Self::OP_PRINTLN),
            OpCode::Len => Self::write_u8(buf, Self::OP_LEN),
            OpCode::TypeOf => Self::write_u8(buf, Self::OP_TYPEOF),
            OpCode::Assert => Self::write_u8(buf, Self::OP_ASSERT),
            OpCode::Import(p) => {
                Self::write_u8(buf, Self::OP_IMPORT);
                Self::write_string(buf, p);
            }
            OpCode::Halt => Self::write_u8(buf, Self::OP_HALT),
        }
    }

    fn read_opcode(bytes: &[u8], pos: &mut usize) -> Result<OpCode> {
        let tag = Self::read_u8(bytes, pos)?;
        match tag {
            Self::OP_NOP => Ok(OpCode::Nop),
            Self::OP_PUSH => Ok(OpCode::Push(Self::read_value(bytes, pos)?)),
            Self::OP_POP => Ok(OpCode::Pop),
            Self::OP_DUP => Ok(OpCode::Dup),
            Self::OP_SWAP => Ok(OpCode::Swap),
            Self::OP_ADD => Ok(OpCode::Add),
            Self::OP_SUB => Ok(OpCode::Sub),
            Self::OP_MUL => Ok(OpCode::Mul),
            Self::OP_DIV => Ok(OpCode::Div),
            Self::OP_MOD => Ok(OpCode::Mod),
            Self::OP_NEG => Ok(OpCode::Neg),
            Self::OP_EQ => Ok(OpCode::Eq),
            Self::OP_NE => Ok(OpCode::Ne),
            Self::OP_LT => Ok(OpCode::Lt),
            Self::OP_LE => Ok(OpCode::Le),
            Self::OP_GT => Ok(OpCode::Gt),
            Self::OP_GE => Ok(OpCode::Ge),
            Self::OP_AND => Ok(OpCode::And),
            Self::OP_OR => Ok(OpCode::Or),
            Self::OP_NOT => Ok(OpCode::Not),
            Self::OP_CONCAT => Ok(OpCode::Concat),
            Self::OP_LOAD_LOCAL => Ok(OpCode::LoadLocal(Self::read_usize(bytes, pos)?)),
            Self::OP_STORE_LOCAL => Ok(OpCode::StoreLocal(Self::read_usize(bytes, pos)?)),
            Self::OP_LOAD_GLOBAL => Ok(OpCode::LoadGlobal(Self::read_string(bytes, pos)?)),
            Self::OP_STORE_GLOBAL => Ok(OpCode::StoreGlobal(Self::read_string(bytes, pos)?)),
            Self::OP_LOAD_FIELD => Ok(OpCode::LoadField(Self::read_string(bytes, pos)?)),
            Self::OP_STORE_FIELD => Ok(OpCode::StoreField(Self::read_string(bytes, pos)?)),
            Self::OP_JUMP => Ok(OpCode::Jump(Self::read_usize(bytes, pos)?)),
            Self::OP_JUMP_IF => Ok(OpCode::JumpIf(Self::read_usize(bytes, pos)?)),
            Self::OP_JUMP_IF_NOT => Ok(OpCode::JumpIfNot(Self::read_usize(bytes, pos)?)),
            Self::OP_CALL => Ok(OpCode::Call(Self::read_usize(bytes, pos)?)),
            Self::OP_CALL_NAMED => {
                let name = Self::read_string(bytes, pos)?;
                let n = Self::read_usize(bytes, pos)?;
                Ok(OpCode::CallNamed(name, n))
            }
            Self::OP_RETURN => Ok(OpCode::Return),
            Self::OP_NEW_STRUCT => {
                let name = Self::read_string(bytes, pos)?;
                let n = Self::read_usize(bytes, pos)?;
                Ok(OpCode::NewStruct(name, n))
            }
            Self::OP_NEW_ARRAY => Ok(OpCode::NewArray(Self::read_usize(bytes, pos)?)),
            Self::OP_NEW_MAP => Ok(OpCode::NewMap(Self::read_usize(bytes, pos)?)),
            Self::OP_INDEX => Ok(OpCode::Index),
            Self::OP_PRINT => Ok(OpCode::Print),
            Self::OP_PRINTLN => Ok(OpCode::PrintLn),
            Self::OP_LEN => Ok(OpCode::Len),
            Self::OP_TYPEOF => Ok(OpCode::TypeOf),
            Self::OP_ASSERT => Ok(OpCode::Assert),
            Self::OP_IMPORT => Ok(OpCode::Import(Self::read_string(bytes, pos)?)),
            Self::OP_HALT => Ok(OpCode::Halt),
            _ => Err(anyhow!("unknown opcode tag 0x{:02x} at position {}", tag, *pos - 1)),
        }
    }

    // -----------------------------------------------------------------------
    // Function serialisation
    // -----------------------------------------------------------------------

    fn write_function(buf: &mut Vec<u8>, func: &CompiledFunction) {
        Self::write_string(buf, &func.name);
        Self::write_usize(buf, func.arity);
        Self::write_usize(buf, func.locals_count);
        Self::write_u32(buf, func.instructions.len() as u32);
        for op in &func.instructions {
            Self::write_opcode(buf, op);
        }
    }

    fn read_function(bytes: &[u8], pos: &mut usize) -> Result<CompiledFunction> {
        let name = Self::read_string(bytes, pos)?;
        let arity = Self::read_usize(bytes, pos)?;
        let locals_count = Self::read_usize(bytes, pos)?;
        let instr_count = Self::read_u32(bytes, pos)? as usize;
        let mut instructions = Vec::with_capacity(instr_count);
        for _ in 0..instr_count {
            instructions.push(Self::read_opcode(bytes, pos)?);
        }
        Ok(CompiledFunction {
            name,
            arity,
            locals_count,
            instructions,
        })
    }

    // -----------------------------------------------------------------------
    // Public API: serialize / deserialize
    // -----------------------------------------------------------------------

    /// Serialize this module to the OVM binary format.
    ///
    /// Binary layout:
    /// ```text
    /// [magic:4] [version:2]
    /// [module_name:str]
    /// [entry_point_present:1] [entry_point:8]?
    /// [num_constants:4] [constants...]
    /// [num_globals:4] [globals...]
    /// [num_functions:4] [functions...]
    /// ```
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(1024);

        // Header
        buf.extend_from_slice(&OVM_MAGIC);
        Self::write_u16(&mut buf, OVM_VERSION);

        // Module name
        Self::write_string(&mut buf, &self.name);

        // Entry point
        match self.entry_point {
            Some(ep) => {
                Self::write_u8(&mut buf, 1);
                Self::write_usize(&mut buf, ep);
            }
            None => {
                Self::write_u8(&mut buf, 0);
            }
        }

        // Constants
        Self::write_u32(&mut buf, self.constants.len() as u32);
        for c in &self.constants {
            Self::write_value(&mut buf, c);
        }

        // Globals
        Self::write_u32(&mut buf, self.globals.len() as u32);
        for g in &self.globals {
            Self::write_string(&mut buf, g);
        }

        // Functions
        Self::write_u32(&mut buf, self.functions.len() as u32);
        for f in &self.functions {
            Self::write_function(&mut buf, f);
        }

        buf
    }

    /// Deserialize an OVM module from bytes.
    pub fn deserialize(bytes: &[u8]) -> Result<Self> {
        let mut pos: usize = 0;

        // Magic
        if bytes.len() < 6 {
            return Err(anyhow!("bytecode too short for header"));
        }
        if &bytes[0..4] != &OVM_MAGIC {
            return Err(anyhow!("invalid OVM magic bytes"));
        }
        pos = 4;

        // Version
        let version = Self::read_u16(bytes, &mut pos)?;
        if version != OVM_VERSION {
            return Err(anyhow!("unsupported OVM version {}", version));
        }

        // Module name
        let name = Self::read_string(bytes, &mut pos)?;

        // Entry point
        let ep_flag = Self::read_u8(bytes, &mut pos)?;
        let entry_point = if ep_flag != 0 {
            Some(Self::read_usize(bytes, &mut pos)?)
        } else {
            None
        };

        // Constants
        let num_constants = Self::read_u32(bytes, &mut pos)? as usize;
        let mut constants = Vec::with_capacity(num_constants);
        for _ in 0..num_constants {
            constants.push(Self::read_value(bytes, &mut pos)?);
        }

        // Globals
        let num_globals = Self::read_u32(bytes, &mut pos)? as usize;
        let mut globals = Vec::with_capacity(num_globals);
        for _ in 0..num_globals {
            globals.push(Self::read_string(bytes, &mut pos)?);
        }

        // Functions
        let num_functions = Self::read_u32(bytes, &mut pos)? as usize;
        let mut functions = Vec::with_capacity(num_functions);
        for _ in 0..num_functions {
            functions.push(Self::read_function(bytes, &mut pos)?);
        }

        Ok(OvmModule {
            name,
            functions,
            globals,
            entry_point,
            constants,
        })
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: roundtrip a module through serialize/deserialize.
    fn roundtrip(module: &OvmModule) -> OvmModule {
        let bytes = module.serialize();
        OvmModule::deserialize(&bytes).expect("deserialization failed")
    }

    #[test]
    fn test_empty_module_roundtrip() {
        let m = OvmModule::new("empty");
        let m2 = roundtrip(&m);
        assert_eq!(m.name, m2.name);
        assert_eq!(m.entry_point, m2.entry_point);
        assert!(m2.functions.is_empty());
        assert!(m2.globals.is_empty());
        assert!(m2.constants.is_empty());
    }

    #[test]
    fn test_module_with_entry_point() {
        let mut m = OvmModule::new("with_ep");
        m.entry_point = Some(42);
        let m2 = roundtrip(&m);
        assert_eq!(m2.entry_point, Some(42));
    }

    #[test]
    fn test_constants_roundtrip() {
        let mut m = OvmModule::new("consts");
        m.constants = vec![
            Value::Null,
            Value::Int(42),
            Value::Int(-999),
            Value::Float(3.14),
            Value::Bool(true),
            Value::Bool(false),
            Value::String("hello world".into()),
        ];
        let m2 = roundtrip(&m);
        assert_eq!(m.constants, m2.constants);
    }

    #[test]
    fn test_globals_roundtrip() {
        let mut m = OvmModule::new("globs");
        m.globals = vec!["x".into(), "counter".into(), "name".into()];
        let m2 = roundtrip(&m);
        assert_eq!(m.globals, m2.globals);
    }

    #[test]
    fn test_simple_function_roundtrip() {
        let mut m = OvmModule::new("funcs");
        m.functions.push(CompiledFunction {
            name: "add".into(),
            arity: 2,
            locals_count: 3,
            instructions: vec![
                OpCode::LoadLocal(0),
                OpCode::LoadLocal(1),
                OpCode::Add,
                OpCode::Return,
            ],
        });
        let m2 = roundtrip(&m);
        assert_eq!(m.functions.len(), m2.functions.len());
        assert_eq!(m.functions[0], m2.functions[0]);
    }

    #[test]
    fn test_all_stack_ops_roundtrip() {
        let mut m = OvmModule::new("stack_ops");
        m.functions.push(CompiledFunction {
            name: "stack_test".into(),
            arity: 0,
            locals_count: 0,
            instructions: vec![
                OpCode::Nop,
                OpCode::Push(Value::Int(10)),
                OpCode::Push(Value::Float(2.5)),
                OpCode::Push(Value::Bool(true)),
                OpCode::Push(Value::String("hi".into())),
                OpCode::Push(Value::Null),
                OpCode::Dup,
                OpCode::Swap,
                OpCode::Pop,
                OpCode::Halt,
            ],
        });
        let m2 = roundtrip(&m);
        assert_eq!(m.functions[0].instructions, m2.functions[0].instructions);
    }

    #[test]
    fn test_arithmetic_and_comparison_roundtrip() {
        let mut m = OvmModule::new("arith");
        m.functions.push(CompiledFunction {
            name: "calc".into(),
            arity: 0,
            locals_count: 0,
            instructions: vec![
                OpCode::Add, OpCode::Sub, OpCode::Mul, OpCode::Div,
                OpCode::Mod, OpCode::Neg,
                OpCode::Eq, OpCode::Ne, OpCode::Lt, OpCode::Le,
                OpCode::Gt, OpCode::Ge,
                OpCode::And, OpCode::Or, OpCode::Not,
                OpCode::Concat,
            ],
        });
        let m2 = roundtrip(&m);
        assert_eq!(m.functions[0].instructions, m2.functions[0].instructions);
    }

    #[test]
    fn test_control_flow_roundtrip() {
        let mut m = OvmModule::new("ctrl");
        m.functions.push(CompiledFunction {
            name: "branch".into(),
            arity: 0,
            locals_count: 2,
            instructions: vec![
                OpCode::Push(Value::Bool(true)),
                OpCode::JumpIfNot(5),
                OpCode::Push(Value::Int(1)),
                OpCode::StoreLocal(0),
                OpCode::Jump(7),
                OpCode::Push(Value::Int(2)),
                OpCode::StoreLocal(0),
                OpCode::LoadLocal(0),
                OpCode::Return,
            ],
        });
        let m2 = roundtrip(&m);
        assert_eq!(m.functions[0].instructions, m2.functions[0].instructions);
    }

    #[test]
    fn test_named_ops_roundtrip() {
        let mut m = OvmModule::new("named");
        m.functions.push(CompiledFunction {
            name: "named_ops".into(),
            arity: 0,
            locals_count: 0,
            instructions: vec![
                OpCode::LoadGlobal("counter".into()),
                OpCode::StoreGlobal("result".into()),
                OpCode::LoadField("x".into()),
                OpCode::StoreField("y".into()),
                OpCode::CallNamed("foo".into(), 3),
                OpCode::NewStruct("Point".into(), 2),
                OpCode::Import("std.io".into()),
            ],
        });
        let m2 = roundtrip(&m);
        assert_eq!(m.functions[0].instructions, m2.functions[0].instructions);
    }

    #[test]
    fn test_composite_and_builtin_roundtrip() {
        let mut m = OvmModule::new("composite");
        m.functions.push(CompiledFunction {
            name: "make_things".into(),
            arity: 0,
            locals_count: 0,
            instructions: vec![
                OpCode::Push(Value::Int(1)),
                OpCode::Push(Value::Int(2)),
                OpCode::Push(Value::Int(3)),
                OpCode::NewArray(3),
                OpCode::Push(Value::String("a".into())),
                OpCode::Push(Value::Int(1)),
                OpCode::NewMap(1),
                OpCode::Index,
                OpCode::Print,
                OpCode::PrintLn,
                OpCode::Len,
                OpCode::TypeOf,
                OpCode::Assert,
                OpCode::Call(2),
                OpCode::Return,
            ],
        });
        let m2 = roundtrip(&m);
        assert_eq!(m.functions[0].instructions, m2.functions[0].instructions);
    }

    #[test]
    fn test_multiple_functions_roundtrip() {
        let mut m = OvmModule::new("multi");
        m.entry_point = Some(1);
        m.globals = vec!["g1".into(), "g2".into()];
        m.constants = vec![Value::Int(100), Value::String("const".into())];
        m.functions.push(CompiledFunction {
            name: "helper".into(),
            arity: 1,
            locals_count: 2,
            instructions: vec![
                OpCode::LoadLocal(0),
                OpCode::Push(Value::Int(1)),
                OpCode::Add,
                OpCode::Return,
            ],
        });
        m.functions.push(CompiledFunction {
            name: "main".into(),
            arity: 0,
            locals_count: 1,
            instructions: vec![
                OpCode::Push(Value::Int(41)),
                OpCode::CallNamed("helper".into(), 1),
                OpCode::PrintLn,
                OpCode::Halt,
            ],
        });
        let m2 = roundtrip(&m);
        assert_eq!(m, m2);
    }

    #[test]
    fn test_invalid_magic_fails() {
        let bad_bytes = vec![0x00, 0x00, 0x00, 0x00, 0x01, 0x00];
        let result = OvmModule::deserialize(&bad_bytes);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("magic"));
    }

    #[test]
    fn test_truncated_input_fails() {
        let result = OvmModule::deserialize(&[b'O', b'V']);
        assert!(result.is_err());
    }

    #[test]
    fn test_opcode_display() {
        assert_eq!(format!("{}", OpCode::Nop), "NOP");
        assert_eq!(format!("{}", OpCode::Push(Value::Int(42))), "PUSH 42");
        assert_eq!(format!("{}", OpCode::LoadLocal(3)), "LOAD_LOCAL 3");
        assert_eq!(
            format!("{}", OpCode::CallNamed("foo".into(), 2)),
            "CALL_NAMED \"foo\" 2"
        );
        assert_eq!(format!("{}", OpCode::Halt), "HALT");
    }

    #[test]
    fn test_value_display() {
        assert_eq!(format!("{}", Value::Null), "null");
        assert_eq!(format!("{}", Value::Int(-7)), "-7");
        assert_eq!(format!("{}", Value::Bool(true)), "true");
        assert_eq!(format!("{}", Value::String("hi".into())), "\"hi\"");
    }
}
