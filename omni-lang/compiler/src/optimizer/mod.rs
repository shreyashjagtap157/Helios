//! Omni Compiler Optimization Passes
//!
//! Provides AST-level optimization passes that run after parsing
//! and before IR generation. Each pass transforms the AST in-place.

pub mod constant_folding;
pub mod dead_code;
pub mod inlining;
pub mod simplify;

use crate::parser::ast::Module;

/// Optimization level controlling which passes run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OptLevel {
    /// No optimizations.
    O0,
    /// Basic optimizations: constant folding + simplification.
    O1,
    /// Standard optimizations: all passes.
    O2,
    /// Aggressive optimizations: all passes with aggressive inlining.
    O3,
}

/// Run all optimization passes on a module in sequence.
///
/// The order matters: constant folding first exposes opportunities
/// for simplification, which in turn exposes dead code. Inlining
/// runs last to avoid interfering with other analyses.
pub fn optimize(module: &mut Module, level: OptLevel) {
    if level == OptLevel::O0 {
        return;
    }

    // Pass 1: Constant folding – evaluate compile-time constants
    constant_folding::fold_constants(module);

    // Pass 2: Expression simplification – algebraic identities
    simplify::simplify_expressions(module);

    // Pass 3: Constant folding again – simplification may expose new constants
    constant_folding::fold_constants(module);

    if level >= OptLevel::O2 {
        // Pass 4: Dead code elimination
        dead_code::eliminate_dead_code(module);

        // Pass 5: Function inlining
        let aggressive = level == OptLevel::O3;
        inlining::inline_functions(module, aggressive);

        // Pass 6: Clean up after inlining
        constant_folding::fold_constants(module);
        simplify::simplify_expressions(module);
        dead_code::eliminate_dead_code(module);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::*;

    #[test]
    fn test_o0_no_change() {
        let mut module = Module { items: vec![] };
        optimize(&mut module, OptLevel::O0);
        assert!(module.items.is_empty());
    }

    #[test]
    fn test_optimize_runs_without_panic() {
        let mut module = Module {
            items: vec![Item::Function(Function {
                name: "main".into(),
                is_async: false,
                attributes: vec![],
                params: vec![],
                return_type: None,
                body: Block {
                    statements: vec![Statement::Let {
                        name: "x".into(),
                        mutable: false,
                        ty: None,
                        value: Some(Expression::Binary(
                            Box::new(Expression::Literal(Literal::Int(2))),
                            BinaryOp::Add,
                            Box::new(Expression::Literal(Literal::Int(3))),
                        )),
                    }],
                },
            })],
        };
        optimize(&mut module, OptLevel::O2);
    }
}
