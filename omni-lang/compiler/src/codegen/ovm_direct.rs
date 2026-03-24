use crate::codegen::ovm::{
    serialize_module, LoopContext, OvmCodegen, OvmConstant, OvmFunction, OvmModule, OvmOpcode,
};
use crate::parser::ast::{BinaryOp, Literal, Type as AstType, UnaryOp};
use crate::semantic::{
    TypedExpr, TypedExprKind, TypedFunction, TypedItem, TypedModule, TypedStatement,
};

/// Direct OVM bytecode generation from Typed AST.
/// Bypasses the IR entirely for clean stack-based codegen.
/// Each expression pushes exactly one value to the stack.
/// Each statement consumes its inputs and produces no stack residue.

pub fn generate_ovm_direct(module: &TypedModule, output: &std::path::Path) -> Result<(), String> {
    let mut codegen = OvmCodegen::new();
    let mut functions = Vec::new();

    // Index functions by name
    for (i, item) in module.items.iter().enumerate() {
        if let TypedItem::Function(f) = item {
            codegen.func_indices.insert(f.name.clone(), i as u32);
        }
    }

    for item in &module.items {
        if let TypedItem::Function(f) = item {
            functions.push(compile_func(&mut codegen, f)?);
        }
    }

    let ovm_module = OvmModule {
        name: "main".to_string(),
        version: 1,
        entry_point: functions.iter().position(|f| f.name == "main").unwrap_or(0) as u32,
        constants: codegen.constants,
        functions,
        types: Vec::new(),
        symbols: Vec::new(),
    };

    let bytecode = serialize_module(&ovm_module)?;
    std::fs::write(output.with_extension("ovm"), &bytecode)
        .map_err(|e| format!("Failed to write OVM: {}", e))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Helper: patch a relative i32 jump at the given bytecode offset
// ---------------------------------------------------------------------------

fn patch_jump(codegen: &mut OvmCodegen, offset: usize) {
    let current = codegen.current_bytecode.len() as i32;
    let jump = current - offset as i32 - 4;
    let bytes = jump.to_le_bytes();
    codegen.current_bytecode[offset..offset + 4].copy_from_slice(&bytes);
}

fn alloc_local(codegen: &mut OvmCodegen, name: &str) -> u16 {
    if let Some(&idx) = codegen.local_indices.get(name) {
        return idx;
    }
    let idx = codegen.local_indices.len() as u16;
    codegen.local_indices.insert(name.to_string(), idx);
    idx
}

// ---------------------------------------------------------------------------
// Compile a function
// ---------------------------------------------------------------------------

fn compile_func(codegen: &mut OvmCodegen, func: &TypedFunction) -> Result<OvmFunction, String> {
    codegen.current_bytecode.clear();
    codegen.local_indices.clear();

    // Index parameters
    for (i, (name, _)) in func.params.iter().enumerate() {
        codegen.local_indices.insert(name.clone(), i as u16);
    }

    // Compile body
    for stmt in &func.body {
        compile_stmt(codegen, stmt)?;
    }

    // Ensure function ends with a return if the last statement isn't one
    if !func
        .body
        .iter()
        .any(|s| matches!(s, TypedStatement::Return(_)))
    {
        codegen.emit(OvmOpcode::RetVoid);
    }

    let name_idx = codegen.add_constant(OvmConstant::String(func.name.clone()));
    Ok(OvmFunction {
        name: func.name.clone(),
        name_index: name_idx,
        param_count: func.params.len() as u16,
        local_count: codegen.local_indices.len() as u16,
        max_stack: 256,
        bytecode: codegen.current_bytecode.clone(),
        is_native: false,
    })
}

// ---------------------------------------------------------------------------
// Compile statements
// ---------------------------------------------------------------------------

fn compile_stmt(codegen: &mut OvmCodegen, stmt: &TypedStatement) -> Result<(), String> {
    match stmt {
        TypedStatement::Let { name, value, .. } => {
            compile_expr(codegen, value)?;
            let idx = alloc_local(codegen, name);
            codegen.emit(OvmOpcode::StoreLoc);
            codegen.emit_u16(idx);
        }

        TypedStatement::Return(expr) => {
            if let Some(e) = expr {
                compile_expr(codegen, e)?;
                codegen.emit(OvmOpcode::Ret);
            } else {
                codegen.emit(OvmOpcode::RetVoid);
            }
        }

        TypedStatement::Expr(expr) => {
            compile_expr(codegen, expr)?;
            codegen.emit(OvmOpcode::Pop);
        }

        TypedStatement::Assignment { target, value, .. } => {
            compile_expr(codegen, value)?;
            match &target.kind {
                TypedExprKind::Identifier(name) => {
                    let idx = alloc_local(codegen, name);
                    codegen.emit(OvmOpcode::StoreLoc);
                    codegen.emit_u16(idx);
                }
                TypedExprKind::Field(obj, field_name) => {
                    // Stack: [value] — need [obj, value]
                    // Load the object first
                    compile_expr(codegen, obj)?;
                    codegen.emit(OvmOpcode::Swap);
                    // Stack: [obj, value]
                    let field_idx = codegen.add_constant(OvmConstant::String(field_name.clone()));
                    codegen.emit(OvmOpcode::SetField);
                    codegen.emit_u32(field_idx);
                }
                TypedExprKind::Index(array, index) => {
                    // Stack: [value] — need [array, index, value]
                    compile_expr(codegen, array)?;
                    // Stack: [value, array]
                    codegen.emit(OvmOpcode::Swap);
                    // Stack: [array, value]
                    compile_expr(codegen, index)?;
                    // Stack: [array, value, index]
                    codegen.emit(OvmOpcode::Swap);
                    // Stack: [array, index, value]
                    codegen.emit(OvmOpcode::ArraySet);
                }
                _ => {
                    // Unknown target, just pop the value
                    codegen.emit(OvmOpcode::Pop);
                }
            }
        }

        // --- Control flow: If ---
        TypedStatement::If {
            condition,
            then_block,
            else_block,
        } => {
            compile_expr(codegen, condition)?;
            // Jz to else (or end if no else)
            codegen.emit(OvmOpcode::Jz);
            let jz_pos = codegen.current_bytecode.len();
            codegen.emit_i32(0); // placeholder

            // Then block
            for s in then_block {
                compile_stmt(codegen, s)?;
            }

            if let Some(else_stmts) = else_block {
                // Jump over else block
                codegen.emit(OvmOpcode::Jmp);
                let jmp_pos = codegen.current_bytecode.len();
                codegen.emit_i32(0); // placeholder

                // Now current_bytecode.len() is the start of the else block
                let else_start = codegen.current_bytecode.len() as i32;
                // Patch the Jz to jump to the else block
                let jz_jump = else_start - jz_pos as i32 - 4;
                let bytes = jz_jump.to_le_bytes();
                codegen.current_bytecode[jz_pos..jz_pos + 4].copy_from_slice(&bytes);

                // Else block
                for s in else_stmts {
                    compile_stmt(codegen, s)?;
                }

                // Patch the Jmp to jump here (end of if)
                patch_jump(codegen, jmp_pos);
            } else {
                // Patch the Jz to jump here (end of if, no else)
                patch_jump(codegen, jz_pos);
            }
        }

        // --- Control flow: While ---
        TypedStatement::While { condition, body } => {
            let loop_start = codegen.current_bytecode.len();

            // Push loop context for break/continue
            codegen.loop_contexts.push(LoopContext {
                break_addrs: Vec::new(),
                continue_target: loop_start, // continue re-evaluates condition
            });

            // Compile condition
            compile_expr(codegen, condition)?;
            codegen.emit(OvmOpcode::Jz);
            let jz_pos = codegen.current_bytecode.len();
            codegen.emit_i32(0); // placeholder — break target

            // Compile body
            for s in body {
                compile_stmt(codegen, s)?;
            }

            // Jump back to condition
            codegen.emit(OvmOpcode::Jmp);
            let jump_back = (loop_start as i32) - (codegen.current_bytecode.len() as i32) - 4;
            codegen.emit_i32(jump_back);

            // Patch the condition's Jz to jump here (after the loop)
            patch_jump(codegen, jz_pos);

            // Patch all break jumps to here
            let ctx = codegen.loop_contexts.pop().unwrap();
            let end_pos = codegen.current_bytecode.len();
            for break_addr in ctx.break_addrs {
                let jump = (end_pos as i32) - (break_addr as i32) - 4;
                let bytes = jump.to_le_bytes();
                codegen.current_bytecode[break_addr..break_addr + 4].copy_from_slice(&bytes);
            }
        }

        // --- Control flow: For ---
        // Desugar: for var in iter { body } → iterator protocol
        // For simplicity, we treat `iter` as a range/collection and use a counter
        TypedStatement::For { var, iter, body } => {
            // We'll desugar for into:
            //   let _iter = iter;  (evaluate once)
            //   let _idx = 0;
            //   loop {
            //     if _idx >= len(_iter) { break; }
            //     let var = _iter[_idx];
            //     body...
            //     _idx = _idx + 1;
            //   }

            let iter_local = alloc_local(codegen, &format!("$for_iter_{}", var));
            let idx_local = alloc_local(codegen, &format!("$for_idx_{}", var));

            // Evaluate iterator and store
            compile_expr(codegen, iter)?;
            codegen.emit(OvmOpcode::StoreLoc);
            codegen.emit_u16(iter_local);

            // Initialize index to 0
            codegen.emit(OvmOpcode::PushI64);
            codegen.emit_u64(0);
            codegen.emit(OvmOpcode::StoreLoc);
            codegen.emit_u16(idx_local);

            let loop_start = codegen.current_bytecode.len();

            // Push loop context
            codegen.loop_contexts.push(LoopContext {
                break_addrs: Vec::new(),
                continue_target: loop_start,
            });

            // Check: if idx >= len(iter) { break; }
            codegen.emit(OvmOpcode::LoadLoc);
            codegen.emit_u16(idx_local);
            codegen.emit(OvmOpcode::LoadLoc);
            codegen.emit_u16(iter_local);
            codegen.emit(OvmOpcode::ArrayLen);
            codegen.emit(OvmOpcode::Ge);
            codegen.emit(OvmOpcode::Jnz);
            let break_pos = codegen.current_bytecode.len();
            codegen.emit_i32(0); // placeholder

            // let var = iter[idx]
            codegen.emit(OvmOpcode::LoadLoc);
            codegen.emit_u16(iter_local);
            codegen.emit(OvmOpcode::LoadLoc);
            codegen.emit_u16(idx_local);
            codegen.emit(OvmOpcode::ArrayGet);
            let var_idx = alloc_local(codegen, var);
            codegen.emit(OvmOpcode::StoreLoc);
            codegen.emit_u16(var_idx);

            // Body
            for s in body {
                compile_stmt(codegen, s)?;
            }

            // idx = idx + 1
            codegen.emit(OvmOpcode::LoadLoc);
            codegen.emit_u16(idx_local);
            codegen.emit(OvmOpcode::PushI64);
            codegen.emit_u64(1);
            codegen.emit(OvmOpcode::AddI64);
            codegen.emit(OvmOpcode::StoreLoc);
            codegen.emit_u16(idx_local);

            // Jump back to loop start
            codegen.emit(OvmOpcode::Jmp);
            let jump_back = (loop_start as i32) - (codegen.current_bytecode.len() as i32) - 4;
            codegen.emit_i32(jump_back);

            // Patch the break-on-condition jump
            patch_jump(codegen, break_pos);

            // Patch all break jumps
            let ctx = codegen.loop_contexts.pop().unwrap();
            let end_pos = codegen.current_bytecode.len();
            for break_addr in ctx.break_addrs {
                let jump = (end_pos as i32) - (break_addr as i32) - 4;
                let bytes = jump.to_le_bytes();
                codegen.current_bytecode[break_addr..break_addr + 4].copy_from_slice(&bytes);
            }
        }

        // --- Control flow: Loop (infinite) ---
        TypedStatement::Loop { body } => {
            let loop_start = codegen.current_bytecode.len();

            codegen.loop_contexts.push(LoopContext {
                break_addrs: Vec::new(),
                continue_target: loop_start,
            });

            for s in body {
                compile_stmt(codegen, s)?;
            }

            // Jump back to start
            codegen.emit(OvmOpcode::Jmp);
            let jump_back = (loop_start as i32) - (codegen.current_bytecode.len() as i32) - 4;
            codegen.emit_i32(jump_back);

            // Patch breaks
            let ctx = codegen.loop_contexts.pop().unwrap();
            let end_pos = codegen.current_bytecode.len();
            for break_addr in ctx.break_addrs {
                let jump = (end_pos as i32) - (break_addr as i32) - 4;
                let bytes = jump.to_le_bytes();
                codegen.current_bytecode[break_addr..break_addr + 4].copy_from_slice(&bytes);
            }
        }

        TypedStatement::Break => {
            if !codegen.loop_contexts.is_empty() {
                codegen.emit(OvmOpcode::Jmp);
                let pos = codegen.current_bytecode.len();
                codegen
                    .loop_contexts
                    .last_mut()
                    .unwrap()
                    .break_addrs
                    .push(pos);
                codegen.emit_i32(0); // placeholder
            }
        }

        TypedStatement::Continue => {
            if !codegen.loop_contexts.is_empty() {
                let target = codegen.loop_contexts.last().unwrap().continue_target;
                codegen.emit(OvmOpcode::Jmp);
                let jump_back = (target as i32) - (codegen.current_bytecode.len() as i32) - 4;
                codegen.emit_i32(jump_back);
            }
        }

        TypedStatement::Pass => {} // No-op
        TypedStatement::Yield(expr) => {
            if let Some(e) = expr {
                compile_expr(codegen, e)?;
                codegen.emit(OvmOpcode::Ret);
            }
        }
        TypedStatement::Spawn(expr) => {
            compile_expr(codegen, expr)?;
            codegen.emit(OvmOpcode::Pop); // spawn result not used in OVM
        }
        TypedStatement::Select { arms } => {
            // Select is not fully supported in OVM; compile first arm body as fallback
            if let Some(arm) = arms.first() {
                for s in &arm.body {
                    compile_stmt(codegen, s)?;
                }
            }
        }

        TypedStatement::Match {
            expr: match_expr,
            arms,
        } => {
            // Compile match: evaluate expr, store to local, test each arm
            compile_expr(codegen, match_expr)?;
            let match_val = alloc_local(codegen, "$match_val");
            codegen.emit(OvmOpcode::StoreLoc);
            codegen.emit_u16(match_val);

            let mut arm_exit_jumps = Vec::new();
            for (i, (pattern, body)) in arms.iter().enumerate() {
                // Load match value for pattern test
                codegen.emit(OvmOpcode::LoadLoc);
                codegen.emit_u16(match_val);
                compile_pattern_test(codegen, pattern)?;
                codegen.emit(OvmOpcode::Jz);
                let jz_pos = codegen.current_bytecode.len();
                codegen.emit_i32(0);
                // Compile arm body
                for s in body {
                    compile_stmt(codegen, s)?;
                }
                // Jump to end of match
                codegen.emit(OvmOpcode::Jmp);
                arm_exit_jumps.push(codegen.current_bytecode.len());
                codegen.emit_i32(0);
                // Patch Jz to next arm (or end if last)
                let next_arm = codegen.current_bytecode.len() as i32;
                let jump = next_arm - jz_pos as i32 - 4;
                let bytes = jump.to_le_bytes();
                codegen.current_bytecode[jz_pos..jz_pos + 4].copy_from_slice(&bytes);
            }
            // Patch all arm exit jumps to end
            let end_pos = codegen.current_bytecode.len();
            for pos in arm_exit_jumps {
                let jump = end_pos as i32 - pos as i32 - 4;
                let bytes = jump.to_le_bytes();
                codegen.current_bytecode[pos..pos + 4].copy_from_slice(&bytes);
            }
        }

        TypedStatement::Defer(inner) => {
            // Defer: compile the inner statement now (simplified — proper impl would
            // push to a cleanup stack and emit on scope exit)
            compile_stmt(codegen, inner)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Compile expressions (each pushes exactly one value onto the stack)
// ---------------------------------------------------------------------------

fn compile_expr(codegen: &mut OvmCodegen, expr: &TypedExpr) -> Result<(), String> {
    match &expr.kind {
        TypedExprKind::Literal(lit) => compile_literal(codegen, lit),

        TypedExprKind::Identifier(name) => {
            if let Some(&idx) = codegen.local_indices.get(name) {
                codegen.emit(OvmOpcode::LoadLoc);
                codegen.emit_u16(idx);
            } else {
                codegen.emit(OvmOpcode::PushNull);
            }
        }

        TypedExprKind::Call(callee, args) => {
            // Push arguments left-to-right
            for arg in args {
                compile_expr(codegen, arg)?;
            }
            if let TypedExprKind::Identifier(name) = &callee.kind {
                if codegen.func_indices.contains_key(name) {
                    // User-defined function: emit Call
                    let idx = codegen.func_indices[name];
                    codegen.emit(OvmOpcode::Call);
                    codegen.emit_u32(idx);
                } else {
                    // Built-in function: emit Syscall with name as constant
                    let name_idx = codegen.add_constant(OvmConstant::String(name.clone()));
                    codegen.emit(OvmOpcode::Syscall);
                    codegen.emit_u16(name_idx as u16);
                }
            }
        }

        TypedExprKind::MethodCall {
            receiver,
            method,
            args,
        } => {
            // Compile receiver
            compile_expr(codegen, receiver)?;
            // Compile args
            for arg in args {
                compile_expr(codegen, arg)?;
            }
            // Dispatch via syscall with method name
            let method_idx = codegen.add_constant(OvmConstant::String(method.clone()));
            codegen.emit(OvmOpcode::Syscall);
            codegen.emit_u16(method_idx as u16);
        }

        TypedExprKind::Binary(left, op, right) => {
            compile_expr(codegen, right)?;
            compile_expr(codegen, left)?;
            let opcode = binary_opcode(op, &expr.ty);
            codegen.emit(opcode);
        }

        TypedExprKind::Unary(op, inner) => {
            compile_expr(codegen, inner)?;
            match op {
                UnaryOp::Neg => {
                    if is_float_ty(&expr.ty) {
                        codegen.emit(OvmOpcode::NegF64);
                    } else {
                        codegen.emit(OvmOpcode::NegI64);
                    }
                }
                UnaryOp::Not => codegen.emit(OvmOpcode::Not),
            }
        }

        TypedExprKind::Field(obj, field_name) => {
            compile_expr(codegen, obj)?;
            let field_idx = codegen.add_constant(OvmConstant::String(field_name.clone()));
            codegen.emit(OvmOpcode::GetField);
            codegen.emit_u32(field_idx);
        }

        TypedExprKind::Index(array, index) => {
            compile_expr(codegen, array)?;
            compile_expr(codegen, index)?;
            codegen.emit(OvmOpcode::ArrayGet);
        }

        TypedExprKind::Array(items) => {
            // Push elements, then create array
            for item in items {
                compile_expr(codegen, item)?;
            }
            codegen.emit(OvmOpcode::NewArray);
            codegen.emit_u32(items.len() as u32);
        }

        TypedExprKind::StructLiteral { name, fields } => {
            // Push fields as key-value pairs, then create struct
            for (field_name, field_expr) in fields {
                let name_idx = codegen.add_constant(OvmConstant::String(field_name.clone()));
                codegen.emit(OvmOpcode::PushStr);
                codegen.emit_u32(name_idx);
                compile_expr(codegen, field_expr)?;
            }
            let type_idx = codegen.add_constant(OvmConstant::String(name.clone()));
            codegen.emit(OvmOpcode::New);
            codegen.emit_u32(type_idx);
            codegen.emit_u16(fields.len() as u16);
        }

        TypedExprKind::Borrow { expr: inner, .. } => {
            // Borrow is a no-op in OVM — just evaluate the inner expression
            compile_expr(codegen, inner)?;
        }

        TypedExprKind::Deref(inner) => {
            compile_expr(codegen, inner)?;
        }

        // New variants — emit PushNull for now, proper impl below
        TypedExprKind::Range { start, end, .. } => {
            // Materialize range: push start, push end, create array
            if let Some(s) = start {
                compile_expr(codegen, s)?;
            } else {
                codegen.emit(OvmOpcode::PushI64);
                codegen.emit_u64(0);
            }
            if let Some(e) = end {
                compile_expr(codegen, e)?;
            } else {
                codegen.emit(OvmOpcode::PushNull);
            }
            codegen.emit(OvmOpcode::PushI64);
            codegen.emit_u64(2); // count placeholder
            codegen.emit(OvmOpcode::NewArray);
            codegen.emit_u32(2);
        }
        TypedExprKind::Lambda { body, .. } => {
            // Compile lambda body inline (single-expression closure)
            compile_expr(codegen, body)?;
        }
        TypedExprKind::Tuple(elems) => {
            for e in elems {
                compile_expr(codegen, e)?;
            }
            codegen.emit(OvmOpcode::NewArray);
            codegen.emit_u32(elems.len() as u32);
        }
        TypedExprKind::Await(inner) => {
            compile_expr(codegen, inner)?;
        }
        TypedExprKind::None => codegen.emit(OvmOpcode::PushNull),
        TypedExprKind::Some(inner) => compile_expr(codegen, inner)?,
        TypedExprKind::Ok(inner) => compile_expr(codegen, inner)?,
        TypedExprKind::Err(inner) => compile_expr(codegen, inner)?,
        TypedExprKind::If {
            condition,
            then_expr,
            else_expr,
        } => {
            compile_expr(codegen, condition)?;
            codegen.emit(OvmOpcode::Jz);
            let jz_pos = codegen.current_bytecode.len();
            codegen.emit_i32(0);
            compile_expr(codegen, then_expr)?;
            codegen.emit(OvmOpcode::Jmp);
            let jmp_pos = codegen.current_bytecode.len();
            codegen.emit_i32(0);
            let else_start = codegen.current_bytecode.len() as i32;
            let jz_jump = else_start - jz_pos as i32 - 4;
            let bytes = jz_jump.to_le_bytes();
            codegen.current_bytecode[jz_pos..jz_pos + 4].copy_from_slice(&bytes);
            compile_expr(codegen, else_expr)?;
            patch_jump(codegen, jmp_pos);
        }
        TypedExprKind::Match {
            expr: match_expr,
            arms,
        } => {
            // Compile match expression: compare each arm, jump to matching body
            compile_expr(codegen, match_expr)?;
            let match_val = alloc_local(codegen, "$match_val");
            codegen.emit(OvmOpcode::StoreLoc);
            codegen.emit_u16(match_val);

            let mut arm_jumps = Vec::new();
            for (i, (pattern, body)) in arms.iter().enumerate() {
                // Load match value for comparison
                codegen.emit(OvmOpcode::LoadLoc);
                codegen.emit_u16(match_val);
                // Compile pattern comparison
                compile_pattern_test(codegen, pattern)?;
                codegen.emit(OvmOpcode::Jz);
                let jz_pos = codegen.current_bytecode.len();
                codegen.emit_i32(0);
                // Arm body — push result
                compile_expr(codegen, body)?;
                // Jump past remaining arms
                codegen.emit(OvmOpcode::Jmp);
                arm_jumps.push(codegen.current_bytecode.len());
                codegen.emit_i32(0);
                // Patch the Jz to next arm
                if i < arms.len() - 1 {
                    // Will be patched to next arm start
                }
                patch_jump(codegen, jz_pos);
            }
            // Patch all arm body jumps to here (end of match)
            let end_pos = codegen.current_bytecode.len();
            for jump_pos in arm_jumps {
                let jump = end_pos as i32 - jump_pos as i32 - 4;
                let bytes = jump.to_le_bytes();
                codegen.current_bytecode[jump_pos..jump_pos + 4].copy_from_slice(&bytes);
            }
        }
    }
    Ok(())
}

fn compile_literal(codegen: &mut OvmCodegen, lit: &Literal) {
    match lit {
        Literal::Int(n) => {
            codegen.emit(OvmOpcode::PushI64);
            codegen.emit_u64(*n as u64);
        }
        Literal::Float(f) => {
            codegen.emit(OvmOpcode::PushF64);
            codegen.emit_u64(f.to_bits());
        }
        Literal::Bool(b) => codegen.emit(if *b {
            OvmOpcode::PushTrue
        } else {
            OvmOpcode::PushFalse
        }),
        Literal::String(s) => {
            let idx = codegen.add_constant(OvmConstant::String(s.clone()));
            codegen.emit(OvmOpcode::PushStr);
            codegen.emit_u32(idx);
        }
        Literal::Null => codegen.emit(OvmOpcode::PushNull),
    }
}

// ---------------------------------------------------------------------------
// Type helpers
// ---------------------------------------------------------------------------

fn is_float_ty(ty: &AstType) -> bool {
    matches!(ty, AstType::F32 | AstType::F64)
}

fn binary_opcode(op: &BinaryOp, ty: &AstType) -> OvmOpcode {
    let float = is_float_ty(ty);
    match op {
        BinaryOp::Add => {
            if float {
                OvmOpcode::AddF64
            } else {
                OvmOpcode::AddI64
            }
        }
        BinaryOp::Sub => {
            if float {
                OvmOpcode::SubF64
            } else {
                OvmOpcode::SubI64
            }
        }
        BinaryOp::Mul => {
            if float {
                OvmOpcode::MulF64
            } else {
                OvmOpcode::MulI64
            }
        }
        BinaryOp::Div => {
            if float {
                OvmOpcode::DivF64
            } else {
                OvmOpcode::DivI64
            }
        }
        BinaryOp::Mod => OvmOpcode::ModI64, // no float mod
        BinaryOp::Eq => OvmOpcode::Eq,
        BinaryOp::NotEq => OvmOpcode::Ne,
        BinaryOp::Lt => OvmOpcode::Lt,
        BinaryOp::LtEq => OvmOpcode::Le,
        BinaryOp::Gt => OvmOpcode::Gt,
        BinaryOp::GtEq => OvmOpcode::Ge,
        BinaryOp::And => OvmOpcode::And,
        BinaryOp::Or => OvmOpcode::Or,
        _ => OvmOpcode::AddI64,
    }
}

fn compile_pattern_test(
    codegen: &mut OvmCodegen,
    pattern: &crate::parser::ast::Pattern,
) -> Result<(), String> {
    use crate::parser::ast::Pattern;
    match pattern {
        Pattern::Wildcard | Pattern::Binding(_) => {
            // Wildcard/binding always matches — push true
            codegen.emit(OvmOpcode::PushTrue);
        }
        Pattern::Literal(lit) => {
            compile_literal(codegen, lit);
            codegen.emit(OvmOpcode::Eq);
        }
        Pattern::Constructor(name, sub_pats) => {
            // For None pattern: check if value is null
            if name == "None" {
                codegen.emit(OvmOpcode::IsNull);
            } else if name == "Some" && sub_pats.len() == 1 {
                // Some(x): not null, then check inner pattern
                codegen.emit(OvmOpcode::PushNull);
                codegen.emit(OvmOpcode::Ne);
            } else {
                // Generic constructor: always match for now
                codegen.emit(OvmOpcode::PushTrue);
            }
        }
        Pattern::Or(_) => {
            // OR patterns: treat as always matching for now
            codegen.emit(OvmOpcode::PushTrue);
        }
    }
    Ok(())
}
