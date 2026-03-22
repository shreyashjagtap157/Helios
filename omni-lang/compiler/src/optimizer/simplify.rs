//! Expression Simplification Pass
//!
//! Applies algebraic identities and boolean simplifications
//! to reduce expression complexity.

use crate::parser::ast::*;

/// Run expression simplification on the entire module.
pub fn simplify_expressions(module: &mut Module) {
    for item in &mut module.items {
        simplify_item(item);
    }
}

fn simplify_item(item: &mut Item) {
    match item {
        Item::Function(func) => simplify_block(&mut func.body),
        Item::Impl(impl_block) => {
            for method in &mut impl_block.methods {
                simplify_block(&mut method.body);
            }
        }
        Item::Module(mod_decl) => {
            for sub in &mut mod_decl.items {
                simplify_item(sub);
            }
        }
        Item::Const(c) => {
            c.value = simplify_expr(&c.value);
        }
        _ => {}
    }
}

fn simplify_block(block: &mut Block) {
    for stmt in &mut block.statements {
        simplify_stmt(stmt);
    }
}

fn simplify_stmt(stmt: &mut Statement) {
    match stmt {
        Statement::Let { value, .. } => {
            *value = simplify_expr(value);
        }
        Statement::Return(Some(expr)) => {
            *expr = simplify_expr(expr);
        }
        Statement::Expression(expr) => {
            *expr = simplify_expr(expr);
        }
        Statement::Assignment { target, value, .. } => {
            *target = simplify_expr(target);
            *value = simplify_expr(value);
        }
        Statement::If {
            condition,
            then_block,
            else_block,
        } => {
            *condition = simplify_expr(condition);
            simplify_block(then_block);
            if let Some(eb) = else_block {
                simplify_block(eb);
            }
        }
        Statement::While { condition, body } => {
            *condition = simplify_expr(condition);
            simplify_block(body);
        }
        Statement::For { iter, body, .. } => {
            *iter = simplify_expr(iter);
            simplify_block(body);
        }
        Statement::Loop { body } => simplify_block(body),
        Statement::Match { expr, arms } => {
            *expr = simplify_expr(expr);
            for arm in arms {
                match &mut arm.body {
                    MatchBody::Expr(e) => *e = simplify_expr(e),
                    MatchBody::Block(b) => simplify_block(b),
                }
            }
        }
        _ => {}
    }
}

/// Simplify an expression by applying algebraic identities.
pub fn simplify_expr(expr: &Expression) -> Expression {
    match expr {
        Expression::Binary(left, op, right) => {
            let l = simplify_expr(left);
            let r = simplify_expr(right);
            simplify_binary(&l, *op, &r)
        }
        Expression::Unary(op, inner) => {
            let simplified = simplify_expr(inner);
            simplify_unary(*op, &simplified)
        }
        Expression::Call(func, args) => {
            let f = simplify_expr(func);
            let a: Vec<Expression> = args.iter().map(|e| simplify_expr(e)).collect();
            Expression::Call(Box::new(f), a)
        }
        Expression::If {
            condition,
            then_expr,
            else_expr,
        } => Expression::If {
            condition: Box::new(simplify_expr(condition)),
            then_expr: Box::new(simplify_expr(then_expr)),
            else_expr: else_expr.as_ref().map(|e| Box::new(simplify_expr(e))),
        },
        Expression::Array(elems) => {
            Expression::Array(elems.iter().map(|e| simplify_expr(e)).collect())
        }
        Expression::Tuple(elems) => {
            Expression::Tuple(elems.iter().map(|e| simplify_expr(e)).collect())
        }
        _ => expr.clone(),
    }
}

/// Apply algebraic simplification rules to a binary expression.
fn simplify_binary(left: &Expression, op: BinaryOp, right: &Expression) -> Expression {
    // Arithmetic identities
    match op {
        // x + 0 → x, 0 + x → x
        BinaryOp::Add => {
            if is_int_zero(right) {
                return left.clone();
            }
            if is_int_zero(left) {
                return right.clone();
            }
            // String: x + "" → x, "" + x → x
            if is_empty_string(right) {
                return left.clone();
            }
            if is_empty_string(left) {
                return right.clone();
            }
        }

        // x - 0 → x
        BinaryOp::Sub => {
            if is_int_zero(right) {
                return left.clone();
            }
            // x - x → 0 (structural equality check)
            if exprs_equal(left, right) {
                return Expression::Literal(Literal::Int(0));
            }
        }

        // x * 1 → x, 1 * x → x
        // x * 0 → 0, 0 * x → 0
        BinaryOp::Mul => {
            if is_int_one(right) {
                return left.clone();
            }
            if is_int_one(left) {
                return right.clone();
            }
            if is_int_zero(right) {
                return Expression::Literal(Literal::Int(0));
            }
            if is_int_zero(left) {
                return Expression::Literal(Literal::Int(0));
            }
        }

        // x / 1 → x
        // x / x → 1 (when x != 0)
        BinaryOp::Div => {
            if is_int_one(right) {
                return left.clone();
            }
            if exprs_equal(left, right) && !is_int_zero(left) {
                return Expression::Literal(Literal::Int(1));
            }
        }

        // Boolean identities
        // x && true → x, true && x → x
        // x && false → false, false && x → false
        BinaryOp::And => {
            if is_bool_true(right) {
                return left.clone();
            }
            if is_bool_true(left) {
                return right.clone();
            }
            if is_bool_false(right) {
                return Expression::Literal(Literal::Bool(false));
            }
            if is_bool_false(left) {
                return Expression::Literal(Literal::Bool(false));
            }
        }

        // x || false → x, false || x → x
        // x || true → true, true || x → true
        BinaryOp::Or => {
            if is_bool_false(right) {
                return left.clone();
            }
            if is_bool_false(left) {
                return right.clone();
            }
            if is_bool_true(right) {
                return Expression::Literal(Literal::Bool(true));
            }
            if is_bool_true(left) {
                return Expression::Literal(Literal::Bool(true));
            }
        }

        _ => {}
    }

    // No simplification applicable
    Expression::Binary(Box::new(left.clone()), op, Box::new(right.clone()))
}

/// Simplify unary expressions.
fn simplify_unary(op: UnaryOp, inner: &Expression) -> Expression {
    match op {
        // !!x → x
        UnaryOp::Not => {
            if let Expression::Unary(UnaryOp::Not, inner2) = inner {
                return (**inner2).clone();
            }
        }
        // -(-x) → x
        UnaryOp::Neg => {
            if let Expression::Unary(UnaryOp::Neg, inner2) = inner {
                return (**inner2).clone();
            }
        }
    }
    Expression::Unary(op, Box::new(inner.clone()))
}

// ─── Helper predicates ───────────────────────────────────────────

fn is_int_zero(expr: &Expression) -> bool {
    matches!(expr, Expression::Literal(Literal::Int(0)))
}

fn is_int_one(expr: &Expression) -> bool {
    matches!(expr, Expression::Literal(Literal::Int(1)))
}

fn is_bool_true(expr: &Expression) -> bool {
    matches!(expr, Expression::Literal(Literal::Bool(true)))
}

fn is_bool_false(expr: &Expression) -> bool {
    matches!(expr, Expression::Literal(Literal::Bool(false)))
}

fn is_empty_string(expr: &Expression) -> bool {
    matches!(expr, Expression::Literal(Literal::String(s)) if s.is_empty())
}

/// Structural equality of two expressions (conservative).
fn exprs_equal(a: &Expression, b: &Expression) -> bool {
    match (a, b) {
        (Expression::Identifier(x), Expression::Identifier(y)) => x == y,
        (Expression::Literal(Literal::Int(x)), Expression::Literal(Literal::Int(y))) => x == y,
        (Expression::Literal(Literal::Float(x)), Expression::Literal(Literal::Float(y))) => x == y,
        (Expression::Literal(Literal::Bool(x)), Expression::Literal(Literal::Bool(y))) => x == y,
        (Expression::Literal(Literal::String(x)), Expression::Literal(Literal::String(y))) => {
            x == y
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn int(v: i64) -> Expression {
        Expression::Literal(Literal::Int(v))
    }

    fn ident(name: &str) -> Expression {
        Expression::Identifier(name.to_string())
    }

    fn bool_lit(v: bool) -> Expression {
        Expression::Literal(Literal::Bool(v))
    }

    fn str_lit(s: &str) -> Expression {
        Expression::Literal(Literal::String(s.to_string()))
    }

    fn bin(left: Expression, op: BinaryOp, right: Expression) -> Expression {
        Expression::Binary(Box::new(left), op, Box::new(right))
    }

    fn unary(op: UnaryOp, expr: Expression) -> Expression {
        Expression::Unary(op, Box::new(expr))
    }

    #[test]
    fn test_add_zero_identity() {
        // x + 0 → x
        let result = simplify_expr(&bin(ident("x"), BinaryOp::Add, int(0)));
        assert!(matches!(result, Expression::Identifier(ref n) if n == "x"));

        // 0 + x → x
        let result = simplify_expr(&bin(int(0), BinaryOp::Add, ident("x")));
        assert!(matches!(result, Expression::Identifier(ref n) if n == "x"));
    }

    #[test]
    fn test_mul_one_identity() {
        // x * 1 → x
        let result = simplify_expr(&bin(ident("x"), BinaryOp::Mul, int(1)));
        assert!(matches!(result, Expression::Identifier(ref n) if n == "x"));

        // 1 * x → x
        let result = simplify_expr(&bin(int(1), BinaryOp::Mul, ident("x")));
        assert!(matches!(result, Expression::Identifier(ref n) if n == "x"));
    }

    #[test]
    fn test_mul_zero() {
        // x * 0 → 0
        let result = simplify_expr(&bin(ident("x"), BinaryOp::Mul, int(0)));
        assert!(matches!(result, Expression::Literal(Literal::Int(0))));

        // 0 * x → 0
        let result = simplify_expr(&bin(int(0), BinaryOp::Mul, ident("x")));
        assert!(matches!(result, Expression::Literal(Literal::Int(0))));
    }

    #[test]
    fn test_sub_self() {
        // x - x → 0
        let result = simplify_expr(&bin(ident("x"), BinaryOp::Sub, ident("x")));
        assert!(matches!(result, Expression::Literal(Literal::Int(0))));
    }

    #[test]
    fn test_div_self() {
        // x / x → 1
        let result = simplify_expr(&bin(ident("x"), BinaryOp::Div, ident("x")));
        assert!(matches!(result, Expression::Literal(Literal::Int(1))));
    }

    #[test]
    fn test_and_true_identity() {
        // x && true → x
        let result = simplify_expr(&bin(ident("x"), BinaryOp::And, bool_lit(true)));
        assert!(matches!(result, Expression::Identifier(ref n) if n == "x"));
    }

    #[test]
    fn test_or_false_identity() {
        // x || false → x
        let result = simplify_expr(&bin(ident("x"), BinaryOp::Or, bool_lit(false)));
        assert!(matches!(result, Expression::Identifier(ref n) if n == "x"));
    }

    #[test]
    fn test_and_false_short_circuit() {
        // x && false → false
        let result = simplify_expr(&bin(ident("x"), BinaryOp::And, bool_lit(false)));
        assert!(matches!(result, Expression::Literal(Literal::Bool(false))));
    }

    #[test]
    fn test_or_true_short_circuit() {
        // x || true → true
        let result = simplify_expr(&bin(ident("x"), BinaryOp::Or, bool_lit(true)));
        assert!(matches!(result, Expression::Literal(Literal::Bool(true))));
    }

    #[test]
    fn test_double_not() {
        // !!x → x
        let result = simplify_expr(&unary(UnaryOp::Not, unary(UnaryOp::Not, ident("x"))));
        assert!(matches!(result, Expression::Identifier(ref n) if n == "x"));
    }

    #[test]
    fn test_string_concat_empty() {
        // x + "" → x
        let result = simplify_expr(&bin(ident("x"), BinaryOp::Add, str_lit("")));
        assert!(matches!(result, Expression::Identifier(ref n) if n == "x"));

        // "" + x → x
        let result = simplify_expr(&bin(str_lit(""), BinaryOp::Add, ident("x")));
        assert!(matches!(result, Expression::Identifier(ref n) if n == "x"));
    }
}
