//! MIR Pretty Printer

use super::{
    BasicBlock, BinOp, BorrowKind, LocalVar, MirModule, Place, Projection, Rvalue, Statement,
    Terminator, Type, UnOp,
};

impl MirModule {
    /// Pretty print the MIR module
    pub fn pretty_print(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("fn {}() {{\n", self.name));

        // Print local variables
        for (local, ty) in &self.locals {
            let name = self
                .var_names
                .get(local)
                .cloned()
                .unwrap_or_else(|| format!("_{}", local.0));
            output.push_str(&format!("    let {}: {};\n", name, ty.pretty_print()));
        }

        output.push_str("\n");

        // Print basic blocks
        for (ref_, block) in &self.basic_blocks {
            output.push_str(&format!("bb{}: {{\n", ref_.0));
            for stmt in &block.statements {
                output.push_str(&format!("        {};\n", stmt.pretty_print()));
            }
            output.push_str(&format!("        {}\n", block.terminator.pretty_print()));
            output.push_str("    }\n\n");
        }

        output.push_str("}\n");
        output
    }

    /// Get a summary of the module
    pub fn summary(&self) -> String {
        format!(
            "MIRModule {}: {} blocks, {} locals",
            self.name,
            self.basic_blocks.len(),
            self.locals.len()
        )
    }
}

impl BasicBlock {
    /// Pretty print a basic block
    pub fn pretty_print(&self) -> String {
        let mut output = format!("bb{}: {{\n", self.name);
        for stmt in &self.statements {
            output.push_str(&format!("    {};\n", stmt.pretty_print()));
        }
        output.push_str(&format!("    {}\n", self.terminator.pretty_print()));
        output.push_str("}");
        output
    }
}

impl Statement {
    pub fn pretty_print(&self) -> String {
        match self {
            Statement::Assign(p, r) => format!("{} = {}", p.pretty_print(), r.pretty_print()),
            Statement::FakeRead(mode) => format!("fake_read({:?})", mode),
            Statement::StorageLive(p) => format!("storage_live({})", p.pretty_print()),
            Statement::StorageDead(p) => format!("storage_dead({})", p.pretty_print()),
            Statement::Drop(p) => format!("drop({})", p.pretty_print()),
            Statement::DebugMarker(s) => format!("debug_marker({})", s),
        }
    }
}

impl Terminator {
    pub fn pretty_print(&self) -> String {
        match self {
            Terminator::Goto(target) => format!("goto bb{}", target.0),
            Terminator::SwitchInt {
                discr,
                switch_ty,
                cases,
                otherwise,
            } => {
                let mut output =
                    format!("switch_int({}, {:?}) {{\n", discr.pretty_print(), switch_ty);
                for (val, target) in cases {
                    output.push_str(&format!("        {} => bb{},\n", val, target.0));
                }
                output.push_str(&format!("        _ => bb{}\n", otherwise.0));
                output.push_str("    }");
                output
            }
            Terminator::Return(p) => format!("return {}", p.pretty_print()),
            Terminator::Call {
                func,
                args,
                destination,
            } => {
                if let Some((ret, target)) = destination {
                    format!(
                        "{} = {}({:?}) -> bb{}",
                        ret.pretty_print(),
                        func,
                        args.iter().map(|p| p.pretty_print()).collect::<Vec<_>>(),
                        target.0
                    )
                } else {
                    format!(
                        "{}({:?})",
                        func,
                        args.iter().map(|p| p.pretty_print()).collect::<Vec<_>>()
                    )
                }
            }
            Terminator::Assert {
                condition,
                expected,
                target,
            } => {
                format!(
                    "assert {} == {} -> bb{}",
                    condition.pretty_print(),
                    expected,
                    target.0
                )
            }
        }
    }
}

impl Rvalue {
    pub fn pretty_print(&self) -> String {
        match self {
            Rvalue::Use(p) => p.pretty_print(),
            Rvalue::BinaryOp(op, a, b) => {
                format!("{:?}({}, {})", op, a.pretty_print(), b.pretty_print())
            }
            Rvalue::UnaryOp(op, a) => format!("{:?}({})", op, a.pretty_print()),
            Rvalue::CheckedBinaryOp(op, a, b) => format!(
                "Checked{:?}({}, {})",
                op,
                a.pretty_print(),
                b.pretty_print()
            ),
            Rvalue::Aggregate(kind, places) => {
                let pstrs = places.iter().map(|p| p.pretty_print()).collect::<Vec<_>>();
                format!("{:?}({:?})", kind, pstrs)
            }
            Rvalue::FunctionCall(name, args) => {
                let astrs = args.iter().map(|p| p.pretty_print()).collect::<Vec<_>>();
                format!("{}({:?})", name, astrs)
            }
            Rvalue::Len(p) => format!("len({})", p.pretty_print()),
            Rvalue::Ref(p, kind) => format!("&{:?}{}", kind, p.pretty_print()),
            Rvalue::AddressOf(p) => format!("addr_of({})", p.pretty_print()),
        }
    }
}

impl Place {
    pub fn pretty_print(&self) -> String {
        if self.projection.is_empty() {
            format!("_{}", self.local.0)
        } else {
            let mut s = format!("_{}", self.local.0);
            for proj in &self.projection {
                match proj {
                    Projection::Field(i) => format!("{}.{}", s, i),
                    Projection::Deref => format!("*{}", s),
                    Projection::Index(i) => format!("{}[{}]", s, i.pretty_print()),
                    Projection::Subslice { start, end } => format!("{}[{}..{}]", s, start, end),
                };
            }
            s
        }
    }
}

impl Type {
    pub fn pretty_print(&self) -> String {
        match self {
            Type::Bool => "bool".to_string(),
            Type::Char => "char".to_string(),
            Type::I8 => "i8".to_string(),
            Type::I16 => "i16".to_string(),
            Type::I32 => "i32".to_string(),
            Type::I64 => "i64".to_string(),
            Type::I128 => "i128".to_string(),
            Type::Isize => "isize".to_string(),
            Type::U8 => "u8".to_string(),
            Type::U16 => "u16".to_string(),
            Type::U32 => "u32".to_string(),
            Type::U64 => "u64".to_string(),
            Type::U128 => "u128".to_string(),
            Type::Usize => "usize".to_string(),
            Type::F32 => "f32".to_string(),
            Type::F64 => "f64".to_string(),
            Type::Str => "str".to_string(),
            Type::Ptr => "ptr".to_string(),
            Type::Never => "!".to_string(),
            Type::Tuple => "tuple".to_string(),
            Type::Array => "array".to_string(),
            Type::Slice => "slice".to_string(),
            Type::Struct => "struct".to_string(),
        }
    }
}
