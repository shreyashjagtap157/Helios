// Abstract Syntax Tree for Omni Language

#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
    FunctionDef(FunctionDef),
    StructDef(StructDef),
    Variable(VariableDecl),
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<(String, Type)>,
}

#[derive(Debug, Clone)]
pub struct VariableDecl {
    pub name: String,
    pub var_type: Option<Type>,
    pub is_mutable: bool,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let(String, Option<Type>, bool, Expression),
    Expression(Expression),
    If(Expression, Block, Option<Block>),
    While(Expression, Block),
    For(String, Expression, Block),
    Return(Option<Expression>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    I32,
    I64,
    F64,
    Bool,
    String,
    Void,
    Custom(String),
    Array(Box<Type>, usize),
}

impl Type {
    pub fn name(&self) -> String {
        match self {
            Type::I32 => "i32".to_string(),
            Type::I64 => "i64".to_string(),
            Type::F64 => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "string".to_string(),
            Type::Void => "void".to_string(),
            Type::Custom(name) => name.clone(),
            Type::Array(inner, size) => format!("[{}; {}]", inner.name(), size),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Identifier(String),
    Literal(Literal),
    BinaryOp(Box<Expression>, BinaryOp, Box<Expression>),
    UnaryOp(UnaryOp, Box<Expression>),
    Call(String, Vec<Expression>),
    MethodCall(Box<Expression>, String, Vec<Expression>),
    If(Box<Expression>, Box<Block>, Option<Box<Block>>),
    Assignment(String, Box<Expression>),
    FieldAccess(Box<Expression>, String),
    Index(Box<Expression>, Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Negate,
    Not,
}
