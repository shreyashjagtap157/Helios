#![allow(dead_code)]
//! Omni AST - Abstract Syntax Tree Definitions
//!
//! Defines the structure of parsed Omni programs.

/// A complete Omni module (compilation unit)
#[derive(Debug, Clone)]
pub struct Module {
    pub items: Vec<Item>,
}

/// Top-level items in a module
#[derive(Debug, Clone)]
pub enum Item {
    Module(ModuleDecl),
    Struct(StructDef),
    Enum(EnumDef),
    Function(Function),
    Trait(TraitDef),
    Impl(ImplBlock),
    Import(ImportDecl),
    Const(ConstDecl),
    TypeAlias(TypeAlias),
    Extern(ExternBlock),
    Comptime(Block),
    Macro(MacroDef),
}

/// Enum definition
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub attributes: Vec<String>,
    pub variants: Vec<EnumVariant>,
}

/// Enum variant
#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Option<EnumFields>,
}

/// Enum variant fields
#[derive(Debug, Clone)]
pub enum EnumFields {
    Tuple(Vec<Type>),
    Struct(Vec<Field>),
}

/// Type alias declaration
#[derive(Debug, Clone)]
pub struct TypeAlias {
    pub name: String,
    pub ty: Type,
}

/// Macro definition
#[derive(Debug, Clone)]
pub struct MacroDef {
    pub name: String,
    pub body: Block,
}

/// External ABI block (e.g. extern "C++")
#[derive(Debug, Clone)]
pub struct ExternBlock {
    pub abi: String,
    pub functions: Vec<Function>,
}

/// Module declaration
#[derive(Debug, Clone)]
pub struct ModuleDecl {
    pub name: String,
    pub attributes: Vec<String>,
    pub items: Vec<Item>,
}

/// Struct definition
#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub attributes: Vec<String>,
    pub implements: Option<String>,
    pub fields: Vec<Field>,
    pub methods: Vec<Function>,
}

/// Struct field
#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}

/// Function definition
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub is_async: bool,
    pub attributes: Vec<String>,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Block,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}

/// Trait definition
#[derive(Debug, Clone)]
pub struct TraitDef {
    pub name: String,
    pub attributes: Vec<String>,
    pub methods: Vec<Function>,
}

/// Implementation block
#[derive(Debug, Clone)]
pub struct ImplBlock {
    pub trait_name: String,
    pub type_name: String,
    pub attributes: Vec<String>,
    pub methods: Vec<Function>,
}

/// Import declaration
#[derive(Debug, Clone)]
pub struct ImportDecl {
    pub path: Vec<String>,
    pub alias: Option<String>,
}

/// Constant declaration
#[derive(Debug, Clone)]
pub struct ConstDecl {
    pub name: String,
    pub attributes: Vec<String>,
    pub ty: Type,
    pub value: Expression,
}

/// Type annotations
#[derive(Debug, Clone)]
pub enum Type {
    // Primitives
    U8, U16, U32, U64, Usize,
    I8, I16, I32, I64, Isize,
    F32, F64,
    Bool,
    Str,
    
    // Compound
    Array(Box<Type>, Option<Box<Expression>>),  // [T; N] or [T]
    Slice(Box<Type>),                           // [T]
    Named(String),                              // Custom types
    Generic(String, Vec<Type>),                 // HashMap<K, V>
    Function(Vec<Type>, Option<Box<Type>>),     // fn(A, B) -> C
    
    // Ownership modifiers
    WithOwnership(Box<Type>, Ownership),
    
    // Self types for methods
    SelfOwned,
    SelfRef { mutable: bool },
}

/// Ownership semantics
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ownership {
    Owned,      // own T
    Borrow,     // &T
    BorrowMut,  // &mut T
    Shared,     // shared T (Arc)
    RawPointer, // unsafe *T
}

/// A block of statements
#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
}

/// Statements
#[derive(Debug, Clone)]
pub enum Statement {
    Let {
        name: String,
        mutable: bool,
        ty: Option<Type>,
        value: Expression,
    },
    Var {
        name: String,
        ty: Option<Type>,
        value: Option<Expression>,
    },
    Assignment {
        target: Expression,
        op: Option<BinaryOp>,  // None = simple assign (=), Some = compound (+=, -=, etc.)
        value: Expression,
    },
    Return(Option<Expression>),
    If {
        condition: Expression,
        then_block: Block,
        else_block: Option<Block>,
    },
    For {
        var: String,
        iter: Expression,
        body: Block,
    },
    While {
        condition: Expression,
        body: Block,
    },
    Loop {
        body: Block,
    },
    Match {
        expr: Expression,
        arms: Vec<MatchArm>,
    },
    Defer(Box<Statement>),
    Break,
    Continue,
    Yield(Option<Expression>),
    Spawn(Box<Expression>),
    Select {
        arms: Vec<SelectArm>,
    },
    Expression(Expression),
}

/// Select arm for channel operations
#[derive(Debug, Clone)]
pub struct SelectArm {
    pub pattern: Pattern,
    pub channel_op: Expression,
    pub body: Block,
}

/// Match arm
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: MatchBody,
}

/// Match arm body
#[derive(Debug, Clone)]
pub enum MatchBody {
    Expr(Expression),
    Block(Block),
}

/// Patterns for matching
#[derive(Debug, Clone)]
pub enum Pattern {
    Literal(Literal),
    Binding(String),
    Constructor(String, Vec<Pattern>),
    Wildcard,
}

/// Expressions
#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    Binary(Box<Expression>, BinaryOp, Box<Expression>),
    Unary(UnaryOp, Box<Expression>),
    Call(Box<Expression>, Vec<Expression>),
    MethodCall {
        receiver: Box<Expression>,
        method: String,
        args: Vec<Expression>,
    },
    Field(Box<Expression>, String),
    Index(Box<Expression>, Box<Expression>),
    Path(Box<Expression>, String),  // module::item
    Array(Vec<Expression>),
    StructLiteral {
        name: String,
        fields: Vec<(String, Expression)>,
    },
    Borrow {
        mutable: bool,
        expr: Box<Expression>,
    },
    Deref(Box<Expression>),
    Await(Box<Expression>),
    Range {
        start: Option<Box<Expression>>,
        end: Option<Box<Expression>>,
        inclusive: bool,
    },
    Lambda {
        params: Vec<Param>,
        body: Box<Expression>,
    },
    If {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Option<Box<Expression>>,
    },
    Match {
        expr: Box<Expression>,
        arms: Vec<MatchArm>,
    },
    ListComprehension {
        expr: Box<Expression>,
        var: String,
        iter: Box<Expression>,
        filter: Option<Box<Expression>>,
    },
    Generator {
        body: Block,
    },
    Tuple(Vec<Expression>),
    None,
    Some(Box<Expression>),
    Ok(Box<Expression>),
    Err(Box<Expression>),
}

/// Literal values
#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    // Arithmetic
    Add, Sub, Mul, Div, Mod,
    // Comparison
    Eq, NotEq, Lt, Gt, LtEq, GtEq,
    // Logical
    And, Or,
    // Range
    Range, RangeInclusive,
}

impl BinaryOp {
    pub fn precedence(&self) -> u8 {
        match self {
            BinaryOp::Or => 1,
            BinaryOp::And => 2,
            BinaryOp::Eq | BinaryOp::NotEq => 3,
            BinaryOp::Lt | BinaryOp::Gt | BinaryOp::LtEq | BinaryOp::GtEq => 4,
            BinaryOp::Range | BinaryOp::RangeInclusive => 5,
            BinaryOp::Add | BinaryOp::Sub => 6,
            BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => 7,
        }
    }
}

/// Unary operators
#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Neg,  // -
    Not,  // !
}
