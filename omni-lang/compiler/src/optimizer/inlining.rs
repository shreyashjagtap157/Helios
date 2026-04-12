//! Function Inlining Pass
//!
//! Replaces function call sites with the body of the called function,
//! substituting parameters. Only inlines small, non-recursive functions.

use crate::parser::ast::*;
use std::collections::{HashMap, HashSet};

/// Maximum number of statements in a function body for it to be inlineable.
const MAX_INLINE_STMTS: usize = 5;

/// Maximum number of statements for aggressive inlining.
const MAX_AGGRESSIVE_INLINE_STMTS: usize = 10;

/// Run function inlining on the module.
/// When `aggressive` is true, the size threshold is raised.
pub fn inline_functions(module: &mut Module, aggressive: bool) {
    let threshold = if aggressive {
        MAX_AGGRESSIVE_INLINE_STMTS
    } else {
        MAX_INLINE_STMTS
    };

    // Build a map of inlineable functions
    let candidates = find_inline_candidates(module, threshold);

    if candidates.is_empty() {
        return;
    }

    // Inline call sites in each function
    for item in &mut module.items {
        if let Item::Function(func) = item {
            inline_in_block(&mut func.body, &candidates);
        }
        if let Item::Impl(impl_block) = item {
            for method in &mut impl_block.methods {
                inline_in_block(&mut method.body, &candidates);
            }
        }
    }
}

/// Information about an inlineable function.
#[derive(Clone)]
struct InlineCandidate {
    params: Vec<String>,
    body: Block,
}

/// Find functions eligible for inlining.
fn find_inline_candidates(module: &Module, threshold: usize) -> HashMap<String, InlineCandidate> {
    // Build call graph to detect recursion
    let call_graph = build_call_graph(module);
    let recursive = find_recursive_functions(&call_graph);

    // Count how many times each function is called
    let call_counts = count_calls(module);

    let mut candidates = HashMap::new();

    for item in &module.items {
        if let Item::Function(func) = item {
            // Skip main – never inline the entry point
            if func.name == "main" {
                continue;
            }

            // Skip recursive functions
            if recursive.contains(&func.name) {
                continue;
            }

            // Skip functions containing closures or complex control flow
            if has_closures_or_complex_cf(&func.body) {
                continue;
            }

            let stmt_count = func.body.statements.len();

            // Inline if small enough OR called only once
            let called_once = call_counts.get(&func.name).copied().unwrap_or(0) == 1;
            if stmt_count <= threshold || called_once {
                candidates.insert(
                    func.name.clone(),
                    InlineCandidate {
                        params: func.params.iter().map(|p| p.name.clone()).collect(),
                        body: func.body.clone(),
                    },
                );
            }
        }
    }

    candidates
}

/// Build a call graph: function name → set of functions it calls.
fn build_call_graph(module: &Module) -> HashMap<String, HashSet<String>> {
    let mut graph: HashMap<String, HashSet<String>> = HashMap::new();
    for item in &module.items {
        if let Item::Function(func) = item {
            let mut callees = HashSet::new();
            collect_callees_in_block(&func.body, &mut callees);
            graph.insert(func.name.clone(), callees);
        }
    }
    graph
}

fn collect_callees_in_block(block: &Block, callees: &mut HashSet<String>) {
    for stmt in &block.statements {
        collect_callees_in_stmt(stmt, callees);
    }
}

fn collect_callees_in_stmt(stmt: &Statement, callees: &mut HashSet<String>) {
    match stmt {
        Statement::Let {
            value: Some(value), ..
        } => collect_callees_in_expr(value, callees),
        Statement::Expression(expr) => collect_callees_in_expr(expr, callees),
        Statement::Return(Some(expr)) => collect_callees_in_expr(expr, callees),
        Statement::Assignment { value, .. } => collect_callees_in_expr(value, callees),
        Statement::If {
            condition,
            then_block,
            else_block,
        } => {
            collect_callees_in_expr(condition, callees);
            collect_callees_in_block(then_block, callees);
            if let Some(eb) = else_block {
                collect_callees_in_block(eb, callees);
            }
        }
        Statement::While { condition, body } => {
            collect_callees_in_expr(condition, callees);
            collect_callees_in_block(body, callees);
        }
        Statement::For { iter, body, .. } => {
            collect_callees_in_expr(iter, callees);
            collect_callees_in_block(body, callees);
        }
        Statement::Loop { body } => collect_callees_in_block(body, callees),
        _ => {}
    }
}

fn collect_callees_in_expr(expr: &Expression, callees: &mut HashSet<String>) {
    match expr {
        Expression::Call(func, args) => {
            if let Expression::Identifier(name) = func.as_ref() {
                callees.insert(name.clone());
            }
            collect_callees_in_expr(func, callees);
            for arg in args {
                collect_callees_in_expr(arg, callees);
            }
        }
        Expression::Binary(l, _, r) => {
            collect_callees_in_expr(l, callees);
            collect_callees_in_expr(r, callees);
        }
        Expression::Unary(_, inner) => collect_callees_in_expr(inner, callees),
        Expression::MethodCall { receiver, args, .. } => {
            collect_callees_in_expr(receiver, callees);
            for a in args {
                collect_callees_in_expr(a, callees);
            }
        }
        Expression::Array(elems) | Expression::Tuple(elems) => {
            for e in elems {
                collect_callees_in_expr(e, callees);
            }
        }
        _ => {}
    }
}

/// Detect recursive functions (direct and indirect) via call graph analysis.
fn find_recursive_functions(graph: &HashMap<String, HashSet<String>>) -> HashSet<String> {
    let mut recursive = HashSet::new();
    for (name, callees) in graph {
        // Direct recursion: function calls itself
        if callees.contains(name) {
            recursive.insert(name.clone());
            continue;
        }
        // Indirect recursion: check if any callee can reach back to us
        let mut visited = HashSet::new();
        visited.insert(name.clone());
        for callee in callees {
            if can_reach_target(graph, callee, name, &mut visited) {
                recursive.insert(name.clone());
                break;
            }
        }
    }
    recursive
}

/// Check if `current` can transitively reach `target` through the call graph.
fn can_reach_target(
    graph: &HashMap<String, HashSet<String>>,
    current: &str,
    target: &str,
    visited: &mut HashSet<String>,
) -> bool {
    if current == target {
        return true;
    }
    if !visited.insert(current.to_string()) {
        return false;
    }
    if let Some(callees) = graph.get(current) {
        for callee in callees {
            if can_reach_target(graph, callee, target, visited) {
                return true;
            }
        }
    }
    false
}

/// Count how many times each function is called across the module.
fn count_calls(module: &Module) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for item in &module.items {
        if let Item::Function(func) = item {
            count_calls_in_block(&func.body, &mut counts);
        }
    }
    counts
}

fn count_calls_in_block(block: &Block, counts: &mut HashMap<String, usize>) {
    for stmt in &block.statements {
        count_calls_in_stmt(stmt, counts);
    }
}

fn count_calls_in_stmt(stmt: &Statement, counts: &mut HashMap<String, usize>) {
    match stmt {
        Statement::Let {
            value: Some(value), ..
        } => count_calls_in_expr(value, counts),
        Statement::Expression(expr) => count_calls_in_expr(expr, counts),
        Statement::Return(Some(expr)) => count_calls_in_expr(expr, counts),
        Statement::Assignment { value, .. } => count_calls_in_expr(value, counts),
        Statement::If {
            condition,
            then_block,
            else_block,
        } => {
            count_calls_in_expr(condition, counts);
            count_calls_in_block(then_block, counts);
            if let Some(eb) = else_block {
                count_calls_in_block(eb, counts);
            }
        }
        Statement::While { condition, body } => {
            count_calls_in_expr(condition, counts);
            count_calls_in_block(body, counts);
        }
        Statement::For { iter, body, .. } => {
            count_calls_in_expr(iter, counts);
            count_calls_in_block(body, counts);
        }
        Statement::Loop { body } => count_calls_in_block(body, counts),
        _ => {}
    }
}

fn count_calls_in_expr(expr: &Expression, counts: &mut HashMap<String, usize>) {
    match expr {
        Expression::Call(func, args) => {
            if let Expression::Identifier(name) = func.as_ref() {
                *counts.entry(name.clone()).or_insert(0) += 1;
            }
            count_calls_in_expr(func, counts);
            for arg in args {
                count_calls_in_expr(arg, counts);
            }
        }
        Expression::Binary(l, _, r) => {
            count_calls_in_expr(l, counts);
            count_calls_in_expr(r, counts);
        }
        Expression::Unary(_, inner) => count_calls_in_expr(inner, counts),
        _ => {}
    }
}

/// Check if a block contains closures/lambdas or complex control flow
/// that would make inlining unsafe.
fn has_closures_or_complex_cf(block: &Block) -> bool {
    for stmt in &block.statements {
        if stmt_has_closure_or_complex(stmt) {
            return true;
        }
    }
    false
}

fn stmt_has_closure_or_complex(stmt: &Statement) -> bool {
    match stmt {
        Statement::Let {
            value: Some(value), ..
        } => expr_has_closure(value),
        Statement::Expression(expr) => expr_has_closure(expr),
        Statement::Return(Some(expr)) => expr_has_closure(expr),
        Statement::Spawn(_) => true,      // complex control flow
        Statement::Select { .. } => true, // complex control flow
        Statement::Yield(_) => true,      // generator
        _ => false,
    }
}

fn expr_has_closure(expr: &Expression) -> bool {
    match expr {
        Expression::Lambda { .. } => true,
        Expression::Generator { .. } => true,
        Expression::Call(func, args) => expr_has_closure(func) || args.iter().any(expr_has_closure),
        Expression::Binary(l, _, r) => expr_has_closure(l) || expr_has_closure(r),
        Expression::Unary(_, inner) => expr_has_closure(inner),
        _ => false,
    }
}

/// Inline function calls within a block.
fn inline_in_block(block: &mut Block, candidates: &HashMap<String, InlineCandidate>) {
    let mut new_stmts = Vec::new();
    for stmt in block.statements.drain(..) {
        match stmt {
            Statement::Expression(Expression::Call(ref func, ref args))
                if matches!(func.as_ref(), Expression::Identifier(_)) =>
            {
                if let Expression::Identifier(name) = func.as_ref() {
                    if let Some(candidate) = candidates.get(name) {
                        // Inline: emit let bindings for params, then body statements
                        let inlined = build_inline_body(candidate, args);
                        new_stmts.extend(inlined);
                        continue;
                    }
                }
                new_stmts.push(stmt);
            }
            Statement::Let {
                ref name,
                mutable,
                ref ty,
                value: Some(Expression::Call(ref func, ref args)),
            } if matches!(func.as_ref(), Expression::Identifier(_)) => {
                if let Expression::Identifier(fname) = func.as_ref() {
                    if let Some(candidate) = candidates.get(fname) {
                        let mut inlined = build_inline_body(candidate, args);
                        // Replace the last return with a let binding to `name`
                        rewrite_return_as_let(&mut inlined, name, mutable, ty.clone());
                        new_stmts.extend(inlined);
                        continue;
                    }
                }
                new_stmts.push(Statement::Let {
                    name: name.clone(),
                    mutable,
                    ty: ty.clone(),
                    value: Some(Expression::Call(func.clone(), args.clone())),
                });
            }
            other => new_stmts.push(other),
        }
    }
    block.statements = new_stmts;
}

/// Build inline body: parameter let-bindings + cloned body statements.
fn build_inline_body(candidate: &InlineCandidate, args: &[Expression]) -> Vec<Statement> {
    let mut stmts = Vec::new();

    // Bind each parameter to its argument
    for (param, arg) in candidate.params.iter().zip(args.iter()) {
        stmts.push(Statement::Let {
            name: param.clone(),
            mutable: false,
            ty: None,
            value: Some(arg.clone()),
        });
    }

    // Copy the body statements
    stmts.extend(candidate.body.statements.clone());

    stmts
}

/// Rewrite the last `return expr` in inlined body to `let name = expr`.
fn rewrite_return_as_let(stmts: &mut Vec<Statement>, name: &str, mutable: bool, ty: Option<Type>) {
    if let Some(last) = stmts.last_mut() {
        if let Statement::Return(Some(expr)) = last {
            let val = expr.clone();
            *last = Statement::Let {
                name: name.to_string(),
                mutable,
                ty,
                value: Some(val),
            };
        }
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

    fn make_fn(name: &str, params: Vec<&str>, stmts: Vec<Statement>) -> Item {
        Item::Function(Function {
            name: name.to_string(),
            is_async: false,
            attributes: vec![],
            params: params
                .into_iter()
                .map(|p| Param {
                    name: p.to_string(),
                    ty: Type::I64,
                })
                .collect(),
            return_type: None,
            effect_row: None,
            body: Block { statements: stmts },
        })
    }

    #[test]
    fn test_inline_small_function() {
        let mut module = Module {
            items: vec![
                make_fn(
                    "add_one",
                    vec!["x"],
                    vec![Statement::Return(Some(Expression::Binary(
                        Box::new(ident("x")),
                        BinaryOp::Add,
                        Box::new(int(1)),
                    )))],
                ),
                make_fn(
                    "main",
                    vec![],
                    vec![Statement::Expression(Expression::Call(
                        Box::new(ident("add_one")),
                        vec![int(5)],
                    ))],
                ),
            ],
        };
        inline_functions(&mut module, false);
        if let Item::Function(main_fn) = &module.items[1] {
            // Should have param binding + return instead of a call
            assert!(
                main_fn.body.statements.len() >= 2,
                "Expected inlined body, got {:?}",
                main_fn.body.statements
            );
        }
    }

    #[test]
    fn test_no_inline_recursive() {
        let mut module = Module {
            items: vec![
                make_fn(
                    "factorial",
                    vec!["n"],
                    vec![Statement::Return(Some(Expression::Call(
                        Box::new(ident("factorial")),
                        vec![Expression::Binary(
                            Box::new(ident("n")),
                            BinaryOp::Sub,
                            Box::new(int(1)),
                        )],
                    )))],
                ),
                make_fn(
                    "main",
                    vec![],
                    vec![Statement::Expression(Expression::Call(
                        Box::new(ident("factorial")),
                        vec![int(5)],
                    ))],
                ),
            ],
        };
        inline_functions(&mut module, false);
        // main should still have the call (factorial is recursive)
        if let Item::Function(main_fn) = &module.items[1] {
            assert_eq!(main_fn.body.statements.len(), 1);
            assert!(matches!(
                &main_fn.body.statements[0],
                Statement::Expression(Expression::Call(..))
            ));
        }
    }

    #[test]
    fn test_no_inline_with_closure() {
        let mut module = Module {
            items: vec![
                make_fn(
                    "with_lambda",
                    vec![],
                    vec![Statement::Return(Some(Expression::Lambda {
                        params: vec![],
                        body: Box::new(int(1)),
                    }))],
                ),
                make_fn(
                    "main",
                    vec![],
                    vec![Statement::Expression(Expression::Call(
                        Box::new(ident("with_lambda")),
                        vec![],
                    ))],
                ),
            ],
        };
        inline_functions(&mut module, false);
        // with_lambda has a closure, should NOT be inlined
        if let Item::Function(main_fn) = &module.items[1] {
            assert_eq!(main_fn.body.statements.len(), 1);
            assert!(matches!(
                &main_fn.body.statements[0],
                Statement::Expression(Expression::Call(..))
            ));
        }
    }

    #[test]
    fn test_no_inline_large_function() {
        let stmts: Vec<Statement> = (0..10).map(|i| Statement::Expression(int(i))).collect();
        let mut module = Module {
            items: vec![
                make_fn("big_func", vec![], stmts),
                make_fn(
                    "main",
                    vec![],
                    vec![
                        // Call twice so the "called once" heuristic doesn't apply
                        Statement::Expression(Expression::Call(
                            Box::new(ident("big_func")),
                            vec![],
                        )),
                        Statement::Expression(Expression::Call(
                            Box::new(ident("big_func")),
                            vec![],
                        )),
                    ],
                ),
            ],
        };
        inline_functions(&mut module, false);
        // big_func has 10 statements (> threshold 5) and called twice, should not inline
        if let Item::Function(main_fn) = &module.items[1] {
            assert_eq!(main_fn.body.statements.len(), 2);
        }
    }

    #[test]
    fn test_inline_called_once_even_if_large() {
        // A function called only once can be inlined regardless of size
        let stmts: Vec<Statement> = (0..8).map(|i| Statement::Expression(int(i))).collect();
        let mut module = Module {
            items: vec![
                make_fn("called_once", vec![], stmts),
                make_fn(
                    "main",
                    vec![],
                    vec![Statement::Expression(Expression::Call(
                        Box::new(ident("called_once")),
                        vec![],
                    ))],
                ),
            ],
        };
        inline_functions(&mut module, false);
        if let Item::Function(main_fn) = &module.items[1] {
            // Should be inlined: 8 body stmts
            assert!(main_fn.body.statements.len() > 1, "Expected inlined body");
        }
    }

    #[test]
    fn test_inline_with_params_substitution() {
        let mut module = Module {
            items: vec![
                make_fn(
                    "double",
                    vec!["x"],
                    vec![Statement::Return(Some(Expression::Binary(
                        Box::new(ident("x")),
                        BinaryOp::Mul,
                        Box::new(int(2)),
                    )))],
                ),
                make_fn(
                    "main",
                    vec![],
                    vec![Statement::Let {
                        name: "result".into(),
                        mutable: false,
                        ty: None,
                        value: Some(Expression::Call(Box::new(ident("double")), vec![int(21)])),
                    }],
                ),
            ],
        };
        inline_functions(&mut module, false);
        if let Item::Function(main_fn) = &module.items[1] {
            // First stmt should be: let x = 21
            assert!(main_fn.body.statements.len() >= 2);
            if let Statement::Let {
                name,
                value: Some(value),
                ..
            } = &main_fn.body.statements[0]
            {
                assert_eq!(name, "x");
                assert!(matches!(value, Expression::Literal(Literal::Int(21))));
            }
        }
    }

    #[test]
    fn test_no_inline_main() {
        let mut module = Module {
            items: vec![make_fn("main", vec![], vec![Statement::Expression(int(1))])],
        };
        let candidates = find_inline_candidates(&module, MAX_INLINE_STMTS);
        assert!(!candidates.contains_key("main"));
    }

    #[test]
    fn test_call_graph_detects_indirect_recursion() {
        // a -> b -> a (indirect recursion)
        let mut module = Module {
            items: vec![
                make_fn(
                    "a",
                    vec![],
                    vec![Statement::Expression(Expression::Call(
                        Box::new(ident("b")),
                        vec![],
                    ))],
                ),
                make_fn(
                    "b",
                    vec![],
                    vec![Statement::Expression(Expression::Call(
                        Box::new(ident("a")),
                        vec![],
                    ))],
                ),
                make_fn(
                    "main",
                    vec![],
                    vec![Statement::Expression(Expression::Call(
                        Box::new(ident("a")),
                        vec![],
                    ))],
                ),
            ],
        };
        let graph = build_call_graph(&module);
        let recursive = find_recursive_functions(&graph);
        assert!(recursive.contains("a"), "a should be detected as recursive");
        assert!(recursive.contains("b"), "b should be detected as recursive");
    }
}
