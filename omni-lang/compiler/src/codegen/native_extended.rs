#![allow(dead_code)]
//! Extended Native Code Generation
//!
//! Additional output format support and architecture emitters:
//! - PE/COFF builder for Windows executables
//! - Mach-O builder for macOS executables
//! - RISC-V emitter for RISC-V 64-bit targets
//! - Linker integration for external linking

use crate::codegen::native_codegen::{AluOp, MachineInst};
use log::info;
use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────────────────────
// PE/COFF Builder (Windows executables)
// ─────────────────────────────────────────────────────────────────────────────

/// PE/COFF output builder for Windows targets
pub struct PeBuilder {
    /// Machine code (.text section)
    text: Vec<u8>,
    /// Data section
    data: Vec<u8>,
    /// Read-only data
    rdata: Vec<u8>,
    /// Symbol table
    symbols: Vec<CoffSymbol>,
    /// Machine type
    machine: PeMachine,
    /// Image characteristics
    characteristics: u16,
}

#[derive(Debug, Clone, Copy)]
pub enum PeMachine {
    Amd64 = 0x8664,
    Arm64 = 0xAA64,
    I386 = 0x14C,
}

#[derive(Debug, Clone)]
pub struct CoffSymbol {
    pub name: String,
    pub value: u32,
    pub section: u16,
    pub sym_type: u16,
    pub storage_class: u8,
}

impl PeBuilder {
    pub fn new(machine: PeMachine) -> Self {
        PeBuilder {
            text: Vec::new(),
            data: Vec::new(),
            rdata: Vec::new(),
            symbols: Vec::new(),
            machine,
            characteristics: 0x0022, // EXECUTABLE_IMAGE | LARGE_ADDRESS_AWARE
        }
    }

    pub fn add_text(&mut self, code: Vec<u8>) {
        self.text = code;
    }

    pub fn add_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }

    pub fn add_rdata(&mut self, data: Vec<u8>) {
        self.rdata = data;
    }

    pub fn add_symbol(&mut self, name: &str, value: u32, section: u16) {
        self.symbols.push(CoffSymbol {
            name: name.to_string(),
            value,
            section,
            sym_type: 0x20,   // DTYPE_FUNCTION
            storage_class: 2, // IMAGE_SYM_CLASS_EXTERNAL
        });
    }

    /// Build a COFF object file
    pub fn build_coff(&self) -> Vec<u8> {
        let mut coff = Vec::with_capacity(1024);

        // Count sections
        let mut section_count: u16 = 0;
        if !self.text.is_empty() {
            section_count += 1;
        }
        if !self.data.is_empty() {
            section_count += 1;
        }
        if !self.rdata.is_empty() {
            section_count += 1;
        }

        // COFF File Header (20 bytes)
        coff.extend_from_slice(&(self.machine as u16).to_le_bytes()); // Machine
        coff.extend_from_slice(&section_count.to_le_bytes()); // NumberOfSections
        coff.extend_from_slice(&0u32.to_le_bytes()); // TimeDateStamp
        let symtab_offset_pos = coff.len();
        coff.extend_from_slice(&0u32.to_le_bytes()); // PointerToSymbolTable (patched)
        coff.extend_from_slice(&(self.symbols.len() as u32).to_le_bytes()); // NumberOfSymbols
        coff.extend_from_slice(&0u16.to_le_bytes()); // SizeOfOptionalHeader
        coff.extend_from_slice(&self.characteristics.to_le_bytes()); // Characteristics

        // Section headers start at offset 20
        let header_size = 20 + section_count as usize * 40;
        let mut current_offset = header_size;

        // Section headers (40 bytes each)
        let sections: Vec<(&str, &[u8], u32)> = {
            let mut s = Vec::new();
            if !self.text.is_empty() {
                s.push((".text\0\0\0", self.text.as_slice(), 0x60000020u32)); // CODE | EXECUTE | READ
            }
            if !self.data.is_empty() {
                s.push((".data\0\0\0", self.data.as_slice(), 0xC0000040u32)); // INITIALIZED | READ | WRITE
            }
            if !self.rdata.is_empty() {
                s.push((".rdata\0\0", self.rdata.as_slice(), 0x40000040u32)); // INITIALIZED | READ
            }
            s
        };

        for (name, data, chars) in &sections {
            // Name (8 bytes)
            let name_bytes = name.as_bytes();
            coff.extend_from_slice(&name_bytes[..8.min(name_bytes.len())]);
            for _ in name_bytes.len()..8 {
                coff.push(0);
            }

            // VirtualSize
            coff.extend_from_slice(&(data.len() as u32).to_le_bytes());
            // VirtualAddress
            coff.extend_from_slice(&0u32.to_le_bytes());
            // SizeOfRawData
            coff.extend_from_slice(&(data.len() as u32).to_le_bytes());
            // PointerToRawData
            coff.extend_from_slice(&(current_offset as u32).to_le_bytes());
            // PointerToRelocations
            coff.extend_from_slice(&0u32.to_le_bytes());
            // PointerToLinenumbers
            coff.extend_from_slice(&0u32.to_le_bytes());
            // NumberOfRelocations
            coff.extend_from_slice(&0u16.to_le_bytes());
            // NumberOfLinenumbers
            coff.extend_from_slice(&0u16.to_le_bytes());
            // Characteristics
            coff.extend_from_slice(&chars.to_le_bytes());

            current_offset += data.len();
        }

        // Section data
        for (_, data, _) in &sections {
            coff.extend_from_slice(data);
        }

        // Symbol table
        let symtab_offset = coff.len() as u32;
        for sym in &self.symbols {
            // Name (8 bytes) - short name or string table reference
            let name_bytes = sym.name.as_bytes();
            if name_bytes.len() <= 8 {
                coff.extend_from_slice(&name_bytes[..name_bytes.len().min(8)]);
                for _ in name_bytes.len()..8 {
                    coff.push(0);
                }
            } else {
                coff.extend_from_slice(&0u32.to_le_bytes()); // zeroes => string table
                coff.extend_from_slice(&4u32.to_le_bytes()); // offset into string table
            }
            coff.extend_from_slice(&sym.value.to_le_bytes());
            coff.extend_from_slice(&sym.section.to_le_bytes());
            coff.extend_from_slice(&sym.sym_type.to_le_bytes());
            coff.push(sym.storage_class);
            coff.push(0); // NumberOfAuxSymbols
        }

        // String table (required, minimum 4 bytes for size)
        let strtab_size = 4u32;
        coff.extend_from_slice(&strtab_size.to_le_bytes());

        // Patch symbol table offset
        coff[symtab_offset_pos..symtab_offset_pos + 4]
            .copy_from_slice(&symtab_offset.to_le_bytes());

        info!(
            "PE/COFF: Built {} bytes ({} sections, {} symbols)",
            coff.len(),
            section_count,
            self.symbols.len()
        );
        coff
    }

    /// Build a PE executable (with PE headers)
    pub fn build_pe(&self) -> Vec<u8> {
        let mut pe = Vec::with_capacity(2048);

        // DOS header stub
        pe.extend_from_slice(&[0x4D, 0x5A]); // "MZ"
        pe.extend_from_slice(&[0; 58]); // Zeroed DOS header
                                        // e_lfanew: offset to PE signature
        let pe_offset = 64u32;
        pe.extend_from_slice(&pe_offset.to_le_bytes());

        // PE Signature
        pe.extend_from_slice(b"PE\0\0");

        // COFF header
        pe.extend_from_slice(&(self.machine as u16).to_le_bytes());
        let section_count: u16 =
            if self.text.is_empty() { 0 } else { 1 } + if self.data.is_empty() { 0 } else { 1 };
        pe.extend_from_slice(&section_count.to_le_bytes());
        pe.extend_from_slice(&0u32.to_le_bytes()); // TimeDateStamp
        pe.extend_from_slice(&0u32.to_le_bytes()); // PointerToSymbolTable
        pe.extend_from_slice(&0u32.to_le_bytes()); // NumberOfSymbols
        pe.extend_from_slice(&240u16.to_le_bytes()); // SizeOfOptionalHeader (PE32+)
        pe.extend_from_slice(&self.characteristics.to_le_bytes());

        // Optional header (PE32+)
        pe.extend_from_slice(&0x20Bu16.to_le_bytes()); // PE32+ magic
        pe.push(14); // Major linker version
        pe.push(0); // Minor linker version
        pe.extend_from_slice(&(self.text.len() as u32).to_le_bytes()); // SizeOfCode
        pe.extend_from_slice(&(self.data.len() as u32).to_le_bytes()); // SizeOfInitializedData
        pe.extend_from_slice(&0u32.to_le_bytes()); // SizeOfUninitializedData
        pe.extend_from_slice(&0x1000u32.to_le_bytes()); // AddressOfEntryPoint
        pe.extend_from_slice(&0x1000u32.to_le_bytes()); // BaseOfCode

        // PE32+ specific
        pe.extend_from_slice(&0x140000000u64.to_le_bytes()); // ImageBase
        pe.extend_from_slice(&0x1000u32.to_le_bytes()); // SectionAlignment
        pe.extend_from_slice(&0x200u32.to_le_bytes()); // FileAlignment

        // OS versions
        pe.extend_from_slice(&6u16.to_le_bytes()); // MajorOperatingSystemVersion
        pe.extend_from_slice(&0u16.to_le_bytes());
        pe.extend_from_slice(&0u16.to_le_bytes());
        pe.extend_from_slice(&0u16.to_le_bytes());
        pe.extend_from_slice(&0u16.to_le_bytes());
        pe.extend_from_slice(&0u16.to_le_bytes());

        pe.extend_from_slice(&0u32.to_le_bytes()); // Win32VersionValue
        pe.extend_from_slice(&0x3000u32.to_le_bytes()); // SizeOfImage
        pe.extend_from_slice(&0x200u32.to_le_bytes()); // SizeOfHeaders
        pe.extend_from_slice(&0u32.to_le_bytes()); // CheckSum
        pe.extend_from_slice(&3u16.to_le_bytes()); // Subsystem (CONSOLE)
        pe.extend_from_slice(&0x8160u16.to_le_bytes()); // DllCharacteristics

        // Stack/heap sizes
        pe.extend_from_slice(&0x100000u64.to_le_bytes()); // SizeOfStackReserve
        pe.extend_from_slice(&0x1000u64.to_le_bytes()); // SizeOfStackCommit
        pe.extend_from_slice(&0x100000u64.to_le_bytes()); // SizeOfHeapReserve
        pe.extend_from_slice(&0x1000u64.to_le_bytes()); // SizeOfHeapCommit
        pe.extend_from_slice(&0u32.to_le_bytes()); // LoaderFlags
        pe.extend_from_slice(&16u32.to_le_bytes()); // NumberOfRvaAndSizes

        // Data directories (16 entries, all zero)
        for _ in 0..16 {
            pe.extend_from_slice(&0u32.to_le_bytes()); // VirtualAddress
            pe.extend_from_slice(&0u32.to_le_bytes()); // Size
        }

        // Pad to file alignment
        while pe.len() % 0x200 != 0 {
            pe.push(0);
        }

        // .text section header
        if !self.text.is_empty() {
            let text_file_offset = pe.len() + 40 * section_count as usize;
            let aligned = ((text_file_offset + 0x1FF) / 0x200) * 0x200;
            pe.extend_from_slice(b".text\0\0\0");
            pe.extend_from_slice(&(self.text.len() as u32).to_le_bytes());
            pe.extend_from_slice(&0x1000u32.to_le_bytes()); // VirtualAddress
            pe.extend_from_slice(&(self.text.len() as u32).to_le_bytes());
            pe.extend_from_slice(&(aligned as u32).to_le_bytes());
            pe.extend_from_slice(&[0; 12]); // Relocations/LineNumbers
            pe.extend_from_slice(&0x60000020u32.to_le_bytes()); // Characteristics
        }

        // Pad then write text section
        while pe.len() % 0x200 != 0 {
            pe.push(0);
        }
        pe.extend_from_slice(&self.text);
        while pe.len() % 0x200 != 0 {
            pe.push(0);
        }

        info!("PE: Built {} bytes", pe.len());
        pe
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Mach-O Builder (macOS executables)
// ─────────────────────────────────────────────────────────────────────────────

/// Mach-O output builder for macOS/iOS targets
pub struct MachOBuilder {
    /// Machine code
    text: Vec<u8>,
    /// Data
    data: Vec<u8>,
    /// Symbol table entries
    symbols: Vec<MachOSymbol>,
    /// CPU type
    cpu_type: MachOCpuType,
}

#[derive(Debug, Clone, Copy)]
pub enum MachOCpuType {
    X86_64,
    Arm64,
}

#[derive(Debug, Clone)]
pub struct MachOSymbol {
    pub name: String,
    pub offset: u64,
    pub size: u64,
}

// Mach-O constants
const MH_MAGIC_64: u32 = 0xFEEDFACF;
const MH_OBJECT: u32 = 1;
const MH_EXECUTE: u32 = 2;
const CPU_TYPE_X86_64: u32 = 0x0100_0007;
const CPU_TYPE_ARM64: u32 = 0x0100_000C;
const CPU_SUBTYPE_ALL: u32 = 3;
const CPU_SUBTYPE_ARM64_ALL: u32 = 0;
const LC_SEGMENT_64: u32 = 0x19;
const LC_SYMTAB: u32 = 0x02;

impl MachOBuilder {
    pub fn new(cpu_type: MachOCpuType) -> Self {
        MachOBuilder {
            text: Vec::new(),
            data: Vec::new(),
            symbols: Vec::new(),
            cpu_type,
        }
    }

    pub fn add_text(&mut self, code: Vec<u8>) {
        self.text = code;
    }

    pub fn add_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }

    pub fn add_symbol(&mut self, name: &str, offset: u64, size: u64) {
        self.symbols.push(MachOSymbol {
            name: name.to_string(),
            offset,
            size,
        });
    }

    /// Build a Mach-O object file
    pub fn build(&self) -> Vec<u8> {
        let mut macho = Vec::with_capacity(2048);

        let (cpu_type, cpu_subtype) = match self.cpu_type {
            MachOCpuType::X86_64 => (CPU_TYPE_X86_64, CPU_SUBTYPE_ALL),
            MachOCpuType::Arm64 => (CPU_TYPE_ARM64, CPU_SUBTYPE_ARM64_ALL),
        };

        // Mach-O 64-bit header (32 bytes)
        macho.extend_from_slice(&MH_MAGIC_64.to_le_bytes()); // magic
        macho.extend_from_slice(&cpu_type.to_le_bytes());
        macho.extend_from_slice(&cpu_subtype.to_le_bytes());
        macho.extend_from_slice(&MH_OBJECT.to_le_bytes()); // filetype
        let ncmds: u32 = 1; // Just __TEXT segment for now
        macho.extend_from_slice(&ncmds.to_le_bytes());
        let sizeofcmds_pos = macho.len();
        macho.extend_from_slice(&0u32.to_le_bytes()); // placeholder
        macho.extend_from_slice(&0u32.to_le_bytes()); // flags
        macho.extend_from_slice(&0u32.to_le_bytes()); // reserved

        let header_size = macho.len(); // 32 bytes

        // LC_SEGMENT_64 load command
        let segment_cmd_start = macho.len();
        macho.extend_from_slice(&LC_SEGMENT_64.to_le_bytes()); // cmd
        let cmdsize_pos = macho.len();
        macho.extend_from_slice(&0u32.to_le_bytes()); // cmdsize placeholder

        // Segment name "__TEXT" (16 bytes)
        let segname = b"__TEXT\0\0\0\0\0\0\0\0\0\0\0";
        macho.extend_from_slice(&segname[..16]);

        macho.extend_from_slice(&0u64.to_le_bytes()); // vmaddr
        macho.extend_from_slice(&(self.text.len() as u64).to_le_bytes()); // vmsize
        let fileoff_pos = macho.len();
        macho.extend_from_slice(&0u64.to_le_bytes()); // fileoff placeholder
        macho.extend_from_slice(&(self.text.len() as u64).to_le_bytes()); // filesize
        macho.extend_from_slice(&5u32.to_le_bytes()); // maxprot (VM_PROT_READ|EXECUTE)
        macho.extend_from_slice(&5u32.to_le_bytes()); // initprot
        macho.extend_from_slice(&1u32.to_le_bytes()); // nsects
        macho.extend_from_slice(&0u32.to_le_bytes()); // flags

        // Section header: __text (80 bytes)
        let sectname = b"__text\0\0\0\0\0\0\0\0\0\0";
        macho.extend_from_slice(&sectname[..16]);
        macho.extend_from_slice(&segname[..16]); // segment name
        macho.extend_from_slice(&0u64.to_le_bytes()); // addr
        macho.extend_from_slice(&(self.text.len() as u64).to_le_bytes()); // size
        let sect_offset_pos = macho.len();
        macho.extend_from_slice(&0u32.to_le_bytes()); // offset placeholder
        macho.extend_from_slice(&4u32.to_le_bytes()); // align (2^4 = 16)
        macho.extend_from_slice(&0u32.to_le_bytes()); // reloff
        macho.extend_from_slice(&0u32.to_le_bytes()); // nreloc
        macho.extend_from_slice(&0x80000400u32.to_le_bytes()); // flags: S_REGULAR | S_ATTR_PURE_INSTRUCTIONS | S_ATTR_SOME_INSTRUCTIONS
        macho.extend_from_slice(&0u32.to_le_bytes()); // reserved1
        macho.extend_from_slice(&0u32.to_le_bytes()); // reserved2
        macho.extend_from_slice(&0u32.to_le_bytes()); // reserved3

        // Patch cmdsize
        let cmdsize = (macho.len() - segment_cmd_start) as u32;
        macho[cmdsize_pos..cmdsize_pos + 4].copy_from_slice(&cmdsize.to_le_bytes());

        // Patch sizeofcmds
        let sizeofcmds = (macho.len() - header_size) as u32;
        macho[sizeofcmds_pos..sizeofcmds_pos + 4].copy_from_slice(&sizeofcmds.to_le_bytes());

        // Align to page boundary before code
        while macho.len() % 16 != 0 {
            macho.push(0);
        }

        // Patch file offsets
        let text_offset = macho.len() as u64;
        macho[fileoff_pos..fileoff_pos + 8].copy_from_slice(&text_offset.to_le_bytes());
        macho[sect_offset_pos..sect_offset_pos + 4]
            .copy_from_slice(&(text_offset as u32).to_le_bytes());

        // Write code
        macho.extend_from_slice(&self.text);

        info!(
            "Mach-O: Built {} bytes ({} code bytes)",
            macho.len(),
            self.text.len()
        );
        macho
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// RISC-V Emitter
// ─────────────────────────────────────────────────────────────────────────────

/// RISC-V 64-bit (RV64GC) instruction emitter
pub struct RiscvEmitter {
    /// Emitted machine code
    code: Vec<u8>,
    /// Label positions
    labels: HashMap<String, usize>,
    /// Fixups for forward branches
    fixups: Vec<RiscvFixup>,
}

#[derive(Debug)]
struct RiscvFixup {
    offset: usize,
    label: String,
    kind: RiscvFixupKind,
}

#[derive(Debug)]
enum RiscvFixupKind {
    Branch, // B-type immediate
    Jal,    // J-type immediate
}

// RISC-V registers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RvReg {
    Zero = 0,
    Ra = 1,
    Sp = 2,
    Gp = 3,
    Tp = 4,
    T0 = 5,
    T1 = 6,
    T2 = 7,
    S0 = 8,
    S1 = 9, // s0 = fp
    A0 = 10,
    A1 = 11,
    A2 = 12,
    A3 = 13,
    A4 = 14,
    A5 = 15,
    A6 = 16,
    A7 = 17,
    S2 = 18,
    S3 = 19,
    S4 = 20,
    S5 = 21,
    S6 = 22,
    S7 = 23,
    S8 = 24,
    S9 = 25,
    S10 = 26,
    S11 = 27,
    T3 = 28,
    T4 = 29,
    T5 = 30,
    T6 = 31,
}

impl RiscvEmitter {
    pub fn new() -> Self {
        RiscvEmitter {
            code: Vec::new(),
            labels: HashMap::new(),
            fixups: Vec::new(),
        }
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }

    fn emit_word(&mut self, word: u32) {
        self.code.extend_from_slice(&word.to_le_bytes());
    }

    /// Emit a label at the current position
    pub fn label(&mut self, name: &str) {
        self.labels.insert(name.to_string(), self.code.len());
    }

    // R-type: funct7[31:25] | rs2[24:20] | rs1[19:15] | funct3[14:12] | rd[11:7] | opcode[6:0]
    fn r_type(&self, opcode: u32, rd: u8, funct3: u32, rs1: u8, rs2: u8, funct7: u32) -> u32 {
        opcode
            | ((rd as u32) << 7)
            | (funct3 << 12)
            | ((rs1 as u32) << 15)
            | ((rs2 as u32) << 20)
            | (funct7 << 25)
    }

    // I-type: imm[31:20] | rs1[19:15] | funct3[14:12] | rd[11:7] | opcode[6:0]
    fn i_type(&self, opcode: u32, rd: u8, funct3: u32, rs1: u8, imm: i32) -> u32 {
        opcode
            | ((rd as u32) << 7)
            | (funct3 << 12)
            | ((rs1 as u32) << 15)
            | (((imm as u32) & 0xFFF) << 20)
    }

    // S-type: imm[11:5] | rs2 | rs1 | funct3 | imm[4:0] | opcode
    fn s_type(&self, opcode: u32, funct3: u32, rs1: u8, rs2: u8, imm: i32) -> u32 {
        let imm = imm as u32;
        opcode
            | ((imm & 0x1F) << 7)
            | (funct3 << 12)
            | ((rs1 as u32) << 15)
            | ((rs2 as u32) << 20)
            | (((imm >> 5) & 0x7F) << 25)
    }

    // U-type: imm[31:12] | rd | opcode
    fn u_type(&self, opcode: u32, rd: u8, imm: u32) -> u32 {
        opcode | ((rd as u32) << 7) | (imm & 0xFFFFF000)
    }

    // ── Instruction emission ────────────────────────────────────────────

    /// ADD rd, rs1, rs2
    pub fn add(&mut self, rd: RvReg, rs1: RvReg, rs2: RvReg) {
        self.emit_word(self.r_type(0x33, rd as u8, 0, rs1 as u8, rs2 as u8, 0));
    }

    /// SUB rd, rs1, rs2
    pub fn sub(&mut self, rd: RvReg, rs1: RvReg, rs2: RvReg) {
        self.emit_word(self.r_type(0x33, rd as u8, 0, rs1 as u8, rs2 as u8, 0x20));
    }

    /// ADDI rd, rs1, imm12
    pub fn addi(&mut self, rd: RvReg, rs1: RvReg, imm: i32) {
        self.emit_word(self.i_type(0x13, rd as u8, 0, rs1 as u8, imm));
    }

    /// AND rd, rs1, rs2
    pub fn and(&mut self, rd: RvReg, rs1: RvReg, rs2: RvReg) {
        self.emit_word(self.r_type(0x33, rd as u8, 7, rs1 as u8, rs2 as u8, 0));
    }

    /// OR rd, rs1, rs2
    pub fn or(&mut self, rd: RvReg, rs1: RvReg, rs2: RvReg) {
        self.emit_word(self.r_type(0x33, rd as u8, 6, rs1 as u8, rs2 as u8, 0));
    }

    /// XOR rd, rs1, rs2
    pub fn xor(&mut self, rd: RvReg, rs1: RvReg, rs2: RvReg) {
        self.emit_word(self.r_type(0x33, rd as u8, 4, rs1 as u8, rs2 as u8, 0));
    }

    /// SLL rd, rs1, rs2 (shift left logical)
    pub fn sll(&mut self, rd: RvReg, rs1: RvReg, rs2: RvReg) {
        self.emit_word(self.r_type(0x33, rd as u8, 1, rs1 as u8, rs2 as u8, 0));
    }

    /// SRL rd, rs1, rs2 (shift right logical)
    pub fn srl(&mut self, rd: RvReg, rs1: RvReg, rs2: RvReg) {
        self.emit_word(self.r_type(0x33, rd as u8, 5, rs1 as u8, rs2 as u8, 0));
    }

    /// SLT rd, rs1, rs2 (set less than)
    pub fn slt(&mut self, rd: RvReg, rs1: RvReg, rs2: RvReg) {
        self.emit_word(self.r_type(0x33, rd as u8, 2, rs1 as u8, rs2 as u8, 0));
    }

    /// MUL rd, rs1, rs2 (RV64M extension)
    pub fn mul(&mut self, rd: RvReg, rs1: RvReg, rs2: RvReg) {
        self.emit_word(self.r_type(0x33, rd as u8, 0, rs1 as u8, rs2 as u8, 1));
    }

    /// DIV rd, rs1, rs2 (RV64M extension)
    pub fn div(&mut self, rd: RvReg, rs1: RvReg, rs2: RvReg) {
        self.emit_word(self.r_type(0x33, rd as u8, 4, rs1 as u8, rs2 as u8, 1));
    }

    /// REM rd, rs1, rs2 (RV64M extension)
    pub fn rem(&mut self, rd: RvReg, rs1: RvReg, rs2: RvReg) {
        self.emit_word(self.r_type(0x33, rd as u8, 6, rs1 as u8, rs2 as u8, 1));
    }

    /// LD rd, offset(rs1) (64-bit load)
    pub fn ld(&mut self, rd: RvReg, rs1: RvReg, offset: i32) {
        self.emit_word(self.i_type(0x03, rd as u8, 3, rs1 as u8, offset));
    }

    /// SD rs2, offset(rs1) (64-bit store)
    pub fn sd(&mut self, rs2: RvReg, rs1: RvReg, offset: i32) {
        self.emit_word(self.s_type(0x23, 3, rs1 as u8, rs2 as u8, offset));
    }

    /// LUI rd, imm (load upper immediate)
    pub fn lui(&mut self, rd: RvReg, imm: u32) {
        self.emit_word(self.u_type(0x37, rd as u8, imm));
    }

    /// AUIPC rd, imm (add upper immediate to PC)
    pub fn auipc(&mut self, rd: RvReg, imm: u32) {
        self.emit_word(self.u_type(0x17, rd as u8, imm));
    }

    /// JAL rd, offset (jump and link)
    pub fn jal(&mut self, rd: RvReg, offset: i32) {
        let imm = offset as u32;
        let word = 0x6F | ((rd as u32) << 7)
            | ((imm & 0xFF000))          // imm[19:12]
            | (((imm >> 11) & 1) << 20)  // imm[11]
            | (((imm >> 1) & 0x3FF) << 21) // imm[10:1]
            | (((imm >> 20) & 1) << 31); // imm[20]
        self.emit_word(word);
    }

    /// JALR rd, rs1, offset
    pub fn jalr(&mut self, rd: RvReg, rs1: RvReg, offset: i32) {
        self.emit_word(self.i_type(0x67, rd as u8, 0, rs1 as u8, offset));
    }

    /// RET (pseudo: JALR x0, ra, 0)
    pub fn ret(&mut self) {
        self.jalr(RvReg::Zero, RvReg::Ra, 0);
    }

    /// NOP (pseudo: ADDI x0, x0, 0)
    pub fn nop(&mut self) {
        self.addi(RvReg::Zero, RvReg::Zero, 0);
    }

    /// MV rd, rs (pseudo: ADDI rd, rs, 0)
    pub fn mv(&mut self, rd: RvReg, rs: RvReg) {
        self.addi(rd, rs, 0);
    }

    /// LI rd, imm (pseudo: load immediate)
    pub fn li(&mut self, rd: RvReg, imm: i64) {
        if imm >= -2048 && imm < 2048 {
            self.addi(rd, RvReg::Zero, imm as i32);
        } else {
            let upper = ((imm + 0x800) >> 12) as u32;
            let lower = (imm as i32) & 0xFFF;
            self.lui(rd, upper << 12);
            if lower != 0 {
                self.addi(rd, rd, lower);
            }
        }
    }

    /// ECALL (system call)
    pub fn ecall(&mut self) {
        self.emit_word(0x73);
    }

    /// Emit MachineInst (from the shared IR)
    pub fn emit(&mut self, inst: &MachineInst) {
        match inst {
            MachineInst::MovRR { dst, src } => {
                let rd = self.reg(*dst);
                let rs = self.reg(*src);
                self.mv(rd, rs);
            }
            MachineInst::MovRI { dst, imm } => {
                let rd = self.reg(*dst);
                self.li(rd, *imm);
            }
            MachineInst::AluRR { op, dst, src } => {
                let rd = self.reg(*dst);
                let rs = self.reg(*src);
                match op {
                    AluOp::Add => self.add(rd, rd, rs),
                    AluOp::Sub => self.sub(rd, rd, rs),
                    AluOp::And => self.and(rd, rd, rs),
                    AluOp::Or => self.or(rd, rd, rs),
                    AluOp::Xor => self.xor(rd, rd, rs),
                    AluOp::Shl => self.sll(rd, rd, rs),
                    AluOp::Shr => self.srl(rd, rd, rs),
                    AluOp::Mul => self.mul(rd, rd, rs),
                    AluOp::Div => self.div(rd, rd, rs),
                    _ => self.nop(),
                }
            }
            MachineInst::Load {
                dst, base, offset, ..
            } => {
                let rd = self.reg(*dst);
                let rs = self.reg(*base);
                self.ld(rd, rs, *offset);
            }
            MachineInst::Store {
                src, base, offset, ..
            } => {
                let rs2 = self.reg(*src);
                let rs1 = self.reg(*base);
                self.sd(rs2, rs1, *offset);
            }
            MachineInst::Push { reg } => {
                let r = self.reg(*reg);
                self.addi(RvReg::Sp, RvReg::Sp, -8);
                self.sd(r, RvReg::Sp, 0);
            }
            MachineInst::Pop { reg } => {
                let r = self.reg(*reg);
                self.ld(r, RvReg::Sp, 0);
                self.addi(RvReg::Sp, RvReg::Sp, 8);
            }
            MachineInst::Return => {
                self.ret();
            }
            MachineInst::Nop => {
                self.nop();
            }
            MachineInst::Call { target: _ } => {
                // JAL ra, target (offset will need fixup)
                self.jal(RvReg::Ra, 0); // Placeholder, needs linker fixup for target
            }
            MachineInst::Syscall => {
                self.ecall();
            }
            MachineInst::Label { name } => {
                self.label(name);
            }
            _ => {
                self.nop(); // Fallback
            }
        }
    }

    fn reg(&self, idx: u8) -> RvReg {
        match idx {
            0 => RvReg::A0,
            1 => RvReg::A1,
            2 => RvReg::A2,
            3 => RvReg::A3,
            4 => RvReg::A4,
            5 => RvReg::A5,
            6 => RvReg::A6,
            7 => RvReg::A7,
            8 => RvReg::S0,
            9 => RvReg::S1,
            10 => RvReg::T0,
            11 => RvReg::T1,
            12 => RvReg::T2,
            _ => RvReg::T3,
        }
    }

    /// Resolve branch fixups
    pub fn resolve_fixups(&mut self) {
        for fixup in &self.fixups {
            if let Some(&target) = self.labels.get(&fixup.label) {
                let offset = (target as i32) - (fixup.offset as i32);
                match fixup.kind {
                    RiscvFixupKind::Jal => {
                        let imm = offset as u32;
                        let word = 0x6F // JAL x0
                            | ((imm & 0xFF000))
                            | (((imm >> 11) & 1) << 20)
                            | (((imm >> 1) & 0x3FF) << 21)
                            | (((imm >> 20) & 1) << 31);
                        self.code[fixup.offset..fixup.offset + 4]
                            .copy_from_slice(&word.to_le_bytes());
                    }
                    RiscvFixupKind::Branch => {
                        // Patch B-type instruction
                    }
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Linker Integration
// ─────────────────────────────────────────────────────────────────────────────

/// External linker integration
pub struct LinkerDriver {
    /// Linker executable path
    linker_path: String,
    /// Library search paths
    lib_paths: Vec<String>,
    /// Libraries to link
    libraries: Vec<String>,
    /// Linker flags
    flags: Vec<String>,
}

impl LinkerDriver {
    pub fn new() -> Self {
        LinkerDriver {
            linker_path: Self::detect_linker(),
            lib_paths: Vec::new(),
            libraries: Vec::new(),
            flags: Vec::new(),
        }
    }

    fn detect_linker() -> String {
        // Try to find system linker
        #[cfg(target_os = "windows")]
        {
            "link.exe".to_string()
        }
        #[cfg(target_os = "linux")]
        {
            "ld".to_string()
        }
        #[cfg(target_os = "macos")]
        {
            "ld".to_string()
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            "ld".to_string()
        }
    }

    pub fn add_lib_path(&mut self, path: &str) {
        self.lib_paths.push(path.to_string());
    }

    pub fn add_library(&mut self, lib: &str) {
        self.libraries.push(lib.to_string());
    }

    pub fn add_flag(&mut self, flag: &str) {
        self.flags.push(flag.to_string());
    }

    /// Generate the linker command for a given object file and output
    pub fn build_command(&self, object_file: &str, output_file: &str) -> Vec<String> {
        let mut args = vec![self.linker_path.clone()];

        for path in &self.lib_paths {
            args.push(format!("-L{}", path));
        }

        args.push(object_file.to_string());
        args.push("-o".to_string());
        args.push(output_file.to_string());

        for lib in &self.libraries {
            args.push(format!("-l{}", lib));
        }

        args.extend(self.flags.iter().cloned());
        args
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pe_coff_builder() {
        let mut pe = PeBuilder::new(PeMachine::Amd64);
        pe.add_text(vec![0x55, 0x48, 0x89, 0xE5, 0xC3]); // push rbp; mov rbp,rsp; ret
        pe.add_symbol("_main", 0, 1);
        let coff = pe.build_coff();

        // Check COFF machine type
        let machine = u16::from_le_bytes([coff[0], coff[1]]);
        assert_eq!(machine, 0x8664); // AMD64
    }

    #[test]
    fn test_pe_exe_builder() {
        let mut pe = PeBuilder::new(PeMachine::Amd64);
        pe.add_text(vec![0xC3]); // ret
        let exe = pe.build_pe();

        // Check MZ signature
        assert_eq!(exe[0], 0x4D);
        assert_eq!(exe[1], 0x5A);

        // Check PE signature at e_lfanew offset
        let pe_offset = u32::from_le_bytes([exe[60], exe[61], exe[62], exe[63]]) as usize;
        assert_eq!(&exe[pe_offset..pe_offset + 4], b"PE\0\0");
    }

    #[test]
    fn test_macho_builder() {
        let mut macho = MachOBuilder::new(MachOCpuType::X86_64);
        macho.add_text(vec![0x55, 0x48, 0x89, 0xE5, 0xC3]);
        macho.add_symbol("_main", 0, 5);
        let binary = macho.build();

        // Check Mach-O magic
        let magic = u32::from_le_bytes([binary[0], binary[1], binary[2], binary[3]]);
        assert_eq!(magic, 0xFEEDFACF);

        // Check CPU type (x86_64)
        let cpu = u32::from_le_bytes([binary[4], binary[5], binary[6], binary[7]]);
        assert_eq!(cpu, CPU_TYPE_X86_64);
    }

    #[test]
    fn test_macho_arm64() {
        let mut macho = MachOBuilder::new(MachOCpuType::Arm64);
        macho.add_text(vec![0xD5, 0x03, 0x20, 0x1F]); // NOP
        let binary = macho.build();

        let cpu = u32::from_le_bytes([binary[4], binary[5], binary[6], binary[7]]);
        assert_eq!(cpu, CPU_TYPE_ARM64);
    }

    #[test]
    fn test_riscv_basic_instructions() {
        let mut emitter = RiscvEmitter::new();

        // NOP
        emitter.nop();
        let code = emitter.code();
        assert_eq!(code.len(), 4);

        // NOP is ADDI x0, x0, 0 = 0x00000013
        let nop = u32::from_le_bytes([code[0], code[1], code[2], code[3]]);
        assert_eq!(nop, 0x00000013);
    }

    #[test]
    fn test_riscv_add() {
        let mut emitter = RiscvEmitter::new();
        emitter.add(RvReg::A0, RvReg::A1, RvReg::A2);

        let code = emitter.code();
        let word = u32::from_le_bytes([code[0], code[1], code[2], code[3]]);

        // ADD a0, a1, a2
        // opcode=0x33, funct3=0, funct7=0
        // rd=a0(10), rs1=a1(11), rs2=a2(12)
        let expected = 0x33 | (10 << 7) | (0 << 12) | (11 << 15) | (12 << 20) | (0 << 25);
        assert_eq!(word, expected);
    }

    #[test]
    fn test_riscv_li_small() {
        let mut emitter = RiscvEmitter::new();
        emitter.li(RvReg::A0, 42);

        // Small immediate should use ADDI
        assert_eq!(emitter.code().len(), 4); // Single instruction
    }

    #[test]
    fn test_riscv_li_large() {
        let mut emitter = RiscvEmitter::new();
        emitter.li(RvReg::A0, 0x12345);

        // Large immediate should use LUI + ADDI
        assert_eq!(emitter.code().len(), 8); // Two instructions
    }

    #[test]
    fn test_riscv_ret() {
        let mut emitter = RiscvEmitter::new();
        emitter.ret();

        let code = emitter.code();
        let word = u32::from_le_bytes([code[0], code[1], code[2], code[3]]);

        // RET = JALR x0, ra, 0 = 0x00008067
        assert_eq!(word, 0x00008067);
    }

    #[test]
    fn test_riscv_emit_machine_inst() {
        let mut emitter = RiscvEmitter::new();
        emitter.emit(&MachineInst::Nop);
        emitter.emit(&MachineInst::Return);

        assert_eq!(emitter.code().len(), 8); // NOP + RET
    }

    #[test]
    fn test_linker_driver() {
        let mut linker = LinkerDriver::new();
        linker.add_lib_path("/usr/lib");
        linker.add_library("c");

        let cmd = linker.build_command("test.o", "test");
        assert!(cmd.len() >= 4);
        assert!(cmd.contains(&"test.o".to_string()));
        assert!(cmd.contains(&"-o".to_string()));
        assert!(cmd.contains(&"test".to_string()));
    }
}
