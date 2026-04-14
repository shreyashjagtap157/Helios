// Semantic Analysis and Type Checking for Omni Language

use crate::ast::*;
use std::collections::HashMap;

pub struct TypeChecker {
    variables: HashMap<String, Type>,
    functions: HashMap<String, (Vec<Type>, Type)>,
    current_return_type: Option<Type>,
    current_inferred_return: Option<Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut functions = HashMap::new();
        // Built-in functions
        functions.insert("println".to_string(), (vec![Type::String], Type::Void));
        functions.insert("print".to_string(), (vec![Type::String], Type::Void));

        TypeChecker {
            variables: HashMap::new(),
            functions,
            current_return_type: None,
            current_inferred_return: None,
        }
    }

    pub fn check(&mut self, program: &Program) -> Result<Program, String> {
        let mut checked_items = Vec::new();

        for item in &program.items {
            match item {
                Item::FunctionDef(func_def) => {
                    let checked_func = self.check_function(func_def)?;
                    checked_items.push(Item::FunctionDef(checked_func));
                }
                Item::StructDef(_) => {
                    checked_items.push(item.clone());
                }
                Item::Variable(var) => {
                    self.check_variable(var)?;
                    checked_items.push(item.clone());
                }
            }
        }

        Ok(Program {
            items: checked_items,
        })
    }

    fn check_function(&mut self, func: &FunctionDef) -> Result<FunctionDef, String> {
        let param_types: Vec<Type> = func.params.iter().map(|(_, t)| t.clone()).collect();
        if let Some(return_type) = &func.return_type {
            self.functions.insert(
                func.name.clone(),
                (param_types.clone(), return_type.clone()),
            );
        }

        // Type check function body
        let mut local_scope = HashMap::new();
        for (param_name, param_type) in &func.params {
            local_scope.insert(param_name.clone(), param_type.clone());
        }

        let saved_vars = self.variables.clone();
        let saved_return_type = self.current_return_type.clone();
        let saved_inferred_return = self.current_inferred_return.clone();
        self.variables = local_scope;
        self.current_return_type = func.return_type.clone();
        self.current_inferred_return = None;

        for stmt in &func.body.statements {
            self.check_statement(stmt)?;
        }

        let inferred_return_type = func
            .return_type
            .clone()
            .or_else(|| self.current_inferred_return.clone())
            .unwrap_or(Type::Void);

        self.functions.insert(
            func.name.clone(),
            (param_types, inferred_return_type.clone()),
        );
        self.variables = saved_vars;
        self.current_return_type = saved_return_type;
        self.current_inferred_return = saved_inferred_return;

        Ok(FunctionDef {
            name: func.name.clone(),
            params: func.params.clone(),
            return_type: Some(inferred_return_type),
            body: func.body.clone(),
        })
    }

    fn check_variable(&mut self, var: &VariableDecl) -> Result<(), String> {
        let value_type = self.infer_type(&var.value)?;

        if let Some(declared_type) = &var.var_type {
            if !self.types_compatible(&value_type, declared_type) {
                return Err(format!(
                    "Type mismatch for variable '{}': expected {}, got {}",
                    var.name,
                    declared_type.name(),
                    value_type.name()
                ));
            }
        }

        self.variables.insert(var.name.clone(), value_type);
        Ok(())
    }

    fn check_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            Statement::Let(name, declared_type, _is_mutable, expr) => {
                let value_type = self.infer_type(expr)?;

                if let Some(declared) = declared_type {
                    if !self.types_compatible(&value_type, declared) {
                        return Err(format!(
                            "Type mismatch: expected {}, got {}",
                            declared.name(),
                            value_type.name()
                        ));
                    }
                }

                self.variables.insert(name.clone(), value_type);
                Ok(())
            }
            Statement::Expression(expr) => {
                self.infer_type(expr)?;
                Ok(())
            }
            Statement::If(cond, then_block, else_block) => {
                let cond_type = self.infer_type(cond)?;
                if !matches!(cond_type, Type::Bool) {
                    return Err("If condition must be boolean".to_string());
                }

                for stmt in &then_block.statements {
                    self.check_statement(stmt)?;
                }

                if let Some(else_blk) = else_block {
                    for stmt in &else_blk.statements {
                        self.check_statement(stmt)?;
                    }
                }
                Ok(())
            }
            Statement::While(cond, body) => {
                let cond_type = self.infer_type(cond)?;
                if !matches!(cond_type, Type::Bool) {
                    return Err("While condition must be boolean".to_string());
                }

                for stmt in &body.statements {
                    self.check_statement(stmt)?;
                }
                Ok(())
            }
            Statement::For(var, iter, body) => {
                let _iter_type = self.infer_type(iter)?;
                self.variables.insert(var.clone(), Type::I64);

                for stmt in &body.statements {
                    self.check_statement(stmt)?;
                }
                Ok(())
            }
            Statement::Return(expr) => {
                match expr {
                    Some(value_expr) => {
                        let value_type = self.infer_type(value_expr)?;
                        if let Some(expected) = self.current_return_type.clone() {
                            if !self.types_compatible(&value_type, &expected) {
                                return Err(format!(
                                    "Return type mismatch: expected {}, got {}",
                                    expected.name(),
                                    value_type.name()
                                ));
                            }
                        } else {
                            self.merge_inferred_return_type(value_type)?;
                        }
                    }
                    None => {
                        if let Some(expected) = self.current_return_type.clone() {
                            if !matches!(expected, Type::Void) {
                                return Err(format!(
                                    "Return type mismatch: expected {}, got void",
                                    expected.name()
                                ));
                            }
                        } else {
                            self.merge_inferred_return_type(Type::Void)?;
                        }
                    }
                }

                Ok(())
            }
        }
    }

    fn infer_type(&mut self, expr: &Expression) -> Result<Type, String> {
        match expr {
            Expression::Literal(lit) => match lit {
                Literal::Number(_) => Ok(Type::I64),
                Literal::String(_) => Ok(Type::String),
                Literal::Bool(_) => Ok(Type::Bool),
            },
            Expression::Identifier(name) => self
                .variables
                .get(name)
                .cloned()
                .ok_or_else(|| format!("Undefined variable: {}", name)),
            Expression::BinaryOp(left, op, right) => {
                let left_type = self.infer_type(left)?;
                let right_type = self.infer_type(right)?;

                if !self.types_compatible(&left_type, &right_type) {
                    return Err(format!(
                        "Type mismatch in binary operation: {} and {}",
                        left_type.name(),
                        right_type.name()
                    ));
                }

                match op {
                    BinaryOp::Add
                    | BinaryOp::Subtract
                    | BinaryOp::Multiply
                    | BinaryOp::Divide
                    | BinaryOp::Modulo => Ok(left_type),
                    BinaryOp::Equal
                    | BinaryOp::NotEqual
                    | BinaryOp::Less
                    | BinaryOp::LessEqual
                    | BinaryOp::Greater
                    | BinaryOp::GreaterEqual
                    | BinaryOp::And
                    | BinaryOp::Or => Ok(Type::Bool),
                }
            }
            Expression::UnaryOp(op, expr) => {
                let expr_type = self.infer_type(expr)?;
                match op {
                    UnaryOp::Negate => Ok(expr_type),
                    UnaryOp::Not => {
                        if !matches!(expr_type, Type::Bool) {
                            return Err("Not operator requires boolean operand".to_string());
                        }
                        Ok(Type::Bool)
                    }
                }
            }
            Expression::Call(func_name, args) => {
                if let Some((param_types, return_type)) = self.functions.get(func_name).cloned() {
                    if args.len() != param_types.len() {
                        return Err(format!(
                            "Function {} expects {} arguments, got {}",
                            func_name,
                            param_types.len(),
                            args.len()
                        ));
                    }

                    for (arg, param_type) in args.iter().zip(param_types.iter()) {
                        let arg_type = self.infer_type(arg)?;
                        if !self.types_compatible(&arg_type, param_type) {
                            return Err(format!(
                                "Type mismatch in argument to {}: expected {}, got {}",
                                func_name,
                                param_type.name(),
                                arg_type.name()
                            ));
                        }
                    }

                    Ok(return_type)
                } else {
                    Err(format!("Undefined function: {}", func_name))
                }
            }
            Expression::Assignment(name, value) => {
                let value_type = self.infer_type(value)?;
                if let Some(var_type) = self.variables.get(name) {
                    if !self.types_compatible(&value_type, var_type) {
                        return Err(format!(
                            "Type mismatch in assignment to {}: expected {}, got {}",
                            name,
                            var_type.name(),
                            value_type.name()
                        ));
                    }
                }
                Ok(value_type)
            }
            Expression::If(cond, _then_block, _else_block) => {
                let _cond_type = self.infer_type(cond)?;
                // For now, return Void; in full version, would unify block types
                Ok(Type::Void)
            }
            _ => Err("Unsupported expression type".to_string()),
        }
    }

    fn types_compatible(&self, t1: &Type, t2: &Type) -> bool {
        t1 == t2
            || (matches!(t1, Type::I64) && matches!(t2, Type::I32))
            || (matches!(t1, Type::I32) && matches!(t2, Type::I64))
    }

    fn merge_inferred_return_type(&mut self, candidate: Type) -> Result<(), String> {
        if let Some(existing) = &self.current_inferred_return {
            let merged = self.merge_types(existing, &candidate).ok_or_else(|| {
                format!(
                    "Incompatible inferred return types: {} and {}",
                    existing.name(),
                    candidate.name()
                )
            })?;
            self.current_inferred_return = Some(merged);
        } else {
            self.current_inferred_return = Some(candidate);
        }
        Ok(())
    }

    fn merge_types(&self, left: &Type, right: &Type) -> Option<Type> {
        if left == right {
            return Some(left.clone());
        }
        if self.types_compatible(left, right) {
            if matches!(left, Type::I64) || matches!(right, Type::I64) {
                return Some(Type::I64);
            }
            return Some(left.clone());
        }
        None
    }
}
