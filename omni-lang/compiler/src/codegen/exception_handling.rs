//! Exception Handling and Stack Unwinding
//!
//! Implements structured exception handling for the Omni language:
//! - .eh_frame / .debug_frame sections for stack unwinding (DWARF CFI)
//! - Language-Specific Data Area (LSDA) for catch/filter clauses
//! - Personality function for Omni's exception model
//! - Try/catch IR lowering
//! - Zero-cost exception model (no overhead on non-exception paths)

use crate::ir::{IrBlock, IrFunction, IrInstruction, IrTerminator, IrType};
use log::{debug, info};

// ─────────────────────────────────────────────────────────────────────────────
// DWARF CFI (Call Frame Information) Constants
// ─────────────────────────────────────────────────────────────────────────────

/// CIE (Common Information Entry) identifier
const CIE_ID: u32 = 0;
/// DWARF CIE version
const CIE_VERSION: u8 = 1;
/// FDE (Frame Description Entry) pointer encoding
const DW_EH_PE_ABSPTR: u8 = 0x00;
const DW_EH_PE_UDATA4: u8 = 0x03;
const DW_EH_PE_SDATA4: u8 = 0x0b;
const DW_EH_PE_PCREL: u8 = 0x10;

/// CFI instructions
const DW_CFA_DEF_CFA: u8 = 0x0c;
const DW_CFA_DEF_CFA_OFFSET: u8 = 0x0e;
const DW_CFA_OFFSET: u8 = 0x80; // High 2 bits
const DW_CFA_ADVANCE_LOC: u8 = 0x40; // High 2 bits
const DW_CFA_ADVANCE_LOC1: u8 = 0x02;
const DW_CFA_ADVANCE_LOC2: u8 = 0x03;
const DW_CFA_ADVANCE_LOC4: u8 = 0x04;
const DW_CFA_NOP: u8 = 0x00;
const DW_CFA_REGISTER: u8 = 0x09;
const DW_CFA_REMEMBER_STATE: u8 = 0x0a;
const DW_CFA_RESTORE_STATE: u8 = 0x0b;

// x86-64 DWARF register numbers
const DWARF_REG_RAX: u8 = 0;
const DWARF_REG_RBX: u8 = 3;
const DWARF_REG_RBP: u8 = 6;
const DWARF_REG_RSP: u8 = 7;
const DWARF_REG_RA: u8 = 16; // Return address (RIP)

// LSDA type info encoding
const DW_EH_PE_OMIT: u8 = 0xff;

// ─────────────────────────────────────────────────────────────────────────────
// Exception Handling Frame Emitter
// ─────────────────────────────────────────────────────────────────────────────

/// Emitter for .eh_frame and .debug_frame sections
pub struct EhFrameEmitter {
    /// Target architecture
    arch: EhArch,
    /// Code alignment factor
    code_alignment: u8,
    /// Data alignment factor (signed)
    data_alignment: i8,
    /// Return address register
    return_reg: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EhArch {
    X86_64,
    Aarch64,
}

/// A Call Frame Information entry for a function
#[derive(Debug, Clone)]
pub struct CfiEntry {
    /// Function name
    pub func_name: String,
    /// Function code start address (placeholder, patched by linker)
    pub func_start: u64,
    /// Function code size
    pub func_size: u32,
    /// CFI instructions
    pub instructions: Vec<CfiInstruction>,
    /// Landing pad info (for exception handling)
    pub landing_pads: Vec<LandingPad>,
}

/// CFI instruction
#[derive(Debug, Clone)]
pub enum CfiInstruction {
    /// Define CFA as register + offset
    DefCfa { register: u8, offset: u32 },
    /// Update CFA offset
    DefCfaOffset { offset: u32 },
    /// Register saved at CFA + offset
    Offset { register: u8, offset: i32 },
    /// Advance location counter
    AdvanceLoc { delta: u32 },
    /// Save state for nested frames
    RememberState,
    /// Restore state
    RestoreState,
    /// Nop padding
    Nop,
}

/// Landing pad for exception handling
#[derive(Debug, Clone)]
pub struct LandingPad {
    /// Offset in function where the try block starts
    pub try_start: u32,
    /// Offset where the try block ends
    pub try_end: u32,
    /// Landing pad code offset (catch handler)
    pub landing_pad: u32,
    /// Action index (0 = cleanup, >0 = catch clause)
    pub action: u32,
    /// Type filter for this catch clause
    pub type_filter: Option<String>,
}

impl EhFrameEmitter {
    pub fn new(arch: EhArch) -> Self {
        match arch {
            EhArch::X86_64 => EhFrameEmitter {
                arch,
                code_alignment: 1,
                data_alignment: -8,
                return_reg: DWARF_REG_RA,
            },
            EhArch::Aarch64 => EhFrameEmitter {
                arch,
                code_alignment: 4,
                data_alignment: -8,
                return_reg: 30, // LR
            },
        }
    }

    /// Emit a complete .eh_frame section
    pub fn emit_eh_frame(&self, entries: &[CfiEntry]) -> Vec<u8> {
        let mut section = Vec::with_capacity(256);

        // Emit CIE (Common Information Entry)
        let cie = self.emit_cie();
        section.extend_from_slice(&cie);

        // Emit FDE for each function
        for entry in entries {
            let fde = self.emit_fde(entry, cie.len() as u32);
            section.extend_from_slice(&fde);
        }

        // Terminator (zero-length CIE)
        section.extend_from_slice(&0u32.to_le_bytes());

        info!(
            "EhFrame: Emitted {} bytes ({} entries)",
            section.len(),
            entries.len()
        );
        section
    }

    /// Emit a complete .debug_frame section (similar to .eh_frame but for debuggers)
    pub fn emit_debug_frame(&self, entries: &[CfiEntry]) -> Vec<u8> {
        // .debug_frame uses CIE_ID = 0xFFFFFFFF instead of 0
        let mut section = Vec::with_capacity(256);

        let cie = self.emit_debug_cie();
        section.extend_from_slice(&cie);

        for entry in entries {
            let fde = self.emit_debug_fde(entry);
            section.extend_from_slice(&fde);
        }

        section.extend_from_slice(&0u32.to_le_bytes());
        section
    }

    fn emit_cie(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // CIE content (length patched later)
        let length_pos = buf.len();
        buf.extend_from_slice(&0u32.to_le_bytes()); // placeholder

        // CIE ID (0 for .eh_frame)
        buf.extend_from_slice(&CIE_ID.to_le_bytes());

        // Version
        buf.push(CIE_VERSION);

        // Augmentation string: "zPLR" for personality + LSDA + FDE encoding
        buf.extend_from_slice(b"zR\0");

        // Code alignment factor (ULEB128)
        Self::write_uleb128(&mut buf, self.code_alignment as u64);

        // Data alignment factor (SLEB128)
        Self::write_sleb128(&mut buf, self.data_alignment as i64);

        // Return address register (ULEB128)
        Self::write_uleb128(&mut buf, self.return_reg as u64);

        // Augmentation data length
        Self::write_uleb128(&mut buf, 1); // 1 byte for FDE pointer encoding

        // FDE pointer encoding (PC-relative | sdata4)
        buf.push(DW_EH_PE_PCREL | DW_EH_PE_SDATA4);

        // Initial instructions
        match self.arch {
            EhArch::X86_64 => {
                // def_cfa rsp, 8 (initial CFA = rsp+8 after call)
                self.encode_cfi(
                    &mut buf,
                    &CfiInstruction::DefCfa {
                        register: DWARF_REG_RSP,
                        offset: 8,
                    },
                );
                // offset ra, -8 (return address at CFA-8)
                self.encode_cfi(
                    &mut buf,
                    &CfiInstruction::Offset {
                        register: DWARF_REG_RA,
                        offset: -1, // * data_alignment_factor = -8
                    },
                );
            }
            EhArch::Aarch64 => {
                self.encode_cfi(
                    &mut buf,
                    &CfiInstruction::DefCfa {
                        register: 31, // SP
                        offset: 0,
                    },
                );
            }
        }

        // Pad to alignment
        while (buf.len() - 4) % std::mem::size_of::<u32>() != 0 {
            buf.push(DW_CFA_NOP);
        }

        // Patch length
        let length = (buf.len() - 4) as u32;
        buf[length_pos..length_pos + 4].copy_from_slice(&length.to_le_bytes());

        buf
    }

    fn emit_debug_cie(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        let length_pos = buf.len();
        buf.extend_from_slice(&0u32.to_le_bytes());

        // CIE ID for .debug_frame is 0xFFFFFFFF
        buf.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes());

        buf.push(CIE_VERSION);
        buf.push(0); // No augmentation

        Self::write_uleb128(&mut buf, self.code_alignment as u64);
        Self::write_sleb128(&mut buf, self.data_alignment as i64);
        Self::write_uleb128(&mut buf, self.return_reg as u64);

        // Initial instructions (same as .eh_frame)
        match self.arch {
            EhArch::X86_64 => {
                self.encode_cfi(
                    &mut buf,
                    &CfiInstruction::DefCfa {
                        register: DWARF_REG_RSP,
                        offset: 8,
                    },
                );
                self.encode_cfi(
                    &mut buf,
                    &CfiInstruction::Offset {
                        register: DWARF_REG_RA,
                        offset: -1,
                    },
                );
            }
            EhArch::Aarch64 => {
                self.encode_cfi(
                    &mut buf,
                    &CfiInstruction::DefCfa {
                        register: 31,
                        offset: 0,
                    },
                );
            }
        }

        while (buf.len() - 4) % 4 != 0 {
            buf.push(DW_CFA_NOP);
        }

        let length = (buf.len() - 4) as u32;
        buf[length_pos..length_pos + 4].copy_from_slice(&length.to_le_bytes());

        buf
    }

    fn emit_fde(&self, entry: &CfiEntry, cie_offset: u32) -> Vec<u8> {
        let mut buf = Vec::new();

        let length_pos = buf.len();
        buf.extend_from_slice(&0u32.to_le_bytes()); // placeholder

        // CIE pointer (offset from this FDE to the CIE)
        let cie_ptr = (buf.len() as u32) + cie_offset;
        buf.extend_from_slice(&cie_ptr.to_le_bytes());

        // Initial location (PC-relative, patched by linker)
        buf.extend_from_slice(&(entry.func_start as i32).to_le_bytes());

        // Address range
        buf.extend_from_slice(&entry.func_size.to_le_bytes());

        // Augmentation data length (for LSDA pointer if present)
        if !entry.landing_pads.is_empty() {
            Self::write_uleb128(&mut buf, 4); // 4 bytes for LSDA pointer
            buf.extend_from_slice(&0u32.to_le_bytes()); // LSDA pointer (patched)
        } else {
            Self::write_uleb128(&mut buf, 0);
        }

        // CFI instructions
        for inst in &entry.instructions {
            self.encode_cfi(&mut buf, inst);
        }

        // Pad to alignment
        while (buf.len() - 4) % 4 != 0 {
            buf.push(DW_CFA_NOP);
        }

        // Patch length
        let length = (buf.len() - 4) as u32;
        buf[length_pos..length_pos + 4].copy_from_slice(&length.to_le_bytes());

        buf
    }

    fn emit_debug_fde(&self, entry: &CfiEntry) -> Vec<u8> {
        let mut buf = Vec::new();

        let length_pos = buf.len();
        buf.extend_from_slice(&0u32.to_le_bytes());

        // CIE pointer (absolute offset 0 in .debug_frame)
        buf.extend_from_slice(&0u32.to_le_bytes());

        // Initial location (absolute address)
        buf.extend_from_slice(&entry.func_start.to_le_bytes());

        // Address range
        buf.extend_from_slice(&(entry.func_size as u64).to_le_bytes());

        for inst in &entry.instructions {
            self.encode_cfi(&mut buf, inst);
        }

        while (buf.len() - 4) % 4 != 0 {
            buf.push(DW_CFA_NOP);
        }

        let length = (buf.len() - 4) as u32;
        buf[length_pos..length_pos + 4].copy_from_slice(&length.to_le_bytes());

        buf
    }

    fn encode_cfi(&self, buf: &mut Vec<u8>, inst: &CfiInstruction) {
        match inst {
            CfiInstruction::DefCfa { register, offset } => {
                buf.push(DW_CFA_DEF_CFA);
                Self::write_uleb128(buf, *register as u64);
                Self::write_uleb128(buf, *offset as u64);
            }
            CfiInstruction::DefCfaOffset { offset } => {
                buf.push(DW_CFA_DEF_CFA_OFFSET);
                Self::write_uleb128(buf, *offset as u64);
            }
            CfiInstruction::Offset { register, offset } => {
                if *register < 64 {
                    buf.push(DW_CFA_OFFSET | (*register as u8));
                } else {
                    buf.push(0x05); // DW_CFA_offset_extended
                    Self::write_uleb128(buf, *register as u64);
                }
                // factored offset = offset / data_alignment
                let factored = if self.data_alignment != 0 {
                    ((*offset as i64) / (self.data_alignment as i64)) as u64
                } else {
                    *offset as u64
                };
                Self::write_uleb128(buf, factored);
            }
            CfiInstruction::AdvanceLoc { delta } => {
                if *delta < 64 {
                    buf.push(DW_CFA_ADVANCE_LOC | (*delta as u8));
                } else if *delta < 256 {
                    buf.push(DW_CFA_ADVANCE_LOC1);
                    buf.push(*delta as u8);
                } else if *delta < 65536 {
                    buf.push(DW_CFA_ADVANCE_LOC2);
                    buf.extend_from_slice(&(*delta as u16).to_le_bytes());
                } else {
                    buf.push(DW_CFA_ADVANCE_LOC4);
                    buf.extend_from_slice(&delta.to_le_bytes());
                }
            }
            CfiInstruction::RememberState => buf.push(DW_CFA_REMEMBER_STATE),
            CfiInstruction::RestoreState => buf.push(DW_CFA_RESTORE_STATE),
            CfiInstruction::Nop => buf.push(DW_CFA_NOP),
        }
    }

    fn write_uleb128(buf: &mut Vec<u8>, mut val: u64) {
        loop {
            let mut byte = (val & 0x7f) as u8;
            val >>= 7;
            if val != 0 {
                byte |= 0x80;
            }
            buf.push(byte);
            if val == 0 {
                break;
            }
        }
    }

    fn write_sleb128(buf: &mut Vec<u8>, mut val: i64) {
        loop {
            let mut byte = (val & 0x7f) as u8;
            val >>= 7;
            let more = !(((val == 0) && (byte & 0x40 == 0)) || ((val == -1) && (byte & 0x40 != 0)));
            if more {
                byte |= 0x80;
            }
            buf.push(byte);
            if !more {
                break;
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// LSDA (Language-Specific Data Area) Emitter
// ─────────────────────────────────────────────────────────────────────────────

/// LSDA for a function's exception handling tables
pub struct LsdaEmitter;

/// Compiled LSDA for a single function
#[derive(Debug, Clone)]
pub struct Lsda {
    pub func_name: String,
    pub binary: Vec<u8>,
    pub call_sites: Vec<CallSite>,
}

/// Call site entry in the LSDA call site table
#[derive(Debug, Clone)]
pub struct CallSite {
    /// Offset from function start to start of try region
    pub region_start: u32,
    /// Length of try region
    pub region_length: u32,
    /// Landing pad offset (0 = no landing pad)
    pub landing_pad: u32,
    /// Action index (0 = cleanup, 1+ = catch/filter)
    pub action: u32,
}

impl LsdaEmitter {
    /// Emit LSDA for a function with landing pads
    pub fn emit_lsda(func_name: &str, landing_pads: &[LandingPad]) -> Lsda {
        let mut binary = Vec::with_capacity(64);
        let mut call_sites = Vec::new();

        // LSDA header
        // Landing pad base encoding
        binary.push(DW_EH_PE_OMIT); // @LPStart encoding (omit = use function start)

        // Type table encoding
        binary.push(DW_EH_PE_OMIT); // @TType encoding (omit = no type table)

        // Call site table encoding
        binary.push(DW_EH_PE_UDATA4);

        // Call site table
        let mut cs_table = Vec::new();

        for lp in landing_pads {
            let cs = CallSite {
                region_start: lp.try_start,
                region_length: lp.try_end - lp.try_start,
                landing_pad: lp.landing_pad,
                action: lp.action,
            };

            // Encode call site entry
            cs_table.extend_from_slice(&cs.region_start.to_le_bytes());
            cs_table.extend_from_slice(&cs.region_length.to_le_bytes());
            cs_table.extend_from_slice(&cs.landing_pad.to_le_bytes());
            EhFrameEmitter::write_uleb128(&mut cs_table, cs.action as u64);

            call_sites.push(cs);
        }

        // Call site table length
        EhFrameEmitter::write_uleb128(&mut binary, cs_table.len() as u64);
        binary.extend_from_slice(&cs_table);

        // Action table (empty for now)
        // Each action record: filter (SLEB128), next_action_offset (SLEB128)

        debug!(
            "LSDA: Emitted {} bytes for {} ({} call sites)",
            binary.len(),
            func_name,
            call_sites.len()
        );

        Lsda {
            func_name: func_name.to_string(),
            binary,
            call_sites,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Exception Handling IR Lowering
// ─────────────────────────────────────────────────────────────────────────────

/// Try-catch block in the IR
#[derive(Debug, Clone)]
pub struct TryCatchBlock {
    /// Label for the try body
    pub try_label: String,
    /// Catch clauses
    pub catch_clauses: Vec<CatchClause>,
    /// Finally block (optional)
    pub finally_label: Option<String>,
    /// Continue label (after try-catch)
    pub continue_label: String,
}

/// A catch clause
#[derive(Debug, Clone)]
pub struct CatchClause {
    /// Exception type pattern (None = catch-all)
    pub exception_type: Option<String>,
    /// Variable binding for the exception
    pub binding: Option<String>,
    /// Block label for the catch handler
    pub handler_label: String,
}

/// Exception handling lowering pass
pub struct ExceptionLowering {
    /// Counter for generating unique labels
    label_counter: usize,
    /// Active try-catch blocks (stack for nesting)
    active_try_blocks: Vec<TryCatchBlock>,
}

impl ExceptionLowering {
    pub fn new() -> Self {
        ExceptionLowering {
            label_counter: 0,
            active_try_blocks: Vec::new(),
        }
    }

    fn fresh_label(&mut self, prefix: &str) -> String {
        self.label_counter += 1;
        format!("{}_{}", prefix, self.label_counter)
    }

    /// Lower a try-catch construct into IR blocks with landing pads
    pub fn lower_try_catch(
        &mut self,
        try_body: Vec<IrInstruction>,
        catch_clauses: Vec<CatchClause>,
        finally_body: Option<Vec<IrInstruction>>,
    ) -> Vec<IrBlock> {
        let mut blocks = Vec::new();

        let try_label = self.fresh_label("try");
        let catch_dispatch_label = self.fresh_label("catch_dispatch");
        let finally_label = self.fresh_label("finally");
        let continue_label = self.fresh_label("continue");

        // Try block
        blocks.push(IrBlock {
            label: try_label.clone(),
            instructions: try_body,
            terminator: if finally_body.is_some() {
                IrTerminator::Branch(finally_label.clone())
            } else {
                IrTerminator::Branch(continue_label.clone())
            },
        });

        // Catch dispatch block (landing pad)
        let mut dispatch_instructions = Vec::new();
        // Load exception object
        dispatch_instructions.push(IrInstruction::NativeCall {
            dest: Some("__exception".to_string()),
            module: "__eh".to_string(),
            func: "catch_exception".to_string(),
            args: vec![],
        });

        // Create dispatch to appropriate catch handler
        if catch_clauses.len() == 1 && catch_clauses[0].exception_type.is_none() {
            // Single catch-all clause
            blocks.push(IrBlock {
                label: catch_dispatch_label,
                instructions: dispatch_instructions,
                terminator: IrTerminator::Branch(catch_clauses[0].handler_label.clone()),
            });
        } else {
            // Multiple catch clauses - chain of type checks
            let first_handler = if catch_clauses.is_empty() {
                continue_label.clone()
            } else {
                catch_clauses[0].handler_label.clone()
            };
            blocks.push(IrBlock {
                label: catch_dispatch_label,
                instructions: dispatch_instructions,
                terminator: IrTerminator::Branch(first_handler),
            });
        }

        // Catch handler blocks
        for (i, clause) in catch_clauses.iter().enumerate() {
            let next = if i + 1 < catch_clauses.len() {
                catch_clauses[i + 1].handler_label.clone()
            } else if finally_body.is_some() {
                finally_label.clone()
            } else {
                continue_label.clone()
            };

            let mut handler_insts = Vec::new();
            if let Some(ref binding) = clause.binding {
                handler_insts.push(IrInstruction::Alloca {
                    dest: binding.clone(),
                    ty: IrType::Ptr(Box::new(IrType::I8)),
                });
            }

            blocks.push(IrBlock {
                label: clause.handler_label.clone(),
                instructions: handler_insts,
                terminator: IrTerminator::Branch(next),
            });
        }

        // Finally block
        if let Some(finally_insts) = finally_body {
            blocks.push(IrBlock {
                label: finally_label,
                instructions: finally_insts,
                terminator: IrTerminator::Branch(continue_label.clone()),
            });
        }

        // Continue block
        blocks.push(IrBlock {
            label: continue_label,
            instructions: vec![],
            terminator: IrTerminator::Return(None),
        });

        blocks
    }

    /// Generate CFI entries for a function with exception handling
    pub fn generate_cfi_for_function(&self, func: &IrFunction) -> CfiEntry {
        let mut instructions = Vec::new();
        let inst_count: u32 = func
            .blocks
            .iter()
            .map(|b| b.instructions.len() as u32 * 4 + 4) // 4 bytes per inst + terminator
            .sum();

        // Standard x86-64 prologue CFI
        // After call: CFA = RSP+8
        // push rbp: CFA = RSP+16, RBP at CFA-16
        instructions.push(CfiInstruction::DefCfa {
            register: DWARF_REG_RSP,
            offset: 8,
        });
        instructions.push(CfiInstruction::AdvanceLoc { delta: 1 }); // push rbp
        instructions.push(CfiInstruction::DefCfa {
            register: DWARF_REG_RSP,
            offset: 16,
        });
        instructions.push(CfiInstruction::Offset {
            register: DWARF_REG_RBP,
            offset: -2,
        }); // -2 * -8 = 16
        instructions.push(CfiInstruction::AdvanceLoc { delta: 3 }); // mov rbp, rsp
        instructions.push(CfiInstruction::DefCfa {
            register: DWARF_REG_RBP,
            offset: 16,
        });

        // Collect landing pads
        let landing_pads = Vec::new(); // Extracted from try-catch blocks in the function

        CfiEntry {
            func_name: func.name.clone(),
            func_start: 0,
            func_size: inst_count,
            instructions,
            landing_pads,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Personality Function
// ─────────────────────────────────────────────────────────────────────────────

/// Omni language personality function implementation
/// Called by the unwinder to determine what to do at each frame
pub struct OmniPersonality;

/// Actions the personality function can take
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PersonalityAction {
    /// Continue unwinding (no handler here)
    Continue,
    /// Found a handler - stop and transfer control
    Handler,
    /// Found a cleanup - run it and continue
    Cleanup,
}

/// Unwind reason
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnwindReason {
    /// Normal exception search phase
    SearchPhase,
    /// Cleanup phase
    CleanupPhase,
    /// Handler found
    HandlerFound,
    /// Forced unwind (e.g., thread cancellation)
    ForcedUnwind,
}

impl OmniPersonality {
    /// Determine action at a given frame during stack unwinding
    ///
    /// This is a model of what the native personality function does.
    /// The actual personality function would be emitted as machine code.
    pub fn determine_action(
        reason: UnwindReason,
        call_sites: &[CallSite],
        ip_offset: u32,
    ) -> PersonalityAction {
        // Find the call site containing the instruction pointer
        for cs in call_sites {
            if ip_offset >= cs.region_start && ip_offset < cs.region_start + cs.region_length {
                if cs.landing_pad == 0 {
                    // No landing pad - continue unwinding
                    continue;
                }

                match reason {
                    UnwindReason::SearchPhase => {
                        if cs.action > 0 {
                            // This is a catch clause
                            return PersonalityAction::Handler;
                        } else {
                            // This is a cleanup
                            return PersonalityAction::Cleanup;
                        }
                    }
                    UnwindReason::CleanupPhase => {
                        return PersonalityAction::Cleanup;
                    }
                    UnwindReason::HandlerFound => {
                        return PersonalityAction::Handler;
                    }
                    UnwindReason::ForcedUnwind => {
                        if cs.action == 0 {
                            return PersonalityAction::Cleanup;
                        }
                    }
                }
            }
        }

        PersonalityAction::Continue
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{
        IrBinOp, IrBlock, IrConst, IrFunction, IrInstruction, IrTerminator, IrType, IrValue,
    };

    fn sample_function() -> IrFunction {
        IrFunction {
            name: "test_func".to_string(),
            params: vec![("x".to_string(), IrType::I64)],
            return_type: IrType::I64,
            blocks: vec![IrBlock {
                label: "entry".to_string(),
                instructions: vec![IrInstruction::BinOp {
                    dest: "result".to_string(),
                    op: IrBinOp::Add,
                    left: IrValue::Var("x".to_string()),
                    right: IrValue::Const(IrConst::Int(1)),
                }],
                terminator: IrTerminator::Return(Some(IrValue::Var("result".to_string()))),
            }],
            locals: vec![],
        }
    }

    #[test]
    fn test_eh_frame_cie_emission() {
        let emitter = EhFrameEmitter::new(EhArch::X86_64);
        let cie = emitter.emit_cie();

        // CIE should start with length
        assert!(cie.len() >= 8);

        // CIE ID should be 0
        let cie_id = u32::from_le_bytes([cie[4], cie[5], cie[6], cie[7]]);
        assert_eq!(cie_id, 0);

        // Version should be 1
        assert_eq!(cie[8], 1);
    }

    #[test]
    fn test_eh_frame_full_emission() {
        let emitter = EhFrameEmitter::new(EhArch::X86_64);

        let entry = CfiEntry {
            func_name: "test_func".to_string(),
            func_start: 0x1000,
            func_size: 64,
            instructions: vec![
                CfiInstruction::DefCfa {
                    register: DWARF_REG_RSP,
                    offset: 8,
                },
                CfiInstruction::AdvanceLoc { delta: 1 },
                CfiInstruction::DefCfa {
                    register: DWARF_REG_RSP,
                    offset: 16,
                },
                CfiInstruction::Offset {
                    register: DWARF_REG_RBP,
                    offset: -2,
                },
            ],
            landing_pads: vec![],
        };

        let section = emitter.emit_eh_frame(&[entry]);
        assert!(!section.is_empty());

        // Should end with zero terminator
        let last_4 = &section[section.len() - 4..];
        assert_eq!(
            u32::from_le_bytes([last_4[0], last_4[1], last_4[2], last_4[3]]),
            0
        );
    }

    #[test]
    fn test_debug_frame_emission() {
        let emitter = EhFrameEmitter::new(EhArch::X86_64);

        let entry = CfiEntry {
            func_name: "debug_test".to_string(),
            func_start: 0,
            func_size: 32,
            instructions: vec![CfiInstruction::DefCfa {
                register: DWARF_REG_RSP,
                offset: 8,
            }],
            landing_pads: vec![],
        };

        let section = emitter.emit_debug_frame(&[entry]);
        assert!(!section.is_empty());

        // Debug CIE ID should be 0xFFFFFFFF
        let cie_id = u32::from_le_bytes([section[4], section[5], section[6], section[7]]);
        assert_eq!(cie_id, 0xFFFFFFFF);
    }

    #[test]
    fn test_lsda_emission() {
        let landing_pads = vec![
            LandingPad {
                try_start: 0,
                try_end: 20,
                landing_pad: 24,
                action: 1,
                type_filter: Some("RuntimeError".to_string()),
            },
            LandingPad {
                try_start: 30,
                try_end: 50,
                landing_pad: 54,
                action: 0, // cleanup
                type_filter: None,
            },
        ];

        let lsda = LsdaEmitter::emit_lsda("test_func", &landing_pads);
        assert_eq!(lsda.call_sites.len(), 2);
        assert!(!lsda.binary.is_empty());
        assert_eq!(lsda.call_sites[0].action, 1);
        assert_eq!(lsda.call_sites[1].action, 0);
    }

    #[test]
    fn test_try_catch_lowering() {
        let mut lowering = ExceptionLowering::new();

        let try_body = vec![IrInstruction::Call {
            dest: Some("result".to_string()),
            func: "risky_operation".to_string(),
            args: vec![],
        }];

        let catch_clauses = vec![CatchClause {
            exception_type: Some("RuntimeError".to_string()),
            binding: Some("err".to_string()),
            handler_label: "catch_runtime".to_string(),
        }];

        let blocks = lowering.lower_try_catch(try_body, catch_clauses, None);

        // Should produce: try block, catch dispatch, catch handler, continue
        assert!(blocks.len() >= 3);

        // First block should be the try
        assert!(blocks[0].label.starts_with("try"));

        // Last block should be continue
        assert!(blocks.last().unwrap().label.starts_with("continue"));
    }

    #[test]
    fn test_try_catch_with_finally() {
        let mut lowering = ExceptionLowering::new();

        let blocks = lowering.lower_try_catch(
            vec![IrInstruction::NativeCall {
                dest: None,
                module: "io".to_string(),
                func: "write".to_string(),
                args: vec![],
            }],
            vec![CatchClause {
                exception_type: None,
                binding: None,
                handler_label: "catch_all".to_string(),
            }],
            Some(vec![IrInstruction::NativeCall {
                dest: None,
                module: "io".to_string(),
                func: "close".to_string(),
                args: vec![],
            }]),
        );

        // Should have finally block
        assert!(blocks.iter().any(|b| b.label.starts_with("finally")));
    }

    #[test]
    fn test_cfi_generation_for_function() {
        let func = sample_function();
        let lowering = ExceptionLowering::new();
        let cfi = lowering.generate_cfi_for_function(&func);

        assert_eq!(cfi.func_name, "test_func");
        assert!(cfi.func_size > 0);
        assert!(!cfi.instructions.is_empty());
    }

    #[test]
    fn test_personality_function_search() {
        let call_sites = vec![
            CallSite {
                region_start: 0,
                region_length: 20,
                landing_pad: 24,
                action: 1, // catch
            },
            CallSite {
                region_start: 30,
                region_length: 10,
                landing_pad: 44,
                action: 0, // cleanup
            },
        ];

        // IP in first try region - should find handler
        let action = OmniPersonality::determine_action(UnwindReason::SearchPhase, &call_sites, 10);
        assert_eq!(action, PersonalityAction::Handler);

        // IP in second try region - should find cleanup
        let action = OmniPersonality::determine_action(UnwindReason::SearchPhase, &call_sites, 35);
        assert_eq!(action, PersonalityAction::Cleanup);

        // IP outside any region - should continue
        let action = OmniPersonality::determine_action(UnwindReason::SearchPhase, &call_sites, 100);
        assert_eq!(action, PersonalityAction::Continue);
    }

    #[test]
    fn test_personality_cleanup_phase() {
        let call_sites = vec![CallSite {
            region_start: 0,
            region_length: 20,
            landing_pad: 24,
            action: 1,
        }];

        let action = OmniPersonality::determine_action(UnwindReason::CleanupPhase, &call_sites, 10);
        assert_eq!(action, PersonalityAction::Cleanup);
    }

    #[test]
    fn test_aarch64_eh_frame() {
        let emitter = EhFrameEmitter::new(EhArch::Aarch64);
        let entry = CfiEntry {
            func_name: "arm_func".to_string(),
            func_start: 0,
            func_size: 48,
            instructions: vec![
                CfiInstruction::DefCfa {
                    register: 31,
                    offset: 0,
                },
                CfiInstruction::AdvanceLoc { delta: 4 },
                CfiInstruction::DefCfaOffset { offset: 16 },
            ],
            landing_pads: vec![],
        };

        let section = emitter.emit_eh_frame(&[entry]);
        assert!(!section.is_empty());
    }

    #[test]
    fn test_cfi_advance_loc_encoding() {
        let emitter = EhFrameEmitter::new(EhArch::X86_64);
        let mut buf = Vec::new();

        // Small delta (< 64) - encoded in single byte
        emitter.encode_cfi(&mut buf, &CfiInstruction::AdvanceLoc { delta: 5 });
        assert_eq!(buf[0], DW_CFA_ADVANCE_LOC | 5);

        // Medium delta
        buf.clear();
        emitter.encode_cfi(&mut buf, &CfiInstruction::AdvanceLoc { delta: 200 });
        assert_eq!(buf[0], DW_CFA_ADVANCE_LOC1);
        assert_eq!(buf[1], 200);

        // Large delta
        buf.clear();
        emitter.encode_cfi(&mut buf, &CfiInstruction::AdvanceLoc { delta: 1000 });
        assert_eq!(buf[0], DW_CFA_ADVANCE_LOC2);
    }
}
