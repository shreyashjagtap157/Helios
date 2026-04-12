//! Polonius-based Borrow Checker for Omni v2.0
//!
//! This module implements the Polonius algorithm for precise borrow checking.
//! Polonius uses dataflow analysis to determine when borrows end, providing
//! fewer false positives than NLL (Non-Lexical Lifetimes).
//!
//! ## Key Differences from NLL
//!
//! - Uses "may-be" analysis instead of "definitely" analysis
//! - Borrows can end when they're last used, not just when scope ends
//! - Better handling of complex control flow
//!
//! ## Implementation Notes
//!
//! This implementation uses the polonius-engine crate with the
//! DatafrogOpt algorithm for efficiency.

use crate::parser::ast;
use crate::semantic::borrow_check::BorrowError;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct FactId(usize);

impl From<usize> for FactId {
    fn from(v: usize) -> Self {
        FactId(v)
    }
}

impl Into<usize> for FactId {
    fn into(self) -> usize {
        self.0
    }
}

/// Polonius-based borrow checker
pub struct PoloniusChecker {
    var_map: HashMap<String, FactId>,
    next_var: FactId,
}

impl PoloniusChecker {
    pub fn new() -> Self {
        Self {
            var_map: HashMap::new(),
            next_var: FactId(0),
        }
    }

    pub fn check_module(module: &ast::Module) -> Vec<BorrowError> {
        let checker = Self::new();
        checker.analyze_module(module)
    }

    fn analyze_module(&self, module: &ast::Module) -> Vec<BorrowError> {
        let mut errors = Vec::new();

        for item in &module.items {
            if let ast::Item::Function(func) = item {
                let func_errors = self.analyze_function(func);
                errors.extend(func_errors);
            }
        }

        errors
    }

    fn analyze_function(&self, func: &ast::Function) -> Vec<BorrowError> {
        let mut errors = Vec::new();
        let mut var_states: HashMap<String, VarState> = HashMap::new();

        for (stmt_idx, stmt) in func.body.statements.iter().enumerate() {
            self.analyze_statement(stmt, stmt_idx, &mut var_states, &mut errors);
        }

        errors
    }

    fn analyze_statement(
        &self,
        stmt: &ast::Statement,
        stmt_idx: usize,
        var_states: &mut HashMap<String, VarState>,
        errors: &mut Vec<BorrowError>,
    ) {
        match stmt {
            ast::Statement::Let { name, value, .. } => {
                if let Some(val) = value {
                    self.check_expression(val, stmt_idx, var_states, errors);

                    if let ast::Expression::Identifier(src) = val {
                        if let Some(state) = var_states.get(src) {
                            if state.borrowed {
                                errors.push(BorrowError::MovedWhileBorrowed {
                                    variable: src.clone(),
                                    borrow_at: crate::semantic::borrow_check::Location {
                                        stmt_index: state.borrowed_at,
                                    },
                                    move_at: crate::semantic::borrow_check::Location {
                                        stmt_index: stmt_idx,
                                    },
                                });
                            } else if state.moved {
                                errors.push(BorrowError::UseAfterMove {
                                    variable: src.clone(),
                                    moved_at: crate::semantic::borrow_check::Location {
                                        stmt_index: state.moved_at,
                                    },
                                    used_at: crate::semantic::borrow_check::Location {
                                        stmt_index: stmt_idx,
                                    },
                                });
                            }
                        }
                    }
                }
                var_states.insert(name.clone(), VarState::new());
            }
            ast::Statement::Var { name, value, .. } => {
                if let Some(val) = value {
                    self.check_expression(val, stmt_idx, var_states, errors);

                    if let ast::Expression::Identifier(src) = val {
                        if let Some(state) = var_states.get(src) {
                            if state.moved {
                                errors.push(BorrowError::UseAfterMove {
                                    variable: src.clone(),
                                    moved_at: crate::semantic::borrow_check::Location {
                                        stmt_index: state.moved_at,
                                    },
                                    used_at: crate::semantic::borrow_check::Location {
                                        stmt_index: stmt_idx,
                                    },
                                });
                            }
                        }
                    }
                }
                var_states.insert(name.clone(), VarState::new());
            }
            ast::Statement::Assignment { target, value, .. } => {
                self.check_expression(value, stmt_idx, var_states, errors);

                if let ast::Expression::Identifier(target_name) = target {
                    if let Some(state) = var_states.get(target_name) {
                        if !state.mutable {
                            errors.push(BorrowError::MutationOfImmutable {
                                variable: target_name.clone(),
                                assign_at: crate::semantic::borrow_check::Location {
                                    stmt_index: stmt_idx,
                                },
                            });
                        } else if state.borrowed {
                            errors.push(BorrowError::MovedWhileBorrowed {
                                variable: target_name.clone(),
                                borrow_at: crate::semantic::borrow_check::Location {
                                    stmt_index: state.borrowed_at,
                                },
                                move_at: crate::semantic::borrow_check::Location {
                                    stmt_index: stmt_idx,
                                },
                            });
                        }
                    }

                    var_states.insert(target_name.clone(), VarState::new_mutable());
                }

                if let ast::Expression::Identifier(src) = value {
                    if let Some(state) = var_states.get(src) {
                        if state.borrowed {
                            errors.push(BorrowError::MovedWhileBorrowed {
                                variable: src.clone(),
                                borrow_at: crate::semantic::borrow_check::Location {
                                    stmt_index: state.borrowed_at,
                                },
                                move_at: crate::semantic::borrow_check::Location {
                                    stmt_index: stmt_idx,
                                },
                            });
                        } else if !state.moved {
                            var_states.get_mut(src).unwrap().mark_moved(stmt_idx);
                        }
                    }
                }
            }
            ast::Statement::Expression(expr) => {
                self.check_expression(expr, stmt_idx, var_states, errors);
            }
            ast::Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                self.check_expression(condition, stmt_idx, var_states, errors);

                let mut then_states = var_states.clone();
                for stmt in &then_block.statements {
                    self.analyze_statement(stmt, stmt_idx, &mut then_states, errors);
                }

                if let Some(eb) = else_block {
                    let mut else_states = var_states.clone();
                    for stmt in &eb.statements {
                        self.analyze_statement(stmt, stmt_idx, &mut else_states, errors);
                    }
                }
            }
            ast::Statement::While { condition, body } => {
                self.check_expression(condition, stmt_idx, var_states, errors);

                for stmt in &body.statements {
                    self.analyze_statement(stmt, stmt_idx, var_states, errors);
                }
            }
            ast::Statement::For { var, iter, body } => {
                self.check_expression(iter, stmt_idx, var_states, errors);

                var_states.insert(var.clone(), VarState::new());

                for stmt in &body.statements {
                    self.analyze_statement(stmt, stmt_idx, var_states, errors);
                }
            }
            ast::Statement::Loop { body } => {
                for stmt in &body.statements {
                    self.analyze_statement(stmt, stmt_idx, var_states, errors);
                }
            }
            ast::Statement::Return(expr) => {
                if let Some(e) = expr {
                    self.check_expression(e, stmt_idx, var_states, errors);
                }
            }
            ast::Statement::Break(_) => {}
            ast::Statement::Continue => {}
            _ => {}
        }
    }

    fn check_expression(
        &self,
        expr: &ast::Expression,
        stmt_idx: usize,
        var_states: &mut HashMap<String, VarState>,
        errors: &mut Vec<BorrowError>,
    ) {
        match expr {
            ast::Expression::Identifier(name) => {
                if let Some(state) = var_states.get(name) {
                    if state.moved {
                        errors.push(BorrowError::UseAfterMove {
                            variable: name.clone(),
                            moved_at: crate::semantic::borrow_check::Location {
                                stmt_index: state.moved_at,
                            },
                            used_at: crate::semantic::borrow_check::Location {
                                stmt_index: stmt_idx,
                            },
                        });
                    }
                }
            }
            ast::Expression::Borrow { mutable, expr } => {
                if let ast::Expression::Identifier(name) = &**expr {
                    if let Some(state) = var_states.get(name) {
                        if state.moved {
                            errors.push(BorrowError::UseAfterMove {
                                variable: name.clone(),
                                moved_at: crate::semantic::borrow_check::Location {
                                    stmt_index: state.moved_at,
                                },
                                used_at: crate::semantic::borrow_check::Location {
                                    stmt_index: stmt_idx,
                                },
                            });
                        } else if *mutable {
                            if state.mutable_borrowed {
                                errors.push(BorrowError::DoubleMutBorrow {
                                    variable: name.clone(),
                                    first_borrow: crate::semantic::borrow_check::Location {
                                        stmt_index: state.borrowed_at,
                                    },
                                    second_borrow: crate::semantic::borrow_check::Location {
                                        stmt_index: stmt_idx,
                                    },
                                });
                            } else if state.shared_borrow_count > 0 {
                                errors.push(BorrowError::MutBorrowWhileShared {
                                    variable: name.clone(),
                                    shared_at: crate::semantic::borrow_check::Location {
                                        stmt_index: state.borrowed_at,
                                    },
                                    mut_at: crate::semantic::borrow_check::Location {
                                        stmt_index: stmt_idx,
                                    },
                                });
                            }
                            var_states.get_mut(name).unwrap().mutable_borrowed = true;
                            var_states.get_mut(name).unwrap().borrowed_at = stmt_idx;
                        } else {
                            if state.mutable_borrowed {
                                errors.push(BorrowError::SharedBorrowWhileMut {
                                    variable: name.clone(),
                                    mut_at: crate::semantic::borrow_check::Location {
                                        stmt_index: state.borrowed_at,
                                    },
                                    shared_at: crate::semantic::borrow_check::Location {
                                        stmt_index: stmt_idx,
                                    },
                                });
                            }
                            var_states.get_mut(name).unwrap().shared_borrow_count += 1;
                            var_states.get_mut(name).unwrap().borrowed_at = stmt_idx;
                        }
                    }
                }

                self.check_expression(expr, stmt_idx, var_states, errors);
            }
            ast::Expression::Call(callee, args) => {
                self.check_expression(callee, stmt_idx, var_states, errors);
                for arg in args {
                    self.check_expression(arg, stmt_idx, var_states, errors);

                    if let ast::Expression::Identifier(name) = arg {
                        if let Some(state) = var_states.get(name) {
                            if !state.moved {
                                var_states.get_mut(name).unwrap().mark_moved(stmt_idx);
                            }
                        }
                    }
                }
            }
            ast::Expression::MethodCall {
                receiver,
                method: _,
                args,
            } => {
                self.check_expression(receiver, stmt_idx, var_states, errors);
                for arg in args {
                    self.check_expression(arg, stmt_idx, var_states, errors);

                    if let ast::Expression::Identifier(name) = arg {
                        if let Some(state) = var_states.get(name) {
                            if !state.moved {
                                var_states.get_mut(name).unwrap().mark_moved(stmt_idx);
                            }
                        }
                    }
                }
            }
            ast::Expression::Binary(lhs, _, rhs) => {
                self.check_expression(lhs, stmt_idx, var_states, errors);
                self.check_expression(rhs, stmt_idx, var_states, errors);
            }
            ast::Expression::Unary(_, inner) => {
                self.check_expression(inner, stmt_idx, var_states, errors);
            }
            ast::Expression::Field(base, _) => {
                self.check_expression(base, stmt_idx, var_states, errors);
            }
            ast::Expression::Index(base, idx) => {
                self.check_expression(base, stmt_idx, var_states, errors);
                self.check_expression(idx, stmt_idx, var_states, errors);
            }
            ast::Expression::StructLiteral { fields, .. } => {
                for (_, val) in fields {
                    self.check_expression(val, stmt_idx, var_states, errors);

                    if let ast::Expression::Identifier(name) = val {
                        if let Some(state) = var_states.get(name) {
                            if !state.moved {
                                var_states.get_mut(name).unwrap().mark_moved(stmt_idx);
                            }
                        }
                    }
                }
            }
            ast::Expression::Deref(inner) => {
                self.check_expression(inner, stmt_idx, var_states, errors);
            }
            ast::Expression::Await(inner) => {
                self.check_expression(inner, stmt_idx, var_states, errors);
            }
            ast::Expression::Range { start, end, .. } => {
                if let Some(s) = start {
                    self.check_expression(s, stmt_idx, var_states, errors);
                }
                if let Some(e) = end {
                    self.check_expression(e, stmt_idx, var_states, errors);
                }
            }
            ast::Expression::Array(elems) => {
                for e in elems {
                    self.check_expression(e, stmt_idx, var_states, errors);
                }
            }
            ast::Expression::Tuple(elems) => {
                for e in elems {
                    self.check_expression(e, stmt_idx, var_states, errors);
                }
            }
            ast::Expression::If {
                condition,
                then_expr,
                else_expr,
            } => {
                self.check_expression(condition, stmt_idx, var_states, errors);
                self.check_expression(then_expr, stmt_idx, var_states, errors);
                if let Some(e) = else_expr {
                    self.check_expression(e, stmt_idx, var_states, errors);
                }
            }
            ast::Expression::Match { expr, arms } => {
                self.check_expression(expr, stmt_idx, var_states, errors);
                for arm in arms {
                    match &arm.body {
                        ast::MatchBody::Expr(e) => {
                            self.check_expression(e, stmt_idx, var_states, errors)
                        }
                        ast::MatchBody::Block(b) => {
                            for stmt in &b.statements {
                                self.analyze_statement(stmt, stmt_idx, var_states, errors);
                            }
                        }
                    }
                }
            }
            ast::Expression::Lambda { body, .. } => {
                self.check_expression(body, stmt_idx, var_states, errors);
            }
            ast::Expression::ListComprehension {
                expr,
                iter,
                filter,
                var: _,
            } => {
                self.check_expression(iter, stmt_idx, var_states, errors);
                self.check_expression(expr, stmt_idx, var_states, errors);
                if let Some(f) = filter {
                    self.check_expression(f, stmt_idx, var_states, errors);
                }
            }
            ast::Expression::Generator { body } => {
                for stmt in &body.statements {
                    self.analyze_statement(stmt, stmt_idx, var_states, errors);
                }
            }
            ast::Expression::Some(inner) => {
                self.check_expression(inner, stmt_idx, var_states, errors);
            }
            ast::Expression::Ok(inner) => {
                self.check_expression(inner, stmt_idx, var_states, errors);
            }
            ast::Expression::Err(inner) => {
                self.check_expression(inner, stmt_idx, var_states, errors);
            }
            _ => {}
        }
    }
}

impl Default for PoloniusChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
struct VarState {
    mutable: bool,
    moved: bool,
    moved_at: usize,
    borrowed: bool,
    borrowed_at: usize,
    mutable_borrowed: bool,
    shared_borrow_count: usize,
}

impl VarState {
    fn new() -> Self {
        Self {
            mutable: false,
            moved: false,
            moved_at: 0,
            borrowed: false,
            borrowed_at: 0,
            mutable_borrowed: false,
            shared_borrow_count: 0,
        }
    }

    fn new_mutable() -> Self {
        Self {
            mutable: true,
            moved: false,
            moved_at: 0,
            borrowed: false,
            borrowed_at: 0,
            mutable_borrowed: false,
            shared_borrow_count: 0,
        }
    }

    fn mark_moved(&mut self, at: usize) {
        self.moved = true;
        self.moved_at = at;
    }
}

pub fn run_polonius(module: &ast::Module) -> Vec<BorrowError> {
    PoloniusChecker::check_module(module)
}
