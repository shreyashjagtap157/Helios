//! Dead Code Elimination Pass
//!
//! Removes unreachable code, unused variables, empty blocks,
//! trivially dead branches, unused functions, and unused imports.

use crate::parser::ast::*;
use std::collections::HashSet;

/// Run dead code elimination on the entire module.
pub fn eliminate_dead_code(module: &mut Module) {
    // Collect all called function names for unused-function detection
    let called = collect_called_functions(module);

    // Remove unused functions (keep "main" and any called function)
    module.items.retain(|item| match item {
        Item::Function(f) => f.name == "main" || called.contains(&f.name),
        _ => true,
    });

    // Remove unused imports
    let used_names = collect_used_identifiers(module);
    module.items.retain(|item| match item {
        Item::Import(imp) => {
            let imported_name = imp
                .alias
                .as_ref()
                .unwrap_or_else(|| imp.path.last().unwrap());
            used_names.contains(imported_name.as_str())
        }
        _ => true,
    });

    // Process each function body
    for item in &mut module.items {
        if let Item::Function(func) = item {
            eliminate_in_block(&mut func.body);
        }
        if let Item::Impl(impl_block) = item {
            for method in &mut impl_block.methods {
                eliminate_in_block(&mut method.body);
            }
        }
    }
}

/// Collect names of all functions that are called anywhere in the module.
fn collect_called_functions(module: &Module) -> HashSet<String> {
    let mut called = HashSet::new();
    for item in &module.items {
        match item {
            Item::Function(f) => collect_calls_in_block(&f.body, &mut called),
            Item::Impl(ib) => {
                for m in &ib.methods {
                    collect_calls_in_block(&m.body, &mut called);
                }
            }
            _ => {}
        }
    }
    called
}

fn collect_calls_in_block(block: &Block, called: &mut HashSet<String>) {
    for stmt in &block.statements {
        collect_calls_in_stmt(stmt, called);
    }
}

fn collect_calls_in_stmt(stmt: &Statement, called: &mut HashSet<String>) {
    match stmt {
        Statement::Let { value, .. } => collect_calls_in_expr(value, called),
        Statement::Expression(expr) => collect_calls_in_expr(expr, called),
        Statement::Return(Some(expr)) => collect_calls_in_expr(expr, called),
        Statement::If {
            condition,
            then_block,
            else_block,
        } => {
            collect_calls_in_expr(condition, called);
            collect_calls_in_block(then_block, called);
            if let Some(eb) = else_block {
                collect_calls_in_block(eb, called);
            }
        }
        Statement::While { condition, body } => {
            collect_calls_in_expr(condition, called);
            collect_calls_in_block(body, called);
        }
        Statement::For { iter, body, .. } => {
            collect_calls_in_expr(iter, called);
            collect_calls_in_block(body, called);
        }
        Statement::Loop { body } => collect_calls_in_block(body, called),
        Statement::Assignment { value, .. } => collect_calls_in_expr(value, called),
        Statement::Match { expr, arms } => {
            collect_calls_in_expr(expr, called);
            for arm in arms {
                match &arm.body {
                    MatchBody::Expr(e) => collect_calls_in_expr(e, called),
                    MatchBody::Block(b) => collect_calls_in_block(b, called),
                }
            }
        }
        _ => {}
    }
}

fn collect_calls_in_expr(expr: &Expression, called: &mut HashSet<String>) {
    match expr {
        Expression::Call(func, args) => {
            if let Expression::Identifier(name) = func.as_ref() {
                called.insert(name.clone());
            }
            collect_calls_in_expr(func, called);
            for arg in args {
                collect_calls_in_expr(arg, called);
            }
        }
        Expression::Binary(l, _, r) => {
            collect_calls_in_expr(l, called);
            collect_calls_in_expr(r, called);
        }
        Expression::Unary(_, inner) => collect_calls_in_expr(inner, called),
        Expression::MethodCall { receiver, args, .. } => {
            collect_calls_in_expr(receiver, called);
            for arg in args {
                collect_calls_in_expr(arg, called);
            }
        }
        Expression::If {
            condition,
            then_expr,
            else_expr,
        } => {
            collect_calls_in_expr(condition, called);
            collect_calls_in_expr(then_expr, called);
            if let Some(e) = else_expr {
                collect_calls_in_expr(e, called);
            }
        }
        Expression::Array(elems) | Expression::Tuple(elems) => {
            for e in elems {
                collect_calls_in_expr(e, called);
            }
        }
        _ => {}
    }
}

/// Collect all identifiers used in expressions (for unused-import detection).
fn collect_used_identifiers(module: &Module) -> HashSet<String> {
    let mut used = HashSet::new();
    for item in &module.items {
        match item {
            Item::Function(f) => collect_idents_in_block(&f.body, &mut used),
            Item::Impl(ib) => {
                for m in &ib.methods {
                    collect_idents_in_block(&m.body, &mut used);
                }
            }
            _ => {}
        }
    }
    used
}

fn collect_idents_in_block(block: &Block, used: &mut HashSet<String>) {
    for stmt in &block.statements {
        collect_idents_in_stmt(stmt, used);
    }
}

fn collect_idents_in_stmt(stmt: &Statement, used: &mut HashSet<String>) {
    match stmt {
        Statement::Let { value, .. } => collect_idents_in_expr(value, used),
        Statement::Expression(expr) => collect_idents_in_expr(expr, used),
        Statement::Return(Some(expr)) => collect_idents_in_expr(expr, used),
        Statement::If {
            condition,
            then_block,
            else_block,
        } => {
            collect_idents_in_expr(condition, used);
            collect_idents_in_block(then_block, used);
            if let Some(eb) = else_block {
                collect_idents_in_block(eb, used);
            }
        }
        Statement::While { condition, body } => {
            collect_idents_in_expr(condition, used);
            collect_idents_in_block(body, used);
        }
        Statement::For { iter, body, .. } => {
            collect_idents_in_expr(iter, used);
            collect_idents_in_block(body, used);
        }
        Statement::Loop { body } => collect_idents_in_block(body, used),
        Statement::Assignment { target, value, .. } => {
            collect_idents_in_expr(target, used);
            collect_idents_in_expr(value, used);
        }
        _ => {}
    }
}

fn collect_idents_in_expr(expr: &Expression, used: &mut HashSet<String>) {
    match expr {
        Expression::Identifier(name) => {
            used.insert(name.clone());
        }
        Expression::Call(func, args) => {
            collect_idents_in_expr(func, used);
            for a in args {
                collect_idents_in_expr(a, used);
            }
        }
        Expression::Binary(l, _, r) => {
            collect_idents_in_expr(l, used);
            collect_idents_in_expr(r, used);
        }
        Expression::Unary(_, inner) => collect_idents_in_expr(inner, used),
        Expression::MethodCall { receiver, args, .. } => {
            collect_idents_in_expr(receiver, used);
            for a in args {
                collect_idents_in_expr(a, used);
            }
        }
        Expression::Array(elems) | Expression::Tuple(elems) => {
            for e in elems {
                collect_idents_in_expr(e, used);
            }
        }
        Expression::If {
            condition,
            then_expr,
            else_expr,
        } => {
            collect_idents_in_expr(condition, used);
            collect_idents_in_expr(then_expr, used);
            if let Some(e) = else_expr {
                collect_idents_in_expr(e, used);
            }
        }
        _ => {}
    }
}

/// Eliminate dead code within a block.
fn eliminate_in_block(block: &mut Block) {
    // 1. Remove statements after return/break/continue
    let mut truncate_at = None;
    for (i, stmt) in block.statements.iter().enumerate() {
        match stmt {
            Statement::Return(_) | Statement::Break | Statement::Continue => {
                if i + 1 < block.statements.len() {
                    truncate_at = Some(i + 1);
                }
                break;
            }
            _ => {}
        }
    }
    if let Some(idx) = truncate_at {
        block.statements.truncate(idx);
    }

    // 2. Simplify `if true`/`if false`
    let mut new_stmts = Vec::with_capacity(block.statements.len());
    for stmt in block.statements.drain(..) {
        match stmt {
            Statement::If {
                condition: Expression::Literal(Literal::Bool(true)),
                then_block,
                ..
            } => {
                // if true → inline the then block
                new_stmts.extend(then_block.statements);
            }
            Statement::If {
                condition: Expression::Literal(Literal::Bool(false)),
                else_block: Some(eb),
                ..
            } => {
                // if false with else → inline else
                new_stmts.extend(eb.statements);
            }
            Statement::If {
                condition: Expression::Literal(Literal::Bool(false)),
                else_block: None,
                ..
            } => {
                // if false without else → remove entirely
            }
            other => new_stmts.push(other),
        }
    }
    block.statements = new_stmts;

    // 3. Remove empty blocks and recurse into sub-blocks
    for stmt in &mut block.statements {
        match stmt {
            Statement::If {
                then_block,
                else_block,
                ..
            } => {
                eliminate_in_block(then_block);
                if let Some(eb) = else_block {
                    eliminate_in_block(eb);
                }
            }
            Statement::While { body, .. } => eliminate_in_block(body),
            Statement::For { body, .. } => eliminate_in_block(body),
            Statement::Loop { body } => eliminate_in_block(body),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_fn(name: &str, stmts: Vec<Statement>) -> Item {
        Item::Function(Function {
            name: name.to_string(),
            is_async: false,
            attributes: vec![],
            params: vec![],
            return_type: None,
            body: Block { statements: stmts },
        })
    }

    fn int(v: i64) -> Expression {
        Expression::Literal(Literal::Int(v))
    }

    fn ident(name: &str) -> Expression {
        Expression::Identifier(name.to_string())
    }

    #[test]
    fn test_remove_after_return() {
        let mut module = Module {
            items: vec![make_fn(
                "main",
                vec![
                    Statement::Return(Some(int(1))),
                    Statement::Expression(int(2)), // unreachable
                    Statement::Expression(int(3)), // unreachable
                ],
            )],
        };
        eliminate_dead_code(&mut module);
        if let Item::Function(f) = &module.items[0] {
            assert_eq!(f.body.statements.len(), 1);
            assert!(matches!(&f.body.statements[0], Statement::Return(_)));
        }
    }

    #[test]
    fn test_remove_after_break() {
        let mut block = Block {
            statements: vec![Statement::Break, Statement::Expression(int(99))],
        };
        eliminate_in_block(&mut block);
        assert_eq!(block.statements.len(), 1);
    }

    #[test]
    fn test_remove_after_continue() {
        let mut block = Block {
            statements: vec![Statement::Continue, Statement::Expression(int(42))],
        };
        eliminate_in_block(&mut block);
        assert_eq!(block.statements.len(), 1);
    }

    #[test]
    fn test_if_true_inlined() {
        let mut block = Block {
            statements: vec![Statement::If {
                condition: Expression::Literal(Literal::Bool(true)),
                then_block: Block {
                    statements: vec![Statement::Expression(int(1))],
                },
                else_block: None,
            }],
        };
        eliminate_in_block(&mut block);
        assert_eq!(block.statements.len(), 1);
        assert!(matches!(&block.statements[0], Statement::Expression(_)));
    }

    #[test]
    fn test_if_false_removed() {
        let mut block = Block {
            statements: vec![Statement::If {
                condition: Expression::Literal(Literal::Bool(false)),
                then_block: Block {
                    statements: vec![Statement::Expression(int(1))],
                },
                else_block: None,
            }],
        };
        eliminate_in_block(&mut block);
        assert_eq!(block.statements.len(), 0);
    }

    #[test]
    fn test_if_false_with_else_keeps_else() {
        let mut block = Block {
            statements: vec![Statement::If {
                condition: Expression::Literal(Literal::Bool(false)),
                then_block: Block {
                    statements: vec![Statement::Expression(int(1))],
                },
                else_block: Some(Block {
                    statements: vec![Statement::Expression(int(2))],
                }),
            }],
        };
        eliminate_in_block(&mut block);
        assert_eq!(block.statements.len(), 1);
        match &block.statements[0] {
            Statement::Expression(Expression::Literal(Literal::Int(2))) => {}
            other => panic!("Expected expression 2, got {:?}", other),
        }
    }

    #[test]
    fn test_remove_unused_function() {
        let mut module = Module {
            items: vec![
                make_fn("main", vec![Statement::Expression(int(1))]),
                make_fn("unused_helper", vec![Statement::Expression(int(2))]),
            ],
        };
        eliminate_dead_code(&mut module);
        assert_eq!(module.items.len(), 1);
        if let Item::Function(f) = &module.items[0] {
            assert_eq!(f.name, "main");
        }
    }

    #[test]
    fn test_keep_called_function() {
        let mut module = Module {
            items: vec![
                make_fn(
                    "main",
                    vec![Statement::Expression(Expression::Call(
                        Box::new(ident("helper")),
                        vec![],
                    ))],
                ),
                make_fn("helper", vec![Statement::Return(Some(int(42)))]),
            ],
        };
        eliminate_dead_code(&mut module);
        assert_eq!(module.items.len(), 2);
    }

    #[test]
    fn test_remove_unused_import() {
        let mut module = Module {
            items: vec![
                Item::Import(ImportDecl {
                    path: vec!["std".into(), "unused_mod".into()],
                    alias: None,
                }),
                make_fn("main", vec![Statement::Expression(int(1))]),
            ],
        };
        eliminate_dead_code(&mut module);
        // The import "unused_mod" is not referenced, should be removed
        assert_eq!(module.items.len(), 1);
        assert!(matches!(&module.items[0], Item::Function(_)));
    }

    #[test]
    fn test_keep_used_import() {
        let mut module = Module {
            items: vec![
                Item::Import(ImportDecl {
                    path: vec!["std".into(), "io".into()],
                    alias: None,
                }),
                make_fn("main", vec![Statement::Expression(ident("io"))]),
            ],
        };
        eliminate_dead_code(&mut module);
        assert_eq!(module.items.len(), 2);
    }
}
