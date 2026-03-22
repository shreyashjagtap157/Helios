//! LLVM Backend using inkwell
//! Full native code generation through LLVM

use crate::ir::{IrBlock, IrFunction, IrInstruction, IrModule, IrTerminator, IrType};
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType, StructType};
use inkwell::values::{
    BasicValue, BasicValueEnum, FloatValue, FunctionValue, IntValue, PointerValue,
};
use inkwell::{AddressSpace, FloatPredicate, IntPredicate, OptimizationLevel};
use log::{debug, info, trace};
use std::collections::HashMap;
use std::path::Path;

/// LLVM code generator
pub struct LlvmCodegen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,

    // Symbol tables
    functions: HashMap<String, FunctionValue<'ctx>>,
    locals: HashMap<String, PointerValue<'ctx>>,
    types: HashMap<String, StructType<'ctx>>,

    // Current function context
    current_function: Option<FunctionValue<'ctx>>,
    current_blocks: HashMap<String, BasicBlock<'ctx>>,

    // Debug info
    enable_debug: bool,
}

impl<'ctx> LlvmCodegen<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        LlvmCodegen {
            context,
            module,
            builder,
            functions: HashMap::new(),
            locals: HashMap::new(),
            types: HashMap::new(),
            current_function: None,
            current_blocks: HashMap::new(),
            enable_debug: false,
        }
    }

    /// Enable debug info generation
    pub fn with_debug(mut self) -> Self {
        self.enable_debug = true;
        self
    }

    /// Generate LLVM IR from Omni IR
    pub fn generate(&mut self, ir: &IrModule) -> Result<(), String> {
        info!("Generating LLVM IR for module: {}", ir.name);

        // Set target triple
        self.module.set_triple(&TargetMachine::get_default_triple());

        // Declare runtime functions
        self.declare_runtime_functions();

        // Forward declare all functions first
        for func in &ir.functions {
            self.declare_function(func)?;
        }

        // Forward declare struct types
        for struct_info in &ir.type_info {
            self.declare_struct(&struct_info.name, &struct_info.kind)?;
        }

        // Generate function bodies
        for func in &ir.functions {
            self.generate_function(func)?;
        }

        // Verify the module
        if let Err(msg) = self.module.verify() {
            return Err(format!("LLVM verification failed: {}", msg.to_string()));
        }

        info!("LLVM IR generation complete");
        Ok(())
    }

    /// Declare runtime support functions
    fn declare_runtime_functions(&mut self) {
        let i64_type = self.context.i64_type();
        let i8_ptr = self.context.i8_type().ptr_type(AddressSpace::default());
        let void_type = self.context.void_type();

        // GC allocation
        let alloc_fn_type = i8_ptr.fn_type(&[i64_type.into()], false);
        self.module
            .add_function("__omni_gc_alloc", alloc_fn_type, None);

        // GC safepoint
        let safepoint_fn_type = void_type.fn_type(&[], false);
        self.module
            .add_function("__omni_check_safepoint", safepoint_fn_type, None);

        // Panic handler
        let panic_fn_type = void_type.fn_type(&[i8_ptr.into()], false);
        self.module
            .add_function("__omni_panic", panic_fn_type, None);

        // Bound check
        let bound_check_type = void_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        self.module
            .add_function("__omni_bounds_check", bound_check_type, None);

        // Debug print
        let print_fn_type = void_type.fn_type(&[i8_ptr.into()], false);
        self.module
            .add_function("__omni_debug_print", print_fn_type, None);
    }

    /// Declare a struct type
    fn declare_struct(&mut self, name: &str, _kind: &crate::ir::TypeKind) -> Result<(), String> {
        if self.types.contains_key(name) {
            return Ok(());
        }

        let struct_type = self.context.opaque_struct_type(name);
        self.types.insert(name.to_string(), struct_type);
        Ok(())
    }

    /// Forward declare a function
    fn declare_function(&mut self, ir_func: &IrFunction) -> Result<(), String> {
        let fn_type = self.ir_function_type(ir_func)?;
        let function = self.module.add_function(&ir_func.name, fn_type, None);

        // Set parameter names
        for (i, (name, _)) in ir_func.params.iter().enumerate() {
            if let Some(param) = function.get_nth_param(i as u32) {
                param.set_name(name);
            }
        }

        self.functions.insert(ir_func.name.clone(), function);
        Ok(())
    }

    /// Generate a function body
    fn generate_function(&mut self, ir_func: &IrFunction) -> Result<(), String> {
        debug!("Generating function: {}", ir_func.name);

        let function = *self
            .functions
            .get(&ir_func.name)
            .ok_or_else(|| format!("Function not declared: {}", ir_func.name))?;

        self.current_function = Some(function);
        self.locals.clear();
        self.current_blocks.clear();

        // Create basic blocks for all IR blocks first
        for block in &ir_func.blocks {
            let bb = self.context.append_basic_block(function, &block.label);
            self.current_blocks.insert(block.label.clone(), bb);
        }

        // Generate entry block with allocas for parameters
        let entry_block = self
            .current_blocks
            .get(&ir_func.blocks[0].label)
            .ok_or("Missing entry block")?;
        self.builder.position_at_end(*entry_block);

        // Allocate locals for parameters
        for (i, (name, ty)) in ir_func.params.iter().enumerate() {
            let llvm_type = self.ir_type_to_llvm(ty)?;
            let alloca = self
                .builder
                .build_alloca(llvm_type, name)
                .map_err(|e| format!("Failed to create alloca: {:?}", e))?;

            // Store parameter value
            let param = function
                .get_nth_param(i as u32)
                .ok_or_else(|| format!("Missing parameter {}", i))?;
            self.builder
                .build_store(alloca, param)
                .map_err(|e| format!("Failed to store param: {:?}", e))?;

            self.locals.insert(name.clone(), alloca);
        }

        // Generate all blocks
        for block in &ir_func.blocks {
            self.generate_block(block)?;
        }

        self.current_function = None;
        Ok(())
    }

    /// Generate a basic block
    fn generate_block(&mut self, block: &IrBlock) -> Result<(), String> {
        let bb = *self
            .current_blocks
            .get(&block.label)
            .ok_or_else(|| format!("Missing block: {}", block.label))?;

        self.builder.position_at_end(bb);

        // Generate instructions
        for inst in &block.instructions {
            self.generate_instruction(inst)?;
        }

        // Generate terminator
        self.generate_terminator(&block.terminator)?;

        Ok(())
    }

    /// Generate an instruction
    fn generate_instruction(&mut self, inst: &IrInstruction) -> Result<(), String> {
        match inst {
            IrInstruction::Alloca { dest, ty } => {
                let llvm_type = self.ir_type_to_llvm(ty)?;
                let alloca = self
                    .builder
                    .build_alloca(llvm_type, dest)
                    .map_err(|e| format!("Alloca failed: {:?}", e))?;
                self.locals.insert(dest.clone(), alloca);
            }

            IrInstruction::Load { dest, ptr, ty } => {
                let ptr_val = self.get_local(ptr)?;
                let llvm_type = self.ir_type_to_llvm(ty)?;
                let loaded = self
                    .builder
                    .build_load(llvm_type, ptr_val, dest)
                    .map_err(|e| format!("Load failed: {:?}", e))?;

                // Store in a temporary local
                let temp = self
                    .builder
                    .build_alloca(llvm_type, &format!("{}_temp", dest))
                    .map_err(|e| format!("Temp alloca failed: {:?}", e))?;
                self.builder
                    .build_store(temp, loaded)
                    .map_err(|e| format!("Store temp failed: {:?}", e))?;
                self.locals.insert(dest.clone(), temp);
            }

            IrInstruction::Store { ptr, value } => {
                let ptr_val = self.get_local(ptr)?;
                let val = self.resolve_value(value)?;
                self.builder
                    .build_store(ptr_val, val)
                    .map_err(|e| format!("Store failed: {:?}", e))?;
            }

            IrInstruction::BinOp {
                dest,
                op,
                left,
                right,
            } => {
                let lhs = self.resolve_value(left)?;
                let rhs = self.resolve_value(right)?;

                let result = match (lhs, rhs) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        self.generate_int_binop(op, l, r, dest)?
                    }
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                        self.generate_float_binop(op, l, r, dest)?
                    }
                    _ => return Err(format!("Unsupported operand types for {}", op)),
                };

                // Store result
                let ty = result.get_type();
                let temp = self
                    .builder
                    .build_alloca(ty, dest)
                    .map_err(|e| format!("Alloca for binop failed: {:?}", e))?;
                self.builder
                    .build_store(temp, result)
                    .map_err(|e| format!("Store binop failed: {:?}", e))?;
                self.locals.insert(dest.clone(), temp);
            }

            IrInstruction::Call { dest, func, args } => {
                let callee = self
                    .functions
                    .get(func)
                    .or_else(|| self.module.get_function(func))
                    .ok_or_else(|| format!("Unknown function: {}", func))?;

                let mut llvm_args = Vec::new();
                for arg in args {
                    let val = self.resolve_value(arg)?;
                    llvm_args.push(val.into());
                }

                let call_site = self
                    .builder
                    .build_call(*callee, &llvm_args, "call")
                    .map_err(|e| format!("Call failed: {:?}", e))?;

                if let Some(dest_name) = dest {
                    if let Some(ret_val) = call_site.try_as_basic_value().left() {
                        let temp = self
                            .builder
                            .build_alloca(ret_val.get_type(), dest_name)
                            .map_err(|e| format!("Alloca for call result failed: {:?}", e))?;
                        self.builder
                            .build_store(temp, ret_val)
                            .map_err(|e| format!("Store call result failed: {:?}", e))?;
                        self.locals.insert(dest_name.clone(), temp);
                    }
                }
            }

            IrInstruction::GetField { dest, ptr, field } => {
                let ptr_val = self.get_local(ptr)?;
                let i32_type = self.context.i32_type();

                // Determine the struct type for this pointer.
                // Try to find it in our type registry; fall back to i8 only as last resort.
                let pointee_type: BasicTypeEnum<'ctx> = self
                    .types
                    .iter()
                    .find(|(name, _)| {
                        ptr.starts_with(name.as_str()) || ptr.ends_with(name.as_str())
                    })
                    .map(|(_, st)| (*st).into())
                    .unwrap_or_else(|| {
                        // If we can't resolve the struct type, try to infer from the first registered type
                        // or use a generic struct with enough fields
                        let fields = (*field as u32) + 1;
                        let field_types: Vec<BasicTypeEnum<'ctx>> = (0..fields)
                            .map(|_| self.context.i64_type().into())
                            .collect();
                        self.context.struct_type(&field_types, false).into()
                    });

                let gep = unsafe {
                    self.builder
                        .build_gep(
                            pointee_type,
                            ptr_val,
                            &[
                                i32_type.const_int(0, false),
                                i32_type.const_int(*field as u64, false),
                            ],
                            dest,
                        )
                        .map_err(|e| format!("GEP failed: {:?}", e))?
                };

                self.locals.insert(dest.clone(), gep);
            }
        }

        Ok(())
    }

    /// Generate integer binary operation
    fn generate_int_binop(
        &self,
        op: &str,
        lhs: IntValue<'ctx>,
        rhs: IntValue<'ctx>,
        name: &str,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        let result = match op {
            "add" => self.builder.build_int_add(lhs, rhs, name),
            "sub" => self.builder.build_int_sub(lhs, rhs, name),
            "mul" => self.builder.build_int_mul(lhs, rhs, name),
            "div" | "sdiv" => self.builder.build_int_signed_div(lhs, rhs, name),
            "udiv" => self.builder.build_int_unsigned_div(lhs, rhs, name),
            "rem" | "srem" => self.builder.build_int_signed_rem(lhs, rhs, name),
            "urem" => self.builder.build_int_unsigned_rem(lhs, rhs, name),
            "and" => self.builder.build_and(lhs, rhs, name),
            "or" => self.builder.build_or(lhs, rhs, name),
            "xor" => self.builder.build_xor(lhs, rhs, name),
            "shl" => self.builder.build_left_shift(lhs, rhs, name),
            "shr" | "ashr" => self.builder.build_right_shift(lhs, rhs, true, name),
            "lshr" => self.builder.build_right_shift(lhs, rhs, false, name),
            "eq" => self
                .builder
                .build_int_compare(IntPredicate::EQ, lhs, rhs, name),
            "ne" => self
                .builder
                .build_int_compare(IntPredicate::NE, lhs, rhs, name),
            "lt" | "slt" => self
                .builder
                .build_int_compare(IntPredicate::SLT, lhs, rhs, name),
            "le" | "sle" => self
                .builder
                .build_int_compare(IntPredicate::SLE, lhs, rhs, name),
            "gt" | "sgt" => self
                .builder
                .build_int_compare(IntPredicate::SGT, lhs, rhs, name),
            "ge" | "sge" => self
                .builder
                .build_int_compare(IntPredicate::SGE, lhs, rhs, name),
            "ult" => self
                .builder
                .build_int_compare(IntPredicate::ULT, lhs, rhs, name),
            "ule" => self
                .builder
                .build_int_compare(IntPredicate::ULE, lhs, rhs, name),
            "ugt" => self
                .builder
                .build_int_compare(IntPredicate::UGT, lhs, rhs, name),
            "uge" => self
                .builder
                .build_int_compare(IntPredicate::UGE, lhs, rhs, name),
            _ => return Err(format!("Unknown integer operation: {}", op)),
        };

        result
            .map(|v| v.into())
            .map_err(|e| format!("Int binop {} failed: {:?}", op, e))
    }

    /// Generate floating point binary operation
    fn generate_float_binop(
        &self,
        op: &str,
        lhs: FloatValue<'ctx>,
        rhs: FloatValue<'ctx>,
        name: &str,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        let result = match op {
            "fadd" | "add" => self.builder.build_float_add(lhs, rhs, name),
            "fsub" | "sub" => self.builder.build_float_sub(lhs, rhs, name),
            "fmul" | "mul" => self.builder.build_float_mul(lhs, rhs, name),
            "fdiv" | "div" => self.builder.build_float_div(lhs, rhs, name),
            "frem" | "rem" => self.builder.build_float_rem(lhs, rhs, name),
            "feq" | "oeq" => self
                .builder
                .build_float_compare(FloatPredicate::OEQ, lhs, rhs, name),
            "fne" | "one" => self
                .builder
                .build_float_compare(FloatPredicate::ONE, lhs, rhs, name),
            "flt" | "olt" => self
                .builder
                .build_float_compare(FloatPredicate::OLT, lhs, rhs, name),
            "fle" | "ole" => self
                .builder
                .build_float_compare(FloatPredicate::OLE, lhs, rhs, name),
            "fgt" | "ogt" => self
                .builder
                .build_float_compare(FloatPredicate::OGT, lhs, rhs, name),
            "fge" | "oge" => self
                .builder
                .build_float_compare(FloatPredicate::OGE, lhs, rhs, name),
            _ => return Err(format!("Unknown float operation: {}", op)),
        };

        result
            .map(|v| v.into())
            .map_err(|e| format!("Float binop {} failed: {:?}", op, e))
    }

    /// Generate a terminator instruction
    fn generate_terminator(&mut self, term: &IrTerminator) -> Result<(), String> {
        match term {
            IrTerminator::Return(Some(value)) => {
                let val = self.resolve_value(value)?;
                self.builder
                    .build_return(Some(&val))
                    .map_err(|e| format!("Return failed: {:?}", e))?;
            }

            IrTerminator::Return(None) => {
                self.builder
                    .build_return(None)
                    .map_err(|e| format!("Return void failed: {:?}", e))?;
            }

            IrTerminator::Branch(label) => {
                let target = *self
                    .current_blocks
                    .get(label)
                    .ok_or_else(|| format!("Unknown branch target: {}", label))?;
                self.builder
                    .build_unconditional_branch(target)
                    .map_err(|e| format!("Branch failed: {:?}", e))?;
            }

            IrTerminator::CondBranch {
                cond,
                then_label,
                else_label,
            } => {
                let cond_val = self.resolve_value(cond)?;
                let cond_int = cond_val.into_int_value();

                let then_block = *self
                    .current_blocks
                    .get(then_label)
                    .ok_or_else(|| format!("Unknown then block: {}", then_label))?;
                let else_block = *self
                    .current_blocks
                    .get(else_label)
                    .ok_or_else(|| format!("Unknown else block: {}", else_label))?;

                self.builder
                    .build_conditional_branch(cond_int, then_block, else_block)
                    .map_err(|e| format!("Cond branch failed: {:?}", e))?;
            }

            IrTerminator::Unreachable => {
                self.builder
                    .build_unreachable()
                    .map_err(|e| format!("Unreachable failed: {:?}", e))?;
            }
        }

        Ok(())
    }

    /// Get a local variable
    fn get_local(&self, name: &str) -> Result<PointerValue<'ctx>, String> {
        self.locals
            .get(name)
            .copied()
            .ok_or_else(|| format!("Unknown local: {}", name))
    }

    /// Resolve a value (could be a local, constant, or literal)
    fn resolve_value(&self, value: &str) -> Result<BasicValueEnum<'ctx>, String> {
        // Check if it's a local
        if let Some(&ptr) = self.locals.get(value) {
            // We need the type to load - assume i64 for now
            let loaded = self
                .builder
                .build_load(self.context.i64_type(), ptr, "load")
                .map_err(|e| format!("Load in resolve failed: {:?}", e))?;
            return Ok(loaded);
        }

        // Check if it's a function parameter
        if let Some(func) = self.current_function {
            for param in func.get_param_iter() {
                if param.get_name().to_str().unwrap_or("") == value {
                    return Ok(param);
                }
            }
        }

        // Try to parse as integer constant
        if let Ok(i) = value.parse::<i64>() {
            return Ok(self.context.i64_type().const_int(i as u64, true).into());
        }

        // Try to parse as float constant
        if let Ok(f) = value.parse::<f64>() {
            return Ok(self.context.f64_type().const_float(f).into());
        }

        // Check for true/false
        if value == "true" {
            return Ok(self.context.bool_type().const_int(1, false).into());
        }
        if value == "false" {
            return Ok(self.context.bool_type().const_int(0, false).into());
        }

        Err(format!("Cannot resolve value: {}", value))
    }

    /// Convert IR type to LLVM type
    fn ir_type_to_llvm(&self, ty: &IrType) -> Result<BasicTypeEnum<'ctx>, String> {
        match ty {
            IrType::Void => Err("Cannot convert void to basic type".to_string()),
            IrType::I8 => Ok(self.context.i8_type().into()),
            IrType::I16 => Ok(self.context.i16_type().into()),
            IrType::I32 => Ok(self.context.i32_type().into()),
            IrType::I64 => Ok(self.context.i64_type().into()),
            IrType::F32 => Ok(self.context.f32_type().into()),
            IrType::F64 => Ok(self.context.f64_type().into()),
            IrType::Bool => Ok(self.context.bool_type().into()),
            IrType::Ptr(_) => Ok(self
                .context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .into()),
            IrType::Array(elem, size) => {
                let elem_type = self.ir_type_to_llvm(elem)?;
                Ok(elem_type.array_type(*size as u32).into())
            }
            IrType::Struct(name) => {
                if let Some(&struct_type) = self.types.get(name) {
                    Ok(struct_type.into())
                } else {
                    // Create opaque struct
                    Ok(self.context.opaque_struct_type(name).into())
                }
            }
            IrType::FnPtr { params, ret } => {
                // Function pointers are represented as ptr
                Ok(self
                    .context
                    .i8_type()
                    .ptr_type(AddressSpace::default())
                    .into())
            }
        }
    }

    /// Convert IR function to LLVM function type
    fn ir_function_type(&self, func: &IrFunction) -> Result<FunctionType<'ctx>, String> {
        let mut param_types: Vec<BasicMetadataTypeEnum> = Vec::new();
        for (_, ty) in &func.params {
            let llvm_ty = self.ir_type_to_llvm(ty)?;
            param_types.push(llvm_ty.into());
        }

        match &func.return_type {
            IrType::Void => Ok(self.context.void_type().fn_type(&param_types, false)),
            ty => {
                let ret_ty = self.ir_type_to_llvm(ty)?;
                Ok(ret_ty.fn_type(&param_types, false))
            }
        }
    }

    /// Write LLVM IR to file
    pub fn write_ir(&self, path: &Path) -> Result<(), String> {
        self.module
            .print_to_file(path)
            .map_err(|e| format!("Failed to write IR: {}", e.to_string()))
    }

    /// Write bitcode to file
    pub fn write_bitcode(&self, path: &Path) -> bool {
        self.module.write_bitcode_to_path(path)
    }

    /// Compile to object file
    pub fn write_object(&self, path: &Path, opt_level: OptimizationLevel) -> Result<(), String> {
        // Initialize targets
        Target::initialize_native(&InitializationConfig::default())
            .map_err(|e| format!("Failed to initialize target: {}", e))?;

        let triple = TargetMachine::get_default_triple();
        let cpu = TargetMachine::get_host_cpu_name();
        let features = TargetMachine::get_host_cpu_features();

        let target = Target::from_triple(&triple)
            .map_err(|e| format!("Failed to get target: {}", e.to_string()))?;

        let target_machine = target
            .create_target_machine(
                &triple,
                cpu.to_str().unwrap_or("generic"),
                features.to_str().unwrap_or(""),
                opt_level,
                RelocMode::PIC,
                CodeModel::Default,
            )
            .ok_or("Failed to create target machine")?;

        // Run optimization passes
        self.run_optimizations(opt_level);

        target_machine
            .write_to_file(&self.module, FileType::Object, path)
            .map_err(|e| format!("Failed to write object: {}", e.to_string()))
    }

    /// Run LLVM optimization passes
    fn run_optimizations(&self, level: OptimizationLevel) {
        let pass_manager = PassManager::create(());

        match level {
            OptimizationLevel::None => {}
            OptimizationLevel::Less => {
                pass_manager.add_instruction_combining_pass();
                pass_manager.add_reassociate_pass();
                pass_manager.add_gvn_pass();
                pass_manager.add_cfg_simplification_pass();
            }
            OptimizationLevel::Default => {
                pass_manager.add_instruction_combining_pass();
                pass_manager.add_reassociate_pass();
                pass_manager.add_gvn_pass();
                pass_manager.add_cfg_simplification_pass();
                pass_manager.add_basic_alias_analysis_pass();
                pass_manager.add_promote_memory_to_register_pass();
                pass_manager.add_loop_vectorize_pass();
                pass_manager.add_slp_vectorize_pass();
            }
            OptimizationLevel::Aggressive => {
                pass_manager.add_instruction_combining_pass();
                pass_manager.add_reassociate_pass();
                pass_manager.add_gvn_pass();
                pass_manager.add_cfg_simplification_pass();
                pass_manager.add_basic_alias_analysis_pass();
                pass_manager.add_promote_memory_to_register_pass();
                pass_manager.add_loop_vectorize_pass();
                pass_manager.add_slp_vectorize_pass();
                pass_manager.add_tail_call_elimination_pass();
                pass_manager.add_function_inlining_pass();
                pass_manager.add_merged_load_store_motion_pass();
                pass_manager.add_aggressive_dce_pass();
            }
        }

        pass_manager.run_on(&self.module);
    }

    /// Link with external object files
    pub fn link_objects(
        output: &Path,
        objects: &[&Path],
        libraries: &[&str],
    ) -> Result<(), String> {
        use std::process::Command;

        #[cfg(target_os = "windows")]
        let linker = "link.exe";
        #[cfg(not(target_os = "windows"))]
        let linker = "ld";

        let mut cmd = Command::new(linker);

        #[cfg(target_os = "windows")]
        {
            cmd.arg("/OUT:").arg(output);
            for obj in objects {
                cmd.arg(obj);
            }
            for lib in libraries {
                cmd.arg(format!("{}.lib", lib));
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            cmd.arg("-o").arg(output);
            for obj in objects {
                cmd.arg(obj);
            }
            for lib in libraries {
                cmd.arg("-l").arg(lib);
            }
        }

        let status = cmd
            .status()
            .map_err(|e| format!("Failed to run linker: {}", e))?;

        if status.success() {
            Ok(())
        } else {
            Err(format!("Linker failed with status: {}", status))
        }
    }
}

/// Convenience function to generate LLVM code from IR
pub fn generate_llvm(ir: &IrModule, output: &Path, opt_level: u8) -> Result<(), String> {
    let context = Context::create();
    let mut codegen = LlvmCodegen::new(&context, &ir.name);

    codegen.generate(ir)?;

    let opt = match opt_level {
        0 => OptimizationLevel::None,
        1 => OptimizationLevel::Less,
        2 => OptimizationLevel::Default,
        _ => OptimizationLevel::Aggressive,
    };

    // Write IR file
    codegen.write_ir(&output.with_extension("ll"))?;

    // Write object file
    codegen.write_object(&output.with_extension("o"), opt)?;

    info!("Generated object file: {:?}", output.with_extension("o"));
    Ok(())
}
