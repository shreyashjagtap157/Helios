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

//! MIR (Mid-level IR) Module
//!
//! Provides a lowered representation from AST for:
//! - Correct borrow checking of conditional moves
//! - Loop moves and NLL (Non-Lexical Lifetimes)
//! - Optimization passes
//! - Clean LLVM/native codegen

pub mod lower;
pub mod pretty;
pub mod place;
pub mod rvalue;
pub mod statement;

use std::collections::HashMap;

/// A MIR BasicBlock with explicit terminators
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub name: String,
    pub statements: Vec<Statement>,
    pub terminator: Terminator,
}

/// MIR Statements (non-terminator instructions)
#[derive(Debug, Clone)]
pub enum Statement {
    Assign(Place, Rvalue),
    FakeRead(FakeReadMode),
    StorageLive(Place),
    StorageDead(Place),
    Drop(Place),
}

/// Terminator instructions (control flow endings)
#[derive(Debug, Clone)]
pub enum Terminator {
    Goto(BasicBlockRef),
    SwitchInt {
        discr: Place,
        switch_ty: Type,
        cases: Vec<(i128, BasicBlockRef)>,
        otherwise: BasicBlockRef,
    },
    Return(Place),
    Call {
        func: String,
        args: Vec<Place>,
        destination: Option<(Place, BasicBlockRef)>,
    },
    Assert {
        condition: Place,
        expected: bool,
        target: BasicBlockRef,
    },
}

/// A memory location: `_1`, `_1.field`, `(*_1)`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Place {
    pub local: LocalVar,
    pub projection: Vec<Projection>,
}

/// Local variable index
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalVar(usize);

/// Projection into a place
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Projection {
    Field(usize),
    Deref,
    Index(Place),
    Subslice { start: usize, end: usize },
}

/// Rvalues: computations that produce values
#[derive(Debug, Clone)]
pub enum Rvalue {
    Use(Place),
    BinaryOp(BinOp, Place, Place),
    UnaryOp(UnOp, Place),
    CheckedBinaryOp(BinOp, Place, Place),
    Aggregate(AggregateKind, Vec<Place>),
    FunctionCall(String, Vec<Place>),
    Len(Place),
    Ref(Place, BorrowKind),
    AddressOf(Place),
}

/// Binary operators
#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    BitAnd, BitOr, BitXor,
    Shl, Shr,
    Eq, Ne, Lt, Le, Gt, Ge,
    Offset,
}

/// Unary operators
#[derive(Debug, Clone, Copy)]
pub enum UnOp {
    Not, Neg, BitNot,
}

/// Borrow kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorrowKind {
    Shared,
    Mutable { allow_two_phase_borrow: bool },
}

/// Aggregate construction
#[derive(Debug, Clone)]
pub enum AggregateKind {
    Tuple,
    Array,
    Struct,
}

/// Fake read mode for borrows
#[derive(Debug, Clone, Copy)]
pub enum FakeReadMode {
    ForInspect,
    ForMatch,
}

/// Reference to a basic block
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BasicBlockRef(usize);

/// MIR Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Bool,
    Char,
    I8, I16, I32, I64, I128, Isize,
    U8, U16, U32, U64, U128, Usize,
    F32, F64,
    Str,
    Ptr,
    Never,
    Tuple,
    Array,
    Slice,
    Struct,
}

impl Type {
    pub fn is_integral(&self) -> bool {
        matches!(self, Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::I128 | Type::Isize | Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::U128 | Type::Usize | Type::Bool | Type::Char)
    }
}

/// MIR Module
#[derive(Debug, Clone)]
pub struct MirModule {
    pub name: String,
    pub basic_blocks: HashMap<BasicBlockRef, BasicBlock>,
    pub locals: HashMap<LocalVar, Type>,
    pub var_names: HashMap<LocalVar, String>,
}

impl MirModule {
    pub fn new(name: String) -> Self {
        MirModule {
            name,
            basic_blocks: HashMap::new(),
            locals: HashMap::new(),
            var_names: HashMap::new(),
        }
    }

    pub fn new_basic_block(&mut self, name: &str) -> BasicBlockRef {
        let idx = self.basic_blocks.len();
        let ref_ = BasicBlockRef(idx);
        self.basic_blocks.insert(ref_, BasicBlock {
            name: name.to_string(),
            statements: Vec::new(),
            terminator: Terminator::Goto(BasicBlockRef(idx + 1)), // placeholder
        });
        ref_
    }

    pub fn push_local(&mut self, name: &str, ty: Type) -> LocalVar {
        let idx = self.locals.len();
        let local = LocalVar(idx);
        self.locals.insert(local, ty.clone());
        self.var_names.insert(local, name.to_string());
        local
    }
}

impl LocalVar {
    pub fn new(index: usize) -> Self {
        LocalVar(index)
    }
}

impl BasicBlockRef {
    pub fn new(index: usize) -> Self {
        BasicBlockRef(index)
    }
}

impl Place {
    pub fn new(local: LocalVar) -> Self {
        Place {
            local,
            projection: Vec::new(),
        }
    }

    pub fn field(self, field: usize) -> Self {
        let mut p = self;
        p.projection.push(Projection::Field(field));
        p
    }
}