//! Advanced Type System Features for Omni v2.0
//!
//! Implements Phase 7 requirements:
//! - Implied bounds (struct bounds auto-inferred in method signatures)
//! - Variadic generics (basic form)
//! - Async traits
//! - Trait upcasting
//! - Negative bounds
//! - Custom diagnostic attributes

use crate::parser::ast::*;
use std::collections::{HashMap, HashSet};

/// Implied bounds - automatically inferred for methods based on struct definition
/// This eliminates the where-clause copy-paste ergonomic burden
#[derive(Debug, Clone)]
pub struct ImpliedBounds {
    pub struct_bounds: HashMap<String, Vec<(String, String)>>,
}

impl ImpliedBounds {
    pub fn new() -> Self {
        Self {
            struct_bounds: HashMap::new(),
        }
    }

    pub fn register_struct_bounds(
        &mut self,
        struct_name: &str,
        type_param: &str,
        trait_bounds: Vec<String>,
    ) {
        let bounds: Vec<(String, String)> = trait_bounds
            .into_iter()
            .map(|tb| (type_param.to_string(), tb))
            .collect();
        self.struct_bounds.insert(struct_name.to_string(), bounds);
    }

    pub fn get_bounds(&self, struct_name: &str) -> Vec<(String, String)> {
        self.struct_bounds
            .get(struct_name)
            .cloned()
            .unwrap_or_default()
    }

    pub fn apply_implied_bounds(
        &self,
        struct_name: &str,
        where_clauses: &mut Vec<(String, String)>,
    ) {
        let bounds = self.get_bounds(struct_name);
        for (type_param, trait_name) in bounds {
            if !where_clauses
                .iter()
                .any(|(tp, tb)| tp == &type_param && tb == &trait_name)
            {
                where_clauses.push((type_param, trait_name));
            }
        }
    }
}

impl Default for ImpliedBounds {
    fn default() -> Self {
        Self::new()
    }
}

/// Variadic generics support
#[derive(Debug, Clone)]
pub struct VariadicGenerics {
    pub variadic_params: HashMap<String, VariadicParamInfo>,
}

#[derive(Debug, Clone)]
pub struct VariadicParamInfo {
    pub param_name: String,
    pub position: usize,
    pub element_type: String,
}

impl VariadicGenerics {
    pub fn new() -> Self {
        Self {
            variadic_params: HashMap::new(),
        }
    }

    pub fn parse_variadic_param(&self, param: &str) -> Option<String> {
        if param.starts_with("..") {
            Some(param[2..].to_string())
        } else {
            None
        }
    }

    pub fn is_variadic(&self, type_param: &str) -> bool {
        self.variadic_params
            .values()
            .any(|v| v.param_name == type_param)
    }

    pub fn get_arity(&self, variadic_type: &[Type]) -> usize {
        variadic_type.len()
    }
}

impl Default for VariadicGenerics {
    fn default() -> Self {
        Self::new()
    }
}

/// Async trait support
#[derive(Debug, Clone)]
pub struct AsyncTraitSupport {
    pub async_methods: HashMap<String, Vec<String>>,
    pub future_types: HashMap<String, HashMap<String, String>>,
}

impl AsyncTraitSupport {
    pub fn new() -> Self {
        Self {
            async_methods: HashMap::new(),
            future_types: HashMap::new(),
        }
    }

    pub fn register_async_method(&mut self, trait_name: &str, method_name: &str) {
        self.async_methods
            .entry(trait_name.to_string())
            .or_default()
            .push(method_name.to_string());
    }

    pub fn is_async_method(&self, trait_name: &str, method_name: &str) -> bool {
        self.async_methods
            .get(trait_name)
            .map(|methods| methods.contains(&method_name.to_string()))
            .unwrap_or(false)
    }

    pub fn generate_future_type(
        &mut self,
        trait_name: &str,
        method_name: &str,
        return_type: &str,
    ) -> String {
        let future = format!("{}::{}::Future<{}>", trait_name, method_name, return_type);

        self.future_types
            .entry(trait_name.to_string())
            .or_default()
            .insert(method_name.to_string(), future.clone());

        future
    }
}

impl Default for AsyncTraitSupport {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait upcasting support
#[derive(Debug, Clone)]
pub struct TraitUpcasting {
    pub inheritance: HashMap<String, Vec<String>>,
    pub transitive_cache: HashMap<String, HashSet<String>>,
}

impl TraitUpcasting {
    pub fn new() -> Self {
        Self {
            inheritance: HashMap::new(),
            transitive_cache: HashMap::new(),
        }
    }

    pub fn register_supertrait(&mut self, subtrait: &str, supertrait: &str) {
        self.inheritance
            .entry(subtrait.to_string())
            .or_default()
            .push(supertrait.to_string());
        self.transitive_cache.remove(subtrait);
    }

    pub fn can_upcast(&mut self, subtrait: &str, supertrait: &str) -> bool {
        if subtrait == supertrait {
            return true;
        }
        let ancestors = self.compute_transitive_supertraits(subtrait);
        ancestors.contains(&supertrait.to_string())
    }

    pub fn compute_transitive_supertraits(&mut self, trait_name: &str) -> HashSet<String> {
        if let Some(cached) = self.transitive_cache.get(trait_name) {
            return cached.clone();
        }

        let mut visited = HashSet::new();
        let mut stack = vec![trait_name.to_string()];

        while let Some(current) = stack.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            if let Some(direct_supertraits) = self.inheritance.get(&current) {
                for supertrait in direct_supertraits {
                    stack.push(supertrait.clone());
                }
            }
        }

        visited.remove(trait_name);
        self.transitive_cache
            .insert(trait_name.to_string(), visited.clone());

        visited
    }
}

impl Default for TraitUpcasting {
    fn default() -> Self {
        Self::new()
    }
}

/// Negative bounds support
#[derive(Debug, Clone)]
pub struct NegativeBounds {
    pub negative_traits: HashMap<String, HashSet<String>>,
}

impl NegativeBounds {
    pub fn new() -> Self {
        Self {
            negative_traits: HashMap::new(),
        }
    }

    pub fn register_negative(&mut self, type_param: &str, trait_name: &str) {
        self.negative_traits
            .entry(type_param.to_string())
            .or_default()
            .insert(trait_name.to_string());
    }

    pub fn is_negative_bound(&self, type_param: &str, trait_name: &str) -> bool {
        self.negative_traits
            .get(type_param)
            .map(|traits| traits.contains(trait_name))
            .unwrap_or(false)
    }

    pub fn validate_negative_bounds(
        &self,
        type_param: &str,
        implemented_traits: &[String],
    ) -> Result<(), String> {
        if let Some(negative_set) = self.negative_traits.get(type_param) {
            for impl_trait in implemented_traits {
                if negative_set.contains(impl_trait) {
                    return Err(format!(
                        "Type parameter {} is constrained to NOT implement {}, but found implementation",
                        type_param, impl_trait
                    ));
                }
            }
        }
        Ok(())
    }
}

impl Default for NegativeBounds {
    fn default() -> Self {
        Self::new()
    }
}

/// Custom diagnostic attributes
#[derive(Debug, Clone)]
pub struct DiagnosticAttributes {
    pub custom_messages: HashMap<String, CustomDiagnostic>,
}

#[derive(Debug, Clone)]
pub struct CustomDiagnostic {
    pub message: String,
    pub label: String,
    pub help: Option<String>,
    pub note: Option<String>,
}

impl DiagnosticAttributes {
    pub fn new() -> Self {
        Self {
            custom_messages: HashMap::new(),
        }
    }

    pub fn register(&mut self, trait_name: &str, message: &str, label: &str) {
        self.custom_messages.insert(
            trait_name.to_string(),
            CustomDiagnostic {
                message: message.to_string(),
                label: label.to_string(),
                help: None,
                note: None,
            },
        );
    }

    pub fn register_full(
        &mut self,
        trait_name: &str,
        message: &str,
        label: &str,
        help: Option<String>,
        note: Option<String>,
    ) {
        self.custom_messages.insert(
            trait_name.to_string(),
            CustomDiagnostic {
                message: message.to_string(),
                label: label.to_string(),
                help,
                note,
            },
        );
    }

    pub fn get(&self, trait_name: &str) -> Option<&CustomDiagnostic> {
        self.custom_messages.get(trait_name)
    }

    pub fn generate_error(&self, trait_name: &str, self_type: &str) -> Option<String> {
        self.custom_messages
            .get(trait_name)
            .map(|diag| diag.message.replace("{Self}", self_type))
    }
}

impl Default for DiagnosticAttributes {
    fn default() -> Self {
        Self::new()
    }
}

/// Combined Phase 7 type system manager
pub struct Phase7TypeSystem {
    pub implied_bounds: ImpliedBounds,
    pub variadic_generics: VariadicGenerics,
    pub async_traits: AsyncTraitSupport,
    pub trait_upcasting: TraitUpcasting,
    pub negative_bounds: NegativeBounds,
    pub diagnostic_attributes: DiagnosticAttributes,
}

impl Phase7TypeSystem {
    pub fn new() -> Self {
        Self {
            implied_bounds: ImpliedBounds::new(),
            variadic_generics: VariadicGenerics::new(),
            async_traits: AsyncTraitSupport::new(),
            trait_upcasting: TraitUpcasting::new(),
            negative_bounds: NegativeBounds::new(),
            diagnostic_attributes: DiagnosticAttributes::new(),
        }
    }

    pub fn init_standard_implied_bounds(&mut self) {
        // Use PascalCase per Omni naming conventions
        self.implied_bounds
            .register_struct_bounds("Option", "T", vec!["Clone".to_string()]);
        self.implied_bounds
            .register_struct_bounds("Result", "T", vec!["Clone".to_string()]);
        self.implied_bounds
            .register_struct_bounds("Vec", "T", vec!["Clone".to_string()]);
        self.implied_bounds
            .register_struct_bounds("HashMap", "K", vec!["Hash".to_string()]);
    }

    pub fn init_standard_diagnostics(&mut self) {
        self.diagnostic_attributes.register(
            "Send",
            "Type `{Self}` cannot be safely sent across threads",
            "add Send bound",
        );
        self.diagnostic_attributes.register(
            "Sync",
            "Type `{Self}` cannot be safely shared across threads",
            "add Sync bound",
        );
    }
}

impl Default for Phase7TypeSystem {
    fn default() -> Self {
        let mut system = Self::new();
        system.init_standard_implied_bounds();
        system.init_standard_diagnostics();
        system
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_implied_bounds() {
        let mut implied = ImpliedBounds::new();
        implied.register_struct_bounds("Cache", "K", vec!["Hash".to_string()]);

        let bounds = implied.get_bounds("Cache");
        assert_eq!(bounds.len(), 1);
    }

    #[test]
    fn test_trait_upcasting() {
        let mut upcasting = TraitUpcasting::new();
        upcasting.register_supertrait("SubIterator", "Iterator");
        upcasting.register_supertrait("Iterator", "Any");

        assert!(upcasting.can_upcast("SubIterator", "Iterator"));
        assert!(upcasting.can_upcast("SubIterator", "Any"));
    }

    #[test]
    fn test_negative_bounds() {
        let mut neg = NegativeBounds::new();
        neg.register_negative("T", "Copy");

        assert!(neg.is_negative_bound("T", "Copy"));
    }

    #[test]
    fn test_diagnostic_attributes() {
        let mut diag = DiagnosticAttributes::new();
        diag.register("Send", "Type `{Self}` cannot be sent", "add Send bound");

        let error = diag.generate_error("Send", "MyType");
        assert!(error.is_some());
    }

    #[test]
    fn test_phase7_system() {
        let system = Phase7TypeSystem::new();
        // Verify system components are initialized
        assert!(system.variadic_generics.variadic_params.is_empty());
        assert!(system.async_traits.async_methods.is_empty());
    }
}
