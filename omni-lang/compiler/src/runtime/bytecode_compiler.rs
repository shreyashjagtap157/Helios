#![allow(dead_code)]
//! AST → OVM Bytecode Compiler
//!
//! Walks the Omni AST produced by the parser and emits a flat sequence of
//! [`OpCode`] instructions for each function, producing an [`OvmModule`] that
//! can be serialized to disk or executed directly by a bytecode interpreter.

use anyhow::Result;
use std::collections::HashMap;

use super::bytecode::{CompiledFunction, OpCode, OvmModule, Value};
use crate::parser::ast::{
    BinaryOp, Block, Expression, Function, Item, Literal, MatchBody, Module, Pattern, Statement,
    UnaryOp,
};

// ---------------------------------------------------------------------------
// Scope – local variable tracking
// ---------------------------------------------------------------------------

/// A lexical scope that maps variable names to stack-slot indices.
#[derive(Debug, Clone)]
struct Scope {
    /// Maps variable name → local slot index.
    locals: HashMap<String, usize>,
    /// The first slot index owned by this scope (used for cleanup).
    base: usize,
}

impl Scope {
    fn new(base: usize) -> Self {
        Self {
            locals: HashMap::new(),
            base,
        }
    }
}

// ---------------------------------------------------------------------------
// BytecodeCompiler
// ---------------------------------------------------------------------------

/// Compiles an Omni AST into OVM bytecode.
pub struct BytecodeCompiler {
    /// The module being built.
    module: OvmModule,

    // -- per-function state (reset for each function) --
    /// The instruction buffer for the function currently being compiled.
    instructions: Vec<OpCode>,
    /// Stack of lexical scopes.
    scopes: Vec<Scope>,
    /// Total number of local slots allocated so far (monotonically increasing
    /// within a function).
    local_count: usize,
    /// Collected global variable names.
    globals: Vec<String>,
    /// Loop context stack — each entry is `(loop_start, break_patches)`.
    loop_stack: Vec<(usize, Vec<usize>)>,
}

impl BytecodeCompiler {
    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    pub fn new() -> Self {
        Self {
            module: OvmModule::new("unnamed"),
            instructions: Vec::new(),
            scopes: Vec::new(),
            local_count: 0,
            globals: Vec::new(),
            loop_stack: Vec::new(),
        }
    }

    // -----------------------------------------------------------------------
    // Public entry points
    // -----------------------------------------------------------------------

    /// Compile a complete AST module into an [`OvmModule`].
    pub fn compile_module(&mut self, module: &Module) -> Result<OvmModule> {
        self.module = OvmModule::new("main");
        self.globals.clear();

        // First pass: collect top-level items (functions, structs, etc.)
        for item in &module.items {
            match item {
                Item::Function(func) => {
                    let compiled = self.compile_function(func)?;
                    let idx = self.module.functions.len();
                    if func.name == "main" {
                        self.module.entry_point = Some(idx);
                    }
                    self.module.functions.push(compiled);
                }
                Item::Import(import_decl) => {
                    let path = import_decl.path.join(".");
                    // Record the import as a top-level instruction in a synthetic init func
                    // For now just track it; the VM will resolve at load time.
                    self.module.constants.push(Value::String(path));
                }
                Item::Const(cdecl) => {
                    // Add to globals
                    self.globals.push(cdecl.name.clone());
                }
                Item::Struct(_sdef) => {
                    // Struct definitions don't generate code directly; they are
                    // used by NewStruct at call sites.
                }
                _ => {
                    // Other items (traits, impls, enums, etc.) are not yet
                    // compiled to bytecode — they are type-level constructs.
                }
            }
        }

        self.module.globals = self.globals.clone();
        Ok(self.module.clone())
    }

    /// Compile a single function AST node.
    pub fn compile_function(&mut self, func: &Function) -> Result<CompiledFunction> {
        // Reset per-function state
        self.instructions = Vec::new();
        self.scopes = Vec::new();
        self.local_count = 0;
        self.loop_stack = Vec::new();

        // Open the function-level scope and register parameters
        self.push_scope();
        for param in &func.params {
            self.declare_local(&param.name);
        }

        // Compile the body
        self.compile_block(&func.body)?;

        // Ensure the function ends with a Return
        if self.instructions.last() != Some(&OpCode::Return) {
            self.emit(OpCode::Push(Value::Null));
            self.emit(OpCode::Return);
        }

        self.pop_scope();

        Ok(CompiledFunction {
            name: func.name.clone(),
            arity: func.params.len(),
            locals_count: self.local_count,
            instructions: self.instructions.clone(),
        })
    }

    // -----------------------------------------------------------------------
    // Scope helpers
    // -----------------------------------------------------------------------

    fn push_scope(&mut self) {
        let base = self.local_count;
        self.scopes.push(Scope::new(base));
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /// Declare a new local variable in the current scope, returning its slot.
    fn declare_local(&mut self, name: &str) -> usize {
        let slot = self.local_count;
        self.local_count += 1;
        if let Some(scope) = self.scopes.last_mut() {
            scope.locals.insert(name.to_string(), slot);
        }
        slot
    }

    /// Look up a local variable across all enclosing scopes.
    fn resolve_local(&self, name: &str) -> Option<usize> {
        for scope in self.scopes.iter().rev() {
            if let Some(&slot) = scope.locals.get(name) {
                return Some(slot);
            }
        }
        None
    }

    // -----------------------------------------------------------------------
    // Instruction emission
    // -----------------------------------------------------------------------

    fn emit(&mut self, op: OpCode) {
        self.instructions.push(op);
    }

    /// Emit a placeholder instruction and return its index so we can patch it
    /// later (for forward jumps).
    fn emit_placeholder(&mut self) -> usize {
        let idx = self.instructions.len();
        self.instructions.push(OpCode::Jump(0)); // will be patched
        idx
    }

    /// Patch a previously emitted jump instruction to point to `target`.
    fn patch_jump(&mut self, placeholder_idx: usize, target: usize) {
        match &self.instructions[placeholder_idx] {
            OpCode::Jump(_) => self.instructions[placeholder_idx] = OpCode::Jump(target),
            OpCode::JumpIf(_) => self.instructions[placeholder_idx] = OpCode::JumpIf(target),
            OpCode::JumpIfNot(_) => self.instructions[placeholder_idx] = OpCode::JumpIfNot(target),
            _ => {} // shouldn't happen
        }
    }

    /// Current instruction index (i.e. where the *next* emitted instruction
    /// will land).
    fn current_offset(&self) -> usize {
        self.instructions.len()
    }

    // -----------------------------------------------------------------------
    // Block compilation
    // -----------------------------------------------------------------------

    fn compile_block(&mut self, block: &Block) -> Result<()> {
        for stmt in &block.statements {
            self.compile_statement(stmt)?;
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Statement compilation
    // -----------------------------------------------------------------------

    pub fn compile_statement(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {
            // -- let binding ---------------------------------------------------
            Statement::Let {
                name,
                value: Some(value),
                ..
            } => {
                self.compile_expression(value)?;
                let slot = self.declare_local(name);
                self.emit(OpCode::StoreLocal(slot));
            }
            Statement::Let { name, .. } => {
                self.emit(OpCode::Push(Value::Null));
                let slot = self.declare_local(name);
                self.emit(OpCode::StoreLocal(slot));
            }

            // -- var binding ---------------------------------------------------
            Statement::Var { name, value, .. } => {
                if let Some(val) = value {
                    self.compile_expression(val)?;
                } else {
                    self.emit(OpCode::Push(Value::Null));
                }
                let slot = self.declare_local(name);
                self.emit(OpCode::StoreLocal(slot));
            }

            // -- assignment ----------------------------------------------------
            Statement::Assignment { target, op, value } => {
                match target {
                    Expression::Identifier(name) => {
                        if let Some(compound_op) = op {
                            // Compound assignment: load current, compute, store
                            if let Some(slot) = self.resolve_local(name) {
                                self.emit(OpCode::LoadLocal(slot));
                            } else {
                                self.emit(OpCode::LoadGlobal(name.clone()));
                            }
                            self.compile_expression(value)?;
                            self.emit_binary_op(*compound_op);
                        } else {
                            self.compile_expression(value)?;
                        }

                        if let Some(slot) = self.resolve_local(name) {
                            self.emit(OpCode::StoreLocal(slot));
                        } else {
                            self.emit(OpCode::StoreGlobal(name.clone()));
                        }
                    }
                    Expression::Field(receiver, field_name) => {
                        self.compile_expression(receiver)?;
                        self.compile_expression(value)?;
                        self.emit(OpCode::StoreField(field_name.clone()));
                    }
                    Expression::Index(_collection, _index_expr) => {
                        // For index assignment we'd need a StoreIndex opcode;
                        // for now emit the value computation as a placeholder.
                        self.compile_expression(value)?;
                        self.emit(OpCode::Pop); // TODO: implement StoreIndex
                    }
                    _ => {
                        // Unsupported assignment target – just evaluate both
                        // sides for effect.
                        self.compile_expression(value)?;
                        self.emit(OpCode::Pop);
                    }
                }
            }

            // -- return --------------------------------------------------------
            Statement::Return(expr) => {
                if let Some(e) = expr {
                    self.compile_expression(e)?;
                } else {
                    self.emit(OpCode::Push(Value::Null));
                }
                self.emit(OpCode::Return);
            }

            // -- if/else -------------------------------------------------------
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                self.compile_expression(condition)?;
                // Emit JumpIfNot → else / end
                let jump_to_else = self.emit_jump_if_not_placeholder();
                self.push_scope();
                self.compile_block(then_block)?;
                self.pop_scope();

                if let Some(else_blk) = else_block {
                    let jump_over_else = self.emit_placeholder();
                    let else_start = self.current_offset();
                    self.patch_jump(jump_to_else, else_start);
                    self.push_scope();
                    self.compile_block(else_blk)?;
                    self.pop_scope();
                    let after_else = self.current_offset();
                    self.patch_jump(jump_over_else, after_else);
                } else {
                    let after_then = self.current_offset();
                    self.patch_jump(jump_to_else, after_then);
                }
            }

            // -- while loop ----------------------------------------------------
            Statement::While { condition, body } => {
                let loop_start = self.current_offset();
                self.loop_stack.push((loop_start, Vec::new()));

                self.compile_expression(condition)?;
                let exit_jump = self.emit_jump_if_not_placeholder();

                self.push_scope();
                self.compile_block(body)?;
                self.pop_scope();
                self.emit(OpCode::Jump(loop_start));

                let loop_end = self.current_offset();
                self.patch_jump(exit_jump, loop_end);

                // Patch break statements
                let (_, break_patches) = self.loop_stack.pop().unwrap();
                for bp in break_patches {
                    self.patch_jump(bp, loop_end);
                }
            }

            // -- infinite loop -------------------------------------------------
            Statement::Loop { body } => {
                let loop_start = self.current_offset();
                self.loop_stack.push((loop_start, Vec::new()));

                self.push_scope();
                self.compile_block(body)?;
                self.pop_scope();
                self.emit(OpCode::Jump(loop_start));

                let loop_end = self.current_offset();
                let (_, break_patches) = self.loop_stack.pop().unwrap();
                for bp in break_patches {
                    self.patch_jump(bp, loop_end);
                }
            }

            // -- for loop (desugared to while-style iteration) -----------------
            Statement::For { var, iter, body } => {
                // Evaluate the iterator expression once
                self.compile_expression(iter)?;
                let iter_slot = self.declare_local("__iter__");
                self.emit(OpCode::StoreLocal(iter_slot));

                // Create a counter
                self.emit(OpCode::Push(Value::Int(0)));
                let counter_slot = self.declare_local("__counter__");
                self.emit(OpCode::StoreLocal(counter_slot));

                let loop_start = self.current_offset();
                self.loop_stack.push((loop_start, Vec::new()));

                // condition: counter < len(iter)
                self.emit(OpCode::LoadLocal(counter_slot));
                self.emit(OpCode::LoadLocal(iter_slot));
                self.emit(OpCode::Len);
                self.emit(OpCode::Lt);
                let exit_jump = self.emit_jump_if_not_placeholder();

                self.push_scope();

                // var = iter[counter]
                let var_slot = self.declare_local(var);
                self.emit(OpCode::LoadLocal(iter_slot));
                self.emit(OpCode::LoadLocal(counter_slot));
                self.emit(OpCode::Index);
                self.emit(OpCode::StoreLocal(var_slot));

                self.compile_block(body)?;
                self.pop_scope();

                // counter += 1
                self.emit(OpCode::LoadLocal(counter_slot));
                self.emit(OpCode::Push(Value::Int(1)));
                self.emit(OpCode::Add);
                self.emit(OpCode::StoreLocal(counter_slot));

                self.emit(OpCode::Jump(loop_start));

                let loop_end = self.current_offset();
                self.patch_jump(exit_jump, loop_end);
                let (_, break_patches) = self.loop_stack.pop().unwrap();
                for bp in break_patches {
                    self.patch_jump(bp, loop_end);
                }
            }

            // -- break ---------------------------------------------------------
            Statement::Break(_) => {
                let ph = self.emit_placeholder();
                if let Some((_start, ref mut patches)) = self.loop_stack.last_mut() {
                    patches.push(ph);
                }
            }

            // -- continue ------------------------------------------------------
            Statement::Continue => {
                if let Some(&(loop_start, _)) = self.loop_stack.last() {
                    self.emit(OpCode::Jump(loop_start));
                }
            }

            // -- match ---------------------------------------------------------
            Statement::Match { expr, arms } => {
                self.compile_expression(expr)?;
                // Store matched value in a temporary slot
                let match_slot = self.declare_local("__match__");
                self.emit(OpCode::StoreLocal(match_slot));

                let mut end_patches = Vec::new();

                for arm in arms {
                    let next_arm_ph;
                    match &arm.pattern {
                        Pattern::Literal(lit) => {
                            self.emit(OpCode::LoadLocal(match_slot));
                            self.compile_literal(lit);
                            self.emit(OpCode::Eq);
                            next_arm_ph = self.emit_jump_if_not_placeholder();
                        }
                        Pattern::Binding(name) => {
                            // Binding always matches – bind the value
                            let slot = self.declare_local(name);
                            self.emit(OpCode::LoadLocal(match_slot));
                            self.emit(OpCode::StoreLocal(slot));
                            next_arm_ph = usize::MAX; // always matches
                        }
                        Pattern::Wildcard => {
                            next_arm_ph = usize::MAX; // always matches
                        }
                        Pattern::Constructor(name, _patterns) => {
                            // Simplified: treat as a type check (by name equality)
                            self.emit(OpCode::LoadLocal(match_slot));
                            self.emit(OpCode::TypeOf);
                            self.emit(OpCode::Push(Value::String(name.clone())));
                            self.emit(OpCode::Eq);
                            next_arm_ph = self.emit_jump_if_not_placeholder();
                        }
                        Pattern::Or(_) => {
                            // OR patterns: always match for now
                            next_arm_ph = usize::MAX;
                        }
                    }

                    // Compile arm body
                    match &arm.body {
                        MatchBody::Expr(e) => {
                            self.compile_expression(e)?;
                            self.emit(OpCode::Pop); // discard expression result
                        }
                        MatchBody::Block(b) => {
                            self.push_scope();
                            self.compile_block(b)?;
                            self.pop_scope();
                        }
                    }
                    let ep = self.emit_placeholder();
                    end_patches.push(ep);

                    if next_arm_ph != usize::MAX {
                        let here = self.current_offset();
                        self.patch_jump(next_arm_ph, here);
                    }
                }

                let end = self.current_offset();
                for ep in end_patches {
                    self.patch_jump(ep, end);
                }
            }

            // -- expression statement ------------------------------------------
            Statement::Expression(expr) => {
                self.compile_expression(expr)?;
                self.emit(OpCode::Pop); // discard unused value
            }

            // -- pass (nop) ---
            Statement::Pass => {
                self.emit(OpCode::Nop);
            }

            // -- yield (simplified: treat as return for now) ---
            Statement::Yield(expr) => {
                if let Some(e) = expr {
                    self.compile_expression(e)?;
                } else {
                    self.emit(OpCode::Push(Value::Null));
                }
                self.emit(OpCode::Return);
            }

            // -- spawn (simplified: just call the expression) ---
            Statement::Spawn(expr) => {
                self.compile_expression(expr)?;
                self.emit(OpCode::Pop);
            }

            // -- defer (simplified: emit inline at end – full impl needs unwinding) ---
            Statement::Defer(inner) => {
                // A proper defer implementation would record the statement and
                // emit it before every Return. For now, just compile inline.
                self.compile_statement(inner)?;
            }

            // -- select (simplified: evaluate arms sequentially) ---
            Statement::Select { arms } => {
                for arm in arms {
                    self.compile_expression(&arm.channel_op)?;
                    self.emit(OpCode::Pop);
                    self.push_scope();
                    self.compile_block(&arm.body)?;
                    self.pop_scope();
                }
            }
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Expression compilation
    // -----------------------------------------------------------------------

    pub fn compile_expression(&mut self, expr: &Expression) -> Result<()> {
        match expr {
            // -- literals --
            Expression::Literal(lit) => {
                self.compile_literal(lit);
            }

            // -- identifiers --
            Expression::Identifier(name) => {
                if let Some(slot) = self.resolve_local(name) {
                    self.emit(OpCode::LoadLocal(slot));
                } else {
                    self.emit(OpCode::LoadGlobal(name.clone()));
                }
            }

            // -- binary ops --
            Expression::Binary(lhs, op, rhs) => {
                self.compile_expression(lhs)?;
                self.compile_expression(rhs)?;
                self.emit_binary_op(*op);
            }

            // -- unary ops --
            Expression::Unary(op, expr) => {
                self.compile_expression(expr)?;
                match op {
                    UnaryOp::Neg => self.emit(OpCode::Neg),
                    UnaryOp::Not => self.emit(OpCode::Not),
                }
            }

            // -- function call --
            Expression::Call(callee, args) => {
                // Evaluate arguments left-to-right
                for arg in args {
                    self.compile_expression(arg)?;
                }
                match callee.as_ref() {
                    Expression::Identifier(name) => {
                        // Built-in function dispatch
                        match name.as_str() {
                            "print" => self.emit(OpCode::Print),
                            "println" => self.emit(OpCode::PrintLn),
                            "len" => self.emit(OpCode::Len),
                            "typeof" => self.emit(OpCode::TypeOf),
                            "assert" => self.emit(OpCode::Assert),
                            _ => self.emit(OpCode::CallNamed(name.clone(), args.len())),
                        }
                    }
                    _ => {
                        self.compile_expression(callee)?;
                        self.emit(OpCode::Call(args.len()));
                    }
                }
            }

            // -- method call --
            Expression::MethodCall {
                receiver,
                method,
                args,
            } => {
                self.compile_expression(receiver)?;
                for arg in args {
                    self.compile_expression(arg)?;
                }
                // Method calls are emitted as CallNamed with receiver as
                // first implicit argument.
                self.emit(OpCode::CallNamed(method.clone(), args.len() + 1));
            }

            // -- field access --
            Expression::Field(receiver, field_name) => {
                self.compile_expression(receiver)?;
                self.emit(OpCode::LoadField(field_name.clone()));
            }

            // -- indexing --
            Expression::Index(collection, index) => {
                self.compile_expression(collection)?;
                self.compile_expression(index)?;
                self.emit(OpCode::Index);
            }

            // -- path (module::item) --
            Expression::Path(_base, name) => {
                self.emit(OpCode::LoadGlobal(name.clone()));
            }

            // -- array literal --
            Expression::Array(elements) => {
                for elem in elements {
                    self.compile_expression(elem)?;
                }
                self.emit(OpCode::NewArray(elements.len()));
            }

            // -- struct literal --
            Expression::StructLiteral { name, fields } => {
                for (_fname, fexpr) in fields {
                    self.compile_expression(fexpr)?;
                }
                self.emit(OpCode::NewStruct(name.clone(), fields.len()));
            }

            // -- borrow/deref (no-ops at bytecode level) --
            Expression::Borrow { expr, .. } => {
                self.compile_expression(expr)?;
            }
            Expression::Deref(expr) => {
                self.compile_expression(expr)?;
            }

            // -- await (simplified: evaluate inner) --
            Expression::Await(inner) => {
                self.compile_expression(inner)?;
            }

            // -- ownership annotations (pass-through) --
            Expression::Shared(inner) | Expression::Own(inner) => {
                self.compile_expression(inner)?;
            }

            // -- range (simplified: create a 2-element array [start, end]) --
            Expression::Range { start, end, .. } => {
                if let Some(s) = start {
                    self.compile_expression(s)?;
                } else {
                    self.emit(OpCode::Push(Value::Int(0)));
                }
                if let Some(e) = end {
                    self.compile_expression(e)?;
                } else {
                    self.emit(OpCode::Push(Value::Null));
                }
                self.emit(OpCode::NewArray(2));
            }

            // -- lambda --
            Expression::Lambda { params, body } => {
                // Compile the lambda as a nested function and reference it
                let lambda_func = Function {
                    name: format!("__lambda_{}", self.module.functions.len()),
                    is_async: false,
                    attributes: Vec::new(),
                    params: params.clone(),
                    return_type: None,
                    body: Block {
                        statements: vec![Statement::Return(Some(*body.clone()))],
                    },
                };
                let compiled = self.compile_function(&lambda_func)?;
                let func_name = compiled.name.clone();
                self.module.functions.push(compiled);
                self.emit(OpCode::LoadGlobal(func_name));
            }

            // -- if expression --
            Expression::If {
                condition,
                then_expr,
                else_expr,
            } => {
                self.compile_expression(condition)?;
                let jump_to_else = self.emit_jump_if_not_placeholder();
                self.compile_expression(then_expr)?;
                let jump_over_else = self.emit_placeholder();
                let else_start = self.current_offset();
                self.patch_jump(jump_to_else, else_start);
                if let Some(else_e) = else_expr {
                    self.compile_expression(else_e)?;
                } else {
                    self.emit(OpCode::Push(Value::Null));
                }
                let after = self.current_offset();
                self.patch_jump(jump_over_else, after);
            }

            // -- match expression --
            Expression::Match { expr, arms } => {
                self.compile_expression(expr)?;
                let match_slot = self.declare_local("__match_expr__");
                self.emit(OpCode::StoreLocal(match_slot));

                let mut end_patches = Vec::new();
                for arm in arms {
                    let next_arm_ph;
                    match &arm.pattern {
                        Pattern::Literal(lit) => {
                            self.emit(OpCode::LoadLocal(match_slot));
                            self.compile_literal(lit);
                            self.emit(OpCode::Eq);
                            next_arm_ph = self.emit_jump_if_not_placeholder();
                        }
                        Pattern::Binding(name) => {
                            let slot = self.declare_local(name);
                            self.emit(OpCode::LoadLocal(match_slot));
                            self.emit(OpCode::StoreLocal(slot));
                            next_arm_ph = usize::MAX;
                        }
                        Pattern::Wildcard => {
                            next_arm_ph = usize::MAX;
                        }
                        Pattern::Constructor(name, _) => {
                            self.emit(OpCode::LoadLocal(match_slot));
                            self.emit(OpCode::TypeOf);
                            self.emit(OpCode::Push(Value::String(name.clone())));
                            self.emit(OpCode::Eq);
                            next_arm_ph = self.emit_jump_if_not_placeholder();
                        }
                        Pattern::Or(_) => {
                            // OR patterns: always match for now
                            next_arm_ph = usize::MAX;
                        }
                    }

                    match &arm.body {
                        MatchBody::Expr(e) => {
                            self.compile_expression(e)?;
                        }
                        MatchBody::Block(b) => {
                            self.push_scope();
                            self.compile_block(b)?;
                            self.pop_scope();
                            self.emit(OpCode::Push(Value::Null));
                        }
                    }
                    let ep = self.emit_placeholder();
                    end_patches.push(ep);

                    if next_arm_ph != usize::MAX {
                        let here = self.current_offset();
                        self.patch_jump(next_arm_ph, here);
                    }
                }

                // Fallback: push null
                self.emit(OpCode::Push(Value::Null));
                let end = self.current_offset();
                for ep in end_patches {
                    self.patch_jump(ep, end);
                }
            }

            // -- list comprehension (simplified) --
            Expression::ListComprehension {
                expr,
                var,
                iter,
                filter,
            } => {
                // Build an empty array, iterate, push matching elements
                self.emit(OpCode::NewArray(0)); // result array
                let result_slot = self.declare_local("__comp_result__");
                self.emit(OpCode::StoreLocal(result_slot));

                self.compile_expression(iter)?;
                let iter_slot = self.declare_local("__comp_iter__");
                self.emit(OpCode::StoreLocal(iter_slot));

                self.emit(OpCode::Push(Value::Int(0)));
                let counter_slot = self.declare_local("__comp_i__");
                self.emit(OpCode::StoreLocal(counter_slot));

                let loop_start = self.current_offset();
                self.emit(OpCode::LoadLocal(counter_slot));
                self.emit(OpCode::LoadLocal(iter_slot));
                self.emit(OpCode::Len);
                self.emit(OpCode::Lt);
                let exit_jump = self.emit_jump_if_not_placeholder();

                self.push_scope();
                let var_slot = self.declare_local(var);
                self.emit(OpCode::LoadLocal(iter_slot));
                self.emit(OpCode::LoadLocal(counter_slot));
                self.emit(OpCode::Index);
                self.emit(OpCode::StoreLocal(var_slot));

                if let Some(filt) = filter {
                    self.compile_expression(filt)?;
                    let skip_ph = self.emit_jump_if_not_placeholder();
                    self.compile_expression(expr)?;
                    self.emit(OpCode::Pop); // simplified — proper impl would append
                    let skip_end = self.current_offset();
                    self.patch_jump(skip_ph, skip_end);
                } else {
                    self.compile_expression(expr)?;
                    self.emit(OpCode::Pop);
                }

                self.pop_scope();

                self.emit(OpCode::LoadLocal(counter_slot));
                self.emit(OpCode::Push(Value::Int(1)));
                self.emit(OpCode::Add);
                self.emit(OpCode::StoreLocal(counter_slot));
                self.emit(OpCode::Jump(loop_start));

                let loop_end = self.current_offset();
                self.patch_jump(exit_jump, loop_end);

                self.emit(OpCode::LoadLocal(result_slot));
            }

            // -- tuple (emit as array) --
            Expression::Tuple(elems) => {
                for e in elems {
                    self.compile_expression(e)?;
                }
                self.emit(OpCode::NewArray(elems.len()));
            }

            // -- Option / Result constructors --
            Expression::None => {
                self.emit(OpCode::Push(Value::Null));
            }
            Expression::Some(inner) => {
                self.compile_expression(inner)?;
            }
            Expression::Ok(inner) => {
                self.compile_expression(inner)?;
            }
            Expression::Err(inner) => {
                self.compile_expression(inner)?;
                // Tag as error — simplified
                self.emit(OpCode::NewStruct("Err".into(), 1));
            }

            // -- generator (simplified: compile body) --
            Expression::Generator { body } => {
                self.push_scope();
                self.compile_block(body)?;
                self.pop_scope();
            }
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn compile_literal(&mut self, lit: &Literal) {
        match lit {
            Literal::Int(i) => self.emit(OpCode::Push(Value::Int(*i))),
            Literal::Float(f) => self.emit(OpCode::Push(Value::Float(*f))),
            Literal::String(s) => self.emit(OpCode::Push(Value::String(s.clone()))),
            Literal::Bool(b) => self.emit(OpCode::Push(Value::Bool(*b))),
            Literal::Null => self.emit(OpCode::Push(Value::Null)),
        }
    }

    fn emit_binary_op(&mut self, op: BinaryOp) {
        match op {
            BinaryOp::Add => self.emit(OpCode::Add),
            BinaryOp::Sub => self.emit(OpCode::Sub),
            BinaryOp::Mul => self.emit(OpCode::Mul),
            BinaryOp::Div => self.emit(OpCode::Div),
            BinaryOp::Mod => self.emit(OpCode::Mod),
            BinaryOp::Eq => self.emit(OpCode::Eq),
            BinaryOp::NotEq => self.emit(OpCode::Ne),
            BinaryOp::Lt => self.emit(OpCode::Lt),
            BinaryOp::Gt => self.emit(OpCode::Gt),
            BinaryOp::LtEq => self.emit(OpCode::Le),
            BinaryOp::GtEq => self.emit(OpCode::Ge),
            BinaryOp::And => self.emit(OpCode::And),
            BinaryOp::Or => self.emit(OpCode::Or),
            BinaryOp::Range | BinaryOp::RangeInclusive => {
                self.emit(OpCode::NewArray(2));
            }
        }
    }

    /// Emit a JumpIfNot placeholder and return its index.
    fn emit_jump_if_not_placeholder(&mut self) -> usize {
        let idx = self.instructions.len();
        self.instructions.push(OpCode::JumpIfNot(0));
        idx
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::*;

    /// Build a minimal module with one function from the given statements.
    fn module_with_main(stmts: Vec<Statement>) -> Module {
        Module {
            items: vec![Item::Function(Function {
                name: "main".into(),
                is_async: false,
                attributes: vec![],
                params: vec![],
                return_type: None,
                body: Block { statements: stmts },
            })],
        }
    }

    /// Compile a module and return the OvmModule.
    fn compile(module: &Module) -> OvmModule {
        let mut c = BytecodeCompiler::new();
        c.compile_module(module).expect("compilation failed")
    }

    /// Get the instructions of the first (main) function.
    fn main_instrs(m: &OvmModule) -> &[OpCode] {
        &m.functions[0].instructions
    }

    // -- Tests ---------------------------------------------------------------

    #[test]
    fn test_empty_main() {
        let m = compile(&module_with_main(vec![]));
        assert_eq!(m.entry_point, Some(0));
        assert_eq!(m.functions[0].name, "main");
        // Should end with PUSH null, RETURN
        let instrs = main_instrs(&m);
        assert!(matches!(instrs.last(), Some(OpCode::Return)));
    }

    #[test]
    fn test_let_binding() {
        let m = compile(&module_with_main(vec![Statement::Let {
            name: "x".into(),
            mutable: false,
            ty: None,
            value: Some(Expression::Literal(Literal::Int(42))),
        }]));
        let instrs = main_instrs(&m);
        assert!(instrs.contains(&OpCode::Push(Value::Int(42))));
        assert!(instrs.contains(&OpCode::StoreLocal(0)));
    }

    #[test]
    fn test_return_expression() {
        let m = compile(&module_with_main(vec![Statement::Return(Some(
            Expression::Literal(Literal::Bool(true)),
        ))]));
        let instrs = main_instrs(&m);
        assert_eq!(instrs[0], OpCode::Push(Value::Bool(true)));
        assert_eq!(instrs[1], OpCode::Return);
    }

    #[test]
    fn test_binary_expression() {
        let m = compile(&module_with_main(vec![Statement::Expression(
            Expression::Binary(
                Box::new(Expression::Literal(Literal::Int(3))),
                BinaryOp::Add,
                Box::new(Expression::Literal(Literal::Int(4))),
            ),
        )]));
        let instrs = main_instrs(&m);
        assert!(instrs.contains(&OpCode::Push(Value::Int(3))));
        assert!(instrs.contains(&OpCode::Push(Value::Int(4))));
        assert!(instrs.contains(&OpCode::Add));
    }

    #[test]
    fn test_unary_neg() {
        let m = compile(&module_with_main(vec![Statement::Expression(
            Expression::Unary(UnaryOp::Neg, Box::new(Expression::Literal(Literal::Int(5)))),
        )]));
        let instrs = main_instrs(&m);
        assert!(instrs.contains(&OpCode::Push(Value::Int(5))));
        assert!(instrs.contains(&OpCode::Neg));
    }

    #[test]
    fn test_if_else_jumps() {
        let m = compile(&module_with_main(vec![Statement::If {
            condition: Expression::Literal(Literal::Bool(true)),
            then_block: Block {
                statements: vec![Statement::Expression(Expression::Literal(Literal::Int(1)))],
            },
            else_block: Some(Block {
                statements: vec![Statement::Expression(Expression::Literal(Literal::Int(2)))],
            }),
        }]));
        let instrs = main_instrs(&m);
        // Should contain at least one JumpIfNot and one Jump (for else skip)
        assert!(instrs.iter().any(|op| matches!(op, OpCode::JumpIfNot(_))));
        assert!(instrs.iter().any(|op| matches!(op, OpCode::Jump(_))));
    }

    #[test]
    fn test_while_loop() {
        let m = compile(&module_with_main(vec![Statement::While {
            condition: Expression::Literal(Literal::Bool(false)),
            body: Block {
                statements: vec![Statement::Pass],
            },
        }]));
        let instrs = main_instrs(&m);
        // Should have a JumpIfNot (exit) and a Jump (back to start)
        assert!(instrs.iter().any(|op| matches!(op, OpCode::JumpIfNot(_))));
        assert!(instrs.iter().any(|op| matches!(op, OpCode::Jump(_))));
    }

    #[test]
    fn test_function_call() {
        let m = compile(&module_with_main(vec![Statement::Expression(
            Expression::Call(
                Box::new(Expression::Identifier("foo".into())),
                vec![
                    Expression::Literal(Literal::Int(1)),
                    Expression::Literal(Literal::Int(2)),
                ],
            ),
        )]));
        let instrs = main_instrs(&m);
        assert!(instrs.contains(&OpCode::CallNamed("foo".into(), 2)));
    }

    #[test]
    fn test_println_builtin() {
        let m = compile(&module_with_main(vec![Statement::Expression(
            Expression::Call(
                Box::new(Expression::Identifier("println".into())),
                vec![Expression::Literal(Literal::String("hello".into()))],
            ),
        )]));
        let instrs = main_instrs(&m);
        assert!(instrs.contains(&OpCode::Push(Value::String("hello".into()))));
        assert!(instrs.contains(&OpCode::PrintLn));
    }

    #[test]
    fn test_array_literal() {
        let m = compile(&module_with_main(vec![Statement::Expression(
            Expression::Array(vec![
                Expression::Literal(Literal::Int(10)),
                Expression::Literal(Literal::Int(20)),
            ]),
        )]));
        let instrs = main_instrs(&m);
        assert!(instrs.contains(&OpCode::NewArray(2)));
    }

    #[test]
    fn test_struct_literal() {
        let m = compile(&module_with_main(vec![Statement::Expression(
            Expression::StructLiteral {
                name: "Point".into(),
                fields: vec![
                    ("x".into(), Expression::Literal(Literal::Int(1))),
                    ("y".into(), Expression::Literal(Literal::Int(2))),
                ],
            },
        )]));
        let instrs = main_instrs(&m);
        assert!(instrs.contains(&OpCode::NewStruct("Point".into(), 2)));
    }

    #[test]
    fn test_break_and_continue_in_loop() {
        let m = compile(&module_with_main(vec![Statement::Loop {
            body: Block {
                statements: vec![
                    Statement::If {
                        condition: Expression::Literal(Literal::Bool(true)),
                        then_block: Block {
                            statements: vec![Statement::Break(None)],
                        },
                        else_block: None,
                    },
                    Statement::Continue,
                ],
            },
        }]));
        let instrs = main_instrs(&m);
        // Should contain multiple jumps for break/continue/loop
        let jump_count = instrs
            .iter()
            .filter(|op| matches!(op, OpCode::Jump(_)))
            .count();
        assert!(
            jump_count >= 2,
            "expected at least 2 jumps, got {}",
            jump_count
        );
    }

    #[test]
    fn test_nested_scopes_separate_locals() {
        let m = compile(&module_with_main(vec![
            Statement::Let {
                name: "a".into(),
                mutable: false,
                ty: None,
                value: Some(Expression::Literal(Literal::Int(1))),
            },
            Statement::If {
                condition: Expression::Literal(Literal::Bool(true)),
                then_block: Block {
                    statements: vec![Statement::Let {
                        name: "b".into(),
                        mutable: false,
                        ty: None,
                        value: Some(Expression::Literal(Literal::Int(2))),
                    }],
                },
                else_block: None,
            },
        ]));
        // locals_count should account for both a and b
        assert!(m.functions[0].locals_count >= 2);
    }

    #[test]
    fn test_multiple_functions_in_module() {
        let module = Module {
            items: vec![
                Item::Function(Function {
                    name: "helper".into(),
                    is_async: false,
                    attributes: vec![],
                    params: vec![Param {
                        name: "n".into(),
                        ty: Type::I64,
                    }],
                    return_type: Some(Type::I64),
                    body: Block {
                        statements: vec![Statement::Return(Some(Expression::Binary(
                            Box::new(Expression::Identifier("n".into())),
                            BinaryOp::Add,
                            Box::new(Expression::Literal(Literal::Int(1))),
                        )))],
                    },
                }),
                Item::Function(Function {
                    name: "main".into(),
                    is_async: false,
                    attributes: vec![],
                    params: vec![],
                    return_type: None,
                    body: Block {
                        statements: vec![Statement::Expression(Expression::Call(
                            Box::new(Expression::Identifier("helper".into())),
                            vec![Expression::Literal(Literal::Int(41))],
                        ))],
                    },
                }),
            ],
        };
        let m = compile(&module);
        assert_eq!(m.functions.len(), 2);
        assert_eq!(m.functions[0].name, "helper");
        assert_eq!(m.functions[0].arity, 1);
        assert_eq!(m.functions[1].name, "main");
        assert_eq!(m.entry_point, Some(1));
    }

    #[test]
    fn test_field_access_compiles() {
        let m = compile(&module_with_main(vec![Statement::Expression(
            Expression::Field(Box::new(Expression::Identifier("obj".into())), "x".into()),
        )]));
        let instrs = main_instrs(&m);
        assert!(instrs.contains(&OpCode::LoadField("x".into())));
    }

    #[test]
    fn test_var_without_init() {
        let m = compile(&module_with_main(vec![Statement::Var {
            name: "v".into(),
            ty: None,
            value: None,
        }]));
        let instrs = main_instrs(&m);
        assert!(instrs.contains(&OpCode::Push(Value::Null)));
        assert!(instrs.contains(&OpCode::StoreLocal(0)));
    }

    #[test]
    fn test_compound_roundtrip() {
        // Compile → serialize → deserialize, verify instructions survive
        let module = module_with_main(vec![
            Statement::Let {
                name: "x".into(),
                mutable: false,
                ty: None,
                value: Some(Expression::Literal(Literal::Int(10))),
            },
            Statement::Return(Some(Expression::Binary(
                Box::new(Expression::Identifier("x".into())),
                BinaryOp::Mul,
                Box::new(Expression::Literal(Literal::Int(2))),
            ))),
        ]);
        let ovm = compile(&module);
        let bytes = ovm.serialize();
        let ovm2 = super::super::bytecode::OvmModule::deserialize(&bytes).expect("deser failed");
        assert_eq!(
            ovm.functions[0].instructions,
            ovm2.functions[0].instructions
        );
    }
}
