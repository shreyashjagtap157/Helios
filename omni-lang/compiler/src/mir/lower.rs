//! AST to MIR lowering

use super::{
    BasicBlock, BasicBlockRef, BinOp, BorrowKind, LocalVar, MirModule, Place, Projection, Rvalue,
    Statement, Terminator, Type, UnOp,
};
use crate::parser::ast::{
    self as ast, BinaryOp as AstBinOp, Expression, Module as AstModule, Statement as AstStatement,
    Type as AstType, UnaryOp as AstUnOp,
};

/// Result of lowering
pub type LowerResult<T> = Result<T, LowerError>;

/// Lowering error
#[derive(Debug)]
pub struct LowerError {
    pub message: String,
}

impl std::fmt::Display for LowerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LowerError: {}", self.message)
    }
}

/// AST to MIR lowerer
pub struct AstToMirLowerer {
    /// The MIR module being built
    mir: MirModule,
    /// Current basic block being lowered into
    current_block: Option<BasicBlockRef>,
    /// Map from AST local names to MIR LocalVars
    local_map: std::collections::HashMap<String, LocalVar>,
    /// Placeholder for return value
    return_place: Option<Place>,
    /// Next available local var index
    next_local: usize,
}

impl AstToMirLowerer {
    /// Create a new lowerer
    pub fn new() -> Self {
        AstToMirLowerer {
            mir: MirModule::new("main".to_string()),
            current_block: None,
            local_map: std::collections::HashMap::new(),
            return_place: None,
            next_local: 0,
        }
    }

    /// Allocate a new local var
    fn alloc_local(&mut self, name: &str, ty: Type) -> LocalVar {
        let local = LocalVar(self.next_local);
        self.next_local += 1;
        self.mir.locals.insert(local, ty);
        self.mir.var_names.insert(local, name.to_string());
        local
    }

    /// Convert AST type to MIR type
    fn convert_type(ty: &AstType) -> Type {
        match ty {
            AstType::U8 => Type::U8,
            AstType::U16 => Type::U16,
            AstType::U32 => Type::U32,
            AstType::U64 => Type::U64,
            AstType::Usize => Type::Usize,
            AstType::I8 => Type::I8,
            AstType::I16 => Type::I16,
            AstType::I32 => Type::I32,
            AstType::I64 => Type::I64,
            AstType::Isize => Type::Isize,
            AstType::F32 => Type::F32,
            AstType::F64 => Type::F64,
            AstType::Bool => Type::Bool,
            AstType::Char => Type::Char,
            AstType::Str => Type::Str,
            _ => Type::Ptr, // Unknown types become pointers
        }
    }

    /// Lower a module
    pub fn lower_module(&mut self, ast_module: &AstModule) -> LowerResult<MirModule> {
        // Create entry block
        let entry = self.mir.new_basic_block("entry");
        self.current_block = Some(entry);
        self.return_place = Some(Place::new(LocalVar::new(0)));

        // Allocate return local
        self.alloc_local("_retval", Type::Ptr);

        // Lower each item in the module
        for item in &ast_module.items {
            match item {
                ast::Item::Function(func) => {
                    self.lower_function(func)?;
                }
                _ => {
                    // Skip non-function items for now
                }
            }
        }

        Ok(self.mir.clone())
    }

    /// Lower a function
    fn lower_function(&mut self, func: &ast::Function) -> LowerResult<()> {
        let body = &func.body;

        // Create new basic block for function body
        let func_block = self.mir.new_basic_block(&func.name);
        self.current_block = Some(func_block);

        // Allocate locals for parameters
        for param in &func.params {
            let mir_ty = Self::convert_type(&param.ty);
            let local = self.alloc_local(&param.name, mir_ty);
            self.local_map.insert(param.name.clone(), local);
        }

        // Lower function body statements
        for stmt in &body.statements {
            self.lower_statement(stmt)?;
        }

        Ok(())
    }

    /// Lower a statement
    fn lower_statement(&mut self, stmt: &ast::Statement) -> LowerResult<()> {
        let block_ref = self.current_block.ok_or(LowerError {
            message: "No current block".to_string(),
        })?;

        let block = self
            .mir
            .basic_blocks
            .get_mut(&block_ref)
            .ok_or(LowerError {
                message: "Basic block not found".to_string(),
            })?;

        match stmt {
            ast::Statement::Let { pattern, value, .. } => {
                // Handle let binding
                if let Some(expr) = value {
                    let place = self.lower_expression(expr)?;
                    // For now, just emit an assignment
                    // In real impl, would extract the local from pattern
                }
            }
            ast::Statement::Return(expr) => {
                if let Some(expr) = expr {
                    let place = self.lower_expression(expr)?;
                    block.statements.push(Statement::Assign(
                        self.return_place.clone().unwrap(),
                        Rvalue::Use(place),
                    ));
                    block.terminator = Terminator::Return(self.return_place.clone().unwrap());
                } else {
                    block.terminator = Terminator::Return(Place::new(LocalVar::new(0)));
                }
            }
            ast::Statement::Expr(expr) => {
                // Expression as statement (likely function call)
                let _ = self.lower_expression(expr)?;
            }
            ast::Statement::Assign { target, value } => {
                let target_place = self.lower_expression(target)?;
                let value_rvalue = self.lower_expression_as_rvalue(value)?;
                block
                    .statements
                    .push(Statement::Assign(target_place, value_rvalue));
            }
            _ => {
                // Other statement types - skip for now
            }
        }

        Ok(())
    }

    /// Lower an expression to a Place (or error)
    fn lower_expression(&self, expr: &Expression) -> LowerResult<Place> {
        // For now, return a placeholder place
        // Real implementation would handle all expression types
        Ok(Place::new(LocalVar::new(0)))
    }

    /// Lower an expression as an Rvalue
    fn lower_expression_as_rvalue(&self, expr: &Expression) -> LowerResult<Rvalue> {
        match expr {
            Expression::Literal(lit) => {
                // For now, just return a use of placeholder
                Ok(Rvalue::Use(Place::new(LocalVar::new(0))))
            }
            Expression::Binary { op, left, right } => {
                let left_place = self.lower_expression(left)?;
                let right_place = self.lower_expression(right)?;
                let binop = self.convert_binop(op);
                Ok(Rvalue::BinaryOp(binop, left_place, right_place))
            }
            Expression::Unary { op, expr } => {
                let expr_place = self.lower_expression(expr)?;
                let unop = self.convert_unop(op);
                Ok(Rvalue::UnaryOp(unop, expr_place))
            }
            _ => {
                // Other expressions - placeholder
                Ok(Rvalue::Use(Place::new(LocalVar::new(0))))
            }
        }
    }

    /// Convert AST binary operator to MIR
    fn convert_binop(&self, op: &AstBinOp) -> BinOp {
        match op {
            AstBinOp::Add => BinOp::Add,
            AstBinOp::Sub => BinOp::Sub,
            AstBinOp::Mul => BinOp::Mul,
            AstBinOp::Div => BinOp::Div,
            AstBinOp::Mod => BinOp::Mod,
            AstBinOp::BitAnd => BinOp::BitAnd,
            AstBinOp::BitOr => BinOp::BitOr,
            AstBinOp::BitXor => BinOp::BitXor,
            AstBinOp::Shl => BinOp::Shl,
            AstBinOp::Shr => BinOp::Shr,
            AstBinOp::Eq => BinOp::Eq,
            AstBinOp::Ne => BinOp::Ne,
            AstBinOp::Lt => BinOp::Lt,
            AstBinOp::Le => BinOp::Le,
            AstBinOp::Gt => BinOp::Gt,
            AstBinOp::Ge => BinOp::Ge,
        }
    }

    /// Convert AST unary operator to MIR
    fn convert_unop(&self, op: &AstUnOp) -> UnOp {
        match op {
            AstUnOp::Not => UnOp::Not,
            AstUnOp::Neg => UnOp::Neg,
            AstUnOp::BitNot => UnOp::BitNot,
        }
    }
}

impl Default for AstToMirLowerer {
    fn default() -> Self {
        Self::new()
    }
}
