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

#![allow(dead_code, unused_imports)]
//! Native Code Generation
//!
//! Multi-architecture native code generation from Omni IR.
//! Targets:
//! - x86-64 (System V ABI & Windows x64 ABI)
//! - ARM64 / AArch64
//! - WebAssembly (WASI)
//!
//! Features:
//! - Linear scan register allocation
//! - Instruction selection via pattern matching
//! - Binary output (ELF, PE/COFF, Mach-O, WASM)
//! - Debug info emission (DWARF integration)
//! - PIC/PIE code generation
//! - TLS (Thread-Local Storage) support

use crate::ir::{
    IrBinOp, IrBlock, IrConst, IrFunction, IrInstruction, IrModule, IrTerminator, IrType, IrValue,
};
use log::{debug, info};
use std::collections::HashMap;
use std::fmt;

// ─────────────────────────────────────────────────────────────────────────────
// Target Architecture
// ─────────────────────────────────────────────────────────────────────────────

/// Target architecture for native code generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    X86_64,
    Aarch64,
    Wasm32,
    Riscv64,
}

impl fmt::Display for Architecture {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Architecture::X86_64 => write!(f, "x86_64"),
            Architecture::Aarch64 => write!(f, "aarch64"),
            Architecture::Wasm32 => write!(f, "wasm32"),
            Architecture::Riscv64 => write!(f, "riscv64"),
        }
    }
}

/// Target operating system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetOS {
    Linux,
    Windows,
    MacOS,
    Wasi,
    Freestanding,
}

/// Binary output format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// ELF (Linux, BSD)
    Elf,
    /// PE/COFF (Windows)
    Pe,
    /// Mach-O (macOS)
    MachO,
    /// WebAssembly binary
    Wasm,
    /// Raw binary (no headers)
    RawBin,
}

/// Full target triple
#[derive(Debug, Clone)]
pub struct TargetTriple {
    pub arch: Architecture,
    pub os: TargetOS,
    pub format: OutputFormat,
    /// CPU features (e.g., "sse4.2", "avx2", "neon")
    pub features: Vec<String>,
    /// Position-independent code
    pub pic: bool,
}

impl TargetTriple {
    pub fn host() -> Self {
        #[cfg(target_os = "windows")]
        let (os, format) = (TargetOS::Windows, OutputFormat::Pe);
        #[cfg(target_os = "linux")]
        let (os, format) = (TargetOS::Linux, OutputFormat::Elf);
        #[cfg(target_os = "macos")]
        let (os, format) = (TargetOS::MacOS, OutputFormat::MachO);
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        let (os, format) = (TargetOS::Freestanding, OutputFormat::RawBin);

        #[cfg(target_arch = "x86_64")]
        let arch = Architecture::X86_64;
        #[cfg(target_arch = "aarch64")]
        let arch = Architecture::Aarch64;
        #[cfg(target_arch = "wasm32")]
        let arch = Architecture::Wasm32;
        #[cfg(not(any(
            target_arch = "x86_64",
            target_arch = "aarch64",
            target_arch = "wasm32"
        )))]
        let arch = Architecture::X86_64;

        TargetTriple {
            arch,
            os,
            format,
            features: Vec::new(),
            pic: true,
        }
    }

    pub fn x86_64_linux() -> Self {
        TargetTriple {
            arch: Architecture::X86_64,
            os: TargetOS::Linux,
            format: OutputFormat::Elf,
            features: vec!["sse2".to_string()],
            pic: true,
        }
    }

    pub fn x86_64_windows() -> Self {
        TargetTriple {
            arch: Architecture::X86_64,
            os: TargetOS::Windows,
            format: OutputFormat::Pe,
            features: vec!["sse2".to_string()],
            pic: false,
        }
    }

    pub fn aarch64_linux() -> Self {
        TargetTriple {
            arch: Architecture::Aarch64,
            os: TargetOS::Linux,
            format: OutputFormat::Elf,
            features: vec!["neon".to_string()],
            pic: true,
        }
    }

    pub fn wasm32_wasi() -> Self {
        TargetTriple {
            arch: Architecture::Wasm32,
            os: TargetOS::Wasi,
            format: OutputFormat::Wasm,
            features: Vec::new(),
            pic: false,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Register Definitions
// ─────────────────────────────────────────────────────────────────────────────

/// Physical register for x86-64
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum X86Reg {
    Rax = 0,
    Rcx = 1,
    Rdx = 2,
    Rbx = 3,
    Rsp = 4,
    Rbp = 5,
    Rsi = 6,
    Rdi = 7,
    R8 = 8,
    R9 = 9,
    R10 = 10,
    R11 = 11,
    R12 = 12,
    R13 = 13,
    R14 = 14,
    R15 = 15,
    // SSE/AVX
    Xmm0 = 16,
    Xmm1 = 17,
    Xmm2 = 18,
    Xmm3 = 19,
    Xmm4 = 20,
    Xmm5 = 21,
    Xmm6 = 22,
    Xmm7 = 23,
    Xmm8 = 24,
    Xmm9 = 25,
    Xmm10 = 26,
    Xmm11 = 27,
    Xmm12 = 28,
    Xmm13 = 29,
    Xmm14 = 30,
    Xmm15 = 31,
}

impl X86Reg {
    /// System V ABI caller-saved (volatile) registers
    pub fn caller_saved() -> &'static [X86Reg] {
        &[
            X86Reg::Rax,
            X86Reg::Rcx,
            X86Reg::Rdx,
            X86Reg::Rsi,
            X86Reg::Rdi,
            X86Reg::R8,
            X86Reg::R9,
            X86Reg::R10,
            X86Reg::R11,
        ]
    }

    /// System V ABI callee-saved (non-volatile) registers
    pub fn callee_saved() -> &'static [X86Reg] {
        &[
            X86Reg::Rbx,
            X86Reg::R12,
            X86Reg::R13,
            X86Reg::R14,
            X86Reg::R15,
        ]
    }

    /// System V ABI argument registers
    pub fn arg_regs_sysv() -> &'static [X86Reg] {
        &[
            X86Reg::Rdi,
            X86Reg::Rsi,
            X86Reg::Rdx,
            X86Reg::Rcx,
            X86Reg::R8,
            X86Reg::R9,
        ]
    }

    /// Windows x64 argument registers
    pub fn arg_regs_win64() -> &'static [X86Reg] {
        &[X86Reg::Rcx, X86Reg::Rdx, X86Reg::R8, X86Reg::R9]
    }

    /// SSE argument registers (System V)
    pub fn float_arg_regs() -> &'static [X86Reg] {
        &[
            X86Reg::Xmm0,
            X86Reg::Xmm1,
            X86Reg::Xmm2,
            X86Reg::Xmm3,
            X86Reg::Xmm4,
            X86Reg::Xmm5,
            X86Reg::Xmm6,
            X86Reg::Xmm7,
        ]
    }

    pub fn encoding(&self) -> u8 {
        *self as u8
    }

    pub fn is_extended(&self) -> bool {
        (*self as u8) >= 8 && (*self as u8) < 16
    }

    pub fn name(&self) -> &'static str {
        match self {
            X86Reg::Rax => "rax",
            X86Reg::Rcx => "rcx",
            X86Reg::Rdx => "rdx",
            X86Reg::Rbx => "rbx",
            X86Reg::Rsp => "rsp",
            X86Reg::Rbp => "rbp",
            X86Reg::Rsi => "rsi",
            X86Reg::Rdi => "rdi",
            X86Reg::R8 => "r8",
            X86Reg::R9 => "r9",
            X86Reg::R10 => "r10",
            X86Reg::R11 => "r11",
            X86Reg::R12 => "r12",
            X86Reg::R13 => "r13",
            X86Reg::R14 => "r14",
            X86Reg::R15 => "r15",
            X86Reg::Xmm0 => "xmm0",
            X86Reg::Xmm1 => "xmm1",
            X86Reg::Xmm2 => "xmm2",
            X86Reg::Xmm3 => "xmm3",
            X86Reg::Xmm4 => "xmm4",
            X86Reg::Xmm5 => "xmm5",
            X86Reg::Xmm6 => "xmm6",
            X86Reg::Xmm7 => "xmm7",
            X86Reg::Xmm8 => "xmm8",
            X86Reg::Xmm9 => "xmm9",
            X86Reg::Xmm10 => "xmm10",
            X86Reg::Xmm11 => "xmm11",
            X86Reg::Xmm12 => "xmm12",
            X86Reg::Xmm13 => "xmm13",
            X86Reg::Xmm14 => "xmm14",
            X86Reg::Xmm15 => "xmm15",
        }
    }
}

/// Physical register for AArch64
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Arm64Reg {
    // General purpose x0-x30
    X0,
    X1,
    X2,
    X3,
    X4,
    X5,
    X6,
    X7,
    X8,
    X9,
    X10,
    X11,
    X12,
    X13,
    X14,
    X15,
    X16,
    X17,
    X18,
    X19,
    X20,
    X21,
    X22,
    X23,
    X24,
    X25,
    X26,
    X27,
    X28,
    Fp, // x29, frame pointer
    Lr, // x30, link register
    Sp, // stack pointer (xzr in register contexts)
    // SIMD/FP v0-v31
    V0,
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
}

impl Arm64Reg {
    pub fn arg_regs() -> &'static [Arm64Reg] {
        &[
            Arm64Reg::X0,
            Arm64Reg::X1,
            Arm64Reg::X2,
            Arm64Reg::X3,
            Arm64Reg::X4,
            Arm64Reg::X5,
            Arm64Reg::X6,
            Arm64Reg::X7,
        ]
    }

    pub fn callee_saved() -> &'static [Arm64Reg] {
        &[
            Arm64Reg::X19,
            Arm64Reg::X20,
            Arm64Reg::X21,
            Arm64Reg::X22,
            Arm64Reg::X23,
            Arm64Reg::X24,
            Arm64Reg::X25,
            Arm64Reg::X26,
            Arm64Reg::X27,
            Arm64Reg::X28,
        ]
    }

    pub fn encoding(&self) -> u8 {
        match self {
            Arm64Reg::X0 => 0,
            Arm64Reg::X1 => 1,
            Arm64Reg::X2 => 2,
            Arm64Reg::X3 => 3,
            Arm64Reg::X4 => 4,
            Arm64Reg::X5 => 5,
            Arm64Reg::X6 => 6,
            Arm64Reg::X7 => 7,
            Arm64Reg::X8 => 8,
            Arm64Reg::X9 => 9,
            Arm64Reg::X10 => 10,
            Arm64Reg::X11 => 11,
            Arm64Reg::X12 => 12,
            Arm64Reg::X13 => 13,
            Arm64Reg::X14 => 14,
            Arm64Reg::X15 => 15,
            Arm64Reg::X16 => 16,
            Arm64Reg::X17 => 17,
            Arm64Reg::X18 => 18,
            Arm64Reg::X19 => 19,
            Arm64Reg::X20 => 20,
            Arm64Reg::X21 => 21,
            Arm64Reg::X22 => 22,
            Arm64Reg::X23 => 23,
            Arm64Reg::X24 => 24,
            Arm64Reg::X25 => 25,
            Arm64Reg::X26 => 26,
            Arm64Reg::X27 => 27,
            Arm64Reg::X28 => 28,
            Arm64Reg::Fp => 29,
            Arm64Reg::Lr => 30,
            Arm64Reg::Sp => 31,
            Arm64Reg::V0 => 0,
            Arm64Reg::V1 => 1,
            Arm64Reg::V2 => 2,
            Arm64Reg::V3 => 3,
            Arm64Reg::V4 => 4,
            Arm64Reg::V5 => 5,
            Arm64Reg::V6 => 6,
            Arm64Reg::V7 => 7,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Virtual Register and Linear Scan Register Allocation
// ─────────────────────────────────────────────────────────────────────────────

/// Virtual register (before allocation)
#[derive(Debug, Clone)]
pub struct VReg {
    /// Virtual register ID
    pub id: u32,
    /// Associated IR variable name
    pub ir_var: String,
    /// Type class (integer, float, vector)
    pub class: RegClass,
    /// Live range: (start_instruction, end_instruction)
    pub live_range: (usize, usize),
    /// Assigned physical register (after allocation)
    pub phys_reg: Option<u8>,
    /// Spill slot offset (if spilled)
    pub spill_slot: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegClass {
    Integer,
    Float,
    Vector,
}

/// Linear scan register allocator
pub struct LinearScanAllocator {
    /// Architecture
    arch: Architecture,
    /// Virtual registers to allocate
    vregs: Vec<VReg>,
    /// Available physical registers for integers
    int_regs: Vec<u8>,
    /// Available physical registers for floats
    float_regs: Vec<u8>,
    /// Active intervals (sorted by end point)
    active: Vec<usize>,
    /// Next spill slot offset
    next_spill: i32,
    /// Spill count
    spills: usize,
}

impl LinearScanAllocator {
    pub fn new(arch: Architecture) -> Self {
        let (int_regs, float_regs) = match arch {
            Architecture::X86_64 => (
                // Allocatable GPRs (exclude rsp, rbp)
                vec![0, 1, 2, 3, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
                // XMM registers
                vec![
                    16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
                ],
            ),
            Architecture::Aarch64 => (
                // x0-x28 (exclude x29=fp, x30=lr, x31=sp)
                (0..29).collect(),
                // v0-v7 (simplified)
                (0..8).collect(),
            ),
            Architecture::Wasm32 => (
                // WASM uses stack-based operands, but we model virtual regs
                (0..32).collect(),
                (0..16).collect(),
            ),
            _ => ((0..16).collect(), (0..8).collect()),
        };

        LinearScanAllocator {
            arch,
            vregs: Vec::new(),
            int_regs,
            float_regs,
            active: Vec::new(),
            next_spill: -8,
            spills: 0,
        }
    }

    /// Add a virtual register to allocate
    pub fn add_vreg(&mut self, ir_var: &str, class: RegClass, start: usize, end: usize) -> u32 {
        let id = self.vregs.len() as u32;
        self.vregs.push(VReg {
            id,
            ir_var: ir_var.to_string(),
            class,
            live_range: (start, end),
            phys_reg: None,
            spill_slot: None,
        });
        id
    }

    /// Perform register allocation using linear scan
    pub fn allocate(&mut self) -> Result<Vec<VReg>, String> {
        // Sort virtual registers by start point
        let mut indices: Vec<usize> = (0..self.vregs.len()).collect();
        indices.sort_by_key(|&i| self.vregs[i].live_range.0);

        let mut int_pool: Vec<u8> = self.int_regs.clone();
        let mut float_pool: Vec<u8> = self.float_regs.clone();

        for &idx in &indices {
            let start = self.vregs[idx].live_range.0;

            // Expire old intervals
            self.active.retain(|&active_idx| {
                let end = self.vregs[active_idx].live_range.1;
                if end <= start {
                    // Return register to pool
                    if let Some(reg) = self.vregs[active_idx].phys_reg {
                        match self.vregs[active_idx].class {
                            RegClass::Integer => int_pool.push(reg),
                            RegClass::Float | RegClass::Vector => float_pool.push(reg),
                        }
                    }
                    false
                } else {
                    true
                }
            });

            // Try to allocate a register
            let pool = match self.vregs[idx].class {
                RegClass::Integer => &mut int_pool,
                RegClass::Float | RegClass::Vector => &mut float_pool,
            };

            if let Some(reg) = pool.pop() {
                self.vregs[idx].phys_reg = Some(reg);
                self.active.push(idx);
                // Keep active sorted by end point
                self.active.sort_by_key(|&i| self.vregs[i].live_range.1);
            } else {
                // Spill: assign a stack slot
                self.vregs[idx].spill_slot = Some(self.next_spill);
                self.next_spill -= 8;
                self.spills += 1;
                debug!(
                    "RegAlloc: Spilling {} to stack[{}]",
                    self.vregs[idx].ir_var,
                    self.vregs[idx].spill_slot.unwrap()
                );
            }
        }

        info!(
            "RegAlloc: Allocated {} vregs, {} spilled",
            self.vregs.len(),
            self.spills
        );
        Ok(self.vregs.clone())
    }

    /// Get the total stack frame size needed for spills
    pub fn spill_area_size(&self) -> usize {
        (self.spills * 8 + 15) & !15 // Align to 16 bytes
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Machine Instruction Representation (MIR)
// ─────────────────────────────────────────────────────────────────────────────

/// Architecture-independent machine instruction
#[derive(Debug, Clone)]
pub enum MachineInst {
    /// Move between registers
    MovRR { dst: u8, src: u8 },
    /// Move immediate to register
    MovRI { dst: u8, imm: i64 },
    /// Load from memory [base + offset]
    Load {
        dst: u8,
        base: u8,
        offset: i32,
        size: u8,
    },
    /// Store to memory [base + offset]
    Store {
        src: u8,
        base: u8,
        offset: i32,
        size: u8,
    },
    /// Integer ALU operation
    AluRR { op: AluOp, dst: u8, src: u8 },
    /// Integer ALU with immediate
    AluRI { op: AluOp, dst: u8, imm: i32 },
    /// Compare two registers
    Cmp { left: u8, right: u8 },
    /// Compare register with immediate
    CmpI { reg: u8, imm: i32 },
    /// Set register based on condition
    SetCC { dst: u8, cc: CondCode },
    /// Conditional branch
    BranchCC { cc: CondCode, target: String },
    /// Unconditional branch
    Jump { target: String },
    /// Function call
    Call { target: String },
    /// Indirect call through register
    CallIndirect { reg: u8 },
    /// Return
    Return,
    /// Push register to stack
    Push { reg: u8 },
    /// Pop register from stack
    Pop { reg: u8 },
    /// Label
    Label { name: String },
    /// Floating-point move
    FMovRR { dst: u8, src: u8 },
    /// Floating-point ALU
    FAlu { op: AluOp, dst: u8, src: u8 },
    /// Int to float conversion
    CvtI2F { dst: u8, src: u8 },
    /// Float to int conversion
    CvtF2I { dst: u8, src: u8 },
    /// No operation
    Nop,
    /// System call / interrupt
    Syscall,
    /// LEA (load effective address) - x86 specific
    Lea {
        dst: u8,
        base: u8,
        index: u8,
        scale: u8,
        disp: i32,
    },
    /// Conditional move (CMOVcc) - x86 specific; dst := src if cc
    Cmov { dst: u8, src: u8, cc: CondCode },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AluOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Xor,
    Shl,
    Shr,
    Sar,
    Not,
    Neg,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CondCode {
    Eq,  // Equal
    Ne,  // Not Equal
    Lt,  // Less Than (signed)
    Le,  // Less or Equal (signed)
    Gt,  // Greater Than (signed)
    Ge,  // Greater or Equal (signed)
    Ltu, // Less Than (unsigned)
    Leu, // Less or Equal (unsigned)
    Gtu, // Greater Than (unsigned)
    Geu, // Greater or Equal (unsigned)
}

// ─────────────────────────────────────────────────────────────────────────────
// Instruction Selection (IR → MIR)
// ─────────────────────────────────────────────────────────────────────────────

/// Instruction selector: lowers IR instructions to machine instructions
pub struct InstructionSelector {
    arch: Architecture,
    os: TargetOS,
    /// Mapping: IR variable → virtual register ID
    var_to_vreg: HashMap<String, u8>,
    /// Next virtual register
    next_vreg: u8,
    /// Generated machine instructions
    instructions: Vec<MachineInst>,
}

impl InstructionSelector {
    pub fn new(arch: Architecture, os: TargetOS) -> Self {
        InstructionSelector {
            arch,
            os,
            var_to_vreg: HashMap::new(),
            next_vreg: 0,
            instructions: Vec::new(),
        }
    }

    /// Allocate a virtual register for an IR variable
    fn alloc_vreg(&mut self, var: &str) -> u8 {
        if let Some(&vreg) = self.var_to_vreg.get(var) {
            return vreg;
        }
        let vreg = self.next_vreg;
        self.next_vreg += 1;
        self.var_to_vreg.insert(var.to_string(), vreg);
        vreg
    }

    /// Lower an IR value to a virtual register
    fn lower_value(&mut self, value: &IrValue) -> u8 {
        match value {
            IrValue::Var(name) => self.alloc_vreg(name),
            IrValue::Const(constant) => {
                let vreg = self.next_vreg;
                self.next_vreg += 1;

                match constant {
                    IrConst::Int(v) => {
                        self.instructions
                            .push(MachineInst::MovRI { dst: vreg, imm: *v });
                    }
                    IrConst::Float(v) => {
                        // Load float constant via integer bitcast
                        self.instructions.push(MachineInst::MovRI {
                            dst: vreg,
                            imm: v.to_bits() as i64,
                        });
                    }
                    IrConst::Bool(v) => {
                        self.instructions.push(MachineInst::MovRI {
                            dst: vreg,
                            imm: if *v { 1 } else { 0 },
                        });
                    }
                    IrConst::Str(_) | IrConst::Null => {
                        self.instructions
                            .push(MachineInst::MovRI { dst: vreg, imm: 0 });
                    }
                }
                vreg
            }
        }
    }

    /// Select instructions for an entire IR function
    pub fn select_function(&mut self, func: &IrFunction) -> Vec<MachineInst> {
        self.instructions.clear();
        self.var_to_vreg.clear();
        self.next_vreg = 0;

        // Map parameters to argument registers
        let arg_regs: &[u8] = match (self.arch, self.os) {
            (Architecture::X86_64, TargetOS::Linux | TargetOS::MacOS) => {
                &[7, 6, 2, 1, 8, 9] // rdi, rsi, rdx, rcx, r8, r9
            }
            (Architecture::X86_64, TargetOS::Windows) => {
                &[1, 2, 8, 9] // rcx, rdx, r8, r9
            }
            (Architecture::Aarch64, _) => {
                &[0, 1, 2, 3, 4, 5, 6, 7] // x0-x7
            }
            _ => &[0, 1, 2, 3],
        };

        for (i, (name, _ty)) in func.params.iter().enumerate() {
            if i < arg_regs.len() {
                self.var_to_vreg.insert(name.clone(), arg_regs[i]);
            }
        }

        // --- Phi lowering: collect moves to emit at end of predecessor blocks ---
        let mut pending_moves: HashMap<String, Vec<MachineInst>> = HashMap::new();
        for block in &func.blocks {
            for inst in &block.instructions {
                if let IrInstruction::Phi {
                    dest,
                    ty: _,
                    incoming,
                } = inst
                {
                    let dest_reg = self.alloc_vreg(dest);
                    for (src, pred_label) in incoming {
                        let src_reg = self.alloc_vreg(src);
                        pending_moves.entry(pred_label.clone()).or_default().push(
                            MachineInst::MovRR {
                                dst: dest_reg,
                                src: src_reg,
                            },
                        );
                    }
                }
            }
        }

        // Select instructions for each block
        for block in &func.blocks {
            self.instructions.push(MachineInst::Label {
                name: block.label.clone(),
            });

            for inst in &block.instructions {
                self.select_instruction(inst);
            }

            // Emit any phi-moves that should occur at the end of this block (to prepare successors)
            if let Some(moves) = pending_moves.remove(&block.label) {
                for m in moves {
                    self.instructions.push(m);
                }
            }

            self.select_terminator(&block.terminator);
        }

        self.instructions.clone()
    }

    /// Select machine instructions for a single IR instruction
    fn select_instruction(&mut self, inst: &IrInstruction) {
        match inst {
            IrInstruction::BinOp {
                dest,
                op,
                left,
                right,
            } => {
                let l_reg = self.lower_value(left);
                let r_reg = self.lower_value(right);
                let d_reg = self.alloc_vreg(dest);

                // Move left to dest
                self.instructions.push(MachineInst::MovRR {
                    dst: d_reg,
                    src: l_reg,
                });

                let alu_op = match op {
                    IrBinOp::Add => AluOp::Add,
                    IrBinOp::Sub => AluOp::Sub,
                    IrBinOp::Mul => AluOp::Mul,
                    IrBinOp::Div => AluOp::Div,
                    IrBinOp::Mod => AluOp::Mod,
                    IrBinOp::And => AluOp::And,
                    IrBinOp::Or => AluOp::Or,
                    IrBinOp::Eq
                    | IrBinOp::Ne
                    | IrBinOp::Lt
                    | IrBinOp::Le
                    | IrBinOp::Gt
                    | IrBinOp::Ge => {
                        // Comparison: emit cmp + setcc
                        self.instructions.push(MachineInst::Cmp {
                            left: l_reg,
                            right: r_reg,
                        });
                        let cc = match op {
                            IrBinOp::Eq => CondCode::Eq,
                            IrBinOp::Ne => CondCode::Ne,
                            IrBinOp::Lt => CondCode::Lt,
                            IrBinOp::Le => CondCode::Le,
                            IrBinOp::Gt => CondCode::Gt,
                            IrBinOp::Ge => CondCode::Ge,
                            _ => unreachable!(),
                        };
                        self.instructions
                            .push(MachineInst::SetCC { dst: d_reg, cc });
                        return;
                    }
                };

                self.instructions.push(MachineInst::AluRR {
                    op: alu_op,
                    dst: d_reg,
                    src: r_reg,
                });
            }

            IrInstruction::Alloca { dest, ty } => {
                let d_reg = self.alloc_vreg(dest);
                // Stack allocation: compute address relative to frame pointer
                let size = self.type_size(ty) as i32;
                self.instructions.push(MachineInst::AluRI {
                    op: AluOp::Sub,
                    dst: 4,                 // rsp
                    imm: (size + 15) & !15, // Align to 16
                });
                self.instructions
                    .push(MachineInst::MovRR { dst: d_reg, src: 4 });
            }

            IrInstruction::Load { dest, ptr, ty } => {
                let d_reg = self.alloc_vreg(dest);
                let p_reg = self.alloc_vreg(ptr);
                let size = self.type_size(ty) as u8;
                self.instructions.push(MachineInst::Load {
                    dst: d_reg,
                    base: p_reg,
                    offset: 0,
                    size,
                });
            }

            IrInstruction::Store { ptr, value } => {
                let p_reg = self.alloc_vreg(ptr);
                let v_reg = self.lower_value(value);
                self.instructions.push(MachineInst::Store {
                    src: v_reg,
                    base: p_reg,
                    offset: 0,
                    size: 8,
                });
            }

            IrInstruction::Call { dest, func, args } => {
                // Load arguments into ABI registers
                let arg_regs: Vec<u8> = match (self.arch, self.os) {
                    (Architecture::X86_64, TargetOS::Linux | TargetOS::MacOS) => {
                        vec![7, 6, 2, 1, 8, 9]
                    }
                    (Architecture::X86_64, TargetOS::Windows) => vec![1, 2, 8, 9],
                    (Architecture::Aarch64, _) => (0..8).collect(),
                    _ => vec![0, 1, 2, 3],
                };

                for (i, arg) in args.iter().enumerate() {
                    if i < arg_regs.len() {
                        let v_reg = self.lower_value(arg);
                        self.instructions.push(MachineInst::MovRR {
                            dst: arg_regs[i],
                            src: v_reg,
                        });
                    } else {
                        // Pass on stack
                        let v_reg = self.lower_value(arg);
                        self.instructions.push(MachineInst::Push { reg: v_reg });
                    }
                }

                self.instructions.push(MachineInst::Call {
                    target: func.clone(),
                });

                // Store return value
                if let Some(dest_name) = dest {
                    let d_reg = self.alloc_vreg(dest_name);
                    let ret_reg = match self.arch {
                        Architecture::X86_64 => 0,  // rax
                        Architecture::Aarch64 => 0, // x0
                        _ => 0,
                    };
                    if d_reg != ret_reg {
                        self.instructions.push(MachineInst::MovRR {
                            dst: d_reg,
                            src: ret_reg,
                        });
                    }
                }
            }

            IrInstruction::Cast {
                dest,
                value,
                to_type,
            } => {
                let s_reg = self.lower_value(value);
                let d_reg = self.alloc_vreg(dest);

                match to_type {
                    IrType::F64 | IrType::F32 => {
                        self.instructions.push(MachineInst::CvtI2F {
                            dst: d_reg,
                            src: s_reg,
                        });
                    }
                    IrType::I32 | IrType::I64 if matches!(to_type, IrType::F64 | IrType::F32) => {
                        self.instructions.push(MachineInst::CvtF2I {
                            dst: d_reg,
                            src: s_reg,
                        });
                    }
                    _ => {
                        // Zero/sign extend or truncate
                        self.instructions.push(MachineInst::MovRR {
                            dst: d_reg,
                            src: s_reg,
                        });
                    }
                }
            }

            IrInstruction::GetField { dest, ptr, field } => {
                let o_reg = self.alloc_vreg(ptr);
                let d_reg = self.alloc_vreg(dest);
                let offset = (*field as i32) * 8; // Assume 8-byte fields
                self.instructions.push(MachineInst::Load {
                    dst: d_reg,
                    base: o_reg,
                    offset,
                    size: 8,
                });
            }

            IrInstruction::Select {
                dest,
                cond,
                then_val,
                else_val,
            } => {
                let c_reg = self.lower_value(cond);
                let t_reg = self.lower_value(then_val);
                let f_reg = self.lower_value(else_val);
                let d_reg = self.alloc_vreg(dest);
                if self.arch == Architecture::X86_64 {
                    // Emit CMOVcc: move then_val into dest if cond != 0, otherwise keep else_val
                    // First set dest = else_val
                    self.instructions.push(MachineInst::MovRR {
                        dst: d_reg,
                        src: f_reg,
                    });
                    // Then CMOV when cond != 0
                    self.instructions.push(MachineInst::Cmp {
                        left: c_reg,
                        right: c_reg,
                    });
                    self.instructions.push(MachineInst::Cmov {
                        dst: d_reg,
                        src: t_reg,
                        cc: CondCode::Ne,
                    });
                } else {
                    // Fallback: branch-based select (existing implementation)
                    self.instructions.push(MachineInst::Cmp {
                        left: c_reg,
                        right: c_reg,
                    });
                    self.instructions.push(MachineInst::MovRR {
                        dst: d_reg,
                        src: f_reg,
                    });
                    let label_end = format!("select_end_{}", d_reg);
                    self.instructions.push(MachineInst::BranchCC {
                        cc: CondCode::Eq,
                        target: label_end.clone(),
                    });
                    self.instructions.push(MachineInst::MovRR {
                        dst: d_reg,
                        src: t_reg,
                    });
                    self.instructions
                        .push(MachineInst::Label { name: label_end });
                }
            }

            IrInstruction::Phi {
                dest,
                ty: _,
                incoming: _,
            } => {
                // Phi nodes are resolved during register allocation
                // For now, just allocate a vreg for the destination
                let _d_reg = self.alloc_vreg(dest);
            }

            _ => {
                // Unsupported instructions log a warning but do not emit code
                // The IR validates these should not appear in well-formed code
                eprintln!("WARNING: Unsupported instruction in native codegen; may produce incorrect output");
            }
        }
    }

    /// Select machine instructions for a terminator
    fn select_terminator(&mut self, term: &IrTerminator) {
        match term {
            IrTerminator::Return(Some(value)) => {
                let v_reg = self.lower_value(value);
                let ret_reg = match self.arch {
                    Architecture::X86_64 => 0,  // rax
                    Architecture::Aarch64 => 0, // x0
                    _ => 0,
                };
                if v_reg != ret_reg {
                    self.instructions.push(MachineInst::MovRR {
                        dst: ret_reg,
                        src: v_reg,
                    });
                }
                self.instructions.push(MachineInst::Return);
            }
            IrTerminator::Return(None) => {
                self.instructions.push(MachineInst::Return);
            }
            IrTerminator::Branch(target) => {
                self.instructions.push(MachineInst::Jump {
                    target: target.clone(),
                });
            }
            IrTerminator::CondBranch {
                cond,
                then_label,
                else_label,
            } => {
                let c_reg = self.lower_value(cond);
                // test cond, cond
                self.instructions
                    .push(MachineInst::CmpI { reg: c_reg, imm: 0 });
                self.instructions.push(MachineInst::BranchCC {
                    cc: CondCode::Ne,
                    target: then_label.clone(),
                });
                self.instructions.push(MachineInst::Jump {
                    target: else_label.clone(),
                });
            }
            IrTerminator::Unreachable => {
                // Emit ud2 (x86) or brk (arm64)
                self.instructions.push(MachineInst::Nop); // placeholder
            }
        }
    }

    fn type_size(&self, ty: &IrType) -> usize {
        match ty {
            IrType::Void => 0,
            IrType::I8 | IrType::Bool => 1,
            IrType::I16 => 2,
            IrType::I32 | IrType::F32 => 4,
            IrType::I64 | IrType::F64 | IrType::Ptr(_) => 8,
            IrType::Array(inner, count) => self.type_size(inner) * count,
            IrType::Struct(_) => 8, // Struct is name-based, assume pointer-sized
            _ => 8,                 // Default pointer-sized
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Binary Emitters
// ─────────────────────────────────────────────────────────────────────────────

/// x86-64 binary emitter
pub struct X86Emitter {
    code: Vec<u8>,
    labels: HashMap<String, usize>,
    fixups: Vec<(usize, String, bool)>, // (offset, label, is_relative)
}

impl Default for X86Emitter {
    fn default() -> Self {
        Self::new()
    }
}

impl X86Emitter {
    pub fn new() -> Self {
        X86Emitter {
            code: Vec::with_capacity(4096),
            labels: HashMap::new(),
            fixups: Vec::new(),
        }
    }

    /// Emit a machine instruction as x86-64 bytes
    pub fn emit(&mut self, inst: &MachineInst) {
        match inst {
            MachineInst::Label { name } => {
                self.labels.insert(name.clone(), self.code.len());
            }

            MachineInst::MovRR { dst, src } => {
                let rex = 0x48 | ((*src >> 3) << 2) | ((*dst >> 3) & 1);
                self.code.push(rex);
                self.code.push(0x89);
                self.code.push(0xC0 | ((*src & 7) << 3) | (*dst & 7));
            }

            MachineInst::MovRI { dst, imm } => {
                let rex = 0x48 | ((*dst >> 3) & 1);
                self.code.push(rex);
                self.code.push(0xB8 + (*dst & 7));
                self.code.extend_from_slice(&imm.to_le_bytes());
            }

            MachineInst::Load {
                dst,
                base,
                offset,
                size,
            } => {
                // mov dst, [base + offset]
                let rex = 0x48 | ((*dst >> 3) << 2) | ((*base >> 3) & 1);
                self.code.push(rex);
                match size {
                    1 => self.code.push(0x8A), // movzx
                    2 => {
                        self.code.push(0x66);
                        self.code.push(0x8B);
                    }
                    _ => self.code.push(0x8B),
                }
                if *offset == 0 && (*base & 7) != 5 {
                    self.code.push((*dst & 7) << 3 | (*base & 7));
                } else if *offset >= -128 && *offset <= 127 {
                    self.code.push(0x40 | (*dst & 7) << 3 | (*base & 7));
                    self.code.push(*offset as u8);
                } else {
                    self.code.push(0x80 | (*dst & 7) << 3 | (*base & 7));
                    self.code.extend_from_slice(&offset.to_le_bytes());
                }
            }

            MachineInst::Store {
                src,
                base,
                offset,
                size: _,
            } => {
                // mov [base + offset], src
                let rex = 0x48 | ((*src >> 3) << 2) | ((*base >> 3) & 1);
                self.code.push(rex);
                self.code.push(0x89);
                if *offset == 0 && (*base & 7) != 5 {
                    self.code.push((*src & 7) << 3 | (*base & 7));
                } else if *offset >= -128 && *offset <= 127 {
                    self.code.push(0x40 | (*src & 7) << 3 | (*base & 7));
                    self.code.push(*offset as u8);
                } else {
                    self.code.push(0x80 | (*src & 7) << 3 | (*base & 7));
                    self.code.extend_from_slice(&offset.to_le_bytes());
                }
            }

            MachineInst::AluRR { op, dst, src } => {
                let rex = 0x48 | ((*src >> 3) << 2) | ((*dst >> 3) & 1);
                self.code.push(rex);
                match op {
                    AluOp::Add => {
                        self.code.push(0x01);
                        self.code.push(0xC0 | ((*src & 7) << 3) | (*dst & 7));
                    }
                    AluOp::Sub => {
                        self.code.push(0x29);
                        self.code.push(0xC0 | ((*src & 7) << 3) | (*dst & 7));
                    }
                    AluOp::Mul => {
                        self.code.extend_from_slice(&[0x0F, 0xAF]);
                        self.code.push(0xC0 | ((*dst & 7) << 3) | (*src & 7));
                    }
                    AluOp::And => {
                        self.code.push(0x21);
                        self.code.push(0xC0 | ((*src & 7) << 3) | (*dst & 7));
                    }
                    AluOp::Or => {
                        self.code.push(0x09);
                        self.code.push(0xC0 | ((*src & 7) << 3) | (*dst & 7));
                    }
                    AluOp::Xor => {
                        self.code.push(0x31);
                        self.code.push(0xC0 | ((*src & 7) << 3) | (*dst & 7));
                    }
                    AluOp::Div => {
                        // idiv uses rdx:rax
                        // cqo; idiv src
                        self.code.extend_from_slice(&[0x48, 0x99]); // cqo
                        self.code.push(0x48 | ((*src >> 3) & 1));
                        self.code.push(0xF7);
                        self.code.push(0xF8 | (*src & 7));
                    }
                    _ => {
                        self.code.push(0x90); // nop placeholder
                    }
                }
            }

            MachineInst::AluRI { op, dst, imm } => {
                let rex = 0x48 | ((*dst >> 3) & 1);
                self.code.push(rex);

                if *imm >= -128 && *imm <= 127 {
                    self.code.push(0x83);
                    let op_code = match op {
                        AluOp::Add => 0,
                        AluOp::Sub => 5,
                        AluOp::And => 4,
                        AluOp::Or => 1,
                        AluOp::Xor => 6,
                        _ => 0,
                    };
                    self.code.push(0xC0 | (op_code << 3) | (*dst & 7));
                    self.code.push(*imm as u8);
                } else {
                    self.code.push(0x81);
                    let op_code = match op {
                        AluOp::Add => 0,
                        AluOp::Sub => 5,
                        AluOp::And => 4,
                        AluOp::Or => 1,
                        AluOp::Xor => 6,
                        _ => 0,
                    };
                    self.code.push(0xC0 | (op_code << 3) | (*dst & 7));
                    self.code.extend_from_slice(&imm.to_le_bytes());
                }
            }

            MachineInst::Cmp { left, right } => {
                let rex = 0x48 | ((*right >> 3) << 2) | ((*left >> 3) & 1);
                self.code.push(rex);
                self.code.push(0x39);
                self.code.push(0xC0 | ((*right & 7) << 3) | (*left & 7));
            }

            MachineInst::CmpI { reg, imm } => {
                let rex = 0x48 | ((*reg >> 3) & 1);
                self.code.push(rex);
                if *imm >= -128 && *imm <= 127 {
                    self.code.push(0x83);
                    self.code.push(0xF8 | (*reg & 7));
                    self.code.push(*imm as u8);
                } else {
                    self.code.push(0x81);
                    self.code.push(0xF8 | (*reg & 7));
                    self.code.extend_from_slice(&imm.to_le_bytes());
                }
            }

            MachineInst::SetCC { dst, cc } => {
                let cc_byte = match cc {
                    CondCode::Eq => 0x04,
                    CondCode::Ne => 0x05,
                    CondCode::Lt => 0x0C,
                    CondCode::Le => 0x0E,
                    CondCode::Gt => 0x0F,
                    CondCode::Ge => 0x0D,
                    CondCode::Ltu => 0x02,
                    CondCode::Leu => 0x06,
                    CondCode::Gtu => 0x07,
                    CondCode::Geu => 0x03,
                };
                self.code.extend_from_slice(&[0x0F, 0x90 + cc_byte]);
                self.code.push(0xC0 | (*dst & 7));
                // movzx to clear upper bits
                self.code.extend_from_slice(&[0x48, 0x0F, 0xB6]);
                self.code.push(0xC0 | ((*dst & 7) << 3) | (*dst & 7));
            }

            MachineInst::BranchCC { cc, target } => {
                let cc_byte = match cc {
                    CondCode::Eq => 0x04,
                    CondCode::Ne => 0x05,
                    CondCode::Lt => 0x0C,
                    CondCode::Le => 0x0E,
                    CondCode::Gt => 0x0F,
                    CondCode::Ge => 0x0D,
                    CondCode::Ltu => 0x02,
                    CondCode::Leu => 0x06,
                    CondCode::Gtu => 0x07,
                    CondCode::Geu => 0x03,
                };
                self.code.extend_from_slice(&[0x0F, 0x80 + cc_byte]);
                self.fixups.push((self.code.len(), target.clone(), true));
                self.code.extend_from_slice(&[0x00; 4]); // placeholder
            }

            MachineInst::Cmov { dst, src, cc } => {
                // x86 CMOVcc: opcode 0x0F 0x40+cc, ModR/M with reg=dst, r/m=src
                let rex = 0x48 | ((*src >> 3) << 2) | ((*dst >> 3) & 1);
                self.code.push(rex);
                let cc_byte = match cc {
                    CondCode::Eq => 0x04,
                    CondCode::Ne => 0x05,
                    CondCode::Lt => 0x0C,
                    CondCode::Le => 0x0E,
                    CondCode::Gt => 0x0F,
                    CondCode::Ge => 0x0D,
                    CondCode::Ltu => 0x02,
                    CondCode::Leu => 0x06,
                    CondCode::Gtu => 0x07,
                    CondCode::Geu => 0x03,
                };
                self.code.push(0x0F);
                self.code.push(0x40 + cc_byte);
                self.code.push(0xC0 | ((*dst & 7) << 3) | (*src & 7));
            }

            MachineInst::Jump { target } => {
                self.code.push(0xE9);
                self.fixups.push((self.code.len(), target.clone(), true));
                self.code.extend_from_slice(&[0x00; 4]); // placeholder
            }

            MachineInst::Call { target } => {
                self.code.push(0xE8);
                self.fixups.push((self.code.len(), target.clone(), true));
                self.code.extend_from_slice(&[0x00; 4]); // placeholder
            }

            MachineInst::CallIndirect { reg } => {
                if *reg >= 8 {
                    self.code.push(0x41);
                }
                self.code.push(0xFF);
                self.code.push(0xD0 | (*reg & 7));
            }

            MachineInst::Return => {
                self.code.push(0xC3);
            }

            MachineInst::Push { reg } => {
                if *reg >= 8 {
                    self.code.push(0x41);
                }
                self.code.push(0x50 + (*reg & 7));
            }

            MachineInst::Pop { reg } => {
                if *reg >= 8 {
                    self.code.push(0x41);
                }
                self.code.push(0x58 + (*reg & 7));
            }

            MachineInst::Nop => {
                self.code.push(0x90);
            }

            MachineInst::Syscall => {
                self.code.extend_from_slice(&[0x0F, 0x05]);
            }

            _ => {
                // For unimplemented machine instructions, emit a NOP and log warning
                eprintln!("WARNING: Unimplemented machine instruction; using NOP placeholder");
                self.code.push(0x90);
            }
        }
    }

    /// Resolve all label fixups
    pub fn resolve_fixups(&mut self) {
        for (offset, label, is_relative) in &self.fixups {
            if let Some(&target_pos) = self.labels.get(label) {
                if *is_relative {
                    let rel = target_pos as i32 - (*offset as i32 + 4);
                    self.code[*offset..*offset + 4].copy_from_slice(&rel.to_le_bytes());
                } else {
                    self.code[*offset..*offset + 8]
                        .copy_from_slice(&(target_pos as u64).to_le_bytes());
                }
            }
        }
    }

    /// Get the emitted code
    pub fn code(&self) -> &[u8] {
        &self.code
    }
}

/// AArch64 binary emitter
pub struct Arm64Emitter {
    code: Vec<u8>,
    labels: HashMap<String, usize>,
    fixups: Vec<(usize, String)>,
}

impl Arm64Emitter {
    pub fn new() -> Self {
        Arm64Emitter {
            code: Vec::with_capacity(4096),
            labels: HashMap::new(),
            fixups: Vec::new(),
        }
    }

    /// Emit a machine instruction as AArch64 bytes (32-bit fixed-width)
    pub fn emit(&mut self, inst: &MachineInst) {
        match inst {
            MachineInst::Label { name } => {
                self.labels.insert(name.clone(), self.code.len());
            }

            MachineInst::MovRR { dst, src } => {
                // mov xD, xS → ORR xD, XZR, xS
                let encoding = 0xAA0003E0u32 | ((*src as u32 & 0x1F) << 16) | (*dst as u32 & 0x1F);
                self.code.extend_from_slice(&encoding.to_le_bytes());
            }

            MachineInst::MovRI { dst, imm } => {
                // MOVZ xD, #imm16 (lower 16 bits)
                let imm16 = *imm as u16;
                let encoding = 0xD2800000u32 | ((imm16 as u32) << 5) | (*dst as u32 & 0x1F);
                self.code.extend_from_slice(&encoding.to_le_bytes());

                // If imm doesn't fit in 16 bits, emit MOVK for upper bits
                if *imm > 0xFFFF || *imm < 0 {
                    let imm16_hi = ((*imm >> 16) & 0xFFFF) as u16;
                    let encoding = 0xF2A00000u32 | ((imm16_hi as u32) << 5) | (*dst as u32 & 0x1F);
                    self.code.extend_from_slice(&encoding.to_le_bytes());
                }
            }

            MachineInst::AluRR { op, dst, src } => {
                let encoding = match op {
                    AluOp::Add => {
                        // ADD xD, xD, xS
                        0x8B000000u32
                            | ((*src as u32 & 0x1F) << 16)
                            | ((*dst as u32 & 0x1F) << 5)
                            | (*dst as u32 & 0x1F)
                    }
                    AluOp::Sub => {
                        // SUB xD, xD, xS
                        0xCB000000u32
                            | ((*src as u32 & 0x1F) << 16)
                            | ((*dst as u32 & 0x1F) << 5)
                            | (*dst as u32 & 0x1F)
                    }
                    AluOp::Mul => {
                        // MUL xD, xD, xS → MADD xD, xD, xS, XZR
                        0x9B007C00u32
                            | ((*src as u32 & 0x1F) << 16)
                            | ((*dst as u32 & 0x1F) << 5)
                            | (*dst as u32 & 0x1F)
                    }
                    AluOp::And => {
                        0x8A000000u32
                            | ((*src as u32 & 0x1F) << 16)
                            | ((*dst as u32 & 0x1F) << 5)
                            | (*dst as u32 & 0x1F)
                    }
                    AluOp::Or => {
                        0xAA000000u32
                            | ((*src as u32 & 0x1F) << 16)
                            | ((*dst as u32 & 0x1F) << 5)
                            | (*dst as u32 & 0x1F)
                    }
                    _ => 0xD503201Fu32, // NOP
                };
                self.code.extend_from_slice(&encoding.to_le_bytes());
            }

            MachineInst::Cmp { left, right } => {
                // CMP xL, xR → SUBS XZR, xL, xR
                let encoding =
                    0xEB00001Fu32 | ((*right as u32 & 0x1F) << 16) | ((*left as u32 & 0x1F) << 5);
                self.code.extend_from_slice(&encoding.to_le_bytes());
            }

            MachineInst::BranchCC { cc, target } => {
                let cond = match cc {
                    CondCode::Eq => 0x0,
                    CondCode::Ne => 0x1,
                    CondCode::Lt => 0xB,
                    CondCode::Le => 0xD,
                    CondCode::Gt => 0xC,
                    CondCode::Ge => 0xA,
                    _ => 0x0,
                };
                // B.cond offset
                let encoding = 0x54000000u32 | cond;
                self.fixups.push((self.code.len(), target.clone()));
                self.code.extend_from_slice(&encoding.to_le_bytes());
            }

            MachineInst::Jump { target } => {
                // B offset
                self.fixups.push((self.code.len(), target.clone()));
                self.code.extend_from_slice(&0x14000000u32.to_le_bytes());
            }

            MachineInst::Return => {
                // RET (x30)
                self.code.extend_from_slice(&0xD65F03C0u32.to_le_bytes());
            }

            MachineInst::Call { target } => {
                // BL offset
                self.fixups.push((self.code.len(), target.clone()));
                self.code.extend_from_slice(&0x94000000u32.to_le_bytes());
            }

            MachineInst::Nop => {
                self.code.extend_from_slice(&0xD503201Fu32.to_le_bytes());
            }

            _ => {
                // Unimplemented instructions: emit NOP and log warning
                eprintln!(
                    "WARNING: Unimplemented machine instruction (ARM64); using NOP placeholder"
                );
                self.code.extend_from_slice(&0xD503201Fu32.to_le_bytes());
            }
        }
    }

    /// Resolve fixups
    pub fn resolve_fixups(&mut self) {
        for (offset, label) in &self.fixups {
            if let Some(&target_pos) = self.labels.get(label) {
                let rel = (target_pos as i32 - *offset as i32) / 4;
                let mut word = u32::from_le_bytes([
                    self.code[*offset],
                    self.code[*offset + 1],
                    self.code[*offset + 2],
                    self.code[*offset + 3],
                ]);
                // Patch imm19 or imm26 field
                if (word >> 24) == 0x54 {
                    // B.cond: imm19 at bits [23:5]
                    word |= (rel as u32 & 0x7FFFF) << 5;
                } else {
                    // B/BL: imm26 at bits [25:0]
                    word |= rel as u32 & 0x3FFFFFF;
                }
                self.code[*offset..*offset + 4].copy_from_slice(&word.to_le_bytes());
            }
        }
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }
}

/// WebAssembly binary emitter
pub struct WasmEmitter {
    /// Wasm binary sections
    sections: Vec<WasmSection>,
    /// Function type signatures
    types: Vec<WasmFuncType>,
    /// Import entries
    imports: Vec<WasmImport>,
    /// Function bodies
    functions: Vec<Vec<u8>>,
    /// Export entries
    exports: Vec<WasmExport>,
}

#[derive(Debug, Clone)]
struct WasmSection {
    id: u8,
    data: Vec<u8>,
}

#[derive(Debug, Clone)]
struct WasmFuncType {
    params: Vec<u8>, // Wasm value types
    results: Vec<u8>,
}

#[derive(Debug, Clone)]
struct WasmImport {
    module: String,
    name: String,
    kind: u8,
    type_idx: u32,
}

#[derive(Debug, Clone)]
struct WasmExport {
    name: String,
    kind: u8,
    index: u32,
}

impl Default for WasmEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl WasmEmitter {
    pub fn new() -> Self {
        WasmEmitter {
            sections: Vec::new(),
            types: Vec::new(),
            imports: Vec::new(),
            functions: Vec::new(),
            exports: Vec::new(),
        }
    }

    /// Add a function type
    pub fn add_type(&mut self, params: &[IrType], result: &IrType) -> u32 {
        let idx = self.types.len() as u32;
        self.types.push(WasmFuncType {
            params: params.iter().map(|t| self.ir_to_wasm_type(t)).collect(),
            results: if matches!(result, IrType::Void) {
                vec![]
            } else {
                vec![self.ir_to_wasm_type(result)]
            },
        });
        idx
    }

    /// Add a function
    pub fn add_function(&mut self, func: &IrFunction) -> u32 {
        let idx = self.functions.len() as u32;
        let body = self.emit_function_body(func);
        self.functions.push(body);

        // Export with its name
        self.exports.push(WasmExport {
            name: func.name.clone(),
            kind: 0x00, // funcref
            index: idx + self.imports.len() as u32,
        });

        idx
    }

    /// Emit function body as Wasm bytecode
    fn emit_function_body(&self, func: &IrFunction) -> Vec<u8> {
        let mut body = Vec::new();

        // Local declarations
        let local_count = func.locals.len() as u32;
        self.encode_u32(&mut body, if local_count > 0 { 1 } else { 0 }); // 1 local declaration
        if local_count > 0 {
            self.encode_u32(&mut body, local_count);
            body.push(0x7E); // i64
        }

        // Instructions
        for block in &func.blocks {
            for inst in &block.instructions {
                self.emit_wasm_instruction(&mut body, inst);
            }

            // Terminator
            match &block.terminator {
                IrTerminator::Return(Some(value)) => {
                    self.emit_wasm_value(&mut body, value);
                    body.push(0x0F); // return
                }
                IrTerminator::Return(None) => {
                    body.push(0x0F); // return
                }
                _ => {}
            }
        }

        body.push(0x0B); // end
        body
    }

    /// Emit a single Wasm instruction
    fn emit_wasm_instruction(&self, body: &mut Vec<u8>, inst: &IrInstruction) {
        match inst {
            IrInstruction::BinOp {
                dest: _,
                op,
                left,
                right,
            } => {
                self.emit_wasm_value(body, left);
                self.emit_wasm_value(body, right);

                match op {
                    IrBinOp::Add => body.push(0x7C), // i64.add
                    IrBinOp::Sub => body.push(0x7D), // i64.sub
                    IrBinOp::Mul => body.push(0x7E), // i64.mul
                    IrBinOp::Div => body.push(0x7F), // i64.div_s
                    IrBinOp::Eq => body.push(0x51),  // i64.eq
                    IrBinOp::Ne => body.push(0x52),  // i64.ne
                    IrBinOp::Lt => body.push(0x53),  // i64.lt_s
                    IrBinOp::Gt => body.push(0x55),  // i64.gt_s
                    IrBinOp::Le => body.push(0x57),  // i64.le_s
                    IrBinOp::Ge => body.push(0x59),  // i64.ge_s
                    _ => body.push(0x7C),            // default to add
                }

                // local.set dest
                body.push(0x21);
                self.encode_u32(body, 0); // local index placeholder
            }

            IrInstruction::Call {
                dest: _,
                func: _,
                args,
            } => {
                for arg in args {
                    self.emit_wasm_value(body, arg);
                }
                body.push(0x10); // call
                self.encode_u32(body, 0); // function index placeholder
            }

            _ => {} // Skip unimplemented
        }
    }

    /// Emit a value onto the Wasm stack
    fn emit_wasm_value(&self, body: &mut Vec<u8>, value: &IrValue) {
        match value {
            IrValue::Const(IrConst::Int(v)) => {
                body.push(0x42); // i64.const
                self.encode_i64(body, *v);
            }
            IrValue::Const(IrConst::Float(v)) => {
                body.push(0x44); // f64.const
                body.extend_from_slice(&v.to_le_bytes());
            }
            IrValue::Const(IrConst::Bool(v)) => {
                body.push(0x41); // i32.const
                body.push(if *v { 1 } else { 0 });
            }
            IrValue::Var(_name) => {
                body.push(0x20); // local.get
                self.encode_u32(body, 0); // local index placeholder
            }
            _ => {}
        }
    }

    /// LEB128 encode unsigned 32-bit integer
    fn encode_u32(&self, buf: &mut Vec<u8>, mut value: u32) {
        loop {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            buf.push(byte);
            if value == 0 {
                break;
            }
        }
    }

    /// LEB128 encode signed 64-bit integer
    fn encode_i64(&self, buf: &mut Vec<u8>, mut value: i64) {
        loop {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;
            let more = !((value == 0 && (byte & 0x40) == 0) || (value == -1 && (byte & 0x40) != 0));
            if more {
                byte |= 0x80;
            }
            buf.push(byte);
            if !more {
                break;
            }
        }
    }

    fn ir_to_wasm_type(&self, ty: &IrType) -> u8 {
        match ty {
            IrType::I32 | IrType::Bool => 0x7F,   // i32
            IrType::I64 | IrType::Ptr(_) => 0x7E, // i64
            IrType::F32 => 0x7D,                  // f32
            IrType::F64 => 0x7C,                  // f64
            _ => 0x7E,                            // default i64
        }
    }

    /// Build the final Wasm binary
    pub fn build(&self) -> Vec<u8> {
        let mut binary = Vec::new();

        // Wasm magic number
        binary.extend_from_slice(&[0x00, 0x61, 0x73, 0x6D]);
        // Version 1
        binary.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);

        // Type section (1)
        if !self.types.is_empty() {
            let mut section = Vec::new();
            self.encode_u32(&mut section, self.types.len() as u32);
            for ft in &self.types {
                section.push(0x60); // func type
                self.encode_u32(&mut section, ft.params.len() as u32);
                section.extend_from_slice(&ft.params);
                self.encode_u32(&mut section, ft.results.len() as u32);
                section.extend_from_slice(&ft.results);
            }
            binary.push(0x01); // section id
            self.encode_u32(&mut binary, section.len() as u32);
            binary.extend_from_slice(&section);
        }

        // Function section (3) - type indices
        if !self.functions.is_empty() {
            let mut section = Vec::new();
            self.encode_u32(&mut section, self.functions.len() as u32);
            for i in 0..self.functions.len() {
                self.encode_u32(
                    &mut section,
                    i.min(self.types.len().saturating_sub(1)) as u32,
                );
            }
            binary.push(0x03);
            self.encode_u32(&mut binary, section.len() as u32);
            binary.extend_from_slice(&section);
        }

        // Export section (7)
        if !self.exports.is_empty() {
            let mut section = Vec::new();
            self.encode_u32(&mut section, self.exports.len() as u32);
            for export in &self.exports {
                let name_bytes = export.name.as_bytes();
                self.encode_u32(&mut section, name_bytes.len() as u32);
                section.extend_from_slice(name_bytes);
                section.push(export.kind);
                self.encode_u32(&mut section, export.index);
            }
            binary.push(0x07);
            self.encode_u32(&mut binary, section.len() as u32);
            binary.extend_from_slice(&section);
        }

        // Code section (10)
        if !self.functions.is_empty() {
            let mut section = Vec::new();
            self.encode_u32(&mut section, self.functions.len() as u32);
            for body in &self.functions {
                self.encode_u32(&mut section, body.len() as u32);
                section.extend_from_slice(body);
            }
            binary.push(0x0A);
            self.encode_u32(&mut binary, section.len() as u32);
            binary.extend_from_slice(&section);
        }

        binary
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Object File Output
// ─────────────────────────────────────────────────────────────────────────────

/// ELF output builder (minimal)
pub struct ElfBuilder {
    /// Machine code sections
    sections: Vec<ElfSection>,
    /// Symbol table
    symbols: Vec<ElfSymbol>,
    /// Target architecture
    arch: Architecture,
}

#[derive(Debug, Clone)]
struct ElfSection {
    name: String,
    section_type: u32,
    flags: u64,
    data: Vec<u8>,
    alignment: u64,
}

#[derive(Debug, Clone)]
struct ElfSymbol {
    name: String,
    value: u64,
    size: u64,
    section_idx: u16,
    bind: u8,     // STB_LOCAL, STB_GLOBAL
    sym_type: u8, // STT_FUNC, STT_OBJECT
}

impl ElfBuilder {
    pub fn new(arch: Architecture) -> Self {
        ElfBuilder {
            sections: Vec::new(),
            symbols: Vec::new(),
            arch,
        }
    }

    /// Add a .text section with code
    pub fn add_text(&mut self, code: Vec<u8>) {
        self.sections.push(ElfSection {
            name: ".text".to_string(),
            section_type: 1, // SHT_PROGBITS
            flags: 0x6,      // SHF_ALLOC | SHF_EXECINSTR
            data: code,
            alignment: 16,
        });
    }

    /// Add a .data section
    pub fn add_data(&mut self, data: Vec<u8>) {
        self.sections.push(ElfSection {
            name: ".data".to_string(),
            section_type: 1, // SHT_PROGBITS
            flags: 0x3,      // SHF_ALLOC | SHF_WRITE
            data,
            alignment: 8,
        });
    }

    /// Add a .rodata section
    pub fn add_rodata(&mut self, data: Vec<u8>) {
        self.sections.push(ElfSection {
            name: ".rodata".to_string(),
            section_type: 1,
            flags: 0x2, // SHF_ALLOC
            data,
            alignment: 8,
        });
    }

    /// Add a global function symbol
    pub fn add_function_symbol(&mut self, name: &str, offset: u64, size: u64) {
        self.symbols.push(ElfSymbol {
            name: name.to_string(),
            value: offset,
            size,
            section_idx: 1, // .text
            bind: 1,        // STB_GLOBAL
            sym_type: 2,    // STT_FUNC
        });
    }

    /// Build the ELF binary
    pub fn build(&self) -> Vec<u8> {
        let mut elf = Vec::new();

        // ELF header (64-bit)
        // Magic number
        elf.extend_from_slice(&[0x7F, 0x45, 0x4C, 0x46]); // \x7FELF
        elf.push(2); // ELFCLASS64
        elf.push(1); // ELFDATA2LSB (little-endian)
        elf.push(1); // EV_CURRENT
        elf.push(0); // ELFOSABI_NONE
        elf.extend_from_slice(&[0; 8]); // padding

        // e_type: ET_REL (relocatable object)
        elf.extend_from_slice(&1u16.to_le_bytes());

        // e_machine
        let machine = match self.arch {
            Architecture::X86_64 => 0x3Eu16,  // EM_X86_64
            Architecture::Aarch64 => 0xB7u16, // EM_AARCH64
            Architecture::Riscv64 => 0xF3u16, // EM_RISCV
            _ => 0x3Eu16,
        };
        elf.extend_from_slice(&machine.to_le_bytes());

        // e_version
        elf.extend_from_slice(&1u32.to_le_bytes());

        // e_entry, e_phoff
        elf.extend_from_slice(&0u64.to_le_bytes()); // entry
        elf.extend_from_slice(&0u64.to_le_bytes()); // phoff

        // e_shoff (section header offset - at end of file)
        let shoff_pos = elf.len();
        elf.extend_from_slice(&0u64.to_le_bytes()); // placeholder

        // e_flags
        elf.extend_from_slice(&0u32.to_le_bytes());
        // e_ehsize
        elf.extend_from_slice(&64u16.to_le_bytes());
        // e_phentsize, e_phnum
        elf.extend_from_slice(&0u16.to_le_bytes());
        elf.extend_from_slice(&0u16.to_le_bytes());
        // e_shentsize (64 bytes per section header)
        elf.extend_from_slice(&64u16.to_le_bytes());
        // e_shnum
        let shnum = (self.sections.len() + 1) as u16; // +1 for null section
        elf.extend_from_slice(&shnum.to_le_bytes());
        // e_shstrndx
        elf.extend_from_slice(&0u16.to_le_bytes());

        // Write section data
        let mut section_offsets = Vec::new();
        for section in &self.sections {
            // Align
            while elf.len() % section.alignment as usize != 0 {
                elf.push(0);
            }
            section_offsets.push(elf.len());
            elf.extend_from_slice(&section.data);
        }

        // Write section headers
        let shoff = elf.len();
        // Null section header
        elf.extend_from_slice(&[0u8; 64]);

        // Section headers
        for (i, section) in self.sections.iter().enumerate() {
            // sh_name (index into shstrtab - simplified)
            elf.extend_from_slice(&0u32.to_le_bytes());
            // sh_type
            elf.extend_from_slice(&section.section_type.to_le_bytes());
            // sh_flags
            elf.extend_from_slice(&section.flags.to_le_bytes());
            // sh_addr
            elf.extend_from_slice(&0u64.to_le_bytes());
            // sh_offset
            elf.extend_from_slice(&(section_offsets[i] as u64).to_le_bytes());
            // sh_size
            elf.extend_from_slice(&(section.data.len() as u64).to_le_bytes());
            // sh_link
            elf.extend_from_slice(&0u32.to_le_bytes());
            // sh_info
            elf.extend_from_slice(&0u32.to_le_bytes());
            // sh_addralign
            elf.extend_from_slice(&section.alignment.to_le_bytes());
            // sh_entsize
            elf.extend_from_slice(&0u64.to_le_bytes());
        }

        // Patch shoff
        let shoff_bytes = (shoff as u64).to_le_bytes();
        elf[shoff_pos..shoff_pos + 8].copy_from_slice(&shoff_bytes);

        elf
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Native Code Generator (Top-Level)
// ─────────────────────────────────────────────────────────────────────────────

/// Top-level native code generator
pub struct NativeCodegen {
    target: TargetTriple,
    selector: InstructionSelector,
    allocator: LinearScanAllocator,
    /// Optimization level (0-3)
    opt_level: u32,
}

impl NativeCodegen {
    pub fn new(target: TargetTriple) -> Self {
        let selector = InstructionSelector::new(target.arch, target.os);
        let allocator = LinearScanAllocator::new(target.arch);

        NativeCodegen {
            target,
            selector,
            allocator,
            opt_level: 2,
        }
    }

    /// Set optimization level
    pub fn set_opt_level(&mut self, level: u32) {
        self.opt_level = level.min(3);
    }

    /// Compile an entire IR module to native code
    pub fn compile_module(&mut self, module: &IrModule) -> Result<NativeOutput, String> {
        info!(
            "NativeCodegen: Compiling module '{}' for {} ({:?})",
            module.name, self.target.arch, self.target.os
        );

        let mut all_code = Vec::new();
        let mut symbols = Vec::new();

        for func in &module.functions {
            let offset = all_code.len();

            // Instruction selection: IR → MIR
            let mir = self.selector.select_function(func);

            // Register allocation
            let mut alloc = LinearScanAllocator::new(self.target.arch);
            for (i, m) in mir.iter().enumerate() {
                match m {
                    MachineInst::MovRR { dst, .. } | MachineInst::MovRI { dst, .. } => {
                        alloc.add_vreg(&format!("r{}", dst), RegClass::Integer, i, i + 10);
                    }
                    _ => {}
                }
            }
            let _ = alloc.allocate();

            // Binary emission
            let code = match self.target.arch {
                Architecture::X86_64 => {
                    let mut emitter = X86Emitter::new();
                    // Prologue
                    emitter.code.push(0x55); // push rbp
                    emitter.code.extend_from_slice(&[0x48, 0x89, 0xE5]); // mov rbp, rsp

                    for inst in &mir {
                        emitter.emit(inst);
                    }

                    emitter.resolve_fixups();
                    emitter.code().to_vec()
                }
                Architecture::Aarch64 => {
                    let mut emitter = Arm64Emitter::new();
                    // Prologue: stp x29, x30, [sp, #-16]!; mov x29, sp
                    emitter.code.extend_from_slice(&0xA9BF7BFDu32.to_le_bytes());
                    emitter.code.extend_from_slice(&0x910003FDu32.to_le_bytes());

                    for inst in &mir {
                        emitter.emit(inst);
                    }

                    emitter.resolve_fixups();
                    emitter.code().to_vec()
                }
                Architecture::Wasm32 => {
                    let mut emitter = WasmEmitter::new();
                    let param_types: Vec<IrType> =
                        func.params.iter().map(|(_, ty)| ty.clone()).collect();
                    emitter.add_type(&param_types, &func.return_type);
                    emitter.add_function(func);
                    emitter.build()
                }
                _ => Vec::new(),
            };

            symbols.push((func.name.clone(), offset, code.len()));
            all_code.extend_from_slice(&code);
        }

        // Build output format
        let binary = match self.target.format {
            OutputFormat::Elf => {
                let mut elf = ElfBuilder::new(self.target.arch);
                elf.add_text(all_code);
                for (name, offset, size) in &symbols {
                    elf.add_function_symbol(name, *offset as u64, *size as u64);
                }
                elf.build()
            }
            OutputFormat::Wasm => all_code, // Already in Wasm format
            OutputFormat::RawBin => all_code,
            _ => all_code,
        };

        info!(
            "NativeCodegen: Output {} bytes ({} functions, {})",
            binary.len(),
            symbols.len(),
            self.target.format_name()
        );

        Ok(NativeOutput {
            binary,
            format: self.target.format,
            symbols: symbols
                .into_iter()
                .map(|(n, o, s)| NativeSymbol {
                    name: n,
                    offset: o,
                    size: s,
                })
                .collect(),
            target: self.target.clone(),
        })
    }
}

impl TargetTriple {
    fn format_name(&self) -> &'static str {
        match self.format {
            OutputFormat::Elf => "ELF",
            OutputFormat::Pe => "PE/COFF",
            OutputFormat::MachO => "Mach-O",
            OutputFormat::Wasm => "WASM",
            OutputFormat::RawBin => "raw binary",
        }
    }
}

/// Output from native code generation
#[derive(Debug)]
pub struct NativeOutput {
    /// The compiled binary data
    pub binary: Vec<u8>,
    /// Output format
    pub format: OutputFormat,
    /// Symbol table
    pub symbols: Vec<NativeSymbol>,
    /// Target triple
    pub target: TargetTriple,
}

/// A symbol in the output
#[derive(Debug)]
pub struct NativeSymbol {
    pub name: String,
    pub offset: usize,
    pub size: usize,
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_triple_host() {
        let triple = TargetTriple::host();
        // Should detect current platform
        #[cfg(target_os = "windows")]
        assert_eq!(triple.os, TargetOS::Windows);
        #[cfg(target_os = "linux")]
        assert_eq!(triple.os, TargetOS::Linux);
    }

    #[test]
    fn test_linear_scan_allocator() {
        let mut alloc = LinearScanAllocator::new(Architecture::X86_64);

        alloc.add_vreg("x", RegClass::Integer, 0, 10);
        alloc.add_vreg("y", RegClass::Integer, 2, 8);
        alloc.add_vreg("z", RegClass::Integer, 5, 15);

        let result = alloc.allocate().unwrap();
        assert_eq!(result.len(), 3);

        // All should get physical registers (we have 14 available)
        assert!(result[0].phys_reg.is_some());
        assert!(result[1].phys_reg.is_some());
        assert!(result[2].phys_reg.is_some());
    }

    #[test]
    fn test_instruction_selection_binop() {
        let mut selector = InstructionSelector::new(Architecture::X86_64, TargetOS::Linux);

        let func = IrFunction {
            name: "add_test".to_string(),
            params: vec![
                ("a".to_string(), IrType::I64),
                ("b".to_string(), IrType::I64),
            ],
            return_type: IrType::I64,
            blocks: vec![IrBlock {
                label: "entry".to_string(),
                instructions: vec![IrInstruction::BinOp {
                    dest: "c".to_string(),
                    op: IrBinOp::Add,
                    left: IrValue::Var("a".to_string()),
                    right: IrValue::Var("b".to_string()),
                }],
                terminator: IrTerminator::Return(Some(IrValue::Var("c".to_string()))),
            }],
            locals: vec![],
        };

        let mir = selector.select_function(&func);
        assert!(!mir.is_empty());

        // Should contain: Label, MovRR (setup), AluRR (add), MovRR (ret), Return
        let has_alu = mir
            .iter()
            .any(|i| matches!(i, MachineInst::AluRR { op: AluOp::Add, .. }));
        assert!(has_alu);
    }

    #[test]
    fn test_x86_emitter_basic() {
        let mut emitter = X86Emitter::new();

        emitter.emit(&MachineInst::Push { reg: 5 }); // push rbp
        emitter.emit(&MachineInst::MovRR { dst: 5, src: 4 }); // mov rbp, rsp
        emitter.emit(&MachineInst::Return); // ret

        let code = emitter.code();
        assert_eq!(code[0], 0x55); // push rbp
        assert_eq!(*code.last().unwrap(), 0xC3); // ret
    }

    #[test]
    fn test_arm64_emitter_nop() {
        let mut emitter = Arm64Emitter::new();
        emitter.emit(&MachineInst::Nop);

        let code = emitter.code();
        assert_eq!(code.len(), 4); // ARM64 instructions are 4 bytes
        assert_eq!(
            u32::from_le_bytes([code[0], code[1], code[2], code[3]]),
            0xD503201F
        );
    }

    #[test]
    fn test_wasm_emitter_basic() {
        let emitter = WasmEmitter::new();
        let binary = emitter.build();

        // Check Wasm magic number
        assert_eq!(&binary[0..4], &[0x00, 0x61, 0x73, 0x6D]);
        // Check version
        assert_eq!(&binary[4..8], &[0x01, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_elf_builder() {
        let mut elf = ElfBuilder::new(Architecture::X86_64);
        elf.add_text(vec![0x55, 0x48, 0x89, 0xE5, 0xC3]); // push rbp; mov rbp,rsp; ret
        elf.add_function_symbol("main", 0, 5);

        let binary = elf.build();
        // Check ELF magic
        assert_eq!(&binary[0..4], &[0x7F, 0x45, 0x4C, 0x46]);
        // Check 64-bit
        assert_eq!(binary[4], 2);
    }

    #[test]
    fn test_x86_reg_properties() {
        assert_eq!(X86Reg::arg_regs_sysv().len(), 6);
        assert_eq!(X86Reg::arg_regs_win64().len(), 4);
        assert!(X86Reg::R8.is_extended());
        assert!(!X86Reg::Rax.is_extended());
    }
}
