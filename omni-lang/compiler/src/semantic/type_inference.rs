//! Omni Type Inference Engine
//!
#![allow(dead_code)]

//! Implements a constraint-based Hindley-Milner style type inference algorithm
//! for the Omni programming language. This engine:
//!
//! 1. Traverses the AST and assigns fresh type variables to unknown types
//! 2. Generates type constraints from the program structure
//! 3. Solves constraints via unification
//! 4. Reports type errors with descriptive messages
//!
//! The inference handles arithmetic, comparisons, if/else, function calls,
//! variable bindings, struct field access, array literals, and method calls.
//!
//! ## Error Handling Model (O-024)
//!
//! Omni v1 uses a two-tier error model:
//!
//! - **Hard errors** (fatal): Explicit type-annotation mismatches where the user
//!   wrote `let x: Int = "hello"` and the initializer's type clearly disagrees.
//!   These cause compilation to abort.
//!
//! - **Soft errors** (warnings): Everything else — unresolved type variables,
//!   undefined builtins, numeric type expectations for `+` with strings, etc.
//!   These are emitted as warnings because the inference engine may have
//!   false positives from missing built-in signatures or dynamic features.
//!
//! At runtime, the OVM uses `Result<(), String>` for all error paths.
//! Hard errors (panic opcode, assertion failure, division by zero, stack overflow)
//! terminate the VM with a descriptive message. There is no try/catch in v1;
//! programs use the Result type for recoverable errors.

use std::collections::HashMap;
use std::fmt;

use crate::monitor;
use crate::parser::ast;

// ---------------------------------------------------------------------------
// Core type representations
// ---------------------------------------------------------------------------

/// Unique identifier for an unresolved type variable.
pub type TypeVarId = usize;

/// Internal type representation used during inference.
/// This is separate from `ast::Type` so we can represent unresolved type
/// variables and perform unification without modifying the AST types.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Unresolved type variable (created by `fresh_var`)
    Var(TypeVarId),

    // -- Primitives --
    Int,
    Float,
    Bool,
    String,
    Void,

    /// The *never* type – for diverging expressions (e.g. `return`, `break`)
    Never,

    /// Nullable wrapper (`T?`)
    Nullable(Box<Type>),

    /// Homogeneous array `[T]`
    Array(Box<Type>),

    /// Map / dictionary `Map<K, V>`
    Map(Box<Type>, Box<Type>),

    /// Tuple `(T1, T2, …)`
    Tuple(Vec<Type>),

    /// Function type `(P1, P2, …) -> R`
    Function(Vec<Type>, Box<Type>),

    /// User-defined struct with ordered fields
    Struct(std::string::String, Vec<(std::string::String, Type)>),

    /// Trait object (dynamic dispatch)
    TraitObject(std::string::String),

    /// Generic type parameter (uninstantiated)
    Generic(std::string::String),

    /// Applied generic, e.g. `Vec<Int>`
    Applied(Box<Type>, Vec<Type>),

    /// Placeholder for error recovery – unifies with anything
    Error,

    /// Error set type (v2.0 spec: finite named sets of errors)
    ErrorSet(String),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Var(id) => write!(f, "?T{}", id),
            Type::Int => write!(f, "Int"),
            Type::Float => write!(f, "Float"),
            Type::Bool => write!(f, "Bool"),
            Type::String => write!(f, "String"),
            Type::Void => write!(f, "Void"),
            Type::Never => write!(f, "Never"),
            Type::Nullable(inner) => write!(f, "{}?", inner),
            Type::Array(elem) => write!(f, "[{}]", elem),
            Type::Map(k, v) => write!(f, "Map<{}, {}>", k, v),
            Type::Tuple(elems) => {
                write!(f, "(")?;
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", e)?;
                }
                write!(f, ")")
            }
            Type::Function(params, ret) => {
                write!(f, "fn(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ") -> {}", ret)
            }
            Type::Struct(name, _) => write!(f, "{}", name),
            Type::TraitObject(name) => write!(f, "dyn {}", name),
            Type::Generic(name) => write!(f, "{}", name),
            Type::Applied(base, args) => {
                write!(f, "{}<", base)?;
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", a)?;
                }
                write!(f, ">")
            }
            Type::Error => write!(f, "<error>"),
            Type::ErrorSet(name) => write!(f, "error set {}", name),
        }
    }
}

// ---------------------------------------------------------------------------
// Type errors
// ---------------------------------------------------------------------------

/// A descriptive type error produced during inference.
#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: std::string::String,
    pub hint: Option<std::string::String>,
}

impl TypeError {
    pub fn new(message: impl Into<std::string::String>) -> Self {
        Self {
            message: message.into(),
            hint: None,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<std::string::String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(ref hint) = self.hint {
            write!(f, " (hint: {})", hint)?;
        }
        Ok(())
    }
}

/// Determine whether a type error is a "hard" (fatal) error.
///
/// Hard errors are **explicit type-annotation mismatches** where the user
/// wrote `let x: Int = "hello"` and the initializer's type clearly
/// disagrees with the annotation.  Everything else is demoted to a
/// warning because the type-inference engine was never calibrated for
/// Omni's dynamic-flavour features (string concat with `+`, implicit
/// conversions, built-in functions not in the environment, etc.).
///
/// Canonical location (O-097): previously duplicated in `main.rs` and
/// `interpreter.rs`. Both now delegate to this function.
pub fn is_hard_type_error(err: &TypeError) -> bool {
    let msg = &err.message;

    // Unresolved type variables — inference couldn't determine the type
    if msg.contains("?T") {
        return false;
    }

    // Undefined variable / function — likely a built-in not registered
    if msg.contains("Undefined variable") || msg.contains("Undefined function") {
        return false;
    }

    // "<error>" is the error-recovery placeholder type
    if msg.contains("<error>") {
        return false;
    }

    // "Expected numeric type" — Omni supports string concat with +,
    // list concat, etc.  The inference engine doesn't model these.
    if msg.contains("Expected numeric type") {
        return false;
    }

    // Only flag explicit annotation mismatches as hard errors:
    // "Type mismatch: X vs Y – let/var binding '…': declared type must match initializer"
    if msg.contains("Type mismatch") && msg.contains("declared type must match initializer") {
        return true;
    }

    // All other type-mismatch or constraint errors are soft
    // (function call argument mismatches, return type mismatches, etc.
    //  may be false positives due to missing built-in signatures).
    false
}

// ---------------------------------------------------------------------------
// Constraints
// ---------------------------------------------------------------------------

/// A type constraint generated while walking the AST.
#[derive(Debug, Clone)]
pub enum Constraint {
    /// Two types must be equal.
    Equal(Type, Type, ConstraintOrigin),

    /// `receiver_ty` must have a field called `field` of type `field_ty`.
    HasField(Type, std::string::String, Type, ConstraintOrigin),

    /// `callee_ty` must be callable with `arg_tys`, producing `ret_ty`.
    Callable(Type, Vec<Type>, Type, ConstraintOrigin),
}

/// Where a constraint originated – used for error messages.
#[derive(Debug, Clone)]
pub struct ConstraintOrigin {
    pub description: std::string::String,
}

impl ConstraintOrigin {
    pub fn new(desc: impl Into<std::string::String>) -> Self {
        Self {
            description: desc.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Function signatures
// ---------------------------------------------------------------------------

type FunctionSignature = (Vec<Type>, Type, Option<crate::semantic::effects::EffectRow>);

// ---------------------------------------------------------------------------
// Substitution
// ---------------------------------------------------------------------------

/// Maps type-variable IDs to their resolved types.
#[derive(Debug, Clone, Default)]
pub struct Substitution {
    map: HashMap<TypeVarId, Type>,
}

impl Substitution {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Bind `var` to `ty` (does **not** check for cycles).
    pub fn bind(&mut self, var: TypeVarId, ty: Type) {
        self.map.insert(var, ty);
    }

    /// Look up a single variable (one step).
    pub fn lookup(&self, var: TypeVarId) -> Option<&Type> {
        self.map.get(&var)
    }

    /// Fully resolve a type through the substitution, chasing variable links.
    pub fn apply(&self, ty: &Type) -> Type {
        match ty {
            Type::Var(id) => {
                if let Some(resolved) = self.map.get(id) {
                    // Chase the chain
                    self.apply(resolved)
                } else {
                    ty.clone()
                }
            }
            Type::Nullable(inner) => Type::Nullable(Box::new(self.apply(inner))),
            Type::Array(elem) => Type::Array(Box::new(self.apply(elem))),
            Type::Map(k, v) => Type::Map(Box::new(self.apply(k)), Box::new(self.apply(v))),
            Type::Tuple(elems) => Type::Tuple(elems.iter().map(|e| self.apply(e)).collect()),
            Type::Function(params, ret) => Type::Function(
                params.iter().map(|p| self.apply(p)).collect(),
                Box::new(self.apply(ret)),
            ),
            Type::Struct(name, fields) => Type::Struct(
                name.clone(),
                fields
                    .iter()
                    .map(|(n, t)| (n.clone(), self.apply(t)))
                    .collect(),
            ),
            Type::Applied(base, args) => Type::Applied(
                Box::new(self.apply(base)),
                args.iter().map(|a| self.apply(a)).collect(),
            ),
            // Leaf types are returned as-is
            _ => ty.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Type environment (scoped symbol table)
// ---------------------------------------------------------------------------

/// A scoped mapping from variable names to their inferred types.
#[derive(Debug, Clone)]
pub struct TypeEnv {
    /// Stack of scopes. The last entry is the innermost scope.
    scopes: Vec<HashMap<std::string::String, Type>>,
    /// Current accumulated effect row (for effect inference)
    pub effect_row: Option<crate::semantic::effects::EffectRow>,
}

impl Default for TypeEnv {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeEnv {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            effect_row: None,
        }
    }

    /// Open a new inner scope.
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Close the innermost scope.
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Clear the effect row (for pure expressions)
    pub fn clear_effects(&mut self) {
        self.effect_row = None;
    }

    /// Set the current effect row
    pub fn set_effect_row(&mut self, effects: crate::semantic::effects::EffectRow) {
        self.effect_row = Some(effects);
    }

    /// Get the current effect row
    pub fn get_effect_row(&self) -> Option<&crate::semantic::effects::EffectRow> {
        self.effect_row.as_ref()
    }

    /// Add an effect to the current effect row
    pub fn add_effect(&mut self, effect: crate::semantic::effects::EffectSymbol) {
        if let Some(ref mut row) = self.effect_row {
            row.insert(effect);
        } else {
            self.effect_row = Some(crate::semantic::effects::EffectRow::just(effect));
        }
    }

    /// Merge another effect row into the current one
    pub fn merge_effects(&mut self, other: &crate::semantic::effects::EffectRow) {
        if let Some(ref mut row) = self.effect_row {
            for effect in other.iter() {
                row.insert(effect.clone());
            }
        } else {
            self.effect_row = Some(other.clone());
        }
    }

    /// Define a variable in the current (innermost) scope.
    pub fn define(&mut self, name: impl Into<std::string::String>, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.into(), ty);
        }
    }

    /// Look up a variable, searching from innermost scope outward.
    pub fn lookup(&self, name: &str) -> Option<&Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }

    /// Look up a variable and return a fresh copy with type variables replaced.
    /// This is used for function calls to ensure each call site gets fresh type variables.
    pub fn lookup_fresh(&self, name: &str, fresh_var_fn: &mut dyn FnMut() -> Type) -> Option<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(self.freshen_type(ty, fresh_var_fn));
            }
        }
        None
    }

    /// Recursively replace type variables with fresh ones.
    fn freshen_type(&self, ty: &Type, fresh_var_fn: &mut dyn FnMut() -> Type) -> Type {
        match ty {
            Type::Var(id) if *id >= 99900 => fresh_var_fn(), // Replace "Any" sentinel variables
            Type::Var(id) => Type::Var(*id),
            Type::Nullable(inner) => {
                Type::Nullable(Box::new(self.freshen_type(inner, fresh_var_fn)))
            }
            Type::Array(elem) => Type::Array(Box::new(self.freshen_type(elem, fresh_var_fn))),
            Type::Map(k, v) => Type::Map(
                Box::new(self.freshen_type(k, fresh_var_fn)),
                Box::new(self.freshen_type(v, fresh_var_fn)),
            ),
            Type::Tuple(elems) => Type::Tuple(
                elems
                    .iter()
                    .map(|e| self.freshen_type(e, fresh_var_fn))
                    .collect(),
            ),
            Type::Function(params, ret) => Type::Function(
                params
                    .iter()
                    .map(|p| self.freshen_type(p, fresh_var_fn))
                    .collect(),
                Box::new(self.freshen_type(ret, fresh_var_fn)),
            ),
            Type::Struct(name, fields) => Type::Struct(
                name.clone(),
                fields
                    .iter()
                    .map(|(n, t)| (n.clone(), self.freshen_type(t, fresh_var_fn)))
                    .collect(),
            ),
            Type::Applied(base, args) => Type::Applied(
                Box::new(self.freshen_type(base, fresh_var_fn)),
                args.iter()
                    .map(|a| self.freshen_type(a, fresh_var_fn))
                    .collect(),
            ),
            _ => ty.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Inference result
// ---------------------------------------------------------------------------

/// The output of a successful inference pass.
#[derive(Debug, Clone)]
pub struct InferenceResult {
    /// Maps expression descriptions / IDs to their resolved types.
    pub expr_types: HashMap<std::string::String, Type>,

    /// Maps variable names (fully qualified) to their resolved types.
    pub variable_types: HashMap<std::string::String, Type>,

    /// Maps function names to their resolved function types.
    pub function_types: HashMap<std::string::String, Type>,

    /// Maps struct names to their field type lists.
    pub struct_types: HashMap<std::string::String, Vec<(std::string::String, Type)>>,

    /// Any non-fatal warnings produced during inference.
    pub warnings: Vec<std::string::String>,
}

impl InferenceResult {
    fn new() -> Self {
        Self {
            expr_types: HashMap::new(),
            variable_types: HashMap::new(),
            function_types: HashMap::new(),
            struct_types: HashMap::new(),
            warnings: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// The inference engine
// ---------------------------------------------------------------------------

/// Constraint-based Hindley-Milner type inference engine for Omni.
pub struct InferenceEngine {
    /// Counter for fresh type variable IDs.
    next_var: TypeVarId,

    /// Accumulated constraints.
    constraints: Vec<Constraint>,

    /// Current substitution (built up during unification).
    substitution: Substitution,

    /// Known struct definitions: name → fields.
    struct_defs: HashMap<std::string::String, Vec<(std::string::String, Type)>>,

    /// Known function signatures: name → (param_types, return_type, effect_row).
    function_sigs: HashMap<std::string::String, FunctionSignature>,

    /// Collected type errors.
    errors: Vec<TypeError>,

    /// Resolved result (populated after solving).
    result: InferenceResult,

    /// Current accumulated effect row during inference
    current_effects: Vec<crate::semantic::effects::EffectSymbol>,
}

impl Default for InferenceEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl InferenceEngine {
    // -----------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------

    pub fn new() -> Self {
        Self {
            next_var: 0,
            constraints: Vec::new(),
            substitution: Substitution::new(),
            struct_defs: HashMap::new(),
            function_sigs: HashMap::new(),
            errors: Vec::new(),
            result: InferenceResult::new(),
            current_effects: Vec::new(),
        }
    }
    // -----------------------------------------------------------------
    // Effect inference helpers
    // -----------------------------------------------------------------

    /// Add an IO effect (for println, print, read, etc.)
    pub fn add_io_effect(&mut self) {
        self.current_effects
            .push(crate::semantic::effects::builtin::io());
    }

    /// Add an async effect (for async functions)
    pub fn add_async_effect(&mut self) {
        self.current_effects
            .push(crate::semantic::effects::builtin::async_());
    }

    /// Add a state effect (for mutable state access)
    pub fn add_state_effect(&mut self) {
        self.current_effects
            .push(crate::semantic::effects::builtin::state());
    }

    /// Add an error effect
    pub fn add_error_effect(&mut self) {
        self.current_effects
            .push(crate::semantic::effects::builtin::error());
    }

    /// Merge another effect row into the current accumulated effects.
    pub fn merge_effects(&mut self, other: &crate::semantic::effects::EffectRow) {
        self.current_effects.extend(other.iter().cloned());
    }

    /// Get the current accumulated effect row
    pub fn get_effect_row(&self) -> crate::semantic::effects::EffectRow {
        crate::semantic::effects::EffectRow::from_effects(self.current_effects.clone())
    }

    /// Clear accumulated effects (for pure expressions)
    pub fn clear_effects(&mut self) {
        self.current_effects.clear();
    }

    /// Check if the current effects satisfy a declared effect row
    pub fn check_effects(
        &self,
        declared: &crate::semantic::effects::EffectRow,
    ) -> Result<(), String> {
        let current = self.get_effect_row();
        if !current.is_subtype_of(declared) {
            Err(format!(
                "Function body effects {:?} do not satisfy declared effects {:?}",
                current, declared
            ))
        } else {
            Ok(())
        }
    }

    /// Register built-in functions and types available in all modules.
    fn register_builtins(&mut self, env: &mut TypeEnv) {
        // Core types
        env.define("String", Type::String);
        env.define("Int", Type::Int);
        env.define("Float", Type::Float);
        env.define("Bool", Type::Bool);
        env.define("Void", Type::Void);

        // Type aliases
        env.define("int", Type::Int);
        env.define("float", Type::Float);
        env.define("bool", Type::Bool);
        env.define("string", Type::String);

        // IO functions - use Any type with a special sentinel ID
        // This allows println to accept any type while the constraint solver
        // creates fresh variables for each call site
        let any_type = Type::Var(99999);
        env.define(
            "println",
            Type::Function(vec![any_type.clone()], Box::new(Type::Void)),
        );
        env.define(
            "print",
            Type::Function(vec![any_type.clone()], Box::new(Type::Void)),
        );
        env.define(
            "println!",
            Type::Function(vec![any_type.clone()], Box::new(Type::Void)),
        );
        env.define(
            "eprintln",
            Type::Function(vec![any_type.clone()], Box::new(Type::Void)),
        );
        env.define(
            "eprint",
            Type::Function(vec![any_type.clone()], Box::new(Type::Void)),
        );

        // String formatting
        env.define(
            "format",
            Type::Function(vec![Type::String], Box::new(Type::String)),
        );
        env.define(
            "format!",
            Type::Function(vec![Type::String], Box::new(Type::String)),
        );

        // Type introspection
        env.define(
            "type_of",
            Type::Function(vec![Type::Var(999)], Box::new(Type::String)),
        );

        // Conversion functions
        env.define(
            "int",
            Type::Function(vec![Type::String], Box::new(Type::Int)),
        );
        env.define(
            "float",
            Type::Function(vec![Type::String], Box::new(Type::Float)),
        );
        env.define(
            "str",
            Type::Function(vec![Type::Int], Box::new(Type::String)),
        );

        // String methods (available as associated functions)
        env.define(
            "len",
            Type::Function(vec![Type::String], Box::new(Type::Int)),
        );

        // Assertion
        env.define(
            "assert",
            Type::Function(vec![Type::Bool, Type::String], Box::new(Type::Void)),
        );

        // Range function
        env.define(
            "range",
            Type::Function(
                vec![Type::Int, Type::Int],
                Box::new(Type::Array(Box::new(Type::Int))),
            ),
        );

        // Register builtin struct-like types
        self.struct_defs.insert("String".to_string(), vec![]);
        self.struct_defs.insert(
            "Vec2".to_string(),
            vec![
                ("x".to_string(), Type::Float),
                ("y".to_string(), Type::Float),
            ],
        );
        self.struct_defs.insert(
            "Stack".to_string(),
            vec![
                ("items".to_string(), Type::Array(Box::new(Type::Var(1000)))),
                ("size".to_string(), Type::Int),
            ],
        );

        // Register builtin function signatures
        let io_effect = Some(crate::semantic::effects::EffectRow::just(
            crate::semantic::effects::builtin::io(),
        ));
        let pure_effect = Some(crate::semantic::effects::EffectRow::pure());
        let println_sig = (vec![any_type.clone()], Type::Void, io_effect.clone());
        self.function_sigs
            .insert("println".to_string(), println_sig.clone());
        self.function_sigs
            .insert("print".to_string(), println_sig.clone());
        self.function_sigs
            .insert("println!".to_string(), println_sig.clone());
        self.function_sigs
            .insert("eprintln".to_string(), println_sig.clone());
        self.function_sigs.insert("eprint".to_string(), println_sig);
        let format_sig = (vec![Type::String], Type::String, pure_effect.clone());
        self.function_sigs
            .insert("format".to_string(), format_sig.clone());
        self.function_sigs.insert("format!".to_string(), format_sig);
    }

    /// Allocate a fresh (unique) type variable.
    pub fn fresh_var(&mut self) -> Type {
        let id = self.next_var;
        self.next_var += 1;
        Type::Var(id)
    }

    // -----------------------------------------------------------------
    // AST ↔ inference-type conversion helpers
    // -----------------------------------------------------------------

    /// Convert an `ast::Type` annotation into the inference engine's `Type`.
    fn from_ast_type(&mut self, ast_ty: &ast::Type) -> Type {
        match ast_ty {
            ast::Type::I8 | ast::Type::I16 | ast::Type::I32 | ast::Type::I64 | ast::Type::Isize => {
                Type::Int
            }
            ast::Type::U8 | ast::Type::U16 | ast::Type::U32 | ast::Type::U64 | ast::Type::Usize => {
                Type::Int
            }
            ast::Type::F32 | ast::Type::F64 => Type::Float,
            ast::Type::Bool => Type::Bool,
            ast::Type::Str => Type::String,
            ast::Type::Named(name) => match name.as_str() {
                "int" | "Int" | "i32" | "i64" => Type::Int,
                "float" | "Float" | "f32" | "f64" => Type::Float,
                "bool" | "Bool" => Type::Bool,
                "string" | "String" | "str" => Type::String,
                "void" | "Void" => Type::Void,
                _ => Type::Generic(name.clone()),
            },
            ast::Type::Array(elem_ty, _size) => Type::Array(Box::new(self.from_ast_type(elem_ty))),
            ast::Type::Slice(elem_ty) => Type::Array(Box::new(self.from_ast_type(elem_ty))),
            ast::Type::Function(params, ret) => {
                let param_tys: Vec<Type> = params.iter().map(|p| self.from_ast_type(p)).collect();
                let ret_ty = match ret {
                    Some(r) => self.from_ast_type(r),
                    None => Type::Void,
                };
                Type::Function(param_tys, Box::new(ret_ty))
            }
            ast::Type::Generic(name, args) => {
                let base = Type::Generic(name.clone());
                let type_args: Vec<Type> = args.iter().map(|a| self.from_ast_type(a)).collect();
                Type::Applied(Box::new(base), type_args)
            }
            ast::Type::WithOwnership(inner, _ownership) => self.from_ast_type(inner),
            ast::Type::SelfOwned | ast::Type::SelfRef { .. } => Type::Generic("Self".into()),
            ast::Type::TraitObject { principal, .. } => Type::TraitObject(principal.clone()),
            ast::Type::AssocType(trait_name, type_name) => {
                Type::Generic(format!("{}::{}", trait_name, type_name))
            }
            ast::Type::ConstGeneric(name) => Type::Generic(name.clone()),
            ast::Type::WhereConstrained { base, .. } => self.from_ast_type(base),
            ast::Type::HigherRanked { bound } => Type::Generic(bound.clone()),
            ast::Type::Tuple(elems) => {
                let elem_tys: Vec<Type> = elems.iter().map(|e| self.from_ast_type(e)).collect();
                Type::Tuple(elem_tys)
            }
            ast::Type::Nullable(inner) => Type::Nullable(Box::new(self.from_ast_type(inner))),
            ast::Type::Infer => self.fresh_var(), // Create fresh type variable for inference
            ast::Type::ErrorSet { name, .. } => Type::ErrorSet(name.clone()), // Error set types
        }
    }

    // -----------------------------------------------------------------
    // Module-level inference
    // -----------------------------------------------------------------

    /// Run type inference on an entire module. Returns either the resolved
    /// `InferenceResult` or a list of type errors.
    pub fn infer_module(
        &mut self,
        module: &ast::Module,
    ) -> Result<InferenceResult, Vec<TypeError>> {
        let mut env = TypeEnv::new();

        // -- Register built-in functions and types --
        self.register_builtins(&mut env);

        // -- First pass: collect struct definitions and function signatures --
        for item in module.items.iter() {
            // Keep the monitor alive during module registration.
            monitor::update_heartbeat();
            match item {
                ast::Item::Struct(sdef) => {
                    self.register_struct(sdef);
                }
                ast::Item::Function(func) => {
                    self.register_function(func, &mut env);
                }
                ast::Item::Impl(impl_block) => {
                    for method in &impl_block.methods {
                        let qualified = format!("{}::{}", impl_block.type_name, method.name);
                        self.register_function_with_name(&qualified, method, &mut env);
                    }
                }
                ast::Item::Const(cdecl) => {
                    let ty = self.from_ast_type(&cdecl.ty);
                    env.define(&cdecl.name, ty.clone());
                    self.result.variable_types.insert(cdecl.name.clone(), ty);
                }
                ast::Item::Static(sdecl) => {
                    let ty = self.from_ast_type(&sdecl.ty);
                    env.define(&sdecl.name, ty.clone());
                    self.result.variable_types.insert(sdecl.name.clone(), ty);
                }
                _ => { /* imports, traits, type aliases, etc. – skip for now */ }
            }
        }

        // -- Second pass: infer function bodies --
        for item in module.items.iter() {
            // Keep the monitor alive during inference.
            monitor::update_heartbeat();
            if let ast::Item::Function(func) = item {
                self.infer_function(func, &mut env);
            }
            if let ast::Item::Impl(impl_block) = item {
                for method in &impl_block.methods {
                    self.infer_function(method, &mut env);
                }
            }
        }

        // -- Solve collected constraints --
        self.solve_constraints();

        // -- Apply final substitution to all recorded types --
        self.finalize_types();

        if self.errors.is_empty() {
            Ok(self.result.clone())
        } else {
            Err(self.errors.clone())
        }
    }

    // -----------------------------------------------------------------
    // Registration helpers (first pass)
    // -----------------------------------------------------------------

    fn register_struct(&mut self, sdef: &ast::StructDef) {
        let fields: Vec<(std::string::String, Type)> = sdef
            .fields
            .iter()
            .map(|f| (f.name.clone(), self.from_ast_type(&f.ty)))
            .collect();
        self.struct_defs.insert(sdef.name.clone(), fields.clone());
        self.result.struct_types.insert(sdef.name.clone(), fields);
    }

    fn register_function(&mut self, func: &ast::Function, env: &mut TypeEnv) {
        self.register_function_with_name(&func.name, func, env);
    }

    fn register_function_with_name(&mut self, name: &str, func: &ast::Function, env: &mut TypeEnv) {
        let param_tys: Vec<Type> = func
            .params
            .iter()
            .map(|p| self.from_ast_type(&p.ty))
            .collect();
        let ret_ty = match &func.return_type {
            Some(t) => self.from_ast_type(t),
            None => self.fresh_var(),
        };
        let effect_row = Self::effect_row_from_ast(func.effect_row.as_ref());

        let func_ty = Type::Function(param_tys.clone(), Box::new(ret_ty.clone()));
        env.define(name, func_ty.clone());

        self.function_sigs
            .insert(name.to_string(), (param_tys, ret_ty, effect_row));
        self.result.function_types.insert(name.to_string(), func_ty);
    }

    fn effect_row_from_ast(
        effect_row: Option<&ast::EffectRow>,
    ) -> Option<crate::semantic::effects::EffectRow> {
        effect_row.map(|row| {
            let effects: Vec<crate::semantic::effects::EffectSymbol> = row
                .effects
                .iter()
                .map(|e| crate::semantic::effects::EffectSymbol::new(&e.name))
                .collect();
            crate::semantic::effects::EffectRow::from_effects(effects)
        })
    }

    fn merge_registered_effects(&mut self, name: &str) -> bool {
        let effect_row = self
            .function_sigs
            .get(name)
            .and_then(|(_, _, effect_row)| effect_row.clone());
        if let Some(effect_row) = effect_row {
            self.merge_effects(&effect_row);
            true
        } else {
            false
        }
    }

    // -----------------------------------------------------------------
    // Function-level inference
    // -----------------------------------------------------------------

    /// Infer types within a single function body.
    pub fn infer_function(&mut self, func: &ast::Function, env: &mut TypeEnv) {
        env.push_scope();

        // Clear any previous effects for this function
        self.clear_effects();

        // Bind parameters
        for param in &func.params {
            let ty = self.from_ast_type(&param.ty);
            env.define(&param.name, ty);
        }

        let expected_ret = match &func.return_type {
            Some(t) => self.from_ast_type(t),
            None => self.fresh_var(),
        };

        // Infer each statement in the body
        for stmt in &func.body.statements {
            self.infer_statement(stmt, env, &expected_ret);
        }

        let inferred_effects = self.get_effect_row();
        let declared = Self::effect_row_from_ast(func.effect_row.as_ref());

        // Only enforce effect compatibility when the function explicitly
        // declares an effect row. Otherwise we infer the function's effects.
        if let Some(declared) = declared.clone() {
            if let Err(e) = self.check_effects(&declared) {
                self.errors.push(TypeError::new(format!(
                    "Function '{}' effect mismatch: {}",
                    func.name, e
                )));
            }
        }

        if let Some(signature) = self.function_sigs.get_mut(&func.name) {
            signature.2 = Some(declared.unwrap_or(inferred_effects));
        }

        env.pop_scope();
    }

    // -----------------------------------------------------------------
    // Statement inference
    // -----------------------------------------------------------------

    /// Infer types for a single statement, emitting constraints as needed.
    pub fn infer_statement(&mut self, stmt: &ast::Statement, env: &mut TypeEnv, fn_ret: &Type) {
        match stmt {
            // -- let / var bindings --
            ast::Statement::Let {
                name, ty, value, ..
            } => {
                let val_ty = match value {
                    Some(v) => self.infer_expr(v, env),
                    None => Type::Void,
                };
                let declared_ty = match ty {
                    Some(t) => {
                        let dt = self.from_ast_type(t);
                        self.add_constraint(Constraint::Equal(
                            dt.clone(),
                            val_ty.clone(),
                            ConstraintOrigin::new(format!(
                                "let binding '{}': declared type must match initializer",
                                name
                            )),
                        ));
                        dt
                    }
                    None => val_ty.clone(),
                };
                env.define(name, declared_ty.clone());
                self.result.variable_types.insert(name.clone(), declared_ty);
            }

            ast::Statement::Var { name, ty, value } => {
                let var_ty = match (ty, value) {
                    (Some(t), Some(v)) => {
                        let dt = self.from_ast_type(t);
                        let vt = self.infer_expr(v, env);
                        self.add_constraint(Constraint::Equal(
                            dt.clone(),
                            vt,
                            ConstraintOrigin::new(format!(
                                "var binding '{}': declared type must match initializer",
                                name
                            )),
                        ));
                        dt
                    }
                    (Some(t), None) => self.from_ast_type(t),
                    (None, Some(v)) => self.infer_expr(v, env),
                    (None, None) => self.fresh_var(),
                };
                env.define(name, var_ty.clone());
                self.result.variable_types.insert(name.clone(), var_ty);
            }

            // -- assignment --
            ast::Statement::Assignment { target, value, op } => {
                let target_ty = self.infer_expr(target, env);
                let value_ty = self.infer_expr(value, env);
                if let Some(bin_op) = op {
                    // Compound assignment (+=, -= etc.) – operands must be numeric
                    if is_arithmetic_op(*bin_op) {
                        self.constrain_numeric(
                            &target_ty,
                            &format!("compound assignment {:?}", bin_op),
                        );
                        self.constrain_numeric(
                            &value_ty,
                            &format!("compound assignment {:?}", bin_op),
                        );
                    }
                }
                // The value must be compatible with the target
                self.add_constraint(Constraint::Equal(
                    target_ty,
                    value_ty,
                    ConstraintOrigin::new("assignment: target type must match value type"),
                ));
            }

            // -- return --
            ast::Statement::Return(maybe_expr) => {
                let ret_ty = match maybe_expr {
                    Some(e) => self.infer_expr(e, env),
                    None => Type::Void,
                };
                self.add_constraint(Constraint::Equal(
                    fn_ret.clone(),
                    ret_ty,
                    ConstraintOrigin::new("return type must match function signature"),
                ));
            }

            // -- if/else --
            ast::Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                let cond_ty = self.infer_expr(condition, env);
                self.add_constraint(Constraint::Equal(
                    cond_ty,
                    Type::Bool,
                    ConstraintOrigin::new("if condition must be Bool"),
                ));
                env.push_scope();
                for s in &then_block.statements {
                    self.infer_statement(s, env, fn_ret);
                }
                env.pop_scope();

                if let Some(else_blk) = else_block {
                    env.push_scope();
                    for s in &else_blk.statements {
                        self.infer_statement(s, env, fn_ret);
                    }
                    env.pop_scope();
                }
            }

            // -- for loop --
            ast::Statement::For { var, iter, body } => {
                let iter_ty = self.infer_expr(iter, env);
                let elem_ty = self.fresh_var();
                // The iterator expression should produce an array of elem_ty
                self.add_constraint(Constraint::Equal(
                    iter_ty,
                    Type::Array(Box::new(elem_ty.clone())),
                    ConstraintOrigin::new(format!(
                        "for loop: iterator must be an array, binding '{}'",
                        var
                    )),
                ));
                env.push_scope();
                env.define(var, elem_ty);
                for s in &body.statements {
                    self.infer_statement(s, env, fn_ret);
                }
                env.pop_scope();
            }

            // -- while loop --
            ast::Statement::While { condition, body } => {
                let cond_ty = self.infer_expr(condition, env);
                self.add_constraint(Constraint::Equal(
                    cond_ty,
                    Type::Bool,
                    ConstraintOrigin::new("while condition must be Bool"),
                ));
                env.push_scope();
                for s in &body.statements {
                    self.infer_statement(s, env, fn_ret);
                }
                env.pop_scope();
            }

            // -- loop --
            ast::Statement::Loop { body } => {
                env.push_scope();
                for s in &body.statements {
                    self.infer_statement(s, env, fn_ret);
                }
                env.pop_scope();
            }

            // -- match --
            ast::Statement::Match { expr, arms } => {
                let scrutinee_ty = self.infer_expr(expr, env);
                for arm in arms {
                    env.push_scope();
                    self.bind_pattern(&arm.pattern, &scrutinee_ty, env);
                    match &arm.body {
                        ast::MatchBody::Expr(e) => {
                            self.infer_expr(e, env);
                        }
                        ast::MatchBody::Block(blk) => {
                            for s in &blk.statements {
                                self.infer_statement(s, env, fn_ret);
                            }
                        }
                    }
                    env.pop_scope();
                }
            }

            // -- expression statement --
            ast::Statement::Expression(e) => {
                self.infer_expr(e, env);
            }

            // -- defer --
            ast::Statement::Defer(inner) => {
                self.infer_statement(inner, env, fn_ret);
            }

            // -- yield --
            ast::Statement::Yield(maybe_expr) => {
                if let Some(e) = maybe_expr {
                    self.infer_expr(e, env);
                }
            }

            // -- spawn --
            ast::Statement::Spawn(expr) => {
                self.infer_expr(expr, env);
            }

            // -- select --
            ast::Statement::Select { arms } => {
                for arm in arms {
                    env.push_scope();
                    self.infer_expr(&arm.channel_op, env);
                    for s in &arm.body.statements {
                        self.infer_statement(s, env, fn_ret);
                    }
                    env.pop_scope();
                }
            }

            // -- break, continue, pass – no types involved --
            ast::Statement::Break(_) | ast::Statement::Continue | ast::Statement::Pass => {}
        }
    }

    // -----------------------------------------------------------------
    // Expression inference
    // -----------------------------------------------------------------

    /// Infer the type of an expression, emitting constraints and returning
    /// the (possibly unresolved) type.
    pub fn infer_expr(&mut self, expr: &ast::Expression, env: &mut TypeEnv) -> Type {
        match expr {
            // -- Literals --
            ast::Expression::Literal(lit) => self.infer_literal(lit),
            // -- F-strings (treated as string) --
            ast::Expression::FString(_) => Type::String,

            // -- Identifiers --
            ast::Expression::Identifier(name) => {
                if let Some(ty) = env.lookup(name) {
                    ty.clone()
                } else {
                    self.errors
                        .push(TypeError::new(format!("Undefined variable '{}'", name)));
                    Type::Error
                }
            }

            // -- Binary operations --
            ast::Expression::Binary(lhs, op, rhs) => {
                let lt = self.infer_expr(lhs, env);
                let rt = self.infer_expr(rhs, env);
                self.infer_binary_op(&lt, *op, &rt)
            }

            // -- Unary operations --
            ast::Expression::Unary(op, operand) => {
                let ot = self.infer_expr(operand, env);
                match op {
                    ast::UnaryOp::Neg => {
                        self.constrain_numeric(&ot, "unary negation");
                        ot
                    }
                    ast::UnaryOp::Not => {
                        self.add_constraint(Constraint::Equal(
                            ot,
                            Type::Bool,
                            ConstraintOrigin::new("logical NOT requires Bool operand"),
                        ));
                        Type::Bool
                    }
                }
            }

            // -- Function / method calls --
            ast::Expression::Call(callee, args) => {
                let callee_ty = self.infer_expr(callee, env);
                let arg_tys: Vec<Type> = args.iter().map(|a| self.infer_expr(a, env)).collect();
                let ret_ty = self.fresh_var();
                let fresh_callee_ty = self.freshen_type_variables(&callee_ty);
                self.add_constraint(Constraint::Callable(
                    fresh_callee_ty,
                    arg_tys,
                    ret_ty.clone(),
                    ConstraintOrigin::new("function call"),
                ));

                // Track effect for known callees first; fall back to the legacy
                // name-based IO heuristic for unresolved builtins.
                if let ast::Expression::Identifier(name) = callee.as_ref() {
                    let propagated = self.merge_registered_effects(name);
                    if !propagated {
                        match name.as_str() {
                            "println" | "print" | "println!" | "eprintln" | "eprint" => {
                                self.add_io_effect();
                            }
                            "read_line" | "read" | "read_to_string" | "read_bytes" => {
                                self.add_io_effect();
                            }
                            "write" | "write_all" | "flush" => {
                                self.add_io_effect();
                            }
                            _ => {}
                        }
                    }
                }

                ret_ty
            }

            ast::Expression::MethodCall {
                receiver,
                method,
                args,
            } => {
                let recv_ty = self.infer_expr(receiver, env);
                let arg_tys: Vec<Type> = args.iter().map(|a| self.infer_expr(a, env)).collect();
                self.infer_method_call(&recv_ty, method, &arg_tys)
            }

            // -- Field access --
            ast::Expression::Field(base, field_name) => {
                let base_ty = self.infer_expr(base, env);
                let field_ty = self.fresh_var();
                self.add_constraint(Constraint::HasField(
                    base_ty,
                    field_name.clone(),
                    field_ty.clone(),
                    ConstraintOrigin::new(format!("field access '.{}'", field_name)),
                ));
                field_ty
            }

            // -- Index --
            ast::Expression::Index(base, index) => {
                let base_ty = self.infer_expr(base, env);
                if matches!(index.as_ref(), ast::Expression::Range { .. }) {
                    // Slicing returns a collection-like value rather than a scalar element.
                    return base_ty;
                }

                let idx_ty = self.infer_expr(index, env);
                self.constrain_numeric(&idx_ty, "array index");
                let elem_ty = self.fresh_var();
                self.add_constraint(Constraint::Equal(
                    base_ty,
                    Type::Array(Box::new(elem_ty.clone())),
                    ConstraintOrigin::new("indexing requires an array"),
                ));
                elem_ty
            }

            // -- Array literal --
            ast::Expression::Array(elems) => {
                if elems.is_empty() {
                    Type::Array(Box::new(self.fresh_var()))
                } else {
                    let first_ty = self.infer_expr(&elems[0], env);
                    for (i, elem) in elems.iter().enumerate().skip(1) {
                        let et = self.infer_expr(elem, env);
                        self.add_constraint(Constraint::Equal(
                            first_ty.clone(),
                            et,
                            ConstraintOrigin::new(format!(
                                "array element {} must have the same type as element 0",
                                i
                            )),
                        ));
                    }
                    Type::Array(Box::new(first_ty))
                }
            }

            // -- Tuple --
            ast::Expression::Tuple(elems) => {
                let tys: Vec<Type> = elems.iter().map(|e| self.infer_expr(e, env)).collect();
                Type::Tuple(tys)
            }

            // -- Struct literal --
            ast::Expression::StructLiteral { name, fields } => {
                let field_tys: Vec<(std::string::String, Type)> = fields
                    .iter()
                    .map(|(fname, fexpr)| {
                        let ft = self.infer_expr(fexpr, env);
                        (fname.clone(), ft)
                    })
                    .collect();

                // If we know the struct definition, constrain fields to match
                if let Some(def_fields) = self.struct_defs.get(name).cloned() {
                    for (fname, fty) in &field_tys {
                        if let Some((_, expected_ty)) = def_fields.iter().find(|(n, _)| n == fname)
                        {
                            self.add_constraint(Constraint::Equal(
                                expected_ty.clone(),
                                fty.clone(),
                                ConstraintOrigin::new(format!(
                                    "struct '{}' field '{}' type mismatch",
                                    name, fname
                                )),
                            ));
                        } else {
                            self.errors.push(TypeError::new(format!(
                                "Struct '{}' has no field '{}'",
                                name, fname
                            )));
                        }
                    }
                }

                Type::Struct(name.clone(), field_tys)
            }

            // -- Borrow --
            ast::Expression::Borrow { expr, .. } => self.infer_expr(expr, env),

            // -- Deref --
            ast::Expression::Deref(inner) => self.infer_expr(inner, env),

            // -- Await --
            ast::Expression::Await(inner) => {
                // For now, the awaited value's type is the inner type
                // Await expressions have async effects
                self.add_async_effect();
                self.infer_expr(inner, env)
            }

            // -- Ownership annotations (shared/own pass-through) --
            ast::Expression::Shared(inner) | ast::Expression::Own(inner) => {
                self.infer_expr(inner, env)
            }

            // -- Lambda --
            ast::Expression::Lambda { params, body, .. } => {
                env.push_scope();
                let param_tys: Vec<Type> = params
                    .iter()
                    .map(|p| {
                        let ty = self.from_ast_type(&p.ty);
                        env.define(&p.name, ty.clone());
                        ty
                    })
                    .collect();
                let body_ty = self.infer_expr(body, env);
                env.pop_scope();
                Type::Function(param_tys, Box::new(body_ty))
            }

            // -- Let-chain --
            ast::Expression::LetChain { name, value, body } => {
                let value_ty = self.infer_expr(value, env);
                env.push_scope();
                env.define(name, value_ty);
                let body_ty = self.infer_expr(body, env);
                env.pop_scope();
                body_ty
            }

            // -- If expression --
            ast::Expression::If {
                condition,
                then_expr,
                else_expr,
            } => {
                let cond_ty = self.infer_expr(condition, env);
                self.add_constraint(Constraint::Equal(
                    cond_ty,
                    Type::Bool,
                    ConstraintOrigin::new("if-expression condition must be Bool"),
                ));
                let then_ty = self.infer_expr(then_expr, env);
                if let Some(else_e) = else_expr {
                    let else_ty = self.infer_expr(else_e, env);
                    self.add_constraint(Constraint::Equal(
                        then_ty.clone(),
                        else_ty,
                        ConstraintOrigin::new(
                            "if-expression: both branches must produce the same type",
                        ),
                    ));
                }
                then_ty
            }

            // -- Match expression --
            ast::Expression::Match { expr, arms } => {
                let scrutinee_ty = self.infer_expr(expr, env);
                let result_ty = self.fresh_var();
                for arm in arms {
                    env.push_scope();
                    self.bind_pattern(&arm.pattern, &scrutinee_ty, env);
                    let arm_ty = match &arm.body {
                        ast::MatchBody::Expr(e) => self.infer_expr(e, env),
                        ast::MatchBody::Block(blk) => {
                            let mut last = Type::Void;
                            for s in &blk.statements {
                                self.infer_statement(s, env, &result_ty);
                                // If the last statement is an expression, use its type
                                if let ast::Statement::Expression(e) = s {
                                    last = self.infer_expr(e, env);
                                }
                            }
                            last
                        }
                    };
                    self.add_constraint(Constraint::Equal(
                        result_ty.clone(),
                        arm_ty,
                        ConstraintOrigin::new("match arms must have the same type"),
                    ));
                    env.pop_scope();
                }
                result_ty
            }

            // -- Range --
            ast::Expression::Range { start, end, .. } => {
                if let Some(s) = start {
                    let st = self.infer_expr(s, env);
                    self.constrain_numeric(&st, "range start");
                }
                if let Some(e) = end {
                    let et = self.infer_expr(e, env);
                    self.constrain_numeric(&et, "range end");
                }
                Type::Array(Box::new(Type::Int))
            }

            // -- List comprehension --
            ast::Expression::ListComprehension {
                expr: comp_expr,
                var,
                iter,
                filter,
            } => {
                let iter_ty = self.infer_expr(iter, env);
                let elem_ty = self.fresh_var();
                self.add_constraint(Constraint::Equal(
                    iter_ty,
                    Type::Array(Box::new(elem_ty.clone())),
                    ConstraintOrigin::new("list comprehension: iterator must be an array"),
                ));
                env.push_scope();
                env.define(var, elem_ty);
                if let Some(f) = filter {
                    let ft = self.infer_expr(f, env);
                    self.add_constraint(Constraint::Equal(
                        ft,
                        Type::Bool,
                        ConstraintOrigin::new("list comprehension filter must be Bool"),
                    ));
                }
                let result_elem = self.infer_expr(comp_expr, env);
                env.pop_scope();
                Type::Array(Box::new(result_elem))
            }

            // -- Generator --
            ast::Expression::Generator { body } => {
                let elem_ty = self.fresh_var();
                env.push_scope();
                for s in &body.statements {
                    self.infer_statement(s, env, &Type::Void);
                }
                env.pop_scope();
                Type::Array(Box::new(elem_ty))
            }

            // -- Path (module::item) --
            ast::Expression::Path(base, member) => {
                // Treat like field access for inference purposes
                let base_ty = self.infer_expr(base, env);
                let field_ty = self.fresh_var();
                self.add_constraint(Constraint::HasField(
                    base_ty,
                    member.clone(),
                    field_ty.clone(),
                    ConstraintOrigin::new(format!("path access '::{}'", member)),
                ));
                field_ty
            }

            // -- Option / Result wrappers --
            ast::Expression::None => Type::Nullable(Box::new(self.fresh_var())),
            ast::Expression::Some(inner) => {
                let inner_ty = self.infer_expr(inner, env);
                Type::Nullable(Box::new(inner_ty))
            }
            ast::Expression::Ok(inner) | ast::Expression::Err(inner) => self.infer_expr(inner, env),
        }
    }

    // -----------------------------------------------------------------
    // Literal inference
    // -----------------------------------------------------------------

    fn infer_literal(&self, lit: &ast::Literal) -> Type {
        match lit {
            ast::Literal::Int(_) => Type::Int,
            ast::Literal::Float(_) => Type::Float,
            ast::Literal::String(_) => Type::String,
            ast::Literal::Bool(_) => Type::Bool,
            ast::Literal::Null => Type::Nullable(Box::new(Type::Void)),
        }
    }

    // -----------------------------------------------------------------
    // Binary operator inference
    // -----------------------------------------------------------------

    fn infer_binary_op(&mut self, lhs_ty: &Type, op: ast::BinaryOp, rhs_ty: &Type) -> Type {
        match op {
            // Arithmetic → both operands numeric, result numeric
            ast::BinaryOp::Add
            | ast::BinaryOp::Sub
            | ast::BinaryOp::Mul
            | ast::BinaryOp::Div
            | ast::BinaryOp::Mod => {
                self.constrain_numeric(lhs_ty, &format!("left operand of {:?}", op));
                self.constrain_numeric(rhs_ty, &format!("right operand of {:?}", op));
                // Allow implicit widening: Int + Float → Float, Float + Int → Float
                // (O-025: numeric coercion rules)
                let resolved_lhs = self.substitution.apply(lhs_ty);
                let resolved_rhs = self.substitution.apply(rhs_ty);
                match (&resolved_lhs, &resolved_rhs) {
                    (Type::Int, Type::Float)
                    | (Type::Float, Type::Int)
                    | (Type::Float, Type::Float) => {
                        // Mixed int/float → widen to Float
                        Type::Float
                    }
                    _ => {
                        // Both Int (or unknown) — require exact match
                        self.add_constraint(Constraint::Equal(
                            lhs_ty.clone(),
                            rhs_ty.clone(),
                            ConstraintOrigin::new(format!(
                                "arithmetic {:?}: operands must be numeric (Int or Float)",
                                op
                            )),
                        ));
                        lhs_ty.clone()
                    }
                }
            }

            // Comparison → both operands same type, result Bool
            ast::BinaryOp::Eq
            | ast::BinaryOp::NotEq
            | ast::BinaryOp::Lt
            | ast::BinaryOp::Gt
            | ast::BinaryOp::LtEq
            | ast::BinaryOp::GtEq => {
                self.add_constraint(Constraint::Equal(
                    lhs_ty.clone(),
                    rhs_ty.clone(),
                    ConstraintOrigin::new(format!(
                        "comparison {:?}: both operands must have the same type",
                        op
                    )),
                ));
                Type::Bool
            }

            // Logical → both Bool, result Bool
            ast::BinaryOp::And | ast::BinaryOp::Or => {
                self.add_constraint(Constraint::Equal(
                    lhs_ty.clone(),
                    Type::Bool,
                    ConstraintOrigin::new(format!("logical {:?}: left operand must be Bool", op)),
                ));
                self.add_constraint(Constraint::Equal(
                    rhs_ty.clone(),
                    Type::Bool,
                    ConstraintOrigin::new(format!("logical {:?}: right operand must be Bool", op)),
                ));
                Type::Bool
            }

            // Range → numeric start/end, result is array of Int
            ast::BinaryOp::Range | ast::BinaryOp::RangeInclusive => {
                self.constrain_numeric(lhs_ty, "range start");
                self.constrain_numeric(rhs_ty, "range end");
                Type::Array(Box::new(Type::Int))
            }
        }
    }

    // -----------------------------------------------------------------
    // Method call inference
    // -----------------------------------------------------------------

    fn infer_method_call(&mut self, recv_ty: &Type, method: &str, arg_tys: &[Type]) -> Type {
        // Try to resolve from registered impl methods
        let resolved = self.substitution.apply(recv_ty);
        if let Type::Struct(ref struct_name, _) = resolved {
            let qualified = format!("{}::{}", struct_name, method);
            if let Some((param_tys, ret_ty, effect_row)) =
                self.function_sigs.get(&qualified).cloned()
            {
                if let Some(effect_row) = effect_row {
                    self.merge_effects(&effect_row);
                }
                // Skip the first param if it's Self
                let method_params = if !param_tys.is_empty() {
                    &param_tys[1..]
                } else {
                    &param_tys[..]
                };
                if arg_tys.len() != method_params.len() {
                    self.errors.push(TypeError::new(format!(
                        "Method '{}' on '{}' expects {} arguments, got {}",
                        method,
                        struct_name,
                        method_params.len(),
                        arg_tys.len()
                    )));
                } else {
                    for (i, (expected, actual)) in
                        method_params.iter().zip(arg_tys.iter()).enumerate()
                    {
                        self.add_constraint(Constraint::Equal(
                            expected.clone(),
                            actual.clone(),
                            ConstraintOrigin::new(format!(
                                "method '{}' argument {} type mismatch",
                                method, i
                            )),
                        ));
                    }
                }
                return ret_ty;
            }
        }

        // Built-in methods on known types
        match (&resolved, method) {
            (Type::Array(_), "len") => return Type::Int,
            (Type::Array(elem), "push") => {
                if let Some(arg_ty) = arg_tys.first() {
                    self.add_constraint(Constraint::Equal(
                        (**elem).clone(),
                        arg_ty.clone(),
                        ConstraintOrigin::new("Array.push: element type must match"),
                    ));
                }
                return Type::Void;
            }
            (Type::Array(elem), "pop") => return Type::Nullable(elem.clone()),
            (Type::Array(elem), "first" | "last") => {
                return Type::Nullable(elem.clone());
            }
            (Type::Array(_elem), "map") => {
                let out_elem = self.fresh_var();
                return Type::Array(Box::new(out_elem));
            }
            (Type::Array(elem), "filter") => return Type::Array(elem.clone()),
            (Type::Array(_), "contains") => return Type::Bool,
            (Type::Array(_), "is_empty") => return Type::Bool,
            (Type::String, "len") => return Type::Int,
            (Type::String, "contains" | "starts_with" | "ends_with") => return Type::Bool,
            (Type::String, "to_uppercase" | "to_lowercase" | "trim") => return Type::String,
            (Type::String, "split") => return Type::Array(Box::new(Type::String)),
            (Type::String, "parse_int") => return Type::Nullable(Box::new(Type::Int)),
            (Type::String, "parse_float") => return Type::Nullable(Box::new(Type::Float)),
            _ => {}
        }

        // Fall back: emit a Callable constraint and return a fresh variable
        let ret_ty = self.fresh_var();
        let callee_ty = self.fresh_var();
        self.add_constraint(Constraint::Callable(
            callee_ty,
            arg_tys.to_vec(),
            ret_ty.clone(),
            ConstraintOrigin::new(format!("method call '.{}(…)'", method)),
        ));
        ret_ty
    }

    // -----------------------------------------------------------------
    // Pattern binding
    // -----------------------------------------------------------------

    fn bind_pattern(&mut self, pat: &ast::Pattern, expected_ty: &Type, env: &mut TypeEnv) {
        match pat {
            ast::Pattern::Binding(name) => {
                env.define(name, expected_ty.clone());
            }
            ast::Pattern::Literal(lit) => {
                let lit_ty = self.infer_literal(lit);
                self.add_constraint(Constraint::Equal(
                    expected_ty.clone(),
                    lit_ty,
                    ConstraintOrigin::new("pattern literal must match scrutinee type"),
                ));
            }
            ast::Pattern::Constructor(_name, sub_pats) => {
                // Each sub-pattern gets a fresh variable
                for sp in sub_pats {
                    let sub_ty = self.fresh_var();
                    self.bind_pattern(sp, &sub_ty, env);
                }
            }
            ast::Pattern::Wildcard => { /* matches anything */ }
            ast::Pattern::Or(patterns) => {
                // OR patterns: bind each sub-pattern with the same expected type
                for sub_pat in patterns {
                    self.bind_pattern(sub_pat, expected_ty, env);
                }
            }
        }
    }

    // -----------------------------------------------------------------
    // Constraint management
    // -----------------------------------------------------------------

    fn add_constraint(&mut self, c: Constraint) {
        self.constraints.push(c);
    }

    /// Emit constraints that ensure `ty` is a numeric type (Int or Float).
    fn constrain_numeric(&mut self, ty: &Type, context: &str) {
        // If the type is already known to be non-numeric, report immediately
        let resolved = self.substitution.apply(ty);
        match &resolved {
            Type::Int | Type::Float | Type::Var(_) | Type::Error => { /* ok or unknown */ }
            other => {
                self.errors.push(
                    TypeError::new(format!(
                        "Expected numeric type (Int or Float) but found {} in {}",
                        other, context
                    ))
                    .with_hint("Arithmetic operations require numeric operands"),
                );
            }
        }
    }

    // -----------------------------------------------------------------
    // Constraint solving (unification)
    // -----------------------------------------------------------------

    /// Process all collected constraints by unifying types.
    fn solve_constraints(&mut self) {
        // Take constraints out so we can iterate without borrow conflicts
        let constraints = std::mem::take(&mut self.constraints);
        for c in constraints.iter() {
            // Ensure the runtime monitor sees progress even for small constraint sets.
            monitor::update_heartbeat();
            match c {
                Constraint::Equal(t1, t2, origin) => {
                    let a = self.substitution.apply(t1);
                    let b = self.substitution.apply(t2);
                    if let Err(e) = self.unify(&a, &b) {
                        self.errors.push(
                            TypeError::new(format!(
                                "Type mismatch: {} vs {} – {}",
                                a, b, origin.description
                            ))
                            .with_hint(e),
                        );
                    }
                }
                Constraint::HasField(recv, field, field_ty, origin) => {
                    let resolved_recv = self.substitution.apply(recv);
                    let resolved_field_ty = self.substitution.apply(field_ty);
                    if let Type::Struct(name, fields) = &resolved_recv {
                        if let Some((_, ft)) = fields.iter().find(|(n, _)| n == field) {
                            if let Err(_e) = self.unify(&resolved_field_ty, ft) {
                                self.errors.push(TypeError::new(format!(
                                    "Field '{}' on struct '{}' has type {}, not {} – {}",
                                    field, name, ft, resolved_field_ty, origin.description
                                )));
                            }
                        } else if let Some(def_fields) = self.struct_defs.get(name).cloned() {
                            if let Some((_, ft)) = def_fields.iter().find(|(n, _)| n == field) {
                                if let Err(_e) = self.unify(&resolved_field_ty, ft) {
                                    self.errors.push(TypeError::new(format!(
                                        "Field '{}' on struct '{}' has type {}, not {}",
                                        field, name, ft, resolved_field_ty
                                    )));
                                }
                            } else {
                                self.errors.push(TypeError::new(format!(
                                    "Struct '{}' has no field '{}' – {}",
                                    name, field, origin.description
                                )));
                            }
                        }
                    }
                    // If the receiver is a type variable, we cannot resolve now –
                    // we just leave it (a more sophisticated engine would iterate).
                }
                Constraint::Callable(callee, args, ret, origin) => {
                    let resolved_callee = self.substitution.apply(callee);
                    if let Type::Function(param_tys, ret_ty) = &resolved_callee {
                        // Unify return type
                        let resolved_ret = self.substitution.apply(ret);
                        if let Err(_e) = self.unify(&resolved_ret, ret_ty) {
                            self.errors.push(TypeError::new(format!(
                                "Return type mismatch in call: expected {}, got {} – {}",
                                ret_ty, resolved_ret, origin.description
                            )));
                        }
                        // Unify argument types
                        if args.len() != param_tys.len() {
                            self.errors.push(TypeError::new(format!(
                                "Function expects {} arguments but {} were provided – {}",
                                param_tys.len(),
                                args.len(),
                                origin.description
                            )));
                        } else {
                            for (i, (expected, actual)) in
                                param_tys.iter().zip(args.iter()).enumerate()
                            {
                                let a = self.substitution.apply(actual);
                                let e = self.substitution.apply(expected);
                                if let Err(_err) = self.unify(&a, &e) {
                                    self.errors.push(TypeError::new(format!(
                                        "Argument {} type mismatch: expected {}, got {} – {}",
                                        i, e, a, origin.description
                                    )));
                                }
                            }
                        }
                    }
                    // If callee is still a Var, we can't resolve yet
                }
            }
        }
    }

    // -----------------------------------------------------------------
    // Unification
    // -----------------------------------------------------------------

    /// Unify two types, updating the substitution. Returns `Ok(())` on success
    /// or an error description on failure.
    pub fn unify(&mut self, t1: &Type, t2: &Type) -> Result<(), std::string::String> {
        let a = self.substitution.apply(t1);
        let b = self.substitution.apply(t2);

        if a == b {
            return Ok(());
        }

        match (&a, &b) {
            // Error type unifies with anything (error recovery)
            (Type::Error, _) | (_, Type::Error) => Ok(()),

            // Never type unifies with anything (diverging code paths)
            (Type::Never, _) | (_, Type::Never) => Ok(()),

            // Bind a variable
            (Type::Var(id), _) => {
                if self.occurs_check(*id, &b) {
                    Err(format!("Infinite type: ?T{} occurs in {}", id, b))
                } else {
                    self.substitution.bind(*id, b);
                    Ok(())
                }
            }
            (_, Type::Var(id)) => {
                if self.occurs_check(*id, &a) {
                    Err(format!("Infinite type: ?T{} occurs in {}", id, a))
                } else {
                    self.substitution.bind(*id, a);
                    Ok(())
                }
            }

            // Nullable
            (Type::Nullable(inner1), Type::Nullable(inner2)) => self.unify(inner1, inner2),

            // Array
            (Type::Array(e1), Type::Array(e2)) => self.unify(e1, e2),

            // Map
            (Type::Map(k1, v1), Type::Map(k2, v2)) => {
                self.unify(k1, k2)?;
                self.unify(v1, v2)
            }

            // Tuple
            (Type::Tuple(elems1), Type::Tuple(elems2)) => {
                if elems1.len() != elems2.len() {
                    return Err(format!(
                        "Tuple length mismatch: {} vs {}",
                        elems1.len(),
                        elems2.len()
                    ));
                }
                for (e1, e2) in elems1.iter().zip(elems2.iter()) {
                    self.unify(e1, e2)?;
                }
                Ok(())
            }

            // Function
            (Type::Function(p1, r1), Type::Function(p2, r2)) => {
                if p1.len() != p2.len() {
                    return Err(format!(
                        "Function arity mismatch: {} params vs {}",
                        p1.len(),
                        p2.len()
                    ));
                }
                for (a, b) in p1.iter().zip(p2.iter()) {
                    self.unify(a, b)?;
                }
                self.unify(r1, r2)
            }

            // Struct – names must match, then unify field types
            (Type::Struct(n1, f1), Type::Struct(n2, f2)) => {
                if n1 != n2 {
                    return Err(format!("Struct type mismatch: {} vs {}", n1, n2));
                }
                for ((fn1, ft1), (fn2, ft2)) in f1.iter().zip(f2.iter()) {
                    if fn1 != fn2 {
                        return Err(format!("Struct field name mismatch: {} vs {}", fn1, fn2));
                    }
                    self.unify(ft1, ft2)?;
                }
                Ok(())
            }

            // Applied generic
            (Type::Applied(b1, a1), Type::Applied(b2, a2)) => {
                self.unify(b1, b2)?;
                if a1.len() != a2.len() {
                    return Err(format!(
                        "Generic argument count mismatch: {} vs {}",
                        a1.len(),
                        a2.len()
                    ));
                }
                for (x, y) in a1.iter().zip(a2.iter()) {
                    self.unify(x, y)?;
                }
                Ok(())
            }

            // Generic names must match exactly
            (Type::Generic(n1), Type::Generic(n2)) if n1 == n2 => Ok(()),

            // Trait objects – names must match
            (Type::TraitObject(n1), Type::TraitObject(n2)) if n1 == n2 => Ok(()),

            // Int and Float are both numeric – allow in some contexts
            // but for strict unification they are different types
            _ => Err(format!("Cannot unify {} with {}", a, b)),
        }
    }

    // -----------------------------------------------------------------
    // Occurs check
    // -----------------------------------------------------------------

    /// Returns `true` if `var` occurs anywhere inside `ty`, which would
    /// create an infinite type during unification.
    pub fn occurs_check(&self, var: TypeVarId, ty: &Type) -> bool {
        match ty {
            Type::Var(id) => {
                if *id == var {
                    return true;
                }
                // Chase through substitution
                if let Some(resolved) = self.substitution.lookup(*id) {
                    self.occurs_check(var, resolved)
                } else {
                    false
                }
            }
            Type::Nullable(inner) => self.occurs_check(var, inner),
            Type::Array(elem) => self.occurs_check(var, elem),
            Type::Map(k, v) => self.occurs_check(var, k) || self.occurs_check(var, v),
            Type::Tuple(elems) => elems.iter().any(|e| self.occurs_check(var, e)),
            Type::Function(params, ret) => {
                params.iter().any(|p| self.occurs_check(var, p)) || self.occurs_check(var, ret)
            }
            Type::Struct(_, fields) => fields.iter().any(|(_, ft)| self.occurs_check(var, ft)),
            Type::Applied(base, args) => {
                self.occurs_check(var, base) || args.iter().any(|a| self.occurs_check(var, a))
            }
            _ => false,
        }
    }

    // -----------------------------------------------------------------
    // Finalization – apply substitution to all recorded types
    // -----------------------------------------------------------------

    fn finalize_types(&mut self) {
        let var_types: HashMap<std::string::String, Type> = self
            .result
            .variable_types
            .iter()
            .map(|(name, ty)| {
                monitor::update_heartbeat();
                (name.clone(), self.substitution.apply(ty))
            })
            .collect();
        self.result.variable_types = var_types;

        let fn_types: HashMap<std::string::String, Type> = self
            .result
            .function_types
            .iter()
            .map(|(name, ty)| {
                monitor::update_heartbeat();
                (name.clone(), self.substitution.apply(ty))
            })
            .collect();
        self.result.function_types = fn_types;

        let struct_types: HashMap<std::string::String, Vec<(std::string::String, Type)>> = self
            .result
            .struct_types
            .iter()
            .map(|(name, fields)| {
                monitor::update_heartbeat();
                let resolved_fields: Vec<(std::string::String, Type)> = fields
                    .iter()
                    .map(|(fn_, ft)| {
                        monitor::update_heartbeat();
                        (fn_.clone(), self.substitution.apply(ft))
                    })
                    .collect();
                (name.clone(), resolved_fields)
            })
            .collect();
        self.result.struct_types = struct_types;

        let expr_types: HashMap<std::string::String, Type> = self
            .result
            .expr_types
            .iter()
            .map(|(k, ty)| {
                monitor::update_heartbeat();
                (k.clone(), self.substitution.apply(ty))
            })
            .collect();
        self.result.expr_types = expr_types;
    }

    // -----------------------------------------------------------------
    // Apply substitution (public convenience)
    // -----------------------------------------------------------------

    /// Resolve all type variables in `ty` using the current substitution.
    pub fn apply_substitution(&self, ty: &Type) -> Type {
        self.substitution.apply(ty)
    }

    // -----------------------------------------------------------------
    // Generate constraints (batch entry point)
    // -----------------------------------------------------------------

    /// Walk an entire module and generate constraints without solving them.
    /// Useful for two-phase workflows (generate then solve).
    pub fn generate_constraints(&mut self, module: &ast::Module) {
        let mut env = TypeEnv::new();

        // Register declarations
        for item in &module.items {
            match item {
                ast::Item::Struct(sdef) => self.register_struct(sdef),
                ast::Item::Function(func) => self.register_function(func, &mut env),
                ast::Item::Impl(impl_block) => {
                    for method in &impl_block.methods {
                        let qualified = format!("{}::{}", impl_block.type_name, method.name);
                        self.register_function_with_name(&qualified, method, &mut env);
                    }
                }
                _ => {}
            }
        }

        // Collect constraints from function bodies
        for item in &module.items {
            if let ast::Item::Function(func) = item {
                self.infer_function(func, &mut env);
            }
            if let ast::Item::Impl(impl_block) = item {
                for method in &impl_block.methods {
                    self.infer_function(method, &mut env);
                }
            }
        }
    }

    /// Freshen all type variables in a type, replacing them with new unique variables.
    /// This is used for function call sites to ensure each call has independent type variables.
    fn freshen_type_variables(&mut self, ty: &Type) -> Type {
        match ty {
            Type::Var(_) => self.fresh_var(),
            Type::Nullable(inner) => Type::Nullable(Box::new(self.freshen_type_variables(inner))),
            Type::Array(elem) => Type::Array(Box::new(self.freshen_type_variables(elem))),
            Type::Map(k, v) => Type::Map(
                Box::new(self.freshen_type_variables(k)),
                Box::new(self.freshen_type_variables(v)),
            ),
            Type::Tuple(elems) => Type::Tuple(
                elems
                    .iter()
                    .map(|e| self.freshen_type_variables(e))
                    .collect(),
            ),
            Type::Function(params, ret) => Type::Function(
                params
                    .iter()
                    .map(|p| self.freshen_type_variables(p))
                    .collect(),
                Box::new(self.freshen_type_variables(ret)),
            ),
            Type::Struct(name, fields) => Type::Struct(
                name.clone(),
                fields
                    .iter()
                    .map(|(n, t)| (n.clone(), self.freshen_type_variables(t)))
                    .collect(),
            ),
            Type::Applied(base, args) => Type::Applied(
                Box::new(self.freshen_type_variables(base)),
                args.iter()
                    .map(|a| self.freshen_type_variables(a))
                    .collect(),
            ),
            _ => ty.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Helper: classify binary operators
// ---------------------------------------------------------------------------

fn is_arithmetic_op(op: ast::BinaryOp) -> bool {
    matches!(
        op,
        ast::BinaryOp::Add
            | ast::BinaryOp::Sub
            | ast::BinaryOp::Mul
            | ast::BinaryOp::Div
            | ast::BinaryOp::Mod
    )
}

// ---------------------------------------------------------------------------
// Public API – convenience entry point
// ---------------------------------------------------------------------------

/// Run full type inference on a parsed module.
///
/// Returns either the resolved `InferenceResult` (with types for every
/// variable, function, and struct) or a list of `TypeError`s.
///
/// # Example
/// ```ignore
/// let module: ast::Module = parse("…");
/// match check_types(&module) {
///     Ok(result) => println!("All types resolved!"),
///     Err(errors) => {
///         for e in &errors {
///             eprintln!("Type error: {}", e);
///         }
///     }
/// }
/// ```
pub fn check_types(module: &ast::Module) -> Result<InferenceResult, Vec<TypeError>> {
    let mut engine = InferenceEngine::new();
    engine.infer_module(module)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    // Import AST types under a qualified prefix to avoid collision with
    // our inference `Type`.
    use crate::parser::ast as a;

    /// Helper: build a minimal module with one function.
    fn module_with_fn(func: a::Function) -> a::Module {
        a::Module {
            items: vec![a::Item::Function(func)],
        }
    }

    /// Helper: build a trivial function with a body.
    fn simple_fn(name: &str, stmts: Vec<a::Statement>) -> a::Function {
        a::Function {
            name: name.to_string(),
            is_async: false,
            attributes: vec![],
            params: vec![],
            return_type: None,
            effect_row: None,
            body: a::Block { statements: stmts },
        }
    }

    fn io_effect_row() -> a::EffectRow {
        a::EffectRow {
            effects: vec![a::EffectSymbol {
                name: "IO".to_string(),
                param: None,
            }],
        }
    }

    fn pure_effect_row() -> a::EffectRow {
        a::EffectRow { effects: vec![] }
    }

    fn call_stmt(name: &str) -> a::Statement {
        a::Statement::Expression(a::Expression::Call(
            Box::new(a::Expression::Identifier(name.to_string())),
            vec![],
        ))
    }
    #[test]
    fn test_fresh_var_ids_increment() {
        let mut engine = InferenceEngine::new();
        let a = engine.fresh_var();
        let b = engine.fresh_var();
        let c = engine.fresh_var();
        assert_eq!(a, Type::Var(0));
        assert_eq!(b, Type::Var(1));
        assert_eq!(c, Type::Var(2));
    }

    #[test]
    fn test_literal_inference() {
        let engine = InferenceEngine::new();
        assert_eq!(engine.infer_literal(&a::Literal::Int(42)), Type::Int);
        assert_eq!(engine.infer_literal(&a::Literal::Float(3.14)), Type::Float);
        assert_eq!(
            engine.infer_literal(&a::Literal::String("hello".into())),
            Type::String
        );
        assert_eq!(engine.infer_literal(&a::Literal::Bool(true)), Type::Bool);
    }

    #[test]
    fn test_unify_same_types() {
        let mut engine = InferenceEngine::new();
        assert!(engine.unify(&Type::Int, &Type::Int).is_ok());
        assert!(engine.unify(&Type::Bool, &Type::Bool).is_ok());
        assert!(engine.unify(&Type::String, &Type::String).is_ok());
    }

    #[test]
    fn test_unify_different_types_fails() {
        let mut engine = InferenceEngine::new();
        assert!(engine.unify(&Type::Int, &Type::String).is_err());
        assert!(engine.unify(&Type::Bool, &Type::Float).is_err());
    }

    #[test]
    fn test_unify_var_binds() {
        let mut engine = InferenceEngine::new();
        let var = engine.fresh_var(); // Var(0)
        assert!(engine.unify(&var, &Type::Int).is_ok());
        let resolved = engine.apply_substitution(&var);
        assert_eq!(resolved, Type::Int);
    }

    #[test]
    fn test_unify_two_vars() {
        let mut engine = InferenceEngine::new();
        let a = engine.fresh_var(); // Var(0)
        let b = engine.fresh_var(); // Var(1)
        assert!(engine.unify(&a, &b).is_ok());
        // Bind one of them to a concrete type
        assert!(engine.unify(&b, &Type::Float).is_ok());
        assert_eq!(engine.apply_substitution(&a), Type::Float);
        assert_eq!(engine.apply_substitution(&b), Type::Float);
    }

    #[test]
    fn test_occurs_check() {
        let mut engine = InferenceEngine::new();
        let var = engine.fresh_var(); // Var(0)
        let infinite = Type::Array(Box::new(Type::Var(0)));
        assert!(engine.occurs_check(0, &infinite));
        assert!(engine.unify(&var, &infinite).is_err());
    }

    #[test]
    fn test_unify_arrays() {
        let mut engine = InferenceEngine::new();
        let a = Type::Array(Box::new(Type::Int));
        let b = Type::Array(Box::new(Type::Int));
        assert!(engine.unify(&a, &b).is_ok());

        let c = Type::Array(Box::new(Type::Int));
        let d = Type::Array(Box::new(Type::String));
        assert!(engine.unify(&c, &d).is_err());
    }

    #[test]
    fn test_unify_functions() {
        let mut engine = InferenceEngine::new();
        let f1 = Type::Function(vec![Type::Int, Type::Int], Box::new(Type::Int));
        let f2 = Type::Function(vec![Type::Int, Type::Int], Box::new(Type::Int));
        assert!(engine.unify(&f1, &f2).is_ok());

        let f3 = Type::Function(vec![Type::Int], Box::new(Type::Int));
        let f4 = Type::Function(vec![Type::Int, Type::Int], Box::new(Type::Int));
        assert!(engine.unify(&f3, &f4).is_err());
    }

    #[test]
    fn test_unify_tuples() {
        let mut engine = InferenceEngine::new();
        let t1 = Type::Tuple(vec![Type::Int, Type::Bool]);
        let t2 = Type::Tuple(vec![Type::Int, Type::Bool]);
        assert!(engine.unify(&t1, &t2).is_ok());

        let t3 = Type::Tuple(vec![Type::Int]);
        assert!(engine.unify(&t1, &t3).is_err());
    }

    #[test]
    fn test_substitution_chaining() {
        let mut sub = Substitution::new();
        // Var(0) → Var(1) → Int
        sub.bind(0, Type::Var(1));
        sub.bind(1, Type::Int);
        assert_eq!(sub.apply(&Type::Var(0)), Type::Int);
    }

    #[test]
    fn test_type_env_scoping() {
        let mut env = TypeEnv::new();
        env.define("x", Type::Int);
        assert_eq!(env.lookup("x"), Some(&Type::Int));

        env.push_scope();
        env.define("y", Type::Bool);
        assert_eq!(env.lookup("x"), Some(&Type::Int));
        assert_eq!(env.lookup("y"), Some(&Type::Bool));

        env.pop_scope();
        assert_eq!(env.lookup("x"), Some(&Type::Int));
        assert_eq!(env.lookup("y"), None);
    }

    #[test]
    fn test_type_env_shadowing() {
        let mut env = TypeEnv::new();
        env.define("x", Type::Int);
        env.push_scope();
        env.define("x", Type::String);
        assert_eq!(env.lookup("x"), Some(&Type::String));
        env.pop_scope();
        assert_eq!(env.lookup("x"), Some(&Type::Int));
    }

    #[test]
    fn test_infer_let_binding() {
        let module = module_with_fn(simple_fn(
            "main",
            vec![a::Statement::Let {
                name: "x".into(),
                mutable: false,
                ty: None,
                value: Some(a::Expression::Literal(a::Literal::Int(42))),
            }],
        ));
        let result = check_types(&module).expect("should succeed");
        assert_eq!(result.variable_types.get("x"), Some(&Type::Int));
    }

    #[test]
    fn test_infer_let_with_annotation_match() {
        let module = module_with_fn(simple_fn(
            "main",
            vec![a::Statement::Let {
                name: "x".into(),
                mutable: false,
                ty: Some(a::Type::I32),
                value: Some(a::Expression::Literal(a::Literal::Int(10))),
            }],
        ));
        let result = check_types(&module).expect("should succeed");
        assert_eq!(result.variable_types.get("x"), Some(&Type::Int));
    }

    #[test]
    fn test_infer_array_literal() {
        let mut engine = InferenceEngine::new();
        let mut env = TypeEnv::new();
        let expr = a::Expression::Array(vec![
            a::Expression::Literal(a::Literal::Int(1)),
            a::Expression::Literal(a::Literal::Int(2)),
            a::Expression::Literal(a::Literal::Int(3)),
        ]);
        let ty = engine.infer_expr(&expr, &mut env);
        assert_eq!(ty, Type::Array(Box::new(Type::Int)));
    }

    #[test]
    fn test_type_display() {
        assert_eq!(format!("{}", Type::Int), "Int");
        assert_eq!(format!("{}", Type::Array(Box::new(Type::Bool))), "[Bool]");
        assert_eq!(
            format!(
                "{}",
                Type::Function(vec![Type::Int, Type::String], Box::new(Type::Bool))
            ),
            "fn(Int, String) -> Bool"
        );
        assert_eq!(format!("{}", Type::Var(7)), "?T7");
        assert_eq!(
            format!("{}", Type::Tuple(vec![Type::Int, Type::Float])),
            "(Int, Float)"
        );
    }

    #[test]
    fn test_never_unifies_with_anything() {
        let mut engine = InferenceEngine::new();
        assert!(engine.unify(&Type::Never, &Type::Int).is_ok());
        assert!(engine.unify(&Type::String, &Type::Never).is_ok());
    }

    #[test]
    fn test_error_type_unifies_with_anything() {
        let mut engine = InferenceEngine::new();
        assert!(engine.unify(&Type::Error, &Type::Int).is_ok());
        assert!(engine.unify(&Type::Float, &Type::Error).is_ok());
    }

    #[test]
    fn test_empty_module_succeeds() {
        let module = a::Module { items: vec![] };
        let result = check_types(&module);
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_signature_stores_effect_row() {
        let mut engine = InferenceEngine::new();
        let module = a::Module {
            items: vec![a::Item::Function(a::Function {
                name: "emit".to_string(),
                is_async: false,
                attributes: vec![],
                params: vec![],
                return_type: None,
                effect_row: Some(io_effect_row()),
                body: a::Block { statements: vec![] },
            })],
        };

        engine.generate_constraints(&module);

        let signature = engine
            .function_sigs
            .get("emit")
            .expect("registered function signature");
        assert!(matches!(
            signature.2.as_ref(),
            Some(effect_row) if effect_row.contains(&crate::semantic::effects::builtin::io())
        ));
    }

    #[test]
    fn test_function_call_propagates_registered_effect_row() {
        let emit_fn = a::Function {
            name: "emit".to_string(),
            is_async: false,
            attributes: vec![],
            params: vec![],
            return_type: None,
            effect_row: Some(io_effect_row()),
            body: a::Block { statements: vec![] },
        };

        let caller_fn = simple_fn("main", vec![call_stmt("emit")]);
        let module = a::Module {
            items: vec![a::Item::Function(emit_fn), a::Item::Function(caller_fn)],
        };

        let result = check_types(&module);
        assert!(result.is_ok(), "unannotated callers should infer callee effects");
    }

    #[test]
    fn test_explicit_pure_effect_annotation_rejects_io_call() {
        let emit_fn = a::Function {
            name: "emit".to_string(),
            is_async: false,
            attributes: vec![],
            params: vec![],
            return_type: None,
            effect_row: Some(io_effect_row()),
            body: a::Block { statements: vec![] },
        };

        let mut caller_fn = simple_fn("main", vec![call_stmt("emit")]);
        caller_fn.effect_row = Some(pure_effect_row());

        let module = a::Module {
            items: vec![a::Item::Function(emit_fn), a::Item::Function(caller_fn)],
        };

        let result = check_types(&module);
        assert!(result.is_err());

        let errors = result.err().expect("expected effect mismatch error");
        assert!(errors.iter().any(|error| {
            error.message.contains("Function 'main' effect mismatch")
                || error.message.contains("function 'main' effect mismatch")
        }));
    }
}
