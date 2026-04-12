//! MIR Statement definitions

use super::{FakeReadMode, Place, Rvalue};

/// A MIR statement (non-terminator instruction)
#[derive(Debug, Clone)]
pub enum Statement {
    /// assignment: place = rvalue
    Assign(Place, Rvalue),
    /// FakeRead is used to simulate a read of a place for borrow checking
    FakeRead(FakeReadMode),
    /// StorageLive marks the start of a variable's lifetime
    StorageLive(Place),
    /// StorageLive marks the end of a variable's lifetime
    StorageDead(Place),
    /// Drop (deallocate) a value
    Drop(Place),
    /// Debug marker for the compiler
    DebugMarker(String),
}

impl Statement {
    /// Get all places used (read or written) in this statement
    pub fn places(&self) -> Vec<Place> {
        match self {
            Statement::Assign(p, r) => {
                let mut places = vec![p.clone()];
                places.extend(r.places());
                places
            }
            Statement::FakeRead(mode) => {
                // FakeRead doesn't actually read, but we track the place for analysis
                vec![]
            }
            Statement::StorageLive(p) => vec![p.clone()],
            Statement::StorageDead(p) => vec![p.clone()],
            Statement::Drop(p) => vec![p.clone()],
            Statement::DebugMarker(_) => vec![],
        }
    }

    /// Check if this statement contains a function call
    pub fn has_call(&self) -> bool {
        match self {
            Statement::Assign(_, r) => r.has_function_call(),
            _ => false,
        }
    }
}
