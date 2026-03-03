//! Trait system with user-defined traits, bounds, and resolution
//!
//! Supports:
//! - User-defined trait definitions
//! - Generic trait bounds
//! - Associated types and where clauses
//! - Trait method resolution
//! - Trait objects (dyn Trait)
//! - Higher-ranked trait bounds (for<'a>)
//! - Generic associated types (GATs)

use crate::parser::ast::*;
use std::collections::HashMap;

/// User-defined trait with methods and associated types
#[derive(Debug, Clone)]
pub struct UserTrait {
    pub name: String,
    pub generic_params: Vec<String>,
    pub methods: Vec<TraitMethodDef>,
    pub associated_types: Vec<AssocTypeDef>,
    pub supertraits: Vec<String>,
    pub is_object_safe: bool, // can be used as dyn Trait
    pub where_clauses: Vec<WhereClause>,
}

/// Where clause in trait
#[derive(Debug, Clone)]
pub struct WhereClause {
    pub type_param: String,
    pub bounds: Vec<String>,
}

/// Trait method definition
#[derive(Debug, Clone)]
pub struct TraitMethodDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
    pub has_default: bool,
    pub is_generic_method: bool,
    pub higher_ranked_bounds: Vec<String>, // for<'a> bounds
}

/// Associated type definition
#[derive(Debug, Clone)]
pub struct AssocTypeDef {
    pub name: String,
    pub bounds: Vec<String>,
    pub default: Option<Type>,
    pub generic_params: Vec<String>, // for GATs
}

/// Trait implementation for a concrete type
#[derive(Debug, Clone)]
pub struct TraitImpl {
    pub trait_name: String,
    pub type_name: String,
    pub type_args: Vec<Type>,
    pub methods: Vec<TraitMethodImpl>,
    pub assoc_types: HashMap<String, Type>,
    pub where_clauses_satisfied: Vec<(String, String)>, // (type_param, trait)
}

/// Trait method implementation
#[derive(Debug, Clone)]
pub struct TraitMethodImpl {
    pub name: String,
    pub body: Block,
}

/// Trait bound on a type parameter
#[derive(Debug, Clone)]
pub struct TraitBound {
    pub type_param: String,
    pub trait_name: String,
    pub assoc_type_bindings: HashMap<String, Type>,
    pub where_clauses: Vec<WhereClause>,
}

/// Higher-ranked trait bound: for<'a> F: Fn(&'a T)
#[derive(Debug, Clone)]
pub struct HigherRankedBound {
    pub for_lifetimes: Vec<String>,
    pub trait_name: String,
    pub type_param: String,
}

/// Trait object: dyn Trait + Trait2 + 'a
#[derive(Debug, Clone)]
pub struct TraitObject {
    pub principal: String,           // Main trait
    pub supertraits: Vec<String>,    // Additional trait bounds
    pub lifetime: Option<String>,    // Optional lifetime bound
}

/// Trait resolver with advanced feature support
pub struct TraitResolver {
    /// User-defined traits
    traits: HashMap<String, UserTrait>,
    /// Trait implementations: (trait, type) -> impl
    impls: Vec<TraitImpl>,
    /// Builtin trait capabilities
    builtin_traits: HashMap<String, Vec<String>>,
    /// Object-safe trait cache
    object_safe_traits: HashMap<String, bool>,
    /// Higher-ranked trait bounds cache
    higher_ranked_cache: HashMap<String, HigherRankedBound>,
    /// Generic associated type cache: (trait, type_name, generic_args) -> resolved type
    gat_cache: HashMap<(String, String, String), Type>,
}

impl TraitResolver {
    pub fn new() -> Self {
        let mut resolver = Self {
            traits: HashMap::new(),
            impls: Vec::new(),
            builtin_traits: HashMap::new(),
            object_safe_traits: HashMap::new(),
            higher_ranked_cache: HashMap::new(),
            gat_cache: HashMap::new(),
        };
        resolver.init_builtin_traits();
        resolver
    }

    /// Initialize builtin traits
    fn init_builtin_traits(&mut self) {
        // Copy trait: types that are bitwise-copyable
        self.builtin_traits.insert(
            "Copy".to_string(),
            vec![
                "i8", "i16", "i32", "i64", "isize", "u8", "u16", "u32", "u64", "usize", "f32",
                "f64", "bool",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );

        // Clone trait: types that can be cloned
        self.builtin_traits.insert(
            "Clone".to_string(),
            vec![
                "i8", "i16", "i32", "i64", "isize", "u8", "u16", "u32", "u64", "usize", "f32",
                "f64", "bool", "str",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );

        // Send trait: safe to send across threads
        self.builtin_traits.insert(
            "Send".to_string(),
            vec![
                "i8", "i16", "i32", "i64", "isize", "u8", "u16", "u32", "u64", "usize", "f32",
                "f64", "bool", "str",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );

        // Sync trait: safe to share across threads
        self.builtin_traits.insert(
            "Sync".to_string(),
            vec![
                "i8", "i16", "i32", "i64", "isize", "u8", "u16", "u32", "u64", "usize", "f32",
                "f64", "bool", "str",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );

        // Debug trait: can be formatted with {:?}
        self.builtin_traits.insert(
            "Debug".to_string(),
            vec![
                "i8", "i16", "i32", "i64", "isize", "u8", "u16", "u32", "u64", "usize", "f32",
                "f64", "bool", "str",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );

        // Iterator trait (pseudo-builtin)
        self.builtin_traits.insert("Iterator".to_string(), vec![]);
    }

    /// Register a user-defined trait
    pub fn register_trait(&mut self, trait_def: UserTrait) -> Result<(), String> {
        if self.traits.contains_key(&trait_def.name) {
            return Err(format!("Trait {} already defined", trait_def.name));
        }
        self.traits.insert(trait_def.name.clone(), trait_def);
        Ok(())
    }

    /// Check if a trait is object-safe (can be used as dyn Trait)
    pub fn is_object_safe(&self, trait_name: &str) -> bool {
        if let Some(cached) = self.object_safe_traits.get(trait_name) {
            return *cached;
        }

        // A trait is object-safe if:
        // 1. It has no generic methods
        // 2. It has no associated types (or they have defaults)
        // 3. All methods have &self or &mut self receiver
        if let Some(trait_def) = self.traits.get(trait_name) {
            let has_generic_methods = trait_def.methods.iter().any(|m| !m.type_params.is_empty());
            let has_non_default_assoc = trait_def.associated_types
                .iter()
                .any(|a| a.default.is_none() && !a.generic_params.is_empty());

            !has_generic_methods && !has_non_default_assoc
        } else {
            // Builtin traits are object-safe by default
            true
        }
    }

    /// Register higher-ranked trait bound: for<'a> F: Fn(&'a T)
    pub fn register_higher_ranked(&mut self, bound: HigherRankedBound) {
        self.higher_ranked_cache.insert(bound.type_param.clone(), bound);
    }

    /// Cache generic associated type resolution
    pub fn cache_gat(
        &mut self,
        trait_name: String,
        type_name: String,
        generic_args: String,
        resolved: Type,
    ) {
        self.gat_cache.insert((trait_name, type_name, generic_args), resolved);
    }

    /// Get cached GAT
    pub fn get_cached_gat(&self, trait_name: &str, type_name: &str, generic_args: &str) -> Option<Type> {
        self.gat_cache.get(&(trait_name.to_string(), type_name.to_string(), generic_args.to_string())).cloned()
    }

    /// Register a trait implementation
    pub fn register_impl(&mut self, impl_def: TraitImpl) -> Result<(), String> {
        // Check trait exists
        if !self.traits.contains_key(&impl_def.trait_name)
            && !self.builtin_traits.contains_key(&impl_def.trait_name)
        {
            return Err(format!(
                "Trait {} not found for impl",
                impl_def.trait_name
            ));
        }
        self.impls.push(impl_def);
        Ok(())
    }

    /// Check if a type implements a trait
    pub fn implements_trait(&self, ty: &Type, trait_name: &str) -> bool {
        // Check builtin traits
        if let Some(types) = self.builtin_traits.get(trait_name) {
            match ty {
                Type::Named(n) => types.iter().any(|t| t == n),
                Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::Isize
                | Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::Usize
                | Type::F32 | Type::F64 | Type::Bool | Type::Str => {
                    let type_name = self.type_to_builtin_name(ty);
                    types.contains(&type_name)
                }
                _ => false,
            }
        } else {
            // Check user-defined trait implementations
            self.impls.iter().any(|impl_def| {
                impl_def.trait_name == trait_name
                    && self.types_match(ty, &impl_def.type_name, &impl_def.type_args)
            })
        }
    }

    /// Resolve a trait method
    pub fn resolve_method(
        &self,
        ty: &Type,
        trait_name: &str,
        method_name: &str,
    ) -> Result<TraitMethodDef, String> {
        // Find the trait definition
        let trait_def = self
            .traits
            .get(trait_name)
            .ok_or_else(|| format!("Trait {} not found", trait_name))?;

        // Find the method
        trait_def
            .methods
            .iter()
            .find(|m| m.name == method_name)
            .cloned()
            .ok_or_else(|| format!("Method {} not found in trait {}", method_name, trait_name))
    }

    /// Get associated type
    pub fn get_assoc_type(
        &self,
        ty: &Type,
        trait_name: &str,
        assoc_type_name: &str,
    ) -> Result<Type, String> {
        // Find implementation
        let impl_def = self
            .impls
            .iter()
            .find(|i| {
                i.trait_name == trait_name
                    && self.types_match(ty, &i.type_name, &i.type_args)
            })
            .ok_or_else(|| {
                format!(
                    "No impl of {} for {:?}",
                    trait_name, ty
                )
            })?;

        // Find associated type
        impl_def
            .assoc_types
            .get(assoc_type_name)
            .cloned()
            .ok_or_else(|| {
                format!(
                    "Associated type {} not found in impl",
                    assoc_type_name
                )
            })
    }

    /// Check trait bounds on a type
    pub fn check_bounds(&self, ty: &Type, bounds: &[TraitBound]) -> Result<(), String> {
        for bound in bounds {
            if !self.implements_trait(ty, &bound.trait_name) {
                return Err(format!(
                    "Type {:?} does not implement trait {}",
                    ty, bound.trait_name
                ));
            }
        }
        Ok(())
    }

    /// Helper: check if types match
    fn types_match(&self, ty: &Type, name: &str, args: &[Type]) -> bool {
        match ty {
            Type::Named(n) => n == name && args.is_empty(),
            Type::Generic(n, a) => {
                n == name && a.len() == args.len() && a.iter().zip(args).all(|(x, y)| {
                    match (x, y) {
                        (Type::Named(nx), Type::Named(ny)) => nx == ny,
                        _ => false,
                    }
                })
            }
            _ => false,
        }
    }

    /// Helper: convert type to builtin name
    fn type_to_builtin_name(&self, ty: &Type) -> String {
        match ty {
            Type::I8 => "i8".to_string(),
            Type::I16 => "i16".to_string(),
            Type::I32 => "i32".to_string(),
            Type::I64 => "i64".to_string(),
            Type::Isize => "isize".to_string(),
            Type::U8 => "u8".to_string(),
            Type::U16 => "u16".to_string(),
            Type::U32 => "u32".to_string(),
            Type::U64 => "u64".to_string(),
            Type::Usize => "usize".to_string(),
            Type::F32 => "f32".to_string(),
            Type::F64 => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Str => "str".to_string(),
            Type::Named(n) => n.clone(),
            _ => "unknown".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_trait_copy() {
        let resolver = TraitResolver::new();
        assert!(resolver.implements_trait(&Type::I64, "Copy"));
        assert!(resolver.implements_trait(&Type::Bool, "Copy"));
    }

    #[test]
    fn test_builtin_trait_clone() {
        let resolver = TraitResolver::new();
        assert!(resolver.implements_trait(&Type::Str, "Clone"));
        assert!(resolver.implements_trait(&Type::I64, "Clone"));
    }

    #[test]
    fn test_builtin_trait_send() {
        let resolver = TraitResolver::new();
        assert!(resolver.implements_trait(&Type::I32, "Send"));
    }

    #[test]
    fn test_check_bounds_success() {
        let resolver = TraitResolver::new();
        let bounds = vec![TraitBound {
            type_param: "T".to_string(),
            trait_name: "Copy".to_string(),
            assoc_type_bindings: HashMap::new(),
            where_clauses: Vec::new(),
        }];
        assert!(resolver.check_bounds(&Type::I64, &bounds).is_ok());
    }

    #[test]
    fn test_check_bounds_failure() {
        let resolver = TraitResolver::new();
        let bounds = vec![TraitBound {
            type_param: "T".to_string(),
            trait_name: "Copy".to_string(),
            assoc_type_bindings: HashMap::new(),
            where_clauses: Vec::new(),
        }];
        // String doesn't implement Copy
        assert!(resolver.check_bounds(&Type::Str, &bounds).is_err());
    }

    #[test]
    fn test_register_user_trait() {
        let mut resolver = TraitResolver::new();
        let trait_def = UserTrait {
            name: "Drawable".to_string(),
            generic_params: vec![],
            methods: vec![],
            associated_types: vec![],
            supertraits: vec![],
            is_object_safe: true,
            where_clauses: vec![],
        };
        assert!(resolver.register_trait(trait_def).is_ok());
    }

    #[test]
    fn test_duplicate_trait_registration_fails() {
        let mut resolver = TraitResolver::new();
        let trait_def = UserTrait {
            name: "MyTrait".to_string(),
            generic_params: vec![],
            methods: vec![],
            associated_types: vec![],
            supertraits: vec![],
            is_object_safe: true,
            where_clauses: vec![],        };
        resolver.register_trait(trait_def.clone()).unwrap();
        assert!(resolver.register_trait(trait_def).is_err());
    }
}