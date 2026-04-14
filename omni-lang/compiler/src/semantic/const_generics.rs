//! Const generic support for const parameters like const N: usize
//!
//! Implements:
//! - Const parameter declarations
//! - Const expression evaluation
//! - Const type substitution

use crate::parser::ast::*;
use std::collections::HashMap;

/// Const generic parameter
#[derive(Debug, Clone)]
pub struct ConstGenericParam {
    pub name: String,
    pub ty: Type, // Usually usize
    pub default: Option<ConstValue>,
}

/// Const value for use in generics
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstValue {
    Int(i64),
    UInt(u64),
    Bool(bool),
}

impl ConstValue {
    pub fn to_type(&self) -> Type {
        match self {
            ConstValue::Int(_) => Type::I64,
            ConstValue::UInt(_) => Type::U64,
            ConstValue::Bool(_) => Type::Bool,
        }
    }
}

/// Const substitution for [T; N] style generics
#[derive(Debug, Clone)]
pub struct ConstSubstitution {
    mappings: HashMap<String, ConstValue>,
}

impl Default for ConstSubstitution {
    fn default() -> Self {
        Self::new()
    }
}

impl ConstSubstitution {
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: String, value: ConstValue) {
        self.mappings.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<&ConstValue> {
        self.mappings.get(name)
    }

    /// Apply const substitution to array type [T; N]
    pub fn apply_to_array(&self, elem_type: &Type, size_expr: Option<&Expression>) -> Type {
        // For now, just return the array type as-is
        // In production, would evaluate size_expr with substitutions
        let boxed_size = size_expr.map(|e| Box::new(e.clone()));
        Type::Array(Box::new(elem_type.clone()), boxed_size)
    }
}

/// Const expression evaluator
pub struct ConstEvaluator {
    const_bindings: HashMap<String, ConstValue>,
}

impl Default for ConstEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl ConstEvaluator {
    pub fn new() -> Self {
        Self {
            const_bindings: HashMap::new(),
        }
    }

    pub fn bind_const(&mut self, name: String, value: ConstValue) {
        self.const_bindings.insert(name, value);
    }

    /// Evaluate const expression at compile time
    pub fn evaluate(&self, expr: &Expression) -> Result<ConstValue, String> {
        match expr {
            Expression::Literal(Literal::Int(n)) => Ok(ConstValue::Int(*n)),
            Expression::Literal(Literal::Bool(b)) => Ok(ConstValue::Bool(*b)),
            Expression::Identifier(name) => self
                .const_bindings
                .get(name)
                .cloned()
                .ok_or_else(|| format!("Unknown const {}", name)),
            Expression::Binary(left, op, right) => {
                let lval = self.evaluate(left)?;
                let rval = self.evaluate(right)?;

                match (lval, rval, op) {
                    (ConstValue::UInt(l), ConstValue::UInt(r), BinaryOp::Add) => {
                        Ok(ConstValue::UInt(l + r))
                    }
                    (ConstValue::UInt(l), ConstValue::UInt(r), BinaryOp::Sub) => {
                        Ok(ConstValue::UInt(l.saturating_sub(r)))
                    }
                    (ConstValue::UInt(l), ConstValue::UInt(r), BinaryOp::Mul) => {
                        Ok(ConstValue::UInt(l * r))
                    }
                    _ => Err("Unsupported const operation".to_string()),
                }
            }
            _ => Err("Cannot evaluate non-const expression".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_const_value_to_type() {
        assert_eq!(ConstValue::Int(42).to_type(), Type::I64);
        assert_eq!(ConstValue::Bool(true).to_type(), Type::Bool);
    }

    #[test]
    fn test_const_substitution() {
        let mut subst = ConstSubstitution::new();
        subst.insert("N".to_string(), ConstValue::UInt(10));
        assert_eq!(subst.get("N"), Some(&ConstValue::UInt(10)));
    }
}
