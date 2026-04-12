use crate::ir::IrFunction;

#[allow(dead_code)]
/// Interface for an optimization pass
pub trait OptimizationPass {
    /// Name of the pass for logging/debugging
    fn name(&self) -> &str;

    /// Run the pass on a function.
    /// Returns true if the function was modified.
    fn run(&self, func: &mut IrFunction) -> bool;
}

pub mod const_prop;
pub mod dce;
pub mod licm;
mod tests;
