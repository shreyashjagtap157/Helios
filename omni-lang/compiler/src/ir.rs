// Intermediate Representation (IR) for Omni Language
// SSA-form IR that bridges AST to LLVM

use crate::ast::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct IRModule {
    pub functions: Vec<IRFunction>,
    pub globals: Vec<IRGlobal>,
    pub instructions: Vec<IRInstruction>,
}

#[derive(Debug, Clone)]
pub struct IRFunction {
    pub name: String,
    pub args: Vec<(String, Type)>,
    pub return_type: Type,
    pub blocks: Vec<BasicBlock>,
}

#[derive(Debug, Clone)]
pub struct IRGlobal {
    pub name: String,
    pub var_type: Type,
    pub value: Option<IRValue>,
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub label: String,
    pub instructions: Vec<IRInstruction>,
    pub terminator: Option<Terminator>,
}

#[derive(Debug, Clone)]
pub enum Terminator {
    Return(Option<IRValue>),
    Br(String),
    CondBr(IRValue, String, String),
}

#[derive(Debug, Clone)]
pub enum IRInstruction {
    Alloca(String, Type),
    Store(String, IRValue),
    Load(String, Type),
    BinaryOp(String, IRValue, BinaryOp, IRValue),
    UnaryOp(String, UnaryOp, IRValue),
    Call(String, String, Vec<IRValue>),
    Const(String, Type, String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum IRValue {
    Register(String),
    Constant(Type, String),
    Global(String),
}

impl IRValue {
    pub fn name(&self) -> String {
        match self {
            IRValue::Register(name) => name.clone(),
            IRValue::Constant(t, v) => format!("{} {}", t.name(), v),
            IRValue::Global(name) => name.clone(),
        }
    }
}

pub struct IRGenerator {
    var_counter: usize,
    block_counter: usize,
    local_vars: HashMap<String, String>,
}

impl IRGenerator {
    pub fn new() -> Self {
        IRGenerator {
            var_counter: 0,
            block_counter: 0,
            local_vars: HashMap::new(),
        }
    }

    pub fn generate(&mut self, program: &Program) -> Result<IRModule, String> {
        let mut functions = Vec::new();
        let mut globals = Vec::new();
        let instructions = Vec::new();

        for item in &program.items {
            match item {
                Item::FunctionDef(func_def) => {
                    let ir_func = self.generate_function(func_def)?;
                    functions.push(ir_func);
                }
                Item::StructDef(_) => {
                    // TODO: Handle structs
                }
                Item::Variable(var_decl) => {
                    let ir_global = self.generate_global(var_decl)?;
                    globals.push(ir_global);
                }
            }
        }

        Ok(IRModule {
            functions,
            globals,
            instructions,
        })
    }

    fn generate_function(&mut self, func: &FunctionDef) -> Result<IRFunction, String> {
        self.local_vars.clear();
        self.var_counter = 0;
        self.block_counter = 0;

        let mut blocks = Vec::new();

        // Create entry block
        let entry_label = self.gen_label();
        let mut entry_block = BasicBlock {
            label: entry_label.clone(),
            instructions: Vec::new(),
            terminator: None,
        };

        // Add alloca instructions for parameters
        for (param_name, param_type) in &func.params {
            let var_reg = self.gen_register();
            self.local_vars.insert(param_name.clone(), var_reg.clone());
            entry_block.instructions.push(IRInstruction::Alloca(
                var_reg,
                param_type.clone(),
            ));
        }

        // Generate block for function body
        let (block_instrs, term) = self.generate_block(&func.body)?;
        for instr in block_instrs {
            entry_block.instructions.push(instr);
        }

        entry_block.terminator = term;
        blocks.push(entry_block);

        Ok(IRFunction {
            name: func.name.clone(),
            args: func.params.clone(),
            return_type: func.return_type.clone(),
            blocks,
        })
    }

    fn generate_global(&mut self, var: &VariableDecl) -> Result<IRGlobal, String> {
        let value = self.generate_value(&var.value)?;
        Ok(IRGlobal {
            name: var.name.clone(),
            var_type: var.var_type.clone().unwrap_or(Type::I64),
            value: Some(value),
        })
    }

    fn generate_block(
        &mut self,
        block: &Block,
    ) -> Result<(Vec<IRInstruction>, Option<Terminator>), String> {
        let mut instructions = Vec::new();
        let mut terminator = None;

        for stmt in &block.statements {
            match stmt {
                Statement::Let(name, _typ, _is_mut, expr) => {
                    let var_reg = self.gen_register();
                    self.local_vars.insert(name.clone(), var_reg.clone());

                    let value = self.generate_value(expr)?;
                    instructions.push(IRInstruction::Store(var_reg, value));
                }
                Statement::Expression(expr) => {
                    let _value = self.generate_value(expr)?;
                    // Expression result is discarded
                }
                Statement::If(cond, _then_block, else_block) => {
                    let cond_val = self.generate_value(cond)?;
                    let then_label = self.gen_label();
                    let else_label = else_block
                        .as_ref()
                        .map(|_| self.gen_label())
                        .unwrap_or_else(|| self.gen_label());
                    let _merge_label = self.gen_label();

                    terminator = Some(Terminator::CondBr(cond_val, then_label, else_label));
                }
                Statement::While(cond, _body) => {
                    let _cond_val = self.generate_value(cond)?;
                    let _body_label = self.gen_label();
                    let _end_label = self.gen_label();
                    // TODO: Generate while loop IR
                }
                Statement::For(var, iter, _body) => {
                    let _var_reg = self.gen_register();
                    self.local_vars.insert(var.clone(), _var_reg.clone());
                    let _iter_val = self.generate_value(iter)?;
                    let _body_label = self.gen_label();
                    let _end_label = self.gen_label();
                    // TODO: Generate for loop IR
                }
                Statement::Return(expr) => {
                    let ret_val = if let Some(e) = expr {
                        Some(self.generate_value(e)?)
                    } else {
                        None
                    };
                    terminator = Some(Terminator::Return(ret_val));
                }
            }
        }

        if terminator.is_none() {
            terminator = Some(Terminator::Return(Some(IRValue::Constant(
                Type::I64,
                "0".to_string(),
            ))));
        }

        Ok((instructions, terminator))
    }

    fn generate_value(&mut self, expr: &Expression) -> Result<IRValue, String> {
        match expr {
            Expression::Literal(lit) => match lit {
                Literal::Number(n) => Ok(IRValue::Constant(Type::I64, n.to_string())),
                Literal::String(s) => Ok(IRValue::Constant(Type::String, s.clone())),
                Literal::Bool(b) => Ok(IRValue::Constant(Type::Bool, b.to_string())),
            },
            Expression::Identifier(name) => {
                if let Some(var_reg) = self.local_vars.get(name) {
                    Ok(IRValue::Register(var_reg.clone()))
                } else {
                    Ok(IRValue::Global(name.clone()))
                }
            }
            Expression::BinaryOp(left, _op, right) => {
                let _left_val = self.generate_value(left)?;
                let _right_val = self.generate_value(right)?;
                let result_reg = self.gen_register();

                Ok(IRValue::Register(result_reg))
            }
            Expression::Call(_func_name, args) => {
                let mut arg_vals = Vec::new();
                for arg in args {
                    arg_vals.push(self.generate_value(arg)?);
                }
                let result_reg = self.gen_register();
                Ok(IRValue::Register(result_reg))
            }
            Expression::Assignment(name, value) => {
                let val = self.generate_value(value)?;
                if let Some(var_reg) = self.local_vars.get(name) {
                    Ok(IRValue::Register(var_reg.clone()))
                } else {
                    Ok(val)
                }
            }
            _ => Err("Unsupported expression in IR generation".to_string()),
        }
    }

    fn gen_register(&mut self) -> String {
        let reg = format!("%{}", self.var_counter);
        self.var_counter += 1;
        reg
    }

    fn gen_label(&mut self) -> String {
        let label = format!("bb{}", self.block_counter);
        self.block_counter += 1;
        label
    }
}
