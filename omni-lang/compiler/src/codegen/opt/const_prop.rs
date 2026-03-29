use super::OptimizationPass;
use crate::ir::{IrBinOp, IrConst, IrFunction, IrInstruction, IrValue};
use std::collections::HashMap;

pub struct ConstantPropagation;

impl OptimizationPass for ConstantPropagation {
    fn name(&self) -> &str {
        "Constant Propagation"
    }

    fn run(&self, func: &mut IrFunction) -> bool {
        let mut changed = false;
        let mut constants = HashMap::new();

        // 1. Identify constants
        for block in &func.blocks {
            for inst in &block.instructions {
                // Determine if instruction defines a constant
                // Currently generated IR might not have explicit "Const" instructions for all,
                // but literals in BinOp are common.
                // However, basic const prop looks for: %v = const 5
                // Our IR uses IrValue in operands.
                // So we look for: %v = something that evaluates to constant.
                // Or simply simple copies/aliasing if we had Copy.

                // For now, let's look for simple arithmetic on constants.
                if let IrInstruction::BinOp {
                    dest,
                    op,
                    left,
                    right,
                } = inst
                {
                    if let (IrValue::Const(l), IrValue::Const(r)) = (left, right) {
                        let val = match (op, l, r) {
                            (IrBinOp::Add, IrConst::Int(a), IrConst::Int(b)) => {
                                Some(IrConst::Int(a + b))
                            }
                            (IrBinOp::Sub, IrConst::Int(a), IrConst::Int(b)) => {
                                Some(IrConst::Int(a - b))
                            }
                            (IrBinOp::Mul, IrConst::Int(a), IrConst::Int(b)) => {
                                Some(IrConst::Int(a * b))
                            }
                            // TODO: Handle more ops and types
                            _ => None,
                        };

                        if let Some(c) = val {
                            constants.insert(dest.clone(), c);
                        }
                    }
                }
            }
        }

        if constants.is_empty() {
            return false;
        }

        // 2. Replace usages
        for block in &mut func.blocks {
            for inst in &mut block.instructions {
                match inst {
                    IrInstruction::BinOp { left, right, .. } => {
                        if let IrValue::Var(v) = left {
                            if let Some(c) = constants.get(v) {
                                *left = IrValue::Const(c.clone());
                                changed = true;
                            }
                        }
                        if let IrValue::Var(v) = right {
                            if let Some(c) = constants.get(v) {
                                *right = IrValue::Const(c.clone());
                                changed = true;
                            }
                        }
                    }
                    IrInstruction::Call { args, .. } => {
                        for arg in args {
                            if let IrValue::Var(v) = arg {
                                if let Some(c) = constants.get(v) {
                                    *arg = IrValue::Const(c.clone());
                                    changed = true;
                                }
                            }
                        }
                    }
                    // TODO: Handle loops/Phi nodes (requires fixed-point iteration)
                    _ => {}
                }
            }
        }

        // 3. (Optional) Remove the defining instructions of the constants
        // This is handled by DCE in the next pass.

        changed
    }
}
