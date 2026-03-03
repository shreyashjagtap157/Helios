//! Type inference using constraint generation and solving
//! 
//! Implements Hindley-Milner style type inference with:
//! - Constraint collection from expressions
//! - Robinson's unification algorithm
//! - Occurs check for soundness
//! - Type variable instantiation

use crate::parser::ast::*;
use std::collections::{HashMap, VecDeque};

/// Type variable for inference
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeVar {
    pub id: usize,
}

/// Substitution of type variables to concrete types
#[derive(Debug, Clone)]
pub struct TypeVarSubst {
    bindings: HashMap<usize, Type>,
    next_var_id: usize,
}

impl TypeVarSubst {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            next_var_id: 0,
        }
    }

    /// Create a fresh type variable
    pub fn fresh_var(&mut self) -> Type {
        let id = self.next_var_id;
        self.next_var_id += 1;
        Type::Named(format!("_T{}", id))
    }

    /// Apply substitution to a type
    pub fn apply(&self, ty: &Type) -> Type {
        match ty {
            Type::Named(name) if name.starts_with("_T") => {
                // Try to parse as type variable
                if let Ok(id) = name[2..].parse::<usize>() {
                    if let Some(substituted) = self.bindings.get(&id) {
                        return self.apply(substituted);
                    }
                }
                ty.clone()
            }
            Type::Array(elem, len) => {
                Type::Array(Box::new(self.apply(elem)), len.clone())
            }
            Type::Slice(elem) => {
                Type::Slice(Box::new(self.apply(elem)))
            }
            Type::Function(params, ret) => {
                let subst_params = params.iter().map(|p| self.apply(p)).collect();
                let subst_ret = ret.as_ref().map(|r| Box::new(self.apply(r)));
                Type::Function(subst_params, subst_ret)
            }
            Type::Generic(name, args) => {
                let subst_args = args.iter().map(|a| self.apply(a)).collect();
                Type::Generic(name.clone(), subst_args)
            }
            Type::WithOwnership(inner, ownership) => {
                Type::WithOwnership(Box::new(self.apply(inner)), *ownership)
            }
            other => other.clone(),
        }
    }

    /// Bind a type variable to a type
    pub fn bind(&mut self, var_id: usize, ty: Type) {
        self.bindings.insert(var_id, ty);
    }
}

/// Constraint for type inference
#[derive(Debug, Clone)]
pub enum InferConstraint {
    /// Two types must be equal
    Equals(Type, Type),
    /// A type must implement a trait
    TraitBound(Type, String),
    /// First type is subtype of second (for variance)
    Subtype(Type, Type),
}

/// Constraint solver for type inference
pub struct ConstraintSolver {
    constraints: VecDeque<InferConstraint>,
    subst: TypeVarSubst,
}

impl ConstraintSolver {
    pub fn new() -> Self {
        Self {
            constraints: VecDeque::new(),
            subst: TypeVarSubst::new(),
        }
    }

    /// Add a constraint
    pub fn add_constraint(&mut self, constraint: InferConstraint) {
        self.constraints.push_back(constraint);
    }

    /// Check if two types can be unified
    pub fn unify(&mut self, t1: &Type, t2: &Type) -> Result<Type, String> {
        let t1 = self.subst.apply(t1);
        let t2 = self.subst.apply(t2);

        // Same type
        if self.types_equal(&t1, &t2) {
            return Ok(t1);
        }

        // Type variables
        if let Type::Named(n) = &t1 {
            if n.starts_with("_T") {
                return self.unify_var(&t1, &t2);
            }
        }
        if let Type::Named(n) = &t2 {
            if n.starts_with("_T") {
                return self.unify_var(&t2, &t1);
            }
        }

        // Structured types
        match (&t1, &t2) {
            (Type::Array(e1, _), Type::Array(e2, _)) => {
                self.unify(e1, e2)
            }
            (Type::Slice(e1), Type::Slice(e2)) => {
                self.unify(e1, e2)
            }
            (Type::Function(p1, r1), Type::Function(p2, r2)) => {
                if p1.len() != p2.len() {
                    return Err(format!("Function arity mismatch: {} vs {}", p1.len(), p2.len()));
                }
                // Unify parameters
                for (a, b) in p1.iter().zip(p2.iter()) {
                    self.unify(a, b)?;
                }
                // Unify returns
                match (r1, r2) {
                    (Some(r1), Some(r2)) => self.unify(r1, r2),
                    (None, None) => Ok(Type::Named("()".to_string())),
                    _ => Err("Return type mismatch".to_string()),
                }
            }
            // Numeric coercions
            (Type::I32, Type::I64) | (Type::I64, Type::I32) => Ok(Type::I64),
            (Type::F32, Type::F64) | (Type::F64, Type::F32) => Ok(Type::F64),

            _ => Err(format!("Cannot unify {:?} with {:?}", t1, t2)),
        }
    }

    /// Unify where one is a type variable
    fn unify_var(&mut self, var: &Type, ty: &Type) -> Result<Type, String> {
        if let Type::Named(n) = var {
            if n.starts_with("_T") {
                if let Ok(id) = n[2..].parse::<usize>() {
                    // Occurs check: ensure var doesn't appear in ty
                    if self.occurs_check(id, ty) {
                        return Err(format!("Occurs check failed: {:?} in {:?}", var, ty));
                    }
                    self.subst.bind(id, ty.clone());
                    return Ok(ty.clone());
                }
            }
        }
        Err(format!("Cannot unify {:?} with {:?}", var, ty))
    }

    /// Occurs check: does type variable appear in type?
    fn occurs_check(&self, var_id: usize, ty: &Type) -> bool {
        let ty = self.subst.apply(ty);
        match ty {
            Type::Named(n) if n.starts_with("_T") => {
                if let Ok(id) = n[2..].parse::<usize>() {
                    id == var_id
                } else {
                    false
                }
            }
            Type::Array(elem, _) => self.occurs_check(var_id, &elem),
            Type::Slice(elem) => self.occurs_check(var_id, &elem),
            Type::Function(params, ret) => {
                params.iter().any(|p| self.occurs_check(var_id, p))
                    || ret.as_ref().map_or(false, |r| self.occurs_check(var_id, r))
            }
            Type::Generic(_, args) => {
                args.iter().any(|a| self.occurs_check(var_id, a))
            }
            Type::WithOwnership(inner, _) => self.occurs_check(var_id, &inner),
            _ => false,
        }
    }

    /// Solve all constraints
    pub fn solve(&mut self) -> Result<TypeVarSubst, String> {
        while let Some(constraint) = self.constraints.pop_front() {
            match constraint {
                InferConstraint::Equals(t1, t2) => {
                    self.unify(&t1, &t2)?;
                }
                InferConstraint::TraitBound(_, _) => {
                    // Trait bounds checked separately in analyzer
                }
                InferConstraint::Subtype(_, _) => {
                    // Subtyping constraints checked separately
                }
            }
        }
        Ok(self.subst.clone())
    }

    /// Check type equality (same structure)
    fn types_equal(&self, a: &Type, b: &Type) -> bool {
        let a = self.subst.apply(a);
        let b = self.subst.apply(b);
        match (&a, &b) {
            (Type::U8, Type::U8) | (Type::U16, Type::U16) | (Type::U32, Type::U32)
            | (Type::U64, Type::U64) | (Type::Usize, Type::Usize)
            | (Type::I8, Type::I8) | (Type::I16, Type::I16) | (Type::I32, Type::I32)
            | (Type::I64, Type::I64) | (Type::Isize, Type::Isize)
            | (Type::F32, Type::F32) | (Type::F64, Type::F64)
            | (Type::Bool, Type::Bool) | (Type::Str, Type::Str)
            | (Type::SelfOwned, Type::SelfOwned) => true,

            (Type::Named(n1), Type::Named(n2)) => n1 == n2,

            (Type::Array(e1, _), Type::Array(e2, _)) => self.types_equal(e1, e2),
            (Type::Slice(e1), Type::Slice(e2)) => self.types_equal(e1, e2),

            (Type::Function(p1, r1), Type::Function(p2, r2)) => {
                p1.len() == p2.len()
                    && p1.iter()
                        .zip(p2.iter())
                        .all(|(a, b)| self.types_equal(a, b))
                    && match (r1, r2) {
                        (Some(r1), Some(r2)) => self.types_equal(r1, r2),
                        (None, None) => true,
                        _ => false,
                    }
            }

            (Type::Generic(n1, a1), Type::Generic(n2, a2)) => {
                n1 == n2
                    && a1.len() == a2.len()
                    && a1.iter()
                        .zip(a2.iter())
                        .all(|(x, y)| self.types_equal(x, y))
            }

            (Type::WithOwnership(t1, o1), Type::WithOwnership(t2, o2)) => {
                o1 == o2 && self.types_equal(t1, t2)
            }

            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unify_identical_types() {
        let mut solver = ConstraintSolver::new();
        let result = solver.unify(&Type::I64, &Type::I64);
        assert!(result.is_ok());
        assert!(solver.types_equal(&result.unwrap(), &Type::I64));
    }

    #[test]
    fn test_unify_different_types_fails() {
        let mut solver = ConstraintSolver::new();
        let result = solver.unify(&Type::I64, &Type::Str);
        assert!(result.is_err());
    }

    #[test]
    fn test_unify_numeric_coercion() {
        let mut solver = ConstraintSolver::new();
        let result = solver.unify(&Type::I32, &Type::I64);
        assert!(result.is_ok());
        assert!(solver.types_equal(&result.unwrap(), &Type::I64));
    }

    #[test]
    fn test_unify_array_types() {
        let mut solver = ConstraintSolver::new();
        let arr1 = Type::Array(Box::new(Type::I64), None);
        let arr2 = Type::Array(Box::new(Type::I64), None);
        let result = solver.unify(&arr1, &arr2);
        assert!(result.is_ok());
    }

    #[test]
    fn test_occurs_check() {
        let mut solver = ConstraintSolver::new();
        let var = solver.subst.fresh_var();
        let recursive_type = Type::Function(vec![var.clone()], Some(Box::new(var.clone())));
        
        // This should fail the occurs check
        let result = solver.unify(&var, &recursive_type);
        assert!(result.is_err());
    }

    #[test]
    fn test_constraint_solving() {
        let mut solver = ConstraintSolver::new();
        solver.add_constraint(InferConstraint::Equals(Type::I64, Type::I64));
        let result = solver.solve();
        assert!(result.is_ok());
    }
}
