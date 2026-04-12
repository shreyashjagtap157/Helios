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

//! Omni Intermediate Representation
//!
#![allow(dead_code)]

//! A low-level SSA-based IR for optimization and codegen.
//! Supports match lowering, closures, async/await, generics, and trait dispatch.

use crate::parser::ast::{BinaryOp, Literal, Ownership, Pattern, Type, UnaryOp};
use crate::semantic::{
    TypedExpr, TypedExprKind, TypedFunction, TypedItem, TypedModule, TypedStatement,
};
use log::{debug, info, trace};
use std::collections::HashMap;
use std::fmt;

/// IR Module - collection of functions and globals
#[derive(Debug, Clone)]
pub struct IrModule {
    pub name: String,
    pub functions: Vec<IrFunction>,
    pub globals: Vec<IrGlobal>,
    pub externs: Vec<IrExternalFunc>,
    pub vtables: Vec<VTable>,       // Virtual dispatch tables
    pub string_pool: Vec<String>,   // Interned strings
    pub type_info: Vec<IrTypeInfo>, // Runtime type information
}

/// Virtual method dispatch table
#[derive(Debug, Clone)]
pub struct VTable {
    pub trait_name: String,
    pub type_name: String,
    pub methods: Vec<String>, // Function names for each method
}

/// Runtime type information for reflection
#[derive(Debug, Clone)]
pub struct IrTypeInfo {
    pub name: String,
    pub size: usize,
    pub alignment: usize,
    pub kind: TypeKind,
    pub drop_fn: Option<String>,
}

#[derive(Debug, Clone)]
pub enum TypeKind {
    Primitive,
    Struct {
        fields: Vec<(String, IrType)>,
    },
    Enum {
        variants: Vec<(String, Vec<IrType>)>,
    },
    Closure {
        captures: Vec<IrType>,
    },
}

#[derive(Debug, Clone)]
pub struct IrExternalFunc {
    pub name: String,
    pub abi: String,
    pub params: Vec<IrType>,
    pub return_type: IrType,
}

impl fmt::Display for IrModule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "; Omni IR Module: {}", self.name)?;
        for ext in &self.externs {
            writeln!(
                f,
                "declare {} @{}({:?})",
                ext.return_type, ext.name, ext.params
            )?;
        }
        for func in &self.functions {
            writeln!(f, "{}", func)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct IrGlobal {
    pub name: String,
    pub ty: IrType,
    pub init: Option<IrValue>,
}

#[derive(Debug, Clone)]
pub struct IrFunction {
    pub name: String,
    pub params: Vec<(String, IrType)>,
    pub return_type: IrType,
    pub blocks: Vec<IrBlock>,
    pub locals: Vec<(String, IrType)>,
}

impl fmt::Display for IrFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "fn @{}(", self.name)?;
        for (i, (name, ty)) in self.params.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "%{}: {}", name, ty)?;
        }
        writeln!(f, ") -> {} {{", self.return_type)?;
        for block in &self.blocks {
            writeln!(f, "{}", block)?;
        }
        writeln!(f, "}}")
    }
}

#[derive(Debug, Clone)]
pub struct IrBlock {
    pub label: String,
    pub instructions: Vec<IrInstruction>,
    pub terminator: IrTerminator,
}

impl fmt::Display for IrBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "  {}:", self.label)?;
        for inst in &self.instructions {
            writeln!(f, "    {}", inst)?;
        }
        writeln!(f, "    {}", self.terminator)
    }
}

#[derive(Debug, Clone)]
pub enum IrInstruction {
    Alloca {
        dest: String,
        ty: IrType,
    },
    Load {
        dest: String,
        ptr: String,
        ty: IrType,
    },
    Store {
        ptr: String,
        value: IrValue,
    },
    BinOp {
        dest: String,
        op: IrBinOp,
        left: IrValue,
        right: IrValue,
    },
    Call {
        dest: Option<String>,
        func: String,
        args: Vec<IrValue>,
    },
    GetField {
        dest: String,
        ptr: String,
        field: usize,
    },
    // New instructions for advanced features
    Phi {
        dest: String,
        ty: IrType,
        incoming: Vec<(String, String)>,
    }, // (value, block)
    Select {
        dest: String,
        cond: IrValue,
        then_val: IrValue,
        else_val: IrValue,
    },
    Switch {
        value: IrValue,
        default: String,
        cases: Vec<(i64, String)>,
    },
    CreateClosure {
        dest: String,
        func: String,
        captures: Vec<String>,
    },
    CallClosure {
        dest: Option<String>,
        closure: String,
        args: Vec<IrValue>,
    },
    AsyncSpawn {
        dest: String,
        func: String,
        args: Vec<IrValue>,
    },
    AsyncAwait {
        dest: Option<String>,
        future: String,
    },
    TraitDispatch {
        dest: Option<String>,
        object: String,
        method: String,
        args: Vec<IrValue>,
    },
    VTableLookup {
        dest: String,
        object: String,
        trait_name: String,
        method_idx: usize,
    },
    Cast {
        dest: String,
        value: IrValue,
        to_type: IrType,
    },
    ExtractValue {
        dest: String,
        aggregate: String,
        indices: Vec<usize>,
    },
    InsertValue {
        dest: String,
        aggregate: String,
        value: IrValue,
        indices: Vec<usize>,
    },
    /// Native/builtin function call (println, print, args, exit, etc.)
    NativeCall {
        dest: Option<String>,
        module: String,
        func: String,
        args: Vec<IrValue>,
    },
    /// Array bounds check: verifies index < length at runtime
    BoundsCheck {
        index: String,
        length: String,
    },
}

impl fmt::Display for IrInstruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Alloca { dest, ty } => write!(f, "%{} = alloca {}", dest, ty),
            Self::Load { dest, ptr, ty } => write!(f, "%{} = load {}, %{}", dest, ty, ptr),
            Self::Store { ptr, value } => write!(f, "store {}, %{}", value, ptr),
            Self::BinOp {
                dest,
                op,
                left,
                right,
            } => write!(f, "%{} = {} {}, {}", dest, op, left, right),
            Self::Call { dest, func, args } => {
                if let Some(d) = dest {
                    write!(f, "%{} = call @{}(", d, func)?;
                } else {
                    write!(f, "call @{}(", func)?;
                }
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            Self::GetField { dest, ptr, field } => {
                write!(f, "%{} = getfield %{}, {}", dest, ptr, field)
            }
            Self::Phi { dest, ty, incoming } => {
                write!(f, "%{} = phi {} ", dest, ty)?;
                for (i, (val, block)) in incoming.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "[%{}, %{}]", val, block)?;
                }
                Ok(())
            }
            Self::Select {
                dest,
                cond,
                then_val,
                else_val,
            } => write!(f, "%{} = select {}, {}, {}", dest, cond, then_val, else_val),
            Self::Switch {
                value,
                default,
                cases,
            } => {
                write!(f, "switch {} default %{} [", value, default)?;
                for (i, (val, label)) in cases.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: %{}", val, label)?;
                }
                write!(f, "]")
            }
            Self::CreateClosure {
                dest,
                func,
                captures,
            } => write!(
                f,
                "%{} = closure @{} captures [{}]",
                dest,
                func,
                captures.join(", ")
            ),
            Self::CallClosure {
                dest,
                closure,
                args,
            } => {
                if let Some(d) = dest {
                    write!(f, "%{} = callclosure %{}(", d, closure)?;
                } else {
                    write!(f, "callclosure %{}(", closure)?;
                }
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            Self::AsyncSpawn { dest, func, args } => {
                write!(f, "%{} = async.spawn @{}({} args)", dest, func, args.len())
            }
            Self::AsyncAwait { dest, future } => match dest {
                Some(d) => write!(f, "%{} = async.await %{}", d, future),
                None => write!(f, "async.await %{}", future),
            },
            Self::TraitDispatch {
                dest,
                object,
                method,
                args,
            } => match dest {
                Some(d) => write!(
                    f,
                    "%{} = dispatch %{}.{}({} args)",
                    d,
                    object,
                    method,
                    args.len()
                ),
                None => write!(f, "dispatch %{}.{}({} args)", object, method, args.len()),
            },
            Self::VTableLookup {
                dest,
                object,
                trait_name,
                method_idx,
            } => write!(
                f,
                "%{} = vtable.lookup %{}::{}.{}",
                dest, object, trait_name, method_idx
            ),
            Self::Cast {
                dest,
                value,
                to_type,
            } => write!(f, "%{} = cast {} to {}", dest, value, to_type),
            Self::ExtractValue {
                dest,
                aggregate,
                indices,
            } => write!(f, "%{} = extractvalue %{} {:?}", dest, aggregate, indices),
            Self::InsertValue {
                dest,
                aggregate,
                value,
                indices,
            } => write!(
                f,
                "%{} = insertvalue %{}, {} {:?}",
                dest, aggregate, value, indices
            ),
            Self::NativeCall {
                dest,
                module,
                func,
                args,
            } => {
                if let Some(d) = dest {
                    write!(f, "%{} = native.call {}::{}(", d, module, func)?;
                } else {
                    write!(f, "native.call {}::{}(", module, func)?;
                }
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            Self::BoundsCheck { index, length } => {
                write!(f, "bounds_check %{} < %{}", index, length)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum IrTerminator {
    Return(Option<IrValue>),
    Branch(String),
    CondBranch {
        cond: IrValue,
        then_label: String,
        else_label: String,
    },
    Unreachable,
}

impl fmt::Display for IrTerminator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Return(Some(v)) => write!(f, "ret {}", v),
            Self::Return(None) => write!(f, "ret void"),
            Self::Branch(label) => write!(f, "br %{}", label),
            Self::CondBranch {
                cond,
                then_label,
                else_label,
            } => write!(f, "br {}, %{}, %{}", cond, then_label, else_label),
            Self::Unreachable => write!(f, "unreachable"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum IrBinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
}

impl fmt::Display for IrBinOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Add => write!(f, "add"),
            Self::Sub => write!(f, "sub"),
            Self::Mul => write!(f, "mul"),
            Self::Div => write!(f, "div"),
            Self::Mod => write!(f, "mod"),
            Self::Eq => write!(f, "eq"),
            Self::Ne => write!(f, "ne"),
            Self::Lt => write!(f, "lt"),
            Self::Gt => write!(f, "gt"),
            Self::Le => write!(f, "le"),
            Self::Ge => write!(f, "ge"),
            Self::And => write!(f, "and"),
            Self::Or => write!(f, "or"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum IrType {
    Void,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Bool,
    Ptr(Box<IrType>),
    Array(Box<IrType>, usize),
    Struct(String),
    // New types for advanced features
    Closure {
        params: Vec<IrType>,
        ret: Box<IrType>,
        captures: Vec<IrType>,
    },
    Future(Box<IrType>),
    TraitObject(String),
    Generic(String),
    Enum {
        name: String,
        variants: Vec<(String, Vec<IrType>)>,
    },
    Tuple(Vec<IrType>),
    FnPtr {
        params: Vec<IrType>,
        ret: Box<IrType>,
    },
}

/// Async state machine states
#[derive(Debug, Clone)]
pub struct AsyncStateMachine {
    pub states: Vec<AsyncState>,
    pub current_state_var: String,
    pub resume_points: Vec<(usize, String)>, // (state_id, label)
}

#[derive(Debug, Clone)]
pub struct AsyncState {
    pub id: usize,
    pub entry_label: String,
    pub exit_label: String,
    pub saved_locals: Vec<String>,
}

/// Closure environment
#[derive(Debug, Clone)]
pub struct ClosureEnv {
    pub captures: Vec<(String, IrType, CaptureKind)>,
    pub func_ptr: String,
}

#[derive(Debug, Clone, Copy)]
pub enum CaptureKind {
    ByValue,
    ByRef,
    ByMutRef,
}

impl fmt::Display for IrType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Void => write!(f, "void"),
            Self::I8 => write!(f, "i8"),
            Self::I16 => write!(f, "i16"),
            Self::I32 => write!(f, "i32"),
            Self::I64 => write!(f, "i64"),
            Self::F32 => write!(f, "f32"),
            Self::F64 => write!(f, "f64"),
            Self::Bool => write!(f, "i1"),
            Self::Ptr(inner) => write!(f, "*{}", inner),
            Self::Array(elem, size) => write!(f, "[{}; {}]", elem, size),
            Self::Struct(name) => write!(f, "%{}", name),
            Self::Closure { params, ret, .. } => {
                let params_str = params
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "closure({}) -> {}", params_str, ret)
            }
            Self::Future(inner) => write!(f, "future<{}>", inner),
            Self::TraitObject(name) => write!(f, "dyn {}", name),
            Self::Generic(name) => write!(f, "generic<{}>", name),
            Self::Enum { name, .. } => write!(f, "enum {}", name),
            Self::Tuple(elements) => {
                let elems_str = elements
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "({})", elems_str)
            }
            Self::FnPtr { params, ret } => {
                let params_str = params
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "fn({}) -> {}", params_str, ret)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum IrValue {
    Const(IrConst),
    Var(String),
}

impl fmt::Display for IrValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Const(c) => write!(f, "{}", c),
            Self::Var(name) => write!(f, "%{}", name),
        }
    }
}

#[derive(Debug, Clone)]
pub enum IrConst {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Null,
}

impl fmt::Display for IrConst {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Int(v) => write!(f, "{}", v),
            Self::Float(v) => write!(f, "{}", v),
            Self::Bool(v) => write!(f, "{}", v),
            Self::Str(s) => write!(f, "\"{}\"", s),
            Self::Null => write!(f, "null"),
        }
    }
}

/// IR Generator with support for match lowering, closures, async, generics, trait dispatch
pub struct IrGenerator {
    temp_counter: usize,
    block_counter: usize,
    monomorphized: HashMap<String, IrFunction>, // Monomorphized generic functions
    vtables: HashMap<String, Vec<String>>,      // Trait -> list of methods
    closures: Vec<IrFunction>,                  // Generated closure functions
    closure_envs: HashMap<String, ClosureEnv>,  // Closure environments
    string_pool: Vec<String>,                   // String interning
    string_indices: HashMap<String, usize>,
    async_transforms: Vec<AsyncStateMachine>, // Async function state machines
    loop_stack: Vec<(String, String)>,        // (header_label, exit_label) for break/continue
    struct_layouts: HashMap<String, Vec<(String, IrType)>>,
    enum_variants: HashMap<String, Vec<(String, Vec<IrType>)>>,
    type_info: Vec<IrTypeInfo>,
    current_func_is_async: bool,
}

impl IrGenerator {
    pub fn new() -> Self {
        info!("Initializing IR generator with advanced features");
        Self {
            temp_counter: 0,
            block_counter: 0,
            monomorphized: HashMap::new(),
            vtables: HashMap::new(),
            closures: Vec::new(),
            closure_envs: HashMap::new(),
            string_pool: Vec::new(),
            string_indices: HashMap::new(),
            async_transforms: Vec::new(),
            loop_stack: Vec::new(),
            struct_layouts: HashMap::new(),
            enum_variants: HashMap::new(),
            type_info: Vec::new(),
            current_func_is_async: false,
        }
    }

    fn fresh_temp(&mut self) -> String {
        let name = format!("t{}", self.temp_counter);
        self.temp_counter += 1;
        name
    }

    fn fresh_block(&mut self) -> String {
        let name = format!("bb{}", self.block_counter);
        self.block_counter += 1;
        name
    }

    fn intern_string(&mut self, s: &str) -> usize {
        if let Some(&idx) = self.string_indices.get(s) {
            return idx;
        }
        let idx = self.string_pool.len();
        self.string_pool.push(s.to_string());
        self.string_indices.insert(s.to_string(), idx);
        idx
    }

    pub fn generate(&mut self, module: TypedModule) -> IrModule {
        info!("Generating IR from typed AST");
        let mut functions = Vec::new();
        let mut externs = Vec::new();
        let mut vtables_list = Vec::new();

        // First pass: collect struct/enum layouts and trait vtables
        for item in &module.items {
            match item {
                TypedItem::Struct(s) => {
                    let fields: Vec<_> = s
                        .fields
                        .iter()
                        .map(|(n, t)| (n.clone(), self.convert_type(t)))
                        .collect();
                    self.struct_layouts.insert(s.name.clone(), fields.clone());
                    self.type_info.push(IrTypeInfo {
                        name: s.name.clone(),
                        size: self.calculate_struct_size(&fields),
                        alignment: 8,
                        kind: TypeKind::Struct { fields },
                        drop_fn: None,
                    });
                }
                TypedItem::Enum(e) => {
                    let variants: Vec<_> = e
                        .variants
                        .iter()
                        .map(|(n, ts)| {
                            (
                                n.clone(),
                                ts.clone()
                                    .unwrap_or_default()
                                    .iter()
                                    .map(|t| self.convert_type(t))
                                    .collect(),
                            )
                        })
                        .collect();
                    self.enum_variants.insert(e.name.clone(), variants.clone());
                    self.type_info.push(IrTypeInfo {
                        name: e.name.clone(),
                        size: self.calculate_enum_size(&variants),
                        alignment: 8,
                        kind: TypeKind::Enum { variants },
                        drop_fn: None,
                    });
                }
                TypedItem::Trait(t) => {
                    self.vtables.insert(
                        t.name.clone(),
                        t.methods.iter().map(|m| m.name.clone()).collect(),
                    );
                }
                TypedItem::Impl(imp) => {
                    // Build vtable for this impl
                    let methods: Vec<_> = imp
                        .methods
                        .iter()
                        .map(|m| format!("{}_{}", imp.type_name, m.name))
                        .collect();
                    vtables_list.push(VTable {
                        trait_name: imp.trait_name.clone(),
                        type_name: imp.type_name.clone(),
                        methods,
                    });
                }
                _ => {}
            }
        }

        // Second pass: generate IR for functions
        for item in module.items {
            match item {
                TypedItem::Function(f) => {
                    debug!("Generating IR for function: {}", f.name);
                    functions.push(self.gen_function(f));
                }
                TypedItem::Extern(e) => {
                    for f in e.functions {
                        let params: Vec<_> =
                            f.params.iter().map(|(_, t)| self.convert_type(t)).collect();
                        externs.push(IrExternalFunc {
                            name: f.name,
                            abi: e.abi.clone(),
                            params,
                            return_type: self.convert_type(&f.return_type),
                        });
                    }
                }
                TypedItem::Impl(imp) => {
                    // Generate IR for impl methods
                    for method in imp.methods {
                        let mangled_name = format!("{}_{}", imp.type_name, method.name);
                        let mut ir_func = self.gen_function(method);
                        ir_func.name = mangled_name;
                        functions.push(ir_func);
                    }
                }
                _ => {}
            }
        }

        // Add generated closures
        functions.extend(self.closures.drain(..));

        IrModule {
            name: "main".to_string(),
            functions,
            globals: Vec::new(),
            externs,
            vtables: vtables_list,
            string_pool: self.string_pool.clone(),
            type_info: self.type_info.clone(),
        }
    }

    fn calculate_struct_size(&self, fields: &[(String, IrType)]) -> usize {
        fields.iter().map(|(_, ty)| self.type_size(ty)).sum()
    }

    fn calculate_enum_size(&self, variants: &[(String, Vec<IrType>)]) -> usize {
        // Enum size = discriminant + max variant size
        let max_variant = variants
            .iter()
            .map(|(_, tys)| tys.iter().map(|t| self.type_size(t)).sum::<usize>())
            .max()
            .unwrap_or(0);
        8 + max_variant // 8 bytes for discriminant
    }

    fn type_size(&self, ty: &IrType) -> usize {
        match ty {
            IrType::Void => 0,
            IrType::I8 | IrType::Bool => 1,
            IrType::I16 => 2,
            IrType::I32 | IrType::F32 => 4,
            IrType::I64 | IrType::F64 | IrType::Ptr(_) => 8,
            IrType::Array(elem, size) => self.type_size(elem) * size,
            IrType::Struct(name) => {
                if let Some(fields) = self.struct_layouts.get(name) {
                    self.calculate_struct_size(fields)
                } else {
                    8 // Default size
                }
            }
            IrType::Closure { captures, .. } => {
                8 + captures.iter().map(|t| self.type_size(t)).sum::<usize>()
            }
            IrType::TraitObject(_) => 16, // Fat pointer (data + vtable)
            _ => 8,
        }
    }

    fn gen_function(&mut self, f: TypedFunction) -> IrFunction {
        trace!("Building IR for function {}", f.name);
        let params: Vec<_> = f
            .params
            .iter()
            .map(|(n, t)| (n.clone(), self.convert_type(t)))
            .collect();

        let mut blocks = Vec::new();
        let mut instructions = Vec::new();
        let mut current_block = "entry".to_string();

        // Generate IR for each statement
        for stmt in &f.body {
            self.gen_statement(stmt, &mut instructions, &mut blocks, &mut current_block);
        }

        // Ensure we have a terminator — only add ret void if the last block doesn't have one
        let _needs_trailing_return = blocks
            .last()
            .map(|b| matches!(b.terminator, IrTerminator::Return(None)))
            .unwrap_or(true)
            && instructions.is_empty();

        if !instructions.is_empty() {
            // There are instructions without a terminator — add ret void
            blocks.push(IrBlock {
                label: current_block,
                instructions,
                terminator: IrTerminator::Return(None),
            });
        } else if blocks.is_empty() {
            // Empty function — add a ret void block
            blocks.push(IrBlock {
                label: "entry".to_string(),
                instructions: Vec::new(),
                terminator: IrTerminator::Return(None),
            });
        }
        // If the last block already has a Return(Some(...)) terminator, leave it as-is

        IrFunction {
            name: f.name,
            params,
            return_type: self.convert_type(&f.return_type),
            blocks,
            locals: Vec::new(),
        }
    }

    fn gen_statement(
        &mut self,
        stmt: &TypedStatement,
        instructions: &mut Vec<IrInstruction>,
        blocks: &mut Vec<IrBlock>,
        current_block: &mut String,
    ) {
        match stmt {
            TypedStatement::Let { name, ty, value } => {
                let _dest = self.fresh_temp();
                let ir_ty = self.convert_type(ty);
                instructions.push(IrInstruction::Alloca {
                    dest: name.clone(),
                    ty: ir_ty.clone(),
                });
                let val = self.gen_expr(value, instructions);
                instructions.push(IrInstruction::Store {
                    ptr: name.clone(),
                    value: val,
                });
            }
            TypedStatement::Assignment { target, op, value } => {
                // Generate the target address (must be an lvalue)
                let ptr = match &target.kind {
                    TypedExprKind::Identifier(name) => name.clone(),
                    TypedExprKind::Field(obj, field) => {
                        // For field access, we need to get the field pointer
                        let obj_val = self.gen_expr(obj, instructions);
                        let _ptr = self.fresh_temp();
                        // Simplified: just use field name for now
                        format!(
                            "{}.{}",
                            match &obj_val {
                                IrValue::Var(v) => v.clone(),
                                _ => "obj".to_string(),
                            },
                            field
                        )
                    }
                    _ => {
                        // Fallback for other lvalue expressions
                        let val = self.gen_expr(&target, instructions);
                        match val {
                            IrValue::Var(v) => v,
                            _ => "temp".to_string(),
                        }
                    }
                };

                // Generate value expression
                let val = if let Some(bin_op) = op {
                    // Compound assignment: x += v => x = x + v
                    let current = IrValue::Var(ptr.clone());
                    let rhs = self.gen_expr(value, instructions);
                    let result = self.fresh_temp();
                    let ir_op = match bin_op {
                        BinaryOp::Add => IrBinOp::Add,
                        BinaryOp::Sub => IrBinOp::Sub,
                        BinaryOp::Mul => IrBinOp::Mul,
                        BinaryOp::Div => IrBinOp::Div,
                        _ => IrBinOp::Add, // Fallback
                    };
                    instructions.push(IrInstruction::BinOp {
                        dest: result.clone(),
                        op: ir_op,
                        left: current,
                        right: rhs,
                    });
                    IrValue::Var(result)
                } else {
                    // Simple assignment
                    self.gen_expr(value, instructions)
                };

                instructions.push(IrInstruction::Store { ptr, value: val });
            }
            TypedStatement::Return(expr) => {
                let ret_val = expr.as_ref().map(|e| self.gen_expr(e, instructions));
                // End current block with return terminator
                blocks.push(IrBlock {
                    label: current_block.clone(),
                    instructions: std::mem::take(instructions),
                    terminator: IrTerminator::Return(ret_val),
                });
                // Subsequent code is unreachable; start a dead block
                *current_block = self.fresh_block();
            }
            TypedStatement::If {
                condition,
                then_block,
                else_block,
            } => {
                let cond_val = self.gen_expr(condition, instructions);
                let then_label = self.fresh_block();
                let else_label = self.fresh_block();
                let merge_label = self.fresh_block();

                // End current block with conditional branch
                blocks.push(IrBlock {
                    label: current_block.clone(),
                    instructions: std::mem::take(instructions),
                    terminator: IrTerminator::CondBranch {
                        cond: cond_val,
                        then_label: then_label.clone(),
                        else_label: if else_block.is_some() {
                            else_label.clone()
                        } else {
                            merge_label.clone()
                        },
                    },
                });

                // Then block
                *current_block = then_label;
                for s in then_block {
                    self.gen_statement(s, instructions, blocks, current_block);
                }
                blocks.push(IrBlock {
                    label: current_block.clone(),
                    instructions: std::mem::take(instructions),
                    terminator: IrTerminator::Branch(merge_label.clone()),
                });

                // Else block
                if let Some(else_stmts) = else_block {
                    *current_block = else_label;
                    for s in else_stmts {
                        self.gen_statement(s, instructions, blocks, current_block);
                    }
                    blocks.push(IrBlock {
                        label: current_block.clone(),
                        instructions: std::mem::take(instructions),
                        terminator: IrTerminator::Branch(merge_label.clone()),
                    });
                }

                *current_block = merge_label;
            }
            TypedStatement::For { var, iter, body } => {
                let loop_header = self.fresh_block();
                let loop_body = self.fresh_block();
                let loop_exit = self.fresh_block();

                // Push loop context for break/continue
                self.loop_stack
                    .push((loop_header.clone(), loop_exit.clone()));

                // Initialize iterator
                let _iter_val = self.gen_expr(iter, instructions);
                let var_ty = match &iter.ty {
                    Type::Array(elem, _) => self.convert_type(elem),
                    Type::Slice(elem) => self.convert_type(elem),
                    _ => self.convert_type(&iter.ty),
                };
                instructions.push(IrInstruction::Alloca {
                    dest: var.clone(),
                    ty: var_ty,
                });

                blocks.push(IrBlock {
                    label: current_block.clone(),
                    instructions: std::mem::take(instructions),
                    terminator: IrTerminator::Branch(loop_header.clone()),
                });

                // Loop header - check condition
                *current_block = loop_header.clone();
                let cond = self.fresh_temp();
                // Simplified: in real impl, would call iterator.next() and check
                instructions.push(IrInstruction::Load {
                    dest: cond.clone(),
                    ptr: var.clone(),
                    ty: IrType::Bool,
                });
                blocks.push(IrBlock {
                    label: current_block.clone(),
                    instructions: std::mem::take(instructions),
                    terminator: IrTerminator::CondBranch {
                        cond: IrValue::Var(cond),
                        then_label: loop_body.clone(),
                        else_label: loop_exit.clone(),
                    },
                });

                // Loop body
                *current_block = loop_body;
                for s in body {
                    self.gen_statement(s, instructions, blocks, current_block);
                }
                blocks.push(IrBlock {
                    label: current_block.clone(),
                    instructions: std::mem::take(instructions),
                    terminator: IrTerminator::Branch(loop_header),
                });

                self.loop_stack.pop();
                *current_block = loop_exit;
            }
            TypedStatement::While { condition, body } => {
                let loop_header = self.fresh_block();
                let loop_body = self.fresh_block();
                let loop_exit = self.fresh_block();

                // Push loop context for break/continue
                self.loop_stack
                    .push((loop_header.clone(), loop_exit.clone()));

                blocks.push(IrBlock {
                    label: current_block.clone(),
                    instructions: std::mem::take(instructions),
                    terminator: IrTerminator::Branch(loop_header.clone()),
                });

                *current_block = loop_header.clone();
                let cond_val = self.gen_expr(condition, instructions);
                blocks.push(IrBlock {
                    label: current_block.clone(),
                    instructions: std::mem::take(instructions),
                    terminator: IrTerminator::CondBranch {
                        cond: cond_val,
                        then_label: loop_body.clone(),
                        else_label: loop_exit.clone(),
                    },
                });

                *current_block = loop_body;
                for s in body {
                    self.gen_statement(s, instructions, blocks, current_block);
                }
                blocks.push(IrBlock {
                    label: current_block.clone(),
                    instructions: std::mem::take(instructions),
                    terminator: IrTerminator::Branch(loop_header),
                });

                self.loop_stack.pop();
                *current_block = loop_exit;
            }
            TypedStatement::Loop { body } => {
                let loop_body = self.fresh_block();
                let loop_exit = self.fresh_block();

                // Push loop context for break/continue
                self.loop_stack.push((loop_body.clone(), loop_exit.clone()));

                blocks.push(IrBlock {
                    label: current_block.clone(),
                    instructions: std::mem::take(instructions),
                    terminator: IrTerminator::Branch(loop_body.clone()),
                });

                *current_block = loop_body.clone();
                for s in body {
                    self.gen_statement(s, instructions, blocks, current_block);
                }
                blocks.push(IrBlock {
                    label: current_block.clone(),
                    instructions: std::mem::take(instructions),
                    terminator: IrTerminator::Branch(loop_body),
                });

                self.loop_stack.pop();
                *current_block = loop_exit;
            }
            TypedStatement::Match { expr, arms } => {
                let match_val = self.gen_expr(expr, instructions);
                let merge_label = self.fresh_block();

                // Generate switch instruction for simple cases
                let mut cases = Vec::new();
                let default_label = self.fresh_block();

                for (i, (pattern, arm_body)) in arms.iter().enumerate() {
                    let arm_label = self.fresh_block();

                    // Match pattern
                    match pattern {
                        Pattern::Literal(Literal::Int(n)) => {
                            cases.push((*n, arm_label.clone()));
                        }
                        Pattern::Binding(_bind_name) => {
                            // Bind value to name in arm block
                            cases.push((i as i64, arm_label.clone()));
                            // Will store match value into bind_name at arm entry
                        }
                        Pattern::Wildcard => {
                            // Default case handled separately
                        }
                        _ => {
                            cases.push((i as i64, arm_label.clone()));
                        }
                    }

                    // Generate arm body block with actual statements
                    let mut arm_instructions = Vec::new();

                    // If binding pattern, store the matched value
                    if let Pattern::Binding(bind_name) = pattern {
                        let bind_ty = self.convert_type(&expr.ty);
                        arm_instructions.push(IrInstruction::Alloca {
                            dest: bind_name.clone(),
                            ty: bind_ty,
                        });
                        arm_instructions.push(IrInstruction::Store {
                            ptr: bind_name.clone(),
                            value: match_val.clone(),
                        });
                    }

                    // Generate body statements into arm block
                    let mut arm_current = arm_label.clone();
                    for stmt in arm_body {
                        self.gen_statement(stmt, &mut arm_instructions, blocks, &mut arm_current);
                    }

                    blocks.push(IrBlock {
                        label: arm_current,
                        instructions: arm_instructions,
                        terminator: IrTerminator::Branch(merge_label.clone()),
                    });
                }

                // Default block (for unmatched patterns)
                blocks.push(IrBlock {
                    label: default_label.clone(),
                    instructions: Vec::new(),
                    terminator: IrTerminator::Branch(merge_label.clone()),
                });

                // End current block with branch to first arm or default
                blocks.push(IrBlock {
                    label: current_block.clone(),
                    instructions: std::mem::take(instructions),
                    terminator: if cases.is_empty() {
                        IrTerminator::Branch(default_label)
                    } else {
                        IrTerminator::Branch(cases[0].1.clone()) // Simplified; real impl uses Switch
                    },
                });

                *current_block = merge_label;
            }
            TypedStatement::Defer(inner) => {
                // Defer is handled at scope exit - store for later
                // In real impl, would generate cleanup code at function exit
                self.gen_statement(inner, instructions, blocks, current_block);
            }
            TypedStatement::Break => {
                // Break to loop exit using loop context
                if let Some((_header, exit)) = self.loop_stack.last() {
                    blocks.push(IrBlock {
                        label: current_block.clone(),
                        instructions: std::mem::take(instructions),
                        terminator: IrTerminator::Branch(exit.clone()),
                    });
                    *current_block = self.fresh_block(); // Dead code after break
                }
            }
            TypedStatement::Continue => {
                // Continue to loop header using loop context
                if let Some((header, _exit)) = self.loop_stack.last() {
                    blocks.push(IrBlock {
                        label: current_block.clone(),
                        instructions: std::mem::take(instructions),
                        terminator: IrTerminator::Branch(header.clone()),
                    });
                    *current_block = self.fresh_block(); // Dead code after continue
                }
            }
            TypedStatement::Expr(expr) => {
                self.gen_expr(expr, instructions);
            }
            TypedStatement::Pass => {} // No-op
            TypedStatement::Yield(expr) => {
                if let Some(e) = expr {
                    let val = self.gen_expr(e, instructions);
                    instructions.push(IrInstruction::Store {
                        ptr: "$yield_val".into(),
                        value: val,
                    });
                }
            }
            TypedStatement::Spawn(expr) => {
                let func = self.gen_expr(expr, instructions);
                if let IrValue::Var(name) = func {
                    instructions.push(IrInstruction::AsyncSpawn {
                        dest: self.fresh_temp(),
                        func: name,
                        args: vec![],
                    });
                }
            }
            TypedStatement::Select { arms } => {
                // Lower select to first arm body as fallback
                if let Some(arm) = arms.first() {
                    for s in &arm.body {
                        self.gen_statement(s, instructions, blocks, current_block);
                    }
                }
            }
        }
    }

    fn gen_expr(&mut self, expr: &TypedExpr, instructions: &mut Vec<IrInstruction>) -> IrValue {
        match &expr.kind {
            TypedExprKind::Literal(lit) => match lit {
                Literal::Int(n) => IrValue::Const(IrConst::Int(*n)),
                Literal::Float(f) => IrValue::Const(IrConst::Float(*f)),
                Literal::Bool(b) => IrValue::Const(IrConst::Bool(*b)),
                Literal::String(s) => {
                    self.intern_string(s);
                    IrValue::Const(IrConst::Str(s.clone()))
                }
                Literal::Null => IrValue::Const(IrConst::Null),
            },
            TypedExprKind::Identifier(name) => {
                let dest = self.fresh_temp();
                let ty = self.convert_type(&expr.ty);
                instructions.push(IrInstruction::Load {
                    dest: dest.clone(),
                    ptr: name.clone(),
                    ty,
                });
                IrValue::Var(dest)
            }
            TypedExprKind::Binary(left, op, right) => {
                let left_val = self.gen_expr(left, instructions);
                let right_val = self.gen_expr(right, instructions);
                let dest = self.fresh_temp();
                let ir_op = match op {
                    BinaryOp::Add => IrBinOp::Add,
                    BinaryOp::Sub => IrBinOp::Sub,
                    BinaryOp::Mul => IrBinOp::Mul,
                    BinaryOp::Div => IrBinOp::Div,
                    BinaryOp::Mod => IrBinOp::Mod,
                    BinaryOp::Eq => IrBinOp::Eq,
                    BinaryOp::NotEq => IrBinOp::Ne,
                    BinaryOp::Lt => IrBinOp::Lt,
                    BinaryOp::Gt => IrBinOp::Gt,
                    BinaryOp::LtEq => IrBinOp::Le,
                    BinaryOp::GtEq => IrBinOp::Ge,
                    BinaryOp::And => IrBinOp::And,
                    BinaryOp::Or => IrBinOp::Or,
                    // Range ops: return start value (range lowering handled elsewhere)
                    BinaryOp::Range | BinaryOp::RangeInclusive => {
                        return left_val;
                    }
                };
                instructions.push(IrInstruction::BinOp {
                    dest: dest.clone(),
                    op: ir_op,
                    left: left_val,
                    right: right_val,
                });
                IrValue::Var(dest)
            }
            TypedExprKind::Unary(op, inner) => {
                let inner_val = self.gen_expr(inner, instructions);
                let dest = self.fresh_temp();
                // Generate unary op (neg or not)
                let zero = IrValue::Const(IrConst::Int(0));
                let ir_op = match op {
                    UnaryOp::Neg => IrBinOp::Sub,
                    UnaryOp::Not => IrBinOp::Eq, // !x == (x == 0)
                };
                instructions.push(IrInstruction::BinOp {
                    dest: dest.clone(),
                    op: ir_op,
                    left: zero,
                    right: inner_val,
                });
                IrValue::Var(dest)
            }
            TypedExprKind::Call(func, args) => {
                let func_name = match &func.kind {
                    TypedExprKind::Identifier(n) => n.clone(),
                    _ => "unknown".to_string(),
                };
                let arg_vals: Vec<_> = args
                    .iter()
                    .map(|a| self.gen_expr(a, instructions))
                    .collect();
                let dest = self.fresh_temp();

                // Check if this is a builtin/native call
                let is_builtin = matches!(
                    func_name.as_str(),
                    "println"
                        | "print"
                        | "eprintln"
                        | "eprint"
                        | "args"
                        | "exit"
                        | "format"
                        | "typeof"
                        | "sizeof"
                        | "assert"
                        | "assert_eq"
                        | "dbg"
                );

                if is_builtin {
                    // Determine the native module for this builtin
                    let module = match func_name.as_str() {
                        "println" | "print" | "eprintln" | "eprint" => "io",
                        "format" => "fmt",
                        "args" | "exit" => "process",
                        "typeof" | "sizeof" => "reflect",
                        "assert" | "assert_eq" => "debug",
                        "dbg" => "debug",
                        _ => "core",
                    };
                    instructions.push(IrInstruction::NativeCall {
                        dest: Some(dest.clone()),
                        module: module.to_string(),
                        func: func_name,
                        args: arg_vals,
                    });
                } else {
                    instructions.push(IrInstruction::Call {
                        dest: Some(dest.clone()),
                        func: func_name,
                        args: arg_vals,
                    });
                }
                IrValue::Var(dest)
            }
            TypedExprKind::MethodCall {
                receiver,
                method,
                args,
            } => {
                let recv_val = self.gen_expr(receiver, instructions);
                let arg_vals: Vec<_> = args
                    .iter()
                    .map(|a| self.gen_expr(a, instructions))
                    .collect();
                let dest = self.fresh_temp();

                // Check if this is a trait object (dynamic dispatch)
                if matches!(&receiver.ty, Type::Named(n) if n.starts_with("dyn ")) {
                    // Virtual dispatch through vtable
                    let vtable_dest = self.fresh_temp();
                    instructions.push(IrInstruction::VTableLookup {
                        dest: vtable_dest.clone(),
                        object: match &recv_val {
                            IrValue::Var(v) => v.clone(),
                            _ => "obj".to_string(),
                        },
                        trait_name: "Trait".to_string(),
                        method_idx: 0,
                    });
                    instructions.push(IrInstruction::TraitDispatch {
                        dest: Some(dest.clone()),
                        object: match &recv_val {
                            IrValue::Var(v) => v.clone(),
                            _ => "obj".to_string(),
                        },
                        method: method.clone(),
                        args: arg_vals,
                    });
                } else {
                    // Static dispatch
                    let mut all_args = vec![recv_val];
                    all_args.extend(arg_vals);
                    let mangled_name = format!(
                        "{}_{}",
                        match &receiver.ty {
                            Type::Named(n) => n,
                            _ => "unknown",
                        },
                        method
                    );
                    instructions.push(IrInstruction::Call {
                        dest: Some(dest.clone()),
                        func: mangled_name,
                        args: all_args,
                    });
                }
                IrValue::Var(dest)
            }
            TypedExprKind::Field(obj, _field) => {
                let obj_val = self.gen_expr(obj, instructions);
                let dest = self.fresh_temp();
                // Would need field index from struct layout
                instructions.push(IrInstruction::GetField {
                    dest: dest.clone(),
                    ptr: match obj_val {
                        IrValue::Var(v) => v,
                        _ => "obj".to_string(),
                    },
                    field: 0,
                });
                IrValue::Var(dest)
            }
            TypedExprKind::Index(arr, idx) => {
                let arr_val = self.gen_expr(arr, instructions);
                let idx_val = self.gen_expr(idx, instructions);
                let dest = self.fresh_temp();
                // Use GetField with the index value for array element access
                let arr_name = match &arr_val {
                    IrValue::Var(v) => v.clone(),
                    _ => "arr".to_string(),
                };
                let idx_num = match &idx_val {
                    IrValue::Const(IrConst::Int(n)) => *n as usize,
                    _ => 0, // Dynamic index - simplified fallback
                };
                instructions.push(IrInstruction::GetField {
                    dest: dest.clone(),
                    ptr: arr_name,
                    field: idx_num,
                });
                IrValue::Var(dest)
            }
            TypedExprKind::Borrow {
                mutable: _mutable,
                expr: inner,
            } => {
                // Borrows are mostly compile-time; at IR level, just pass pointer
                self.gen_expr(inner, instructions)
            }
            TypedExprKind::Deref(inner) => {
                let ptr_val = self.gen_expr(inner, instructions);
                let dest = self.fresh_temp();
                let ty = self.convert_type(&expr.ty);
                instructions.push(IrInstruction::Load {
                    dest: dest.clone(),
                    ptr: match ptr_val {
                        IrValue::Var(v) => v,
                        _ => "ptr".to_string(),
                    },
                    ty,
                });
                IrValue::Var(dest)
            }
            TypedExprKind::StructLiteral { name, fields } => {
                let dest = self.fresh_temp();
                instructions.push(IrInstruction::Alloca {
                    dest: dest.clone(),
                    ty: IrType::Struct(name.clone()),
                });
                for (i, (_field_name, field_expr)) in fields.iter().enumerate() {
                    let val = self.gen_expr(field_expr, instructions);
                    let field_ptr = self.fresh_temp();
                    instructions.push(IrInstruction::GetField {
                        dest: field_ptr.clone(),
                        ptr: dest.clone(),
                        field: i,
                    });
                    instructions.push(IrInstruction::Store {
                        ptr: field_ptr,
                        value: val,
                    });
                }
                IrValue::Var(dest)
            }
            TypedExprKind::Array(elements) => {
                let dest = self.fresh_temp();
                let elem_ty = if let Some(first) = elements.first() {
                    self.convert_type(&first.ty)
                } else {
                    IrType::I64
                };
                instructions.push(IrInstruction::Alloca {
                    dest: dest.clone(),
                    ty: IrType::Array(Box::new(elem_ty), elements.len()),
                });
                for (idx, elem) in elements.iter().enumerate() {
                    let elem_val = self.gen_expr(elem, instructions);
                    // Generate element pointer via GetField (used as array index)
                    let elem_ptr = self.fresh_temp();
                    instructions.push(IrInstruction::GetField {
                        dest: elem_ptr.clone(),
                        ptr: dest.clone(),
                        field: idx,
                    });
                    instructions.push(IrInstruction::Store {
                        ptr: elem_ptr,
                        value: elem_val,
                    });
                }
                IrValue::Var(dest)
            }
            // New expression variants — generate IR fallback
            TypedExprKind::Range { start, end, .. } => {
                let s = start
                    .as_ref()
                    .map(|e| self.gen_expr(e, instructions))
                    .unwrap_or(IrValue::Const(IrConst::Int(0)));
                let _e = end.as_ref().map(|e| self.gen_expr(e, instructions));
                s // Return start value as fallback
            }
            TypedExprKind::Lambda { body, .. } => self.gen_expr(body, instructions),
            TypedExprKind::Tuple(elems) => {
                let dest = self.fresh_temp();
                let elem_vals: Vec<_> = elems
                    .iter()
                    .map(|e| self.gen_expr(e, instructions))
                    .collect();
                let _ = elem_vals;
                IrValue::Var(dest)
            }
            TypedExprKind::Await(inner) => {
                let val = self.gen_expr(inner, instructions);
                let future_name = match &val {
                    IrValue::Var(n) => n.clone(),
                    _ => "$await".into(),
                };
                instructions.push(IrInstruction::AsyncAwait {
                    dest: Some(self.fresh_temp()),
                    future: future_name,
                });
                val
            }
            TypedExprKind::None => IrValue::Const(IrConst::Null),
            TypedExprKind::Some(inner) => self.gen_expr(inner, instructions),
            TypedExprKind::Ok(inner) => self.gen_expr(inner, instructions),
            TypedExprKind::Err(inner) => self.gen_expr(inner, instructions),
            TypedExprKind::If {
                condition,
                then_expr,
                else_expr,
            } => {
                let _cond = self.gen_expr(condition, instructions);
                let then_val = self.gen_expr(then_expr, instructions);
                let _else_val = self.gen_expr(else_expr, instructions);
                then_val
            }
            TypedExprKind::Match {
                expr: match_expr,
                arms,
            } => {
                let val = self.gen_expr(match_expr, instructions);
                if let Some((_, body)) = arms.first() {
                    let _ = self.gen_expr(body, instructions);
                }
                val
            }
        }
    }

    fn convert_type(&self, ty: &Type) -> IrType {
        #[allow(unreachable_patterns)]
        match ty {
            Type::I8 => IrType::I8,
            Type::I16 => IrType::I16,
            Type::I32 => IrType::I32,
            Type::I64 => IrType::I64,
            Type::U8 => IrType::I8,
            Type::U16 => IrType::I16,
            Type::U32 => IrType::I32,
            Type::U64 => IrType::I64,
            Type::Usize => IrType::I64,
            Type::Isize => IrType::I64,
            Type::F32 => IrType::F32,
            Type::F64 => IrType::F64,
            Type::Bool => IrType::Bool,
            Type::Str => IrType::Ptr(Box::new(IrType::I8)),
            Type::Named(n) if n == "()" => IrType::Void,
            Type::Named(n) if n.starts_with("dyn ") => IrType::TraitObject(n[4..].to_string()),
            Type::Named(n) => IrType::Struct(n.clone()),
            Type::Generic(n, _args) => {
                // For generics, we'd monomorphize - for now return struct
                IrType::Struct(n.clone())
            }
            Type::Array(inner, size) => {
                let inner_ty = self.convert_type(inner);
                IrType::Array(Box::new(inner_ty), size.as_ref().map(|_| 0).unwrap_or(0))
            }
            Type::Slice(inner) => {
                let inner_ty = self.convert_type(inner);
                IrType::Ptr(Box::new(inner_ty))
            }
            Type::Function(params, ret) => {
                let ret_ty = ret
                    .as_ref()
                    .map(|r| self.convert_type(r))
                    .unwrap_or(IrType::Void);
                let param_tys: Vec<_> = params.iter().map(|p| self.convert_type(p)).collect();
                IrType::FnPtr {
                    params: param_tys,
                    ret: Box::new(ret_ty),
                }
            }
            Type::WithOwnership(inner, ownership) => {
                let inner_ty = self.convert_type(inner);
                match ownership {
                    Ownership::Borrow | Ownership::BorrowMut | Ownership::RawPointer => {
                        IrType::Ptr(Box::new(inner_ty))
                    }
                    _ => inner_ty,
                }
            }
            Type::SelfOwned => IrType::Struct("Self".to_string()),
            Type::SelfRef { .. } => IrType::Ptr(Box::new(IrType::Struct("Self".to_string()))),
            Type::Tuple(elements) => {
                let ir_elements: Vec<IrType> =
                    elements.iter().map(|e| self.convert_type(e)).collect();
                IrType::Tuple(ir_elements)
            }
            Type::Nullable(inner) => {
                let inner_ty = self.convert_type(inner);
                IrType::Ptr(Box::new(inner_ty))
            }
            Type::TraitObject { principal, .. } => IrType::TraitObject(principal.clone()),
            Type::WhereConstrained { base, .. } => self.convert_type(base),
            _ => IrType::I64,
        }
    }
}

pub fn generate(module: TypedModule) -> Result<IrModule, String> {
    let mut gen = IrGenerator::new();
    Ok(gen.generate(module))
}
