//! DWARF v5 Debug Info Emitter
//!
//! Maps Omni source locations to machine code addresses for DAP (Debug Adapter Protocol) support.
//! Generates .debug_info, .debug_abbrev, .debug_line, and .debug_str sections.

use crate::ir::{IrFunction, IrType};
use log::debug;

/// DWARF constants
const DW_TAG_COMPILE_UNIT: u8 = 0x11;
const DW_TAG_SUBPROGRAM: u8 = 0x2e;
const DW_TAG_VARIABLE: u8 = 0x34;
const DW_TAG_FORMAL_PARAMETER: u8 = 0x05;
const DW_TAG_BASE_TYPE: u8 = 0x24;
const DW_TAG_POINTER_TYPE: u8 = 0x0f;
const DW_TAG_STRUCTURE_TYPE: u8 = 0x13;
const DW_TAG_MEMBER: u8 = 0x0d;
const DW_TAG_ARRAY_TYPE: u8 = 0x01;

const DW_AT_NAME: u16 = 0x03;
const DW_AT_PRODUCER: u16 = 0x25;
const DW_AT_LANGUAGE: u16 = 0x13;
const DW_AT_LOW_PC: u16 = 0x11;
const DW_AT_HIGH_PC: u16 = 0x12;
const DW_AT_STMT_LIST: u16 = 0x10;
const DW_AT_TYPE: u16 = 0x49;
const DW_AT_BYTE_SIZE: u16 = 0x0b;
const DW_AT_ENCODING: u16 = 0x3e;
const DW_AT_LOCATION: u16 = 0x02;

const DW_FORM_ADDR: u8 = 0x01;
const DW_FORM_STRING: u8 = 0x08;
const DW_FORM_DATA1: u8 = 0x0b;
const DW_FORM_DATA2: u8 = 0x05;
const DW_FORM_DATA4: u8 = 0x06;
const DW_FORM_SEC_OFFSET: u8 = 0x17;
const DW_FORM_REF4: u8 = 0x13;

const DW_ATE_SIGNED: u8 = 0x05;
const DW_ATE_UNSIGNED: u8 = 0x07;
const DW_ATE_FLOAT: u8 = 0x04;
const DW_ATE_BOOLEAN: u8 = 0x02;

const DW_LANG_RUST: u16 = 0x001c; // Closest match for Omni
const DWARF_VERSION: u16 = 5;

pub struct DwarfEmitter {
    string_pool: Vec<String>,
    abbrev_code: u32,
}

impl DwarfEmitter {
    pub fn new() -> Self {
        Self {
            string_pool: Vec::new(),
            abbrev_code: 1,
        }
    }

    fn next_abbrev(&mut self) -> u32 {
        let code = self.abbrev_code;
        self.abbrev_code += 1;
        code
    }

    fn add_string(&mut self, s: &str) -> u32 {
        if let Some(idx) = self.string_pool.iter().position(|x| x == s) {
            return idx as u32;
        }
        let idx = self.string_pool.len() as u32;
        self.string_pool.push(s.to_string());
        idx
    }

    pub fn emit_debug_info(module_name: &str, funcs: &[IrFunction]) -> Vec<u8> {
        let mut emitter = Self::new();
        let mut buffer = Vec::new();

        debug!(
            "DWARF: Emitting debug info for module '{}' ({} functions)",
            module_name,
            funcs.len()
        );

        // 1. Emit .debug_abbrev section
        let abbrev_section = emitter.emit_abbrev_table(funcs);

        // 2. Emit .debug_info section
        let info_section = emitter.emit_info_section(module_name, funcs);

        // 3. Emit .debug_line section
        let line_section = emitter.emit_line_section(funcs);

        // 4. Emit .debug_str section
        let str_section = emitter.emit_string_section();

        // Combine sections with headers
        // Section: .debug_abbrev
        buffer.extend_from_slice(b".debug_abbrev\0");
        Self::write_u32(&mut buffer, abbrev_section.len() as u32);
        buffer.extend_from_slice(&abbrev_section);

        // Section: .debug_info
        buffer.extend_from_slice(b".debug_info\0");
        Self::write_u32(&mut buffer, info_section.len() as u32);
        buffer.extend_from_slice(&info_section);

        // Section: .debug_line
        buffer.extend_from_slice(b".debug_line\0");
        Self::write_u32(&mut buffer, line_section.len() as u32);
        buffer.extend_from_slice(&line_section);

        // Section: .debug_str
        buffer.extend_from_slice(b".debug_str\0");
        Self::write_u32(&mut buffer, str_section.len() as u32);
        buffer.extend_from_slice(&str_section);

        debug!("DWARF: Generated {} bytes of debug info", buffer.len());
        buffer
    }

    fn emit_abbrev_table(&mut self, funcs: &[IrFunction]) -> Vec<u8> {
        let mut buf = Vec::new();

        // Abbrev 1: DW_TAG_compile_unit
        let cu_code = self.next_abbrev();
        Self::write_uleb128(&mut buf, cu_code as u64);
        buf.push(DW_TAG_COMPILE_UNIT);
        buf.push(1); // has children
        Self::write_uleb128(&mut buf, DW_AT_PRODUCER as u64);
        buf.push(DW_FORM_STRING);
        Self::write_uleb128(&mut buf, DW_AT_LANGUAGE as u64);
        buf.push(DW_FORM_DATA2);
        Self::write_uleb128(&mut buf, DW_AT_NAME as u64);
        buf.push(DW_FORM_STRING);
        Self::write_uleb128(&mut buf, DW_AT_STMT_LIST as u64);
        buf.push(DW_FORM_SEC_OFFSET);
        buf.push(0);
        buf.push(0); // End of attributes

        // Abbrev 2: DW_TAG_subprogram
        let sub_code = self.next_abbrev();
        Self::write_uleb128(&mut buf, sub_code as u64);
        buf.push(DW_TAG_SUBPROGRAM);
        buf.push(1); // has children (parameters)
        Self::write_uleb128(&mut buf, DW_AT_NAME as u64);
        buf.push(DW_FORM_STRING);
        Self::write_uleb128(&mut buf, DW_AT_LOW_PC as u64);
        buf.push(DW_FORM_ADDR);
        Self::write_uleb128(&mut buf, DW_AT_HIGH_PC as u64);
        buf.push(DW_FORM_DATA4);
        buf.push(0);
        buf.push(0);

        // Abbrev 3: DW_TAG_formal_parameter
        let param_code = self.next_abbrev();
        Self::write_uleb128(&mut buf, param_code as u64);
        buf.push(DW_TAG_FORMAL_PARAMETER);
        buf.push(0); // no children
        Self::write_uleb128(&mut buf, DW_AT_NAME as u64);
        buf.push(DW_FORM_STRING);
        Self::write_uleb128(&mut buf, DW_AT_TYPE as u64);
        buf.push(DW_FORM_REF4);
        buf.push(0);
        buf.push(0);

        // Abbrev 4: DW_TAG_variable
        let var_code = self.next_abbrev();
        Self::write_uleb128(&mut buf, var_code as u64);
        buf.push(DW_TAG_VARIABLE);
        buf.push(0); // no children
        Self::write_uleb128(&mut buf, DW_AT_NAME as u64);
        buf.push(DW_FORM_STRING);
        Self::write_uleb128(&mut buf, DW_AT_TYPE as u64);
        buf.push(DW_FORM_REF4);
        Self::write_uleb128(&mut buf, DW_AT_LOCATION as u64);
        buf.push(DW_FORM_DATA4);
        buf.push(0);
        buf.push(0);

        // Abbrev 5: DW_TAG_base_type
        let base_code = self.next_abbrev();
        Self::write_uleb128(&mut buf, base_code as u64);
        buf.push(DW_TAG_BASE_TYPE);
        buf.push(0); // no children
        Self::write_uleb128(&mut buf, DW_AT_NAME as u64);
        buf.push(DW_FORM_STRING);
        Self::write_uleb128(&mut buf, DW_AT_BYTE_SIZE as u64);
        buf.push(DW_FORM_DATA1);
        Self::write_uleb128(&mut buf, DW_AT_ENCODING as u64);
        buf.push(DW_FORM_DATA1);
        buf.push(0);
        buf.push(0);

        // End of abbreviation table
        buf.push(0);

        buf
    }

    fn emit_info_section(&mut self, module_name: &str, funcs: &[IrFunction]) -> Vec<u8> {
        let mut buf = Vec::new();

        // Compilation unit header
        Self::write_u32(&mut buf, 0); // Placeholder for unit_length
        Self::write_u16(&mut buf, DWARF_VERSION);
        buf.push(1); // DW_UT_compile
        buf.push(8); // address_size
        Self::write_u32(&mut buf, 0); // debug_abbrev_offset

        // CU DIE (abbrev 1)
        Self::write_uleb128(&mut buf, 1);
        Self::write_string(&mut buf, "Omni Compiler v4.0");
        Self::write_u16(&mut buf, DW_LANG_RUST);
        Self::write_string(&mut buf, module_name);
        Self::write_u32(&mut buf, 0); // stmt_list offset

        // Base types
        let base_type_offset = buf.len() as u32;
        // i64
        Self::write_uleb128(&mut buf, 5); // abbrev 5 = base_type
        Self::write_string(&mut buf, "i64");
        buf.push(8); // byte_size
        buf.push(DW_ATE_SIGNED);

        // f64
        Self::write_uleb128(&mut buf, 5);
        Self::write_string(&mut buf, "f64");
        buf.push(8);
        buf.push(DW_ATE_FLOAT);

        // bool
        Self::write_uleb128(&mut buf, 5);
        Self::write_string(&mut buf, "bool");
        buf.push(1);
        buf.push(DW_ATE_BOOLEAN);

        // Function DIEs
        let mut func_addr = 0u64;
        for func in funcs {
            // Subprogram DIE (abbrev 2)
            Self::write_uleb128(&mut buf, 2);
            Self::write_string(&mut buf, &func.name);
            Self::write_u64(&mut buf, func_addr); // low_pc
            let func_size = func
                .blocks
                .iter()
                .map(|b| b.instructions.len() as u32 * 4 + 4)
                .sum::<u32>();
            Self::write_u32(&mut buf, func_size); // high_pc (offset from low_pc)

            // Parameters (abbrev 3)
            for (name, ty) in &func.params {
                Self::write_uleb128(&mut buf, 3);
                Self::write_string(&mut buf, name);
                Self::write_u32(&mut buf, base_type_offset); // type reference
            }

            // Local variables (abbrev 4)
            for (name, ty) in &func.locals {
                Self::write_uleb128(&mut buf, 4);
                Self::write_string(&mut buf, name);
                Self::write_u32(&mut buf, base_type_offset); // type reference
                Self::write_u32(&mut buf, 0); // location expression
            }

            buf.push(0); // end of children for subprogram
            func_addr += func_size as u64;
        }

        buf.push(0); // end of children for CU

        // Patch unit_length
        let unit_length = (buf.len() - 4) as u32;
        buf[0..4].copy_from_slice(&unit_length.to_le_bytes());

        buf
    }

    fn emit_line_section(&self, funcs: &[IrFunction]) -> Vec<u8> {
        let mut buf = Vec::new();

        // Line number program header (DWARF v5)
        Self::write_u32(&mut buf, 0); // Placeholder for total length
        Self::write_u16(&mut buf, DWARF_VERSION);
        buf.push(8); // address_size
        buf.push(0); // segment_selector_size
        Self::write_u32(&mut buf, 0); // Placeholder for header_length

        let header_start = buf.len();

        // Standard opcode lengths
        buf.push(1); // minimum_instruction_length
        buf.push(1); // maximum_operations_per_instruction
        buf.push(1); // default_is_stmt
        buf.push((-5i8) as u8); // line_base
        buf.push(14); // line_range
        buf.push(13); // opcode_base
                      // Standard opcode arg counts
        buf.extend_from_slice(&[0, 1, 1, 1, 1, 0, 0, 0, 1, 0, 0, 1]);

        // Directory table (DWARF v5 format)
        buf.push(1); // directory entry format count
        Self::write_uleb128(&mut buf, DW_AT_NAME as u64); // content type
        Self::write_uleb128(&mut buf, DW_FORM_STRING as u64); // form
        Self::write_uleb128(&mut buf, 1); // directories count
        Self::write_string(&mut buf, "."); // current directory

        // File table (DWARF v5 format)
        buf.push(1); // file entry format count
        Self::write_uleb128(&mut buf, DW_AT_NAME as u64);
        Self::write_uleb128(&mut buf, DW_FORM_STRING as u64);
        Self::write_uleb128(&mut buf, 1); // files count
        Self::write_string(&mut buf, "main.omni");

        let header_length = (buf.len() - header_start) as u32;

        // Line number program body
        let mut address: u64 = 0;
        let mut line: u32 = 1;

        for func in funcs {
            // Set address to function start
            buf.push(0); // extended opcode
            buf.push(9); // size
            buf.push(2); // DW_LNE_set_address
            Self::write_u64(&mut buf, address);

            // Advance line for each block
            for (bi, block) in func.blocks.iter().enumerate() {
                let num_instructions = block.instructions.len() as u32;

                // Special opcode to advance address and line simultaneously
                if num_instructions > 0 {
                    let line_advance = 1i32; // 1 line per block
                    let addr_advance = num_instructions * 4;

                    // DW_LNS_advance_line
                    buf.push(3);
                    Self::write_sleb128(&mut buf, line_advance as i64);

                    // DW_LNS_advance_pc
                    buf.push(2);
                    Self::write_uleb128(&mut buf, addr_advance as u64);

                    // DW_LNS_copy
                    buf.push(1);

                    let _ = line + 1; // Track line advancement
                    address += addr_advance as u64;
                }
            }
        }

        // End sequence
        buf.push(0); // extended opcode
        buf.push(1); // size
        buf.push(1); // DW_LNE_end_sequence

        // Patch lengths
        let total_length = (buf.len() - 4) as u32;
        buf[0..4].copy_from_slice(&total_length.to_le_bytes());
        let header_offset = 10; // After unit_length + version + addr_size + seg_size
        buf[header_offset..header_offset + 4].copy_from_slice(&header_length.to_le_bytes());

        buf
    }

    fn emit_string_section(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        for s in &self.string_pool {
            buf.extend_from_slice(s.as_bytes());
            buf.push(0);
        }
        buf
    }

    // Helper encoding functions
    fn write_u16(buf: &mut Vec<u8>, val: u16) {
        buf.extend_from_slice(&val.to_le_bytes());
    }

    fn write_u32(buf: &mut Vec<u8>, val: u32) {
        buf.extend_from_slice(&val.to_le_bytes());
    }

    fn write_u64(buf: &mut Vec<u8>, val: u64) {
        buf.extend_from_slice(&val.to_le_bytes());
    }

    fn write_string(buf: &mut Vec<u8>, s: &str) {
        buf.extend_from_slice(s.as_bytes());
        buf.push(0);
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
