#![allow(dead_code)]
//! Omni Compiler Linker
//!
//! Takes machine code sections produced by native codegen and links them into
//! final executables for Linux (ELF64), Windows (PE/COFF), and macOS (Mach-O).
//!
//! Supports:
//!   - .text   — executable code
//!   - .data   — initialized read/write data
//!   - .rodata — read-only data (string literals, constants)
//!   - .bss    — uninitialized data (zero-filled at load time)
//!   - Symbol table with global / local / function / object bindings
//!   - Relocations (absolute 64-bit and PC-relative 32-bit)
//!   - Entry-point specification
//!   - Auto-detection of the host platform

use anyhow::{anyhow, bail, Context, Result};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

// ─────────────────────────────────────────────────────────────────────────────
// Constants — ELF64
// ─────────────────────────────────────────────────────────────────────────────

/// ELF magic: \x7FELF
const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];

// ELF identification indices
const EI_CLASS_64: u8 = 2;
const EI_DATA_LSB: u8 = 1; // Little-endian
const EI_VERSION_CURRENT: u8 = 1;
const EI_OSABI_NONE: u8 = 0;

// ELF types
const ET_EXEC: u16 = 2; // Executable
const ET_DYN: u16 = 3; // Shared object (PIE)

// ELF machine types
const EM_X86_64: u16 = 0x3E;
const EM_AARCH64: u16 = 0xB7;
const EM_RISCV: u16 = 0xF3;

// ELF section header types
const SHT_NULL: u32 = 0;
const SHT_PROGBITS: u32 = 1;
const SHT_SYMTAB: u32 = 2;
const SHT_STRTAB: u32 = 3;
const SHT_RELA: u32 = 4;
const SHT_NOBITS: u32 = 8;

// ELF section flags
const SHF_WRITE: u64 = 0x1;
const SHF_ALLOC: u64 = 0x2;
const SHF_EXECINSTR: u64 = 0x4;

// ELF program header types
const PT_LOAD: u32 = 1;

// ELF program header flags
const PF_X: u32 = 0x1; // Execute
const PF_W: u32 = 0x2; // Write
const PF_R: u32 = 0x4; // Read

// ELF symbol binding
const STB_LOCAL: u8 = 0;
const STB_GLOBAL: u8 = 1;

// ELF symbol type
const STT_NOTYPE: u8 = 0;
const STT_OBJECT: u8 = 1;
const STT_FUNC: u8 = 2;
const STT_SECTION: u8 = 3;

// ELF special section indices
const SHN_UNDEF: u16 = 0;
const SHN_ABS: u16 = 0xFFF1;

// ELF relocation types (x86-64)
const R_X86_64_64: u32 = 1; // Absolute 64-bit
const R_X86_64_PC32: u32 = 2; // PC-relative 32-bit
const R_X86_64_PLT32: u32 = 4; // PLT-relative 32-bit

// Standard ELF virtual base address
const ELF_VADDR_BASE: u64 = 0x400000;

// Sizes
const ELF64_EHDR_SIZE: u16 = 64;
const ELF64_PHDR_SIZE: u16 = 56;
const ELF64_SHDR_SIZE: u16 = 64;
const ELF64_SYM_SIZE: u64 = 24;
const ELF64_RELA_SIZE: u64 = 24;

// ─────────────────────────────────────────────────────────────────────────────
// Constants — PE/COFF (Windows)
// ─────────────────────────────────────────────────────────────────────────────

/// DOS signature "MZ"
const PE_DOS_MAGIC: [u8; 2] = [b'M', b'Z'];
/// PE signature "PE\0\0"
const PE_SIGNATURE: [u8; 4] = [b'P', b'E', 0x00, 0x00];

// COFF machine types
const IMAGE_FILE_MACHINE_AMD64: u16 = 0x8664;
const IMAGE_FILE_MACHINE_ARM64: u16 = 0xAA64;

// PE optional header magic
const PE32_PLUS_MAGIC: u16 = 0x020B; // PE32+ (64-bit)

// PE section characteristics
const IMAGE_SCN_CNT_CODE: u32 = 0x0000_0020;
const IMAGE_SCN_CNT_INITIALIZED_DATA: u32 = 0x0000_0040;
const IMAGE_SCN_CNT_UNINITIALIZED_DATA: u32 = 0x0000_0080;
const IMAGE_SCN_MEM_EXECUTE: u32 = 0x2000_0000;
const IMAGE_SCN_MEM_READ: u32 = 0x4000_0000;
const IMAGE_SCN_MEM_WRITE: u32 = 0x8000_0000;

// PE file characteristics
const IMAGE_FILE_EXECUTABLE_IMAGE: u16 = 0x0002;
const IMAGE_FILE_LARGE_ADDRESS_AWARE: u16 = 0x0020;

// PE DLL characteristics
const IMAGE_DLLCHARACTERISTICS_HIGH_ENTROPY_VA: u16 = 0x0020;
const IMAGE_DLLCHARACTERISTICS_DYNAMIC_BASE: u16 = 0x0040;
const IMAGE_DLLCHARACTERISTICS_NX_COMPAT: u16 = 0x0100;
const IMAGE_DLLCHARACTERISTICS_TERMINAL_SERVER_AWARE: u16 = 0x8000;

// PE subsystem
const IMAGE_SUBSYSTEM_WINDOWS_CUI: u16 = 3; // Console application

// Standard PE alignments
const PE_SECTION_ALIGNMENT: u32 = 0x1000;
const PE_FILE_ALIGNMENT: u32 = 0x200;
const PE_IMAGE_BASE: u64 = 0x0000_0001_4000_0000; // Default 64-bit image base

// PE DOS stub size (we emit a minimal 64-byte DOS header)
const PE_DOS_HEADER_SIZE: usize = 64;
// Number of data directory entries
const PE_NUM_DATA_DIRS: u32 = 16;

// ─────────────────────────────────────────────────────────────────────────────
// Constants — Mach-O (macOS)
// ─────────────────────────────────────────────────────────────────────────────

/// Mach-O 64-bit magic (little-endian)
const MH_MAGIC_64: u32 = 0xFEED_FACF;

// Mach-O CPU types
const CPU_TYPE_X86_64: u32 = 0x0100_0007; // CPU_TYPE_X86 | CPU_ARCH_ABI64
const CPU_TYPE_ARM64: u32 = 0x0100_000C; // CPU_TYPE_ARM | CPU_ARCH_ABI64

// CPU subtypes
const CPU_SUBTYPE_ALL: u32 = 3;
const CPU_SUBTYPE_ARM64_ALL: u32 = 0;

// Mach-O file types
const MH_EXECUTE: u32 = 2;

// Mach-O flags
const MH_NOUNDEFS: u32 = 0x0000_0001;
const MH_PIE: u32 = 0x0020_0000;

// Load command types
const LC_SEGMENT_64: u32 = 0x19;
const LC_SYMTAB: u32 = 0x02;
const LC_DYSYMTAB: u32 = 0x0B;
const LC_MAIN: u32 = 0x8000_0028; // LC_MAIN (0x28 | LC_REQ_DYLD)
const LC_UNIXTHREAD: u32 = 0x05;

// Mach-O segment / section constants
const VM_PROT_READ: u32 = 0x01;
const VM_PROT_WRITE: u32 = 0x02;
const VM_PROT_EXECUTE: u32 = 0x04;

// Mach-O section types
const S_REGULAR: u32 = 0x0;
const S_ZEROFILL: u32 = 0x1;
const S_ATTR_PURE_INSTRUCTIONS: u32 = 0x8000_0000;
const S_ATTR_SOME_INSTRUCTIONS: u32 = 0x0000_0400;

// Mach-O header size (64-bit)
const MACHO64_HEADER_SIZE: usize = 32;

// Page size used for Mach-O segment alignment
const MACHO_PAGE_SIZE: u64 = 0x4000; // 16 KiB (arm64 macOS)

// ─────────────────────────────────────────────────────────────────────────────
// Data Types
// ─────────────────────────────────────────────────────────────────────────────

/// Target platform the linker emits for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetPlatform {
    LinuxX86_64,
    LinuxAarch64,
    WindowsX86_64,
    WindowsAarch64,
    MacOSX86_64,
    MacOSAarch64,
}

impl TargetPlatform {
    /// Detect the host platform at compile time.
    pub fn host() -> Result<Self> {
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        {
            return Ok(TargetPlatform::LinuxX86_64);
        }
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        {
            return Ok(TargetPlatform::LinuxAarch64);
        }
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        {
            return Ok(TargetPlatform::WindowsX86_64);
        }
        #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
        {
            return Ok(TargetPlatform::WindowsAarch64);
        }
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        {
            return Ok(TargetPlatform::MacOSX86_64);
        }
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            return Ok(TargetPlatform::MacOSAarch64);
        }

        #[allow(unreachable_code)]
        Err(anyhow!("Unsupported host platform"))
    }

    /// Parse a target triple string (e.g. "x86_64-unknown-linux-gnu").
    pub fn from_triple(triple: &str) -> Result<Self> {
        let t = triple.to_lowercase();
        if t.contains("x86_64") || t.contains("x86-64") || t.contains("amd64") {
            if t.contains("linux") {
                return Ok(Self::LinuxX86_64);
            } else if t.contains("windows")
                || t.contains("win32")
                || t.contains("msvc")
                || t.contains("mingw")
            {
                return Ok(Self::WindowsX86_64);
            } else if t.contains("apple") || t.contains("darwin") || t.contains("macos") {
                return Ok(Self::MacOSX86_64);
            }
        }
        if t.contains("aarch64") || t.contains("arm64") {
            if t.contains("linux") {
                return Ok(Self::LinuxAarch64);
            } else if t.contains("windows") {
                return Ok(Self::WindowsAarch64);
            } else if t.contains("apple") || t.contains("darwin") || t.contains("macos") {
                return Ok(Self::MacOSAarch64);
            }
        }
        bail!("Unrecognised target triple: {}", triple)
    }

    fn is_linux(self) -> bool {
        matches!(self, Self::LinuxX86_64 | Self::LinuxAarch64)
    }

    fn is_windows(self) -> bool {
        matches!(self, Self::WindowsX86_64 | Self::WindowsAarch64)
    }

    fn is_macos(self) -> bool {
        matches!(self, Self::MacOSX86_64 | Self::MacOSAarch64)
    }

    fn elf_machine(self) -> u16 {
        match self {
            Self::LinuxX86_64 | Self::WindowsX86_64 | Self::MacOSX86_64 => EM_X86_64,
            Self::LinuxAarch64 | Self::WindowsAarch64 | Self::MacOSAarch64 => EM_AARCH64,
        }
    }

    fn pe_machine(self) -> u16 {
        match self {
            Self::WindowsX86_64 | Self::LinuxX86_64 | Self::MacOSX86_64 => IMAGE_FILE_MACHINE_AMD64,
            Self::WindowsAarch64 | Self::LinuxAarch64 | Self::MacOSAarch64 => {
                IMAGE_FILE_MACHINE_ARM64
            }
        }
    }

    fn macho_cpu(self) -> (u32, u32) {
        match self {
            Self::MacOSX86_64 | Self::LinuxX86_64 | Self::WindowsX86_64 => {
                (CPU_TYPE_X86_64, CPU_SUBTYPE_ALL)
            }
            Self::MacOSAarch64 | Self::LinuxAarch64 | Self::WindowsAarch64 => {
                (CPU_TYPE_ARM64, CPU_SUBTYPE_ARM64_ALL)
            }
        }
    }
}

impl fmt::Display for TargetPlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LinuxX86_64 => write!(f, "x86_64-unknown-linux-gnu"),
            Self::LinuxAarch64 => write!(f, "aarch64-unknown-linux-gnu"),
            Self::WindowsX86_64 => write!(f, "x86_64-pc-windows-msvc"),
            Self::WindowsAarch64 => write!(f, "aarch64-pc-windows-msvc"),
            Self::MacOSX86_64 => write!(f, "x86_64-apple-darwin"),
            Self::MacOSAarch64 => write!(f, "aarch64-apple-darwin"),
        }
    }
}

/// Symbol binding — local or global.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolBinding {
    Local,
    Global,
}

/// Symbol kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Object,
    Section,
    NoType,
}

/// A symbol in the linker's symbol table.
#[derive(Debug, Clone)]
pub struct LinkerSymbol {
    pub name: String,
    /// Byte offset within its section.
    pub offset: u64,
    /// Size in bytes (0 if unknown).
    pub size: u64,
    /// Which section this symbol lives in (by name), or None for absolute/extern.
    pub section: Option<String>,
    pub binding: SymbolBinding,
    pub kind: SymbolKind,
}

/// Relocation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelocationType {
    /// Absolute 64-bit address.
    Abs64,
    /// PC-relative 32-bit (used for calls/branches).
    PcRel32,
}

/// A pending relocation.
#[derive(Debug, Clone)]
pub struct LinkerRelocation {
    /// Section containing the site to patch.
    pub section: String,
    /// Byte offset within that section.
    pub offset: u64,
    /// Name of the symbol being referenced.
    pub symbol: String,
    /// Addend (added to the resolved address).
    pub addend: i64,
    pub reloc_type: RelocationType,
}

/// A raw section supplied to the linker.
#[derive(Debug, Clone)]
pub struct InputSection {
    pub name: String,
    pub data: Vec<u8>,
    /// Desired alignment (must be a power of two).
    pub alignment: u64,
}

// ─────────────────────────────────────────────────────────────────────────────
// Linker
// ─────────────────────────────────────────────────────────────────────────────

/// The Omni Linker.
///
/// Consumes machine-code sections, a symbol table, and relocations, then
/// produces a final executable binary for the chosen [`TargetPlatform`].
pub struct Linker {
    /// Target platform (auto-detected or explicit).
    target: TargetPlatform,
    /// Named sections in input order.
    sections: Vec<InputSection>,
    /// Global + local symbol table.
    symbols: Vec<LinkerSymbol>,
    /// Pending relocations.
    relocations: Vec<LinkerRelocation>,
    /// Entry point symbol name (default: `_start` / `main`).
    entry_point: String,
    /// Generate position-independent executable.
    pie: bool,
    /// Output path (optional — only needed by `link_to_file`).
    output_path: Option<PathBuf>,
}

impl Linker {
    // ── Construction ──────────────────────────────────────────────────────

    /// Create a linker for the given target.
    pub fn new(target: TargetPlatform) -> Self {
        let default_entry = if target.is_windows() {
            "mainCRTStartup".to_string()
        } else {
            "_start".to_string()
        };

        Linker {
            target,
            sections: Vec::new(),
            symbols: Vec::new(),
            relocations: Vec::new(),
            entry_point: default_entry,
            pie: false,
            output_path: None,
        }
    }

    /// Create a linker that auto-detects the host platform.
    pub fn for_host() -> Result<Self> {
        Ok(Self::new(TargetPlatform::host()?))
    }

    /// Create a linker from a target-triple string.
    pub fn from_triple(triple: &str) -> Result<Self> {
        Ok(Self::new(TargetPlatform::from_triple(triple)?))
    }

    // ── Configuration ────────────────────────────────────────────────────

    /// Set the entry-point symbol name.
    pub fn set_entry_point(&mut self, name: &str) {
        self.entry_point = name.to_string();
    }

    /// Enable / disable PIE output.
    pub fn set_pie(&mut self, enable: bool) {
        self.pie = enable;
    }

    /// Set output file path (used by `link_to_file`).
    pub fn set_output_path(&mut self, path: impl Into<PathBuf>) {
        self.output_path = Some(path.into());
    }

    // ── Adding content ───────────────────────────────────────────────────

    /// Add a `.text` section (executable code).
    pub fn add_text(&mut self, code: Vec<u8>) {
        self.sections.push(InputSection {
            name: ".text".into(),
            data: code,
            alignment: 16,
        });
    }

    /// Add a `.data` section (initialized read/write data).
    pub fn add_data(&mut self, data: Vec<u8>) {
        self.sections.push(InputSection {
            name: ".data".into(),
            data,
            alignment: 8,
        });
    }

    /// Add a `.rodata` section (read-only data / string literals).
    pub fn add_rodata(&mut self, data: Vec<u8>) {
        self.sections.push(InputSection {
            name: ".rodata".into(),
            data,
            alignment: 8,
        });
    }

    /// Add a `.bss` section (uninitialized data — only size matters).
    pub fn add_bss(&mut self, size: usize) {
        self.sections.push(InputSection {
            name: ".bss".into(),
            data: vec![0u8; size], // logical zeros; never written to the file in ELF NOBITS
            alignment: 8,
        });
    }

    /// Add an arbitrary named section.
    pub fn add_section(&mut self, section: InputSection) {
        self.sections.push(section);
    }

    /// Register a symbol.
    pub fn add_symbol(&mut self, sym: LinkerSymbol) {
        self.symbols.push(sym);
    }

    /// Register a relocation.
    pub fn add_relocation(&mut self, reloc: LinkerRelocation) {
        self.relocations.push(reloc);
    }

    // ── Linking ──────────────────────────────────────────────────────────

    /// Link and return the final executable bytes.
    ///
    /// Automatically dispatches to the correct binary format based on the
    /// configured [`TargetPlatform`].
    pub fn link(&self) -> Result<Vec<u8>> {
        // Resolve symbols into a virtual-address map.
        let layout = self
            .compute_layout()
            .context("Failed to compute section layout")?;

        let resolved = self
            .resolve_symbols(&layout)
            .context("Symbol resolution failed")?;

        // Merge section data and apply relocations.
        let mut merged = self.merge_sections(&layout);
        self.apply_relocations(&mut merged, &layout, &resolved)
            .context("Relocation application failed")?;

        // Emit the correct binary format.
        if self.target.is_linux() {
            self.emit_elf64(&merged, &layout, &resolved)
                .context("ELF64 emission failed")
        } else if self.target.is_windows() {
            self.emit_pe(&merged, &layout, &resolved)
                .context("PE emission failed")
        } else if self.target.is_macos() {
            self.emit_macho(&merged, &layout, &resolved)
                .context("Mach-O emission failed")
        } else {
            bail!("No binary format for target {}", self.target)
        }
    }

    /// Link and write the executable to the configured output path.
    pub fn link_to_file(&self) -> Result<PathBuf> {
        let path = self
            .output_path
            .clone()
            .ok_or_else(|| anyhow!("No output path set"))?;
        let binary = self.link()?;
        std::fs::write(&path, &binary)
            .with_context(|| format!("Failed to write executable to {}", path.display()))?;

        // On Unix-like systems, mark executable.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
                .with_context(|| format!("Failed to chmod {}", path.display()))?;
        }

        Ok(path)
    }

    // ── Layout ───────────────────────────────────────────────────────────

    /// Compute the virtual address layout of all sections.
    fn compute_layout(&self) -> Result<SectionLayout> {
        let mut layout = SectionLayout::new();

        // Determine the base address depending on format.
        let base = if self.target.is_linux() {
            ELF_VADDR_BASE
        } else if self.target.is_windows() {
            PE_IMAGE_BASE
        } else {
            // Mach-O uses a pagezero + text segment starting at 0x100000000 on arm64.
            0x0000_0001_0000_0000u64
        };

        // Headers occupy the first page.
        let header_size: u64 = if self.target.is_linux() {
            // ELF header + up to 4 program headers
            ELF64_EHDR_SIZE as u64 + 4 * ELF64_PHDR_SIZE as u64
        } else if self.target.is_windows() {
            PE_SECTION_ALIGNMENT as u64 // first section starts at section alignment
        } else {
            MACHO_PAGE_SIZE
        };

        let mut vaddr = base + header_size;
        let page_size = if self.target.is_macos() {
            MACHO_PAGE_SIZE
        } else {
            0x1000
        };

        // Align vaddr to page boundary.
        vaddr = align_up(vaddr, page_size);

        for section in &self.sections {
            let align = section.alignment.max(1);
            vaddr = align_up(vaddr, align);

            layout.add(
                section.name.clone(),
                vaddr,
                section.data.len() as u64,
                align,
            );

            vaddr += section.data.len() as u64;
            // Each section gets page-aligned in the virtual address space.
            vaddr = align_up(vaddr, page_size);
        }

        Ok(layout)
    }

    /// Resolve every symbol to its final virtual address.
    fn resolve_symbols(&self, layout: &SectionLayout) -> Result<HashMap<String, u64>> {
        let mut resolved: HashMap<String, u64> = HashMap::new();

        for sym in &self.symbols {
            let addr = match &sym.section {
                Some(sec_name) => {
                    let sec_info = layout.get(sec_name).ok_or_else(|| {
                        anyhow!(
                            "Symbol '{}' references unknown section '{}'",
                            sym.name,
                            sec_name
                        )
                    })?;
                    sec_info.vaddr + sym.offset
                }
                None => sym.offset, // absolute symbol
            };
            if resolved.contains_key(&sym.name) && sym.binding == SymbolBinding::Global {
                bail!("Duplicate global symbol: '{}'", sym.name);
            }
            resolved.insert(sym.name.clone(), addr);
        }

        Ok(resolved)
    }

    /// Concatenate section data into a flat map keyed by section name.
    fn merge_sections(&self, _layout: &SectionLayout) -> HashMap<String, Vec<u8>> {
        let mut merged: HashMap<String, Vec<u8>> = HashMap::new();
        for section in &self.sections {
            merged
                .entry(section.name.clone())
                .or_default()
                .extend_from_slice(&section.data);
        }
        merged
    }

    /// Apply relocations in-place on the merged section data.
    fn apply_relocations(
        &self,
        merged: &mut HashMap<String, Vec<u8>>,
        layout: &SectionLayout,
        resolved: &HashMap<String, u64>,
    ) -> Result<()> {
        for reloc in &self.relocations {
            let target_addr = *resolved
                .get(&reloc.symbol)
                .ok_or_else(|| anyhow!("Undefined symbol in relocation: '{}'", reloc.symbol))?;

            let section_data = merged.get_mut(&reloc.section).ok_or_else(|| {
                anyhow!("Relocation references unknown section '{}'", reloc.section)
            })?;

            let site_sec = layout
                .get(&reloc.section)
                .ok_or_else(|| anyhow!("Section '{}' not in layout", reloc.section))?;
            let site_addr = site_sec.vaddr + reloc.offset;

            match reloc.reloc_type {
                RelocationType::Abs64 => {
                    let value = (target_addr as i64 + reloc.addend) as u64;
                    let off = reloc.offset as usize;
                    if off + 8 > section_data.len() {
                        bail!(
                            "Abs64 relocation at offset {} overflows section '{}'",
                            off,
                            reloc.section
                        );
                    }
                    section_data[off..off + 8].copy_from_slice(&value.to_le_bytes());
                }
                RelocationType::PcRel32 => {
                    // value = S + A - P   (S = target, A = addend, P = site address)
                    let value = (target_addr as i64 + reloc.addend - site_addr as i64) as i32;
                    let off = reloc.offset as usize;
                    if off + 4 > section_data.len() {
                        bail!(
                            "PcRel32 relocation at offset {} overflows section '{}'",
                            off,
                            reloc.section
                        );
                    }
                    section_data[off..off + 4].copy_from_slice(&value.to_le_bytes());
                }
            }
        }
        Ok(())
    }

    // ── Entry point resolution ───────────────────────────────────────────

    /// Resolve the entry-point address. Falls back to the start of .text.
    fn resolve_entry(&self, layout: &SectionLayout, resolved: &HashMap<String, u64>) -> u64 {
        if let Some(&addr) = resolved.get(&self.entry_point) {
            return addr;
        }
        // Fallback: start of .text
        layout.get(".text").map(|s| s.vaddr).unwrap_or(0)
    }

    // ─────────────────────────────────────────────────────────────────────
    // ELF64 Emission
    // ─────────────────────────────────────────────────────────────────────

    fn emit_elf64(
        &self,
        merged: &HashMap<String, Vec<u8>>,
        layout: &SectionLayout,
        resolved: &HashMap<String, u64>,
    ) -> Result<Vec<u8>> {
        let entry = self.resolve_entry(layout, resolved);
        let machine = self.target.elf_machine();

        // Collect sections in a stable order: .text, .rodata, .data, .bss, then others.
        let ordered = self.ordered_section_names();

        // We emit: ELF header, program headers, section data, section headers.
        // Program headers: one PT_LOAD per loadable section.
        let phdr_count = ordered.len() as u16;
        let phdr_offset = ELF64_EHDR_SIZE as u64;
        let data_start_file =
            align_up(phdr_offset + phdr_count as u64 * ELF64_PHDR_SIZE as u64, 16);

        // ── Build file content ───────────────────────────────────────────
        let mut binary = Vec::new();

        // ---------- ELF Header (64 bytes) ----------
        binary.extend_from_slice(&ELF_MAGIC);
        binary.push(EI_CLASS_64);
        binary.push(EI_DATA_LSB);
        binary.push(EI_VERSION_CURRENT);
        binary.push(EI_OSABI_NONE);
        binary.extend_from_slice(&[0u8; 8]); // EI_ABIVERSION + padding

        let e_type = if self.pie { ET_DYN } else { ET_EXEC };
        binary.extend_from_slice(&e_type.to_le_bytes());
        binary.extend_from_slice(&machine.to_le_bytes());
        binary.extend_from_slice(&1u32.to_le_bytes()); // e_version

        binary.extend_from_slice(&entry.to_le_bytes()); // e_entry
        binary.extend_from_slice(&phdr_offset.to_le_bytes()); // e_phoff
        let shoff_patch_pos = binary.len();
        binary.extend_from_slice(&0u64.to_le_bytes()); // e_shoff (patched later)

        binary.extend_from_slice(&0u32.to_le_bytes()); // e_flags
        binary.extend_from_slice(&ELF64_EHDR_SIZE.to_le_bytes()); // e_ehsize
        binary.extend_from_slice(&ELF64_PHDR_SIZE.to_le_bytes()); // e_phentsize
        binary.extend_from_slice(&phdr_count.to_le_bytes()); // e_phnum
        binary.extend_from_slice(&ELF64_SHDR_SIZE.to_le_bytes()); // e_shentsize
        let shnum = (ordered.len() + 1) as u16; // +1 null section
        binary.extend_from_slice(&shnum.to_le_bytes()); // e_shnum
        binary.extend_from_slice(&0u16.to_le_bytes()); // e_shstrndx

        // ---------- Program Headers ----------
        struct PhdrEntry {
            p_type: u32,
            p_flags: u32,
            p_offset: u64,
            p_vaddr: u64,
            p_filesz: u64,
            p_memsz: u64,
            p_align: u64,
        }

        let mut file_offset = data_start_file;
        let mut phdrs: Vec<PhdrEntry> = Vec::new();
        let mut section_file_offsets: HashMap<String, u64> = HashMap::new();

        for name in &ordered {
            let info = layout.get(name).unwrap();
            let sec_data = merged.get(name.as_str());
            let sec_len = sec_data.map(|d| d.len() as u64).unwrap_or(0);
            let is_bss = name == ".bss";
            let filesz = if is_bss { 0 } else { sec_len };
            let memsz = if is_bss { info.size } else { sec_len };

            // Align file offset.
            file_offset = align_up(file_offset, info.alignment.max(16));
            section_file_offsets.insert(name.clone(), file_offset);

            let flags = elf_section_pflags(name);

            phdrs.push(PhdrEntry {
                p_type: PT_LOAD,
                p_flags: flags,
                p_offset: file_offset,
                p_vaddr: info.vaddr,
                p_filesz: filesz,
                p_memsz: memsz,
                p_align: 0x1000,
            });

            if !is_bss {
                file_offset += filesz;
            }
        }

        // Write program headers into binary.
        for ph in &phdrs {
            binary.extend_from_slice(&ph.p_type.to_le_bytes());
            binary.extend_from_slice(&ph.p_flags.to_le_bytes());
            binary.extend_from_slice(&ph.p_offset.to_le_bytes());
            binary.extend_from_slice(&ph.p_vaddr.to_le_bytes());
            binary.extend_from_slice(&ph.p_vaddr.to_le_bytes()); // p_paddr = p_vaddr
            binary.extend_from_slice(&ph.p_filesz.to_le_bytes());
            binary.extend_from_slice(&ph.p_memsz.to_le_bytes());
            binary.extend_from_slice(&ph.p_align.to_le_bytes());
        }

        // ---------- Section Data ----------
        for name in &ordered {
            let target_offset = *section_file_offsets.get(name.as_str()).unwrap() as usize;
            // Pad to the correct file offset.
            while binary.len() < target_offset {
                binary.push(0);
            }
            if name != ".bss" {
                if let Some(data) = merged.get(name.as_str()) {
                    binary.extend_from_slice(data);
                }
            }
        }

        // ---------- Section Headers ----------
        // Align to 8 bytes for section header table.
        while binary.len() % 8 != 0 {
            binary.push(0);
        }
        let shoff = binary.len() as u64;

        // Null section header (index 0).
        binary.extend_from_slice(&[0u8; ELF64_SHDR_SIZE as usize]);

        for name in &ordered {
            let info = layout.get(name).unwrap();
            let is_bss = name == ".bss";
            let sh_type = if is_bss { SHT_NOBITS } else { SHT_PROGBITS };
            let sh_flags = elf_section_shflags(name);
            let sh_offset = section_file_offsets
                .get(name.as_str())
                .copied()
                .unwrap_or(0);
            let sh_size = if is_bss {
                info.size
            } else {
                merged
                    .get(name.as_str())
                    .map(|d| d.len() as u64)
                    .unwrap_or(0)
            };

            binary.extend_from_slice(&0u32.to_le_bytes()); // sh_name (simplified)
            binary.extend_from_slice(&sh_type.to_le_bytes()); // sh_type
            binary.extend_from_slice(&sh_flags.to_le_bytes()); // sh_flags
            binary.extend_from_slice(&info.vaddr.to_le_bytes()); // sh_addr
            binary.extend_from_slice(&sh_offset.to_le_bytes()); // sh_offset
            binary.extend_from_slice(&sh_size.to_le_bytes()); // sh_size
            binary.extend_from_slice(&0u32.to_le_bytes()); // sh_link
            binary.extend_from_slice(&0u32.to_le_bytes()); // sh_info
            binary.extend_from_slice(&info.alignment.to_le_bytes()); // sh_addralign
            binary.extend_from_slice(&0u64.to_le_bytes()); // sh_entsize
        }

        // Patch e_shoff in the ELF header.
        binary[shoff_patch_pos..shoff_patch_pos + 8].copy_from_slice(&shoff.to_le_bytes());

        Ok(binary)
    }

    // ─────────────────────────────────────────────────────────────────────
    // PE/COFF Emission  (Windows x64)
    // ─────────────────────────────────────────────────────────────────────

    fn emit_pe(
        &self,
        merged: &HashMap<String, Vec<u8>>,
        layout: &SectionLayout,
        resolved: &HashMap<String, u64>,
    ) -> Result<Vec<u8>> {
        let entry_rva = {
            let abs = self.resolve_entry(layout, resolved);
            // RVA = absolute address - image base
            (abs.wrapping_sub(PE_IMAGE_BASE)) as u32
        };
        let machine = self.target.pe_machine();
        let ordered = self.ordered_section_names();
        let num_sections = ordered.len() as u16;

        let mut binary = Vec::new();

        // ---------- DOS Header (64 bytes) ----------
        binary.extend_from_slice(&PE_DOS_MAGIC);
        // Remaining DOS header fields (mostly zero for a minimal stub).
        binary.extend_from_slice(&[0u8; 58]);
        // e_lfanew at offset 60: pointer to PE signature
        binary.extend_from_slice(&(PE_DOS_HEADER_SIZE as u32).to_le_bytes());

        // ---------- PE Signature ----------
        binary.extend_from_slice(&PE_SIGNATURE);

        // ---------- COFF Header (20 bytes) ----------
        binary.extend_from_slice(&machine.to_le_bytes()); // Machine
        binary.extend_from_slice(&num_sections.to_le_bytes()); // NumberOfSections
        binary.extend_from_slice(&0u32.to_le_bytes()); // TimeDateStamp
        binary.extend_from_slice(&0u32.to_le_bytes()); // PointerToSymbolTable
        binary.extend_from_slice(&0u32.to_le_bytes()); // NumberOfSymbols
        let optional_hdr_size: u16 = 112 + (PE_NUM_DATA_DIRS * 8) as u16; // PE32+ opt header
        binary.extend_from_slice(&optional_hdr_size.to_le_bytes()); // SizeOfOptionalHeader
        let characteristics = IMAGE_FILE_EXECUTABLE_IMAGE | IMAGE_FILE_LARGE_ADDRESS_AWARE;
        binary.extend_from_slice(&characteristics.to_le_bytes()); // Characteristics

        // ---------- Optional Header (PE32+) ----------
        binary.extend_from_slice(&PE32_PLUS_MAGIC.to_le_bytes()); // Magic
        binary.push(14); // MajorLinkerVersion
        binary.push(0); // MinorLinkerVersion

        // Compute sizes
        let total_code_size: u32 = ordered
            .iter()
            .filter(|n| *n == ".text")
            .map(|n| merged.get(n.as_str()).map(|d| d.len() as u32).unwrap_or(0))
            .sum();
        let total_init_data: u32 = ordered
            .iter()
            .filter(|n| *n == ".data" || *n == ".rodata")
            .map(|n| merged.get(n.as_str()).map(|d| d.len() as u32).unwrap_or(0))
            .sum();
        let total_uninit_data: u32 = ordered
            .iter()
            .filter(|n| *n == ".bss")
            .map(|n| layout.get(n).map(|i| i.size as u32).unwrap_or(0))
            .sum();

        binary.extend_from_slice(&total_code_size.to_le_bytes()); // SizeOfCode
        binary.extend_from_slice(&total_init_data.to_le_bytes()); // SizeOfInitializedData
        binary.extend_from_slice(&total_uninit_data.to_le_bytes()); // SizeOfUninitializedData
        binary.extend_from_slice(&entry_rva.to_le_bytes()); // AddressOfEntryPoint

        // BaseOfCode — RVA of .text
        let text_rva = layout
            .get(".text")
            .map(|s| (s.vaddr.wrapping_sub(PE_IMAGE_BASE)) as u32)
            .unwrap_or(PE_SECTION_ALIGNMENT);
        binary.extend_from_slice(&text_rva.to_le_bytes()); // BaseOfCode

        // PE32+ fields (64-bit)
        binary.extend_from_slice(&PE_IMAGE_BASE.to_le_bytes()); // ImageBase
        binary.extend_from_slice(&PE_SECTION_ALIGNMENT.to_le_bytes()); // SectionAlignment
        binary.extend_from_slice(&PE_FILE_ALIGNMENT.to_le_bytes()); // FileAlignment
        binary.extend_from_slice(&6u16.to_le_bytes()); // MajorOSVersion
        binary.extend_from_slice(&0u16.to_le_bytes()); // MinorOSVersion
        binary.extend_from_slice(&0u16.to_le_bytes()); // MajorImageVersion
        binary.extend_from_slice(&0u16.to_le_bytes()); // MinorImageVersion
        binary.extend_from_slice(&6u16.to_le_bytes()); // MajorSubsystemVersion
        binary.extend_from_slice(&0u16.to_le_bytes()); // MinorSubsystemVersion
        binary.extend_from_slice(&0u32.to_le_bytes()); // Win32VersionValue

        // SizeOfImage — must be a multiple of SectionAlignment.
        // Compute from the highest section end.
        let image_end = ordered
            .iter()
            .filter_map(|n| layout.get(n))
            .map(|s| s.vaddr + s.size)
            .max()
            .unwrap_or(PE_IMAGE_BASE);
        let size_of_image = align_up(
            (image_end.wrapping_sub(PE_IMAGE_BASE)) as u64,
            PE_SECTION_ALIGNMENT as u64,
        ) as u32;
        binary.extend_from_slice(&size_of_image.to_le_bytes()); // SizeOfImage

        // SizeOfHeaders — headers + section table, file-aligned.
        let headers_raw = PE_DOS_HEADER_SIZE
            + 4 // PE sig
            + 20 // COFF header
            + optional_hdr_size as usize
            + num_sections as usize * 40; // section headers
        let size_of_headers = align_up(headers_raw as u64, PE_FILE_ALIGNMENT as u64) as u32;
        binary.extend_from_slice(&size_of_headers.to_le_bytes()); // SizeOfHeaders

        binary.extend_from_slice(&0u32.to_le_bytes()); // CheckSum
        binary.extend_from_slice(&IMAGE_SUBSYSTEM_WINDOWS_CUI.to_le_bytes()); // Subsystem
        let dll_chars = IMAGE_DLLCHARACTERISTICS_HIGH_ENTROPY_VA
            | IMAGE_DLLCHARACTERISTICS_DYNAMIC_BASE
            | IMAGE_DLLCHARACTERISTICS_NX_COMPAT
            | IMAGE_DLLCHARACTERISTICS_TERMINAL_SERVER_AWARE;
        binary.extend_from_slice(&dll_chars.to_le_bytes()); // DllCharacteristics

        // Stack / Heap sizes
        binary.extend_from_slice(&(1u64 << 20).to_le_bytes()); // SizeOfStackReserve (1 MiB)
        binary.extend_from_slice(&(4096u64).to_le_bytes()); // SizeOfStackCommit
        binary.extend_from_slice(&(1u64 << 20).to_le_bytes()); // SizeOfHeapReserve
        binary.extend_from_slice(&(4096u64).to_le_bytes()); // SizeOfHeapCommit
        binary.extend_from_slice(&0u32.to_le_bytes()); // LoaderFlags
        binary.extend_from_slice(&PE_NUM_DATA_DIRS.to_le_bytes()); // NumberOfRvaAndSizes

        // Data directories (all zeroed for minimal executable)
        for _ in 0..PE_NUM_DATA_DIRS {
            binary.extend_from_slice(&0u32.to_le_bytes()); // VirtualAddress
            binary.extend_from_slice(&0u32.to_le_bytes()); // Size
        }

        // ---------- Section Headers (40 bytes each) ----------
        let mut file_offset = size_of_headers;

        for name in &ordered {
            let info = layout.get(name).unwrap();
            let sec_data = merged.get(name.as_str());
            let raw_size = sec_data.map(|d| d.len() as u32).unwrap_or(0);
            let is_bss = name == ".bss";

            // Section name (8 bytes, zero-padded).
            let mut name_buf = [0u8; 8];
            let name_bytes = name.as_bytes();
            let copy_len = name_bytes.len().min(8);
            name_buf[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
            binary.extend_from_slice(&name_buf);

            let virtual_size = if is_bss { info.size as u32 } else { raw_size };
            let section_rva = (info.vaddr.wrapping_sub(PE_IMAGE_BASE)) as u32;

            binary.extend_from_slice(&virtual_size.to_le_bytes()); // VirtualSize
            binary.extend_from_slice(&section_rva.to_le_bytes()); // VirtualAddress

            let aligned_raw = if is_bss {
                0u32
            } else {
                align_up(raw_size as u64, PE_FILE_ALIGNMENT as u64) as u32
            };
            binary.extend_from_slice(&aligned_raw.to_le_bytes()); // SizeOfRawData
            let ptr_raw = if is_bss { 0u32 } else { file_offset };
            binary.extend_from_slice(&ptr_raw.to_le_bytes()); // PointerToRawData

            binary.extend_from_slice(&0u32.to_le_bytes()); // PointerToRelocations
            binary.extend_from_slice(&0u32.to_le_bytes()); // PointerToLinenumbers
            binary.extend_from_slice(&0u16.to_le_bytes()); // NumberOfRelocations
            binary.extend_from_slice(&0u16.to_le_bytes()); // NumberOfLinenumbers

            let chars = pe_section_characteristics(name);
            binary.extend_from_slice(&chars.to_le_bytes()); // Characteristics

            if !is_bss {
                file_offset += aligned_raw;
            }
        }

        // ---------- Pad to SizeOfHeaders ----------
        while binary.len() < size_of_headers as usize {
            binary.push(0);
        }

        // ---------- Section Data ----------
        for name in &ordered {
            if name == ".bss" {
                continue;
            }
            if let Some(data) = merged.get(name.as_str()) {
                binary.extend_from_slice(data);
                // Pad to file alignment.
                let target_len = align_up(binary.len() as u64, PE_FILE_ALIGNMENT as u64) as usize;
                binary.resize(target_len, 0);
            }
        }

        Ok(binary)
    }

    // ─────────────────────────────────────────────────────────────────────
    // Mach-O Emission  (macOS)
    // ─────────────────────────────────────────────────────────────────────

    fn emit_macho(
        &self,
        merged: &HashMap<String, Vec<u8>>,
        layout: &SectionLayout,
        resolved: &HashMap<String, u64>,
    ) -> Result<Vec<u8>> {
        let entry = self.resolve_entry(layout, resolved);
        let (cpu_type, cpu_subtype) = self.target.macho_cpu();
        let ordered = self.ordered_section_names();

        // Build load commands first so we can compute sizes.
        // We emit:
        //   1. LC_SEGMENT_64 "__PAGEZERO"  (no file data)
        //   2. LC_SEGMENT_64 "__TEXT"       (contains .text + .rodata)
        //   3. LC_SEGMENT_64 "__DATA"       (contains .data + .bss)
        //   4. LC_UNIXTHREAD               (sets entry point — simpler than LC_MAIN for static)
        //   5. LC_SYMTAB                   (empty symbol table, keeps format valid)

        // Group sections into Mach-O segments.
        let text_sections: Vec<&String> = ordered
            .iter()
            .filter(|n| *n == ".text" || *n == ".rodata")
            .collect();
        let data_sections: Vec<&String> = ordered
            .iter()
            .filter(|n| *n == ".data" || *n == ".bss")
            .collect();

        // Mach-O section header size = 80 bytes (64-bit).
        let macho_sect_size: usize = 80;

        // Compute load-command sizes.
        let lc_pagezero_size: u32 = 72; // segment_command_64 with 0 sections
        let lc_text_size: u32 = 72 + (text_sections.len() as u32) * macho_sect_size as u32;
        let lc_data_size: u32 = 72 + (data_sections.len() as u32) * macho_sect_size as u32;
        let lc_unixthread_size: u32 = if self.target.macho_cpu().0 == CPU_TYPE_X86_64 {
            // x86_64 thread state: cmd(4) + cmdsize(4) + flavor(4) + count(4) + 21 regs * 8
            4 + 4 + 4 + 4 + 21 * 8 // = 184
        } else {
            // arm64 thread state: cmd(4) + cmdsize(4) + flavor(4) + count(4) + 34 regs * 8
            4 + 4 + 4 + 4 + 34 * 8 // = 288
        };
        let lc_symtab_size: u32 = 24; // LC_SYMTAB command

        let mut ncmds: u32 = 0;
        let mut sizeofcmds: u32 = 0;

        // __PAGEZERO
        ncmds += 1;
        sizeofcmds += lc_pagezero_size;

        // __TEXT
        if !text_sections.is_empty() {
            ncmds += 1;
            sizeofcmds += lc_text_size;
        }

        // __DATA
        if !data_sections.is_empty() {
            ncmds += 1;
            sizeofcmds += lc_data_size;
        }

        // LC_UNIXTHREAD
        ncmds += 1;
        sizeofcmds += lc_unixthread_size;

        // LC_SYMTAB
        ncmds += 1;
        sizeofcmds += lc_symtab_size;

        let header_plus_lc = MACHO64_HEADER_SIZE + sizeofcmds as usize;
        let text_file_offset = align_up(header_plus_lc as u64, 16) as usize;

        let mut binary = Vec::new();

        // ---------- Mach-O Header (32 bytes) ----------
        binary.extend_from_slice(&MH_MAGIC_64.to_le_bytes());
        binary.extend_from_slice(&cpu_type.to_le_bytes());
        binary.extend_from_slice(&cpu_subtype.to_le_bytes());
        binary.extend_from_slice(&MH_EXECUTE.to_le_bytes());
        binary.extend_from_slice(&ncmds.to_le_bytes());
        binary.extend_from_slice(&sizeofcmds.to_le_bytes());
        let flags = MH_NOUNDEFS | MH_PIE;
        binary.extend_from_slice(&flags.to_le_bytes());
        binary.extend_from_slice(&0u32.to_le_bytes()); // reserved

        // ---------- LC_SEGMENT_64 __PAGEZERO ----------
        let pagezero_vmsize: u64 = 0x0000_0001_0000_0000; // 4 GiB
        self.write_macho_segment(
            &mut binary,
            "__PAGEZERO",
            0,
            pagezero_vmsize,
            0,
            0,
            0,
            0,
            0, // no sections
            lc_pagezero_size,
        );

        // ---------- LC_SEGMENT_64 __TEXT ----------
        if !text_sections.is_empty() {
            // Compute segment file range and VM range.
            let seg_vm_start = text_sections
                .iter()
                .filter_map(|n| layout.get(n.as_str()))
                .map(|s| s.vaddr)
                .min()
                .unwrap_or(0);
            let seg_vm_end = text_sections
                .iter()
                .filter_map(|n| layout.get(n.as_str()))
                .map(|s| s.vaddr + s.size)
                .max()
                .unwrap_or(0);
            let seg_vmsize = align_up(seg_vm_end - seg_vm_start, MACHO_PAGE_SIZE);

            // File layout: section data starts at text_file_offset.
            let mut seg_filesize: u64 = 0;
            for name in &text_sections {
                if let Some(d) = merged.get(name.as_str()) {
                    seg_filesize += d.len() as u64;
                    seg_filesize = align_up(seg_filesize, 16);
                }
            }

            self.write_macho_segment(
                &mut binary,
                "__TEXT",
                seg_vm_start,
                seg_vmsize,
                text_file_offset as u64,
                seg_filesize,
                VM_PROT_READ | VM_PROT_EXECUTE,
                VM_PROT_READ | VM_PROT_EXECUTE,
                text_sections.len() as u32,
                lc_text_size,
            );

            // Write section headers within the __TEXT segment.
            let mut sect_file_off = text_file_offset as u64;
            for sec_name in &text_sections {
                let info = layout.get(sec_name.as_str()).unwrap();
                let sec_len = merged
                    .get(sec_name.as_str())
                    .map(|d| d.len() as u64)
                    .unwrap_or(0);

                let macho_sec_name = macho_section_name(sec_name);
                let macho_seg_name = "__TEXT";
                let sec_type = if *sec_name == ".text" {
                    S_ATTR_PURE_INSTRUCTIONS | S_ATTR_SOME_INSTRUCTIONS | S_REGULAR
                } else {
                    S_REGULAR
                };

                self.write_macho_section(
                    &mut binary,
                    &macho_sec_name,
                    macho_seg_name,
                    info.vaddr,
                    sec_len,
                    sect_file_off as u32,
                    info.alignment.trailing_zeros(),
                    sec_type,
                );

                sect_file_off += sec_len;
                sect_file_off = align_up(sect_file_off, 16);
            }
        }

        // ---------- LC_SEGMENT_64 __DATA ----------
        if !data_sections.is_empty() {
            let seg_vm_start = data_sections
                .iter()
                .filter_map(|n| layout.get(n.as_str()))
                .map(|s| s.vaddr)
                .min()
                .unwrap_or(0);
            let seg_vm_end = data_sections
                .iter()
                .filter_map(|n| layout.get(n.as_str()))
                .map(|s| s.vaddr + s.size)
                .max()
                .unwrap_or(0);
            let seg_vmsize = align_up(seg_vm_end - seg_vm_start, MACHO_PAGE_SIZE);

            // Compute file offset for DATA segment.
            let mut data_file_offset = text_file_offset as u64;
            for name in &text_sections {
                if let Some(d) = merged.get(name.as_str()) {
                    data_file_offset += d.len() as u64;
                    data_file_offset = align_up(data_file_offset, 16);
                }
            }
            data_file_offset = align_up(data_file_offset, MACHO_PAGE_SIZE);

            let mut seg_filesize: u64 = 0;
            for name in &data_sections {
                if *name != ".bss" {
                    if let Some(d) = merged.get(name.as_str()) {
                        seg_filesize += d.len() as u64;
                        seg_filesize = align_up(seg_filesize, 8);
                    }
                }
            }

            self.write_macho_segment(
                &mut binary,
                "__DATA",
                seg_vm_start,
                seg_vmsize,
                data_file_offset,
                seg_filesize,
                VM_PROT_READ | VM_PROT_WRITE,
                VM_PROT_READ | VM_PROT_WRITE,
                data_sections.len() as u32,
                lc_data_size,
            );

            let mut sect_file_off = data_file_offset;
            for sec_name in &data_sections {
                let info = layout.get(sec_name.as_str()).unwrap();
                let is_bss = *sec_name == ".bss";
                let sec_len = if is_bss {
                    info.size
                } else {
                    merged
                        .get(sec_name.as_str())
                        .map(|d| d.len() as u64)
                        .unwrap_or(0)
                };

                let macho_sec_name = macho_section_name(sec_name);
                let sec_type = if is_bss { S_ZEROFILL } else { S_REGULAR };

                self.write_macho_section(
                    &mut binary,
                    &macho_sec_name,
                    "__DATA",
                    info.vaddr,
                    sec_len,
                    if is_bss { 0 } else { sect_file_off as u32 },
                    info.alignment.trailing_zeros(),
                    sec_type,
                );

                if !is_bss {
                    sect_file_off += sec_len;
                    sect_file_off = align_up(sect_file_off, 8);
                }
            }
        }

        // ---------- LC_UNIXTHREAD (entry point) ----------
        binary.extend_from_slice(&LC_UNIXTHREAD.to_le_bytes());
        binary.extend_from_slice(&lc_unixthread_size.to_le_bytes());
        if cpu_type == CPU_TYPE_X86_64 {
            // x86_64_THREAD_STATE = flavor 4, count 42 (21 uint64 values)
            binary.extend_from_slice(&4u32.to_le_bytes()); // flavor
            binary.extend_from_slice(&42u32.to_le_bytes()); // count (number of u32s = 21 regs * 2)
                                                            // 21 registers: rax..rflags, rip, ... (rip is register index 16)
            for i in 0u32..21 {
                if i == 16 {
                    // RIP — set to entry point
                    binary.extend_from_slice(&entry.to_le_bytes());
                } else {
                    binary.extend_from_slice(&0u64.to_le_bytes());
                }
            }
        } else {
            // ARM_THREAD_STATE64 = flavor 6, count 68 (34 uint64 values)
            binary.extend_from_slice(&6u32.to_le_bytes()); // flavor
            binary.extend_from_slice(&68u32.to_le_bytes()); // count
                                                            // 34 registers: x0-x28, fp, lr, sp, pc, cpsr(+pad)
                                                            // PC is register index 32
            for i in 0u32..34 {
                if i == 32 {
                    binary.extend_from_slice(&entry.to_le_bytes());
                } else {
                    binary.extend_from_slice(&0u64.to_le_bytes());
                }
            }
        }

        // ---------- LC_SYMTAB (empty) ----------
        binary.extend_from_slice(&LC_SYMTAB.to_le_bytes());
        binary.extend_from_slice(&lc_symtab_size.to_le_bytes());
        binary.extend_from_slice(&0u32.to_le_bytes()); // symoff
        binary.extend_from_slice(&0u32.to_le_bytes()); // nsyms
        binary.extend_from_slice(&0u32.to_le_bytes()); // stroff
        binary.extend_from_slice(&0u32.to_le_bytes()); // strsize

        // ---------- Pad to text_file_offset ----------
        while binary.len() < text_file_offset {
            binary.push(0);
        }

        // ---------- Section Data (__TEXT) ----------
        for name in &text_sections {
            if let Some(data) = merged.get(name.as_str()) {
                binary.extend_from_slice(data);
                while binary.len() % 16 != 0 {
                    binary.push(0);
                }
            }
        }

        // ---------- Section Data (__DATA) ----------
        if !data_sections.is_empty() {
            // Pad to page alignment before __DATA segment.
            let page = MACHO_PAGE_SIZE as usize;
            while binary.len() % page != 0 {
                binary.push(0);
            }
            for name in &data_sections {
                if *name == ".bss" {
                    continue;
                }
                if let Some(data) = merged.get(name.as_str()) {
                    binary.extend_from_slice(data);
                    while binary.len() % 8 != 0 {
                        binary.push(0);
                    }
                }
            }
        }

        Ok(binary)
    }

    // ── Mach-O helpers ───────────────────────────────────────────────────

    /// Write a `segment_command_64` (72 bytes, without sections).
    fn write_macho_segment(
        &self,
        buf: &mut Vec<u8>,
        segname: &str,
        vmaddr: u64,
        vmsize: u64,
        fileoff: u64,
        filesize: u64,
        maxprot: u32,
        initprot: u32,
        nsects: u32,
        cmdsize: u32,
    ) {
        buf.extend_from_slice(&LC_SEGMENT_64.to_le_bytes()); // cmd
        buf.extend_from_slice(&cmdsize.to_le_bytes()); // cmdsize

        // segname (16 bytes, zero-padded)
        let mut name_buf = [0u8; 16];
        let bytes = segname.as_bytes();
        let len = bytes.len().min(16);
        name_buf[..len].copy_from_slice(&bytes[..len]);
        buf.extend_from_slice(&name_buf);

        buf.extend_from_slice(&vmaddr.to_le_bytes());
        buf.extend_from_slice(&vmsize.to_le_bytes());
        buf.extend_from_slice(&fileoff.to_le_bytes());
        buf.extend_from_slice(&filesize.to_le_bytes());
        buf.extend_from_slice(&maxprot.to_le_bytes());
        buf.extend_from_slice(&initprot.to_le_bytes());
        buf.extend_from_slice(&nsects.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes()); // flags
    }

    /// Write a Mach-O 64-bit section header (80 bytes).
    fn write_macho_section(
        &self,
        buf: &mut Vec<u8>,
        sectname: &str,
        segname: &str,
        addr: u64,
        size: u64,
        offset: u32,
        align_log2: u32,
        flags: u32,
    ) {
        // sectname (16 bytes)
        let mut sn = [0u8; 16];
        let sb = sectname.as_bytes();
        let sl = sb.len().min(16);
        sn[..sl].copy_from_slice(&sb[..sl]);
        buf.extend_from_slice(&sn);

        // segname (16 bytes)
        let mut sg = [0u8; 16];
        let sgb = segname.as_bytes();
        let sgl = sgb.len().min(16);
        sg[..sgl].copy_from_slice(&sgb[..sgl]);
        buf.extend_from_slice(&sg);

        buf.extend_from_slice(&addr.to_le_bytes()); // addr
        buf.extend_from_slice(&size.to_le_bytes()); // size
        buf.extend_from_slice(&offset.to_le_bytes()); // offset
        buf.extend_from_slice(&align_log2.to_le_bytes()); // align (log2)
        buf.extend_from_slice(&0u32.to_le_bytes()); // reloff
        buf.extend_from_slice(&0u32.to_le_bytes()); // nreloc
        buf.extend_from_slice(&flags.to_le_bytes()); // flags
        buf.extend_from_slice(&0u32.to_le_bytes()); // reserved1
        buf.extend_from_slice(&0u32.to_le_bytes()); // reserved2
        buf.extend_from_slice(&0u32.to_le_bytes()); // reserved3 (64-bit only)
    }

    // ── Section ordering ─────────────────────────────────────────────────

    /// Return section names in canonical order: .text, .rodata, .data, .bss, others.
    fn ordered_section_names(&self) -> Vec<String> {
        let canonical_order = [".text", ".rodata", ".data", ".bss"];
        let mut result: Vec<String> = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // First add canonical sections in order.
        for &name in &canonical_order {
            if self.sections.iter().any(|s| s.name == name) {
                result.push(name.to_string());
                seen.insert(name.to_string());
            }
        }

        // Then any non-canonical sections.
        for section in &self.sections {
            if !seen.contains(&section.name) {
                result.push(section.name.clone());
                seen.insert(section.name.clone());
            }
        }

        result
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Section Layout
// ─────────────────────────────────────────────────────────────────────────────

/// Information about a section's placement in the output.
#[derive(Debug, Clone)]
struct SectionInfo {
    vaddr: u64,
    size: u64,
    alignment: u64,
}

/// Maps section names to their virtual addresses and sizes.
#[derive(Debug, Clone)]
struct SectionLayout {
    sections: HashMap<String, SectionInfo>,
    order: Vec<String>,
}

impl SectionLayout {
    fn new() -> Self {
        SectionLayout {
            sections: HashMap::new(),
            order: Vec::new(),
        }
    }

    fn add(&mut self, name: String, vaddr: u64, size: u64, alignment: u64) {
        self.sections.insert(
            name.clone(),
            SectionInfo {
                vaddr,
                size,
                alignment,
            },
        );
        self.order.push(name);
    }

    fn get(&self, name: &str) -> Option<&SectionInfo> {
        self.sections.get(name)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Align `value` up to the next multiple of `align` (must be power of two).
fn align_up(value: u64, align: u64) -> u64 {
    if align == 0 {
        return value;
    }
    (value + align - 1) & !(align - 1)
}

/// Map an ELF section name to program-header flags.
fn elf_section_pflags(name: &str) -> u32 {
    match name {
        ".text" => PF_R | PF_X,
        ".rodata" => PF_R,
        ".data" => PF_R | PF_W,
        ".bss" => PF_R | PF_W,
        _ => PF_R,
    }
}

/// Map an ELF section name to section-header flags.
fn elf_section_shflags(name: &str) -> u64 {
    match name {
        ".text" => SHF_ALLOC | SHF_EXECINSTR,
        ".rodata" => SHF_ALLOC,
        ".data" => SHF_ALLOC | SHF_WRITE,
        ".bss" => SHF_ALLOC | SHF_WRITE,
        _ => SHF_ALLOC,
    }
}

/// Map a PE section name to section characteristics.
fn pe_section_characteristics(name: &str) -> u32 {
    match name {
        ".text" => IMAGE_SCN_CNT_CODE | IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ,
        ".rodata" => IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ,
        ".data" => IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE,
        ".bss" => IMAGE_SCN_CNT_UNINITIALIZED_DATA | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE,
        _ => IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ,
    }
}

/// Map a Unix-style section name (".text") to a Mach-O section name ("__text").
fn macho_section_name(name: &str) -> String {
    match name {
        ".text" => "__text".to_string(),
        ".rodata" => "__const".to_string(),
        ".data" => "__data".to_string(),
        ".bss" => "__bss".to_string(),
        other => {
            let trimmed = other.trim_start_matches('.');
            format!("__{}", trimmed)
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Convenience
// ─────────────────────────────────────────────────────────────────────────────

/// One-shot link: create a minimal executable from raw code bytes for the host.
pub fn link_code_for_host(code: Vec<u8>, entry_name: &str) -> Result<Vec<u8>> {
    let mut linker = Linker::for_host()?;
    linker.add_text(code);
    linker.set_entry_point(entry_name);
    linker.add_symbol(LinkerSymbol {
        name: entry_name.to_string(),
        offset: 0,
        size: 0,
        section: Some(".text".to_string()),
        binding: SymbolBinding::Global,
        kind: SymbolKind::Function,
    });
    linker.link()
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: minimal x86-64 code that does `xor edi, edi; mov eax, 60; syscall`
    // (exit(0) on Linux).
    fn exit0_code() -> Vec<u8> {
        vec![
            0x31, 0xFF, // xor edi, edi
            0xB8, 0x3C, 0x00, 0x00, 0x00, // mov eax, 60
            0x0F, 0x05, // syscall
        ]
    }

    // ── Target triple parsing ────────────────────────────────────────────

    #[test]
    fn test_parse_triple_linux() {
        let t = TargetPlatform::from_triple("x86_64-unknown-linux-gnu").unwrap();
        assert_eq!(t, TargetPlatform::LinuxX86_64);
    }

    #[test]
    fn test_parse_triple_windows() {
        let t = TargetPlatform::from_triple("x86_64-pc-windows-msvc").unwrap();
        assert_eq!(t, TargetPlatform::WindowsX86_64);
    }

    #[test]
    fn test_parse_triple_macos_arm() {
        let t = TargetPlatform::from_triple("aarch64-apple-darwin").unwrap();
        assert_eq!(t, TargetPlatform::MacOSAarch64);
    }

    #[test]
    fn test_parse_triple_invalid() {
        assert!(TargetPlatform::from_triple("mips-unknown-freebsd").is_err());
    }

    // ── ELF64 output ────────────────────────────────────────────────────

    #[test]
    fn test_elf64_magic() {
        let mut linker = Linker::new(TargetPlatform::LinuxX86_64);
        linker.add_text(exit0_code());
        linker.add_symbol(LinkerSymbol {
            name: "_start".into(),
            offset: 0,
            size: 9,
            section: Some(".text".into()),
            binding: SymbolBinding::Global,
            kind: SymbolKind::Function,
        });

        let binary = linker.link().unwrap();
        // ELF magic
        assert_eq!(&binary[0..4], &ELF_MAGIC);
        // 64-bit
        assert_eq!(binary[4], EI_CLASS_64);
        // Little-endian
        assert_eq!(binary[5], EI_DATA_LSB);
    }

    #[test]
    fn test_elf64_entry_point() {
        let mut linker = Linker::new(TargetPlatform::LinuxX86_64);
        linker.add_text(exit0_code());
        linker.add_symbol(LinkerSymbol {
            name: "_start".into(),
            offset: 0,
            size: 9,
            section: Some(".text".into()),
            binding: SymbolBinding::Global,
            kind: SymbolKind::Function,
        });

        let binary = linker.link().unwrap();
        // e_entry at offset 24 (8 bytes, little-endian)
        let entry = u64::from_le_bytes(binary[24..32].try_into().unwrap());
        // Should be >= ELF_VADDR_BASE
        assert!(
            entry >= ELF_VADDR_BASE,
            "entry {:#x} < base {:#x}",
            entry,
            ELF_VADDR_BASE
        );
    }

    #[test]
    fn test_elf64_executable_type() {
        let mut linker = Linker::new(TargetPlatform::LinuxX86_64);
        linker.add_text(exit0_code());

        let binary = linker.link().unwrap();
        let e_type = u16::from_le_bytes(binary[16..18].try_into().unwrap());
        assert_eq!(e_type, ET_EXEC);
    }

    #[test]
    fn test_elf64_pie() {
        let mut linker = Linker::new(TargetPlatform::LinuxX86_64);
        linker.set_pie(true);
        linker.add_text(exit0_code());

        let binary = linker.link().unwrap();
        let e_type = u16::from_le_bytes(binary[16..18].try_into().unwrap());
        assert_eq!(e_type, ET_DYN);
    }

    #[test]
    fn test_elf64_machine_x86_64() {
        let mut linker = Linker::new(TargetPlatform::LinuxX86_64);
        linker.add_text(vec![0xC3]); // ret

        let binary = linker.link().unwrap();
        let e_machine = u16::from_le_bytes(binary[18..20].try_into().unwrap());
        assert_eq!(e_machine, EM_X86_64);
    }

    #[test]
    fn test_elf64_machine_aarch64() {
        let mut linker = Linker::new(TargetPlatform::LinuxAarch64);
        linker.add_text(vec![0xC0, 0x03, 0x5F, 0xD6]); // ret (arm64)

        let binary = linker.link().unwrap();
        let e_machine = u16::from_le_bytes(binary[18..20].try_into().unwrap());
        assert_eq!(e_machine, EM_AARCH64);
    }

    #[test]
    fn test_elf64_with_all_sections() {
        let mut linker = Linker::new(TargetPlatform::LinuxX86_64);
        linker.add_text(exit0_code());
        linker.add_data(vec![0x01, 0x02, 0x03, 0x04]);
        linker.add_rodata(b"Hello, world!\0".to_vec());
        linker.add_bss(256);

        let binary = linker.link().unwrap();
        assert_eq!(&binary[0..4], &ELF_MAGIC);
        // Should contain section data.
        assert!(binary.len() > 64 + 56); // header + at least one phdr
    }

    // ── PE output ───────────────────────────────────────────────────────

    #[test]
    fn test_pe_dos_magic() {
        let mut linker = Linker::new(TargetPlatform::WindowsX86_64);
        linker.add_text(vec![0xC3]); // ret

        let binary = linker.link().unwrap();
        assert_eq!(&binary[0..2], &PE_DOS_MAGIC);
    }

    #[test]
    fn test_pe_signature() {
        let mut linker = Linker::new(TargetPlatform::WindowsX86_64);
        linker.add_text(vec![0xC3]);

        let binary = linker.link().unwrap();
        // PE signature at offset 64
        assert_eq!(&binary[64..68], &PE_SIGNATURE);
    }

    #[test]
    fn test_pe_machine() {
        let mut linker = Linker::new(TargetPlatform::WindowsX86_64);
        linker.add_text(vec![0xC3]);

        let binary = linker.link().unwrap();
        let machine = u16::from_le_bytes(binary[68..70].try_into().unwrap());
        assert_eq!(machine, IMAGE_FILE_MACHINE_AMD64);
    }

    #[test]
    fn test_pe_optional_header_magic() {
        let mut linker = Linker::new(TargetPlatform::WindowsX86_64);
        linker.add_text(vec![0xC3]);

        let binary = linker.link().unwrap();
        // Optional header starts at offset 64+4+20 = 88
        let magic = u16::from_le_bytes(binary[88..90].try_into().unwrap());
        assert_eq!(magic, PE32_PLUS_MAGIC);
    }

    #[test]
    fn test_pe_with_data_sections() {
        let mut linker = Linker::new(TargetPlatform::WindowsX86_64);
        linker.add_text(vec![0x55, 0x48, 0x89, 0xE5, 0xC3]); // push rbp; mov rbp,rsp; ret
        linker.add_data(vec![0xDE, 0xAD, 0xBE, 0xEF]);
        linker.add_rodata(b"Omni\0".to_vec());
        linker.add_bss(128);

        let binary = linker.link().unwrap();
        assert_eq!(&binary[0..2], &PE_DOS_MAGIC);
        assert!(binary.len() > PE_DOS_HEADER_SIZE);
    }

    // ── Mach-O output ───────────────────────────────────────────────────

    #[test]
    fn test_macho_magic() {
        let mut linker = Linker::new(TargetPlatform::MacOSX86_64);
        linker.add_text(vec![0xC3]);

        let binary = linker.link().unwrap();
        let magic = u32::from_le_bytes(binary[0..4].try_into().unwrap());
        assert_eq!(magic, MH_MAGIC_64);
    }

    #[test]
    fn test_macho_cpu_type_x86() {
        let mut linker = Linker::new(TargetPlatform::MacOSX86_64);
        linker.add_text(vec![0xC3]);

        let binary = linker.link().unwrap();
        let cpu = u32::from_le_bytes(binary[4..8].try_into().unwrap());
        assert_eq!(cpu, CPU_TYPE_X86_64);
    }

    #[test]
    fn test_macho_cpu_type_arm64() {
        let mut linker = Linker::new(TargetPlatform::MacOSAarch64);
        linker.add_text(vec![0xC0, 0x03, 0x5F, 0xD6]); // ret

        let binary = linker.link().unwrap();
        let cpu = u32::from_le_bytes(binary[4..8].try_into().unwrap());
        assert_eq!(cpu, CPU_TYPE_ARM64);
    }

    #[test]
    fn test_macho_file_type_execute() {
        let mut linker = Linker::new(TargetPlatform::MacOSX86_64);
        linker.add_text(vec![0xC3]);

        let binary = linker.link().unwrap();
        let filetype = u32::from_le_bytes(binary[12..16].try_into().unwrap());
        assert_eq!(filetype, MH_EXECUTE);
    }

    #[test]
    fn test_macho_with_data() {
        let mut linker = Linker::new(TargetPlatform::MacOSAarch64);
        linker.add_text(vec![0xC0, 0x03, 0x5F, 0xD6]);
        linker.add_data(vec![0x42; 16]);
        linker.add_rodata(b"Helios\0".to_vec());
        linker.add_bss(64);

        let binary = linker.link().unwrap();
        let magic = u32::from_le_bytes(binary[0..4].try_into().unwrap());
        assert_eq!(magic, MH_MAGIC_64);
    }

    // ── Symbol resolution ───────────────────────────────────────────────

    #[test]
    fn test_symbol_resolution() {
        let mut linker = Linker::new(TargetPlatform::LinuxX86_64);
        linker.add_text(exit0_code());
        linker.add_symbol(LinkerSymbol {
            name: "_start".into(),
            offset: 0,
            size: 9,
            section: Some(".text".into()),
            binding: SymbolBinding::Global,
            kind: SymbolKind::Function,
        });
        linker.add_symbol(LinkerSymbol {
            name: "helper".into(),
            offset: 4,
            size: 5,
            section: Some(".text".into()),
            binding: SymbolBinding::Local,
            kind: SymbolKind::Function,
        });

        let layout = linker.compute_layout().unwrap();
        let resolved = linker.resolve_symbols(&layout).unwrap();
        assert!(resolved.contains_key("_start"));
        assert!(resolved.contains_key("helper"));
        let start_addr = resolved["_start"];
        let helper_addr = resolved["helper"];
        assert_eq!(helper_addr - start_addr, 4);
    }

    #[test]
    fn test_duplicate_global_symbol() {
        let mut linker = Linker::new(TargetPlatform::LinuxX86_64);
        linker.add_text(vec![0xC3]);
        linker.add_symbol(LinkerSymbol {
            name: "dup".into(),
            offset: 0,
            size: 1,
            section: Some(".text".into()),
            binding: SymbolBinding::Global,
            kind: SymbolKind::Function,
        });
        linker.add_symbol(LinkerSymbol {
            name: "dup".into(),
            offset: 0,
            size: 1,
            section: Some(".text".into()),
            binding: SymbolBinding::Global,
            kind: SymbolKind::Function,
        });

        let layout = linker.compute_layout().unwrap();
        let result = linker.resolve_symbols(&layout);
        assert!(result.is_err());
    }

    // ── Relocations ─────────────────────────────────────────────────────

    #[test]
    fn test_abs64_relocation() {
        let mut linker = Linker::new(TargetPlatform::LinuxX86_64);
        // 16 bytes of .text — bytes 0..8 are a placeholder for an absolute address.
        linker.add_text(vec![0; 16]);
        linker.add_data(vec![0xAA; 4]);

        linker.add_symbol(LinkerSymbol {
            name: "my_data".into(),
            offset: 0,
            size: 4,
            section: Some(".data".into()),
            binding: SymbolBinding::Global,
            kind: SymbolKind::Object,
        });

        linker.add_relocation(LinkerRelocation {
            section: ".text".into(),
            offset: 0,
            symbol: "my_data".into(),
            addend: 0,
            reloc_type: RelocationType::Abs64,
        });

        let binary = linker.link().unwrap();
        assert!(!binary.is_empty());
    }

    #[test]
    fn test_pcrel32_relocation() {
        let mut linker = Linker::new(TargetPlatform::LinuxX86_64);
        // 16 bytes code, with a PC-relative reference at offset 2.
        let mut code = vec![0x90; 16]; // NOPs
        code[0] = 0xE8; // CALL rel32 placeholder
        linker.add_text(code);

        linker.add_symbol(LinkerSymbol {
            name: "callee".into(),
            offset: 10,
            size: 6,
            section: Some(".text".into()),
            binding: SymbolBinding::Global,
            kind: SymbolKind::Function,
        });

        linker.add_relocation(LinkerRelocation {
            section: ".text".into(),
            offset: 1,
            symbol: "callee".into(),
            addend: -4, // CALL instruction: target = rip + rel32, rip = site + 4
            reloc_type: RelocationType::PcRel32,
        });

        let binary = linker.link().unwrap();
        assert!(!binary.is_empty());
    }

    #[test]
    fn test_undefined_symbol_error() {
        let mut linker = Linker::new(TargetPlatform::LinuxX86_64);
        linker.add_text(vec![0; 8]);
        linker.add_relocation(LinkerRelocation {
            section: ".text".into(),
            offset: 0,
            symbol: "nonexistent".into(),
            addend: 0,
            reloc_type: RelocationType::Abs64,
        });

        let result = linker.link();
        assert!(result.is_err());
    }

    // ── Convenience API ─────────────────────────────────────────────────

    #[test]
    fn test_link_code_for_host() {
        let binary = link_code_for_host(vec![0xC3], "_start").unwrap();
        assert!(!binary.is_empty());

        // Verify format based on host OS.
        #[cfg(target_os = "linux")]
        assert_eq!(&binary[0..4], &ELF_MAGIC);

        #[cfg(target_os = "windows")]
        assert_eq!(&binary[0..2], &PE_DOS_MAGIC);

        #[cfg(target_os = "macos")]
        {
            let magic = u32::from_le_bytes(binary[0..4].try_into().unwrap());
            assert_eq!(magic, MH_MAGIC_64);
        }
    }

    // ── Align helper ────────────────────────────────────────────────────

    #[test]
    fn test_align_up() {
        assert_eq!(align_up(0, 16), 0);
        assert_eq!(align_up(1, 16), 16);
        assert_eq!(align_up(16, 16), 16);
        assert_eq!(align_up(17, 16), 32);
        assert_eq!(align_up(4096, 4096), 4096);
        assert_eq!(align_up(4097, 4096), 8192);
    }

    // ── Section ordering ────────────────────────────────────────────────

    #[test]
    fn test_section_ordering() {
        let mut linker = Linker::new(TargetPlatform::LinuxX86_64);
        linker.add_bss(16);
        linker.add_rodata(vec![0x42]);
        linker.add_text(vec![0xC3]);
        linker.add_data(vec![0x01]);

        let ordered = linker.ordered_section_names();
        assert_eq!(ordered, vec![".text", ".rodata", ".data", ".bss"]);
    }
}
