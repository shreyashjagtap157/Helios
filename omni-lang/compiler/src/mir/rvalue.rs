//! MIR Rvalue definitions

use super::{BinOp, BorrowKind, UnOp};

/// Rvalues: computations that produce values
#[derive(Debug, Clone)]
pub enum Rvalue {
    /// Simple use of a place
    Use(super::Place),
    /// Binary operation
    BinaryOp(BinOp, super::Place, super::Place),
    /// Unary operation
    UnaryOp(UnOp, super::Place),
    /// Checked binary operation (for overflow/wrap handling)
    CheckedBinaryOp(BinOp, super::Place, super::Place),
    /// Aggregate (tuple/array/struct) construction
    Aggregate(AggregateKind, Vec<super::Place>),
    /// Function call
    FunctionCall(String, Vec<super::Place>),
    /// Length of a slice/array
    Len(super::Place),
    /// Reference to a place (borrow)
    Ref(super::Place, BorrowKind),
    /// Address of a place (raw pointer)
    AddressOf(super::Place),
}

/// Aggregate construction kinds
#[derive(Debug, Clone)]
pub enum AggregateKind {
    Tuple,
    Array,
    Struct,
}

impl Rvalue {
    /// Get all places used in this rvalue
    pub fn places(&self) -> Vec<super::Place> {
        match self {
            Rvalue::Use(p) => vec![p.clone()],
            Rvalue::BinaryOp(_, a, b) => vec![a.clone(), b.clone()],
            Rvalue::UnaryOp(_, p) => vec![p.clone()],
            Rvalue::CheckedBinaryOp(_, a, b) => vec![a.clone(), b.clone()],
            Rvalue::Aggregate(_, places) => places.clone(),
            Rvalue::FunctionCall(_, args) => args.clone(),
            Rvalue::Len(p) => vec![p.clone()],
            Rvalue::Ref(p, _) => vec![p.clone()],
            Rvalue::AddressOf(p) => vec![p.clone()],
        }
    }

    /// Check if this rvalue contains a function call
    pub fn has_function_call(&self) -> bool {
        matches!(self, Rvalue::FunctionCall(..))
    }

    /// Get the function name if this is a function call
    pub fn function_name(&self) -> Option<&str> {
        match self {
            Rvalue::FunctionCall(name, _) => Some(name),
            _ => None,
        }
    }
}
