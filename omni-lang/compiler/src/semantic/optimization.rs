//! Optimization layer with caching and memoization
//!
//! Implements:
//! - Constraint solving memoization
//! - Monomorphization caching
//! - Trait resolution caching
//! - Type equivalence caching

use crate::parser::ast::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Global type cache for memoization
#[derive(Debug, Clone)]
pub struct TypeCache {
    // Type equality cache: (Type, Type) -> bool
    equality_cache: Arc<Mutex<HashMap<(String, String), bool>>>,
    // Type unification cache
    unification_cache: Arc<Mutex<HashMap<(String, String), Option<Type>>>>,
    // Monomorphization cache
    mono_cache: Arc<Mutex<HashMap<(String, Vec<String>), String>>>,
    // Trait resolution cache
    trait_cache: Arc<Mutex<HashMap<(String, String), bool>>>,
    // Associated type cache
    assoc_cache: Arc<Mutex<HashMap<(String, String), Type>>>,
}

impl TypeCache {
    pub fn new() -> Self {
        Self {
            equality_cache: Arc::new(Mutex::new(HashMap::new())),
            unification_cache: Arc::new(Mutex::new(HashMap::new())),
            mono_cache: Arc::new(Mutex::new(HashMap::new())),
            trait_cache: Arc::new(Mutex::new(HashMap::new())),
            assoc_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Cache type equality check
    pub fn cache_equality(&self, t1_str: String, t2_str: String, result: bool) {
        if let Ok(mut cache) = self.equality_cache.lock() {
            cache.insert((t1_str, t2_str), result);
        }
    }

    /// Get cached type equality
    pub fn get_equality(&self, t1_str: &str, t2_str: &str) -> Option<bool> {
        if let Ok(cache) = self.equality_cache.lock() {
            cache
                .get(&(t1_str.to_string(), t2_str.to_string()))
                .copied()
        } else {
            None
        }
    }

    /// Cache unification result
    pub fn cache_unification(&self, t1_str: String, t2_str: String, result: Option<Type>) {
        if let Ok(mut cache) = self.unification_cache.lock() {
            cache.insert((t1_str, t2_str), result);
        }
    }

    /// Get cached unification
    pub fn get_unification(&self, t1_str: &str, t2_str: &str) -> Option<Option<Type>> {
        if let Ok(cache) = self.unification_cache.lock() {
            cache
                .get(&(t1_str.to_string(), t2_str.to_string()))
                .cloned()
        } else {
            None
        }
    }

    /// Cache monomorphization
    pub fn cache_mono(&self, func_name: String, type_args: Vec<String>, mangled: String) {
        if let Ok(mut cache) = self.mono_cache.lock() {
            cache.insert((func_name, type_args), mangled);
        }
    }

    /// Get cached monomorphization
    pub fn get_mono(&self, func_name: &str, type_args: &[String]) -> Option<String> {
        if let Ok(cache) = self.mono_cache.lock() {
            cache
                .get(&(func_name.to_string(), type_args.to_vec()))
                .cloned()
        } else {
            None
        }
    }

    /// Cache trait resolution
    pub fn cache_trait(&self, ty_str: String, trait_name: String, result: bool) {
        if let Ok(mut cache) = self.trait_cache.lock() {
            cache.insert((ty_str, trait_name), result);
        }
    }

    /// Get cached trait resolution
    pub fn get_trait(&self, ty_str: &str, trait_name: &str) -> Option<bool> {
        if let Ok(cache) = self.trait_cache.lock() {
            cache
                .get(&(ty_str.to_string(), trait_name.to_string()))
                .copied()
        } else {
            None
        }
    }

    /// Cache associated type
    pub fn cache_assoc(&self, trait_type: String, type_name: String, resolved: Type) {
        if let Ok(mut cache) = self.assoc_cache.lock() {
            cache.insert((trait_type, type_name), resolved);
        }
    }

    /// Get cached associated type
    pub fn get_assoc(&self, trait_type: &str, type_name: &str) -> Option<Type> {
        if let Ok(cache) = self.assoc_cache.lock() {
            cache
                .get(&(trait_type.to_string(), type_name.to_string()))
                .cloned()
        } else {
            None
        }
    }

    /// Clear all caches
    pub fn clear(&self) {
        if let Ok(mut c) = self.equality_cache.lock() {
            c.clear();
        }
        if let Ok(mut c) = self.unification_cache.lock() {
            c.clear();
        }
        if let Ok(mut c) = self.mono_cache.lock() {
            c.clear();
        }
        if let Ok(mut c) = self.trait_cache.lock() {
            c.clear();
        }
        if let Ok(mut c) = self.assoc_cache.lock() {
            c.clear();
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            equality_entries: self.equality_cache.lock().map(|c| c.len()).unwrap_or(0),
            unification_entries: self.unification_cache.lock().map(|c| c.len()).unwrap_or(0),
            mono_entries: self.mono_cache.lock().map(|c| c.len()).unwrap_or(0),
            trait_entries: self.trait_cache.lock().map(|c| c.len()).unwrap_or(0),
            assoc_entries: self.assoc_cache.lock().map(|c| c.len()).unwrap_or(0),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub equality_entries: usize,
    pub unification_entries: usize,
    pub mono_entries: usize,
    pub trait_entries: usize,
    pub assoc_entries: usize,
}

impl CacheStats {
    pub fn total_entries(&self) -> usize {
        self.equality_entries
            + self.unification_entries
            + self.mono_entries
            + self.trait_entries
            + self.assoc_entries
    }
}

/// Constraint solving optimization
pub struct ConstraintSolvingOptimizer {
    /// Reuse previous solutions
    solution_cache: HashMap<String, Vec<(String, String)>>,
    /// Track most common constraint patterns
    pattern_frequency: HashMap<String, usize>,
}

impl ConstraintSolvingOptimizer {
    pub fn new() -> Self {
        Self {
            solution_cache: HashMap::new(),
            pattern_frequency: HashMap::new(),
        }
    }

    /// Cache constraint solution
    pub fn cache_solution(&mut self, constraints_sig: String, solution: Vec<(String, String)>) {
        self.solution_cache.insert(constraints_sig, solution);
    }

    /// Retrieve cached solution
    pub fn get_solution(&self, constraints_sig: &str) -> Option<Vec<(String, String)>> {
        self.solution_cache.get(constraints_sig).cloned()
    }

    /// Track constraint pattern frequency
    pub fn record_pattern(&mut self, pattern: String) {
        *self.pattern_frequency.entry(pattern).or_insert(0) += 1;
    }

    /// Get most frequent patterns
    pub fn hot_patterns(&self) -> Vec<(String, usize)> {
        let mut patterns: Vec<_> = self
            .pattern_frequency
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        patterns.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        patterns.into_iter().take(10).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_cache_equality() {
        let cache = TypeCache::new();
        cache.cache_equality("i32".to_string(), "i32".to_string(), true);
        assert_eq!(cache.get_equality("i32", "i32"), Some(true));
    }

    #[test]
    fn test_cache_stats() {
        let cache = TypeCache::new();
        cache.cache_equality("i32".to_string(), "i32".to_string(), true);
        let stats = cache.stats();
        assert_eq!(stats.total_entries(), 1);
    }

    #[test]
    fn test_optimizer_patterns() {
        let mut opt = ConstraintSolvingOptimizer::new();
        opt.record_pattern("unify_int".to_string());
        opt.record_pattern("unify_int".to_string());
        opt.record_pattern("unify_str".to_string());

        let patterns = opt.hot_patterns();
        assert_eq!(patterns[0].1, 2); // unify_int appears twice
    }
}
