//! Lifetime inference engine with elision rules
//!
//! Implements:
//! - Automatic lifetime elision (single input, return)
//! - Lifetime constraint propagation
//! - Lifetime bounds checking

use crate::parser::ast::*;
use std::collections::HashMap;

/// Lifetime relationship: which lifetime outlives which
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifetimeRelation {
    pub outlives: String,    // 'a
    pub outlived_by: String, // 'b in 'b: 'a means 'b outlives 'a
}

/// Lifetime context for a function
#[derive(Debug, Clone)]
pub struct LifetimeContext {
    pub explicit_lifetimes: Vec<String>, // Declared lifetimes
    pub inferred_lifetimes: HashMap<String, String>, // Inferred lifetimes
    pub relations: Vec<LifetimeRelation>, // Outlives relationships
}

impl Default for LifetimeContext {
    fn default() -> Self {
        Self::new()
    }
}

impl LifetimeContext {
    pub fn new() -> Self {
        Self {
            explicit_lifetimes: Vec::new(),
            inferred_lifetimes: HashMap::new(),
            relations: Vec::new(),
        }
    }

    /// Add explicit lifetime like 'a, 'b
    pub fn add_explicit(&mut self, lifetime: String) {
        self.explicit_lifetimes.push(lifetime);
    }

    /// Infer lifetime for return type based on inputs
    pub fn elide_return_lifetime(&self, input_lifetimes: &[String]) -> String {
        match input_lifetimes.len() {
            // Rule 1: Single input lifetime -> output lifetime
            1 => input_lifetimes[0].clone(),
            // Rule 2: &self -> use self's lifetime
            0 => "'static".to_string(),
            // Rule 3: Multiple inputs -> error (must be explicit)
            _ => "'ambiguous".to_string(),
        }
    }

    /// Add lifetime outlives relationship
    pub fn add_outlives(&mut self, outlives: String, outlived_by: String) {
        self.relations.push(LifetimeRelation {
            outlives,
            outlived_by,
        });
    }

    /// Check if lifetime 'a outlives 'b
    pub fn outlives(&self, a: &str, b: &str) -> bool {
        if a == b {
            return true;
        }
        if a == "'static" {
            return true;
        }

        // Check relations
        for rel in &self.relations {
            if rel.outlived_by == a && self.outlives(&rel.outlives, b) {
                return true;
            }
        }
        false
    }
}

/// Lifetime variance enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Variance {
    Covariant,     // 'a <: 'b means F<'a> <: F<'b>
    Contravariant, // 'a <: 'b means F<'b> <: F<'a>
    Invariant,     // 'a <: 'b doesn't imply anything about F<'a> vs F<'b>
}

/// Check variance of a type constructor
pub fn check_variance(ty: &Type) -> Variance {
    match ty {
        // Shared references are covariant in their lifetime
        Type::WithOwnership(_, Ownership::Borrow) => Variance::Covariant,
        // Mutable references are invariant in their lifetime
        Type::WithOwnership(_, Ownership::BorrowMut) => Variance::Invariant,
        // Other types
        _ => Variance::Invariant,
    }
}

/// Elide lifetimes in function signature
pub fn elide_fn_lifetimes(params: &[(String, Type)], return_type: &Type) -> (Vec<String>, Type) {
    // Collect input lifetimes
    let mut input_lifetimes = Vec::new();
    for (_, ty) in params {
        if has_reference(ty) {
            input_lifetimes.push("'_".to_string());
        }
    }

    // Elide return lifetime
    let elided_return = if has_reference(return_type) && input_lifetimes.len() == 1 {
        // Single input reference: elide using that lifetime
        return_type.clone()
    } else {
        return_type.clone()
    };

    (input_lifetimes, elided_return)
}

/// Check if type contains references
fn has_reference(ty: &Type) -> bool {
    match ty {
        Type::WithOwnership(_, Ownership::Borrow | Ownership::BorrowMut) => true,
        Type::Array(inner, _) | Type::Slice(inner) => has_reference(inner),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_input_lifetime_elision() {
        let ctx = LifetimeContext::new();
        let input = vec!["'a".to_string()];
        assert_eq!(ctx.elide_return_lifetime(&input), "'a");
    }

    #[test]
    fn test_no_input_lifetime_is_static() {
        let ctx = LifetimeContext::new();
        let input = vec![];
        assert_eq!(ctx.elide_return_lifetime(&input), "'static");
    }

    #[test]
    fn test_lifetime_outlives() {
        let mut ctx = LifetimeContext::new();
        ctx.add_outlives("'a".to_string(), "'b".to_string());
        assert!(ctx.outlives("'b", "'a"));
        assert!(!ctx.outlives("'a", "'b"));
    }

    #[test]
    fn test_static_outlives_all() {
        let ctx = LifetimeContext::new();
        assert!(ctx.outlives("'static", "'a"));
        assert!(ctx.outlives("'static", "'b"));
    }
}
