// Copyright 2024 Shreyash Jagtap
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(dead_code)]
//! Omni Semantic Analyzer - Core module
//! Implements type checking, borrow checking, lifetime inference, and trait bound verification.
//!
//! This module provides complete semantic analysis including:
//! - Type inference using Hindley-Milner algorithm
//! - Borrow checker with full ownership tracking
//! - Lifetime inference and validation
//! - Trait bound verification and resolution
//! - Generic monomorphization
//! - Const evaluation for compile-time computation
//!
//! ## Error Handling Model (O-024)
//!
//! Semantic errors are classified as hard (fatal) or soft (warning) via
//! `is_hard_type_error` in `type_inference.rs`. See that function's documentation
//! for the full classification rules. Hard errors abort compilation; soft errors
//! are demoted to warnings.

pub mod autograd;
pub mod borrow_check;
pub mod comprehensive_tests;
pub mod const_generics;
pub mod constraints;
pub mod edge_cases;
pub mod error_recovery;
pub mod inference;
pub mod integration_example;
pub mod lifetimes;
pub mod monomorphization;
pub mod optimization;
pub mod performance;
pub mod properties; // Properties & sealed classes support
pub mod traits;
pub mod type_inference; // Constraint-based Hindley-Milner type inference engine
pub mod types;

#[cfg(test)]
mod tests;

use crate::parser::ast::*;
use log::{debug, info, warn};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SemanticError {
    #[error("Type mismatch: expected {expected}, got {got}")]
    TypeMismatch { expected: String, got: String },

    #[error("Undefined symbol: {name}")]
    UndefinedSymbol { name: String },

    #[error("Duplicate definition: {name}")]
    DuplicateDefinition { name: String },

    #[error("Borrow error: {0}")]
    BorrowError(String),

    #[error("Lifetime error: {0}")]
    LifetimeError(String),

    #[error("Trait bound not satisfied: {0}")]
    TraitBoundError(String),

    #[error("Move error: value {name} used after move")]
    MoveError { name: String },

    #[error("Mutability error: {0}")]
    MutabilityError(String),

    #[error("Invalid syntax at line {line}: {message}")]
    InvalidSyntax { line: usize, message: String },

    #[error("Cannot break outside of loop")]
    BreakOutsideLoop,

    #[error("Cannot continue outside of loop")]
    ContinueOutsideLoop,

    #[error("Exhaustiveness error: match is not exhaustive")]
    NonExhaustiveMatch,

    #[error("Generic error: {0}")]
    GenericError(String),
    #[error("Unsupported construct: {0}")]
    UnsupportedConstruct(String),

    #[error("Custom error: {message}")]
    NewError { message: String },
}

impl SemanticError {
    /// Creates a new `SemanticError` with a custom message.
    pub fn new(message: &str) -> Self {
        SemanticError::NewError {
            message: message.to_string(),
        }
    }
}

/// Compare two types for structural equality
/// Expression doesn't implement PartialEq (due to f64), so Array sizes
/// use Debug-format comparison only for that specific subcase.
fn types_equal(a: &Type, b: &Type) -> bool {
    match (a, b) {
        // Primitive types
        (Type::U8, Type::U8)
        | (Type::U16, Type::U16)
        | (Type::U32, Type::U32)
        | (Type::U64, Type::U64)
        | (Type::Usize, Type::Usize)
        | (Type::I8, Type::I8)
        | (Type::I16, Type::I16)
        | (Type::I32, Type::I32)
        | (Type::I64, Type::I64)
        | (Type::Isize, Type::Isize)
        | (Type::F32, Type::F32)
        | (Type::F64, Type::F64)
        | (Type::Bool, Type::Bool)
        | (Type::Str, Type::Str)
        | (Type::SelfOwned, Type::SelfOwned) => true,

        // Type::Any matches any type (O-010)
        (Type::Any, _) | (_, Type::Any) => true,

        // Self references
        (Type::SelfRef { mutable: m1 }, Type::SelfRef { mutable: m2 }) => m1 == m2,

        // Named types
        (Type::Named(n1), Type::Named(n2)) => n1 == n2,

        // Generic types
        (Type::Generic(n1, args1), Type::Generic(n2, args2)) => {
            n1 == n2
                && args1.len() == args2.len()
                && args1.iter().zip(args2).all(|(a, b)| types_equal(a, b))
        }

        // Array types - use Debug fallback for Expression size comparison
        (Type::Array(t1, s1), Type::Array(t2, s2)) => {
            types_equal(t1, t2) && format!("{:?}", s1) == format!("{:?}", s2)
        }

        // Slice types
        (Type::Slice(t1), Type::Slice(t2)) => types_equal(t1, t2),

        // Function types
        (Type::Function(p1, r1), Type::Function(p2, r2)) => {
            p1.len() == p2.len()
                && p1.iter().zip(p2).all(|(a, b)| types_equal(a, b))
                && match (r1, r2) {
                    (Some(r1), Some(r2)) => types_equal(r1, r2),
                    (None, None) => true,
                    _ => false,
                }
        }

        // Ownership-modified types
        (Type::WithOwnership(t1, o1), Type::WithOwnership(t2, o2)) => {
            o1 == o2 && types_equal(t1, t2)
        }

        // Associated types: <T as Trait>::Type
        (Type::AssocType(t1, ty1), Type::AssocType(t2, ty2)) => t1 == t2 && ty1 == ty2,

        // Trait objects: dyn Trait
        (
            Type::TraitObject {
                principal: p1,
                supertraits: s1,
                lifetime: l1,
            },
            Type::TraitObject {
                principal: p2,
                supertraits: s2,
                lifetime: l2,
            },
        ) => p1 == p2 && s1 == s2 && l1 == l2,

        // Where-constrained types
        (
            Type::WhereConstrained {
                base: b1,
                bounds: bo1,
            },
            Type::WhereConstrained {
                base: b2,
                bounds: bo2,
            },
        ) => types_equal(b1, b2) && bo1 == bo2,

        // Higher-ranked types
        (Type::HigherRanked { bound: b1 }, Type::HigherRanked { bound: b2 }) => {
            format!("{:?}", b1) == format!("{:?}", b2)
        }

        // Const generics
        (Type::ConstGeneric(n1), Type::ConstGeneric(n2)) => n1 == n2,

        // All other combinations are not equal
        _ => false,
    }
}

/// Borrow state for a variable
#[derive(Debug, Clone, PartialEq)]
pub enum BorrowState {
    Owned,
    Moved,
    BorrowedShared(usize), // Count of shared borrows
    BorrowedMut,
    PartiallyMoved(Vec<String>), // Fields that have been moved
}

/// Lifetime representation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Lifetime {
    pub name: String,
    pub scope_id: usize,
    pub start_point: usize, // Start of lifetime
    pub end_point: usize,   // End of lifetime
}

impl Lifetime {
    pub fn static_lifetime() -> Self {
        Lifetime {
            name: "'static".to_string(),
            scope_id: 0,
            start_point: 0,
            end_point: usize::MAX,
        }
    }

    pub fn outlives(&self, other: &Lifetime) -> bool {
        self.start_point <= other.start_point && self.end_point >= other.end_point
    }
}

/// Borrow information for tracking active borrows
#[derive(Debug, Clone)]
pub struct BorrowInfo {
    pub origin: String, // Variable being borrowed
    pub mutable: bool,
    pub lifetime: Lifetime,
    pub location: usize, // Where borrow occurs
}

/// Symbol with ownership tracking
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub ty: Type,
    pub mutable: bool,
    pub borrow_state: BorrowState,
    pub lifetime: Option<Lifetime>,
    pub scope_id: usize,
    pub active_borrows: Vec<BorrowInfo>,
    pub drop_flag: bool, // Whether this needs to be dropped
}

/// Scope for tracking variables and their states
#[derive(Debug, Clone)]
pub struct Scope {
    pub id: usize,
    pub parent: Option<usize>,
    pub symbols: HashMap<String, Symbol>,
    pub borrows: Vec<(String, BorrowState)>,
    pub deferred_drops: Vec<String>, // Variables to drop at scope exit
    pub loop_context: Option<LoopContext>, // For break/continue
}

/// Loop context for break/continue analysis
#[derive(Debug, Clone)]
pub struct LoopContext {
    pub label: Option<String>,
    pub break_type: Option<Type>,
}

/// Trait definition for trait bound checking
#[derive(Debug, Clone)]
pub struct TraitInfo {
    pub name: String,
    pub methods: Vec<TraitMethod>,
    pub supertraits: Vec<String>,
    pub associated_types: HashMap<String, Option<Type>>,
}

/// Trait method signature
#[derive(Debug, Clone)]
pub struct TraitMethod {
    pub name: String,
    pub params: Vec<Type>,
    pub return_type: Type,
    pub has_default: bool,
}

/// Type constraint for generic type inference
#[derive(Debug, Clone)]
pub enum TypeConstraint {
    Equals(Type, Type),
    TraitBound(Type, String),
    Subtype(Type, Type),
}

/// Monomorphized function instance
#[derive(Debug, Clone)]
pub struct MonomorphizedFunc {
    pub original_name: String,
    pub type_args: Vec<Type>,
    pub mangled_name: String,
    pub typed_body: Vec<TypedStatement>,
}

/// Const value for compile-time evaluation
#[derive(Debug, Clone)]
pub enum ConstValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Null,
    Array(Vec<ConstValue>),
    Struct(String, Vec<(String, ConstValue)>),
}

#[derive(Debug, Clone)]
pub struct TypedModule {
    pub items: Vec<TypedItem>,
}

#[derive(Debug, Clone)]
pub enum TypedItem {
    Function(TypedFunction),
    Struct(TypedStruct),
    Enum(TypedEnum),
    Extern(TypedExternBlock),
    Trait(TraitInfo),
    Impl(TypedImpl),
    Static(TypedStatic),
}

#[derive(Debug, Clone)]
pub struct TypedStatic {
    pub name: String,
    pub mutable: bool,
    pub ty: Type,
    pub value: TypedExpr,
}

#[derive(Debug, Clone)]
pub struct TypedFunction {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
    pub body: Vec<TypedStatement>,
    pub is_async: bool,
}

#[derive(Debug, Clone)]
pub struct TypedStruct {
    pub name: String,
    pub fields: Vec<(String, Type)>,
    pub implements: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TypedEnum {
    pub name: String,
    pub variants: Vec<(String, Option<Vec<Type>>)>,
}

#[derive(Debug, Clone)]
pub struct TypedExternBlock {
    pub abi: String,
    pub functions: Vec<TypedFunction>,
}

#[derive(Debug, Clone)]
pub struct TypedImpl {
    pub trait_name: String,
    pub type_name: String,
    pub methods: Vec<TypedFunction>,
}

#[derive(Debug, Clone)]
pub enum TypedStatement {
    Let {
        name: String,
        ty: Type,
        value: TypedExpr,
    },
    Assignment {
        target: TypedExpr,
        op: Option<BinaryOp>,
        value: TypedExpr,
    },
    Return(Option<TypedExpr>),
    If {
        condition: TypedExpr,
        then_block: Vec<TypedStatement>,
        else_block: Option<Vec<TypedStatement>>,
    },
    For {
        var: String,
        iter: TypedExpr,
        body: Vec<TypedStatement>,
    },
    While {
        condition: TypedExpr,
        body: Vec<TypedStatement>,
    },
    Loop {
        body: Vec<TypedStatement>,
    },
    Match {
        expr: TypedExpr,
        arms: Vec<(Pattern, Vec<TypedStatement>)>,
    },
    Defer(Box<TypedStatement>),
    Break,
    Continue,
    Expr(TypedExpr),
    Pass,
    Yield(Option<TypedExpr>),
    Spawn(Box<TypedExpr>),
    Select {
        arms: Vec<TypedSelectArm>,
    },
}

#[derive(Debug, Clone)]
pub struct TypedExpr {
    pub kind: TypedExprKind,
    pub ty: Type,
}

/// Select arm for channel operations (typed)
#[derive(Debug, Clone)]
pub struct TypedSelectArm {
    pub pattern: Pattern,
    pub channel_op: TypedExpr,
    pub body: Vec<TypedStatement>,
}

impl TypedExpr {
    /// Determines if the expression is a valid lvalue (can appear on the left-hand side of an assignment).
    pub fn is_lvalue(&self) -> bool {
        match &self.kind {
            TypedExprKind::Identifier(_) => true,
            TypedExprKind::Field(_, _) => true,
            TypedExprKind::Index(_, _) => true,
            TypedExprKind::Deref(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TypedExprKind {
    Literal(Literal),
    Identifier(String),
    Binary(Box<TypedExpr>, BinaryOp, Box<TypedExpr>),
    Unary(UnaryOp, Box<TypedExpr>),
    Call(Box<TypedExpr>, Vec<TypedExpr>),
    MethodCall {
        receiver: Box<TypedExpr>,
        method: String,
        args: Vec<TypedExpr>,
    },
    Field(Box<TypedExpr>, String),
    Index(Box<TypedExpr>, Box<TypedExpr>),
    Borrow {
        mutable: bool,
        expr: Box<TypedExpr>,
    },
    Deref(Box<TypedExpr>),
    StructLiteral {
        name: String,
        fields: Vec<(String, TypedExpr)>,
    },
    Array(Vec<TypedExpr>),
    // New variants for full language support
    Range {
        start: Option<Box<TypedExpr>>,
        end: Option<Box<TypedExpr>>,
        inclusive: bool,
    },
    Lambda {
        params: Vec<(String, Type)>,
        body: Box<TypedExpr>,
        return_type: Type,
    },
    Tuple(Vec<TypedExpr>),
    Await(Box<TypedExpr>),
    None,
    Some(Box<TypedExpr>),
    Ok(Box<TypedExpr>),
    Err(Box<TypedExpr>),
    If {
        condition: Box<TypedExpr>,
        then_expr: Box<TypedExpr>,
        else_expr: Box<TypedExpr>,
    },
    Match {
        expr: Box<TypedExpr>,
        arms: Vec<(Pattern, TypedExpr)>,
    },
}

/// The main semantic analyzer
pub struct Analyzer {
    scopes: Vec<Scope>,
    current_scope: usize,
    next_scope_id: usize,
    traits: HashMap<String, TraitInfo>,
    type_impls: HashMap<String, Vec<String>>, // Type -> list of implemented traits
    generic_bounds: HashMap<String, Vec<String>>, // Generic -> trait bounds
    type_constraints: Vec<TypeConstraint>,    // Collected type constraints
    monomorphized: HashMap<String, MonomorphizedFunc>, // Monomorphized functions
    generic_functions: HashMap<String, Function>, // Original generic function definitions
    struct_layouts: HashMap<String, Vec<(String, Type)>>, // Struct field layouts
    enum_variants: HashMap<String, Vec<(String, Option<Vec<Type>>)>>, // Enum variants
    const_values: HashMap<String, ConstValue>, // Evaluated constants
    current_function: Option<String>,         // Current function being analyzed
    current_function_return: Option<Type>,    // Return type of current function
    loop_depth: usize,                        // Current loop nesting depth
    next_lifetime_id: usize,
    active_lifetimes: HashMap<String, Lifetime>,
    borrow_graph: Vec<BorrowInfo>, // All active borrows

    // Phase 3 Integration: New module instances
    /// Constraint solver for type inference
    constraint_solver: constraints::ConstraintSolver,
    /// Trait resolver for bound checking
    trait_resolver: traits::TraitResolver,
    /// Borrow checker for ownership enforcement
    borrow_checker: borrow_check::BorrowChecker,
}

impl Analyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            scopes: vec![Scope {
                id: 0,
                parent: None,
                symbols: HashMap::new(),
                borrows: Vec::new(),
                deferred_drops: Vec::new(),
                loop_context: None,
            }],
            current_scope: 0,
            next_scope_id: 1,
            traits: Self::init_builtin_traits(),
            type_impls: Self::init_primitive_impls(),
            generic_bounds: HashMap::new(),
            type_constraints: Vec::new(),
            monomorphized: HashMap::new(),
            generic_functions: HashMap::new(),
            struct_layouts: HashMap::new(),
            enum_variants: HashMap::new(),
            const_values: HashMap::new(),
            current_function: None,
            current_function_return: None,
            loop_depth: 0,
            next_lifetime_id: 0,
            active_lifetimes: HashMap::new(),
            borrow_graph: Vec::new(),
            // Phase 3 Integration: Initialize new module instances
            constraint_solver: constraints::ConstraintSolver::new(),
            trait_resolver: traits::TraitResolver::new(),
            borrow_checker: borrow_check::BorrowChecker::new(),
        };

        // Register builtin functions in the root scope
        analyzer.register_builtins();

        analyzer
    }

    /// Register all builtin functions so they are available without imports
    fn register_builtins(&mut self) {
        let builtins: Vec<(&str, Type, bool)> = vec![
            // I/O builtins — accept Any type (O-014)
            ("println", Type::Function(vec![Type::Any], None), false),
            ("print", Type::Function(vec![Type::Any], None), false),
            ("eprintln", Type::Function(vec![Type::Any], None), false),
            ("eprint", Type::Function(vec![Type::Any], None), false),
            // String formatting
            (
                "format",
                Type::Function(vec![Type::Any], Some(Box::new(Type::Str))),
                false,
            ),
            // Type introspection — use type_of to match VM (O-016)
            (
                "type_of",
                Type::Function(vec![Type::Any], Some(Box::new(Type::Str))),
                false,
            ),
            (
                "sizeof",
                Type::Function(vec![Type::Any], Some(Box::new(Type::I64))),
                false,
            ),
            // Missing builtins (O-002)
            (
                "len",
                Type::Function(vec![Type::Any], Some(Box::new(Type::I64))),
                false,
            ),
            (
                "int",
                Type::Function(vec![Type::Any], Some(Box::new(Type::I64))),
                false,
            ),
            (
                "float",
                Type::Function(vec![Type::Any], Some(Box::new(Type::F64))),
                false,
            ),
            (
                "sqrt",
                Type::Function(vec![Type::Any], Some(Box::new(Type::F64))),
                false,
            ),
            (
                "abs",
                Type::Function(vec![Type::Any], Some(Box::new(Type::I64))),
                false,
            ),
            // Process/runtime
            (
                "args",
                Type::Function(
                    vec![],
                    Some(Box::new(Type::Array(Box::new(Type::Str), None))),
                ),
                false,
            ),
            ("exit", Type::Function(vec![Type::I32], None), false),
            // Assertions — accept Any type, VM checks truthiness (O-033)
            ("assert", Type::Function(vec![Type::Any], None), false),
            (
                "assert_eq",
                Type::Function(vec![Type::Any, Type::Any], None),
                false,
            ),
            // Debug
            (
                "dbg",
                Type::Function(vec![Type::Any], Some(Box::new(Type::Any))),
                false,
            ),
            // Bootstrap string/file helpers used by compiler_minimal.omni
            (
                "str_eq",
                Type::Function(vec![Type::Str, Type::Str], Some(Box::new(Type::Bool))),
                false,
            ),
            (
                "str_len",
                Type::Function(vec![Type::Str], Some(Box::new(Type::I64))),
                false,
            ),
            (
                "str_char_at",
                Type::Function(vec![Type::Str, Type::I64], Some(Box::new(Type::I64))),
                false,
            ),
            (
                "str_append",
                Type::Function(vec![Type::Str, Type::I64], Some(Box::new(Type::Str))),
                false,
            ),
            (
                "str_slice",
                Type::Function(vec![Type::Str, Type::I64, Type::I64], Some(Box::new(Type::Str))),
                false,
            ),
            (
                "file_read",
                Type::Function(vec![Type::Str], Some(Box::new(Type::Str))),
                false,
            ),
            (
                "file_write_bytes",
                Type::Function(vec![Type::Str, Type::I64], None),
                false,
            ),
            (
                "str_replace_ext",
                Type::Function(vec![Type::Str, Type::Str], Some(Box::new(Type::Str))),
                false,
            ),
            (
                "arg",
                Type::Function(vec![Type::I64], Some(Box::new(Type::Str))),
                false,
            ),
        ];

        for (name, ty, mutable) in builtins {
            // Insert directly into root scope, bypassing define_symbol to avoid
            // duplicate-definition errors and lifetime tracking for builtins
            self.scopes[0].symbols.insert(
                name.to_string(),
                Symbol {
                    name: name.to_string(),
                    ty,
                    mutable,
                    borrow_state: BorrowState::Owned,
                    lifetime: Some(Lifetime::static_lifetime()),
                    scope_id: 0,
                    active_borrows: Vec::new(),
                    drop_flag: false,
                },
            );
        }

        debug!(
            "Registered {} builtin functions",
            self.scopes[0].symbols.len()
        );
    }

    /// Initialize builtin traits like Copy, Clone, Send, Sync
    fn init_builtin_traits() -> HashMap<String, TraitInfo> {
        let mut traits = HashMap::new();

        traits.insert(
            "Copy".to_string(),
            TraitInfo {
                name: "Copy".to_string(),
                methods: vec![],
                supertraits: vec!["Clone".to_string()],
                associated_types: HashMap::new(),
            },
        );

        traits.insert(
            "Clone".to_string(),
            TraitInfo {
                name: "Clone".to_string(),
                methods: vec![TraitMethod {
                    name: "clone".to_string(),
                    params: vec![Type::SelfRef { mutable: false }],
                    return_type: Type::SelfOwned,
                    has_default: false,
                }],
                supertraits: vec![],
                associated_types: HashMap::new(),
            },
        );

        traits.insert(
            "Send".to_string(),
            TraitInfo {
                name: "Send".to_string(),
                methods: vec![],
                supertraits: vec![],
                associated_types: HashMap::new(),
            },
        );

        traits.insert(
            "Sync".to_string(),
            TraitInfo {
                name: "Sync".to_string(),
                methods: vec![],
                supertraits: vec![],
                associated_types: HashMap::new(),
            },
        );

        traits.insert(
            "Drop".to_string(),
            TraitInfo {
                name: "Drop".to_string(),
                methods: vec![TraitMethod {
                    name: "drop".to_string(),
                    params: vec![Type::SelfRef { mutable: true }],
                    return_type: Type::Named("()".to_string()),
                    has_default: false,
                }],
                supertraits: vec![],
                associated_types: HashMap::new(),
            },
        );

        traits.insert(
            "Debug".to_string(),
            TraitInfo {
                name: "Debug".to_string(),
                methods: vec![TraitMethod {
                    name: "fmt".to_string(),
                    params: vec![Type::SelfRef { mutable: false }],
                    return_type: Type::Named("String".to_string()),
                    has_default: false,
                }],
                supertraits: vec![],
                associated_types: HashMap::new(),
            },
        );

        traits.insert(
            "Default".to_string(),
            TraitInfo {
                name: "Default".to_string(),
                methods: vec![TraitMethod {
                    name: "default".to_string(),
                    params: vec![],
                    return_type: Type::SelfOwned,
                    has_default: false,
                }],
                supertraits: vec![],
                associated_types: HashMap::new(),
            },
        );

        traits.insert(
            "Iterator".to_string(),
            TraitInfo {
                name: "Iterator".to_string(),
                methods: vec![TraitMethod {
                    name: "next".to_string(),
                    params: vec![Type::SelfRef { mutable: true }],
                    return_type: Type::Generic(
                        "Option".to_string(),
                        vec![Type::Named("Self::Item".to_string())],
                    ),
                    has_default: false,
                }],
                supertraits: vec![],
                associated_types: {
                    let mut at = HashMap::new();
                    at.insert("Item".to_string(), None);
                    at
                },
            },
        );

        traits
    }

    /// Initialize primitive type trait implementations
    fn init_primitive_impls() -> HashMap<String, Vec<String>> {
        let mut impls = HashMap::new();

        let copy_types = vec![
            "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char",
            "usize", "isize",
        ];
        let basic_traits = vec!["Copy", "Clone", "Send", "Sync", "Debug", "Default"];

        for ty in copy_types {
            impls.insert(
                ty.to_string(),
                basic_traits.iter().map(|s| s.to_string()).collect(),
            );
        }

        impls
    }

    fn fresh_lifetime(&mut self) -> Lifetime {
        let id = self.next_lifetime_id;
        self.next_lifetime_id += 1;
        Lifetime {
            name: format!("'_{}", id),
            scope_id: self.current_scope,
            start_point: self.current_scope,
            end_point: self.current_scope,
        }
    }

    fn push_scope(&mut self) {
        let new_scope = Scope {
            id: self.next_scope_id,
            parent: Some(self.current_scope),
            symbols: HashMap::new(),
            borrows: Vec::new(),
            deferred_drops: Vec::new(),
            loop_context: None,
        };
        self.next_scope_id += 1;
        self.scopes.push(new_scope);
        self.current_scope = self.scopes.len() - 1;
    }

    fn pop_scope(&mut self) -> Result<(), SemanticError> {
        // Release all borrows from this scope
        self.release_scope_borrows()?;

        // Check for dangling borrows before popping
        let scope = &self.scopes[self.current_scope];
        for (name, state) in &scope.borrows {
            if *state != BorrowState::Owned {
                debug!("Dropping borrow for {} at scope end", name);
            }
        }

        // Execute deferred drops in reverse order
        let drops = scope.deferred_drops.clone();
        for name in drops.iter().rev() {
            debug!("Executing deferred drop for {}", name);
            // Would call drop() if implemented
        }

        if let Some(parent) = self.scopes[self.current_scope].parent {
            self.current_scope = parent;
        }
        Ok(())
    }

    /// Release all borrows that originated in the current scope
    fn release_scope_borrows(&mut self) -> Result<(), SemanticError> {
        let scope_id = self.current_scope;
        self.borrow_graph
            .retain(|b| b.lifetime.scope_id != scope_id);

        // Update symbol borrow states
        for scope in &mut self.scopes {
            for (_, sym) in &mut scope.symbols {
                sym.active_borrows
                    .retain(|b| b.lifetime.scope_id != scope_id);

                // Reset borrow state if all borrows released
                if sym.active_borrows.is_empty() {
                    if sym.borrow_state != BorrowState::Moved {
                        sym.borrow_state = BorrowState::Owned;
                    }
                }
            }
        }

        Ok(())
    }

    fn define_symbol(
        &mut self,
        name: String,
        ty: Type,
        mutable: bool,
    ) -> Result<(), SemanticError> {
        // Compute values that need &self before taking &mut scope
        let needs_drop = self.type_needs_drop(&ty);
        let lifetime = self.fresh_lifetime();
        let scope_id = self.current_scope;

        let scope = &mut self.scopes[self.current_scope];
        if scope.symbols.contains_key(&name) {
            return Err(SemanticError::DuplicateDefinition { name });
        }

        scope.symbols.insert(
            name.clone(),
            Symbol {
                name: name.clone(),
                ty,
                mutable,
                borrow_state: BorrowState::Owned,
                lifetime: Some(lifetime),
                scope_id,
                active_borrows: Vec::new(),
                drop_flag: needs_drop,
            },
        );

        if needs_drop {
            scope.deferred_drops.push(name);
        }

        Ok(())
    }

    fn define_binding_symbols(
        &mut self,
        binding: &str,
        ty: Type,
        mutable: bool,
    ) -> Result<(), SemanticError> {
        let trimmed = binding.trim();
        if trimmed.starts_with('(') && trimmed.ends_with(')') {
            let inner = &trimmed[1..trimmed.len() - 1];
            for part in inner.split(',') {
                let name = part.trim();
                if name.is_empty() || name == "_" {
                    continue;
                }
                self.define_symbol(name.to_string(), ty.clone(), mutable)?;
            }
            Ok(())
        } else {
            self.define_symbol(trimmed.to_string(), ty, mutable)
        }
    }

    /// Check if a type requires drop (not Copy)
    fn type_needs_drop(&self, ty: &Type) -> bool {
        match ty {
            Type::I8
            | Type::I16
            | Type::I32
            | Type::I64
            | Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::F32
            | Type::F64
            | Type::Bool
            | Type::Usize
            | Type::Isize => false,
            Type::Named(n) => {
                // Check if type implements Copy
                if let Some(impls) = self.type_impls.get(n) {
                    !impls.contains(&"Copy".to_string())
                } else {
                    true // Unknown types need drop by default
                }
            }
            Type::Generic(_, _) => true,
            Type::Array(inner, _) => self.type_needs_drop(inner),
            Type::WithOwnership(_inner, Ownership::Borrow | Ownership::BorrowMut) => false,
            Type::WithOwnership(inner, _) => self.type_needs_drop(inner),
            _ => true,
        }
    }

    fn lookup_symbol(&self, name: &str) -> Result<&Symbol, SemanticError> {
        let mut scope_idx = self.current_scope;
        loop {
            if let Some(sym) = self.scopes[scope_idx].symbols.get(name) {
                return Ok(sym);
            }
            if let Some(parent) = self.scopes[scope_idx].parent {
                scope_idx = parent;
            } else {
                break;
            }
        }
        Err(SemanticError::UndefinedSymbol {
            name: name.to_string(),
        })
    }

    fn lookup_symbol_mut(&mut self, name: &str) -> Result<&mut Symbol, SemanticError> {
        let mut scope_idx = self.current_scope;
        loop {
            if self.scopes[scope_idx].symbols.contains_key(name) {
                return Ok(self.scopes[scope_idx].symbols.get_mut(name).unwrap());
            }
            if let Some(parent) = self.scopes[scope_idx].parent {
                scope_idx = parent;
            } else {
                break;
            }
        }
        Err(SemanticError::UndefinedSymbol {
            name: name.to_string(),
        })
    }

    /// Check if a value can be borrowed
    fn check_borrow(&self, name: &str, mutable: bool) -> Result<(), SemanticError> {
        let sym = self.lookup_symbol(name)?;

        match &sym.borrow_state {
            BorrowState::Moved => {
                return Err(SemanticError::MoveError {
                    name: name.to_string(),
                });
            }
            BorrowState::BorrowedMut => {
                // Phase-5 bootstrap relaxation: allow reborrows in linear control flow.
                return Ok(());
            }
            BorrowState::BorrowedShared(_) if mutable => {
                // Phase-5 bootstrap relaxation: permit upgrading shared borrow
                // to mutable borrow in this reduced checker mode.
                return Ok(());
            }
            _ => {}
        }

        if mutable && !sym.mutable {
            return Err(SemanticError::MutabilityError(format!(
                "Cannot mutably borrow immutable variable '{}'",
                name
            )));
        }

        Ok(())
    }

    /// Record a borrow
    fn record_borrow(&mut self, name: &str, mutable: bool) -> Result<Lifetime, SemanticError> {
        self.check_borrow(name, mutable)?;

        let lifetime = self.fresh_lifetime();
        let borrow_info = BorrowInfo {
            origin: name.to_string(),
            mutable,
            lifetime: lifetime.clone(),
            location: self.current_scope,
        };

        self.borrow_graph.push(borrow_info.clone());

        let sym = self.lookup_symbol_mut(name)?;
        sym.active_borrows.push(borrow_info);
        sym.borrow_state = if mutable {
            BorrowState::BorrowedMut
        } else {
            match &sym.borrow_state {
                BorrowState::BorrowedShared(n) => BorrowState::BorrowedShared(n + 1),
                _ => BorrowState::BorrowedShared(1),
            }
        };

        Ok(lifetime)
    }

    /// Move a value (transfer ownership)
    fn move_value(&mut self, name: &str) -> Result<(), SemanticError> {
        let sym = self.lookup_symbol(name)?;

        // Check if value can be moved
        if sym.borrow_state == BorrowState::Moved {
            return Err(SemanticError::MoveError {
                name: name.to_string(),
            });
        }

        if !sym.active_borrows.is_empty() {
            return Err(SemanticError::BorrowError(format!(
                "Cannot move '{}' while it is borrowed",
                name
            )));
        }

        // Check if type is Copy
        if self.type_implements_copy(&sym.ty) {
            return Ok(()); // Copy types don't move
        }

        // Mark as moved
        let sym = self.lookup_symbol_mut(name)?;
        sym.borrow_state = BorrowState::Moved;

        Ok(())
    }

    /// Check if a type implements Copy
    fn type_implements_copy(&self, ty: &Type) -> bool {
        match ty {
            Type::I8
            | Type::I16
            | Type::I32
            | Type::I64
            | Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::F32
            | Type::F64
            | Type::Bool
            | Type::Usize
            | Type::Isize => true,
            Type::Named(n) => {
                if let Some(impls) = self.type_impls.get(n) {
                    impls.contains(&"Copy".to_string())
                } else {
                    false
                }
            }
            Type::WithOwnership(_, Ownership::Borrow | Ownership::BorrowMut) => true,
            Type::Array(inner, Some(_)) => self.type_implements_copy(inner),
            _ => false,
        }
    }

    /// Check if a trait bound is satisfied
    fn check_trait_bound(&self, ty: &Type, trait_name: &str) -> Result<(), SemanticError> {
        let type_name = match ty {
            Type::Named(n) => n.clone(),
            Type::Generic(n, _) => n.clone(),
            _ => self.type_to_string(ty),
        };

        // Check explicit implementations
        if let Some(impls) = self.type_impls.get(&type_name) {
            if impls.contains(&trait_name.to_string()) {
                return Ok(());
            }
        }

        // Check primitive types for common traits
        match ty {
            Type::I8
            | Type::I16
            | Type::I32
            | Type::I64
            | Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::F32
            | Type::F64
            | Type::Bool
            | Type::Usize
            | Type::Isize => {
                let builtin_traits = [
                    "Copy",
                    "Clone",
                    "Send",
                    "Sync",
                    "Debug",
                    "Default",
                    "PartialEq",
                    "Eq",
                    "PartialOrd",
                    "Ord",
                    "Hash",
                ];
                if builtin_traits.contains(&trait_name) {
                    return Ok(());
                }
            }
            Type::Str => {
                let str_traits = [
                    "Clone",
                    "Debug",
                    "PartialEq",
                    "Eq",
                    "PartialOrd",
                    "Ord",
                    "Hash",
                ];
                if str_traits.contains(&trait_name) {
                    return Ok(());
                }
            }
            Type::WithOwnership(inner, Ownership::Borrow | Ownership::BorrowMut) => {
                // References are Send/Sync if their referent is
                if trait_name == "Send" || trait_name == "Sync" {
                    return self.check_trait_bound(inner, trait_name);
                }
            }
            _ => {}
        }

        // Check supertraits
        if let Some(trait_info) = self.traits.get(trait_name) {
            for supertrait in &trait_info.supertraits {
                if self.check_trait_bound(ty, supertrait).is_err() {
                    return Err(SemanticError::TraitBoundError(format!(
                        "Type '{}' does not implement supertrait '{}' required by '{}'",
                        type_name, supertrait, trait_name
                    )));
                }
            }
        }

        Err(SemanticError::TraitBoundError(format!(
            "Type '{}' does not implement trait '{}'",
            type_name, trait_name
        )))
    }

    /// Convert type to string for error messages
    fn type_to_string(&self, ty: &Type) -> String {
        match ty {
            Type::I8 => "i8".to_string(),
            Type::I16 => "i16".to_string(),
            Type::I32 => "i32".to_string(),
            Type::I64 => "i64".to_string(),
            Type::U8 => "u8".to_string(),
            Type::U16 => "u16".to_string(),
            Type::U32 => "u32".to_string(),
            Type::U64 => "u64".to_string(),
            Type::F32 => "f32".to_string(),
            Type::F64 => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Str => "str".to_string(),
            Type::Named(n) => n.clone(),
            Type::Generic(n, args) => {
                let args_str: Vec<_> = args.iter().map(|a| self.type_to_string(a)).collect();
                format!("{}<{}>", n, args_str.join(", "))
            }
            Type::Array(inner, size) => {
                if let Some(_) = size {
                    format!("[{}; N]", self.type_to_string(inner))
                } else {
                    format!("[{}]", self.type_to_string(inner))
                }
            }
            Type::WithOwnership(inner, own) => {
                let prefix = match own {
                    Ownership::Borrow => "&",
                    Ownership::BorrowMut => "&mut ",
                    Ownership::Owned => "own ",
                    Ownership::Shared => "shared ",
                    Ownership::RawPointer => "*",
                };
                format!("{}{}", prefix, self.type_to_string(inner))
            }
            _ => "unknown".to_string(),
        }
    }

    /// Resolve trait method for a type
    fn resolve_trait_method(&self, ty: &Type, method: &str) -> Option<(String, TraitMethod)> {
        let type_name = match ty {
            Type::Named(n) => n.clone(),
            _ => return None,
        };

        // Find which trait provides this method
        if let Some(impls) = self.type_impls.get(&type_name) {
            for trait_name in impls {
                if let Some(trait_info) = self.traits.get(trait_name) {
                    for m in &trait_info.methods {
                        if m.name == method {
                            return Some((trait_name.clone(), m.clone()));
                        }
                    }
                }
            }
        }

        None
    }

    /// Resolve a method's return type from impl blocks
    fn resolve_method_return_type(&self, receiver_ty: &Type, method: &str) -> Type {
        let type_name = match receiver_ty {
            Type::Named(n) => n.clone(),
            Type::WithOwnership(inner, _) => {
                if let Type::Named(n) = inner.as_ref() {
                    n.clone()
                } else {
                    return Type::Named("_".into());
                }
            }
            _ => return Type::Named("_".into()),
        };
        // Look up method in registered impl methods (via function_sigs or trait methods)
        if let Some(impls) = self.type_impls.get(&type_name) {
            for trait_name in impls {
                if let Some(trait_info) = self.traits.get(trait_name) {
                    for m in &trait_info.methods {
                        if m.name == method {
                            return m.return_type.clone();
                        }
                    }
                }
            }
        }
        // Check struct-defined methods via generic_functions
        if let Some(func) = self
            .generic_functions
            .get(&format!("{}.{}", type_name, method))
        {
            return func.return_type.clone().unwrap_or(Type::Named("()".into()));
        }
        Type::Named("_".into())
    }

    /// Resolve a field's type on a struct, or error if field doesn't exist
    fn resolve_field_type(&self, obj_ty: &Type, field: &str) -> Result<Type, SemanticError> {
        let type_name = match obj_ty {
            Type::Named(n) => n.clone(),
            Type::WithOwnership(inner, _) => {
                if let Type::Named(n) = inner.as_ref() {
                    n.clone()
                } else {
                    return Ok(Type::Named("_".into()));
                }
            }
            _ => return Ok(Type::Named("_".into())),
        };
        if let Some(fields) = self.struct_layouts.get(&type_name) {
            for (fname, ftype) in fields {
                if fname == field {
                    return Ok(ftype.clone());
                }
            }
            return Err(SemanticError::UndefinedSymbol {
                name: format!("{}.{}", type_name, field),
            });
        }
        Ok(Type::Named("_".into()))
    }

    /// Check if a type is an integer type
    fn is_integer_type(&self, ty: &Type) -> bool {
        matches!(
            ty,
            Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::Usize
                | Type::Isize
        )
    }

    /// Evaluate a constant expression at compile time
    fn eval_const(&self, expr: &Expression) -> Result<ConstValue, SemanticError> {
        match expr {
            Expression::Literal(lit) => Ok(match lit {
                Literal::Int(i) => ConstValue::Int(*i),
                Literal::Float(f) => ConstValue::Float(*f),
                Literal::Bool(b) => ConstValue::Bool(*b),
                Literal::String(s) => ConstValue::String(s.clone()),
                Literal::Null => ConstValue::Null,
            }),
            Expression::Identifier(name) => {
                if let Some(val) = self.const_values.get(name) {
                    Ok(val.clone())
                } else {
                    Err(SemanticError::UndefinedSymbol { name: name.clone() })
                }
            }
            Expression::Binary(left, op, right) => {
                let l = self.eval_const(left)?;
                let r = self.eval_const(right)?;
                self.eval_const_binary(op, l, r)
            }
            Expression::Unary(op, inner) => {
                let val = self.eval_const(inner)?;
                self.eval_const_unary(op, val)
            }
            _ => Err(SemanticError::InvalidSyntax {
                line: 0,
                message: "Expression is not const-evaluable".to_string(),
            }),
        }
    }

    fn eval_const_binary(
        &self,
        op: &BinaryOp,
        left: ConstValue,
        right: ConstValue,
    ) -> Result<ConstValue, SemanticError> {
        match (op, left, right) {
            (BinaryOp::Add, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Int(a + b)),
            (BinaryOp::Sub, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Int(a - b)),
            (BinaryOp::Mul, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Int(a * b)),
            (BinaryOp::Div, ConstValue::Int(a), ConstValue::Int(b)) if b != 0 => {
                Ok(ConstValue::Int(a / b))
            }
            (BinaryOp::Mod, ConstValue::Int(a), ConstValue::Int(b)) if b != 0 => {
                Ok(ConstValue::Int(a % b))
            }
            (BinaryOp::Eq, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Bool(a == b)),
            (BinaryOp::NotEq, ConstValue::Int(a), ConstValue::Int(b)) => {
                Ok(ConstValue::Bool(a != b))
            }
            (BinaryOp::Lt, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Bool(a < b)),
            (BinaryOp::Gt, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Bool(a > b)),
            (BinaryOp::Add, ConstValue::Float(a), ConstValue::Float(b)) => {
                Ok(ConstValue::Float(a + b))
            }
            (BinaryOp::And, ConstValue::Bool(a), ConstValue::Bool(b)) => {
                Ok(ConstValue::Bool(a && b))
            }
            (BinaryOp::Or, ConstValue::Bool(a), ConstValue::Bool(b)) => {
                Ok(ConstValue::Bool(a || b))
            }
            _ => Err(SemanticError::TypeMismatch {
                expected: "compatible const types".to_string(),
                got: "incompatible types".to_string(),
            }),
        }
    }

    fn eval_const_unary(&self, op: &UnaryOp, val: ConstValue) -> Result<ConstValue, SemanticError> {
        match (op, val) {
            (UnaryOp::Neg, ConstValue::Int(i)) => Ok(ConstValue::Int(-i)),
            (UnaryOp::Neg, ConstValue::Float(f)) => Ok(ConstValue::Float(-f)),
            (UnaryOp::Not, ConstValue::Bool(b)) => Ok(ConstValue::Bool(!b)),
            _ => Err(SemanticError::TypeMismatch {
                expected: "compatible type for unary op".to_string(),
                got: "incompatible type".to_string(),
            }),
        }
    }

    /// Monomorphize a generic function with concrete type arguments
    fn monomorphize(
        &mut self,
        func_name: &str,
        type_args: &[Type],
    ) -> Result<String, SemanticError> {
        let mangled_name = self.mangle_name(func_name, type_args);

        if self.monomorphized.contains_key(&mangled_name) {
            return Ok(mangled_name);
        }

        debug!("Monomorphizing {} with {:?}", func_name, type_args);

        // Try to retrieve the generic function definition and monomorphize it
        if let Some(generic_func) = self.generic_functions.get(func_name).cloned() {
            use crate::semantic::monomorphization::{StatementSubstitutor, TypeSubstitution};

            let mut substitution = TypeSubstitution::new();

            // Map type arguments (T, U, V, etc.) to concrete types
            let type_param_names = vec!["T", "U", "V", "W", "X"];
            for (i, type_arg) in type_args.iter().enumerate() {
                if i < type_param_names.len() {
                    substitution.add(type_param_names[i].to_string(), type_arg.clone());
                }
            }

            // Transform function body by substituting type parameters
            let stmt_sub = StatementSubstitutor::new(substitution.clone());
            let mut typed_body = Vec::new();

            // Transform statements from the original function body
            for stmt in &generic_func.body.statements {
                let transformed = stmt_sub.transform_statement(stmt);
                // Convert to typed statement via the semantic analyzer
                match self.statement_to_typed(&transformed) {
                    Ok(typed_stmt) => typed_body.push(typed_stmt),
                    Err(e) => {
                        debug!("Monomorphization: statement_to_typed failed: {:?}", e);
                    }
                }
            }

            self.monomorphized.insert(
                mangled_name.clone(),
                MonomorphizedFunc {
                    original_name: func_name.to_string(),
                    type_args: type_args.to_vec(),
                    mangled_name: mangled_name.clone(),
                    typed_body,
                },
            );
        } else {
            // Function not found, create placeholder
            self.monomorphized.insert(
                mangled_name.clone(),
                MonomorphizedFunc {
                    original_name: func_name.to_string(),
                    type_args: type_args.to_vec(),
                    mangled_name: mangled_name.clone(),
                    typed_body: Vec::new(),
                },
            );
        }

        Ok(mangled_name)
    }

    fn mangle_name(&self, name: &str, type_args: &[Type]) -> String {
        if type_args.is_empty() {
            name.to_string()
        } else {
            let args: Vec<_> = type_args.iter().map(|t| self.type_to_string(t)).collect();
            format!("{}$${}", name, args.join("$"))
        }
    }

    /// Unify two types, adding constraints if generic
    fn unify(&mut self, expected: &Type, got: &Type) -> Result<Type, SemanticError> {
        match (expected, got) {
            // Identical types
            (a, b) if types_equal(a, b) => Ok(a.clone()),

            // Generic type placeholder unifies with anything
            (Type::Named(n), other) if n == "_" => Ok(other.clone()),
            (other, Type::Named(n)) if n == "_" => Ok(other.clone()),

            // Generic types create constraints
            (Type::Generic(name, args1), Type::Generic(name2, args2)) if name == name2 => {
                if args1.len() != args2.len() {
                    return Err(SemanticError::TypeMismatch {
                        expected: self.type_to_string(expected),
                        got: self.type_to_string(got),
                    });
                }
                let unified_args: Result<Vec<_>, _> = args1
                    .iter()
                    .zip(args2.iter())
                    .map(|(a, b)| self.unify(a, b))
                    .collect();
                Ok(Type::Generic(name.clone(), unified_args?))
            }

            // Arrays
            (Type::Array(elem1, _), Type::Array(elem2, _)) => {
                let unified_elem = self.unify(elem1, elem2)?;
                Ok(Type::Array(Box::new(unified_elem), None))
            }

            // Slices
            (Type::Slice(elem1), Type::Slice(elem2)) => {
                let unified_elem = self.unify(elem1, elem2)?;
                Ok(Type::Slice(Box::new(unified_elem)))
            }

            // References - covariant
            (
                Type::WithOwnership(inner1, Ownership::Borrow),
                Type::WithOwnership(inner2, Ownership::Borrow),
            ) => {
                let unified = self.unify(inner1, inner2)?;
                Ok(Type::WithOwnership(Box::new(unified), Ownership::Borrow))
            }

            // Numeric coercions
            (Type::I32, Type::I64) | (Type::I64, Type::I32) => Ok(Type::I64),
            (Type::F32, Type::F64) | (Type::F64, Type::F32) => Ok(Type::F64),

            _ => Err(SemanticError::TypeMismatch {
                expected: self.type_to_string(expected),
                got: self.type_to_string(got),
            }),
        }
    }

    pub fn analyze(&mut self, module: Module) -> Result<TypedModule, SemanticError> {
        info!("Starting semantic analysis with full borrow checking and lifetime inference");
        let mut items = Vec::new();

        // Forward declaration pass: register all top-level function signatures
        // so functions can be called before their definition (like C prototypes)
        for item in &module.items {
            if let Item::Function(f) = item {
                let param_types: Vec<Type> = f.params.iter().map(|p| p.ty.clone()).collect();
                let ret_type = f.return_type.clone().map(Box::new);
                let fn_type = Type::Function(param_types, ret_type);

                // Only register if not already defined (e.g., as a builtin)
                if self.lookup_symbol(&f.name).is_err() {
                    // Insert directly into root scope for forward visibility
                    self.scopes[0].symbols.insert(
                        f.name.clone(),
                        Symbol {
                            name: f.name.clone(),
                            ty: fn_type,
                            mutable: false,
                            borrow_state: BorrowState::Owned,
                            lifetime: Some(Lifetime::static_lifetime()),
                            scope_id: 0,
                            active_borrows: Vec::new(),
                            drop_flag: false,
                        },
                    );
                    debug!("Forward-declared function: {}", f.name);
                }
            }
        }

        // First pass: collect all type definitions (structs, enums, traits)
        for item in &module.items {
            match item {
                Item::Struct(s) => {
                    let fields: Vec<_> = s
                        .fields
                        .iter()
                        .map(|f| (f.name.clone(), f.ty.clone()))
                        .collect();
                    self.struct_layouts.insert(s.name.clone(), fields);
                }
                Item::Enum(e) => {
                    let variants: Vec<_> = e
                        .variants
                        .iter()
                        .map(|v| {
                            let types = match &v.fields {
                                Some(EnumFields::Tuple(ts)) => Some(ts.clone()),
                                _ => None,
                            };
                            (v.name.clone(), types)
                        })
                        .collect();
                    self.enum_variants.insert(e.name.clone(), variants);
                }
                Item::Trait(t) => {
                    let methods: Vec<_> = t
                        .methods
                        .iter()
                        .map(|m| TraitMethod {
                            name: m.name.clone(),
                            params: m.params.iter().map(|p| p.ty.clone()).collect(),
                            return_type: m
                                .return_type
                                .clone()
                                .unwrap_or(Type::Named("()".to_string())),
                            has_default: !m.body.statements.is_empty(),
                        })
                        .collect();
                    self.traits.insert(
                        t.name.clone(),
                        TraitInfo {
                            name: t.name.clone(),
                            methods,
                            supertraits: Vec::new(),
                            associated_types: HashMap::new(),
                        },
                    );
                }
                Item::Impl(i) => {
                    self.type_impls
                        .entry(i.type_name.clone())
                        .or_insert_with(Vec::new)
                        .push(i.trait_name.clone());
                }
                Item::Const(c) => {
                    // Make const symbols available across the module.
                    let _ = self.define_symbol(c.name.clone(), c.ty.clone(), false);
                    // Evaluate const at compile time
                    if let Ok(val) = self.eval_const(&c.value) {
                        self.const_values.insert(c.name.clone(), val);
                    }
                }
                Item::Static(s) => {
                    // Define static variables in root scope to allow access throughout module
                    let ty = s.ty.clone();
                    let _ = self.define_symbol(s.name.clone(), ty.clone(), s.mutable);
                }
                _ => {}
            }
        }

        // Second pass: analyze all items with full context
        for item in module.items {
            debug!("Analyzing item");
            match item {
                Item::Function(f) => items.push(self.analyze_function(f)?),
                Item::Struct(s) => items.push(self.analyze_struct(s)?),
                Item::Enum(e) => items.push(self.analyze_enum(e)?),
                Item::Extern(e) => items.push(self.analyze_extern(e)?),
                Item::Trait(t) => items.push(TypedItem::Trait(TraitInfo {
                    name: t.name.clone(),
                    methods: t
                        .methods
                        .iter()
                        .map(|m| TraitMethod {
                            name: m.name.clone(),
                            params: m.params.iter().map(|p| p.ty.clone()).collect(),
                            return_type: m
                                .return_type
                                .clone()
                                .unwrap_or(Type::Named("()".to_string())),
                            has_default: !m.body.statements.is_empty(),
                        })
                        .collect(),
                    supertraits: Vec::new(),
                    associated_types: HashMap::new(),
                })),
                Item::Impl(i) => items.push(self.analyze_impl(i)?),
                Item::Static(s) => items.push(self.analyze_static(s)?),
                Item::Comptime(block) => {
                    info!("Evaluating comptime block");
                    // Execute comptime statements
                    for stmt in &block.statements {
                        if let Statement::Let {
                            name,
                            value: Some(value),
                            ..
                        } = stmt
                        {
                            if let Ok(val) = self.eval_const(value) {
                                self.const_values.insert(name.clone(), val);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Resolve all type constraints
        self.solve_constraints()?;

        info!(
            "Semantic analysis complete - {} items analyzed",
            items.len()
        );
        Ok(TypedModule { items })
    }

    /// Solve collected type constraints
    fn solve_constraints(&mut self) -> Result<(), SemanticError> {
        for constraint in self.type_constraints.drain(..).collect::<Vec<_>>() {
            match constraint {
                TypeConstraint::Equals(t1, t2) => {
                    self.unify(&t1, &t2)?;
                }
                TypeConstraint::TraitBound(ty, trait_name) => {
                    self.check_trait_bound(&ty, &trait_name)?;
                }
                TypeConstraint::Subtype(sub, super_ty) => {
                    // Check subtyping relationship
                    self.unify(&sub, &super_ty)?;
                }
            }
        }
        Ok(())
    }

    fn analyze_enum(&mut self, e: EnumDef) -> Result<TypedItem, SemanticError> {
        debug!("Analyzing enum: {}", e.name);
        let variants: Vec<_> = e
            .variants
            .iter()
            .map(|v| {
                let fields = match &v.fields {
                    Some(EnumFields::Tuple(types)) => Some(types.clone()),
                    _ => None,
                };
                (v.name.clone(), fields)
            })
            .collect();

        Ok(TypedItem::Enum(TypedEnum {
            name: e.name,
            variants,
        }))
    }

    fn analyze_impl(&mut self, i: ImplBlock) -> Result<TypedItem, SemanticError> {
        debug!("Analyzing impl {} for {}", i.trait_name, i.type_name);

        let mut methods = Vec::new();
        for m in i.methods {
            if let TypedItem::Function(f) = self.analyze_function(m)? {
                methods.push(f);
            }
        }

        Ok(TypedItem::Impl(TypedImpl {
            trait_name: i.trait_name,
            type_name: i.type_name,
            methods,
        }))
    }

    fn analyze_static(&mut self, s: StaticDecl) -> Result<TypedItem, SemanticError> {
        debug!("Analyzing static variable: {}", s.name);
        let ty = s.ty.clone();
        let typed_value = self.analyze_expression(&s.value)?;

        // Ensure the initializer is assignable to the static type
        self.unify(&ty, &typed_value.ty)?;

        Ok(TypedItem::Static(TypedStatic {
            name: s.name,
            mutable: s.mutable,
            ty,
            value: typed_value,
        }))
    }

    fn analyze_extern(&mut self, e: ExternBlock) -> Result<TypedItem, SemanticError> {
        let mut functions = Vec::new();
        for f in e.functions {
            let params: Vec<_> = f
                .params
                .iter()
                .map(|p| (p.name.clone(), p.ty.clone()))
                .collect();

            let return_type = f.return_type.unwrap_or(Type::Named("()".into()));

            functions.push(TypedFunction {
                name: f.name,
                params,
                return_type,
                body: Vec::new(),
                is_async: f.is_async,
            });
        }
        Ok(TypedItem::Extern(TypedExternBlock {
            abi: e.abi,
            functions,
        }))
    }

    fn analyze_function(&mut self, f: Function) -> Result<TypedItem, SemanticError> {
        debug!("Analyzing function: {}", f.name);

        self.current_function = Some(f.name.clone());
        self.push_scope();

        // Add parameters to scope with proper ownership tracking
        for param in &f.params {
            let mutable = matches!(
                &param.ty,
                Type::WithOwnership(_, Ownership::BorrowMut) | Type::SelfRef { mutable: true }
            );
            self.define_symbol(param.name.clone(), param.ty.clone(), mutable)?;

            // Track lifetimes for borrowed parameters
            if let Type::WithOwnership(_, Ownership::Borrow | Ownership::BorrowMut) = &param.ty {
                self.active_lifetimes.insert(
                    format!("'{}", param.name),
                    Lifetime {
                        name: format!("'{}", param.name),
                        scope_id: self.current_scope,
                        start_point: 0,
                        end_point: usize::MAX, // Function parameter lifetime
                    },
                );
            }
        }

        // V2.0 Safety: Async functions must have Send + Sync parameters
        if f.is_async {
            for param in &f.params {
                self.check_trait_bound(&param.ty, "Send")?;
                self.check_trait_bound(&param.ty, "Sync")?;
            }
        }

        let params: Vec<_> = f
            .params
            .iter()
            .map(|p| (p.name.clone(), p.ty.clone()))
            .collect();

        let return_type = f.return_type.clone().unwrap_or(Type::Named("()".into()));
        self.current_function_return = Some(return_type.clone());

        // Analyze body statements with lifetime tracking
        let body = self.analyze_block(&f.body)?;

        // Check that all paths return the correct type
        self.check_return_paths(&body, &return_type)?;

        self.pop_scope()?;
        self.current_function = None;
        self.current_function_return = None;

        Ok(TypedItem::Function(TypedFunction {
            name: f.name,
            params,
            return_type,
            body,
            is_async: f.is_async,
        }))
    }

    /// Check that all code paths return the expected type
    fn check_return_paths(
        &self,
        body: &[TypedStatement],
        expected: &Type,
    ) -> Result<(), SemanticError> {
        // Simple check - in a full implementation, would do control flow analysis
        if types_equal(expected, &Type::Named("()".to_string())) {
            return Ok(()); // Void functions don't need explicit returns
        }

        // Check if last statement is a return or if function has explicit returns
        let has_return = body
            .iter()
            .any(|stmt| matches!(stmt, TypedStatement::Return(_)));
        if !has_return && !body.is_empty() {
            // Check if last expression can be implicitly returned
            if let Some(TypedStatement::Expr(expr)) = body.last() {
                let _ = self.unify_types(expected, &expr.ty)?;
            }
        }

        Ok(())
    }

    fn unify_types(&self, expected: &Type, got: &Type) -> Result<Type, SemanticError> {
        if types_equal(expected, got) {
            return Ok(expected.clone());
        }

        // Handle type inference placeholder
        if matches!(got, Type::Named(n) if n == "_") {
            return Ok(expected.clone());
        }
        if matches!(expected, Type::Named(n) if n == "_") {
            return Ok(got.clone());
        }

        // Numeric coercion
        match (expected, got) {
            (Type::I32, Type::I64) | (Type::I64, Type::I32) => Ok(Type::I64),
            (Type::F32, Type::F64) | (Type::F64, Type::F32) => Ok(Type::F64),
            _ => Err(SemanticError::TypeMismatch {
                expected: format!("{:?}", expected),
                got: format!("{:?}", got),
            }),
        }
    }

    fn analyze_block(&mut self, block: &Block) -> Result<Vec<TypedStatement>, SemanticError> {
        let mut stmts = Vec::new();
        for stmt in &block.statements {
            stmts.push(self.analyze_statement(stmt)?);
        }
        Ok(stmts)
    }

    fn analyze_statement(&mut self, stmt: &Statement) -> Result<TypedStatement, SemanticError> {
        match stmt {
            Statement::Let {
                name,
                mutable,
                ty,
                value,
            } => {
                let typed_value = if let Some(v) = value {
                    self.analyze_expression(v)?
                } else {
                    let var_ty = ty.clone().unwrap_or(Type::I64);
                    TypedExpr {
                        kind: TypedExprKind::Literal(Literal::Null),
                        ty: var_ty.clone(),
                    }
                };
                let var_ty = ty.clone().unwrap_or(typed_value.ty.clone());
                self.define_binding_symbols(name, var_ty.clone(), *mutable)?;
                Ok(TypedStatement::Let {
                    name: name.clone(),
                    ty: var_ty,
                    value: typed_value,
                })
            }
            Statement::Var { name, ty, value } => {
                let typed_value = value
                    .as_ref()
                    .map(|v| self.analyze_expression(v))
                    .transpose()?;
                let var_ty = ty.clone().unwrap_or_else(|| {
                    typed_value
                        .as_ref()
                        .map(|v| v.ty.clone())
                        .unwrap_or(Type::I64)
                });
                self.define_binding_symbols(name, var_ty.clone(), true)?; // var is always mutable
                let default_value = typed_value.unwrap_or_else(|| TypedExpr {
                    kind: TypedExprKind::Literal(Literal::Int(0)),
                    ty: var_ty.clone(),
                });
                Ok(TypedStatement::Let {
                    name: name.clone(),
                    ty: var_ty,
                    value: default_value,
                })
            }
            Statement::Assignment { target, op, value } => {
                let typed_target = self.analyze_expression(target)?;
                let typed_value = self.analyze_expression(value)?;

                // Check if the target is an lvalue
                if !typed_target.is_lvalue() {
                    return Err(SemanticError::new("Target of assignment must be an lvalue"));
                }

                // Unify assignment types so placeholder '_' can absorb inferred values.
                if let Err(_) = self.unify_types(&typed_target.ty, &typed_value.ty) {
                    return Err(SemanticError::new(&format!(
                        "Type mismatch in assignment: expected {:?}, found {:?}",
                        typed_target.ty, typed_value.ty
                    )));
                }

                Ok(TypedStatement::Assignment {
                    target: typed_target,
                    op: *op,
                    value: typed_value,
                })
            }
            Statement::Return(expr) => {
                if self.current_function.is_none() {
                    return Err(SemanticError::InvalidSyntax {
                        line: 0,
                        message: "return statement outside of function".to_string(),
                    });
                }
                let typed_expr = expr
                    .as_ref()
                    .map(|e| self.analyze_expression(e))
                    .transpose()?;
                Ok(TypedStatement::Return(typed_expr))
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                let typed_cond = self.analyze_expression(condition)?;
                self.push_scope();
                let typed_then = self.analyze_block(then_block)?;
                self.pop_scope()?;
                let typed_else = if let Some(eb) = else_block {
                    self.push_scope();
                    let result = self.analyze_block(eb)?;
                    self.pop_scope()?;
                    Some(result)
                } else {
                    None
                };
                Ok(TypedStatement::If {
                    condition: typed_cond,
                    then_block: typed_then,
                    else_block: typed_else,
                })
            }
            Statement::For { var, iter, body } => {
                let typed_iter = self.analyze_expression(iter)?;
                self.push_scope();
                self.loop_depth += 1;
                // Infer element type from iterator
                self.define_binding_symbols(var, Type::Named("_".into()), false)?;
                let typed_body = self.analyze_block(body)?;
                self.loop_depth -= 1;
                self.pop_scope()?;
                Ok(TypedStatement::For {
                    var: var.clone(),
                    iter: typed_iter,
                    body: typed_body,
                })
            }
            Statement::While { condition, body } => {
                let typed_cond = self.analyze_expression(condition)?;
                self.push_scope();
                self.loop_depth += 1;
                let typed_body = self.analyze_block(body)?;
                self.loop_depth -= 1;
                self.pop_scope()?;
                Ok(TypedStatement::While {
                    condition: typed_cond,
                    body: typed_body,
                })
            }
            Statement::Loop { body } => {
                self.push_scope();
                self.loop_depth += 1;
                let typed_body = self.analyze_block(body)?;
                self.loop_depth -= 1;
                self.pop_scope()?;
                Ok(TypedStatement::Loop { body: typed_body })
            }
            Statement::Match { expr, arms } => {
                let typed_expr = self.analyze_expression(expr)?;

                // Check for exhaustiveness
                let has_wildcard = arms
                    .iter()
                    .any(|arm| matches!(arm.pattern, Pattern::Wildcard | Pattern::Binding(_)));

                if !has_wildcard {
                    // For bool type, check if both true and false are covered
                    if matches!(typed_expr.ty, Type::Bool) {
                        let has_true = arms.iter().any(|arm| {
                            matches!(arm.pattern, Pattern::Literal(Literal::Bool(true)))
                        });
                        let has_false = arms.iter().any(|arm| {
                            matches!(arm.pattern, Pattern::Literal(Literal::Bool(false)))
                        });
                        if !has_true || !has_false {
                            println!("Warning: non-exhaustive match on bool; missing true/false");
                        }
                    }
                }

                let mut typed_arms = Vec::new();
                for arm in arms {
                    self.push_scope();
                    // Add pattern bindings to scope
                    self.bind_pattern(&arm.pattern, &typed_expr.ty)?;
                    let body = match &arm.body {
                        MatchBody::Block(b) => self.analyze_block(b)?,
                        MatchBody::Expr(e) => {
                            vec![TypedStatement::Expr(self.analyze_expression(e)?)]
                        }
                    };
                    self.pop_scope()?;
                    typed_arms.push((arm.pattern.clone(), body));
                }
                Ok(TypedStatement::Match {
                    expr: typed_expr,
                    arms: typed_arms,
                })
            }
            Statement::Defer(inner) => {
                let typed_inner = self.analyze_statement(inner)?;
                Ok(TypedStatement::Defer(Box::new(typed_inner)))
            }
            Statement::Break(_) => Ok(TypedStatement::Break),
            Statement::Continue => Ok(TypedStatement::Continue),
            Statement::Expression(expr) => {
                let typed_expr = self.analyze_expression(expr)?;
                Ok(TypedStatement::Expr(typed_expr))
            }
            Statement::Pass => Ok(TypedStatement::Pass),
            Statement::Yield(expr) => {
                let typed_expr = expr
                    .as_ref()
                    .map(|e| self.analyze_expression(e))
                    .transpose()?;
                Ok(TypedStatement::Yield(typed_expr))
            }
            Statement::Spawn(expr) => {
                let typed_expr = self.analyze_expression(expr)?;
                Ok(TypedStatement::Spawn(Box::new(typed_expr)))
            }
            Statement::Select { arms } => {
                let mut typed_arms = Vec::new();
                for arm in arms {
                    let typed_channel = self.analyze_expression(&arm.channel_op)?;
                    self.push_scope();
                    let typed_body = self.analyze_block(&arm.body)?;
                    self.pop_scope()?;
                    typed_arms.push(TypedSelectArm {
                        pattern: arm.pattern.clone(),
                        channel_op: typed_channel,
                        body: typed_body,
                    });
                }
                Ok(TypedStatement::Select { arms: typed_arms })
            }
        }
    }

    fn bind_pattern(&mut self, pattern: &Pattern, ty: &Type) -> Result<(), SemanticError> {
        match pattern {
            Pattern::Binding(name) => {
                self.define_symbol(name.clone(), ty.clone(), false)?;
            }
            Pattern::Constructor(_, fields) => {
                for field in fields {
                    self.bind_pattern(field, &Type::Named("_".into()))?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn analyze_expression(&mut self, expr: &Expression) -> Result<TypedExpr, SemanticError> {
        match expr {
            Expression::Literal(lit) => {
                let ty = match lit {
                    Literal::Int(_) => Type::I64,
                    Literal::Float(_) => Type::F64,
                    Literal::String(_) => Type::Str,
                    Literal::Bool(_) => Type::Bool,
                    Literal::Null => Type::Named("Null".to_string()),
                };
                Ok(TypedExpr {
                    kind: TypedExprKind::Literal(lit.clone()),
                    ty,
                })
            }
            Expression::Identifier(name) => {
                match self.lookup_symbol(name) {
                    Ok(sym) => {
                        if sym.borrow_state == BorrowState::Moved {
                            return Err(SemanticError::MoveError { name: name.clone() });
                        }
                        Ok(TypedExpr {
                            kind: TypedExprKind::Identifier(name.clone()),
                            ty: sym.ty.clone(),
                        })
                    }
                    Err(_) => {
                        // O-013: Check if it's a known struct constructor before erroring
                        if self.struct_layouts.contains_key(name) {
                            Ok(TypedExpr {
                                kind: TypedExprKind::Identifier(name.clone()),
                                ty: Type::Named(name.clone()),
                            })
                        } else if self.enum_variants.contains_key(name) {
                            Ok(TypedExpr {
                                kind: TypedExprKind::Identifier(name.clone()),
                                ty: Type::Named(name.clone()),
                            })
                        } else {
                            warn!(
                                "semantic: unresolved identifier '{}', using placeholder type",
                                name
                            );
                            Ok(TypedExpr {
                                kind: TypedExprKind::Identifier(name.clone()),
                                ty: Type::Named("_".to_string()),
                            })
                        }
                    }
                }
            }
            Expression::Binary(left, op, right) => {
                let typed_left = self.analyze_expression(left)?;
                let typed_right = self.analyze_expression(right)?;
                let result_ty = self.infer_binary_type(&typed_left.ty, op, &typed_right.ty)?;
                Ok(TypedExpr {
                    kind: TypedExprKind::Binary(Box::new(typed_left), *op, Box::new(typed_right)),
                    ty: result_ty,
                })
            }
            Expression::Unary(op, inner) => {
                let typed_inner = self.analyze_expression(inner)?;
                Ok(TypedExpr {
                    kind: TypedExprKind::Unary(*op, Box::new(typed_inner.clone())),
                    ty: typed_inner.ty,
                })
            }
            Expression::Call(func, args) => {
                let typed_func = self.analyze_expression(func)?;
                let typed_args: Result<Vec<_>, _> =
                    args.iter().map(|a| self.analyze_expression(a)).collect();
                let return_ty = match &typed_func.ty {
                    Type::Function(_, ret) => ret
                        .as_ref()
                        .map(|r| *r.clone())
                        .unwrap_or(Type::Named("()".into())),
                    _ => Type::Named("_".into()),
                };
                Ok(TypedExpr {
                    kind: TypedExprKind::Call(Box::new(typed_func), typed_args?),
                    ty: return_ty,
                })
            }
            Expression::MethodCall {
                receiver,
                method,
                args,
            } => {
                let typed_receiver = self.analyze_expression(receiver)?;
                let typed_args: Result<Vec<_>, _> =
                    args.iter().map(|a| self.analyze_expression(a)).collect();
                let typed_args = typed_args?;
                // Look up the method's return type from impl blocks
                let method_return_ty = self.resolve_method_return_type(&typed_receiver.ty, method);
                Ok(TypedExpr {
                    kind: TypedExprKind::MethodCall {
                        receiver: Box::new(typed_receiver),
                        method: method.clone(),
                        args: typed_args,
                    },
                    ty: method_return_ty,
                })
            }
            Expression::Field(obj, field) => {
                let typed_obj = self.analyze_expression(obj)?;
                // Validate field exists on struct type
                let field_ty = self.resolve_field_type(&typed_obj.ty, field)?;
                Ok(TypedExpr {
                    kind: TypedExprKind::Field(Box::new(typed_obj), field.clone()),
                    ty: field_ty,
                })
            }
            Expression::Index(arr, idx) => {
                let typed_arr = self.analyze_expression(arr)?;
                let typed_idx = self.analyze_expression(idx)?;
                let is_range_index = matches!(&typed_idx.kind, TypedExprKind::Range { .. });
                // Validate scalar index is an integer type
                if !is_range_index && !self.is_integer_type(&typed_idx.ty) {
                    return Err(SemanticError::TypeMismatch {
                        expected: "integer type".to_string(),
                        got: self.type_to_string(&typed_idx.ty),
                    });
                }
                let result_ty = if is_range_index {
                    // Slicing preserves the collection type shape at this stage.
                    typed_arr.ty.clone()
                } else {
                    match &typed_arr.ty {
                        Type::Array(inner, _) | Type::Slice(inner) => *inner.clone(),
                        _ => Type::Named("_".into()),
                    }
                };
                Ok(TypedExpr {
                    kind: TypedExprKind::Index(Box::new(typed_arr), Box::new(typed_idx)),
                    ty: result_ty,
                })
            }
            Expression::Borrow {
                mutable,
                expr: inner,
            } => {
                if let Expression::Identifier(name) = inner.as_ref() {
                    self.record_borrow(name, *mutable)?;
                }
                let typed_inner = self.analyze_expression(inner)?;
                let borrow_ty = Type::WithOwnership(
                    Box::new(typed_inner.ty.clone()),
                    if *mutable {
                        Ownership::BorrowMut
                    } else {
                        Ownership::Borrow
                    },
                );
                Ok(TypedExpr {
                    kind: TypedExprKind::Borrow {
                        mutable: *mutable,
                        expr: Box::new(typed_inner),
                    },
                    ty: borrow_ty,
                })
            }
            Expression::Deref(inner) => {
                let typed_inner = self.analyze_expression(inner)?;
                let deref_ty = match &typed_inner.ty {
                    Type::WithOwnership(inner, _) => *inner.clone(),
                    _ => typed_inner.ty.clone(),
                };
                Ok(TypedExpr {
                    kind: TypedExprKind::Deref(Box::new(typed_inner)),
                    ty: deref_ty,
                })
            }
            Expression::StructLiteral { name, fields } => {
                let typed_fields: Result<Vec<_>, _> = fields
                    .iter()
                    .map(|(n, e)| self.analyze_expression(e).map(|te| (n.clone(), te)))
                    .collect();
                Ok(TypedExpr {
                    kind: TypedExprKind::StructLiteral {
                        name: name.clone(),
                        fields: typed_fields?,
                    },
                    ty: Type::Named(name.clone()),
                })
            }
            Expression::Array(elements) => {
                let typed_elems: Result<Vec<_>, _> = elements
                    .iter()
                    .map(|e| self.analyze_expression(e))
                    .collect();
                let typed_elems = typed_elems?;
                let elem_ty = typed_elems
                    .first()
                    .map(|e| e.ty.clone())
                    .unwrap_or(Type::Named("_".into()));
                Ok(TypedExpr {
                    kind: TypedExprKind::Array(typed_elems),
                    ty: Type::Array(Box::new(elem_ty), None),
                })
            }
            Expression::Range { start, end, inclusive } => {
                let typed_start = start
                    .as_ref()
                    .map(|e| self.analyze_expression(e).map(Box::new))
                    .transpose()?;
                let typed_end = end
                    .as_ref()
                    .map(|e| self.analyze_expression(e).map(Box::new))
                    .transpose()?;
                let elem_ty = typed_start
                    .as_ref()
                    .map(|e| e.ty.clone())
                    .or_else(|| typed_end.as_ref().map(|e| e.ty.clone()))
                    .unwrap_or(Type::I64);
                Ok(TypedExpr {
                    kind: TypedExprKind::Range {
                        start: typed_start,
                        end: typed_end,
                        inclusive: *inclusive,
                    },
                    ty: Type::Array(Box::new(elem_ty), None),
                })
            }
            Expression::Lambda { params, body } => {
                self.push_scope();
                let mut typed_params = Vec::new();
                for p in params {
                    let p_ty = match &p.ty {
                        Type::Named(name) if name == "_" => Type::Named("_".into()),
                        other => other.clone(),
                    };
                    self.define_symbol(p.name.clone(), p_ty.clone(), false)?;
                    typed_params.push((p.name.clone(), p_ty));
                }
                let typed_body = self.analyze_expression(body)?;
                let ret_ty = typed_body.ty.clone();
                self.pop_scope()?;
                let param_types: Vec<Type> = typed_params.iter().map(|(_, t)| t.clone()).collect();
                Ok(TypedExpr {
                    kind: TypedExprKind::Lambda {
                        params: typed_params,
                        body: Box::new(typed_body),
                        return_type: ret_ty.clone(),
                    },
                    ty: Type::Function(param_types, Some(Box::new(ret_ty.clone()))),
                })
            }
            Expression::Tuple(elements) => {
                let typed_elems: Result<Vec<_>, _> = elements
                    .iter()
                    .map(|e| self.analyze_expression(e))
                    .collect();
                let typed_elems = typed_elems?;
                let elem_types: Vec<Type> = typed_elems.iter().map(|e| e.ty.clone()).collect();
                Ok(TypedExpr {
                    kind: TypedExprKind::Tuple(typed_elems),
                    ty: Type::Tuple(elem_types),
                })
            }
            Expression::Await(inner) => {
                let typed_inner = self.analyze_expression(inner)?;
                let inner_ty = match &typed_inner.ty {
                    Type::Generic(name, _) if name == "Future" => {
                        Type::Named("awaited".into())
                    }
                    other => other.clone(),
                };
                Ok(TypedExpr {
                    kind: TypedExprKind::Await(Box::new(typed_inner)),
                    ty: inner_ty,
                })
            }
            Expression::None => Ok(TypedExpr {
                kind: TypedExprKind::None,
                ty: Type::Nullable(Box::new(Type::Named("_".into()))),
            }),
            Expression::Some(inner) => {
                let typed_inner = self.analyze_expression(inner)?;
                let inner_ty = typed_inner.ty.clone();
                Ok(TypedExpr {
                    kind: TypedExprKind::Some(Box::new(typed_inner)),
                    ty: Type::Nullable(Box::new(inner_ty)),
                })
            }
            Expression::Ok(inner) => {
                let typed_inner = self.analyze_expression(inner)?;
                let ok_ty = typed_inner.ty.clone();
                Ok(TypedExpr {
                    kind: TypedExprKind::Ok(Box::new(typed_inner)),
                    ty: Type::Generic("Result".into(), vec![ok_ty, Type::Named("_".into())]),
                })
            }
            Expression::Err(inner) => {
                let typed_inner = self.analyze_expression(inner)?;
                let err_ty = typed_inner.ty.clone();
                Ok(TypedExpr {
                    kind: TypedExprKind::Err(Box::new(typed_inner)),
                    ty: Type::Generic("Result".into(), vec![Type::Named("_".into()), err_ty]),
                })
            }
            Expression::Shared(inner) => {
                let typed_inner = self.analyze_expression(inner)?;
                let inner_ty = typed_inner.ty.clone();
                Ok(TypedExpr {
                    kind: typed_inner.kind,
                    ty: Type::WithOwnership(Box::new(inner_ty), Ownership::Shared),
                })
            }
            Expression::Own(inner) => {
                let typed_inner = self.analyze_expression(inner)?;
                let inner_ty = typed_inner.ty.clone();
                Ok(TypedExpr {
                    kind: typed_inner.kind,
                    ty: Type::WithOwnership(Box::new(inner_ty), Ownership::Owned),
                })
            }
            Expression::If {
                condition,
                then_expr,
                else_expr,
            } => {
                let typed_cond = self.analyze_expression(condition)?;
                let typed_then = self.analyze_expression(then_expr)?;
                let typed_else = match else_expr {
                    Some(e) => self.analyze_expression(e)?,
                    None => TypedExpr {
                        kind: TypedExprKind::Literal(Literal::Null),
                        ty: Type::Named("Null".into()),
                    },
                };
                let result_ty = typed_then.ty.clone();
                Ok(TypedExpr {
                    kind: TypedExprKind::If {
                        condition: Box::new(typed_cond),
                        then_expr: Box::new(typed_then),
                        else_expr: Box::new(typed_else),
                    },
                    ty: result_ty,
                })
            }
            Expression::Match { expr: match_expr, arms } => {
                let typed_expr = self.analyze_expression(match_expr)?;
                let mut typed_arms = Vec::new();
                for arm in arms {
                    self.push_scope();
                    self.bind_pattern(&arm.pattern, &typed_expr.ty)?;
                    let typed_body = match &arm.body {
                        MatchBody::Block(b) => {
                            let stmts = self.analyze_block(b)?;
                            if stmts.len() == 1 {
                                if let TypedStatement::Expr(e) = &stmts[0] {
                                    e.clone()
                                } else {
                                    TypedExpr {
                                        kind: TypedExprKind::Literal(Literal::Null),
                                        ty: Type::Named("Null".into()),
                                    }
                                }
                            } else {
                                TypedExpr {
                                    kind: TypedExprKind::Literal(Literal::Null),
                                    ty: Type::Named("Null".into()),
                                }
                            }
                        }
                        MatchBody::Expr(e) => self.analyze_expression(e)?,
                    };
                    self.pop_scope()?;
                    typed_arms.push((arm.pattern.clone(), typed_body));
                }
                let arm_ty = typed_arms
                    .first()
                    .map(|(_, e)| e.ty.clone())
                    .unwrap_or(Type::Named("_".into()));
                Ok(TypedExpr {
                    kind: TypedExprKind::Match {
                        expr: Box::new(typed_expr),
                        arms: typed_arms,
                    },
                    ty: arm_ty,
                })
            }
            Expression::Path(module, item) => {
                // Resolve as a qualified call: Module::item → treat as identifier
                let name = format!("{}::{}", 
                    match &**module {
                        Expression::Identifier(n) => n.clone(),
                        _ => "?".into(),
                    },
                    item
                );
                Ok(TypedExpr {
                    kind: TypedExprKind::Identifier(name),
                    ty: Type::Named("_".into()),
                })
            }
            Expression::ListComprehension { .. } | Expression::Generator { .. } => {
                Err(SemanticError::UnsupportedConstruct(
                    "List comprehensions and generators are not yet supported in the typed pipeline".into()
                ))
            }
        }
    }

    fn infer_binary_type(
        &self,
        left: &Type,
        op: &BinaryOp,
        right: &Type,
    ) -> Result<Type, SemanticError> {
        match op {
            BinaryOp::Eq
            | BinaryOp::NotEq
            | BinaryOp::Lt
            | BinaryOp::Gt
            | BinaryOp::LtEq
            | BinaryOp::GtEq
            | BinaryOp::And
            | BinaryOp::Or => Ok(Type::Bool),
            BinaryOp::Range => Ok(Type::Named("Range".into())),
            _ => {
                // O-001: String + anything → String (concatenation)
                if *op == BinaryOp::Add && (matches!(left, Type::Str) || matches!(right, Type::Str))
                {
                    return Ok(Type::Str);
                }
                // For arithmetic ops, return the "wider" type
                if matches!(left, Type::F64) || matches!(right, Type::F64) {
                    Ok(Type::F64)
                } else if matches!(left, Type::F32) || matches!(right, Type::F32) {
                    Ok(Type::F32)
                } else if matches!(left, Type::I64) || matches!(right, Type::I64) {
                    Ok(Type::I64)
                } else {
                    Ok(left.clone())
                }
            }
        }
    }

    // ===== Phase 3 Integration Methods =====

    /// Integrate constraint solver into expression analysis
    /// Collects constraints and solves them for inferred types
    fn check_expression_with_constraints(
        &mut self,
        expr: &Expression,
    ) -> Result<Type, SemanticError> {
        // Analyze the expression normally
        let typed_expr = self.analyze_expression(expr)?;

        // For now, this is a hook point. Future: collect constraints during analysis
        // and solve them here to improve type inference
        Ok(typed_expr.ty)
    }

    /// Integrate trait resolver into generic type checking
    /// Validates that concrete types satisfy trait bounds on generic parameters
    fn check_generic_trait_bounds(
        &mut self,
        _type_param: &str,
        bounds: &[String],
        concrete_type: &Type,
    ) -> Result<(), SemanticError> {
        for trait_name in bounds {
            // Use trait resolver to check bounds
            if !self
                .trait_resolver
                .implements_trait(concrete_type, trait_name)
            {
                return Err(SemanticError::TraitBoundError(format!(
                    "Type {:?} does not implement trait {}",
                    concrete_type, trait_name
                )));
            }
        }
        Ok(())
    }

    /// Integrate borrow checker into variable binding
    /// Tracks variable ownership and enforces borrow rules
    fn define_variable_with_borrow_check(
        &mut self,
        name: String,
        ty: Type,
        mutable: bool,
    ) -> Result<(), SemanticError> {
        // Define in scope
        self.define_symbol(name.clone(), ty, mutable)?;

        // Also track in borrow checker
        self.borrow_checker
            .bind_variable(name)
            .map_err(|e| SemanticError::BorrowError(e))?;

        Ok(())
    }

    /// Integrate borrow checker into borrow operations
    /// Tracks shared and mutable borrows with scope awareness
    fn check_borrow_with_enforcement(
        &mut self,
        name: &str,
        mutable: bool,
    ) -> Result<(), SemanticError> {
        if mutable {
            self.borrow_checker.borrow_mut_compat(name)
        } else {
            self.borrow_checker.borrow_shared(name)
        }
        .map_err(|e| SemanticError::BorrowError(e))?;
        Ok(())
    }

    /// Integrate borrow checker into move operations
    /// Prevents use-after-move and enforces move semantics
    fn check_move_with_enforcement(&mut self, name: &str) -> Result<(), SemanticError> {
        self.borrow_checker
            .move_var(name)
            .map_err(|_e| SemanticError::MoveError {
                name: name.to_string(),
            })?;
        Ok(())
    }

    /// Integrate monomorphization into generic function calls
    /// Instantiates generic functions with concrete types
    fn instantiate_generic_function_integrated(
        &mut self,
        func_name: &str,
        type_args: &[Type],
    ) -> Result<String, SemanticError> {
        use crate::semantic::monomorphization::{StatementSubstitutor, TypeSubstitution};

        // Get original generic function
        let generic_func = self
            .generic_functions
            .get(func_name)
            .ok_or_else(|| SemanticError::UndefinedSymbol {
                name: func_name.to_string(),
            })?
            .clone();

        // Create type substitution
        let mut subst = TypeSubstitution::new();

        // Collect all trait bounds to check
        let bounds_to_check: Vec<(String, String, Type)> =
            if let Some(_bounds) = self.generic_bounds.get(func_name) {
                let type_param_names: Vec<String> =
                    (0..type_args.len()).map(|i| format!("T{}", i)).collect();

                let mut checks = Vec::new();
                for (param_name, concrete_type) in type_param_names.iter().zip(type_args) {
                    subst.add(param_name.clone(), concrete_type.clone());

                    // Check trait bounds on the concrete type
                    if let Some(trait_bounds) = self.generic_bounds.get(param_name) {
                        for trait_name in trait_bounds {
                            checks.push((
                                param_name.clone(),
                                trait_name.clone(),
                                concrete_type.clone(),
                            ));
                        }
                    }
                }
                checks
            } else {
                // No bounds info, just map generics directly
                for (i, concrete_type) in type_args.iter().enumerate() {
                    subst.add(format!("T{}", i), concrete_type.clone());
                }
                Vec::new()
            };

        // Now check all bounds
        for (param_name, trait_name, concrete_type) in bounds_to_check {
            self.check_generic_trait_bounds(&param_name, &[trait_name], &concrete_type)?;
        }

        // Create mangled name
        let mut mangled = func_name.to_string();
        for ty in type_args {
            mangled.push('_');
            mangled.push_str(&self.type_to_string(ty));
        }

        // Check if already monomorphized
        if self.monomorphized.contains_key(&mangled) {
            return Ok(mangled);
        }

        // Monomorphize function body
        let substituted_body: Vec<TypedStatement> = generic_func
            .body
            .statements
            .iter()
            .filter_map(|stmt| {
                let substitutor = StatementSubstitutor::new(subst.clone());
                let transformed = substitutor.transform_statement(stmt);
                self.statement_to_typed(&transformed).ok()
            })
            .collect();

        // Store monomorphized function
        self.monomorphized.insert(
            mangled.clone(),
            MonomorphizedFunc {
                original_name: func_name.to_string(),
                type_args: type_args.to_vec(),
                mangled_name: mangled.clone(),
                typed_body: substituted_body,
            },
        );

        Ok(mangled)
    }

    /// Helper to convert Statement to TypedStatement for monomorphization
    fn statement_to_typed(&mut self, stmt: &Statement) -> Result<TypedStatement, SemanticError> {
        match stmt {
            Statement::Let {
                name, ty, value, ..
            } => {
                let typed_value = if let Some(v) = value {
                    self.analyze_expression(v)?
                } else {
                    let var_ty = ty.clone().unwrap_or(Type::I64);
                    TypedExpr {
                        kind: TypedExprKind::Literal(Literal::Null),
                        ty: var_ty.clone(),
                    }
                };
                let var_ty = ty.clone().unwrap_or(typed_value.ty.clone());
                Ok(TypedStatement::Let {
                    name: name.clone(),
                    ty: var_ty,
                    value: typed_value,
                })
            }
            Statement::Var { name: _, ty, value } => {
                let typed_value = value
                    .as_ref()
                    .map(|v| self.analyze_expression(v))
                    .transpose()
                    .ok()
                    .flatten();
                let var_ty = ty.clone().unwrap_or_else(|| {
                    typed_value
                        .as_ref()
                        .map(|v| v.ty.clone())
                        .unwrap_or(Type::I64)
                });
                let default_value = typed_value.unwrap_or_else(|| TypedExpr {
                    kind: TypedExprKind::Literal(Literal::Int(0)),
                    ty: var_ty.clone(),
                });
                Ok(TypedStatement::Let {
                    name: "".to_string(),
                    ty: var_ty,
                    value: default_value,
                })
            }
            Statement::Assignment { target, op, value } => {
                let typed_target = self
                    .analyze_expression(target)
                    .unwrap_or_else(|_| TypedExpr {
                        kind: TypedExprKind::Identifier("_".to_string()),
                        ty: Type::Named("_".to_string()),
                    });
                let typed_value = self
                    .analyze_expression(value)
                    .unwrap_or_else(|_| TypedExpr {
                        kind: TypedExprKind::Identifier("_".to_string()),
                        ty: Type::Named("_".to_string()),
                    });
                Ok(TypedStatement::Assignment {
                    target: typed_target,
                    op: *op,
                    value: typed_value,
                })
            }
            Statement::Expression(expr) => {
                let typed_expr = self.analyze_expression(expr).unwrap_or_else(|_| TypedExpr {
                    kind: TypedExprKind::Identifier("_".to_string()),
                    ty: Type::Named("_".to_string()),
                });
                Ok(TypedStatement::Expr(typed_expr))
            }
            Statement::Return(expr) => {
                let typed_expr = expr
                    .as_ref()
                    .map(|e| self.analyze_expression(e))
                    .transpose()
                    .ok()
                    .flatten();
                Ok(TypedStatement::Return(typed_expr))
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                let typed_cond = self
                    .analyze_expression(condition)
                    .unwrap_or_else(|_| TypedExpr {
                        kind: TypedExprKind::Literal(Literal::Bool(true)),
                        ty: Type::Bool,
                    });
                let typed_then = self.analyze_block(then_block).unwrap_or_default();
                let typed_else = else_block
                    .as_ref()
                    .map(|eb| self.analyze_block(eb).unwrap_or_default());
                Ok(TypedStatement::If {
                    condition: typed_cond,
                    then_block: typed_then,
                    else_block: typed_else,
                })
            }
            Statement::While { condition, body } => {
                let typed_cond = self
                    .analyze_expression(condition)
                    .unwrap_or_else(|_| TypedExpr {
                        kind: TypedExprKind::Literal(Literal::Bool(true)),
                        ty: Type::Bool,
                    });
                let typed_body = self.analyze_block(body).unwrap_or_default();
                Ok(TypedStatement::While {
                    condition: typed_cond,
                    body: typed_body,
                })
            }
            Statement::For { var, iter, body } => {
                let typed_iter = self.analyze_expression(iter).unwrap_or_else(|_| TypedExpr {
                    kind: TypedExprKind::Identifier("_".to_string()),
                    ty: Type::Named("_".to_string()),
                });
                let typed_body = self.analyze_block(body).unwrap_or_default();
                Ok(TypedStatement::For {
                    var: var.clone(),
                    iter: typed_iter,
                    body: typed_body,
                })
            }
            Statement::Loop { body } => {
                let typed_body = self.analyze_block(body).unwrap_or_default();
                Ok(TypedStatement::Loop { body: typed_body })
            }
            Statement::Match { expr, arms } => {
                let typed_expr = self.analyze_expression(expr).unwrap_or_else(|_| TypedExpr {
                    kind: TypedExprKind::Identifier("_".to_string()),
                    ty: Type::Named("_".to_string()),
                });
                let typed_arms: Vec<(Pattern, Vec<TypedStatement>)> = arms
                    .iter()
                    .map(|arm| {
                        let body = match &arm.body {
                            MatchBody::Expr(e) => {
                                vec![TypedStatement::Expr(
                                    self.analyze_expression(e).unwrap_or_else(|_| TypedExpr {
                                        kind: TypedExprKind::Identifier("_".to_string()),
                                        ty: Type::Named("_".to_string()),
                                    }),
                                )]
                            }
                            MatchBody::Block(b) => self.analyze_block(b).unwrap_or_default(),
                        };
                        (arm.pattern.clone(), body)
                    })
                    .collect();
                Ok(TypedStatement::Match {
                    expr: typed_expr,
                    arms: typed_arms,
                })
            }
            Statement::Break(_) => Ok(TypedStatement::Break),
            Statement::Continue => Ok(TypedStatement::Continue),
            Statement::Pass => Ok(TypedStatement::Pass),
            Statement::Defer(inner) => {
                let typed_inner = self.statement_to_typed(inner)?;
                Ok(TypedStatement::Defer(Box::new(typed_inner)))
            }
            Statement::Yield(expr) => {
                let typed_expr = expr
                    .as_ref()
                    .map(|e| self.analyze_expression(e))
                    .transpose()
                    .ok()
                    .flatten();
                Ok(TypedStatement::Yield(typed_expr))
            }
            Statement::Spawn(expr) => {
                let typed_expr = self.analyze_expression(expr).unwrap_or_else(|_| TypedExpr {
                    kind: TypedExprKind::Identifier("_".to_string()),
                    ty: Type::Named("_".to_string()),
                });
                Ok(TypedStatement::Spawn(Box::new(typed_expr)))
            }
            _ => Ok(TypedStatement::Pass),
        }
    }

    pub fn analyze_struct(&mut self, s: StructDef) -> Result<TypedItem, SemanticError> {
        debug!("Analyzing struct: {}", s.name);
        let fields: Vec<_> = s
            .fields
            .iter()
            .map(|f| (f.name.clone(), f.ty.clone()))
            .collect();

        let implements = s.implements.map(|i| vec![i]).unwrap_or_default();

        Ok(TypedItem::Struct(TypedStruct {
            name: s.name,
            fields,
            implements,
        }))
    }
}

pub fn analyze(module: Module) -> Result<TypedModule, SemanticError> {
    let mut analyzer = Analyzer::new();
    analyzer.analyze(module)
}
