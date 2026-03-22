//! Type parameter substitution and monomorphization
//!
//! This module handles cloning and transforming generic functions to create
//! monomorphic instances with concrete type parameters substituted.

use crate::parser::ast::*;
use std::collections::HashMap;

/// Maps generic type parameter names to concrete types
#[derive(Debug, Clone)]
pub struct TypeSubstitution {
    pub mapping: HashMap<String, Type>,
}

impl TypeSubstitution {
    pub fn new() -> Self {
        Self {
            mapping: HashMap::new(),
        }
    }

    pub fn add(&mut self, param_name: String, concrete_type: Type) {
        self.mapping.insert(param_name, concrete_type);
    }

    /// Substitute type parameters in a type
    pub fn apply_to_type(&self, ty: &Type) -> Type {
        match ty {
            // If this is a generic type that we have a substitution for, use it
            Type::Named(name) if self.mapping.contains_key(name) => self.mapping[name].clone(),

            // Generic type with constraints - substitute in the constraints
            Type::Generic(name, args) => {
                if let Some(concrete) = self.mapping.get(name) {
                    concrete.clone()
                } else {
                    let substituted_args: Vec<Type> =
                        args.iter().map(|arg| self.apply_to_type(arg)).collect();
                    Type::Generic(name.clone(), substituted_args)
                }
            }

            // Array types - substitute element type
            Type::Array(elem, len) => {
                let substituted_elem = self.apply_to_type(elem);
                Type::Array(Box::new(substituted_elem), len.clone())
            }

            // Slice types - substitute element type
            Type::Slice(elem) => {
                let substituted_elem = self.apply_to_type(elem);
                Type::Slice(Box::new(substituted_elem))
            }

            // Function types - substitute parameter and return types
            Type::Function(params, ret) => {
                let substituted_params: Vec<Type> =
                    params.iter().map(|p| self.apply_to_type(p)).collect();
                let substituted_ret = ret.as_ref().map(|r| Box::new(self.apply_to_type(r)));
                Type::Function(substituted_params, substituted_ret)
            }

            // Ownership-wrapped types - substitute inner type
            Type::WithOwnership(inner, ownership) => {
                let substituted_inner = self.apply_to_type(inner);
                Type::WithOwnership(Box::new(substituted_inner), *ownership)
            }

            // All other types pass through unchanged
            other => other.clone(),
        }
    }
}

/// Substitutes type parameters in expressions
pub struct ExpressionSubstitutor {
    substitution: TypeSubstitution,
}

impl ExpressionSubstitutor {
    pub fn new(substitution: TypeSubstitution) -> Self {
        Self { substitution }
    }

    /// Transform an expression by substituting type parameters
    pub fn transform_expression(&self, expr: &Expression) -> Expression {
        match expr {
            // Literals and identifiers pass through
            Expression::Literal(lit) => Expression::Literal(lit.clone()),
            Expression::Identifier(name) => Expression::Identifier(name.clone()),

            // Binary operations - recursively transform operands
            Expression::Binary(left, op, right) => {
                let transformed_left = self.transform_expression(left);
                let transformed_right = self.transform_expression(right);
                Expression::Binary(Box::new(transformed_left), *op, Box::new(transformed_right))
            }

            // Unary operations
            Expression::Unary(op, operand) => {
                let transformed_operand = self.transform_expression(operand);
                Expression::Unary(*op, Box::new(transformed_operand))
            }

            // Function calls
            Expression::Call(func, args) => {
                let transformed_func = self.transform_expression(func);
                let transformed_args: Vec<Expression> = args
                    .iter()
                    .map(|arg| self.transform_expression(arg))
                    .collect();
                Expression::Call(Box::new(transformed_func), transformed_args)
            }

            // Array construction - transform elements
            Expression::Array(elements) => {
                let transformed_elements: Vec<Expression> = elements
                    .iter()
                    .map(|elem| self.transform_expression(elem))
                    .collect();
                Expression::Array(transformed_elements)
            }

            // Indexing
            Expression::Index(array, index) => {
                let transformed_array = self.transform_expression(array);
                let transformed_index = self.transform_expression(index);
                Expression::Index(Box::new(transformed_array), Box::new(transformed_index))
            }

            // Method calls
            Expression::MethodCall {
                receiver,
                method,
                args,
            } => {
                let transformed_receiver = self.transform_expression(receiver);
                let transformed_args: Vec<Expression> = args
                    .iter()
                    .map(|arg| self.transform_expression(arg))
                    .collect();
                Expression::MethodCall {
                    receiver: Box::new(transformed_receiver),
                    method: method.clone(),
                    args: transformed_args,
                }
            }

            // Other expression types pass through
            other => other.clone(),
        }
    }
}

/// Substitutes type parameters in statements
pub struct StatementSubstitutor {
    substitution: TypeSubstitution,
}

impl StatementSubstitutor {
    pub fn new(substitution: TypeSubstitution) -> Self {
        Self { substitution }
    }

    pub fn transform_statement(&self, stmt: &Statement) -> Statement {
        match stmt {
            // Variable declarations - substitute type and transform initialization
            Statement::Let {
                name,
                mutable,
                ty,
                value,
            } => {
                let substituted_type = ty.as_ref().map(|t| self.substitution.apply_to_type(t));
                let expr_sub = ExpressionSubstitutor::new(self.substitution.clone());
                let transformed_value = expr_sub.transform_expression(value);
                Statement::Let {
                    name: name.clone(),
                    mutable: *mutable,
                    ty: substituted_type,
                    value: transformed_value,
                }
            }

            // Expression statements - transform the expression
            Statement::Expression(expr) => {
                let expr_sub = ExpressionSubstitutor::new(self.substitution.clone());
                Statement::Expression(expr_sub.transform_expression(expr))
            }

            // Return statements - transform the expression
            Statement::Return(Some(expr)) => {
                let expr_sub = ExpressionSubstitutor::new(self.substitution.clone());
                Statement::Return(Some(expr_sub.transform_expression(expr)))
            }

            // Assignment statements
            Statement::Assignment { target, op, value } => {
                let expr_sub = ExpressionSubstitutor::new(self.substitution.clone());
                Statement::Assignment {
                    target: expr_sub.transform_expression(target),
                    op: *op,
                    value: expr_sub.transform_expression(value),
                }
            }

            // If statements - transform condition and both branches
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                let expr_sub = ExpressionSubstitutor::new(self.substitution.clone());
                let transformed_condition = expr_sub.transform_expression(condition);
                let transformed_then = self.transform_block(then_block);
                let transformed_else = else_block.as_ref().map(|body| self.transform_block(body));
                Statement::If {
                    condition: transformed_condition,
                    then_block: transformed_then,
                    else_block: transformed_else,
                }
            }

            // While loops - transform condition and body
            Statement::While { condition, body } => {
                let expr_sub = ExpressionSubstitutor::new(self.substitution.clone());
                let transformed_condition = expr_sub.transform_expression(condition);
                let transformed_body = self.transform_block(body);
                Statement::While {
                    condition: transformed_condition,
                    body: transformed_body,
                }
            }

            // For loops - transform all components
            Statement::For { var, iter, body } => {
                let expr_sub = ExpressionSubstitutor::new(self.substitution.clone());
                let transformed_iter = expr_sub.transform_expression(iter);
                let transformed_body = self.transform_block(body);
                Statement::For {
                    var: var.clone(),
                    iter: transformed_iter,
                    body: transformed_body,
                }
            }

            // Match statements - transform scrutinee and all arms
            Statement::Match { expr, arms } => {
                let expr_sub = ExpressionSubstitutor::new(self.substitution.clone());
                let transformed_expr = expr_sub.transform_expression(expr);
                let transformed_arms = arms
                    .iter()
                    .map(|arm| {
                        let transformed_body = match &arm.body {
                            MatchBody::Expr(e) => MatchBody::Expr(expr_sub.transform_expression(e)),
                            MatchBody::Block(b) => MatchBody::Block(self.transform_block(b)),
                        };
                        MatchArm {
                            pattern: arm.pattern.clone(),
                            body: transformed_body,
                        }
                    })
                    .collect();
                Statement::Match {
                    expr: transformed_expr,
                    arms: transformed_arms,
                }
            }

            // Loop statements
            Statement::Loop { body } => {
                let transformed_body = self.transform_block(body);
                Statement::Loop {
                    body: transformed_body,
                }
            }

            // Break and Continue - pass through unchanged
            Statement::Break | Statement::Continue => stmt.clone(),

            // Return without value - pass through
            Statement::Return(None) => stmt.clone(),

            // Other statements pass through
            _ => stmt.clone(),
        }
    }

    /// Transform all statements in a block
    pub fn transform_block(&self, block: &Block) -> Block {
        Block {
            statements: block
                .statements
                .iter()
                .map(|stmt| self.transform_statement(stmt))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute_generic_named_type() {
        let mut sub = TypeSubstitution::new();
        sub.add("T".to_string(), Type::I64);

        let ty = Type::Named("T".to_string());
        let result = sub.apply_to_type(&ty);
        assert!(matches!(result, Type::I64));
    }

    #[test]
    fn test_substitute_array_element_type() {
        let mut sub = TypeSubstitution::new();
        sub.add("T".to_string(), Type::Str);

        let ty = Type::Array(Box::new(Type::Named("T".to_string())), None);
        let result = sub.apply_to_type(&ty);

        match result {
            Type::Array(elem, _) => assert!(matches!(*elem, Type::Str)),
            _ => panic!("Expected array type"),
        }
    }

    #[test]
    fn test_substitute_function_parameter_types() {
        let mut sub = TypeSubstitution::new();
        sub.add("T".to_string(), Type::I32);

        let ty = Type::Function(
            vec![Type::Named("T".to_string()), Type::Named("T".to_string())],
            Some(Box::new(Type::Bool)),
        );
        let result = sub.apply_to_type(&ty);

        match result {
            Type::Function(params, _) => {
                assert_eq!(params.len(), 2);
                assert!(matches!(params[0], Type::I32));
                assert!(matches!(params[1], Type::I32));
            }
            _ => panic!("Expected function type"),
        }
    }

    #[test]
    fn test_substitute_nested_generic() {
        let mut sub = TypeSubstitution::new();
        sub.add("T".to_string(), Type::I64);

        let ty = Type::Generic("Vec".to_string(), vec![Type::Named("T".to_string())]);
        let result = sub.apply_to_type(&ty);

        match result {
            Type::Generic(name, args) => {
                assert_eq!(name, "Vec");
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0], Type::I64));
            }
            _ => panic!("Expected generic type"),
        }
    }

    #[test]
    fn test_no_substitute_unknown_type_param() {
        let sub = TypeSubstitution::new();
        let ty = Type::Named("T".to_string());
        let result = sub.apply_to_type(&ty);

        // Should remain unchanged if no substitution exists
        match result {
            Type::Named(n) => assert_eq!(n, "T"),
            _ => panic!("Expected named type"),
        }
    }

    #[test]
    fn test_substitute_in_binary_expression() {
        let sub = TypeSubstitution::new();
        let expr_sub = ExpressionSubstitutor::new(sub);

        let expr = Expression::Binary(
            Box::new(Expression::Literal(Literal::Int(1))),
            BinaryOp::Add,
            Box::new(Expression::Literal(Literal::Int(2))),
        );

        let result = expr_sub.transform_expression(&expr);
        assert!(matches!(result, Expression::Binary(_, _, _)));
    }

    #[test]
    fn test_substitute_in_function_call() {
        let sub = TypeSubstitution::new();
        let expr_sub = ExpressionSubstitutor::new(sub);

        let expr = Expression::Call(
            Box::new(Expression::Identifier("process".to_string())),
            vec![Expression::Literal(Literal::Int(42))],
        );

        let result = expr_sub.transform_expression(&expr);
        assert!(matches!(result, Expression::Call(_, _)));
    }
}
