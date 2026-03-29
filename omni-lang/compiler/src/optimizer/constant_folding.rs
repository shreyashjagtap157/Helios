//! Constant Folding Optimization Pass
//!
//! Evaluates constant expressions at compile time, replacing them
//! with their computed values. Also propagates constants through
//! simple let bindings.

use crate::parser::ast::*;
use std::collections::HashMap;

/// Run constant folding on the entire module.
pub fn fold_constants(module: &mut Module) {
    for item in &mut module.items {
        fold_item(item);
    }
}

fn fold_item(item: &mut Item) {
    match item {
        Item::Function(func) => {
            let mut env = ConstEnv::new();
            fold_block(&mut func.body, &mut env);
        }
        Item::Impl(impl_block) => {
            for method in &mut impl_block.methods {
                let mut env = ConstEnv::new();
                fold_block(&mut method.body, &mut env);
            }
        }
        Item::Module(mod_decl) => {
            for sub in &mut mod_decl.items {
                fold_item(sub);
            }
        }
        Item::Const(c) => {
            c.value = fold_expr(&c.value);
        }
        _ => {}
    }
}

/// Environment for constant propagation – tracks known constant bindings.
struct ConstEnv {
    bindings: HashMap<String, Expression>,
}

impl ConstEnv {
    fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    fn insert(&mut self, name: String, value: Expression) {
        self.bindings.insert(name, value);
    }

    fn get(&self, name: &str) -> Option<&Expression> {
        self.bindings.get(name)
    }
}

fn fold_block(block: &mut Block, env: &mut ConstEnv) {
    let mut new_stmts = Vec::with_capacity(block.statements.len());
    for stmt in block.statements.drain(..) {
        new_stmts.push(fold_statement(stmt, env));
    }
    block.statements = new_stmts;
}

fn fold_statement(stmt: Statement, env: &mut ConstEnv) -> Statement {
    match stmt {
        Statement::Let {
            name,
            mutable,
            ty,
            value,
        } => {
            let folded = value.map(|v| fold_expr_with_env(&v, env));
            // If the binding is immutable and the value is a literal, propagate it
            if !mutable {
                if let Some(ref f) = folded {
                    if is_literal(f) {
                        env.insert(name.clone(), f.clone());
                    }
                }
            }
            Statement::Let {
                name,
                mutable,
                ty,
                value: folded,
            }
        }
        Statement::Return(Some(expr)) => Statement::Return(Some(fold_expr_with_env(&expr, env))),
        Statement::Return(None) => Statement::Return(None),
        Statement::If {
            condition,
            mut then_block,
            mut else_block,
        } => {
            let cond = fold_expr_with_env(&condition, env);
            fold_block(&mut then_block, env);
            if let Some(ref mut eb) = else_block {
                fold_block(eb, env);
            }
            Statement::If {
                condition: cond,
                then_block,
                else_block,
            }
        }
        Statement::While {
            condition,
            mut body,
        } => {
            let cond = fold_expr_with_env(&condition, env);
            fold_block(&mut body, env);
            Statement::While {
                condition: cond,
                body,
            }
        }
        Statement::For {
            var,
            iter,
            mut body,
        } => {
            let iter_folded = fold_expr_with_env(&iter, env);
            fold_block(&mut body, env);
            Statement::For {
                var,
                iter: iter_folded,
                body,
            }
        }
        Statement::Loop { mut body } => {
            fold_block(&mut body, env);
            Statement::Loop { body }
        }
        Statement::Expression(expr) => Statement::Expression(fold_expr_with_env(&expr, env)),
        Statement::Assignment { target, op, value } => Statement::Assignment {
            target: fold_expr_with_env(&target, env),
            op,
            value: fold_expr_with_env(&value, env),
        },
        Statement::Match { expr, arms } => {
            let folded_expr = fold_expr_with_env(&expr, env);
            let folded_arms: Vec<MatchArm> = arms
                .into_iter()
                .map(|arm| MatchArm {
                    pattern: arm.pattern,
                    body: match arm.body {
                        MatchBody::Expr(e) => MatchBody::Expr(fold_expr_with_env(&e, env)),
                        MatchBody::Block(mut b) => {
                            fold_block(&mut b, env);
                            MatchBody::Block(b)
                        }
                    },
                })
                .collect();
            Statement::Match {
                expr: folded_expr,
                arms: folded_arms,
            }
        }
        other => other,
    }
}

fn fold_expr_with_env(expr: &Expression, env: &ConstEnv) -> Expression {
    match expr {
        Expression::Identifier(name) => {
            if let Some(val) = env.get(name) {
                val.clone()
            } else {
                expr.clone()
            }
        }
        Expression::Binary(left, op, right) => {
            let l = fold_expr_with_env(left, env);
            let r = fold_expr_with_env(right, env);
            fold_binary(&l, *op, &r)
        }
        Expression::Unary(op, inner) => {
            let folded = fold_expr_with_env(inner, env);
            fold_unary(*op, &folded)
        }
        Expression::Call(func, args) => {
            let folded_args: Vec<Expression> =
                args.iter().map(|a| fold_expr_with_env(a, env)).collect();
            Expression::Call(Box::new(fold_expr_with_env(func, env)), folded_args)
        }
        Expression::If {
            condition,
            then_expr,
            else_expr,
        } => {
            let cond = fold_expr_with_env(condition, env);
            let then_e = fold_expr_with_env(then_expr, env);
            let else_e = else_expr
                .as_ref()
                .map(|e| Box::new(fold_expr_with_env(e, env)));
            // If condition is constant, pick the right branch
            if let Expression::Literal(Literal::Bool(b)) = &cond {
                if *b {
                    return then_e;
                } else if let Some(e) = else_e {
                    return *e;
                }
            }
            Expression::If {
                condition: Box::new(cond),
                then_expr: Box::new(then_e),
                else_expr: else_e,
            }
        }
        Expression::Array(elems) => {
            Expression::Array(elems.iter().map(|e| fold_expr_with_env(e, env)).collect())
        }
        Expression::Tuple(elems) => {
            Expression::Tuple(elems.iter().map(|e| fold_expr_with_env(e, env)).collect())
        }
        _ => expr.clone(),
    }
}

/// Fold an expression without an environment (top-level).
pub fn fold_expr(expr: &Expression) -> Expression {
    let env = ConstEnv::new();
    fold_expr_with_env(expr, &env)
}

fn is_literal(expr: &Expression) -> bool {
    matches!(expr, Expression::Literal(_))
}

/// Try to fold a binary expression with constant operands.
fn fold_binary(left: &Expression, op: BinaryOp, right: &Expression) -> Expression {
    match (left, op, right) {
        // Integer arithmetic
        (
            Expression::Literal(Literal::Int(a)),
            BinaryOp::Add,
            Expression::Literal(Literal::Int(b)),
        ) => Expression::Literal(Literal::Int(a + b)),
        (
            Expression::Literal(Literal::Int(a)),
            BinaryOp::Sub,
            Expression::Literal(Literal::Int(b)),
        ) => Expression::Literal(Literal::Int(a - b)),
        (
            Expression::Literal(Literal::Int(a)),
            BinaryOp::Mul,
            Expression::Literal(Literal::Int(b)),
        ) => Expression::Literal(Literal::Int(a * b)),
        (
            Expression::Literal(Literal::Int(a)),
            BinaryOp::Div,
            Expression::Literal(Literal::Int(b)),
        ) if *b != 0 => Expression::Literal(Literal::Int(a / b)),
        (
            Expression::Literal(Literal::Int(a)),
            BinaryOp::Mod,
            Expression::Literal(Literal::Int(b)),
        ) if *b != 0 => Expression::Literal(Literal::Int(a % b)),

        // Float arithmetic
        (
            Expression::Literal(Literal::Float(a)),
            BinaryOp::Add,
            Expression::Literal(Literal::Float(b)),
        ) => Expression::Literal(Literal::Float(a + b)),
        (
            Expression::Literal(Literal::Float(a)),
            BinaryOp::Sub,
            Expression::Literal(Literal::Float(b)),
        ) => Expression::Literal(Literal::Float(a - b)),
        (
            Expression::Literal(Literal::Float(a)),
            BinaryOp::Mul,
            Expression::Literal(Literal::Float(b)),
        ) => Expression::Literal(Literal::Float(a * b)),
        (
            Expression::Literal(Literal::Float(a)),
            BinaryOp::Div,
            Expression::Literal(Literal::Float(b)),
        ) if *b != 0.0 => Expression::Literal(Literal::Float(a / b)),

        // String concatenation
        (
            Expression::Literal(Literal::String(a)),
            BinaryOp::Add,
            Expression::Literal(Literal::String(b)),
        ) => Expression::Literal(Literal::String(format!("{}{}", a, b))),

        // Integer comparisons
        (
            Expression::Literal(Literal::Int(a)),
            BinaryOp::Eq,
            Expression::Literal(Literal::Int(b)),
        ) => Expression::Literal(Literal::Bool(a == b)),
        (
            Expression::Literal(Literal::Int(a)),
            BinaryOp::NotEq,
            Expression::Literal(Literal::Int(b)),
        ) => Expression::Literal(Literal::Bool(a != b)),
        (
            Expression::Literal(Literal::Int(a)),
            BinaryOp::Lt,
            Expression::Literal(Literal::Int(b)),
        ) => Expression::Literal(Literal::Bool(a < b)),
        (
            Expression::Literal(Literal::Int(a)),
            BinaryOp::Gt,
            Expression::Literal(Literal::Int(b)),
        ) => Expression::Literal(Literal::Bool(a > b)),
        (
            Expression::Literal(Literal::Int(a)),
            BinaryOp::LtEq,
            Expression::Literal(Literal::Int(b)),
        ) => Expression::Literal(Literal::Bool(a <= b)),
        (
            Expression::Literal(Literal::Int(a)),
            BinaryOp::GtEq,
            Expression::Literal(Literal::Int(b)),
        ) => Expression::Literal(Literal::Bool(a >= b)),

        // Boolean logic
        (
            Expression::Literal(Literal::Bool(a)),
            BinaryOp::And,
            Expression::Literal(Literal::Bool(b)),
        ) => Expression::Literal(Literal::Bool(*a && *b)),
        (
            Expression::Literal(Literal::Bool(a)),
            BinaryOp::Or,
            Expression::Literal(Literal::Bool(b)),
        ) => Expression::Literal(Literal::Bool(*a || *b)),

        // Not foldable – return rebuilt expression
        _ => Expression::Binary(Box::new(left.clone()), op, Box::new(right.clone())),
    }
}

/// Try to fold a unary expression with a constant operand.
fn fold_unary(op: UnaryOp, inner: &Expression) -> Expression {
    match (op, inner) {
        // Negate integer
        (UnaryOp::Neg, Expression::Literal(Literal::Int(v))) => {
            Expression::Literal(Literal::Int(-v))
        }
        // Negate float
        (UnaryOp::Neg, Expression::Literal(Literal::Float(v))) => {
            Expression::Literal(Literal::Float(-v))
        }
        // Double negation: -(-x) → x
        (UnaryOp::Neg, Expression::Unary(UnaryOp::Neg, inner2)) => (**inner2).clone(),
        // Boolean not
        (UnaryOp::Not, Expression::Literal(Literal::Bool(v))) => {
            Expression::Literal(Literal::Bool(!v))
        }
        // Double not: !!x → x
        (UnaryOp::Not, Expression::Unary(UnaryOp::Not, inner2)) => (**inner2).clone(),
        _ => Expression::Unary(op, Box::new(inner.clone())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to make int literal
    fn int(v: i64) -> Expression {
        Expression::Literal(Literal::Int(v))
    }

    // Helper to make float literal
    fn float(v: f64) -> Expression {
        Expression::Literal(Literal::Float(v))
    }

    // Helper to make bool literal
    fn bool_lit(v: bool) -> Expression {
        Expression::Literal(Literal::Bool(v))
    }

    // Helper to make string literal
    fn str_lit(s: &str) -> Expression {
        Expression::Literal(Literal::String(s.to_string()))
    }

    // Helper to make binary expression
    fn bin(left: Expression, op: BinaryOp, right: Expression) -> Expression {
        Expression::Binary(Box::new(left), op, Box::new(right))
    }

    // Helper to make unary expression
    fn unary(op: UnaryOp, expr: Expression) -> Expression {
        Expression::Unary(op, Box::new(expr))
    }

    fn ident(name: &str) -> Expression {
        Expression::Identifier(name.to_string())
    }

    #[test]
    fn test_fold_int_add() {
        let result = fold_expr(&bin(int(2), BinaryOp::Add, int(3)));
        assert!(matches!(result, Expression::Literal(Literal::Int(5))));
    }

    #[test]
    fn test_fold_int_sub() {
        let result = fold_expr(&bin(int(10), BinaryOp::Sub, int(4)));
        assert!(matches!(result, Expression::Literal(Literal::Int(6))));
    }

    #[test]
    fn test_fold_int_mul() {
        let result = fold_expr(&bin(int(3), BinaryOp::Mul, int(7)));
        assert!(matches!(result, Expression::Literal(Literal::Int(21))));
    }

    #[test]
    fn test_fold_int_div() {
        let result = fold_expr(&bin(int(20), BinaryOp::Div, int(4)));
        assert!(matches!(result, Expression::Literal(Literal::Int(5))));
    }

    #[test]
    fn test_fold_int_div_by_zero_no_fold() {
        let result = fold_expr(&bin(int(5), BinaryOp::Div, int(0)));
        assert!(matches!(result, Expression::Binary(..)));
    }

    #[test]
    fn test_fold_int_mod() {
        let result = fold_expr(&bin(int(10), BinaryOp::Mod, int(3)));
        assert!(matches!(result, Expression::Literal(Literal::Int(1))));
    }

    #[test]
    fn test_fold_float_add() {
        let result = fold_expr(&bin(float(1.5), BinaryOp::Add, float(2.5)));
        match result {
            Expression::Literal(Literal::Float(v)) => assert!((v - 4.0).abs() < 1e-10),
            _ => panic!("Expected float literal"),
        }
    }

    #[test]
    fn test_fold_string_concat() {
        let result = fold_expr(&bin(str_lit("hello"), BinaryOp::Add, str_lit(" world")));
        match result {
            Expression::Literal(Literal::String(s)) => assert_eq!(s, "hello world"),
            _ => panic!("Expected string literal"),
        }
    }

    #[test]
    fn test_fold_bool_and() {
        let result = fold_expr(&bin(bool_lit(true), BinaryOp::And, bool_lit(false)));
        assert!(matches!(result, Expression::Literal(Literal::Bool(false))));
    }

    #[test]
    fn test_fold_bool_or() {
        let result = fold_expr(&bin(bool_lit(false), BinaryOp::Or, bool_lit(true)));
        assert!(matches!(result, Expression::Literal(Literal::Bool(true))));
    }

    #[test]
    fn test_fold_comparison_gt() {
        let result = fold_expr(&bin(int(5), BinaryOp::Gt, int(3)));
        assert!(matches!(result, Expression::Literal(Literal::Bool(true))));
    }

    #[test]
    fn test_fold_comparison_eq() {
        let result = fold_expr(&bin(int(7), BinaryOp::Eq, int(7)));
        assert!(matches!(result, Expression::Literal(Literal::Bool(true))));
    }

    #[test]
    fn test_fold_comparison_lt() {
        let result = fold_expr(&bin(int(2), BinaryOp::Lt, int(10)));
        assert!(matches!(result, Expression::Literal(Literal::Bool(true))));
    }

    #[test]
    fn test_fold_unary_neg_int() {
        let result = fold_expr(&unary(UnaryOp::Neg, int(5)));
        assert!(matches!(result, Expression::Literal(Literal::Int(-5))));
    }

    #[test]
    fn test_fold_unary_double_neg() {
        // -(-5) → 5
        let result = fold_expr(&unary(UnaryOp::Neg, unary(UnaryOp::Neg, int(5))));
        assert!(matches!(result, Expression::Literal(Literal::Int(5))));
    }

    #[test]
    fn test_fold_unary_not_bool() {
        let result = fold_expr(&unary(UnaryOp::Not, bool_lit(true)));
        assert!(matches!(result, Expression::Literal(Literal::Bool(false))));
    }

    #[test]
    fn test_fold_nested() {
        // (2 + 3) * (4 - 1) → 5 * 3 → 15
        let result = fold_expr(&bin(
            bin(int(2), BinaryOp::Add, int(3)),
            BinaryOp::Mul,
            bin(int(4), BinaryOp::Sub, int(1)),
        ));
        assert!(matches!(result, Expression::Literal(Literal::Int(15))));
    }

    #[test]
    fn test_constant_propagation() {
        // let x = 5; let y = x + 3; → let y = 8;
        let mut module = Module {
            items: vec![Item::Function(Function {
                name: "test".into(),
                is_async: false,
                attributes: vec![],
                params: vec![],
                return_type: None,
                body: Block {
                    statements: vec![
                        Statement::Let {
                            name: "x".into(),
                            mutable: false,
                            ty: None,
                            value: Some(int(5)),
                        },
                        Statement::Let {
                            name: "y".into(),
                            mutable: false,
                            ty: None,
                            value: Some(bin(ident("x"), BinaryOp::Add, int(3))),
                        },
                    ],
                },
            })],
        };
        fold_constants(&mut module);
        if let Item::Function(f) = &module.items[0] {
            if let Statement::Let {
                value: Some(value), ..
            } = &f.body.statements[1]
            {
                assert!(
                    matches!(value, Expression::Literal(Literal::Int(8))),
                    "Expected 8, got {:?}",
                    value
                );
            } else {
                panic!("Expected let statement");
            }
        }
    }

    #[test]
    fn test_no_propagation_for_mutable() {
        // let mut x = 5; let y = x + 3; → x should NOT be propagated
        let mut module = Module {
            items: vec![Item::Function(Function {
                name: "test".into(),
                is_async: false,
                attributes: vec![],
                params: vec![],
                return_type: None,
                body: Block {
                    statements: vec![
                        Statement::Let {
                            name: "x".into(),
                            mutable: true,
                            ty: None,
                            value: Some(int(5)),
                        },
                        Statement::Let {
                            name: "y".into(),
                            mutable: false,
                            ty: None,
                            value: Some(bin(ident("x"), BinaryOp::Add, int(3))),
                        },
                    ],
                },
            })],
        };
        fold_constants(&mut module);
        if let Item::Function(f) = &module.items[0] {
            if let Statement::Let {
                value: Some(value), ..
            } = &f.body.statements[1]
            {
                // x is mutable, so y should still reference x
                assert!(
                    matches!(value, Expression::Binary(..)),
                    "Mutable var should not be propagated, got {:?}",
                    value
                );
            }
        }
    }

    #[test]
    fn test_fold_complex_propagation() {
        // let a = 2; let b = 3; let c = a * b + 1; → let c = 7;
        let mut module = Module {
            items: vec![Item::Function(Function {
                name: "test".into(),
                is_async: false,
                attributes: vec![],
                params: vec![],
                return_type: None,
                body: Block {
                    statements: vec![
                        Statement::Let {
                            name: "a".into(),
                            mutable: false,
                            ty: None,
                            value: Some(int(2)),
                        },
                        Statement::Let {
                            name: "b".into(),
                            mutable: false,
                            ty: None,
                            value: Some(int(3)),
                        },
                        Statement::Let {
                            name: "c".into(),
                            mutable: false,
                            ty: None,
                            value: Some(bin(
                                bin(ident("a"), BinaryOp::Mul, ident("b")),
                                BinaryOp::Add,
                                int(1),
                            )),
                        },
                    ],
                },
            })],
        };
        fold_constants(&mut module);
        if let Item::Function(f) = &module.items[0] {
            if let Statement::Let {
                value: Some(value), ..
            } = &f.body.statements[2]
            {
                assert!(
                    matches!(value, Expression::Literal(Literal::Int(7))),
                    "Expected 7, got {:?}",
                    value
                );
            }
        }
    }

    #[test]
    fn test_fold_comparison_noteq() {
        let result = fold_expr(&bin(int(3), BinaryOp::NotEq, int(5)));
        assert!(matches!(result, Expression::Literal(Literal::Bool(true))));
    }

    #[test]
    fn test_fold_comparison_lteq() {
        let result = fold_expr(&bin(int(5), BinaryOp::LtEq, int(5)));
        assert!(matches!(result, Expression::Literal(Literal::Bool(true))));
    }

    #[test]
    fn test_fold_comparison_gteq() {
        let result = fold_expr(&bin(int(3), BinaryOp::GtEq, int(5)));
        assert!(matches!(result, Expression::Literal(Literal::Bool(false))));
    }
}
