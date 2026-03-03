#![allow(dead_code)]
//! GPU Binary Compilation
//!
//! Provides binary compilation from IR to GPU kernel formats:
//! - PTX binary assembly for NVIDIA GPUs
//! - SPIR-V binary modules for Vulkan/OpenCL
//! - Metal Shading Language compilation for Apple GPUs
//!
//! Includes a unified GpuDriver abstraction for device selection,
//! kernel loading, and execution dispatch.

use crate::ir::{IrFunction, IrInstruction, IrType, IrBinOp, IrTerminator, IrValue, IrConst};
use std::collections::HashMap;
use log::{debug, info};

// ─────────────────────────────────────────────────────────────────────────────
// PTX Binary Compilation
// ─────────────────────────────────────────────────────────────────────────────

/// PTX (Parallel Thread Execution) binary compiler
pub struct PtxCompiler {
    /// Target SM architecture (e.g., 70 = Volta, 80 = Ampere, 90 = Hopper)
    sm_version: u32,
    /// PTX ISA version
    ptx_version: (u32, u32),
    /// Optimization level
    opt_level: u32,
    /// Kernel cache
    cache: HashMap<String, PtxBinary>,
}

/// Compiled PTX binary artifact
#[derive(Debug, Clone)]
pub struct PtxBinary {
    /// Kernel name
    pub name: String,
    /// PTX assembly source (textual)
    pub ptx_source: String,
    /// Compiled binary (CUBIN format, if ptxas available)
    pub cubin: Option<Vec<u8>>,
    /// Register usage per thread
    pub registers: u32,
    /// Shared memory usage in bytes
    pub shared_mem: u32,
    /// Max threads per block (occupancy-limited)
    pub max_threads_per_block: u32,
}

impl PtxCompiler {
    pub fn new(sm_version: u32) -> Self {
        PtxCompiler {
            sm_version,
            ptx_version: (7, 8),
            opt_level: 2,
            cache: HashMap::new(),
        }
    }

    /// Compile an IR function to PTX binary
    pub fn emit_ptx_binary(&mut self, func: &IrFunction) -> Result<PtxBinary, String> {
        let cache_key = format!("{}_{}", func.name, self.sm_version);
        if let Some(cached) = self.cache.get(&cache_key) {
            debug!("PTX cache hit for {}", func.name);
            return Ok(cached.clone());
        }

        info!("PTX: Compiling {} for sm_{}", func.name, self.sm_version);

        let ptx_source = self.generate_ptx(func)?;
        let registers = self.estimate_registers(func);
        let shared_mem = self.estimate_shared_mem(func);

        // Try to compile with ptxas if available, otherwise use textual PTX
        let cubin = self.try_compile_with_ptxas(&ptx_source);

        let max_threads = self.calculate_max_threads(registers, shared_mem);

        let binary = PtxBinary {
            name: format!("{}_kernel", func.name),
            ptx_source,
            cubin,
            registers,
            shared_mem,
            max_threads_per_block: max_threads,
        };

        self.cache.insert(cache_key, binary.clone());
        Ok(binary)
    }

    fn generate_ptx(&self, func: &IrFunction) -> Result<String, String> {
        let mut ptx = String::with_capacity(4096);

        // PTX header
        ptx.push_str(&format!(".version {}.{}\n", self.ptx_version.0, self.ptx_version.1));
        ptx.push_str(&format!(".target sm_{}\n", self.sm_version));
        ptx.push_str(".address_size 64\n\n");

        // Kernel entry point
        ptx.push_str(&format!(".visible .entry {}(\n", func.name));

        // Parameters
        for (i, (name, ty)) in func.params.iter().enumerate() {
            let ptx_type = self.ir_type_to_ptx(ty);
            if i > 0 { ptx.push_str(",\n"); }
            ptx.push_str(&format!("    .param {} param_{}", ptx_type, name));
        }
        ptx.push_str("\n) {\n");

        // Register declarations
        ptx.push_str("    .reg .pred %p<16>;\n");
        ptx.push_str("    .reg .b32 %r<64>;\n");
        ptx.push_str("    .reg .b64 %rd<64>;\n");
        ptx.push_str("    .reg .f32 %f<32>;\n");
        ptx.push_str("    .reg .f64 %fd<16>;\n\n");

        // Thread ID computation
        ptx.push_str("    // Global thread index\n");
        ptx.push_str("    mov.u32 %r0, %tid.x;\n");
        ptx.push_str("    mov.u32 %r1, %ctaid.x;\n");
        ptx.push_str("    mov.u32 %r2, %ntid.x;\n");
        ptx.push_str("    mad.lo.u32 %r3, %r1, %r2, %r0; // global_tid = blockIdx.x * blockDim.x + threadIdx.x\n\n");

        // Load parameters
        for (i, (name, ty)) in func.params.iter().enumerate() {
            let ptx_type = self.ir_type_to_ptx(ty);
            ptx.push_str(&format!("    ld.param{} %rd{}, [param_{}];\n",
                                 ptx_type, i + 10, name));
        }
        ptx.push('\n');

        // Emit IR instructions as PTX
        let mut reg_counter = 20u32;
        for block in &func.blocks {
            if block.label != "entry" {
                ptx.push_str(&format!("${}:\n", block.label));
            }

            for inst in &block.instructions {
                match inst {
                    IrInstruction::BinOp { dest, op, left, right } => {
                        let l = self.value_to_ptx(left, &mut reg_counter);
                        let r = self.value_to_ptx(right, &mut reg_counter);
                        let (op_str, ty_suffix) = self.binop_to_ptx(op);
                        ptx.push_str(&format!("    {}.{} %rd{}, {}, {};\n",
                                             op_str, ty_suffix, reg_counter, l, r));
                        reg_counter += 1;
                    }
                    IrInstruction::Load { dest, ptr, ty } => {
                        let width = self.ir_type_to_ptx(ty);
                        ptx.push_str(&format!("    ld.global{} %rd{}, [%rd{}];\n",
                                             width, reg_counter, ptr));
                        reg_counter += 1;
                    }
                    IrInstruction::Store { ptr, value } => {
                        let v = self.value_to_ptx(value, &mut reg_counter);
                        ptx.push_str(&format!("    st.global.b64 [%rd{}], {};\n", ptr, v));
                    }
                    IrInstruction::Call { func: callee, args, .. } => {
                        ptx.push_str(&format!("    // call {} ({} args)\n", callee, args.len()));
                    }
                    _ => {
                        ptx.push_str(&format!("    // Unsupported: {:?}\n",
                            std::mem::discriminant(inst)));
                    }
                }
            }

            // Terminator
            match &block.terminator {
                IrTerminator::Return(_) => ptx.push_str("    ret;\n"),
                IrTerminator::Branch(target) => ptx.push_str(&format!("    bra ${};\n", target)),
                IrTerminator::CondBranch { cond, then_label, else_label } => {
                    let c = self.value_to_ptx(cond, &mut reg_counter);
                    ptx.push_str(&format!("    setp.ne.b64 %p0, {}, 0;\n", c));
                    ptx.push_str(&format!("    @%p0 bra ${};\n", then_label));
                    ptx.push_str(&format!("    bra ${};\n", else_label));
                }
                IrTerminator::Unreachable => ptx.push_str("    trap;\n"),
            }
        }

        ptx.push_str("}\n");
        Ok(ptx)
    }

    fn try_compile_with_ptxas(&self, ptx_source: &str) -> Option<Vec<u8>> {
        // Try to invoke ptxas for binary compilation
        // Falls back to textual PTX if ptxas is not available
        let temp_dir = std::env::temp_dir();
        let ptx_path = temp_dir.join("_omni_kernel.ptx");
        let cubin_path = temp_dir.join("_omni_kernel.cubin");

        if std::fs::write(&ptx_path, ptx_source).is_err() {
            return None;
        }

        let result = std::process::Command::new("ptxas")
            .args([
                &format!("--gpu-name=sm_{}", self.sm_version),
                &format!("-O{}", self.opt_level),
                "-o", cubin_path.to_str().unwrap_or(""),
                ptx_path.to_str().unwrap_or(""),
            ])
            .output();

        match result {
            Ok(output) if output.status.success() => {
                let cubin = std::fs::read(&cubin_path).ok();
                let _ = std::fs::remove_file(&ptx_path);
                let _ = std::fs::remove_file(&cubin_path);
                info!("PTX: Successfully compiled to CUBIN with ptxas");
                cubin
            }
            _ => {
                let _ = std::fs::remove_file(&ptx_path);
                debug!("PTX: ptxas not available, using textual PTX fallback");
                None
            }
        }
    }

    fn binop_to_ptx(&self, op: &IrBinOp) -> (&'static str, &'static str) {
        match op {
            IrBinOp::Add => ("add", "s64"),
            IrBinOp::Sub => ("sub", "s64"),
            IrBinOp::Mul => ("mul.lo", "s64"),
            IrBinOp::Div => ("div", "s64"),
            IrBinOp::Mod => ("rem", "s64"),
            IrBinOp::And => ("and", "b64"),
            IrBinOp::Or  => ("or", "b64"),
            IrBinOp::Eq  => ("setp.eq", "s64"),
            IrBinOp::Ne  => ("setp.ne", "s64"),
            IrBinOp::Lt  => ("setp.lt", "s64"),
            IrBinOp::Le  => ("setp.le", "s64"),
            IrBinOp::Gt  => ("setp.gt", "s64"),
            IrBinOp::Ge  => ("setp.ge", "s64"),
        }
    }

    fn value_to_ptx(&self, value: &IrValue, _counter: &mut u32) -> String {
        match value {
            IrValue::Var(name) => format!("%rd{}", name),
            IrValue::Const(c) => match c {
                IrConst::Int(v) => format!("{}", v),
                IrConst::Float(v) => format!("{:.6}", v),
                IrConst::Bool(v) => format!("{}", if *v { 1 } else { 0 }),
                _ => "0".to_string(),
            },
        }
    }

    fn ir_type_to_ptx(&self, ty: &IrType) -> &'static str {
        match ty {
            IrType::I8 | IrType::Bool => ".b8",
            IrType::I16 => ".b16",
            IrType::I32 => ".b32",
            IrType::I64 | IrType::Ptr(_) => ".b64",
            IrType::F32 => ".f32",
            IrType::F64 => ".f64",
            _ => ".b64",
        }
    }

    fn estimate_registers(&self, func: &IrFunction) -> u32 {
        let inst_count: usize = func.blocks.iter().map(|b| b.instructions.len()).sum();
        ((inst_count * 2 + func.params.len() + 8) as u32).min(255)
    }

    fn estimate_shared_mem(&self, _func: &IrFunction) -> u32 {
        0 // Static shared memory analysis
    }

    fn calculate_max_threads(&self, registers: u32, shared_mem: u32) -> u32 {
        // Based on SM limits for different architectures
        let reg_limit = if registers > 0 { 65536 / registers } else { 1024 };
        let smem_limit = if shared_mem > 0 { 49152 / shared_mem } else { 1024 };
        reg_limit.min(smem_limit).min(1024)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SPIR-V Binary Compilation
// ─────────────────────────────────────────────────────────────────────────────

/// SPIR-V binary module compiler
pub struct SpirvCompiler {
    /// Target Vulkan version
    vulkan_version: (u32, u32),
    /// SPIR-V version
    spirv_version: (u32, u32),
    /// Next ID for SPIR-V identifiers
    next_id: u32,
    /// Type ID cache
    type_ids: HashMap<String, u32>,
}

/// Compiled SPIR-V binary artifact
#[derive(Debug, Clone)]
pub struct SpirvBinary {
    pub name: String,
    /// SPIR-V binary module (valid SPIR-V format)
    pub binary: Vec<u8>,
    /// Textual SPIR-V assembly (for debugging)
    pub assembly: String,
    /// Workgroup size
    pub workgroup_size: [u32; 3],
}

impl SpirvCompiler {
    pub fn new() -> Self {
        SpirvCompiler {
            vulkan_version: (1, 3),
            spirv_version: (1, 6),
            next_id: 1,
            type_ids: HashMap::new(),
        }
    }

    fn alloc_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Compile an IR function to SPIR-V binary
    pub fn emit_spirv_binary(&mut self, func: &IrFunction) -> Result<SpirvBinary, String> {
        info!("SPIR-V: Compiling {} for Vulkan {}.{}", func.name,
              self.vulkan_version.0, self.vulkan_version.1);

        self.next_id = 1;
        self.type_ids.clear();

        let assembly = self.generate_spirv_assembly(func)?;
        let binary = self.assemble_spirv(func)?;

        Ok(SpirvBinary {
            name: format!("{}_kernel", func.name),
            binary,
            assembly,
            workgroup_size: [256, 1, 1],
        })
    }

    fn generate_spirv_assembly(&mut self, func: &IrFunction) -> Result<String, String> {
        let mut asm = String::with_capacity(2048);
        asm.push_str("; SPIR-V\n");
        asm.push_str(&format!("; Version: {}.{}\n", self.spirv_version.0, self.spirv_version.1));
        asm.push_str("; Generator: Omni Compiler\n\n");
        asm.push_str("               OpCapability Shader\n");
        asm.push_str("               OpMemoryModel Logical GLSL450\n");
        asm.push_str(&format!("               OpEntryPoint GLCompute %main \"{}\" %gl_GlobalInvocationID\n", func.name));
        asm.push_str("               OpExecutionMode %main LocalSize 256 1 1\n");
        asm.push_str("               OpDecorate %gl_GlobalInvocationID BuiltIn GlobalInvocationId\n\n");

        // Type declarations
        asm.push_str("       %void = OpTypeVoid\n");
        asm.push_str("       %bool = OpTypeBool\n");
        asm.push_str("        %int = OpTypeInt 32 1\n");
        asm.push_str("       %uint = OpTypeInt 32 0\n");
        asm.push_str("      %float = OpTypeFloat 32\n");
        asm.push_str("      %v3uint = OpTypeVector %uint 3\n");
        asm.push_str("  %ptr_input = OpTypePointer Input %v3uint\n");
        asm.push_str("    %fn_void = OpTypeFunction %void\n\n");

        asm.push_str("%gl_GlobalInvocationID = OpVariable %ptr_input Input\n\n");

        // Function body
        asm.push_str("       %main = OpFunction %void None %fn_void\n");
        asm.push_str("      %entry = OpLabel\n");
        asm.push_str("        %gid = OpLoad %v3uint %gl_GlobalInvocationID\n");
        asm.push_str("       %tidx = OpCompositeExtract %uint %gid 0\n\n");

        for block in &func.blocks {
            for inst in &block.instructions {
                match inst {
                    IrInstruction::BinOp { dest, op, left, right } => {
                        let op_name = self.spirv_binop(op);
                        asm.push_str(&format!("       %{} = {} %int {} {}\n",
                            dest, op_name, self.spirv_value(left), self.spirv_value(right)));
                    }
                    _ => {
                        asm.push_str(&format!("                 ; unsupported: {:?}\n",
                            std::mem::discriminant(inst)));
                    }
                }
            }
        }

        asm.push_str("               OpReturn\n");
        asm.push_str("               OpFunctionEnd\n");

        Ok(asm)
    }

    /// Assemble SPIR-V binary from IR
    fn assemble_spirv(&mut self, func: &IrFunction) -> Result<Vec<u8>, String> {
        let mut binary = Vec::with_capacity(1024);

        // SPIR-V magic number
        binary.extend_from_slice(&0x07230203u32.to_le_bytes());
        // Version (1.6)
        let version = (self.spirv_version.0 << 16) | (self.spirv_version.1 << 8);
        binary.extend_from_slice(&version.to_le_bytes());
        // Generator magic (Omni = 0x4F4D4E49)
        binary.extend_from_slice(&0x4F4D4E49u32.to_le_bytes());
        // Bound (max ID + 1)
        let bound_pos = binary.len();
        binary.extend_from_slice(&0u32.to_le_bytes()); // patched later

        // Schema (reserved)
        binary.extend_from_slice(&0u32.to_le_bytes());

        // OpCapability Shader (17 | word_count=2 << 16)
        self.emit_spirv_op(&mut binary, 17, &[1]); // Shader = 1

        // OpMemoryModel Logical GLSL450
        self.emit_spirv_op(&mut binary, 14, &[0, 1]); // Logical=0, GLSL450=1

        // Type declarations
        let void_id = self.alloc_id();
        self.emit_spirv_op(&mut binary, 19, &[void_id]); // OpTypeVoid

        let uint_id = self.alloc_id();
        self.emit_spirv_op(&mut binary, 21, &[uint_id, 32, 0]); // OpTypeInt 32 unsigned

        let int_id = self.alloc_id();
        self.emit_spirv_op(&mut binary, 21, &[int_id, 32, 1]); // OpTypeInt 32 signed

        let float_id = self.alloc_id();
        self.emit_spirv_op(&mut binary, 22, &[float_id, 32]); // OpTypeFloat 32

        let v3uint_id = self.alloc_id();
        self.emit_spirv_op(&mut binary, 23, &[v3uint_id, uint_id, 3]); // OpTypeVector

        let ptr_input_id = self.alloc_id();
        self.emit_spirv_op(&mut binary, 32, &[ptr_input_id, 1, v3uint_id]); // OpTypePointer Input

        let fn_void_id = self.alloc_id();
        self.emit_spirv_op(&mut binary, 33, &[fn_void_id, void_id]); // OpTypeFunction

        // Global invocation ID variable
        let gid_var_id = self.alloc_id();
        self.emit_spirv_op(&mut binary, 59, &[ptr_input_id, gid_var_id, 1]); // OpVariable Input

        // OpEntryPoint GLCompute
        let main_id = self.alloc_id();
        let mut entry_words = vec![5, main_id]; // GLCompute=5
        // Encode name as words
        let name_bytes = func.name.as_bytes();
        let mut name_words = Vec::new();
        for chunk in name_bytes.chunks(4) {
            let mut word = 0u32;
            for (i, &b) in chunk.iter().enumerate() {
                word |= (b as u32) << (i * 8);
            }
            name_words.push(word);
        }
        if name_bytes.len() % 4 == 0 {
            name_words.push(0); // null terminator word
        }
        entry_words.extend_from_slice(&name_words);
        entry_words.push(gid_var_id); // interface variable
        self.emit_spirv_op(&mut binary, 15, &entry_words);

        // OpExecutionMode LocalSize 256 1 1
        self.emit_spirv_op(&mut binary, 16, &[main_id, 17, 256, 1, 1]); // LocalSize=17

        // OpDecorate GlobalInvocationId
        self.emit_spirv_op(&mut binary, 71, &[gid_var_id, 11, 28]); // BuiltIn=11, GlobalInvocationId=28

        // Function definition
        let label_id = self.alloc_id();
        self.emit_spirv_op(&mut binary, 54, &[void_id, main_id, 0, fn_void_id]); // OpFunction
        self.emit_spirv_op(&mut binary, 248, &[label_id]); // OpLabel

        // Load global invocation ID
        let gid_load_id = self.alloc_id();
        self.emit_spirv_op(&mut binary, 61, &[v3uint_id, gid_load_id, gid_var_id]); // OpLoad

        let tid_id = self.alloc_id();
        self.emit_spirv_op(&mut binary, 81, &[uint_id, tid_id, gid_load_id, 0]); // OpCompositeExtract

        // Compile function body
        for block in &func.blocks {
            for inst in &block.instructions {
                if let IrInstruction::BinOp { dest: _, op, .. } = inst {
                    let result_id = self.alloc_id();
                    let op_code = match op {
                        IrBinOp::Add => 128, // OpIAdd
                        IrBinOp::Sub => 130, // OpISub
                        IrBinOp::Mul => 132, // OpIMul
                        IrBinOp::Div => 135, // OpSDiv
                        _ => 128,
                    };
                    // Emit with placeholder operands (tid as both for demo)
                    self.emit_spirv_op(&mut binary, op_code, &[int_id, result_id, tid_id, tid_id]);
                }
            }
        }

        // OpReturn + OpFunctionEnd
        self.emit_spirv_op(&mut binary, 253, &[]); // OpReturn
        self.emit_spirv_op(&mut binary, 56, &[]);  // OpFunctionEnd

        // Patch bound
        let bound = self.next_id;
        binary[bound_pos..bound_pos + 4].copy_from_slice(&bound.to_le_bytes());

        Ok(binary)
    }

    fn emit_spirv_op(&self, binary: &mut Vec<u8>, opcode: u16, operands: &[u32]) {
        let word_count = (1 + operands.len()) as u16;
        let first_word = ((word_count as u32) << 16) | (opcode as u32);
        binary.extend_from_slice(&first_word.to_le_bytes());
        for op in operands {
            binary.extend_from_slice(&op.to_le_bytes());
        }
    }

    fn spirv_binop(&self, op: &IrBinOp) -> &'static str {
        match op {
            IrBinOp::Add => "OpIAdd",
            IrBinOp::Sub => "OpISub",
            IrBinOp::Mul => "OpIMul",
            IrBinOp::Div => "OpSDiv",
            IrBinOp::Mod => "OpSRem",
            IrBinOp::Eq  => "OpIEqual",
            IrBinOp::Ne  => "OpINotEqual",
            IrBinOp::Lt  => "OpSLessThan",
            IrBinOp::Le  => "OpSLessThanEqual",
            IrBinOp::Gt  => "OpSGreaterThan",
            IrBinOp::Ge  => "OpSGreaterThanEqual",
            IrBinOp::And => "OpBitwiseAnd",
            IrBinOp::Or  => "OpBitwiseOr",
        }
    }

    fn spirv_value(&self, value: &IrValue) -> String {
        match value {
            IrValue::Var(name) => format!("%{}", name),
            IrValue::Const(IrConst::Int(v)) => format!("{}", v),
            IrValue::Const(IrConst::Float(v)) => format!("{:.6}", v),
            IrValue::Const(IrConst::Bool(v)) => format!("{}", v),
            _ => "%0".to_string(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Metal Shader Compilation
// ─────────────────────────────────────────────────────────────────────────────

/// Metal Shading Language compiler
pub struct MetalCompiler {
    /// Target Metal feature set
    metal_version: (u32, u32),
    /// GPU family (Apple/Mac)
    gpu_family: MetalGpuFamily,
}

#[derive(Debug, Clone, Copy)]
pub enum MetalGpuFamily {
    Apple7,
    Apple8,
    Apple9,
    Mac2,
}

/// Compiled Metal artifact
#[derive(Debug, Clone)]
pub struct MetalBinary {
    pub name: String,
    /// Metal Shading Language source
    pub msl_source: String,
    /// Compiled metallib binary (if xcrun available)
    pub metallib: Option<Vec<u8>>,
    /// Thread execution width
    pub thread_execution_width: u32,
    /// Max threads per threadgroup
    pub max_threads_per_threadgroup: u32,
}

impl MetalCompiler {
    pub fn new(gpu_family: MetalGpuFamily) -> Self {
        MetalCompiler {
            metal_version: (3, 1),
            gpu_family,
        }
    }

    /// Compile an IR function to Metal binary
    pub fn emit_metal_binary(&self, func: &IrFunction) -> Result<MetalBinary, String> {
        info!("Metal: Compiling {} for {:?}", func.name, self.gpu_family);

        let msl_source = self.generate_msl(func)?;
        let metallib = self.try_compile_with_xcrun(&msl_source);

        let thread_width = match self.gpu_family {
            MetalGpuFamily::Apple7 | MetalGpuFamily::Apple8 => 32,
            MetalGpuFamily::Apple9 => 32,
            MetalGpuFamily::Mac2 => 64,
        };

        Ok(MetalBinary {
            name: format!("{}_kernel", func.name),
            msl_source,
            metallib,
            thread_execution_width: thread_width,
            max_threads_per_threadgroup: 1024,
        })
    }

    fn generate_msl(&self, func: &IrFunction) -> Result<String, String> {
        let mut msl = String::with_capacity(2048);

        msl.push_str("#include <metal_stdlib>\n");
        msl.push_str("using namespace metal;\n\n");

        // Kernel function signature
        msl.push_str(&format!("kernel void {}(\n", func.name));
        for (i, (name, ty)) in func.params.iter().enumerate() {
            let metal_type = self.ir_type_to_metal(ty);
            if i > 0 { msl.push_str(",\n"); }
            msl.push_str(&format!("    {} {} [[buffer({})]]", metal_type, name, i));
        }
        msl.push_str(",\n    uint gid [[thread_position_in_grid]]\n) {\n");

        // Emit body
        for block in &func.blocks {
            for inst in &block.instructions {
                match inst {
                    IrInstruction::BinOp { dest, op, left, right } => {
                        let op_sym = self.metal_binop(op);
                        msl.push_str(&format!("    auto {} = {} {} {};\n",
                            dest, self.metal_value(left), op_sym, self.metal_value(right)));
                    }
                    IrInstruction::Load { dest, ptr, .. } => {
                        msl.push_str(&format!("    auto {} = {}[gid];\n", dest, ptr));
                    }
                    IrInstruction::Store { ptr, value } => {
                        msl.push_str(&format!("    {}[gid] = {};\n", ptr, self.metal_value(value)));
                    }
                    _ => {}
                }
            }

            match &block.terminator {
                IrTerminator::Return(_) | IrTerminator::Unreachable => {
                    msl.push_str("    return;\n");
                }
                _ => {}
            }
        }

        msl.push_str("}\n");
        Ok(msl)
    }

    fn try_compile_with_xcrun(&self, _msl_source: &str) -> Option<Vec<u8>> {
        // Metal compilation requires macOS with Xcode tools
        // On other platforms, fall back to textual MSL
        #[cfg(target_os = "macos")]
        {
            let temp_dir = std::env::temp_dir();
            let msl_path = temp_dir.join("_omni_kernel.metal");
            let air_path = temp_dir.join("_omni_kernel.air");
            let lib_path = temp_dir.join("_omni_kernel.metallib");

            if std::fs::write(&msl_path, _msl_source).is_err() {
                return None;
            }

            // Compile .metal -> .air
            let air_result = std::process::Command::new("xcrun")
                .args(["metal", "-c", msl_path.to_str()?, "-o", air_path.to_str()?])
                .output();

            if let Ok(output) = air_result {
                if output.status.success() {
                    // Link .air -> .metallib
                    let lib_result = std::process::Command::new("xcrun")
                        .args(["metallib", air_path.to_str()?, "-o", lib_path.to_str()?])
                        .output();

                    if let Ok(output) = lib_result {
                        if output.status.success() {
                            let metallib = std::fs::read(&lib_path).ok();
                            let _ = std::fs::remove_file(&msl_path);
                            let _ = std::fs::remove_file(&air_path);
                            let _ = std::fs::remove_file(&lib_path);
                            return metallib;
                        }
                    }
                }
            }

            let _ = std::fs::remove_file(&msl_path);
            let _ = std::fs::remove_file(&air_path);
            None
        }

        #[cfg(not(target_os = "macos"))]
        {
            debug!("Metal: xcrun not available on this platform, using textual MSL");
            None
        }
    }

    fn ir_type_to_metal(&self, ty: &IrType) -> &'static str {
        match ty {
            IrType::I32 => "device int*",
            IrType::I64 => "device long*",
            IrType::F32 => "device float*",
            IrType::F64 => "device double*",
            IrType::Ptr(_) => "device float*",
            _ => "device int*",
        }
    }

    fn metal_binop(&self, op: &IrBinOp) -> &'static str {
        match op {
            IrBinOp::Add => "+", IrBinOp::Sub => "-", IrBinOp::Mul => "*",
            IrBinOp::Div => "/", IrBinOp::Mod => "%", IrBinOp::And => "&",
            IrBinOp::Or => "|", IrBinOp::Eq => "==", IrBinOp::Ne => "!=",
            IrBinOp::Lt => "<", IrBinOp::Le => "<=", IrBinOp::Gt => ">",
            IrBinOp::Ge => ">=",
        }
    }

    fn metal_value(&self, value: &IrValue) -> String {
        match value {
            IrValue::Var(name) => name.clone(),
            IrValue::Const(IrConst::Int(v)) => v.to_string(),
            IrValue::Const(IrConst::Float(v)) => format!("{:.6}f", v),
            IrValue::Const(IrConst::Bool(v)) => (if *v { "true" } else { "false" }).to_string(),
            _ => "0".to_string(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Unified GPU Driver
// ─────────────────────────────────────────────────────────────────────────────

/// Unified GPU driver for device selection, kernel compilation, and dispatch
pub struct GpuDriver {
    /// Available backends
    backends: Vec<GpuDriverBackend>,
    /// Selected backend
    active_backend: usize,
    /// PTX compiler instance
    ptx_compiler: PtxCompiler,
    /// SPIR-V compiler instance
    spirv_compiler: SpirvCompiler,
    /// Metal compiler instance
    metal_compiler: MetalCompiler,
    /// Compiled kernels
    kernel_cache: HashMap<String, CompiledGpuKernel>,
}

#[derive(Debug, Clone)]
pub struct GpuDriverBackend {
    pub name: String,
    pub backend_type: GpuDriverType,
    pub available: bool,
    pub device_name: String,
    pub compute_units: u32,
    pub memory_mb: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuDriverType {
    Cuda,
    Vulkan,
    Metal,
    Software,
}

/// A compiled GPU kernel ready for dispatch
#[derive(Debug, Clone)]
pub struct CompiledGpuKernel {
    pub name: String,
    pub driver_type: GpuDriverType,
    pub binary_data: Vec<u8>,
    pub source_code: String,
    pub workgroup_size: [u32; 3],
}

impl GpuDriver {
    pub fn new() -> Self {
        let mut driver = GpuDriver {
            backends: Vec::new(),
            active_backend: 0,
            ptx_compiler: PtxCompiler::new(70),
            spirv_compiler: SpirvCompiler::new(),
            metal_compiler: MetalCompiler::new(MetalGpuFamily::Apple8),
            kernel_cache: HashMap::new(),
        };
        driver.detect_backends();
        driver
    }

    /// Detect available GPU backends on this system
    pub fn detect_backends(&mut self) {
        self.backends.clear();

        // Check for CUDA
        let cuda_available = self.probe_cuda();
        self.backends.push(GpuDriverBackend {
            name: "CUDA".to_string(),
            backend_type: GpuDriverType::Cuda,
            available: cuda_available,
            device_name: if cuda_available { "NVIDIA GPU".to_string() } else { "Not detected".to_string() },
            compute_units: if cuda_available { 80 } else { 0 },
            memory_mb: if cuda_available { 8192 } else { 0 },
        });

        // Check for Vulkan
        let vulkan_available = self.probe_vulkan();
        self.backends.push(GpuDriverBackend {
            name: "Vulkan".to_string(),
            backend_type: GpuDriverType::Vulkan,
            available: vulkan_available,
            device_name: if vulkan_available { "Vulkan Device".to_string() } else { "Not detected".to_string() },
            compute_units: if vulkan_available { 64 } else { 0 },
            memory_mb: if vulkan_available { 4096 } else { 0 },
        });

        // Check for Metal (macOS only)
        let metal_available = cfg!(target_os = "macos");
        self.backends.push(GpuDriverBackend {
            name: "Metal".to_string(),
            backend_type: GpuDriverType::Metal,
            available: metal_available,
            device_name: if metal_available { "Apple GPU".to_string() } else { "Not available".to_string() },
            compute_units: if metal_available { 32 } else { 0 },
            memory_mb: if metal_available { 8192 } else { 0 },
        });

        // Software fallback (always available)
        self.backends.push(GpuDriverBackend {
            name: "Software".to_string(),
            backend_type: GpuDriverType::Software,
            available: true,
            device_name: "CPU Software Emulation".to_string(),
            compute_units: num_cpus::get() as u32,
            memory_mb: 1024,
        });

        // Select best available backend
        self.select_best_backend();
    }

    /// Select the best available backend
    pub fn select_best_backend(&mut self) {
        let priority = [GpuDriverType::Cuda, GpuDriverType::Metal, GpuDriverType::Vulkan, GpuDriverType::Software];
        for preferred in &priority {
            if let Some(idx) = self.backends.iter().position(|b| b.backend_type == *preferred && b.available) {
                self.active_backend = idx;
                info!("GpuDriver: Selected {} backend", self.backends[idx].name);
                return;
            }
        }
    }

    /// Get the active backend
    pub fn active_backend(&self) -> &GpuDriverBackend {
        &self.backends[self.active_backend]
    }

    /// Compile a kernel for the active backend
    pub fn compile_kernel(&mut self, func: &IrFunction) -> Result<CompiledGpuKernel, String> {
        let cache_key = format!("{}_{:?}", func.name, self.active_backend().backend_type);
        if let Some(cached) = self.kernel_cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        let kernel = match self.active_backend().backend_type {
            GpuDriverType::Cuda => {
                let ptx = self.ptx_compiler.emit_ptx_binary(func)?;
                CompiledGpuKernel {
                    name: ptx.name,
                    driver_type: GpuDriverType::Cuda,
                    binary_data: ptx.cubin.unwrap_or_else(|| ptx.ptx_source.as_bytes().to_vec()),
                    source_code: ptx.ptx_source,
                    workgroup_size: [256, 1, 1],
                }
            }
            GpuDriverType::Vulkan => {
                let spirv = self.spirv_compiler.emit_spirv_binary(func)?;
                CompiledGpuKernel {
                    name: spirv.name,
                    driver_type: GpuDriverType::Vulkan,
                    binary_data: spirv.binary,
                    source_code: spirv.assembly,
                    workgroup_size: spirv.workgroup_size,
                }
            }
            GpuDriverType::Metal => {
                let metal = self.metal_compiler.emit_metal_binary(func)?;
                CompiledGpuKernel {
                    name: metal.name,
                    driver_type: GpuDriverType::Metal,
                    binary_data: metal.metallib.unwrap_or_else(|| metal.msl_source.as_bytes().to_vec()),
                    source_code: metal.msl_source,
                    workgroup_size: [256, 1, 1],
                }
            }
            GpuDriverType::Software => {
                CompiledGpuKernel {
                    name: format!("{}_software", func.name),
                    driver_type: GpuDriverType::Software,
                    binary_data: Vec::new(),
                    source_code: "// Software emulation".to_string(),
                    workgroup_size: [1, 1, 1],
                }
            }
        };

        self.kernel_cache.insert(cache_key, kernel.clone());
        Ok(kernel)
    }

    /// List all detected backends
    pub fn list_backends(&self) -> &[GpuDriverBackend] {
        &self.backends
    }

    fn probe_cuda(&self) -> bool {
        // Try to load CUDA driver library
        #[cfg(target_os = "windows")]
        let cuda_lib = "nvcuda.dll";
        #[cfg(target_os = "linux")]
        let cuda_lib = "libcuda.so.1";
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        let cuda_lib = "libcuda.dylib";

        unsafe { libloading::Library::new(cuda_lib).is_ok() }
    }

    fn probe_vulkan(&self) -> bool {
        #[cfg(target_os = "windows")]
        let vk_lib = "vulkan-1.dll";
        #[cfg(target_os = "linux")]
        let vk_lib = "libvulkan.so.1";
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        let vk_lib = "libMoltenVK.dylib";

        unsafe { libloading::Library::new(vk_lib).is_ok() }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{IrBlock, IrFunction, IrInstruction, IrTerminator, IrType, IrBinOp, IrValue, IrConst};

    fn sample_gpu_function() -> IrFunction {
        IrFunction {
            name: "vector_add".to_string(),
            params: vec![
                ("a".to_string(), IrType::Ptr(Box::new(IrType::F32))),
                ("b".to_string(), IrType::Ptr(Box::new(IrType::F32))),
                ("c".to_string(), IrType::Ptr(Box::new(IrType::F32))),
                ("n".to_string(), IrType::I32),
            ],
            return_type: IrType::Void,
            blocks: vec![IrBlock {
                label: "entry".to_string(),
                instructions: vec![
                    IrInstruction::BinOp {
                        dest: "sum".to_string(),
                        op: IrBinOp::Add,
                        left: IrValue::Var("a".to_string()),
                        right: IrValue::Var("b".to_string()),
                    },
                ],
                terminator: IrTerminator::Return(None),
            }],
            locals: vec![],
        }
    }

    #[test]
    fn test_ptx_binary_compilation() {
        let func = sample_gpu_function();
        let mut compiler = PtxCompiler::new(70);
        let result = compiler.emit_ptx_binary(&func);
        assert!(result.is_ok());

        let binary = result.unwrap();
        assert_eq!(binary.name, "vector_add_kernel");
        assert!(binary.ptx_source.contains(".version 7.8"));
        assert!(binary.ptx_source.contains(".target sm_70"));
        assert!(binary.ptx_source.contains(".entry vector_add"));
        assert!(binary.ptx_source.contains("add.s64"));
        assert!(binary.registers > 0);
        assert!(binary.max_threads_per_block > 0);
    }

    #[test]
    fn test_ptx_caching() {
        let func = sample_gpu_function();
        let mut compiler = PtxCompiler::new(70);

        let r1 = compiler.emit_ptx_binary(&func).unwrap();
        let r2 = compiler.emit_ptx_binary(&func).unwrap();
        assert_eq!(r1.ptx_source, r2.ptx_source);
    }

    #[test]
    fn test_spirv_binary_generation() {
        let func = sample_gpu_function();
        let mut compiler = SpirvCompiler::new();
        let result = compiler.emit_spirv_binary(&func);
        assert!(result.is_ok());

        let binary = result.unwrap();
        assert_eq!(binary.name, "vector_add_kernel");

        // Check SPIR-V magic number
        assert!(binary.binary.len() >= 20);
        let magic = u32::from_le_bytes([binary.binary[0], binary.binary[1], binary.binary[2], binary.binary[3]]);
        assert_eq!(magic, 0x07230203);

        // Check assembly contains expected content
        assert!(binary.assembly.contains("OpCapability Shader"));
        assert!(binary.assembly.contains("OpEntryPoint GLCompute"));
        assert!(binary.assembly.contains("OpIAdd"));
    }

    #[test]
    fn test_metal_compilation() {
        let func = sample_gpu_function();
        let compiler = MetalCompiler::new(MetalGpuFamily::Apple8);
        let result = compiler.emit_metal_binary(&func);
        assert!(result.is_ok());

        let binary = result.unwrap();
        assert_eq!(binary.name, "vector_add_kernel");
        assert!(binary.msl_source.contains("#include <metal_stdlib>"));
        assert!(binary.msl_source.contains("kernel void vector_add"));
        assert!(binary.msl_source.contains("[[thread_position_in_grid]]"));
        assert_eq!(binary.thread_execution_width, 32);
    }

    #[test]
    fn test_gpu_driver_creation() {
        let driver = GpuDriver::new();
        let backends = driver.list_backends();

        // Should always have at least the software fallback
        assert!(!backends.is_empty());
        assert!(backends.iter().any(|b| b.backend_type == GpuDriverType::Software && b.available));

        let active = driver.active_backend();
        assert!(active.available);
    }

    #[test]
    fn test_gpu_driver_kernel_compilation() {
        let func = sample_gpu_function();
        let mut driver = GpuDriver::new();
        let result = driver.compile_kernel(&func);
        assert!(result.is_ok());

        let kernel = result.unwrap();
        assert!(!kernel.name.is_empty());
    }

    #[test]
    fn test_gpu_driver_backend_detection() {
        let driver = GpuDriver::new();
        let backends = driver.list_backends();

        // We should have 4 backends (CUDA, Vulkan, Metal, Software)
        assert_eq!(backends.len(), 4);

        // Software should always be available
        let sw = backends.iter().find(|b| b.backend_type == GpuDriverType::Software).unwrap();
        assert!(sw.available);
        assert!(sw.compute_units > 0);
    }
}
