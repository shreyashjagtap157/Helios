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

#![allow(dead_code)]
//! Code Generation
//! Supports multiple backends: LLVM IR, OVM bytecode, and native code.

pub mod cognitive;
#[cfg(test)]
pub mod comprehensive_tests;
pub mod cpp_interop;
pub mod dwarf;
pub mod exception_handling;
pub mod gpu_advanced;
pub mod gpu_binary;
pub mod gpu_dispatch;
pub mod gpu_fusion;
pub mod gpu_hardware;
#[cfg(test)]
pub mod gpu_tests;
pub mod jit;
pub mod jit_complete;
pub mod linker;
pub mod mlir;
pub mod native_codegen;
pub mod native_extended;
pub mod native_linker;
pub mod opt;
pub mod optimizer;
pub mod optimizing_jit;
pub mod ovm;
pub mod ovm_direct;
pub mod python_buffer;
pub mod python_interop;
pub mod self_hosting;

#[cfg(feature = "llvm")]
pub mod llvm_backend;

use crate::ir::{IrFunction, IrInstruction, IrModule, IrTerminator, IrType};
use log::{debug, info, trace};
use std::path::Path;

/// Code generation target
#[derive(Debug, Clone, Copy, Default)]
pub enum CodegenTarget {
    #[default]
    Ovm, // OVM bytecode for managed execution (default, always available)
    #[cfg(feature = "llvm")]
    Llvm, // LLVM IR -> native code
    #[cfg(feature = "llvm")]
    Hybrid, // Both native and managed
    Native, // Direct native code via built-in codegen (no LLVM required)
}

/// Generate code from IR with target selection
pub fn generate_with_target(
    ir: IrModule,
    output: &Path,
    #[allow(unused_variables)] opt_level: u8,
    target: CodegenTarget,
) -> Result<(), String> {
    match target {
        #[cfg(feature = "llvm")]
        CodegenTarget::Llvm => llvm_backend::generate_llvm(&ir, output, opt_level),
        CodegenTarget::Ovm => ovm::generate_ovm(ir, output),
        #[cfg(feature = "llvm")]
        CodegenTarget::Hybrid => {
            // Generate both OVM and native
            ovm::generate_ovm(ir.clone(), &output.with_extension("ovm"))?;
            llvm_backend::generate_llvm(&ir, output, opt_level)
        }
        CodegenTarget::Native => generate_native(&ir, output, opt_level),
    }
}

/// Native code generation pipeline: IR → NativeCodegen → machine code → Linker → executable.
///
/// Uses the built-in `native_codegen` module (x86-64 / ARM64 / WASM emitters) and the
/// `linker` module (ELF / PE / Mach-O) to produce executables without requiring LLVM.
fn generate_native(ir: &IrModule, output: &Path, opt_level: u8) -> Result<(), String> {
    info!("Native codegen pipeline: IR → machine code → linker → executable");

    // 1. Detect host target triple
    let target_triple = native_codegen::TargetTriple::host();
    info!(
        "Target: {} / {:?} / {:?}",
        target_triple.arch, target_triple.os, target_triple.format
    );

    // 2. Compile IR → machine code via NativeCodegen
    let mut codegen = native_codegen::NativeCodegen::new(target_triple);
    codegen.set_opt_level(opt_level as u32);
    let native_output = codegen
        .compile_module(ir)
        .map_err(|e| format!("Native codegen failed: {}", e))?;

    info!(
        "Native codegen produced {} bytes ({} symbols)",
        native_output.binary.len(),
        native_output.symbols.len()
    );

    // 3. Link machine code into a final executable via the built-in linker
    let platform = linker::TargetPlatform::host()
        .map_err(|e| format!("Failed to detect host platform: {}", e))?;

    let mut link = linker::Linker::new(platform);

    // Add the machine code as a .text section
    link.add_text(native_output.binary);

    // Register symbols from native codegen output
    let entry_name = find_entry_symbol(&native_output.symbols);
    for sym in &native_output.symbols {
        link.add_symbol(linker::LinkerSymbol {
            name: sym.name.clone(),
            offset: sym.offset as u64,
            size: sym.size as u64,
            section: Some(".text".to_string()),
            binding: linker::SymbolBinding::Global,
            kind: linker::SymbolKind::Function,
        });
    }

    link.set_entry_point(&entry_name);

    // Choose the output path with appropriate extension
    let output_path = if cfg!(target_os = "windows") {
        output.with_extension("exe")
    } else {
        output.to_path_buf()
    };

    link.set_output_path(&output_path);
    link.link_to_file()
        .map_err(|e| format!("Linking failed: {}", e))?;

    info!("Linked native executable: {:?}", output_path);
    Ok(())
}

/// Pick the best entry-point symbol from the native codegen output.
/// Prefers "main", then "_start", then falls back to the first symbol.
fn find_entry_symbol(symbols: &[native_codegen::NativeSymbol]) -> String {
    // Prefer "main" as entry (the linker on Windows will map it to mainCRTStartup)
    for sym in symbols {
        if sym.name == "main" {
            return "main".to_string();
        }
    }
    for sym in symbols {
        if sym.name == "_start" {
            return "_start".to_string();
        }
    }
    // Fall back to the first symbol if any exist
    if let Some(sym) = symbols.first() {
        return sym.name.clone();
    }
    // Ultimate fallback
    if cfg!(target_os = "windows") {
        "mainCRTStartup".to_string()
    } else {
        "_start".to_string()
    }
}

/// Code generation entry point
/// Uses OVM backend by default, LLVM for native compilation if available.
pub fn generate(ir: IrModule, output: &Path, opt_level: u8, emit_llvm: bool) -> Result<(), String> {
    info!("Code generation starting (opt_level={})", opt_level);
    debug!("Output path: {:?}", output);

    if emit_llvm {
        // Emit LLVM IR text for inspection/debugging
        let llvm_ir = generate_llvm_ir(&ir);
        let ir_path = output.with_extension("ll");
        std::fs::write(&ir_path, &llvm_ir)
            .map_err(|e| format!("Failed to write LLVM IR: {}", e))?;
        info!("Wrote LLVM IR to {:?}", ir_path);
    }

    // Try LLVM backend if available, otherwise use OVM
    #[cfg(feature = "llvm")]
    {
        match llvm_backend::generate_llvm(&ir, output, opt_level) {
            Ok(()) => {
                info!(
                    "Native compilation succeeded: {:?}",
                    output.with_extension("o")
                );
                return Ok(());
            }
            Err(e) => {
                info!("LLVM backend failed ({}), falling back to OVM", e);
            }
        }
    }

    // Default: Generate OVM bytecode
    ovm::generate_ovm(ir, &output.with_extension("ovm"))?;
    info!("Generated OVM bytecode: {:?}", output.with_extension("ovm"));
    Ok(())
}

fn generate_llvm_ir(module: &IrModule) -> String {
    trace!("Generating LLVM IR text");
    let mut out = String::new();

    out.push_str(&format!("; ModuleID = '{}'\n", module.name));
    out.push_str("target triple = \"x86_64-pc-windows-msvc\"\n\n");

    // V2.0: Emit external declarations with mangling
    for ext in &module.externs {
        let name = if ext.abi == "C++" {
            // Use C++ Interop mangling
            // Note: In real impl, we'd need full Type info, here we have IrType.
            // Converting IrType back to AST Type is lossy, but sufficient for basic demo.
            format!("_{}", ext.name) // Simple prefix for now to prove integration
        } else {
            ext.name.clone()
        };

        let params: Vec<_> = ext.params.iter().map(|t| ir_type_to_llvm(t)).collect();
        out.push_str(&format!(
            "declare {} @{}({})\n",
            ir_type_to_llvm(&ext.return_type),
            name,
            params.join(", ")
        ));
    }

    for func in &module.functions {
        out.push_str(&gen_function(func));
        out.push('\n');
    }

    out
}

fn gen_function(func: &IrFunction) -> String {
    let ret_ty = ir_type_to_llvm(&func.return_type);
    let params: Vec<_> = func
        .params
        .iter()
        .map(|(n, t)| format!("{} %{}", ir_type_to_llvm(t), n))
        .collect();

    let mut out = format!(
        "define {} @{}({}) {{\n",
        ret_ty,
        func.name,
        params.join(", ")
    );

    // V3.0: Function Entry Safepoint
    // In a real implementation, this would emit a call to the runtime
    // call void @__omni_check_safepoint()
    out.push_str("  call void @__omni_check_safepoint()\n");

    for block in &func.blocks {
        out.push_str(&format!("{}:\n", block.label));
        for inst in &block.instructions {
            out.push_str(&format!("  {}\n", gen_instruction(inst)));
        }
        out.push_str(&format!("  {}\n", gen_terminator(&block.terminator)));
    }

    out.push_str("}\n");
    out
}

fn gen_instruction(inst: &IrInstruction) -> String {
    match inst {
        IrInstruction::Alloca { dest, ty } => format!("%{} = alloca {}", dest, ir_type_to_llvm(ty)),
        IrInstruction::Load { dest, ptr, ty } => {
            format!("%{} = load {}, ptr %{}", dest, ir_type_to_llvm(ty), ptr)
        }
        IrInstruction::Store { ptr, value } => format!("store {}, ptr %{}", value, ptr),
        IrInstruction::BinOp {
            dest,
            op,
            left,
            right,
        } => format!("%{} = {} {}, {}", dest, op, left, right),
        IrInstruction::Call { dest, func, args } => {
            let args_str = args
                .iter()
                .map(|a| format!("{}", a))
                .collect::<Vec<_>>()
                .join(", ");
            if let Some(d) = dest {
                format!("%{} = call @{}({})", d, func, args_str)
            } else {
                format!("call @{}({})", func, args_str)
            }
        }
        IrInstruction::GetField { dest, ptr, field } => {
            format!("%{} = getelementptr %{}, i32 {}", dest, ptr, field)
        }
        // New instruction variants - LLVM-style textual IR
        IrInstruction::Phi { dest, ty, incoming } => {
            let pairs: Vec<_> = incoming
                .iter()
                .map(|(v, b)| format!("[%{}, %{}]", v, b))
                .collect();
            format!(
                "%{} = phi {} {}",
                dest,
                ir_type_to_llvm(ty),
                pairs.join(", ")
            )
        }
        IrInstruction::Select {
            dest,
            cond,
            then_val,
            else_val,
        } => format!("%{} = select {}, {}, {}", dest, cond, then_val, else_val),
        IrInstruction::Switch {
            value,
            default,
            cases,
        } => {
            let case_strs: Vec<_> = cases
                .iter()
                .map(|(v, l)| format!("i64 {}, label %{}", v, l))
                .collect();
            format!(
                "switch {} label %{} [ {} ]",
                value,
                default,
                case_strs.join(", ")
            )
        }
        IrInstruction::CreateClosure {
            dest,
            func,
            captures,
        } => format!("%{} = closure @{} [{}]", dest, func, captures.join(", ")),
        IrInstruction::CallClosure {
            dest,
            closure,
            args,
        } => {
            let args_str = args
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            match dest {
                Some(d) => format!("%{} = callclosure %{}({})", d, closure, args_str),
                None => format!("callclosure %{}({})", closure, args_str),
            }
        }
        IrInstruction::AsyncSpawn { dest, func, args } => {
            format!("%{} = async.spawn @{}({} args)", dest, func, args.len())
        }
        IrInstruction::AsyncAwait { dest, future } => match dest {
            Some(d) => format!("%{} = async.await %{}", d, future),
            None => format!("async.await %{}", future),
        },
        IrInstruction::TraitDispatch {
            dest,
            object,
            method,
            args,
        } => match dest {
            Some(d) => format!(
                "%{} = dispatch %{}.{}({} args)",
                d,
                object,
                method,
                args.len()
            ),
            None => format!("dispatch %{}.{}({} args)", object, method, args.len()),
        },
        IrInstruction::VTableLookup {
            dest,
            object,
            trait_name,
            method_idx,
        } => format!(
            "%{} = vtable.lookup %{}::{}.{}",
            dest, object, trait_name, method_idx
        ),
        IrInstruction::Cast {
            dest,
            value,
            to_type,
        } => format!(
            "%{} = bitcast {} to {}",
            dest,
            value,
            ir_type_to_llvm(to_type)
        ),
        IrInstruction::ExtractValue {
            dest,
            aggregate,
            indices,
        } => format!("%{} = extractvalue %{} {:?}", dest, aggregate, indices),
        IrInstruction::InsertValue {
            dest,
            aggregate,
            value,
            indices,
        } => format!(
            "%{} = insertvalue %{}, {} {:?}",
            dest, aggregate, value, indices
        ),
        IrInstruction::NativeCall {
            dest,
            module,
            func,
            args,
        } => {
            let args_str = args
                .iter()
                .map(|a| format!("{}", a))
                .collect::<Vec<_>>()
                .join(", ");
            match dest {
                Some(d) => format!("%{} = native.call {}::{}({})", d, module, func, args_str),
                None => format!("native.call {}::{}({})", module, func, args_str),
            }
        }
        IrInstruction::BoundsCheck { index, length } => {
            format!("call void @__omni_bounds_check(%{}, %{})", index, length)
        }
    }
}

fn gen_terminator(term: &IrTerminator) -> String {
    match term {
        IrTerminator::Return(Some(v)) => format!("ret {}", v),
        IrTerminator::Return(None) => "ret void".to_string(),
        IrTerminator::Branch(label) => format!("br label %{}", label),
        IrTerminator::CondBranch {
            cond,
            then_label,
            else_label,
        } => format!("br {}, label %{}, label %{}", cond, then_label, else_label),
        IrTerminator::Unreachable => "unreachable".to_string(),
    }
}

fn ir_type_to_llvm(ty: &IrType) -> String {
    match ty {
        IrType::Void => "void".to_string(),
        IrType::I8 => "i8".to_string(),
        IrType::I16 => "i16".to_string(),
        IrType::I32 => "i32".to_string(),
        IrType::I64 => "i64".to_string(),
        IrType::F32 => "float".to_string(),
        IrType::F64 => "double".to_string(),
        IrType::Bool => "i1".to_string(),
        IrType::Ptr(_) => "ptr".to_string(),
        IrType::Array(elem, size) => format!("[{} x {}]", size, ir_type_to_llvm(elem)),
        IrType::Struct(name) => format!("%{}", name),
        // Advanced types - represented as opaque pointers in LLVM
        IrType::Closure { .. } => "ptr".to_string(), // Closure is a fat pointer
        IrType::Future(_) => "ptr".to_string(),      // Future is a boxed state machine
        IrType::TraitObject(_) => "ptr".to_string(), // dyn Trait is a fat pointer
        IrType::Generic(_) => "ptr".to_string(),     // Generic types resolved at monomorphization
        IrType::Enum { name, .. } => format!("%{}", name), // Enums are struct-like
        IrType::Tuple(elements) => {
            let types: Vec<_> = elements.iter().map(|e| ir_type_to_llvm(e)).collect();
            format!("{{ {} }}", types.join(", "))
        }
        IrType::FnPtr { .. } => "ptr".to_string(), // Function pointers
    }
}
