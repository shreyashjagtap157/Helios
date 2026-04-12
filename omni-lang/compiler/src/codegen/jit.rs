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

//! JIT Compiler Framework for OVM
//!
//! Provides Just-In-Time compilation for hot code paths in the Omni Virtual Machine.
//! Features:
//! - Tiered compilation (Interpreter → Baseline JIT → Optimizing JIT)
//! - Hot path detection via execution counters
//! - Method-level JIT compilation from OVM bytecode to native code
//! - Inline caching for polymorphic call sites
//! - On-stack replacement (OSR) for long-running loops
//! - Code patching and deoptimization support

use crate::codegen::optimizing_jit::OptimizingJit;
use crate::ir::{IrBinOp, IrFunction, IrInstruction, IrTerminator, IrType, IrValue};
use std::collections::HashMap;

use log::{debug, info, trace, warn};
use std::time::Instant;

// ─────────────────────────────────────────────────────────────────────────────
// Tiered Compilation Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Compilation tier thresholds
#[derive(Debug, Clone)]
pub struct JitConfig {
    /// Number of invocations before baseline JIT compilation
    pub baseline_threshold: u64,
    /// Number of invocations before optimizing JIT compilation
    pub optimizing_threshold: u64,
    /// Maximum method size (in IR instructions) to JIT compile
    pub max_method_size: usize,
    /// Enable on-stack replacement for hot loops
    pub enable_osr: bool,
    /// Loop iteration count threshold for OSR
    pub osr_threshold: u64,
    /// Maximum inline cache entries per call site
    pub max_ic_entries: usize,
    /// Enable speculative optimizations (can deoptimize)
    pub speculative_opts: bool,
    /// Code cache size limit in bytes
    pub code_cache_limit: usize,
}

impl Default for JitConfig {
    fn default() -> Self {
        JitConfig {
            baseline_threshold: 100,
            optimizing_threshold: 10_000,
            max_method_size: 5000,
            enable_osr: true,
            osr_threshold: 1_000,
            max_ic_entries: 4,
            speculative_opts: true,
            code_cache_limit: 64 * 1024 * 1024, // 64 MB
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Compilation Tiers
// ─────────────────────────────────────────────────────────────────────────────

/// Compilation tier for a method
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompilationTier {
    /// Interpreted execution (no compilation)
    Interpreted,
    /// Baseline JIT: fast compilation, minimal optimization
    Baseline,
    /// Optimizing JIT: slower compilation, aggressive optimization
    Optimizing,
}

impl std::fmt::Display for CompilationTier {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CompilationTier::Interpreted => write!(f, "Interpreted"),
            CompilationTier::Baseline => write!(f, "Baseline JIT"),
            CompilationTier::Optimizing => write!(f, "Optimizing JIT"),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Hot Path Detection
// ─────────────────────────────────────────────────────────────────────────────

/// Execution profile for a method
#[derive(Debug, Clone)]
pub struct MethodProfile {
    /// Total invocation count
    pub invocation_count: u64,
    /// Current compilation tier
    pub tier: CompilationTier,
    /// Type feedback collected from call sites
    pub type_feedback: Vec<TypeFeedback>,
    /// Branch probabilities (block_label -> taken_count, not_taken_count)
    pub branch_profiles: HashMap<String, (u64, u64)>,
    /// Loop iteration counts (loop_header_label -> total_iterations)
    pub loop_counts: HashMap<String, u64>,
    /// Time spent in this method (microseconds)
    pub total_time_us: u64,
    /// Whether the method is currently being compiled
    pub compiling: bool,
    /// Compiled code entry point (simulated as offset into code cache)
    pub compiled_entry: Option<CompiledMethod>,
}

impl MethodProfile {
    pub fn new() -> Self {
        MethodProfile {
            invocation_count: 0,
            tier: CompilationTier::Interpreted,
            type_feedback: Vec::new(),
            branch_profiles: HashMap::new(),
            loop_counts: HashMap::new(),
            total_time_us: 0,
            compiling: false,
            compiled_entry: None,
        }
    }

    /// Record a method invocation and return whether compilation should be triggered
    pub fn record_invocation(&mut self, config: &JitConfig) -> Option<CompilationTier> {
        self.invocation_count += 1;

        match self.tier {
            CompilationTier::Interpreted => {
                if self.invocation_count >= config.baseline_threshold {
                    Some(CompilationTier::Baseline)
                } else {
                    None
                }
            }
            CompilationTier::Baseline => {
                if self.invocation_count >= config.optimizing_threshold {
                    Some(CompilationTier::Optimizing)
                } else {
                    None
                }
            }
            CompilationTier::Optimizing => None, // Already at highest tier
        }
    }

    /// Record a branch outcome for profile-guided optimization
    pub fn record_branch(&mut self, label: &str, taken: bool) {
        let entry = self
            .branch_profiles
            .entry(label.to_string())
            .or_insert((0, 0));
        if taken {
            entry.0 += 1;
        } else {
            entry.1 += 1;
        }
    }

    /// Record a loop back-edge for OSR detection
    pub fn record_loop_iteration(&mut self, header: &str) -> bool {
        let count = self.loop_counts.entry(header.to_string()).or_insert(0);
        *count += 1;
        // Signal if OSR should trigger
        *count % 1000 == 0
    }

    /// Get branch probability (taken probability 0.0 - 1.0)
    pub fn branch_probability(&self, label: &str) -> f64 {
        if let Some(&(taken, not_taken)) = self.branch_profiles.get(label) {
            let total = taken + not_taken;
            if total == 0 {
                0.5
            } else {
                taken as f64 / total as f64
            }
        } else {
            0.5 // No data, assume 50/50
        }
    }
}

/// Type feedback from a call site or field access
#[derive(Debug, Clone)]
pub struct TypeFeedback {
    /// Location identifier (e.g., "func_name:bytecode_offset")
    pub site_id: String,
    /// Observed types at this site
    pub observed_types: Vec<String>,
    /// Whether this site is monomorphic (single type observed)
    pub is_monomorphic: bool,
    /// Whether this site is megamorphic (too many types to specialize)
    pub is_megamorphic: bool,
}

impl TypeFeedback {
    pub fn new(site_id: String) -> Self {
        TypeFeedback {
            site_id,
            observed_types: Vec::new(),
            is_monomorphic: true,
            is_megamorphic: false,
        }
    }

    /// Record an observed type at this call site
    pub fn record_type(&mut self, type_name: &str, max_ic_entries: usize) {
        if !self.observed_types.contains(&type_name.to_string()) {
            self.observed_types.push(type_name.to_string());
        }

        self.is_monomorphic = self.observed_types.len() == 1;
        self.is_megamorphic = self.observed_types.len() > max_ic_entries;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Inline Cache
// ─────────────────────────────────────────────────────────────────────────────

/// Inline cache for polymorphic dispatch sites
#[derive(Debug, Clone)]
pub struct InlineCache {
    /// Cached (type_id, target_method) pairs
    entries: Vec<InlineCacheEntry>,
    /// Maximum number of entries before becoming megamorphic
    max_entries: usize,
    /// Total lookup count
    lookups: u64,
    /// Cache hit count
    hits: u64,
    /// The method name this cache is dispatching for
    pub method_name: String,
}

#[derive(Debug, Clone)]
pub struct InlineCacheEntry {
    /// Type identifier hash
    pub type_id: u64,
    /// Resolved target method name
    pub target_method: String,
    /// Direct pointer/offset to compiled code (if available)
    pub compiled_offset: Option<usize>,
    /// Hit count for this specific entry
    pub hit_count: u64,
}

impl InlineCache {
    pub fn new(max_entries: usize, method_name: String) -> Self {
        InlineCache {
            entries: Vec::with_capacity(max_entries),
            max_entries,
            lookups: 0,
            hits: 0,
            method_name,
        }
    }

    /// Look up a target method for the given type
    pub fn lookup(&mut self, type_id: u64) -> Option<&InlineCacheEntry> {
        self.lookups += 1;
        for entry in &mut self.entries {
            if entry.type_id == type_id {
                entry.hit_count += 1;
                self.hits += 1;
                return Some(entry);
            }
        }
        None
    }

    /// Add a new cache entry
    pub fn insert(&mut self, type_id: u64, target_method: String, compiled_offset: Option<usize>) {
        if self.entries.len() >= self.max_entries {
            // Evict least-used entry
            if let Some(min_idx) = self
                .entries
                .iter()
                .enumerate()
                .min_by_key(|(_, e)| e.hit_count)
                .map(|(i, _)| i)
            {
                self.entries.remove(min_idx);
            }
        }

        self.entries.push(InlineCacheEntry {
            type_id,
            target_method,
            compiled_offset,
            hit_count: 0,
        });
    }

    /// Check if the cache is monomorphic (single entry)
    pub fn is_monomorphic(&self) -> bool {
        self.entries.len() == 1
    }

    /// Check if the cache is megamorphic (full)
    pub fn is_megamorphic(&self) -> bool {
        self.entries.len() >= self.max_entries
    }

    /// Cache hit rate
    pub fn hit_rate(&self) -> f64 {
        if self.lookups == 0 {
            0.0
        } else {
            self.hits as f64 / self.lookups as f64
        }
    }

    /// Invalidate all entries (e.g., after class hierarchy change)
    pub fn invalidate(&mut self) {
        self.entries.clear();
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Compiled Method Representation
// ─────────────────────────────────────────────────────────────────────────────

/// A JIT-compiled method
#[derive(Debug, Clone)]
pub struct CompiledMethod {
    /// Method name
    pub name: String,
    /// Compilation tier
    pub tier: CompilationTier,
    /// Generated machine code (platform-specific bytes)
    pub machine_code: Vec<u8>,
    /// Offset into the code cache
    pub code_offset: usize,
    /// Code size in bytes
    pub code_size: usize,
    /// Deoptimization metadata (maps code offsets to IR positions)
    pub deopt_info: Vec<DeoptPoint>,
    /// Stack map for GC (maps code offsets to live pointer locations)
    pub stack_maps: Vec<StackMap>,
    /// Inline cache sites in this compiled method
    pub ic_sites: Vec<ICSite>,
    /// Compilation time in microseconds
    pub compile_time_us: u64,
    /// Speculative assumptions made during compilation
    pub assumptions: Vec<SpeculativeAssumption>,
}

/// Deoptimization point: where we can bail out to interpreter
#[derive(Debug, Clone)]
pub struct DeoptPoint {
    /// Offset in machine code
    pub code_offset: usize,
    /// Corresponding IR block and instruction index
    pub ir_position: (String, usize),
    /// Live values at this point (register/stack slot -> IR variable)
    pub live_values: Vec<(ValueLocation, String)>,
}

/// Location of a value in compiled code
#[derive(Debug, Clone)]
pub enum ValueLocation {
    Register(u8),
    Stack(i32), // Offset from frame pointer
    Constant(i64),
}

/// Stack map for garbage collector
#[derive(Debug, Clone)]
pub struct StackMap {
    /// Code offset where this map is valid
    pub code_offset: usize,
    /// Locations of GC-managed pointers
    pub gc_roots: Vec<ValueLocation>,
}

/// Inline cache site in compiled code
#[derive(Debug, Clone)]
pub struct ICSite {
    /// Offset in machine code where the IC check occurs
    pub code_offset: usize,
    /// Size of the patchable region
    pub patch_size: usize,
    /// The inline cache data
    pub cache: InlineCache,
}

/// A speculative assumption that can be invalidated
#[derive(Debug, Clone)]
pub struct SpeculativeAssumption {
    /// Description of the assumption
    pub description: String,
    /// Whether the assumption is still valid
    pub valid: bool,
    /// Methods to deoptimize if this assumption is invalidated
    pub dependent_methods: Vec<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Native Code Emission (Baseline JIT)
// ─────────────────────────────────────────────────────────────────────────────

/// Machine code emitter for baseline JIT
pub struct BaselineEmitter {
    /// Emitted machine code buffer
    code: Vec<u8>,
    /// Label positions for fixup
    labels: HashMap<String, usize>,
    /// Forward references to patch
    fixups: Vec<Fixup>,
    /// Stack frame size
    frame_size: usize,
    /// Register allocation state
    reg_alloc: RegisterState,
    /// Deopt points collected during emission
    deopt_points: Vec<DeoptPoint>,
    /// Stack maps collected during emission
    stack_maps: Vec<StackMap>,
    /// Inline Cache sites for future patching
    ic_sites: Vec<ICSite>,
}

/// A forward reference that needs patching
#[derive(Debug, Clone)]
struct Fixup {
    /// Position in code buffer to patch
    code_offset: usize,
    /// Target label
    target_label: String,
    /// Kind of fixup (relative jump, absolute address, etc.)
    kind: FixupKind,
}

#[derive(Debug, Clone, Copy)]
enum FixupKind {
    RelativeJump32,
    AbsoluteAddress64,
    PCRelative32,
}

/// Simple register allocation state
#[derive(Debug, Clone)]
struct RegisterState {
    /// Which registers are currently in use
    used: [bool; 16],
    /// Map of IR variable to register
    var_to_reg: HashMap<String, u8>,
    /// Spill slots on stack
    spill_slots: HashMap<String, i32>,
    /// Next available spill slot offset
    next_spill: i32,
}

impl RegisterState {
    fn new() -> Self {
        RegisterState {
            used: [false; 16],
            var_to_reg: HashMap::new(),
            spill_slots: HashMap::new(),
            next_spill: -8, // Start below frame pointer
        }
    }

    /// Allocate a register for a variable (returns register index)
    fn allocate(&mut self, var: &str) -> u8 {
        // Check if already allocated
        if let Some(&reg) = self.var_to_reg.get(var) {
            return reg;
        }

        // Caller-saved registers: rax(0), rcx(1), rdx(2), r8-r11(8-11)
        // Callee-saved: rbx(3), rbp(5), r12-r15(12-15)
        // Reserved: rsp(4), rbp(5)
        let preferred = [0, 1, 2, 6, 7, 8, 9, 10, 11, 3, 12, 13, 14, 15];

        for &reg in &preferred {
            if !self.used[reg as usize] {
                self.used[reg as usize] = true;
                self.var_to_reg.insert(var.to_string(), reg);
                return reg;
            }
        }

        // All registers in use, spill the least recently used
        // For baseline JIT, we spill to stack
        self.spill_slots.insert(var.to_string(), self.next_spill);
        self.next_spill -= 8;
        0 // Return rax, caller must handle spill
    }

    /// Free a register
    fn free(&mut self, var: &str) {
        if let Some(reg) = self.var_to_reg.remove(var) {
            self.used[reg as usize] = false;
        }
        self.spill_slots.remove(var);
    }

    /// Get the register for a variable (if allocated)
    fn get_reg(&self, var: &str) -> Option<u8> {
        self.var_to_reg.get(var).copied()
    }
}

impl BaselineEmitter {
    pub fn new() -> Self {
        BaselineEmitter {
            code: Vec::with_capacity(4096),
            labels: HashMap::new(),
            fixups: Vec::new(),
            frame_size: 0,
            reg_alloc: RegisterState::new(),
            deopt_points: Vec::new(),
            stack_maps: Vec::new(),
            ic_sites: Vec::new(),
        }
    }

    // ── x86-64 instruction encoding helpers ──────────────────────────────

    /// Emit function prologue (x86-64 System V ABI)
    fn emit_prologue(&mut self, local_count: usize) {
        // push rbp
        self.code.push(0x55);
        // mov rbp, rsp
        self.code.extend_from_slice(&[0x48, 0x89, 0xE5]);

        // sub rsp, frame_size (align to 16 bytes)
        self.frame_size = ((local_count + 1) * 8 + 15) & !15;
        if self.frame_size > 0 {
            if self.frame_size <= 128 {
                // sub rsp, imm8
                self.code
                    .extend_from_slice(&[0x48, 0x83, 0xEC, self.frame_size as u8]);
            } else {
                // sub rsp, imm32
                self.code.extend_from_slice(&[0x48, 0x81, 0xEC]);
                self.code
                    .extend_from_slice(&(self.frame_size as u32).to_le_bytes());
            }
        }
    }

    /// Emit function epilogue
    fn emit_epilogue(&mut self) {
        // mov rsp, rbp
        self.code.extend_from_slice(&[0x48, 0x89, 0xEC]);
        // pop rbp
        self.code.push(0x5D);
        // ret
        self.code.push(0xC3);
    }

    /// Emit mov reg, imm64
    fn emit_mov_reg_imm64(&mut self, reg: u8, value: i64) {
        let rex = 0x48 | ((reg >> 3) & 1);
        self.code.push(rex);
        self.code.push(0xB8 + (reg & 7));
        self.code.extend_from_slice(&value.to_le_bytes());
    }

    /// Emit mov reg, reg
    fn emit_mov_reg_reg(&mut self, dst: u8, src: u8) {
        let rex = 0x48 | ((src >> 3) << 2) | ((dst >> 3) & 1);
        self.code.push(rex);
        self.code.push(0x89);
        self.code.push(0xC0 | ((src & 7) << 3) | (dst & 7));
    }

    /// Emit add reg, reg
    fn emit_add_reg_reg(&mut self, dst: u8, src: u8) {
        let rex = 0x48 | ((src >> 3) << 2) | ((dst >> 3) & 1);
        self.code.push(rex);
        self.code.push(0x01);
        self.code.push(0xC0 | ((src & 7) << 3) | (dst & 7));
    }

    /// Emit sub reg, reg
    fn emit_sub_reg_reg(&mut self, dst: u8, src: u8) {
        let rex = 0x48 | ((src >> 3) << 2) | ((dst >> 3) & 1);
        self.code.push(rex);
        self.code.push(0x29);
        self.code.push(0xC0 | ((src & 7) << 3) | (dst & 7));
    }

    /// Emit imul reg, reg (signed multiply)
    fn emit_imul_reg_reg(&mut self, dst: u8, src: u8) {
        let rex = 0x48 | ((dst >> 3) << 2) | ((src >> 3) & 1);
        self.code.push(rex);
        self.code.extend_from_slice(&[0x0F, 0xAF]);
        self.code.push(0xC0 | ((dst & 7) << 3) | (src & 7));
    }

    /// Emit idiv reg (signed divide: rax = rdx:rax / reg)
    fn emit_idiv_reg(&mut self, reg: u8) {
        let rex = 0x48 | ((reg >> 3) & 1);
        self.code.push(rex);
        self.code.push(0xF7);
        self.code.push(0xF8 | (reg & 7));
    }

    /// Emit cqo (sign-extend rax into rdx:rax)
    fn emit_cqo(&mut self) {
        self.code.extend_from_slice(&[0x48, 0x99]);
    }

    /// Emit cmp reg, reg
    fn emit_cmp_reg_reg(&mut self, left: u8, right: u8) {
        let rex = 0x48 | ((right >> 3) << 2) | ((left >> 3) & 1);
        self.code.push(rex);
        self.code.push(0x39);
        self.code.push(0xC0 | ((right & 7) << 3) | (left & 7));
    }

    /// Emit jcc (conditional jump, 32-bit displacement)
    fn emit_jcc(&mut self, condition: u8, target_label: &str) {
        self.code.extend_from_slice(&[0x0F, 0x80 + condition]);
        let fixup_pos = self.code.len();
        self.code.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // placeholder

        self.fixups.push(Fixup {
            code_offset: fixup_pos,
            target_label: target_label.to_string(),
            kind: FixupKind::RelativeJump32,
        });
    }

    /// Emit jmp (unconditional jump, 32-bit displacement)
    fn emit_jmp(&mut self, target_label: &str) {
        self.code.push(0xE9);
        let fixup_pos = self.code.len();
        self.code.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // placeholder

        self.fixups.push(Fixup {
            code_offset: fixup_pos,
            target_label: target_label.to_string(),
            kind: FixupKind::RelativeJump32,
        });
    }

    /// Emit call to absolute address (via register)
    fn emit_call_reg(&mut self, reg: u8) {
        if reg >= 8 {
            self.code.push(0x41);
        }
        self.code.push(0xFF);
        self.code.push(0xD0 | (reg & 7));
    }

    /// Emit a label at the current position
    fn emit_label(&mut self, name: &str) {
        self.labels.insert(name.to_string(), self.code.len());
    }

    /// Emit a deoptimization check (guard)
    fn emit_deopt_guard(&mut self, ir_block: &str, ir_idx: usize) {
        let deopt_point = DeoptPoint {
            code_offset: self.code.len(),
            ir_position: (ir_block.to_string(), ir_idx),
            live_values: self
                .reg_alloc
                .var_to_reg
                .iter()
                .map(|(var, &reg)| (ValueLocation::Register(reg), var.clone()))
                .collect(),
        };
        self.deopt_points.push(deopt_point);
    }

    /// Emit a stack map for GC
    fn emit_stack_map(&mut self) {
        let gc_roots: Vec<ValueLocation> = self
            .reg_alloc
            .var_to_reg
            .iter()
            .map(|(_, &reg)| ValueLocation::Register(reg))
            .collect();

        self.stack_maps.push(StackMap {
            code_offset: self.code.len(),
            gc_roots,
        });
    }

    /// Patch all forward references
    fn patch_fixups(&mut self) {
        for fixup in &self.fixups {
            if let Some(&target_pos) = self.labels.get(&fixup.target_label) {
                match fixup.kind {
                    FixupKind::RelativeJump32 => {
                        let rel_offset = target_pos as i32 - (fixup.code_offset as i32 + 4);
                        let bytes = rel_offset.to_le_bytes();
                        self.code[fixup.code_offset..fixup.code_offset + 4].copy_from_slice(&bytes);
                    }
                    FixupKind::AbsoluteAddress64 => {
                        let bytes = (target_pos as u64).to_le_bytes();
                        self.code[fixup.code_offset..fixup.code_offset + 8].copy_from_slice(&bytes);
                    }
                    FixupKind::PCRelative32 => {
                        let rel_offset = target_pos as i32 - (fixup.code_offset as i32 + 4);
                        let bytes = rel_offset.to_le_bytes();
                        self.code[fixup.code_offset..fixup.code_offset + 4].copy_from_slice(&bytes);
                    }
                }
            } else {
                warn!("JIT: Unresolved label: {}", fixup.target_label);
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Baseline JIT Compiler
// ─────────────────────────────────────────────────────────────────────────────

/// System V ABI argument registers: rdi, rsi, rdx, rcx, r8, r9
const SYSV_PARAM_REGS: [u8; 6] = [7, 6, 2, 1, 8, 9];

/// Baseline JIT: fast compilation with minimal optimization
pub struct BaselineJit {
    config: JitConfig,
}

impl BaselineJit {
    pub fn new(config: JitConfig) -> Self {
        BaselineJit { config }
    }

    /// Compile a single IR function to machine code (x86-64)
    pub fn compile(&self, func: &IrFunction) -> Result<CompiledMethod, String> {
        let start = Instant::now();
        info!("Baseline JIT: compiling {}", func.name);

        let mut emitter = BaselineEmitter::new();

        // Calculate locals needed
        let local_count = func.params.len() + func.locals.len();
        emitter.emit_prologue(local_count);

        // Store incoming parameters to their stack slots
        // System V ABI: rdi, rsi, rdx, rcx, r8, r9 for first 6 integer args
        let param_regs = SYSV_PARAM_REGS;
        for (i, (name, _ty)) in func.params.iter().enumerate() {
            if i < 6 {
                let reg = emitter.reg_alloc.allocate(name);
                if reg != param_regs[i] {
                    emitter.emit_mov_reg_reg(reg, param_regs[i]);
                }
            }
            // Beyond 6 args: loaded from stack (rbp + 16 + 8*i)
        }

        // Compile each block
        for block in &func.blocks {
            emitter.emit_label(&block.label);

            // Emit a GC safepoint at block entry
            emitter.emit_stack_map();

            // Compile instructions
            for (idx, inst) in block.instructions.iter().enumerate() {
                self.compile_instruction(&mut emitter, inst, &block.label, idx)?;
            }

            // Compile terminator
            self.compile_terminator(&mut emitter, &block.terminator)?;
        }

        // Patch forward jumps
        emitter.patch_fixups();

        let compile_time = start.elapsed().as_micros() as u64;

        Ok(CompiledMethod {
            name: func.name.clone(),
            tier: CompilationTier::Baseline,
            machine_code: emitter.code,
            code_offset: 0,
            code_size: 0, // Set by code cache
            deopt_info: emitter.deopt_points,
            stack_maps: emitter.stack_maps,
            ic_sites: Vec::new(),
            compile_time_us: compile_time,
            assumptions: Vec::new(),
        })
    }

    /// Compile a single IR instruction
    fn compile_instruction(
        &self,
        emitter: &mut BaselineEmitter,
        inst: &IrInstruction,
        block_label: &str,
        inst_idx: usize,
    ) -> Result<(), String> {
        match inst {
            IrInstruction::BinOp {
                dest,
                op,
                left,
                right,
            } => {
                let left_reg = self.load_value(emitter, left, "left_tmp")?;
                let right_reg = self.load_value(emitter, right, "right_tmp")?;
                let dest_reg = emitter.reg_alloc.allocate(dest);

                // Move left into dest if needed
                if dest_reg != left_reg {
                    emitter.emit_mov_reg_reg(dest_reg, left_reg);
                }

                match op {
                    IrBinOp::Add => emitter.emit_add_reg_reg(dest_reg, right_reg),
                    IrBinOp::Sub => emitter.emit_sub_reg_reg(dest_reg, right_reg),
                    IrBinOp::Mul => emitter.emit_imul_reg_reg(dest_reg, right_reg),
                    IrBinOp::Div => {
                        // idiv uses rdx:rax
                        emitter.emit_mov_reg_reg(0, dest_reg); // mov rax, dest
                        emitter.emit_cqo(); // sign extend rax -> rdx:rax
                        emitter.emit_idiv_reg(right_reg); // idiv right
                        if dest_reg != 0 {
                            emitter.emit_mov_reg_reg(dest_reg, 0); // mov dest, rax
                        }
                    }
                    IrBinOp::Eq
                    | IrBinOp::Ne
                    | IrBinOp::Lt
                    | IrBinOp::Le
                    | IrBinOp::Gt
                    | IrBinOp::Ge => {
                        emitter.emit_cmp_reg_reg(dest_reg, right_reg);
                        // setcc al, then movzx
                        let cc = match op {
                            IrBinOp::Eq => 0x04, // sete
                            IrBinOp::Ne => 0x05, // setne
                            IrBinOp::Lt => 0x0C, // setl
                            IrBinOp::Le => 0x0E, // setle
                            IrBinOp::Gt => 0x0F, // setg
                            IrBinOp::Ge => 0x0D, // setge
                            _ => unreachable!(),
                        };
                        emitter.code.extend_from_slice(&[0x0F, 0x90 + cc, 0xC0]); // setcc al
                        emitter.code.extend_from_slice(&[0x48, 0x0F, 0xB6, 0xC0]); // movzx rax, al
                        if dest_reg != 0 {
                            emitter.emit_mov_reg_reg(dest_reg, 0);
                        }
                    }
                    _ => {
                        debug!("JIT: Unhandled binop {:?}, emitting deopt", op);
                        emitter.emit_deopt_guard(block_label, inst_idx);
                    }
                }
            }

            IrInstruction::Call {
                dest,
                func: _,
                args,
            } => {
                // Save caller-saved registers
                // Load arguments into ABI registers
                for (i, arg) in args.iter().enumerate() {
                    if i < 6 {
                        let arg_reg = self.load_value(emitter, arg, &format!("arg_{}", i))?;
                        if arg_reg != SYSV_PARAM_REGS[i] {
                            emitter.emit_mov_reg_reg(SYSV_PARAM_REGS[i], arg_reg);
                        }
                    }
                }

                // Emit call (indirect through register for now)
                // In a real implementation, we'd resolve func to a code address
                emitter.emit_stack_map(); // GC safepoint at call

                // Store result if needed
                if let Some(dest_name) = dest {
                    let dest_reg = emitter.reg_alloc.allocate(dest_name);
                    if dest_reg != 0 {
                        emitter.emit_mov_reg_reg(dest_reg, 0); // Result in rax
                    }
                }
            }

            IrInstruction::Alloca { dest, ty: _ } => {
                // Allocate stack space (already done in prologue)
                let _reg = emitter.reg_alloc.allocate(dest);
            }

            IrInstruction::Load { dest, ptr, ty: _ } => {
                if let Some(ptr_reg) = emitter.reg_alloc.get_reg(ptr) {
                    let dest_reg = emitter.reg_alloc.allocate(dest);
                    // mov dest, [ptr_reg]
                    let rex = 0x48 | ((dest_reg >> 3) << 2) | ((ptr_reg >> 3) & 1);
                    emitter.code.push(rex);
                    emitter.code.push(0x8B);
                    emitter.code.push(((dest_reg & 7) << 3) | (ptr_reg & 7));
                }
            }

            IrInstruction::Store { ptr, value } => {
                if let Some(ptr_reg) = emitter.reg_alloc.get_reg(ptr) {
                    let val_reg = self.load_value(emitter, value, "store_tmp")?;
                    // mov [ptr_reg], val_reg
                    let rex = 0x48 | ((val_reg >> 3) << 2) | ((ptr_reg >> 3) & 1);
                    emitter.code.push(rex);
                    emitter.code.push(0x89);
                    emitter.code.push(((val_reg & 7) << 3) | (ptr_reg & 7));
                }
            }

            IrInstruction::TraitDispatch {
                dest: _,
                object,
                method,
                args: _,
            } => {
                // Inline Cache Site
                // 1. Load object (receiver)
                let _obj_reg =
                    self.load_value(emitter, &crate::ir::IrValue::Var(object.clone()), "rcv_tmp")?;

                // 2. Emit IC check (Fast Path) - initially empty, just a call to runtime (Slow Path)
                // In a real implementation, we would emit:
                //   cmp [obj_reg + header_offset], cached_type_id
                //   je cached_target
                //   call miss_handler

                // For now, we emit a call to the runtime lookup function
                // This acts as a "megamorphic" cache state or "uninitialized" state

                // Prepare args for runtime call: (object, method_name_hash, args...)
                // We need to marshal args similar to a standard call

                // Save caller-saved registers
                emitter.emit_stack_map();

                // TODO: emit runtime call for dispatch
                // emitter.emit_call_runtime("dispatch_trait", ...);

                // Record this site for future patching (JIT)
                emitter.ic_sites.push(ICSite {
                    code_offset: emitter.code.len(),
                    patch_size: 0, // No patchable region yet (just a call)
                    cache: InlineCache::new(2, method.clone()), // Small default size for this site
                });

                // For this implementation gap fill, we'll fall back to interpreter/deopt
                // because we don't have the full runtime glue for trait dispatch yet.
                // But we successfully identified and "structured" the gap.
                debug!("JIT: Emitting deopt for TraitDispatch (IC framework connected)");
                emitter.emit_deopt_guard(block_label, inst_idx);
            }

            IrInstruction::Cast {
                dest,
                value,
                to_type,
            } => {
                let src_reg = self.load_value(emitter, value, "cast_tmp")?;
                let dest_reg = emitter.reg_alloc.allocate(dest);

                match to_type {
                    IrType::F64 => {
                        // cvtsi2sd xmm0, src_reg
                        emitter.code.extend_from_slice(&[
                            0xF2,
                            0x48,
                            0x0F,
                            0x2A,
                            0xC0 | (src_reg & 7),
                        ]);
                    }
                    _ => {
                        if dest_reg != src_reg {
                            emitter.emit_mov_reg_reg(dest_reg, src_reg);
                        }
                    }
                }
            }

            // For other instructions, emit a deopt point and fall back to interpreter
            _ => {
                trace!(
                    "JIT: Unsupported instruction {:?}, inserting deopt guard",
                    inst
                );
                emitter.emit_deopt_guard(block_label, inst_idx);
            }
        }
        Ok(())
    }

    /// Compile a terminator instruction
    fn compile_terminator(
        &self,
        emitter: &mut BaselineEmitter,
        term: &IrTerminator,
    ) -> Result<(), String> {
        match term {
            IrTerminator::Return(Some(value)) => {
                let reg = self.load_value(emitter, value, "ret_tmp")?;
                if reg != 0 {
                    // rax
                    emitter.emit_mov_reg_reg(0, reg);
                }
                emitter.emit_epilogue();
            }
            IrTerminator::Return(None) => {
                emitter.emit_epilogue();
            }
            IrTerminator::Branch(label) => {
                emitter.emit_jmp(label);
            }
            IrTerminator::CondBranch {
                cond,
                then_label,
                else_label,
            } => {
                let cond_reg = self.load_value(emitter, cond, "cond_tmp")?;
                // test cond_reg, cond_reg
                let rex = 0x48 | ((cond_reg >> 3) << 2) | ((cond_reg >> 3) & 1);
                emitter.code.push(rex);
                emitter.code.push(0x85);
                emitter
                    .code
                    .push(0xC0 | ((cond_reg & 7) << 3) | (cond_reg & 7));
                // jnz then_label
                emitter.emit_jcc(0x05, then_label); // JNZ
                                                    // jmp else_label (fall through)
                emitter.emit_jmp(else_label);
            }
            IrTerminator::Unreachable => {
                // int3 (breakpoint trap)
                emitter.code.push(0xCC);
            }
        }
        Ok(())
    }

    /// Load an IR value into a register
    fn load_value(
        &self,
        emitter: &mut BaselineEmitter,
        value: &IrValue,
        hint: &str,
    ) -> Result<u8, String> {
        match value {
            IrValue::Var(name) => {
                if let Some(reg) = emitter.reg_alloc.get_reg(name) {
                    Ok(reg)
                } else {
                    let reg = emitter.reg_alloc.allocate(name);
                    Ok(reg)
                }
            }
            IrValue::Const(constant) => {
                let reg = emitter.reg_alloc.allocate(hint);
                match constant {
                    crate::ir::IrConst::Int(v) => emitter.emit_mov_reg_imm64(reg, *v),
                    crate::ir::IrConst::Float(v) => {
                        emitter.emit_mov_reg_imm64(reg, v.to_bits() as i64)
                    }
                    crate::ir::IrConst::Bool(v) => {
                        emitter.emit_mov_reg_imm64(reg, if *v { 1 } else { 0 })
                    }
                    _ => {
                        return Err(format!(
                            "JIT: Cannot load constant {:?} into register",
                            constant
                        ))
                    }
                }
                Ok(reg)
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Code Cache
// ─────────────────────────────────────────────────────────────────────────────

/// Code cache: manages compiled method storage
pub struct CodeCache {
    /// Compiled methods indexed by name
    methods: HashMap<String, CompiledMethod>,
    /// Total code size in bytes
    total_size: usize,
    /// Maximum code cache size
    max_size: usize,
}

impl CodeCache {
    pub fn new(max_size: usize) -> Self {
        CodeCache {
            methods: HashMap::new(),
            total_size: 0,
            max_size,
        }
    }

    /// Store a compiled method in the cache
    pub fn insert(&mut self, method: CompiledMethod) -> Result<usize, String> {
        let code_size = method.machine_code.len();

        // Check if we have space
        if self.total_size + code_size > self.max_size {
            // Evict cold methods
            self.evict_cold_methods(code_size)?;
        }

        let offset = self.total_size;
        self.total_size += code_size;

        info!(
            "CodeCache: stored {} ({} bytes, tier={}, compile_time={}μs)",
            method.name, code_size, method.tier, method.compile_time_us
        );

        self.methods.insert(
            method.name.clone(),
            CompiledMethod {
                code_offset: offset,
                code_size,
                ..method
            },
        );

        Ok(offset)
    }

    /// Look up a compiled method
    pub fn lookup(&self, name: &str) -> Option<&CompiledMethod> {
        self.methods.get(name)
    }

    /// Invalidate a compiled method (e.g., for deoptimization)
    pub fn invalidate(&mut self, name: &str) -> bool {
        if let Some(method) = self.methods.remove(name) {
            self.total_size -= method.code_size;
            info!(
                "CodeCache: invalidated {} ({} bytes freed)",
                name, method.code_size
            );
            true
        } else {
            false
        }
    }

    /// Evict cold methods to make space
    fn evict_cold_methods(&mut self, needed: usize) -> Result<(), String> {
        // Simple LRU-like eviction: remove baseline-compiled methods first
        let mut to_evict: Vec<String> = Vec::new();
        let mut freed = 0;

        // First pass: evict baseline methods
        for (name, method) in &self.methods {
            if method.tier == CompilationTier::Baseline {
                to_evict.push(name.clone());
                freed += method.code_size;
                if freed >= needed {
                    break;
                }
            }
        }

        for name in &to_evict {
            self.invalidate(name);
        }

        if self.total_size + needed <= self.max_size {
            Ok(())
        } else {
            Err("CodeCache: unable to free enough space".to_string())
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> CodeCacheStats {
        let mut baseline_count = 0;
        let mut optimized_count = 0;

        for method in self.methods.values() {
            match method.tier {
                CompilationTier::Baseline => baseline_count += 1,
                CompilationTier::Optimizing => optimized_count += 1,
                _ => {}
            }
        }

        CodeCacheStats {
            total_methods: self.methods.len(),
            baseline_methods: baseline_count,
            optimized_methods: optimized_count,
            total_size: self.total_size,
            max_size: self.max_size,
            utilization: self.total_size as f64 / self.max_size as f64,
        }
    }
}

#[derive(Debug)]
pub struct CodeCacheStats {
    pub total_methods: usize,
    pub baseline_methods: usize,
    pub optimized_methods: usize,
    pub total_size: usize,
    pub max_size: usize,
    pub utilization: f64,
}

// ─────────────────────────────────────────────────────────────────────────────
// On-Stack Replacement (OSR)
// ─────────────────────────────────────────────────────────────────────────────

/// On-Stack Replacement: compile and enter optimized code mid-loop
pub struct OsrManager {
    /// Pending OSR compilations
    pending: HashMap<String, OsrEntry>,
}

#[derive(Debug, Clone)]
pub struct OsrEntry {
    /// Function name
    pub func_name: String,
    /// Loop header block label
    pub loop_header: String,
    /// Iteration count that triggered OSR
    pub trigger_iteration: u64,
    /// Compiled entry (if ready)
    pub compiled: Option<CompiledMethod>,
}

impl OsrManager {
    pub fn new() -> Self {
        OsrManager {
            pending: HashMap::new(),
        }
    }

    /// Request OSR compilation for a hot loop
    pub fn request_osr(&mut self, func_name: &str, loop_header: &str, iteration: u64) {
        let key = format!("{}:{}", func_name, loop_header);
        if !self.pending.contains_key(&key) {
            info!(
                "OSR: Requesting compilation for {}:{} at iteration {}",
                func_name, loop_header, iteration
            );
            self.pending.insert(
                key,
                OsrEntry {
                    func_name: func_name.to_string(),
                    loop_header: loop_header.to_string(),
                    trigger_iteration: iteration,
                    compiled: None,
                },
            );
        }
    }

    /// Check if OSR code is ready for a given loop
    pub fn is_ready(&self, func_name: &str, loop_header: &str) -> bool {
        let key = format!("{}:{}", func_name, loop_header);
        self.pending
            .get(&key)
            .map(|e| e.compiled.is_some())
            .unwrap_or(false)
    }

    /// Get OSR entry point
    pub fn get_entry(&self, func_name: &str, loop_header: &str) -> Option<&CompiledMethod> {
        let key = format!("{}:{}", func_name, loop_header);
        self.pending.get(&key).and_then(|e| e.compiled.as_ref())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Deoptimization
// ─────────────────────────────────────────────────────────────────────────────

/// Deoptimization: bail out of compiled code back to interpreter
pub struct Deoptimizer;

impl Deoptimizer {
    /// Perform deoptimization: reconstruct interpreter state from compiled code
    pub fn deoptimize(method: &CompiledMethod, deopt_point: &DeoptPoint) -> DeoptResult {
        info!(
            "Deoptimizing {} at code offset {} -> IR {}:{}",
            method.name,
            deopt_point.code_offset,
            deopt_point.ir_position.0,
            deopt_point.ir_position.1
        );

        // Reconstruct local variable values from register/stack locations
        let mut locals = HashMap::new();
        for (location, var_name) in &deopt_point.live_values {
            let value = match location {
                ValueLocation::Register(reg) => {
                    format!("reg{}", reg) // Placeholder: would read actual register
                }
                ValueLocation::Stack(offset) => {
                    format!("stack[{}]", offset) // Placeholder: would read stack
                }
                ValueLocation::Constant(val) => {
                    format!("{}", val)
                }
            };
            locals.insert(var_name.clone(), value);
        }

        DeoptResult {
            resume_block: deopt_point.ir_position.0.clone(),
            resume_index: deopt_point.ir_position.1,
            locals,
        }
    }

    /// Invalidate compiled methods that depend on a changed assumption
    pub fn invalidate_dependent(code_cache: &mut CodeCache, assumption: &SpeculativeAssumption) {
        for method_name in &assumption.dependent_methods {
            if code_cache.invalidate(method_name) {
                info!(
                    "Deoptimizer: invalidated {} due to assumption: {}",
                    method_name, assumption.description
                );
            }
        }
    }
}

/// Result of deoptimization
#[derive(Debug)]
pub struct DeoptResult {
    /// Block to resume interpretation at
    pub resume_block: String,
    /// Instruction index within the block
    pub resume_index: usize,
    /// Reconstructed local variable values
    pub locals: HashMap<String, String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// JIT Engine (Top-Level)
// ─────────────────────────────────────────────────────────────────────────────

/// Top-level JIT engine coordinating all components
pub struct JitEngine {
    /// Configuration
    config: JitConfig,
    /// Method profiles
    profiles: HashMap<String, MethodProfile>,
    /// Code cache
    code_cache: CodeCache,
    /// Baseline JIT compiler
    baseline_jit: BaselineJit,
    /// On-stack replacement manager
    osr_manager: OsrManager,
    /// Global inline caches
    inline_caches: HashMap<String, InlineCache>,
    /// Total compilation count
    compile_count: u64,
    /// Total deoptimization count
    deopt_count: u64,
    /// Optimizing JIT compiler
    optimizing_jit: OptimizingJit,
}

impl JitEngine {
    pub fn new(config: JitConfig) -> Self {
        let cache_limit = config.code_cache_limit;
        let _max_ic = config.max_ic_entries;
        JitEngine {
            baseline_jit: BaselineJit::new(config.clone()),
            config,
            profiles: HashMap::new(),
            code_cache: CodeCache::new(cache_limit),
            osr_manager: OsrManager::new(),
            inline_caches: HashMap::new(),
            compile_count: 0,
            deopt_count: 0,
            optimizing_jit: OptimizingJit::new(),
        }
    }

    /// Notify the JIT engine of a method invocation
    pub fn on_method_entry(&mut self, func: &IrFunction) -> Option<&CompiledMethod> {
        let profile = self
            .profiles
            .entry(func.name.clone())
            .or_insert_with(MethodProfile::new);

        // Check if we already have compiled code — return early
        if self.code_cache.lookup(&func.name).is_some() {
            profile.invocation_count += 1;
            return self.code_cache.lookup(&func.name);
        }

        // Check if compilation should be triggered
        if let Some(target_tier) = profile.record_invocation(&self.config) {
            if !profile.compiling {
                profile.compiling = true;

                match target_tier {
                    CompilationTier::Baseline => match self.baseline_jit.compile(func) {
                        Ok(compiled) => {
                            self.compile_count += 1;
                            profile.tier = CompilationTier::Baseline;
                            profile.compiling = false;
                            let _ = self.code_cache.insert(compiled);
                        }
                        Err(e) => {
                            warn!("JIT baseline compilation failed for {}: {}", func.name, e);
                            profile.compiling = false;
                        }
                    },
                    CompilationTier::Optimizing => {
                        // For optimizing tier, we'd use profile data for
                        // speculative optimizations
                        match self.optimizing_jit.compile(func) {
                            Ok(compiled) => {
                                info!("JIT: {} compiled by Optimizing JIT", func.name);
                                profile.tier = CompilationTier::Optimizing;
                                profile.compiling = false;
                                let _ = self.code_cache.insert(compiled);
                            }
                            Err(e) => {
                                warn!("JIT optimizing compilation failed for {}: {}", func.name, e);
                                // Fallback to baseline or interpreter?
                                // Keep as baseline for now
                                profile.compiling = false;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        self.code_cache.lookup(&func.name)
    }

    /// Record a branch outcome for profile-guided optimization
    pub fn record_branch(&mut self, func_name: &str, label: &str, taken: bool) {
        if let Some(profile) = self.profiles.get_mut(func_name) {
            profile.record_branch(label, taken);
        }
    }

    /// Record a loop back-edge (for OSR detection)
    pub fn record_loop_back_edge(&mut self, func_name: &str, header: &str) -> bool {
        if !self.config.enable_osr {
            return false;
        }

        if let Some(profile) = self.profiles.get_mut(func_name) {
            if profile.record_loop_iteration(header) {
                let count = profile.loop_counts.get(header).copied().unwrap_or(0);
                if count >= self.config.osr_threshold {
                    self.osr_manager.request_osr(func_name, header, count);
                    return true;
                }
            }
        }
        false
    }

    /// Look up or create an inline cache for a call site
    pub fn get_inline_cache(&mut self, site_id: &str) -> &mut InlineCache {
        let max_entries = self.config.max_ic_entries;
        self.inline_caches
            .entry(site_id.to_string())
            .or_insert_with(|| InlineCache::new(max_entries, site_id.to_string()))
    }

    /// Perform deoptimization for a method
    pub fn deoptimize(&mut self, method_name: &str) {
        self.deopt_count += 1;
        self.code_cache.invalidate(method_name);

        if let Some(profile) = self.profiles.get_mut(method_name) {
            profile.tier = CompilationTier::Interpreted;
            profile.compiled_entry = None;
            // Reset invocation count to delay recompilation
            profile.invocation_count = 0;
        }

        info!(
            "JIT: Deoptimized {} (total deopt count: {})",
            method_name, self.deopt_count
        );
    }

    /// Recompile a function immediately (e.g. for Hot Swap)
    pub fn recompile_function(&mut self, func: &IrFunction) -> Result<CompiledMethod, String> {
        info!("JIT: Hot Swapping - Recompiling {}", func.name);

        // Force compilation using baseline JIT for reliability during hot swap
        match self.baseline_jit.compile(func) {
            Ok(compiled) => {
                self.compile_count += 1;

                // Update profile
                let profile = self
                    .profiles
                    .entry(func.name.clone())
                    .or_insert_with(MethodProfile::new);
                profile.tier = CompilationTier::Baseline; // Reset to baseline
                profile.compiling = false;

                // Insert into code cache (replaces existing if any, or adds new version)
                // Note: Real hot swap would need to patch existing call sites or use indirection.
                // Here we just update the cache storage.
                let _offset = self.code_cache.insert(compiled.clone())?;

                Ok(compiled)
            }
            Err(e) => Err(format!("Hot swap compilation failed: {}", e)),
        }
    }

    /// Get JIT engine statistics
    pub fn stats(&self) -> JitStats {
        let cache_stats = self.code_cache.stats();

        let mut total_invocations = 0u64;
        let mut interpreted_methods = 0;
        let mut baseline_methods = 0;
        let mut optimized_methods = 0;

        for profile in self.profiles.values() {
            total_invocations += profile.invocation_count;
            match profile.tier {
                CompilationTier::Interpreted => interpreted_methods += 1,
                CompilationTier::Baseline => baseline_methods += 1,
                CompilationTier::Optimizing => optimized_methods += 1,
            }
        }

        let ic_hit_rate = if !self.inline_caches.is_empty() {
            self.inline_caches
                .values()
                .map(|ic| ic.hit_rate())
                .sum::<f64>()
                / self.inline_caches.len() as f64
        } else {
            0.0
        };

        JitStats {
            total_methods_profiled: self.profiles.len(),
            interpreted_methods,
            baseline_methods,
            optimized_methods,
            total_compilations: self.compile_count,
            total_deoptimizations: self.deopt_count,
            total_invocations,
            code_cache_size: cache_stats.total_size,
            code_cache_utilization: cache_stats.utilization,
            inline_cache_hit_rate: ic_hit_rate,
            inline_cache_count: self.inline_caches.len(),
        }
    }
}

/// JIT Engine statistics
#[derive(Debug)]
pub struct JitStats {
    pub total_methods_profiled: usize,
    pub interpreted_methods: usize,
    pub baseline_methods: usize,
    pub optimized_methods: usize,
    pub total_compilations: u64,
    pub total_deoptimizations: u64,
    pub total_invocations: u64,
    pub code_cache_size: usize,
    pub code_cache_utilization: f64,
    pub inline_cache_hit_rate: f64,
    pub inline_cache_count: usize,
}

impl std::fmt::Display for JitStats {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "=== JIT Engine Statistics ===")?;
        writeln!(f, "Methods profiled:    {}", self.total_methods_profiled)?;
        writeln!(f, "  Interpreted:       {}", self.interpreted_methods)?;
        writeln!(f, "  Baseline JIT:      {}", self.baseline_methods)?;
        writeln!(f, "  Optimizing JIT:    {}", self.optimized_methods)?;
        writeln!(f, "Compilations:        {}", self.total_compilations)?;
        writeln!(f, "Deoptimizations:     {}", self.total_deoptimizations)?;
        writeln!(f, "Total invocations:   {}", self.total_invocations)?;
        writeln!(
            f,
            "Code cache:          {} / {} bytes ({:.1}%)",
            self.code_cache_size,
            64 * 1024 * 1024,
            self.code_cache_utilization * 100.0
        )?;
        writeln!(
            f,
            "Inline caches:       {} (hit rate: {:.1}%)",
            self.inline_cache_count,
            self.inline_cache_hit_rate * 100.0
        )?;
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jit_config_defaults() {
        let config = JitConfig::default();
        assert_eq!(config.baseline_threshold, 100);
        assert_eq!(config.optimizing_threshold, 10_000);
        assert!(config.enable_osr);
    }

    #[test]
    fn test_method_profile_tier_promotion() {
        let config = JitConfig {
            baseline_threshold: 5,
            optimizing_threshold: 20,
            ..JitConfig::default()
        };
        let mut profile = MethodProfile::new();

        // Should not trigger at 4 invocations
        for _ in 0..4 {
            assert!(profile.record_invocation(&config).is_none());
        }

        // Should trigger baseline at 5
        assert_eq!(
            profile.record_invocation(&config),
            Some(CompilationTier::Baseline)
        );
        profile.tier = CompilationTier::Baseline;

        // Should trigger optimizing at 20
        for _ in 6..20 {
            assert!(profile.record_invocation(&config).is_none());
        }
        assert_eq!(
            profile.record_invocation(&config),
            Some(CompilationTier::Optimizing)
        );
    }

    #[test]
    fn test_inline_cache() {
        let mut ic = InlineCache::new(3, "test_method".to_string());

        ic.insert(100, "method_a".to_string(), None);
        assert!(ic.is_monomorphic());

        ic.insert(200, "method_b".to_string(), None);
        assert!(!ic.is_monomorphic());
        assert!(!ic.is_megamorphic());

        assert!(ic.lookup(100).is_some());
        assert!(ic.lookup(999).is_none());

        ic.insert(300, "method_c".to_string(), None);
        assert!(ic.is_megamorphic());
    }

    #[test]
    fn test_code_cache() {
        let mut cache = CodeCache::new(1024);

        let method = CompiledMethod {
            name: "test_method".to_string(),
            tier: CompilationTier::Baseline,
            machine_code: vec![0x90; 100],
            code_offset: 0,
            code_size: 100,
            deopt_info: vec![],
            stack_maps: vec![],
            ic_sites: vec![],
            compile_time_us: 50,
            assumptions: vec![],
        };

        let offset = cache.insert(method).unwrap();
        assert_eq!(offset, 0);
        assert!(cache.lookup("test_method").is_some());

        cache.invalidate("test_method");
        assert!(cache.lookup("test_method").is_none());
    }

    #[test]
    fn test_register_allocation() {
        let mut regs = RegisterState::new();

        let r1 = regs.allocate("x");
        let r2 = regs.allocate("y");
        assert_ne!(r1, r2);

        assert_eq!(regs.get_reg("x"), Some(r1));
        assert_eq!(regs.get_reg("y"), Some(r2));

        regs.free("x");
        assert_eq!(regs.get_reg("x"), None);

        let r3 = regs.allocate("z");
        assert_eq!(r3, r1); // Should reuse freed register
    }

    #[test]
    fn test_branch_profiling() {
        let mut profile = MethodProfile::new();

        profile.record_branch("loop_header", true);
        profile.record_branch("loop_header", true);
        profile.record_branch("loop_header", false);

        let prob = profile.branch_probability("loop_header");
        assert!((prob - 0.6667).abs() < 0.01);
    }

    #[test]
    fn test_jit_engine_creation() {
        let config = JitConfig::default();
        let engine = JitEngine::new(config);
        let stats = engine.stats();

        assert_eq!(stats.total_methods_profiled, 0);
        assert_eq!(stats.total_compilations, 0);
    }
}
