//! GPU Kernel Fusion Pass
//! 
//! Merges consecutive element-wise operations into single GPU kernels,
//! reducing memory bandwidth by keeping intermediate results in registers/shared memory.

use crate::ir::{IrModule, IrFunction, IrBlock, IrInstruction, IrValue, IrBinOp};

pub struct KernelFusionPass;

/// A chain of fusable operations that can be merged into a single kernel
#[derive(Debug, Clone)]
struct FusionChain {
    /// Indices of instructions in the original block
    instruction_indices: Vec<usize>,
    /// Collected input variables (the "roots" that feed the chain from outside)
    external_inputs: Vec<String>,
    /// The final output variable
    output: String,
    /// The fused operation names (for debug/logging)
    op_names: Vec<String>,
}

impl KernelFusionPass {
    pub fn run(module: &mut IrModule) {
        log::info!("Running Kernel Fusion Pass...");
        let mut total_fused = 0;
        
        for func in &mut module.functions {
            let fused = Self::fuse_kernels(func);
            total_fused += fused;
        }
        
        log::info!("Kernel Fusion: merged {} operation chains into fused kernels", total_fused);
    }
    
    fn fuse_kernels(func: &mut IrFunction) -> usize {
        let mut total_fused = 0;
        
        for block_idx in 0..func.blocks.len() {
            let chains = Self::find_fusion_chains(&func.blocks[block_idx]);
            
            if chains.is_empty() {
                continue;
            }
            
            log::info!("Found {} fusion chains in {}::{}", 
                chains.len(), func.name, func.blocks[block_idx].label);
            
            // Apply fusion: replace instruction chains with fused calls
            // Process chains in reverse order so indices remain valid
            let mut sorted_chains = chains;
            sorted_chains.sort_by(|a, b| b.instruction_indices[0].cmp(&a.instruction_indices[0]));
            
            for chain in &sorted_chains {
                Self::apply_fusion(&mut func.blocks[block_idx], chain);
                total_fused += 1;
            }
        }
        
        total_fused
    }
    
    /// Find sequences of fusable element-wise operations in a block
    fn find_fusion_chains(block: &IrBlock) -> Vec<FusionChain> {
        let mut chains = Vec::new();
        let mut current_chain: Vec<usize> = Vec::new();
        let mut chain_outputs: Vec<String> = Vec::new();
        let mut chain_op_names: Vec<String> = Vec::new();
        
        for (i, inst) in block.instructions.iter().enumerate() {
            let (is_fusable, op_name) = Self::classify_instruction(inst);
            
            if is_fusable {
                // Check if this instruction consumes output of previous in chain
                let consumes_prev = if let Some(last_output) = chain_outputs.last() {
                    Self::instruction_uses(inst, last_output)
                } else {
                    true // First in potential chain
                };
                
                if consumes_prev || current_chain.is_empty() {
                    current_chain.push(i);
                    if let Some(dest) = Self::instruction_dest(inst) {
                        chain_outputs.push(dest);
                    }
                    chain_op_names.push(op_name);
                } else {
                    // Break the chain, save if long enough
                    Self::maybe_save_chain(&mut chains, &current_chain, &chain_op_names, &block.instructions);
                    current_chain = vec![i];
                    chain_outputs.clear();
                    chain_op_names = vec![op_name];
                    if let Some(dest) = Self::instruction_dest(inst) {
                        chain_outputs.push(dest);
                    }
                }
            } else {
                Self::maybe_save_chain(&mut chains, &current_chain, &chain_op_names, &block.instructions);
                current_chain.clear();
                chain_outputs.clear();
                chain_op_names.clear();
            }
        }
        
        // Don't forget the last chain
        Self::maybe_save_chain(&mut chains, &current_chain, &chain_op_names, &block.instructions);
        
        chains
    }
    
    fn maybe_save_chain(
        chains: &mut Vec<FusionChain>,
        indices: &[usize],
        op_names: &[String],
        instructions: &[IrInstruction],
    ) {
        if indices.len() >= 2 {
            // Collect external inputs: values used by the chain but not produced within it
            let chain_dests: Vec<String> = indices.iter()
                .filter_map(|&i| Self::instruction_dest(&instructions[i]))
                .collect();
            
            let mut external_inputs = Vec::new();
            for &idx in indices {
                for input in Self::instruction_inputs(&instructions[idx]) {
                    if !chain_dests.contains(&input) && !external_inputs.contains(&input) {
                        external_inputs.push(input);
                    }
                }
            }
            
            let output = chain_dests.last().cloned().unwrap_or_default();
            
            chains.push(FusionChain {
                instruction_indices: indices.to_vec(),
                external_inputs,
                output,
                op_names: op_names.to_vec(),
            });
        }
    }
    
    /// Classify whether an instruction is element-wise fusable
    fn classify_instruction(inst: &IrInstruction) -> (bool, String) {
        match inst {
            IrInstruction::BinOp { op, .. } => {
                let name = format!("{:?}", op);
                (true, name)
            },
            IrInstruction::Call { func, args, .. } if Self::is_elementwise(func) => {
                (true, func.clone())
            },
            _ => (false, String::new()),
        }
    }
    
    /// Check if an instruction uses a given variable
    fn instruction_uses(inst: &IrInstruction, var: &str) -> bool {
        Self::instruction_inputs(inst).iter().any(|v| v == var)
    }
    
    /// Get destination variable of an instruction
    fn instruction_dest(inst: &IrInstruction) -> Option<String> {
        match inst {
            IrInstruction::BinOp { dest, .. } => Some(dest.clone()),
            IrInstruction::Call { dest, .. } => dest.clone(),
            IrInstruction::Load { dest, .. } => Some(dest.clone()),
            IrInstruction::Alloca { dest, .. } => Some(dest.clone()),
            IrInstruction::Cast { dest, .. } => Some(dest.clone()),
            _ => None,
        }
    }
    
    /// Get input variable names from an instruction
    fn instruction_inputs(inst: &IrInstruction) -> Vec<String> {
        let mut inputs = Vec::new();
        match inst {
            IrInstruction::BinOp { left, right, .. } => {
                if let IrValue::Var(v) = left { inputs.push(v.clone()); }
                if let IrValue::Var(v) = right { inputs.push(v.clone()); }
            },
            IrInstruction::Call { args, .. } => {
                for a in args {
                    if let IrValue::Var(v) = a { inputs.push(v.clone()); }
                }
            },
            IrInstruction::Load { ptr, .. } => {
                inputs.push(ptr.clone());
            },
            _ => {},
        }
        inputs
    }
    
    /// Replace a chain of instructions with a single fused kernel call
    fn apply_fusion(block: &mut IrBlock, chain: &FusionChain) {
        let fused_name = format!("__fused_kernel_{}", 
            chain.op_names.join("_"));
        
        // Build args from external inputs
        let args: Vec<IrValue> = chain.external_inputs.iter()
            .map(|name| IrValue::Var(name.clone()))
            .collect();
        
        // Create the fused call instruction
        let fused_inst = IrInstruction::Call {
            dest: Some(chain.output.clone()),
            func: fused_name.clone(),
            args,
        };
        
        // Remove old instructions (in reverse to preserve indices)
        let mut indices = chain.instruction_indices.clone();
        indices.sort();
        indices.reverse();
        for &idx in &indices {
            if idx < block.instructions.len() {
                block.instructions.remove(idx);
            }
        }
        
        // Insert fused call at the position of the first original instruction
        let insert_pos = *chain.instruction_indices.iter().min().unwrap_or(&0);
        let insert_pos = insert_pos.min(block.instructions.len());
        block.instructions.insert(insert_pos, fused_inst);
        
        log::debug!("Fused {} ops into {}: inputs={:?}, output={}", 
            chain.op_names.len(), fused_name, chain.external_inputs, chain.output);
    }
    
    fn is_elementwise(name: &str) -> bool {
        matches!(name, 
            "add" | "sub" | "mul" | "div" | "neg" |
            "relu" | "sigmoid" | "tanh" | "gelu" | "silu" | "swish" |
            "exp" | "log" | "sqrt" | "rsqrt" | "abs" |
            "sin" | "cos" | "pow" |
            "clamp" | "min" | "max" |
            "fma" | "lerp"
        )
    }
}
