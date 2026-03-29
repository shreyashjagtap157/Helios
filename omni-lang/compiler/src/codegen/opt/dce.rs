use super::OptimizationPass;
use crate::ir::{IrFunction, IrInstruction, IrTerminator};
use std::collections::HashSet;

pub struct DeadCodeElimination;

impl OptimizationPass for DeadCodeElimination {
    fn name(&self) -> &str {
        "Dead Code Elimination"
    }

    fn run(&self, func: &mut IrFunction) -> bool {
        let mut changed = false;
        let mut worklist = true;

        while worklist {
            worklist = false;
            let mut used_vars = HashSet::new();

            // 1. Mark used variables
            for block in &func.blocks {
                for inst in &block.instructions {
                    match inst {
                        IrInstruction::Load { ptr, .. } => {
                            used_vars.insert(ptr.clone());
                        }
                        IrInstruction::Store { ptr, value } => {
                            used_vars.insert(ptr.clone());
                            if let crate::ir::IrValue::Var(v) = value {
                                used_vars.insert(v.clone());
                            }
                        }
                        IrInstruction::BinOp { left, right, .. } => {
                            if let crate::ir::IrValue::Var(v) = left {
                                used_vars.insert(v.clone());
                            }
                            if let crate::ir::IrValue::Var(v) = right {
                                used_vars.insert(v.clone());
                            }
                        }
                        IrInstruction::Call { func: _, args, .. } => {
                            for arg in args {
                                if let crate::ir::IrValue::Var(v) = arg {
                                    used_vars.insert(v.clone());
                                }
                            }
                        }
                        IrInstruction::Phi { incoming, .. } => {
                            for (val, _) in incoming {
                                used_vars.insert(val.clone());
                            }
                        }
                        IrInstruction::Select {
                            cond,
                            then_val,
                            else_val,
                            ..
                        } => {
                            if let crate::ir::IrValue::Var(v) = cond {
                                used_vars.insert(v.clone());
                            }
                            if let crate::ir::IrValue::Var(v) = then_val {
                                used_vars.insert(v.clone());
                            }
                            if let crate::ir::IrValue::Var(v) = else_val {
                                used_vars.insert(v.clone());
                            }
                        }
                        // ... other instructions ...
                        _ => {}
                    }
                }
                match &block.terminator {
                    IrTerminator::Return(Some(crate::ir::IrValue::Var(v))) => {
                        used_vars.insert(v.clone());
                    }
                    IrTerminator::CondBranch { cond, .. } => {
                        if let crate::ir::IrValue::Var(v) = cond {
                            used_vars.insert(v.clone());
                        }
                    }
                    _ => {}
                }
            }

            // 2. Sweep unused instructions
            for block in &mut func.blocks {
                let initial_len = block.instructions.len();
                block.instructions.retain(|inst| {
                    match inst {
                        // Instructions with side effects must be kept
                        IrInstruction::Call { .. }
                        | IrInstruction::Store { .. }
                        | IrInstruction::NativeCall { .. }
                        | IrInstruction::AsyncSpawn { .. }
                        | IrInstruction::CallClosure { .. } => true,

                        // Instructions defining a value
                        IrInstruction::Alloca { dest, .. }
                        | IrInstruction::Load { dest, .. }
                        | IrInstruction::BinOp { dest, .. }
                        | IrInstruction::GetField { dest, .. }
                        | IrInstruction::Phi { dest, .. }
                        | IrInstruction::Select { dest, .. }
                        | IrInstruction::Cast { dest, .. } => used_vars.contains(dest),

                        _ => true,
                    }
                });

                if block.instructions.len() != initial_len {
                    changed = true;
                    worklist = true; // Re-run as removing uses might tree-shake more
                }
            }
        }

        changed
    }
}
