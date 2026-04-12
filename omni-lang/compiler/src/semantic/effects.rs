//! Effect System for Omni v2.0
//!
//! This module provides the foundational effect system that enables fine-grained
//! computational effects (I/O, exceptions, async, nondeterminism) with clean
//! effect handlers. The design is inspired by Koka/Frank/ALEFF but simplified
//! for practical use in a systems language.
//!
//! ## Core Concepts
//!
//! - **Effects**: Tagged algebraic effects that describe what a computation *does*
//! - **Handlers**: Scoped interpreters that *handle* effects with custom behavior
//! - **Effect rows**: Open, extensible sets of effects (like a kind of union type)
//! - **Purity**: A function is pure if its effect row is empty (no effects)
//!
//! ## Built-in Effects
//!
//! - `IO`         — File, console, network I/O
//! - `Async`      — Asynchronous computation / green threads
//! - `State`      — Mutable state access
//! - `Error`      — Error handling with error payload
//! - `NonDet`     — Nondeterministic choice
//! - `Debug`      — Debugging / logging
//! - `Alloc`      — Dynamic memory allocation
//! - `Yield`      — Cooperative yielding (for generators/coroutines)
//! - `Div`        — Divergence (infinite loops, panics)
//!
//! ## Effect Row Examples
//!
//! ```ignore
//! pure_fn(x: Int) -> Int        // effect row: {} (empty = pure)
//! io_fn(x: Int) -> Int          // effect row: {IO}
//! async_fn(x: Int) -> Int        // effect row: {Async}
//! state_fn(x: Int) -> Int        // effect row: {State}
//! multi_fn(x: Int) -> Int       // effect row: {IO, State}
//! ```

// ============================================================================
// Effect Symbol — the basic unit of an effect
// ============================================================================

/// A named effect symbol (the "tag" of an algebraic effect).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EffectSymbol {
    /// The name of the effect (e.g., "IO", "Async", "State").
    pub name: String,
    /// Optional type parameter (for polymorphic effects like Error[E]).
    pub param: Option<String>,
}

impl EffectSymbol {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            param: None,
        }
    }

    pub fn with_param(mut self, param: &str) -> Self {
        self.param = Some(param.to_string());
        self
    }

    pub fn display_name(&self) -> String {
        match &self.param {
            Some(p) => format!("{}[{}]", self.name, p),
            None => self.name.clone(),
        }
    }
}

impl std::fmt::Display for EffectSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

// ============================================================================
// Effect Row — an open set of effects
// ============================================================================

/// An effect row represents an open, unordered set of effects.
/// Two effect rows can be combined with `+` (union).
/// Empty row means the computation is pure (no effects).
///
/// Examples:
/// - `{}`        — pure (no effects)
/// - `{IO}`      — has I/O effects
/// - `{IO, State}` — has I/O and state effects
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectRow {
    /// Sorted, deduplicated list of effect symbols.
    effects: Vec<EffectSymbol>,
}

impl Default for EffectRow {
    fn default() -> Self {
        Self::pure()
    }
}

impl EffectRow {
    /// Create an effect row from a vector of effect symbols.
    pub fn from_effects(effects: Vec<EffectSymbol>) -> Self {
        let mut row = Self { effects };
        row.normalize();
        row
    }

    /// The empty effect row — represents a pure computation.
    pub fn pure() -> Self {
        Self {
            effects: Vec::new(),
        }
    }

    /// The universal effect row — represents a computation with arbitrary effects.
    pub fn any() -> Self {
        Self {
            effects: vec![EffectSymbol::new("*")],
        }
    }

    /// Create a singleton effect row containing one effect.
    pub fn just(effect: EffectSymbol) -> Self {
        let mut row = Self {
            effects: vec![effect],
        };
        row.normalize();
        row
    }

    /// Check if this row is empty (pure computation).
    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }

    /// Check if this row is the universal row (arbitrary effects).
    pub fn is_any(&self) -> bool {
        self.effects.len() == 1 && self.effects[0].name == "*"
    }

    /// Check if this row contains a specific effect.
    pub fn contains(&self, effect: &EffectSymbol) -> bool {
        self.effects.iter().any(|e| e.name == effect.name)
    }

    /// Get the number of effects in this row.
    pub fn len(&self) -> usize {
        self.effects.len()
    }

    /// Iterate over the effects in this row.
    pub fn iter(&self) -> impl Iterator<Item = &EffectSymbol> {
        self.effects.iter()
    }

    /// Combine two effect rows (union of effects).
    /// `{IO} + {State}` = `{IO, State}`.
    pub fn union(&self, other: &EffectRow) -> Self {
        let mut effects: Vec<EffectSymbol> = self
            .effects
            .iter()
            .chain(other.effects.iter())
            .cloned()
            .collect();
        effects.sort_by_key(|e| e.name.clone());
        effects.dedup_by(|a, b| a.name == b.name);
        Self { effects }
    }

    /// Subtract effects from this row.
    /// `{IO, State} - {State}` = `{IO}`.
    pub fn subtract(&self, other: &EffectRow) -> Self {
        let effects: Vec<EffectSymbol> = self
            .effects
            .iter()
            .filter(|e| !other.contains(e))
            .cloned()
            .collect();
        Self { effects }
    }

    /// Check if `self` is a sub-row of `other` (effect subtyping).
    /// `{IO}` <: `{IO, State}` because every effect in self is in other.
    pub fn is_subtype_of(&self, other: &EffectRow) -> bool {
        if other.is_any() {
            return true;
        }
        self.effects.iter().all(|e| other.contains(e))
    }

    /// Add a single effect to this row.
    pub fn insert(&mut self, effect: EffectSymbol) {
        if !self.contains(&effect) {
            self.effects.push(effect);
            self.normalize();
        }
    }

    /// Normalize: sort and deduplicate. Always called after mutation.
    fn normalize(&mut self) {
        self.effects.sort_by_key(|e| e.name.clone());
        self.effects.dedup_by(|a, b| a.name == b.name);
    }
}

impl std::fmt::Display for EffectRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            write!(f, "{{}}")
        } else if self.is_any() {
            write!(f, "{{*}}")
        } else {
            let names = self
                .effects
                .iter()
                .map(|e| e.display_name())
                .collect::<Vec<_>>()
                .join(", ");
            write!(f, "{{{}}}", names)
        }
    }
}

impl std::ops::Add for EffectRow {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        self.union(&other)
    }
}

// ============================================================================
// Built-in Effect Definitions
// ============================================================================

/// Built-in effect constructors.
pub mod builtin {
    use super::*;

    pub fn io() -> EffectSymbol {
        EffectSymbol::new("IO")
    }

    pub fn async_() -> EffectSymbol {
        EffectSymbol::new("Async")
    }

    pub fn state() -> EffectSymbol {
        EffectSymbol::new("State")
    }

    pub fn error() -> EffectSymbol {
        EffectSymbol::new("Error")
    }

    pub fn non_det() -> EffectSymbol {
        EffectSymbol::new("NonDet")
    }

    pub fn debug() -> EffectSymbol {
        EffectSymbol::new("Debug")
    }

    pub fn alloc() -> EffectSymbol {
        EffectSymbol::new("Alloc")
    }

    pub fn yield_() -> EffectSymbol {
        EffectSymbol::new("Yield")
    }

    pub fn diverge() -> EffectSymbol {
        EffectSymbol::new("Div")
    }
}

// ============================================================================
// Effect Inference — constraint-based effect tracking
// ============================================================================

/// A constraint on the effect row of a computation.
#[derive(Debug, Clone)]
pub enum EffectConstraint {
    Equal(EffectRow, EffectRow, String),
    Subtype(EffectRow, EffectRow, String),
    CallEffect {
        func: String,
        caller_effect: EffectRow,
        callee_effect: EffectRow,
        origin: String,
    },
    Handled {
        effect: EffectSymbol,
        handler_effect: EffectRow,
        residual_effect: EffectRow,
        origin: String,
    },
}

/// The effect inference engine.
pub struct EffectInference {
    constraints: Vec<EffectConstraint>,
    effect_vars: std::collections::HashMap<usize, EffectRow>,
    next_var: usize,
    errors: Vec<EffectError>,
}

impl Default for EffectInference {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectInference {
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            effect_vars: std::collections::HashMap::new(),
            next_var: 0,
            errors: Vec::new(),
        }
    }

    pub fn fresh_var(&mut self) -> EffectRow {
        let id = self.next_var;
        self.next_var += 1;
        EffectRow::just(EffectSymbol::new(&format!("?e{}", id)))
    }

    pub fn add_constraint(&mut self, c: EffectConstraint) {
        self.constraints.push(c);
    }

    pub fn equal(&mut self, left: EffectRow, right: EffectRow, origin: &str) {
        self.add_constraint(EffectConstraint::Equal(left, right, origin.to_string()));
    }

    pub fn subtype(&mut self, sub: EffectRow, sup: EffectRow, origin: &str) {
        self.add_constraint(EffectConstraint::Subtype(sub, sup, origin.to_string()));
    }

    pub fn call_effect(
        &mut self,
        func: &str,
        caller_effect: EffectRow,
        callee_effect: EffectRow,
        origin: &str,
    ) -> EffectRow {
        self.add_constraint(EffectConstraint::CallEffect {
            func: func.to_string(),
            caller_effect: caller_effect.clone(),
            callee_effect: callee_effect.clone(),
            origin: origin.to_string(),
        });
        caller_effect + callee_effect
    }

    pub fn handle(
        &mut self,
        effect: EffectSymbol,
        handler_effect: EffectRow,
        origin: &str,
    ) -> EffectRow {
        let residual = handler_effect.clone();
        self.add_constraint(EffectConstraint::Handled {
            effect,
            handler_effect,
            residual_effect: residual.clone(),
            origin: origin.to_string(),
        });
        residual
    }

    pub fn solve(&mut self) -> Result<EffectRow, Vec<EffectError>> {
        let constraints = std::mem::take(&mut self.constraints);
        for c in &constraints {
            if let Err(e) = self.solve_constraint(c) {
                self.errors.push(e);
            }
        }
        if !self.errors.is_empty() {
            return Err(std::mem::take(&mut self.errors));
        }
        Ok(EffectRow::pure())
    }

    fn solve_constraint(&mut self, c: &EffectConstraint) -> Result<(), EffectError> {
        match c {
            EffectConstraint::Equal(left, right, origin) => {
                if !left.is_subtype_of(right) && !right.is_subtype_of(left) {
                    return Err(EffectError::Mismatch {
                        left: left.clone(),
                        right: right.clone(),
                        origin: origin.clone(),
                    });
                }
                Ok(())
            }
            EffectConstraint::Subtype(sub, sup, origin) => {
                if !sub.is_subtype_of(sup) {
                    return Err(EffectError::Subtype {
                        sub: sub.clone(),
                        sup: sup.clone(),
                        origin: origin.clone(),
                    });
                }
                Ok(())
            }
            EffectConstraint::CallEffect { .. } => Ok(()),
            EffectConstraint::Handled { .. } => Ok(()),
        }
    }

    pub fn infer_function_effect(
        &mut self,
        declared_effects: &EffectRow,
        body_effects: EffectRow,
    ) -> Result<EffectRow, Vec<EffectError>> {
        if !body_effects.is_subtype_of(declared_effects) {
            return Err(vec![EffectError::Subtype {
                sub: body_effects,
                sup: declared_effects.clone(),
                origin: "function body effect must be subtype of declared effect".to_string(),
            }]);
        }
        Ok(body_effects)
    }
}

// ============================================================================
// Effect Errors
// ============================================================================

#[derive(Debug, Clone)]
pub enum EffectError {
    Mismatch {
        left: EffectRow,
        right: EffectRow,
        origin: String,
    },
    Subtype {
        sub: EffectRow,
        sup: EffectRow,
        origin: String,
    },
    UnhandledEffect {
        effect: EffectSymbol,
        origin: String,
    },
    Custom {
        message: String,
        origin: String,
    },
}

impl std::fmt::Display for EffectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EffectError::Mismatch {
                left,
                right,
                origin,
            } => {
                write!(
                    f,
                    "effect row mismatch: {} vs {} at {}",
                    left, right, origin
                )
            }
            EffectError::Subtype { sub, sup, origin } => {
                write!(
                    f,
                    "effect row {} is not a subtype of {} at {}",
                    sub, sup, origin
                )
            }
            EffectError::UnhandledEffect { effect, origin } => {
                write!(
                    f,
                    "unhandled effect '{}' at {}",
                    effect.display_name(),
                    origin
                )
            }
            EffectError::Custom { message, origin } => {
                write!(f, "{} at {}", message, origin)
            }
        }
    }
}

// ============================================================================
// Effect Handler Registration
// ============================================================================

#[derive(Debug, Clone)]
pub struct EffectHandler {
    pub name: String,
    pub handles: EffectSymbol,
    pub return_ty: String,
    pub residual: EffectRow,
}

impl EffectHandler {
    pub fn new(name: &str, handles: EffectSymbol, return_ty: &str) -> Self {
        Self {
            name: name.to_string(),
            handles,
            return_ty: return_ty.to_string(),
            residual: EffectRow::pure(),
        }
    }

    pub fn with_residual(mut self, residual: EffectRow) -> Self {
        self.residual = residual;
        self
    }
}

// ============================================================================
// Effect Registry — tracks all registered effects and handlers
// ============================================================================

pub struct EffectRegistry {
    effects: std::collections::HashMap<String, EffectSymbol>,
    handlers: std::collections::HashMap<String, EffectHandler>,
    defaults: std::collections::HashMap<String, String>,
}

impl Default for EffectRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectRegistry {
    pub fn new() -> Self {
        Self {
            effects: std::collections::HashMap::new(),
            handlers: std::collections::HashMap::new(),
            defaults: std::collections::HashMap::new(),
        }
    }

    pub fn register_effect(&mut self, effect: EffectSymbol) {
        self.effects.insert(effect.name.clone(), effect.clone());
    }

    pub fn register_handler(&mut self, handler: EffectHandler) {
        self.handlers.insert(handler.name.clone(), handler.clone());
        self.defaults
            .insert(handler.handles.name.clone(), handler.name.clone());
    }

    pub fn get_effect(&self, name: &str) -> Option<&EffectSymbol> {
        self.effects.get(name)
    }

    pub fn get_handler(&self, name: &str) -> Option<&EffectHandler> {
        self.handlers.get(name)
    }

    pub fn default_handler(&self, effect: &str) -> Option<&str> {
        self.defaults.get(effect).map(|s| s.as_str())
    }

    pub fn is_registered(&self, name: &str) -> bool {
        self.effects.contains_key(name)
    }

    pub fn effect_names(&self) -> Vec<&String> {
        let mut names: Vec<_> = self.effects.keys().collect();
        names.sort();
        names
    }

    pub fn with_builtins() -> Self {
        let mut reg = Self::new();
        reg.register_effect(builtin::io());
        reg.register_effect(builtin::async_());
        reg.register_effect(builtin::state());
        reg.register_effect(builtin::error());
        reg.register_effect(builtin::non_det());
        reg.register_effect(builtin::debug());
        reg.register_effect(builtin::alloc());
        reg.register_effect(builtin::yield_());
        reg.register_effect(builtin::diverge());
        reg
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_row_pure() {
        let pure = EffectRow::pure();
        assert!(pure.is_empty());
        assert_eq!(format!("{}", pure), "{}");
    }

    #[test]
    fn test_effect_row_union() {
        let io = EffectRow::just(builtin::io());
        let state = EffectRow::just(builtin::state());
        let combined = io + state;
        assert_eq!(combined.len(), 2);
        assert!(combined.contains(&builtin::io()));
        assert!(combined.contains(&builtin::state()));
    }

    #[test]
    fn test_effect_row_subtype() {
        let io = EffectRow::just(builtin::io());
        let state = EffectRow::just(builtin::state());
        let combined = io.clone() + state;
        assert!(io.is_subtype_of(&combined));
        assert!(!combined.is_subtype_of(&io));
    }

    #[test]
    fn test_effect_row_any() {
        let any = EffectRow::any();
        assert!(any.is_any());
        let io = EffectRow::just(builtin::io());
        assert!(io.is_subtype_of(&any));
    }

    #[test]
    fn test_effect_row_subtract() {
        let combined = EffectRow::just(builtin::io()) + EffectRow::just(builtin::state());
        let io_only = combined.subtract(&EffectRow::just(builtin::state()));
        assert_eq!(io_only.len(), 1);
    }

    #[test]
    fn test_effect_symbol_display() {
        let err = builtin::error().with_param("String");
        assert_eq!(err.display_name(), "Error[String]");
        let io = builtin::io();
        assert_eq!(io.display_name(), "IO");
    }

    #[test]
    fn test_effect_inference_fresh_var() {
        let mut engine = EffectInference::new();
        let v1 = engine.fresh_var();
        let v2 = engine.fresh_var();
        assert_eq!(v1.len(), 1);
        assert_eq!(v2.len(), 1);
        assert_ne!(format!("{}", v1), format!("{}", v2));
    }

    #[test]
    fn test_effect_registry() {
        let reg = EffectRegistry::with_builtins();
        assert!(reg.is_registered("IO"));
        assert!(reg.is_registered("Async"));
        assert!(reg.is_registered("State"));
        assert!(reg.is_registered("Error"));
        assert!(!reg.is_registered("NonExistent"));
    }

    #[test]
    fn test_effect_row_dedup() {
        let io1 = EffectRow::just(builtin::io());
        let io2 = EffectRow::just(builtin::io());
        let combined = io1 + io2;
        assert_eq!(combined.len(), 1);
    }

    #[test]
    fn test_effect_inference_subtype_ok() {
        let mut engine = EffectInference::new();
        let result =
            engine.infer_function_effect(&EffectRow::just(builtin::io()), EffectRow::pure());
        assert!(result.is_ok());
    }

    #[test]
    fn test_handler_registration() {
        let mut reg = EffectRegistry::with_builtins();
        let handler = EffectHandler::new("try_catch", builtin::error(), "Int");
        reg.register_handler(handler);
        assert!(reg.get_handler("try_catch").is_some());
        assert_eq!(reg.default_handler("Error"), Some("try_catch"));
    }

    #[test]
    fn test_effect_row_display() {
        assert_eq!(format!("{}", EffectRow::pure()), "{}");
        assert_eq!(format!("{}", EffectRow::any()), "{*}");
        assert_eq!(
            format!(
                "{}",
                EffectRow::just(builtin::io()) + EffectRow::just(builtin::state())
            ),
            "{IO, State}"
        );
    }
}
