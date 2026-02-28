/// Operator Overloading Support
/// Enables custom implementations of operators like +, -, *, /, etc.
/// 
/// Syntax: impl Add for MyType { fn add(self, other: Self) -> Self { ... } }

use crate::ir::{Type, Trait, TraitImpl};
use std::collections::HashMap;

/// All supported operators that can be overloaded
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operator {
    // Arithmetic
    Add,      // +
    Sub,      // -
    Mul,      // *
    Div,      // /
    Mod,      // %
    Pow,      // **
    Neg,      // unary -
    
    // Comparison
    Eq,       // ==
    NotEq,    // !=
    Lt,       // <
    Lte,      // <=
    Gt,       // >
    Gte,      // >=
    
    // Bitwise
    BitAnd,   // &
    BitOr,    // |
    BitXor,   // ^
    BitNot,   // ~
    Shl,      // <<
    Shr,      // >>
    
    // Logical
    And,      // && (short-circuit in syntax, not method)
    Or,       // || (short-circuit in syntax, not method)
    Not,      // !
    
    // Index
    Index,    // []
    IndexMut, // []mut
    
    // Other
    Call,     // ()
    Deref,    // *
    DerefMut, // *mut
    Range,    // ..
}

impl Operator {
    /// Get the trait name associated with this operator
    pub fn trait_name(&self) -> &'static str {
        match self {
            Operator::Add => "Add",
            Operator::Sub => "Sub",
            Operator::Mul => "Mul",
            Operator::Div => "Div",
            Operator::Mod => "Rem",
            Operator::Pow => "Pow",
            Operator::Neg => "Neg",
            Operator::Eq => "PartialEq",
            Operator::NotEq => "PartialEq",
            Operator::Lt => "PartialOrd",
            Operator::Lte => "PartialOrd",
            Operator::Gt => "PartialOrd",
            Operator::Gte => "PartialOrd",
            Operator::BitAnd => "BitAnd",
            Operator::BitOr => "BitOr",
            Operator::BitXor => "BitXor",
            Operator::BitNot => "BitNot",
            Operator::Shl => "Shl",
            Operator::Shr => "Shr",
            Operator::And => "LogicalAnd",
            Operator::Or => "LogicalOr",
            Operator::Not => "Not",
            Operator::Index => "Index",
            Operator::IndexMut => "IndexMut",
            Operator::Call => "Fn",
            Operator::Deref => "Deref",
            Operator::DerefMut => "DerefMut",
            Operator::Range => "Range",
        }
    }

    /// Get the method name for this operator
    pub fn method_name(&self) -> &'static str {
        match self {
            Operator::Add => "add",
            Operator::Sub => "sub",
            Operator::Mul => "mul",
            Operator::Div => "div",
            Operator::Mod => "rem",
            Operator::Pow => "pow",
            Operator::Neg => "neg",
            Operator::Eq => "eq",
            Operator::NotEq => "ne",
            Operator::Lt => "lt",
            Operator::Lte => "le",
            Operator::Gt => "gt",
            Operator::Gte => "ge",
            Operator::BitAnd => "bitand",
            Operator::BitOr => "bitor",
            Operator::BitXor => "bitxor",
            Operator::BitNot => "bitnot",
            Operator::Shl => "shl",
            Operator::Shr => "shr",
            Operator::And => "and",
            Operator::Or => "or",
            Operator::Not => "not",
            Operator::Index => "index",
            Operator::IndexMut => "index_mut",
            Operator::Call => "call",
            Operator::Deref => "deref",
            Operator::DerefMut => "deref_mut",
            Operator::Range => "range",
        }
    }

    /// Check if this is a binary operator
    pub fn is_binary(&self) -> bool {
        matches!(
            self,
            Operator::Add
                | Operator::Sub
                | Operator::Mul
                | Operator::Div
                | Operator::Mod
                | Operator::Pow
                | Operator::Eq
                | Operator::NotEq
                | Operator::Lt
                | Operator::Lte
                | Operator::Gt
                | Operator::Gte
                | Operator::BitAnd
                | Operator::BitOr
                | Operator::BitXor
                | Operator::Shl
                | Operator::Shr
                | Operator::Index
                | Operator::Range
        )
    }

    /// Check if this is a unary operator
    pub fn is_unary(&self) -> bool {
        matches!(
            self,
            Operator::Neg
                | Operator::BitNot
                | Operator::Not
                | Operator::Deref
                | Operator::DerefMut
        )
    }
}

/// Represents an overloaded operator implementation
#[derive(Debug, Clone)]
pub struct OperatorOverload {
    pub operator: Operator,
    pub impl_type: Type,
    pub rhs_type: Option<Type>, // None for unary operators
    pub return_type: Type,
    pub method_name: String,
}

/// Registry of operator overloads for a type
#[derive(Debug, Clone)]
pub struct OperatorRegistry {
    overloads: HashMap<(Type, Operator), OperatorOverload>,
}

impl OperatorRegistry {
    pub fn new() -> Self {
        OperatorRegistry {
            overloads: HashMap::new(),
        }
    }

    /// Register an operator overload for a type
    pub fn register(
        &mut self,
        ty: Type,
        op: Operator,
        rhs_type: Option<Type>,
        return_type: Type,
    ) {
        let method_name = op.method_name().to_string();
        self.overloads.insert(
            (ty.clone(), op),
            OperatorOverload {
                operator: op,
                impl_type: ty,
                rhs_type,
                return_type,
                method_name,
            },
        );
    }

    /// Look up an operator overload
    pub fn lookup(&self, ty: &Type, op: Operator) -> Option<&OperatorOverload> {
        self.overloads.get(&(ty.clone(), op))
    }

    /// Get all overloads for a type
    pub fn overloads_for_type(&self, ty: &Type) -> Vec<&OperatorOverload> {
        self.overloads
            .values()
            .filter(|o| o.impl_type == *ty)
            .collect()
    }
}

/// Syntax sugar for operator overloading
pub mod syntax {
    use super::*;

    /// Convert operator symbol to Operator enum
    pub fn parse_operator(sym: &str) -> Option<Operator> {
        match sym {
            "+" => Some(Operator::Add),
            "-" => Some(Operator::Sub),
            "*" => Some(Operator::Mul),
            "/" => Some(Operator::Div),
            "%" => Some(Operator::Mod),
            "**" => Some(Operator::Pow),
            "==" => Some(Operator::Eq),
            "!=" => Some(Operator::NotEq),
            "<" => Some(Operator::Lt),
            "<=" => Some(Operator::Lte),
            ">" => Some(Operator::Gt),
            ">=" => Some(Operator::Gte),
            "&" => Some(Operator::BitAnd),
            "|" => Some(Operator::BitOr),
            "^" => Some(Operator::BitXor),
            "~" => Some(Operator::BitNot),
            "<<" => Some(Operator::Shl),
            ">>" => Some(Operator::Shr),
            "!" => Some(Operator::Not),
            "[]" => Some(Operator::Index),
            "()" => Some(Operator::Call),
            ".." => Some(Operator::Range),
            _ => None,
        }
    }

    /// Convert Operator enum back to symbol
    pub fn operator_symbol(op: Operator) -> &'static str {
        match op {
            Operator::Add => "+",
            Operator::Sub => "-",
            Operator::Mul => "*",
            Operator::Div => "/",
            Operator::Mod => "%",
            Operator::Pow => "**",
            Operator::Neg => "-",
            Operator::Eq => "==",
            Operator::NotEq => "!=",
            Operator::Lt => "<",
            Operator::Lte => "<=",
            Operator::Gt => ">",
            Operator::Gte => ">=",
            Operator::BitAnd => "&",
            Operator::BitOr => "|",
            Operator::BitXor => "^",
            Operator::BitNot => "~",
            Operator::Shl => "<<",
            Operator::Shr => ">>",
            Operator::And => "&&",
            Operator::Or => "||",
            Operator::Not => "!",
            Operator::Index => "[]",
            Operator::IndexMut => "[]mut",
            Operator::Call => "()",
            Operator::Deref => "*",
            Operator::DerefMut => "*mut",
            Operator::Range => "..",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operator_traits() {
        assert_eq!(Operator::Add.trait_name(), "Add");
        assert_eq!(Operator::Sub.trait_name(), "Sub");
        assert!(Operator::Add.is_binary());
        assert!(Operator::Neg.is_unary());
    }

    #[test]
    fn test_operator_registry() {
        let mut reg = OperatorRegistry::new();
        let ty = Type::Struct("Vec2".to_string());

        reg.register(ty.clone(), Operator::Add, Some(ty.clone()), ty.clone());
        assert!(reg.lookup(&ty, Operator::Add).is_some());
        assert!(reg.lookup(&ty, Operator::Sub).is_none());
    }

    #[test]
    fn test_parse_operator() {
        use syntax::parse_operator;
        assert_eq!(parse_operator("+"), Some(Operator::Add));
        assert_eq!(parse_operator("=="), Some(Operator::Eq));
        assert_eq!(parse_operator("invalid"), None);
    }
}
