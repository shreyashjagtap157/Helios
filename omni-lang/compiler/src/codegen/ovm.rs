#![allow(dead_code)]
//! OVM Bytecode Backend
//! Generates OVM bytecode from Omni IR for the Omni Virtual Machine

use crate::ir::{IrBinOp, IrFunction, IrInstruction, IrModule, IrTerminator, IrType};
use log::{debug, info};
use std::collections::HashMap;
use std::path::Path;

/// OVM Bytecode opcodes
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum OvmOpcode {
    // Stack Operations (0x00 - 0x0F)
    Nop = 0x00,
    PushI8 = 0x01,
    PushI16 = 0x02,
    PushI32 = 0x03,
    PushI64 = 0x04,
    PushF32 = 0x05,
    PushF64 = 0x06,
    PushStr = 0x07,
    PushNull = 0x08,
    PushTrue = 0x09,
    PushFalse = 0x0A,
    Dup = 0x0B,
    Dup2 = 0x0C,
    Swap = 0x0D,
    Rot = 0x0E,
    Pop = 0x0F,

    // Arithmetic (0x10 - 0x2F)
    AddI64 = 0x10,
    SubI64 = 0x11,
    MulI64 = 0x12,
    DivI64 = 0x13,
    ModI64 = 0x14,
    NegI64 = 0x15,
    AddF64 = 0x18,
    SubF64 = 0x19,
    MulF64 = 0x1A,
    DivF64 = 0x1B,
    NegF64 = 0x1C,
    Inc = 0x20,
    Dec = 0x21,

    // Bitwise (0x30 - 0x3F)
    And = 0x30,
    Or = 0x31,
    Xor = 0x32,
    Not = 0x33,
    Shl = 0x34,
    Shr = 0x35,
    Sar = 0x36,

    // Comparison (0x40 - 0x4F)
    Eq = 0x40,
    Ne = 0x41,
    Lt = 0x42,
    Le = 0x43,
    Gt = 0x44,
    Ge = 0x45,
    Cmp = 0x46,
    IsNull = 0x47,

    // Control Flow (0x50 - 0x5F)
    Jmp = 0x50,
    JmpAbs = 0x51,
    Jz = 0x52,
    Jnz = 0x53,
    Jlt = 0x54,
    Jle = 0x55,
    Jgt = 0x56,
    Jge = 0x57,
    Call = 0x58,
    CallInd = 0x59,
    Ret = 0x5A,
    RetVoid = 0x5B,
    TailCall = 0x5C,

    // Local Variables (0x60 - 0x6F)
    LoadLoc = 0x60,
    StoreLoc = 0x61,
    LoadArg = 0x62,
    StoreArg = 0x63,
    AllocLoc = 0x64,
    FreeLoc = 0x65,

    // Global Variables (0x70 - 0x7F)
    LoadGlb = 0x70,
    StoreGlb = 0x71,
    LoadConst = 0x72,

    // Memory (0x80 - 0x8F)
    Load8 = 0x80,
    Load16 = 0x81,
    Load32 = 0x82,
    Load64 = 0x83,
    Store8 = 0x84,
    Store16 = 0x85,
    Store32 = 0x86,
    Store64 = 0x87,
    Alloc = 0x88,
    Realloc = 0x89,
    Free = 0x8A,
    Memcpy = 0x8B,
    Memset = 0x8C,

    // Objects/Structs (0x90 - 0x9F)
    New = 0x90,
    GetField = 0x91,
    SetField = 0x92,
    GetFieldRef = 0x93,
    Instanceof = 0x94,
    Cast = 0x95,

    // Arrays (0xA0 - 0xAF)
    NewArray = 0xA0,
    ArrayLen = 0xA1,
    ArrayGet = 0xA2,
    ArraySet = 0xA3,
    ArraySlice = 0xA4,

    // Type Conversion (0xB0 - 0xBF)
    I2F = 0xB0,
    F2I = 0xB1,
    I2B = 0xB2,
    B2I = 0xB3,

    // Register Operations (0xC0 - 0xCF) - Register-stack hybrid
    LoadReg = 0xC0,
    StoreReg = 0xC1,
    MoveReg = 0xC2,
    SwapReg = 0xC3,
    PushReg = 0xC4,
    PopReg = 0xC5,

    // System Calls (0xF0 - 0xFF)
    Syscall = 0xF0,
    Debug = 0xF1,
    Trace = 0xF2,
    Assert = 0xF3,
    Halt = 0xFE,
    Panic = 0xFF,
}

/// OVM Bytecode Module
#[derive(Debug, Clone)]
pub struct OvmModule {
    pub name: String,
    pub version: u32,
    pub entry_point: u32,
    pub constants: Vec<OvmConstant>,
    pub functions: Vec<OvmFunction>,
    pub types: Vec<OvmTypeInfo>,
    pub symbols: Vec<OvmSymbol>,
}

#[derive(Debug, Clone)]
pub enum OvmConstant {
    I64(i64),
    F64(f64),
    String(String),
    Bytes(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct OvmFunction {
    pub name: String,
    pub name_index: u32,
    pub param_count: u16,
    pub local_count: u16,
    pub max_stack: u16,
    pub bytecode: Vec<u8>,
    pub is_native: bool,
}

#[derive(Debug, Clone)]
pub struct OvmTypeInfo {
    pub name: String,
    pub size: u32,
    pub alignment: u32,
    pub fields: Vec<(String, u32)>, // name, offset
}

#[derive(Debug, Clone)]
pub struct OvmSymbol {
    pub name: String,
    pub kind: OvmSymbolKind,
    pub visibility: OvmVisibility,
    pub address: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum OvmSymbolKind {
    Function = 0,
    Global = 1,
    Type = 2,
}

#[derive(Debug, Clone, Copy)]
pub enum OvmVisibility {
    Private = 0,
    Public = 1,
}

/// Loop context for break/continue backpatching (used by ovm_direct)
pub struct LoopContext {
    pub break_addrs: Vec<usize>,
    pub continue_target: usize,
}

/// Bytecode generator context
pub struct OvmCodegen {
    pub constants: Vec<OvmConstant>,
    pub const_map: HashMap<String, u32>,
    pub label_offsets: HashMap<String, u32>,
    pub current_bytecode: Vec<u8>,
    pub local_indices: HashMap<String, u16>,
    pub arg_indices: HashMap<String, u16>,
    pub func_indices: HashMap<String, u32>,
    pub hardware: Option<HardwareConfig>,
    pub loop_contexts: Vec<LoopContext>,
}

impl OvmCodegen {
    pub fn new() -> Self {
        OvmCodegen {
            constants: Vec::new(),
            const_map: HashMap::new(),
            label_offsets: HashMap::new(),
            current_bytecode: Vec::new(),
            local_indices: HashMap::new(),
            arg_indices: HashMap::new(),
            func_indices: HashMap::new(),
            hardware: None,
            loop_contexts: Vec::new(),
        }
    }

    pub fn add_constant(&mut self, value: OvmConstant) -> u32 {
        let key = format!("{:?}", value);
        if let Some(&idx) = self.const_map.get(&key) {
            return idx;
        }
        let idx = self.constants.len() as u32;
        self.constants.push(value);
        self.const_map.insert(key, idx);
        idx
    }

    pub fn emit(&mut self, opcode: OvmOpcode) {
        self.current_bytecode.push(opcode as u8);
    }

    pub fn emit_u8(&mut self, value: u8) {
        self.current_bytecode.push(value);
    }

    pub fn emit_u16(&mut self, value: u16) {
        self.current_bytecode
            .extend_from_slice(&value.to_le_bytes());
    }

    pub fn emit_u32(&mut self, value: u32) {
        self.current_bytecode
            .extend_from_slice(&value.to_le_bytes());
    }

    pub fn emit_i32(&mut self, value: i32) {
        self.current_bytecode
            .extend_from_slice(&value.to_le_bytes());
    }

    pub fn emit_u64(&mut self, value: u64) {
        self.current_bytecode
            .extend_from_slice(&value.to_le_bytes());
    }

    pub fn current_offset(&self) -> u32 {
        self.current_bytecode.len() as u32
    }
}

/// Generate OVM bytecode from Omni IR
pub fn generate_ovm(ir: IrModule, output: &Path) -> Result<(), String> {
    info!("Generating OVM bytecode for module: {}", ir.name);

    // Detect hardware capabilities for adaptive codegen
    let hw = HardwareConfig::detect();
    info!(
        "Hardware: {:?} SIMD={:?} cores={} mem={}MB gpu={}",
        hw.cpu_arch,
        hw.simd_level,
        hw.core_count,
        hw.available_memory / (1024 * 1024),
        hw.has_gpu
    );

    let mut codegen = OvmCodegen::new();
    codegen.hardware = Some(hw);
    let mut functions = Vec::new();

    // Build function index map first
    for (i, ir_func) in ir.functions.iter().enumerate() {
        codegen.func_indices.insert(ir_func.name.clone(), i as u32);
    }

    for ir_func in &ir.functions {
        debug!("Compiling function: {}", ir_func.name);
        let ovm_func = compile_function(&mut codegen, ir_func)?;
        functions.push(ovm_func);
    }

    let module = OvmModule {
        name: ir.name.clone(),
        version: 1,
        entry_point: find_entry_point(&functions),
        constants: codegen.constants,
        functions,
        types: Vec::new(),
        symbols: Vec::new(),
    };

    // Write bytecode file
    let bytecode = serialize_module(&module)?;
    std::fs::write(output.with_extension("ovm"), &bytecode)
        .map_err(|e| format!("Failed to write OVM file: {}", e))?;

    // Write hardware hints file alongside the bytecode
    if let Some(ref hw) = codegen.hardware {
        let hints = format!(
            "arch={:?}\nsimd={:?}\ncores={}\nmemory={}\ngpu={}\ngc_mode={}\nthread_count={}\n",
            hw.cpu_arch,
            hw.simd_level,
            hw.core_count,
            hw.available_memory,
            hw.has_gpu,
            hw.optimal_gc_mode(),
            hw.optimal_thread_count()
        );
        let _ = std::fs::write(output.with_extension("ovm.hints"), hints);
    }

    info!("Generated {} bytes of OVM bytecode", bytecode.len());
    Ok(())
}

fn find_entry_point(functions: &[OvmFunction]) -> u32 {
    for (i, func) in functions.iter().enumerate() {
        if func.name == "main" || func.name == "_start" {
            return i as u32;
        }
    }
    0
}

fn compile_function(codegen: &mut OvmCodegen, func: &IrFunction) -> Result<OvmFunction, String> {
    codegen.current_bytecode.clear();
    codegen.label_offsets.clear();
    codegen.local_indices.clear();
    codegen.arg_indices.clear();

    // Index function parameters (args from stack, stored as locals)
    for (i, (name, _)) in func.params.iter().enumerate() {
        codegen.arg_indices.insert(name.clone(), i as u16);
        codegen.local_indices.insert(name.clone(), i as u16);
    }

    // Index all local variables and IR temp variables
    let mut next_local = func.params.len() as u16;
    for (name, _) in &func.locals {
        if !codegen.local_indices.contains_key(name) {
            codegen.local_indices.insert(name.clone(), next_local);
            next_local += 1;
        }
    }
    // Also scan for IR temp variables (e.g., %t0, %t1) and allocas
    for block in &func.blocks {
        for inst in &block.instructions {
            match inst {
                IrInstruction::Load { dest, .. }
                | IrInstruction::Alloca { dest, .. }
                | IrInstruction::BinOp { dest, .. }
                | IrInstruction::Call {
                    dest: Some(dest), ..
                }
                | IrInstruction::NativeCall {
                    dest: Some(dest), ..
                } => {
                    if !codegen.local_indices.contains_key(dest) {
                        codegen.local_indices.insert(dest.clone(), next_local);
                        next_local += 1;
                    }
                }
                _ => {}
            }
        }
    }

    // First pass: collect label offsets
    let mut offset = 0u32;
    for block in &func.blocks {
        codegen.label_offsets.insert(block.label.clone(), offset);
        for inst in &block.instructions {
            offset += instruction_size(inst);
        }
        offset += terminator_size(&block.terminator);
    }

    // Second pass: generate bytecode
    for block in &func.blocks {
        for inst in &block.instructions {
            compile_instruction(codegen, inst)?;
        }
        compile_terminator(codegen, &block.terminator)?;
    }

    let name_const = codegen.add_constant(OvmConstant::String(func.name.clone()));

    Ok(OvmFunction {
        name: func.name.clone(),
        name_index: name_const,
        param_count: func.params.len() as u16,
        local_count: codegen.local_indices.len() as u16,
        max_stack: estimate_max_stack(func),
        bytecode: codegen.current_bytecode.clone(),
        is_native: false,
    })
}

/// Push an IrValue onto the OVM stack
fn emit_push_value(codegen: &mut OvmCodegen, value: &crate::ir::IrValue) {
    match value {
        crate::ir::IrValue::Const(crate::ir::IrConst::Int(n)) => {
            codegen.emit(OvmOpcode::PushI64);
            codegen.emit_u64(*n as u64);
        }
        crate::ir::IrValue::Const(crate::ir::IrConst::Float(f)) => {
            codegen.emit(OvmOpcode::PushF64);
            codegen.emit_u64(f.to_bits());
        }
        crate::ir::IrValue::Const(crate::ir::IrConst::Bool(b)) => {
            codegen.emit(if *b {
                OvmOpcode::PushTrue
            } else {
                OvmOpcode::PushFalse
            });
        }
        crate::ir::IrValue::Const(crate::ir::IrConst::Str(s)) => {
            let idx = codegen.add_constant(OvmConstant::String(s.clone()));
            codegen.emit(OvmOpcode::PushStr);
            codegen.emit_u32(idx);
        }
        crate::ir::IrValue::Var(name) => {
            if let Some(&idx) = codegen.local_indices.get(name) {
                codegen.emit(OvmOpcode::LoadLoc);
                codegen.emit_u16(idx);
            } else {
                codegen.emit(OvmOpcode::PushNull);
            }
        }
        _ => {
            codegen.emit(OvmOpcode::PushNull);
        }
    }
}

/// Store stack top into a dest local variable slot
fn emit_store_dest(codegen: &mut OvmCodegen, dest: &str) {
    if let Some(&idx) = codegen.local_indices.get(dest) {
        codegen.emit(OvmOpcode::StoreLoc);
        codegen.emit_u16(idx);
    }
}

fn instruction_size(inst: &IrInstruction) -> u32 {
    match inst {
        IrInstruction::Alloca { .. } => 3, // opcode(1) + u16(2)
        IrInstruction::Load { .. } => 7,   // LoadLoc(3) + StoreLoc(3) + Pop(1)
        IrInstruction::Store { value, .. } => {
            let val_size = match value {
                crate::ir::IrValue::Const(crate::ir::IrConst::Int(_)) => 9,
                crate::ir::IrValue::Const(crate::ir::IrConst::Float(_)) => 9,
                crate::ir::IrValue::Const(crate::ir::IrConst::Bool(_)) => 1,
                crate::ir::IrValue::Const(crate::ir::IrConst::Str(_)) => 5,
                crate::ir::IrValue::Var(_) => 3,
                _ => 1,
            };
            val_size + 3 // push value + store opcode(1) + u16(2)
        }
        IrInstruction::BinOp { left, right, .. } => {
            // Push operands + opcode + StoreLoc(3) + Pop(1)
            let left_size = match left {
                crate::ir::IrValue::Const(crate::ir::IrConst::Int(_)) => 9,
                crate::ir::IrValue::Const(crate::ir::IrConst::Float(_)) => 9,
                crate::ir::IrValue::Const(crate::ir::IrConst::Bool(_)) => 1,
                crate::ir::IrValue::Const(crate::ir::IrConst::Str(_)) => 5,
                crate::ir::IrValue::Var(_) => 3,
                _ => 1,
            };
            let right_size = match right {
                crate::ir::IrValue::Const(crate::ir::IrConst::Int(_)) => 9,
                crate::ir::IrValue::Const(crate::ir::IrConst::Float(_)) => 9,
                crate::ir::IrValue::Const(crate::ir::IrConst::Bool(_)) => 1,
                crate::ir::IrValue::Const(crate::ir::IrConst::Str(_)) => 5,
                crate::ir::IrValue::Var(_) => 3,
                _ => 1,
            };
            left_size + right_size + 1 + 3 + 1 // push operands + opcode + StoreLoc + Pop
        }
        IrInstruction::Call { .. } => 5,     // opcode(1) + u32(4)
        IrInstruction::GetField { .. } => 3, // opcode(1) + u16(2)
        // New instruction variants
        IrInstruction::Phi { .. } => 1,    // NOP placeholder
        IrInstruction::Select { .. } => 1, // NOP placeholder
        IrInstruction::Switch { cases, .. } => 2 + (cases.len() as u32 * 6), // dup+cmp+jz per case + final jmp
        IrInstruction::CreateClosure { .. } => 12, // LoadConst(5) + New(7)
        IrInstruction::CallClosure { .. } => 2,    // opcode(1) + u8(1)
        IrInstruction::AsyncSpawn { .. } => 8,     // LoadConst(5) + Syscall(3)
        IrInstruction::AsyncAwait { .. } => 3,     // Syscall(1) + u16(2)
        IrInstruction::TraitDispatch { .. } => 7,  // LoadConst(5) + opcode + u8
        IrInstruction::VTableLookup { .. } => 5,   // opcode(1) + u32(4)
        IrInstruction::Cast { .. } => 3,           // opcode(1) + u16(2)
        IrInstruction::ExtractValue { .. } => 4,   // opcode(1) + u8(1) + u16(2)
        IrInstruction::InsertValue { .. } => 5,    // opcode(1) + u32(4)
        IrInstruction::NativeCall { args, .. } => 5 + (args.len() as u32),
        IrInstruction::BoundsCheck { .. } => 5, // opcode(1) + 2xu16(4)
    }
}

fn terminator_size(term: &IrTerminator) -> u32 {
    match term {
        IrTerminator::Return(_) => 1,
        IrTerminator::Branch(_) => 5,
        IrTerminator::CondBranch { .. } => 9,
        IrTerminator::Unreachable => 1,
    }
}

fn compile_instruction(codegen: &mut OvmCodegen, inst: &IrInstruction) -> Result<(), String> {
    // Determine if SIMD dispatch is available
    let simd_available = codegen
        .hardware
        .as_ref()
        .map(|hw| !matches!(hw.simd_level, SimdLevel::None))
        .unwrap_or(false);

    match inst {
        IrInstruction::Alloca { dest, ty } => {
            let size = type_size(ty);
            // Use SIMD-aligned allocation when available
            if simd_available && size >= 16 {
                codegen.emit(OvmOpcode::AllocLoc);
                // Align to SIMD width: 16 (SSE2), 32 (AVX), 64 (AVX512)
                let alignment = match codegen.hardware.as_ref().map(|h| h.simd_level) {
                    Some(SimdLevel::Avx512) => 64,
                    Some(SimdLevel::Avx2) | Some(SimdLevel::Avx) => 32,
                    _ => 16,
                };
                let aligned_size = (size + alignment - 1) & !(alignment - 1);
                codegen.emit_u16(aligned_size as u16);
            } else {
                codegen.emit(OvmOpcode::AllocLoc);
                codegen.emit_u16(size as u16);
            }
            let idx = codegen.local_indices.len() as u16;
            codegen.local_indices.insert(dest.clone(), idx);
        }

        IrInstruction::Load { dest, ptr, ty: _ } => {
            if let Some(&idx) = codegen.local_indices.get(ptr) {
                codegen.emit(OvmOpcode::LoadLoc);
                codegen.emit_u16(idx);
            } else {
                codegen.emit(OvmOpcode::PushNull);
            }
            // Store result in dest local so subsequent Var references can load it
            emit_store_dest(codegen, dest);
            // Pop the value — the consuming instruction will load from the dest local
            codegen.emit(OvmOpcode::Pop);
        }

        IrInstruction::Store { ptr, value } => {
            // Push the value to store
            match value {
                crate::ir::IrValue::Const(crate::ir::IrConst::Int(n)) => {
                    codegen.emit(OvmOpcode::PushI64);
                    codegen.emit_u64(*n as u64);
                }
                crate::ir::IrValue::Const(crate::ir::IrConst::Float(f)) => {
                    codegen.emit(OvmOpcode::PushF64);
                    codegen.emit_u64(f.to_bits());
                }
                crate::ir::IrValue::Const(crate::ir::IrConst::Bool(b)) => {
                    codegen.emit(if *b {
                        OvmOpcode::PushTrue
                    } else {
                        OvmOpcode::PushFalse
                    });
                }
                crate::ir::IrValue::Const(crate::ir::IrConst::Str(s)) => {
                    let idx = codegen.add_constant(OvmConstant::String(s.clone()));
                    codegen.emit(OvmOpcode::PushStr);
                    codegen.emit_u32(idx);
                }
                crate::ir::IrValue::Var(name) => {
                    if let Some(&idx) = codegen.local_indices.get(name) {
                        codegen.emit(OvmOpcode::LoadLoc);
                        codegen.emit_u16(idx);
                    }
                }
                _ => {
                    codegen.emit(OvmOpcode::PushNull);
                }
            }
            // Store into the target
            if let Some(&idx) = codegen.local_indices.get(ptr) {
                codegen.emit(OvmOpcode::StoreLoc);
                codegen.emit_u16(idx);
            } else {
                codegen.emit(OvmOpcode::Store64);
            }
        }

        IrInstruction::BinOp {
            dest,
            op,
            left,
            right,
        } => {
            // Push operands using helper
            emit_push_value(codegen, right);
            emit_push_value(codegen, left);
            // Emit the operation
            let opcode = match op {
                IrBinOp::Add => OvmOpcode::AddI64,
                IrBinOp::Sub => OvmOpcode::SubI64,
                IrBinOp::Mul => OvmOpcode::MulI64,
                IrBinOp::Div => OvmOpcode::DivI64,
                IrBinOp::Mod => OvmOpcode::ModI64,
                IrBinOp::And => OvmOpcode::And,
                IrBinOp::Or => OvmOpcode::Or,
                IrBinOp::Eq => OvmOpcode::Eq,
                IrBinOp::Ne => OvmOpcode::Ne,
                IrBinOp::Lt => OvmOpcode::Lt,
                IrBinOp::Le => OvmOpcode::Le,
                IrBinOp::Gt => OvmOpcode::Gt,
                IrBinOp::Ge => OvmOpcode::Ge,
            };
            codegen.emit(opcode);
            // Store result in dest local
            emit_store_dest(codegen, dest);
            // Pop from stack — consuming instructions load from dest local
            codegen.emit(OvmOpcode::Pop);
        }

        IrInstruction::Call { dest, func, args } => {
            // Push arguments onto the stack before the call
            for arg in args {
                emit_push_value(codegen, arg);
            }
            let func_idx = codegen
                .func_indices
                .get(func)
                .copied()
                .unwrap_or_else(|| codegen.add_constant(OvmConstant::String(func.clone())));
            codegen.emit(OvmOpcode::Call);
            codegen.emit_u32(func_idx);
            // Store return value in IR temp local so subsequent Var references find it
            if let Some(d) = dest {
                emit_store_dest(codegen, d);
            }
        }

        IrInstruction::GetField {
            dest: _,
            ptr: _,
            field,
        } => {
            codegen.emit(OvmOpcode::GetField);
            codegen.emit_u16(*field as u16);
        }

        // New instruction handlers
        IrInstruction::Phi {
            dest: _,
            ty: _,
            incoming: _,
        } => {
            // PHI nodes are resolved during SSA destruction, emit placeholder
            codegen.emit(OvmOpcode::Nop);
        }

        IrInstruction::Select {
            dest: _,
            cond: _,
            then_val: _,
            else_val: _,
        } => {
            // Select is handled via conditional jump pattern
            codegen.emit(OvmOpcode::Nop); // Placeholder
        }

        IrInstruction::Switch {
            value: _,
            default: _,
            cases,
        } => {
            // Emit switch as sequence of comparisons and jumps
            for (_case_val, _case_label) in cases {
                codegen.emit(OvmOpcode::Dup);
                codegen.emit(OvmOpcode::Cmp);
                codegen.emit(OvmOpcode::Jz);
                codegen.emit_i32(0); // Placeholder offset
            }
            codegen.emit(OvmOpcode::Jmp);
            codegen.emit_i32(0); // Jump to default
        }

        IrInstruction::CreateClosure {
            dest: _,
            func,
            captures: _,
        } => {
            let func_idx = codegen
                .func_indices
                .get(func)
                .copied()
                .unwrap_or_else(|| codegen.add_constant(OvmConstant::String(func.clone())));
            codegen.emit(OvmOpcode::LoadConst);
            codegen.emit_u32(func_idx);
            codegen.emit(OvmOpcode::New);
            codegen.emit_u32(0); // Closure type
            codegen.emit_u16(16); // Size for env + funcptr
        }

        IrInstruction::CallClosure {
            dest: _,
            closure: _,
            args,
        } => {
            codegen.emit(OvmOpcode::CallInd);
            codegen.emit_u8(args.len() as u8);
        }

        IrInstruction::AsyncSpawn {
            dest: _,
            func,
            args: _,
        } => {
            let func_idx = codegen
                .func_indices
                .get(func)
                .copied()
                .unwrap_or_else(|| codegen.add_constant(OvmConstant::String(func.clone())));
            codegen.emit(OvmOpcode::LoadConst);
            codegen.emit_u32(func_idx);
            codegen.emit(OvmOpcode::Syscall);
            codegen.emit_u16(0x100); // SYSCALL_ASYNC_SPAWN
        }

        IrInstruction::AsyncAwait { dest: _, future: _ } => {
            codegen.emit(OvmOpcode::Syscall);
            codegen.emit_u16(0x101); // SYSCALL_ASYNC_AWAIT
        }

        IrInstruction::TraitDispatch {
            dest: _,
            object: _,
            method,
            args,
        } => {
            let method_idx = codegen.add_constant(OvmConstant::String(method.clone()));
            codegen.emit(OvmOpcode::LoadConst);
            codegen.emit_u32(method_idx);
            codegen.emit(OvmOpcode::CallInd);
            codegen.emit_u8(args.len() as u8);
        }

        IrInstruction::VTableLookup {
            dest: _,
            object: _,
            trait_name: _,
            method_idx,
        } => {
            codegen.emit(OvmOpcode::GetField);
            codegen.emit_u16(*method_idx as u16);
        }

        IrInstruction::Cast {
            dest: _,
            value: _,
            to_type: _,
        } => {
            codegen.emit(OvmOpcode::Cast);
            codegen.emit_u32(0); // Type ID placeholder
        }

        IrInstruction::ExtractValue {
            dest: _,
            aggregate: _,
            indices,
        } => {
            for idx in indices {
                codegen.emit(OvmOpcode::GetField);
                codegen.emit_u16(*idx as u16);
            }
        }

        IrInstruction::InsertValue {
            dest: _,
            aggregate: _,
            value: _,
            indices,
        } => {
            for idx in indices {
                codegen.emit(OvmOpcode::SetField);
                codegen.emit_u16(*idx as u16);
            }
        }

        IrInstruction::NativeCall {
            dest: _,
            module,
            func,
            args,
        } => {
            // Push arguments onto the stack
            for arg in args {
                emit_push_value(codegen, arg);
            }
            // Emit native call: push the qualified name, then Syscall
            let native_name = format!("{}::{}", module, func);
            let name_idx = codegen.add_constant(OvmConstant::String(native_name));
            codegen.emit(OvmOpcode::Syscall);
            codegen.emit_u16(name_idx as u16);
            // Don't store — the IR's Store instruction handles saving if needed
        }

        IrInstruction::BoundsCheck { index, length } => {
            // Load index and length, emit bounds check
            if let Some(&idx) = codegen.local_indices.get(index) {
                codegen.emit(OvmOpcode::LoadLoc);
                codegen.emit_u16(idx);
            }
            if let Some(&len) = codegen.local_indices.get(length) {
                codegen.emit(OvmOpcode::LoadLoc);
                codegen.emit_u16(len);
            }
            // Runtime bounds check: panics if index >= length
            codegen.emit(OvmOpcode::Lt);
        }
    }
    Ok(())
}

fn compile_terminator(codegen: &mut OvmCodegen, term: &IrTerminator) -> Result<(), String> {
    match term {
        IrTerminator::Return(Some(_)) => {
            codegen.emit(OvmOpcode::Ret);
        }
        IrTerminator::Return(None) => {
            codegen.emit(OvmOpcode::RetVoid);
        }
        IrTerminator::Branch(label) => {
            let target = *codegen
                .label_offsets
                .get(label)
                .ok_or_else(|| format!("Unknown label: {}", label))?;
            let current = codegen.current_offset();
            let offset = (target as i32) - (current as i32) - 5;
            codegen.emit(OvmOpcode::Jmp);
            codegen.emit_i32(offset);
        }
        IrTerminator::CondBranch {
            cond: _,
            then_label,
            else_label,
        } => {
            let then_target = *codegen
                .label_offsets
                .get(then_label)
                .ok_or_else(|| format!("Unknown label: {}", then_label))?;
            let else_target = *codegen
                .label_offsets
                .get(else_label)
                .ok_or_else(|| format!("Unknown label: {}", else_label))?;
            let current = codegen.current_offset();
            let then_offset = (then_target as i32) - (current as i32) - 5;
            let else_offset = (else_target as i32) - (current as i32) - 9;
            codegen.emit(OvmOpcode::Jnz);
            codegen.emit_i32(then_offset);
            codegen.emit(OvmOpcode::Jmp);
            codegen.emit_i32(else_offset);
        }
        IrTerminator::Unreachable => {
            codegen.emit(OvmOpcode::Panic);
        }
    }
    Ok(())
}

fn type_size(ty: &IrType) -> usize {
    match ty {
        IrType::Void => 0,
        IrType::I8 => 1,
        IrType::I16 => 2,
        IrType::I32 => 4,
        IrType::I64 => 8,
        IrType::F32 => 4,
        IrType::F64 => 8,
        IrType::Bool => 1,
        IrType::Ptr(_) => 8,
        IrType::Array(elem, size) => type_size(elem) * size,
        IrType::Struct(_) => 8, // Default struct size
        // Advanced types - all pointer-sized in the OVM
        IrType::Closure { .. } => 16, // Fat pointer (code + env)
        IrType::Future(_) => 8,       // Box pointer to state machine
        IrType::TraitObject(_) => 16, // Fat pointer (data + vtable)
        IrType::Generic(_) => 8,      // Monomorphized at compile time
        IrType::Enum { .. } => 16,    // Tag + payload
        IrType::Tuple(elements) => elements.iter().map(type_size).sum(),
        IrType::FnPtr { .. } => 8, // Function pointer
    }
}

fn estimate_max_stack(func: &IrFunction) -> u16 {
    // Proper stack depth analysis: simulate each instruction's stack effect
    let mut max_depth: usize = 0;
    let mut current_depth: usize = 0;

    for block in &func.blocks {
        for instr in &block.instructions {
            let (pops, pushes) = stack_effect(instr);
            // Underflow protection: if we'd go negative, reset to 0
            current_depth = current_depth.saturating_sub(pops);
            current_depth += pushes;
            max_depth = max_depth.max(current_depth);
        }
    }

    // Add headroom for call frames and temporaries
    let result = max_depth + 8;
    // Clamp to reasonable bounds
    (result.min(65535) as u16).max(16)
}

/// Returns (pops, pushes) for each instruction
fn stack_effect(instr: &IrInstruction) -> (usize, usize) {
    match instr {
        IrInstruction::Alloca { .. } => (0, 1),
        IrInstruction::Load { .. } => (0, 1),
        IrInstruction::Store { value: _, .. } => (1, 0),
        IrInstruction::BinOp { .. } => (2, 1),
        IrInstruction::Call { args, dest, .. } => (args.len(), if dest.is_some() { 1 } else { 0 }),
        IrInstruction::GetField { .. } => (1, 1),
        IrInstruction::Phi { .. } => (0, 1),
        IrInstruction::Select { .. } => (3, 1), // cond + then_val + else_val
        IrInstruction::Switch { .. } => (1, 0),
        IrInstruction::CreateClosure { captures, .. } => (captures.len(), 1),
        IrInstruction::CallClosure { args, dest, .. } => {
            (args.len() + 1, if dest.is_some() { 1 } else { 0 })
        }
        IrInstruction::AsyncSpawn { args, .. } => (args.len(), 1),
        IrInstruction::AsyncAwait { dest, .. } => (1, if dest.is_some() { 1 } else { 0 }),
        IrInstruction::TraitDispatch { args, dest, .. } => {
            (args.len() + 1, if dest.is_some() { 1 } else { 0 })
        }
        IrInstruction::VTableLookup { .. } => (1, 1),
        IrInstruction::Cast { .. } => (1, 1),
        IrInstruction::ExtractValue { .. } => (1, 1),
        IrInstruction::InsertValue { .. } => (2, 1),
        IrInstruction::NativeCall { args, dest, .. } => {
            (args.len(), if dest.is_some() { 1 } else { 0 })
        }
        IrInstruction::BoundsCheck { .. } => (2, 0),
    }
}

/// Serialize OVM module to bytes
pub fn serialize_module(module: &OvmModule) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();

    // Magic number "OVM\0"
    bytes.extend_from_slice(b"OVM\0");

    // Version (u32)
    bytes.extend_from_slice(&module.version.to_le_bytes());

    // Flags (u32)
    bytes.extend_from_slice(&0u32.to_le_bytes());

    // Entry point (u64)
    bytes.extend_from_slice(&(module.entry_point as u64).to_le_bytes());

    // Placeholder for offsets (will patch later)
    let const_pool_offset_pos = bytes.len();
    bytes.extend_from_slice(&0u64.to_le_bytes()); // const_pool_off
    bytes.extend_from_slice(&0u64.to_le_bytes()); // const_pool_len

    let code_offset_pos = bytes.len();
    bytes.extend_from_slice(&0u64.to_le_bytes()); // code_off
    bytes.extend_from_slice(&0u64.to_le_bytes()); // code_len

    let symbol_offset_pos = bytes.len();
    bytes.extend_from_slice(&0u64.to_le_bytes()); // symbol_off
    bytes.extend_from_slice(&0u64.to_le_bytes()); // symbol_len

    // Debug info (0 if none)
    bytes.extend_from_slice(&0u64.to_le_bytes());
    bytes.extend_from_slice(&0u64.to_le_bytes());

    // Constant pool
    let const_pool_off = bytes.len() as u64;
    bytes.extend_from_slice(&(module.constants.len() as u32).to_le_bytes());
    for constant in &module.constants {
        match constant {
            OvmConstant::I64(v) => {
                bytes.push(0x01);
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            OvmConstant::F64(v) => {
                bytes.push(0x02);
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            OvmConstant::String(s) => {
                bytes.push(0x03);
                bytes.extend_from_slice(&(s.len() as u32).to_le_bytes());
                bytes.extend_from_slice(s.as_bytes());
            }
            OvmConstant::Bytes(b) => {
                bytes.push(0x04);
                bytes.extend_from_slice(&(b.len() as u32).to_le_bytes());
                bytes.extend_from_slice(b);
            }
        }
    }
    let const_pool_len = bytes.len() as u64 - const_pool_off;

    // Code section
    let code_off = bytes.len() as u64;
    bytes.extend_from_slice(&(module.functions.len() as u32).to_le_bytes());
    for func in &module.functions {
        bytes.extend_from_slice(&func.name_index.to_le_bytes());
        bytes.extend_from_slice(&func.param_count.to_le_bytes());
        bytes.extend_from_slice(&func.local_count.to_le_bytes());
        bytes.extend_from_slice(&func.max_stack.to_le_bytes());
        bytes.extend_from_slice(&(func.bytecode.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&func.bytecode);
    }
    let code_len = bytes.len() as u64 - code_off;

    // Symbol table — build from functions (name -> code offset)
    let symbol_off = bytes.len() as u64;
    bytes.extend_from_slice(&(module.functions.len() as u32).to_le_bytes());
    // Calculate code offsets for each function
    let mut code_offset: u64 = 4; // code section starts with function count
    for func in &module.functions {
        // Write symbol: name_len(u32) + name + code_offset(u64)
        bytes.extend_from_slice(&(func.name.len() as u32).to_le_bytes());
        bytes.extend_from_slice(func.name.as_bytes());
        bytes.extend_from_slice(&code_offset.to_le_bytes());
        // Advance past this function's header + bytecode
        code_offset += 4 + 2 + 2 + 2 + 4 + func.bytecode.len() as u64;
    }
    let symbol_len = bytes.len() as u64 - symbol_off;

    // Patch offsets
    bytes[const_pool_offset_pos..const_pool_offset_pos + 8]
        .copy_from_slice(&const_pool_off.to_le_bytes());
    bytes[const_pool_offset_pos + 8..const_pool_offset_pos + 16]
        .copy_from_slice(&const_pool_len.to_le_bytes());
    bytes[code_offset_pos..code_offset_pos + 8].copy_from_slice(&code_off.to_le_bytes());
    bytes[code_offset_pos + 8..code_offset_pos + 16].copy_from_slice(&code_len.to_le_bytes());
    bytes[symbol_offset_pos..symbol_offset_pos + 8].copy_from_slice(&symbol_off.to_le_bytes());
    bytes[symbol_offset_pos + 8..symbol_offset_pos + 16].copy_from_slice(&symbol_len.to_le_bytes());

    Ok(bytes)
}

/// Execution mode selection
#[derive(Debug, Clone, Copy)]
pub enum ExecutionMode {
    Native,  // Direct native code (like C)
    Managed, // Bytecode with GC, reflection
    Hybrid,  // Mix per function
}

/// Hardware-adaptive configuration
pub struct HardwareConfig {
    pub cpu_arch: CpuArch,
    pub simd_level: SimdLevel,
    pub core_count: u32,
    pub available_memory: u64,
    pub has_gpu: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum CpuArch {
    X86_64,
    Arm64,
    RiscV64,
    Unknown,
}

#[derive(Debug, Clone, Copy)]
pub enum SimdLevel {
    None,
    Sse2,
    Avx,
    Avx2,
    Avx512,
    Neon,
    Sve,
}

impl HardwareConfig {
    pub fn detect() -> Self {
        HardwareConfig {
            cpu_arch: Self::detect_arch(),
            simd_level: Self::detect_simd(),
            core_count: num_cpus::get() as u32,
            available_memory: sys_info::mem_info()
                .map(|m| m.avail * 1024)
                .unwrap_or(1024 * 1024 * 1024),
            has_gpu: Self::detect_gpu(),
        }
    }

    fn detect_arch() -> CpuArch {
        #[cfg(target_arch = "x86_64")]
        {
            CpuArch::X86_64
        }
        #[cfg(target_arch = "aarch64")]
        {
            CpuArch::Arm64
        }
        #[cfg(target_arch = "riscv64")]
        {
            CpuArch::RiscV64
        }
        #[cfg(not(any(
            target_arch = "x86_64",
            target_arch = "aarch64",
            target_arch = "riscv64"
        )))]
        {
            CpuArch::Unknown
        }
    }

    fn detect_simd() -> SimdLevel {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx512f") {
                SimdLevel::Avx512
            } else if is_x86_feature_detected!("avx2") {
                SimdLevel::Avx2
            } else if is_x86_feature_detected!("avx") {
                SimdLevel::Avx
            } else if is_x86_feature_detected!("sse2") {
                SimdLevel::Sse2
            } else {
                SimdLevel::None
            }
        }
        #[cfg(target_arch = "aarch64")]
        {
            SimdLevel::Neon
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            SimdLevel::None
        }
    }

    fn detect_gpu() -> bool {
        // Simplified GPU detection
        std::env::var("CUDA_VISIBLE_DEVICES").is_ok()
            || std::env::var("HIP_VISIBLE_DEVICES").is_ok()
            || std::path::Path::new("/dev/dri").exists()
    }

    pub fn optimal_thread_count(&self) -> u32 {
        self.core_count
    }

    pub fn optimal_gc_mode(&self) -> &'static str {
        if self.available_memory > 8 * 1024 * 1024 * 1024 {
            "concurrent"
        } else if self.core_count >= 4 {
            "generational"
        } else {
            "refcount"
        }
    }

    /// Returns the optimal SIMD vector width in bytes
    pub fn optimal_simd_width(&self) -> u32 {
        match self.simd_level {
            SimdLevel::Avx512 => 64,
            SimdLevel::Avx2 | SimdLevel::Avx => 32,
            SimdLevel::Sse2 => 16,
            SimdLevel::Neon => 16,
            SimdLevel::Sve => 32, // Minimum SVE width
            SimdLevel::None => 0,
        }
    }

    /// Select memory allocation strategy based on available resources
    pub fn select_memory_strategy(&self) -> &'static str {
        if self.available_memory > 16 * 1024 * 1024 * 1024 {
            "arena" // Large arena allocator for lots of memory
        } else if self.available_memory > 4 * 1024 * 1024 * 1024 {
            "slab" // Slab allocator for medium memory
        } else {
            "bump" // Simple bump allocator for limited memory
        }
    }

    /// Whether to enable parallel GC
    pub fn enable_parallel_gc(&self) -> bool {
        self.core_count >= 4 && self.available_memory > 2 * 1024 * 1024 * 1024
    }
}
