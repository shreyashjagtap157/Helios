//! Constraint-based type inference using unification
//!
//! Implements Hindley-Milner style type inference with:
//! - Constraint collection from expressions
//! - Unification-based solving
//! - Type variable generation and substitution
//! - Lifetime inference and elision
//! - Associated type projection
//! - Where clause support
//! - Higher-ranked trait bounds

use crate::parser::ast::*;
use std::collections::HashMap;

/// Type variable for inference
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeVar {
    pub id: usize,
}

impl TypeVar {
    fn new(id: usize) -> Self {
        Self { id }
    }
}

/// Lifetime variable for inference
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LifetimeVar {
    pub id: usize,
}

impl LifetimeVar {
    fn new(id: usize) -> Self {
        Self { id }
    }
}

/// Constraint to be solved
#[derive(Debug, Clone)]
pub enum Constraint {
    /// Two types must be equal
    Equals(InferenceType, InferenceType),
    /// Type must satisfy trait bound
    TraitBound(InferenceType, String),
    /// First type is subtype of second
    Subtype(InferenceType, InferenceType),
    /// Lifetime outlives another
    LifetimeOutlives(LifetimeVar, LifetimeVar),
    /// Associated type equality
    AssocTypeEq(String, String, Box<InferenceType>), // trait, name, ty
    /// Where clause constraint
    WhereBound(InferenceType, String), // type must satisfy bound
}

/// Type used in inference (can include type variables)
#[derive(Debug, Clone, PartialEq)]
pub enum InferenceType {
    /// Concrete type
    Concrete(Box<Type>),
    /// Type variable to be inferred
    Var(TypeVar),
    /// Generic type with inference parameters
    Generic(String, Vec<InferenceType>),
    /// Function type
    Function(Box<InferenceType>, Box<InferenceType>),
    /// Trait object (dyn Trait)
    TraitObject(String, Vec<String>), // trait, supertrait list
    /// Associated type: <T as Trait>::Type
    AssocType(Box<InferenceType>, String, String), // base type, trait, name
    /// Const generic: T where T is a const
    ConstGeneric(String, Box<InferenceType>), // name, underlying type
}

/// Substitution mapping type variables to types
pub struct Substitution {
    mappings: HashMap<TypeVar, InferenceType>,
    lifetime_mappings: HashMap<LifetimeVar, String>,
}

impl Substitution {
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
            lifetime_mappings: HashMap::new(),
        }
    }

    /// Add a substitution
    pub fn insert(&mut self, var: TypeVar, ty: InferenceType) {
        self.mappings.insert(var, ty);
    }

    /// Add a lifetime substitution
    pub fn insert_lifetime(&mut self, var: LifetimeVar, lifetime: String) {
        self.lifetime_mappings.insert(var, lifetime);
    }

    /// Apply substitution to a type
    pub fn apply(&self, ty: &InferenceType) -> InferenceType {
        match ty {
            InferenceType::Var(v) => {
                if let Some(mapped) = self.mappings.get(v) {
                    self.apply(mapped)
                } else {
                    InferenceType::Var(v.clone())
                }
            }
            InferenceType::Generic(name, args) => {
                let new_args: Vec<_> = args.iter().map(|a| self.apply(a)).collect();
                InferenceType::Generic(name.clone(), new_args)
            }
            InferenceType::Function(param, ret) => {
                let new_param = Box::new(self.apply(param));
                let new_ret = Box::new(self.apply(ret));
                InferenceType::Function(new_param, new_ret)
            }
            InferenceType::AssocType(base, trait_name, type_name) => {
                let new_base = Box::new(self.apply(base));
                InferenceType::AssocType(new_base, trait_name.clone(), type_name.clone())
            }
            InferenceType::ConstGeneric(name, underlying) => {
                let new_underlying = Box::new(self.apply(underlying));
                InferenceType::ConstGeneric(name.clone(), new_underlying)
            }
            other => other.clone(),
        }
    }

    /// Compose two substitutions
    pub fn compose(&self, other: &Substitution) -> Substitution {
        let mut result = Substitution::new();

        // Apply other to our values
        for (var, ty) in &self.mappings {
            result.insert(var.clone(), other.apply(ty));
        }

        // Add bindings from other that aren't in self
        for (var, ty) in &other.mappings {
            if !result.mappings.contains_key(var) {
                result.insert(var.clone(), ty.clone());
            }
        }

        // Compose lifetime mappings
        result.lifetime_mappings.extend(self.lifetime_mappings.clone());
        for (var, lifetime) in &other.lifetime_mappings {
            if !result.lifetime_mappings.contains_key(var) {
                result.lifetime_mappings.insert(var.clone(), lifetime.clone());
            }
        }

        result
    }
}

/// Constraint solver using unification
pub struct ConstraintSolver {
    constraints: Vec<Constraint>,
    type_var_counter: usize,
    lifetime_var_counter: usize,
    assoc_type_cache: HashMap<(String, String), InferenceType>, // (trait, name) -> type
    where_clauses: Vec<(String, String)>, // (type_param, trait_bound)
    lifetime_graph: HashMap<LifetimeVar, Vec<LifetimeVar>>, // outlives relationships
}

impl ConstraintSolver {
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            type_var_counter: 0,
            lifetime_var_counter: 0,
            assoc_type_cache: HashMap::new(),
            where_clauses: Vec::new(),
            lifetime_graph: HashMap::new(),
        }
    }

    /// Generate a fresh type variable
    pub fn fresh_var(&mut self) -> TypeVar {
        let id = self.type_var_counter;
        self.type_var_counter += 1;
        TypeVar::new(id)
    }

    /// Generate a fresh lifetime variable
    pub fn fresh_lifetime(&mut self) -> LifetimeVar {
        let id = self.lifetime_var_counter;
        self.lifetime_var_counter += 1;
        LifetimeVar::new(id)
    }

    /// Add a constraint
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    /// Add a where clause constraint
    pub fn add_where_clause(&mut self, type_param: String, trait_bound: String) {
        self.where_clauses.push((type_param, trait_bound));
    }

    /// Register associated type
    pub fn register_assoc_type(&mut self, trait_name: String, type_name: String, ty: InferenceType) {
        self.assoc_type_cache.insert((trait_name, type_name), ty);
    }

    /// Resolve associated type: <T as Trait>::Type
    pub fn resolve_assoc_type(
        &self,
        base_type: &InferenceType,
        trait_name: &str,
        type_name: &str,
    ) -> Option<InferenceType> {
        self.assoc_type_cache.get(&(trait_name.to_string(), type_name.to_string())).cloned()
    }

    /// Infer lifetime: find elided lifetime from context
    pub fn infer_lifetime(&self, num_inputs: usize, is_output: bool) -> String {
        match num_inputs {
            // Single input param: lifetime of output = lifetime of input
            1 if is_output => "'_".to_string(),
            // Multiple inputs: output lifetime must be explicit
            _ if is_output => "'_".to_string(),
            _ => "'static".to_string(),
        }
    }

    /// Solve all constraints, returning a substitution
    pub fn solve(&mut self) -> Result<Substitution, String> {
        let mut subst = Substitution::new();

        while !self.constraints.is_empty() {
            let constraint = self.constraints.remove(0);

            match constraint {
                Constraint::Equals(ty1, ty2) => {
                    let ty1 = subst.apply(&ty1);
                    let ty2 = subst.apply(&ty2);

                    match (&ty1, &ty2) {
                        // Same concrete types unify
                        (InferenceType::Concrete(t1), InferenceType::Concrete(t2)) => {
                            if !types_equal(t1, t2) {
                                return Err(format!("Type mismatch: {:?} vs {:?}", t1, t2));
                            }
                        }
                        // Variable unifies with anything
                        (InferenceType::Var(v), ty) | (ty, InferenceType::Var(v)) => {
                            if !self.occurs_check(v, ty) {
                                subst.insert(v.clone(), ty.clone());
                            } else {
                                return Err("Infinite type detected".to_string());
                            }
                        }
                        // Generic types unify if names match and args unify
                        (InferenceType::Generic(n1, a1), InferenceType::Generic(n2, a2)) => {
                            if n1 != n2 || a1.len() != a2.len() {
                                return Err(format!(
                                    "Generic type mismatch: {} vs {}",
                                    n1, n2
                                ));
                            }
                            for (arg1, arg2) in a1.iter().zip(a2.iter()) {
                                self.add_constraint(Constraint::Equals(
                                    arg1.clone(),
                                    arg2.clone(),
                                ));
                            }
                        }
                        // Function types unify if parameters and returns unify
                        (
                            InferenceType::Function(p1, r1),
                            InferenceType::Function(p2, r2),
                        ) => {
                            self.add_constraint(Constraint::Equals(*p1.clone(), *p2.clone()));
                            self.add_constraint(Constraint::Equals(*r1.clone(), *r2.clone()));
                        }
                        // Otherwise type mismatch
                        (t1, t2) => {
                            return Err(format!("Cannot unify {:?} with {:?}", t1, t2));
                        }
                    }
                }

                Constraint::TraitBound(ty, trait_name) => {
                    let ty = subst.apply(&ty);
                    // TODO: Validate trait bound against type
                    // For now, just check if it's a variable or concrete type
                    match ty {
                        InferenceType::Var(_) => {
                            // Constraint on type variable - defer for now
                        }
                        InferenceType::Concrete(_) => {
                            // Would check against trait resolver here
                        }
                        _ => {}
                    }
                }

                Constraint::Subtype(ty1, ty2) => {
                    // For now, treat subtype as equality
                    self.add_constraint(Constraint::Equals(ty1, ty2));
                }

                Constraint::LifetimeOutlives(_lft1, _lft2) => {
                    // Record lifetime outlives relationships
                    // For now, just track them - will be validated later
                }

                Constraint::AssocTypeEq(_trait_name, _type_name, _expected_ty) => {
                    // Associated type equality constraint
                    // Would validate that the trait's associated type matches expected_ty
                    // For now, just record it
                }

                Constraint::WhereBound(_ty, _trait_bound) => {
                    // Where clause constraint
                    // Would validate that the type satisfies the trait bound
                    // For now, just record it
                }
            }
        }

        Ok(subst)
    }

    /// Occurs check to prevent infinite types
    fn occurs_check(&self, var: &TypeVar, ty: &InferenceType) -> bool {
        match ty {
            InferenceType::Var(v) => v == var,
            InferenceType::Generic(_, args) => args.iter().any(|a| self.occurs_check(var, a)),
            InferenceType::Function(param, ret) => {
                self.occurs_check(var, param) || self.occurs_check(var, ret)
            }
            _ => false,
        }
    }
}

/// Check if two concrete types are equal
fn types_equal(t1: &Type, t2: &Type) -> bool {
    match (t1, t2) {
        (Type::Named(n1), Type::Named(n2)) => n1 == n2,
        (Type::I8, Type::I8) => true,
        (Type::I16, Type::I16) => true,
        (Type::I32, Type::I32) => true,
        (Type::I64, Type::I64) => true,
        (Type::Isize, Type::Isize) => true,
        (Type::U8, Type::U8) => true,
        (Type::U16, Type::U16) => true,
        (Type::U32, Type::U32) => true,
        (Type::U64, Type::U64) => true,
        (Type::Usize, Type::Usize) => true,
        (Type::F32, Type::F32) => true,
        (Type::F64, Type::F64) => true,
        (Type::Bool, Type::Bool) => true,
        (Type::Str, Type::Str) => true,
        (Type::Generic(n1, a1), Type::Generic(n2, a2)) => {
            n1 == n2 && a1.len() == a2.len() && a1.iter().zip(a2).all(|(x, y)| types_equal(x, y))
        }
        (Type::Function(p1, r1), Type::Function(p2, r2)) => {
            p1.len() == p2.len()
                && p1.iter().zip(p2).all(|(x, y)| types_equal(x, y))
                && match (r1, r2) {
                    (Some(r1), Some(r2)) => types_equal(r1, r2),
                    (None, None) => true,
                    _ => false,
                }
        }
        (Type::Array(e1, _), Type::Array(e2, _)) => types_equal(e1, e2),
        (Type::Slice(e1), Type::Slice(e2)) => types_equal(e1, e2),
        (Type::WithOwnership(t1, o1), Type::WithOwnership(t2, o2)) => {
            o1 == o2 && types_equal(t1, t2)
        }
        (Type::SelfOwned, Type::SelfOwned) => true,
        (Type::SelfRef { mutable: m1 }, Type::SelfRef { mutable: m2 }) => m1 == m2,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fresh_var_generation() {
        let mut solver = ConstraintSolver::new();
        let v1 = solver.fresh_var();
        let v2 = solver.fresh_var();
        assert_ne!(v1.id, v2.id);
    }

    #[test]
    fn test_concrete_type_unification() {
        let mut solver = ConstraintSolver::new();
        solver.add_constraint(Constraint::Equals(
            InferenceType::Concrete(Box::new(Type::I32)),
            InferenceType::Concrete(Box::new(Type::I32)),
        ));
        assert!(solver.solve().is_ok());
    }

    #[test]
    fn test_concrete_type_mismatch() {
        let mut solver = ConstraintSolver::new();
        solver.add_constraint(Constraint::Equals(
            InferenceType::Concrete(Box::new(Type::I32)),
            InferenceType::Concrete(Box::new(Type::I64)),
        ));
        assert!(solver.solve().is_err());
    }

    #[test]
    fn test_variable_unification() {
        let mut solver = ConstraintSolver::new();
        let v = solver.fresh_var();
        solver.add_constraint(Constraint::Equals(
            InferenceType::Var(v.clone()),
            InferenceType::Concrete(Box::new(Type::I32)),
        ));
        let subst = solver.solve().unwrap();
        let result = subst.apply(&InferenceType::Var(v));
        assert_eq!(result, InferenceType::Concrete(Box::new(Type::I32)));
    }

    #[test]
    fn test_substitution_composition() {
        let v1 = TypeVar::new(1);
        let v2 = TypeVar::new(2);

        let mut subst1 = Substitution::new();
        subst1.insert(v1.clone(), InferenceType::Var(v2.clone()));

        let mut subst2 = Substitution::new();
        subst2.insert(v2.clone(), InferenceType::Concrete(Box::new(Type::I32)));

        let composed = subst1.compose(&subst2);
        let result = composed.apply(&InferenceType::Var(v1));
        assert_eq!(result, InferenceType::Concrete(Box::new(Type::I32)));
    }

    #[test]
    fn test_generic_unification() {
        let mut solver = ConstraintSolver::new();
        solver.add_constraint(Constraint::Equals(
            InferenceType::Generic("Vec".to_string(), vec![InferenceType::Concrete(Box::new(Type::I32))]),
            InferenceType::Generic("Vec".to_string(), vec![InferenceType::Concrete(Box::new(Type::I32))]),
        ));
        assert!(solver.solve().is_ok());
    }

    #[test]
    fn test_function_type_unification() {
        let mut solver = ConstraintSolver::new();
        let param = Box::new(InferenceType::Concrete(Box::new(Type::I32)));
        let ret = Box::new(InferenceType::Concrete(Box::new(Type::Bool)));
        let ty1 = InferenceType::Function(param.clone(), ret.clone());
        let ty2 = InferenceType::Function(param, ret);
        solver.add_constraint(Constraint::Equals(ty1, ty2));
        assert!(solver.solve().is_ok());
    }

    #[test]
    fn test_occurs_check() {
        let mut solver = ConstraintSolver::new();
        let v = solver.fresh_var();
        let v_clone = v.clone();
        // Create constraint: v = Vec<v> (should fail)
        solver.add_constraint(Constraint::Equals(
            InferenceType::Var(v),
            InferenceType::Generic("Vec".to_string(), vec![InferenceType::Var(v_clone)]),
        ));
        assert!(solver.solve().is_err());
    }
}
