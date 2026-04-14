//! Linear Type Enforcement for Omni v2.0
//!
//! Linear types are types that must be used exactly once - they cannot be
//! copied or dropped without being explicitly used. This module provides
//! the linear type checking logic.

use crate::parser::ast::{self, Expression, Statement};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LinearVar {
    pub name: String,
    pub used: bool,
    pub moved: bool,
    pub last_use: Option<Location>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Location {
    pub stmt_index: usize,
}

pub struct LinearChecker {
    linear_vars: HashMap<String, LinearVar>,
}

#[derive(Debug, Clone)]
pub enum LinearError {
    LinearNotUsed {
        var: String,
        declared_at: Location,
    },
    LinearMovedTwice {
        var: String,
        first_use: Location,
        second_use: Location,
    },
    LinearDroppedWithoutUse {
        var: String,
        drop_at: Location,
        declared_at: Location,
    },
    LinearBorrowed {
        var: String,
    },
    LinearCaptured {
        var: String,
        capture_at: Location,
    },
}

impl fmt::Display for LinearError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LinearError::LinearNotUsed { var, declared_at } => {
                write!(
                    f,
                    "linear variable '{}' not used before drop (declared at stmt {})",
                    var, declared_at.stmt_index
                )
            }
            LinearError::LinearMovedTwice {
                var,
                first_use,
                second_use,
            } => {
                write!(
                    f,
                    "linear variable '{}' used twice (first at stmt {}, second at stmt {})",
                    var, first_use.stmt_index, second_use.stmt_index
                )
            }
            LinearError::LinearDroppedWithoutUse {
                var,
                drop_at,
                declared_at,
            } => {
                write!(f, "linear variable '{}' dropped without use (dropped at stmt {}, declared at stmt {})",
                    var, drop_at.stmt_index, declared_at.stmt_index)
            }
            LinearError::LinearBorrowed { var } => {
                write!(f, "linear variable '{}' cannot be borrowed", var)
            }
            LinearError::LinearCaptured { var, capture_at } => {
                write!(
                    f,
                    "linear variable '{}' captured in closure at stmt {}",
                    var, capture_at.stmt_index
                )
            }
        }
    }
}

impl LinearChecker {
    pub fn new() -> Self {
        Self {
            linear_vars: HashMap::new(),
        }
    }

    pub fn check_module(module: &ast::Module) -> Vec<LinearError> {
        let checker = Self::new();
        checker.analyze_module(module)
    }

    fn analyze_module(&self, module: &ast::Module) -> Vec<LinearError> {
        let mut errors = Vec::new();

        for item in &module.items {
            if let ast::Item::Function(func) = item {
                let func_errors = self.analyze_function(func);
                errors.extend(func_errors);
            }
        }

        errors
    }

    fn analyze_function(&self, func: &ast::Function) -> Vec<LinearError> {
        let mut errors = Vec::new();
        let mut linear_vars: HashMap<String, LinearVar> = HashMap::new();

        for (stmt_idx, stmt) in func.body.statements.iter().enumerate() {
            let location = Location {
                stmt_index: stmt_idx,
            };
            self.analyze_statement(stmt, location, &mut linear_vars, &mut errors);
        }

        for (name, var) in &linear_vars {
            if !var.used && var.moved {
                let location = var.last_use.unwrap_or(Location { stmt_index: 0 });
                errors.push(LinearError::LinearNotUsed {
                    var: name.clone(),
                    declared_at: location,
                });
            }
        }

        errors
    }

    fn analyze_statement(
        &self,
        stmt: &Statement,
        location: Location,
        linear_vars: &mut HashMap<String, LinearVar>,
        errors: &mut Vec<LinearError>,
    ) {
        match stmt {
            Statement::Let { name, value, .. } => {
                if let Some(val) = value {
                    if self.is_linear_value(val) {
                        linear_vars.insert(
                            name.clone(),
                            LinearVar {
                                name: name.clone(),
                                used: false,
                                moved: false,
                                last_use: Some(location),
                            },
                        );
                    }
                }
            }
            Statement::Assignment { target, value } => {
                if let Some(expr) = value {
                    if let Expression::Identifier(name) = target.as_ref() {
                        if let Some(var) = linear_vars.get_mut(name) {
                            if var.moved {
                                errors.push(LinearError::LinearMovedTwice {
                                    var: name.clone(),
                                    first_use: var.last_use.unwrap_or(location),
                                    second_use: location,
                                });
                            }
                            var.used = true;
                            var.last_use = Some(location);
                            var.moved = true;
                        }
                    }

                    if self.is_linear_value(expr) {
                        if let Some(var) = linear_vars.get_mut(name) {
                            if var.moved {
                                errors.push(LinearError::LinearMovedTwice {
                                    var: name.clone(),
                                    first_use: var.last_use.unwrap_or(location),
                                    second_use: location,
                                });
                            }
                            var.moved = true;
                            var.last_use = Some(location);
                        }
                    }
                }
            }
            Statement::Return { value } => {
                if let Some(expr) = value {
                    if let Expression::Identifier(name) = expr {
                        if let Some(var) = linear_vars.get_mut(name) {
                            var.used = true;
                            var.last_use = Some(location);
                        }
                    }
                }
            }
            Statement::Expression(expr) => {
                self.analyze_expression(expr, location, linear_vars, errors);
            }
            _ => {}
        }
    }

    fn analyze_expression(
        &self,
        expr: &Expression,
        location: Location,
        linear_vars: &mut HashMap<String, LinearVar>,
        errors: &mut Vec<LinearError>,
    ) {
        match expr {
            Expression::Identifier(name) => {
                if let Some(var) = linear_vars.get_mut(name) {
                    if var.moved {
                        errors.push(LinearError::LinearMovedTwice {
                            var: name.clone(),
                            first_use: var.last_use.unwrap_or(location),
                            second_use: location,
                        });
                    }
                    var.used = true;
                    var.last_use = Some(location);
                }
            }
            Expression::Call { func, args } => {
                for arg in args {
                    if let Expression::Identifier(name) = arg {
                        if let Some(var) = linear_vars.get_mut(name) {
                            if var.moved {
                                errors.push(LinearError::LinearMovedTwice {
                                    var: name.clone(),
                                    first_use: var.last_use.unwrap_or(location),
                                    second_use: location,
                                });
                            }
                            var.moved = true;
                            var.last_use = Some(location);
                        }
                    }
                }
            }
            Expression::Lambda { params, body } => {
                for param in params {
                    if let Some(var) = linear_vars.get_mut(&param.name) {
                        errors.push(LinearError::LinearCaptured {
                            var: param.name.clone(),
                            capture_at: location,
                        });
                    }
                }
            }
            _ => {}
        }
    }

    fn is_linear_value(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Identifier(_) => true,
            Expression::Tuple(_) => true,
            Expression::Struct(_, _) => true,
            _ => false,
        }
    }
}

impl Default for LinearChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_not_used_error() {
        let source = r#"
module test
fn foo():
    let linear x = (1, 2)
"#;
    }

    #[test]
    fn test_linear_used_once() {
        let source = r#"
module test
fn foo():
    let linear x = (1, 2)
    return x
"#;
    }
}
