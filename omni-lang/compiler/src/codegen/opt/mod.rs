use crate::ir::IrFunction;

/// Interface for an optimization pass
pub trait OptimizationPass {
    /// Name of the pass for logging/debugging
    fn name(&self) -> &str;
    
    /// Run the pass on a function.
    /// Returns true if the function was modified.
    fn run(&self, func: &mut IrFunction) -> bool;
}

pub mod dce;
pub mod const_prop;
pub mod licm;
mod tests;
