// LLVM Code Generation for Omni
// Converts IR to LLVM IR and native executables

use crate::ir::*;
use crate::ast::Type;
use std::fs::File;
use std::io::Write;
use std::process::Command;

pub struct LLVMCodegen {
    output: String,
    var_counter: usize,
}

impl LLVMCodegen {
    pub fn new() -> Self {
        LLVMCodegen {
            output: String::new(),
            var_counter: 0,
        }
    }

    pub fn generate(&mut self, module: &IRModule, output_name: &str) -> Result<(), String> {
        self.output.clear();
        self.var_counter = 0;

        // LLVM header
        self.emit("; Omni Program");
        self.emit("target triple = \"x86_64-pc-linux-gnu\"");
        self.emit("target datalayout = \"e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128\"");
        self.emit("");

        // Declare external C functions
        self.emit("declare i32 @puts(i8* %str)");
        self.emit("declare i32 @printf(i8* %fmt, ...)");
        self.emit("declare i8* @malloc(i64)");
        self.emit("declare void @free(i8*)");
        self.emit("");

        // Global string constants
        let mut str_counter = 0;
        for global in &module.globals {
            match &global.var_type {
                Type::String => {
                    self.emit(&format!(
                        "@str.{} = private constant [{}x i8] c\"{}\", align 1",
                        str_counter,
                        global.name.len() + 1,
                        global.name
                    ));
                    str_counter += 1;
                }
                _ => {}
            }
        }
        self.emit("");

        // Generate functions
        for func in &module.functions {
            self.generate_function(func)?;
        }

        // Generate main if not defined
        if !module
            .functions
            .iter()
            .any(|f| f.name == "main")
        {
            self.emit_main();
        }

        // Write LLVM IR to file
        let ll_file = format!("{}.ll", output_name);
        let mut file = File::create(&ll_file)
            .map_err(|e| format!("Failed to create {}: {}", ll_file, e))?;
        file.write_all(self.output.as_bytes())
            .map_err(|e| format!("Failed to write {}: {}", ll_file, e))?;

        // Compile LLVM to object file
        self.compile_llvm_to_object(&ll_file, output_name)?;

        // Link
        self.link_object_to_binary(output_name)?;

        println!("Generated executable: {}", output_name);
        Ok(())
    }

    fn generate_function(&mut self, func: &IRFunction) -> Result<(), String> {
        let return_type = self.type_to_llvm(&func.return_type);
        let mut param_types = String::new();
        for (_, param_type) in &func.args {
            if !param_types.is_empty() {
                param_types.push_str(", ");
            }
            param_types.push_str(&self.type_to_llvm(param_type));
        }

        self.emit(&format!(
            "define {} @{}({}) {{",
            return_type, func.name, param_types
        ));

        for block in &func.blocks {
            self.emit(&format!("{}", block.label));
            for instr in &block.instructions {
                self.generate_instruction(instr)?;
            }

            if let Some(term) = &block.terminator {
                self.generate_terminator(term)?;
            }
        }

        self.emit("}");
        self.emit("");
        Ok(())
    }

    fn generate_instruction(&mut self, instr: &IRInstruction) -> Result<(), String> {
        match instr {
            IRInstruction::Alloca(dest, typ) => {
                let llvm_type = self.type_to_llvm(typ);
                self.emit(&format!("  {} = alloca {}", dest, llvm_type));
            }
            IRInstruction::Store(dest, src) => {
                let src_str = self.value_to_llvm(src);
                self.emit(&format!("  store {} {}, {}* {}", 
                    self.ir_value_type(src), 
                    src_str,
                    self.ir_value_type(src),
                    dest
                ));
            }
            IRInstruction::Load(dest, typ) => {
                let llvm_type = self.type_to_llvm(typ);
                self.emit(&format!("  {} = load {}, {}* %tmp", dest, llvm_type, llvm_type));
            }
            IRInstruction::BinaryOp(dest, left, op, right) => {
                let op_str = match op {
                    crate::ast::BinaryOp::Add => "add",
                    crate::ast::BinaryOp::Subtract => "sub",
                    crate::ast::BinaryOp::Multiply => "mul",
                    crate::ast::BinaryOp::Divide => "sdiv",
                    crate::ast::BinaryOp::Modulo => "srem",
                    crate::ast::BinaryOp::Equal => "eq",
                    crate::ast::BinaryOp::NotEqual => "ne",
                    crate::ast::BinaryOp::Less => "slt",
                    crate::ast::BinaryOp::LessEqual => "sle",
                    crate::ast::BinaryOp::Greater => "sgt",
                    crate::ast::BinaryOp::GreaterEqual => "sge",
                    crate::ast::BinaryOp::And => "and",
                    crate::ast::BinaryOp::Or => "or",
                };
                let left_str = self.value_to_llvm(left);
                let right_str = self.value_to_llvm(right);
                self.emit(&format!(
                    "  {} = {} i64 {}, {}",
                    dest, op_str, left_str, right_str
                ));
            }
            IRInstruction::Call(dest, func_name, args) => {
                let mut args_str = String::new();
                for arg in args {
                    if !args_str.is_empty() {
                        args_str.push_str(", ");
                    }
                    args_str.push_str(&format!(
                        "{} {}",
                        self.ir_value_type(arg),
                        self.value_to_llvm(arg)
                    ));
                }

                self.emit(&format!(
                    "  {} = call i32 @{}({})",
                    dest, func_name, args_str
                ));
            }
            IRInstruction::Const(dest, _typ, val) => {
                self.emit(&format!("  {} = {}", dest, val));
            }
        }
        Ok(())
    }

    fn generate_terminator(&mut self, term: &Terminator) -> Result<(), String> {
        match term {
            Terminator::Return(val) => {
                if let Some(v) = val {
                    let val_str = self.value_to_llvm(v);
                    self.emit(&format!("  ret i64 {}", val_str));
                } else {
                    self.emit("  ret void");
                }
            }
            Terminator::Br(label) => {
                self.emit(&format!("  br label %{}", label));
            }
            Terminator::CondBr(cond, then_label, else_label) => {
                let cond_str = self.value_to_llvm(cond);
                self.emit(&format!(
                    "  br i1 {}, label %{}, label %{}",
                    cond_str, then_label, else_label
                ));
            }
        }
        Ok(())
    }

    fn emit_main(&mut self) {
        self.emit("define i32 @main() {");
        self.emit("entry:");
        self.emit("  ret i32 0");
        self.emit("}");
        self.emit("");
    }

    fn type_to_llvm(&self, typ: &Type) -> String {
        match typ {
            Type::I32 => "i32".to_string(),
            Type::I64 => "i64".to_string(),
            Type::F64 => "double".to_string(),
            Type::Bool => "i1".to_string(),
            Type::String => "i8*".to_string(),
            Type::Void => "void".to_string(),
            Type::Custom(name) => format!("%struct.{}", name),
            Type::Array(inner, size) => {
                format!("[{} x {}]", size, self.type_to_llvm(inner))
            }
        }
    }

    fn value_to_llvm(&self, val: &IRValue) -> String {
        match val {
            IRValue::Register(name) => name.clone(),
            IRValue::Constant(typ, value) => {
                if matches!(typ, Type::String) {
                    value.clone()
                } else {
                    value.clone()
                }
            }
            IRValue::Global(name) => format!("@{}", name),
        }
    }

    fn ir_value_type(&self, val: &IRValue) -> String {
        match val {
            IRValue::Constant(typ, _) => self.type_to_llvm(typ),
            _ => "i64".to_string(),
        }
    }

    fn emit(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }

    fn compile_llvm_to_object(&self, ll_file: &str, output_name: &str) -> Result<(), String> {
        let obj_file = format!("{}.o", output_name);
        
        let output = Command::new("llc")
            .args(&["-filetype=obj", "-o", &obj_file, ll_file])
            .output()
            .map_err(|e| format!("Failed to run llc: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "llc compilation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    fn link_object_to_binary(&self, output_name: &str) -> Result<(), String> {
        let obj_file = format!("{}.o", output_name);
        
        let output = Command::new("ld")
            .args(&[
                "-dynamic-linker",
                "/lib64/ld-linux-x86-64.so.2",
                "-lc",
                "-o",
                output_name,
                &obj_file,
            ])
            .output()
            .map_err(|e| format!("Failed to run ld: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Linking failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }
}
