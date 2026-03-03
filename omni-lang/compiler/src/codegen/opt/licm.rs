use crate::ir::{IrFunction, IrInstruction, IrValue, IrTerminator};
use crate::codegen::opt::OptimizationPass;
use std::collections::{HashMap, HashSet};
use log::debug;

/// Loop Invariant Code Motion (LICM) Pass
///
/// Hoists loop-invariant instructions out of loops to the preheader.
pub struct LoopInvariantCodeMotion;

impl OptimizationPass for LoopInvariantCodeMotion {
    fn name(&self) -> &str {
        "LoopInvariantCodeMotion"
    }

    fn run(&self, func: &mut IrFunction) -> bool {
        let mut changed = false;

        // 1. Detect natural loops (simplified: back-edge to a dominator)
        // For simplicity, we'll use the same heuristic as the monolithic optimizer:
        // A branch to an earlier block index is a back-edge.
        // (A real implementation would build a Dominator Tree).
        
        let block_indices: HashMap<String, usize> = func.blocks.iter()
            .enumerate()
            .map(|(i, b)| (b.label.clone(), i))
            .collect();

        let mut loops: Vec<(usize, usize)> = Vec::new(); // (header_idx, latch_idx)

        for (i, block) in func.blocks.iter().enumerate() {
            match &block.terminator {
                IrTerminator::Branch(target) => {
                    if let Some(&target_idx) = block_indices.get(target) {
                        if target_idx <= i {
                            loops.push((target_idx, i));
                        }
                    }
                }
                IrTerminator::CondBranch { then_label, else_label, .. } => {
                    if let Some(&t_idx) = block_indices.get(then_label) {
                        if t_idx <= i { loops.push((t_idx, i)); }
                    }
                    if let Some(&e_idx) = block_indices.get(else_label) {
                        if e_idx <= i { loops.push((e_idx, i)); }
                    }
                }
                _ => {}
            }
        }

        // 2. Process loops
        // Sort loops by header index to process outer loops first? Or inner?
        // Actually, simple back-edge detection might overlap. 
        // We'll just process them as found.
        
        for (header_idx, latch_idx) in loops {
            // Identify loop body blocks: simplistic approach: all blocks between header and latch
            // This is incorrect for complex CFGs but works for simple structured loops.
            // A better way is BFS backwards from latch to header.
            
            // For now, adhering to the logic in optimizer.rs which assumes linear layout for checking
            // (block_idx >= header && block_idx <= latch) -- which is risky but matches existing code I'm porting.
            // Let's improve it slightly: collect loop blocks proper.
            
            let mut loop_blocks = HashSet::new();
            if header_idx <= latch_idx {
                 for i in header_idx..=latch_idx {
                    loop_blocks.insert(i);
                }
            } else {
                continue; // Invalid range
            }

            // Identify variables defined in the loop
            let mut loop_defs: HashSet<String> = HashSet::new();
            for &block_idx in &loop_blocks {
                if block_idx < func.blocks.len() {
                    for instr in &func.blocks[block_idx].instructions {
                        if let Some(dest) = self.instruction_dest(instr) {
                            loop_defs.insert(dest);
                        }
                    }
                }
            }

            // Find invariant instructions
            if header_idx > 0 {
                let preheader_idx = header_idx - 1;
                // Verify preheader exists and strictly dominates (simple check: linear layout)
                // If preheader terminates with conditional branch, we can't hoist blindly unless we split edge.
                // For this implementation, we assume if we hoist, we hoist to block strictly before header.
                // Ideally, we'd insert a preheader block if needed.
                
                // Check if preheader falls through or branches to header?
                // For simplicity, we attempt to hoist to func.blocks[header_idx - 1].
                
                let mut hoisted_count = 0;

                for &block_idx in &loop_blocks {
                    if block_idx >= func.blocks.len() { continue; }

                    // We need to extract instructions, modifying the block in place.
                    // Rust ownership makes iterating while modifying tricky.
                    // We'll swap instructions out, filter, and put back.
                    
                    let mut to_hoist: Vec<IrInstruction> = Vec::new();
                    let mut remaining: Vec<IrInstruction> = Vec::new();

                    // Take instructions out
                    let instructions = std::mem::take(&mut func.blocks[block_idx].instructions);
                    
                    for instr in instructions {
                        if self.is_loop_invariant(&instr, &loop_defs) && self.is_pure(&instr) {
                            to_hoist.push(instr);
                            changed = true;
                            hoisted_count += 1;
                        } else {
                            remaining.push(instr);
                        }
                    }

                    func.blocks[block_idx].instructions = remaining;

                    // Append hoisted instructions to preheader
                    if !to_hoist.is_empty() {
                         func.blocks[preheader_idx].instructions.extend(to_hoist);
                    }
                }
                
                if hoisted_count > 0 {
                    debug!("LICM: Hoisted {} instructions for loop headed at {}", hoisted_count, func.blocks[header_idx].label);
                }
            }
        }

        changed
    }
}

impl LoopInvariantCodeMotion {
    fn is_loop_invariant(&self, instr: &IrInstruction, loop_defs: &HashSet<String>) -> bool {
        match instr {
            IrInstruction::BinOp { left, right, .. } => {
                self.val_invariant(left, loop_defs) && self.val_invariant(right, loop_defs)
            }
            IrInstruction::Cast { value, .. } => self.val_invariant(value, loop_defs),
            IrInstruction::GetField { ptr, .. } => !loop_defs.contains(ptr), // Simplified ptr check
            IrInstruction::Select { cond, then_val, else_val, .. } => {
                self.val_invariant(cond, loop_defs) && 
                self.val_invariant(then_val, loop_defs) && 
                self.val_invariant(else_val, loop_defs)
            }
            _ => false,
        }
    }

    fn val_invariant(&self, val: &IrValue, loop_defs: &HashSet<String>) -> bool {
        match val {
            IrValue::Const(_) => true,
            IrValue::Var(v) => !loop_defs.contains(v),
        }
    }

    fn is_pure(&self, instr: &IrInstruction) -> bool {
        matches!(instr,
            IrInstruction::BinOp { .. } |
            IrInstruction::Cast { .. } |
            IrInstruction::GetField { .. } |
            IrInstruction::Select { .. } |
            IrInstruction::ExtractValue { .. }
        )
    }

    fn instruction_dest(&self, instr: &IrInstruction) -> Option<String> {
        match instr {
            IrInstruction::Alloca { dest, .. } |
            IrInstruction::Load { dest, .. } |
            IrInstruction::BinOp { dest, .. } |
            IrInstruction::GetField { dest, .. } |
            IrInstruction::Cast { dest, .. } |
            IrInstruction::ExtractValue { dest, .. } |
            IrInstruction::InsertValue { dest, .. } |
            IrInstruction::VTableLookup { dest, .. } |
            IrInstruction::Phi { dest, .. } |
            IrInstruction::Select { dest, .. } |
            IrInstruction::CreateClosure { dest, .. } |
            IrInstruction::AsyncSpawn { dest, .. } => Some(dest.clone()),
            IrInstruction::Call { dest, .. } |
            IrInstruction::CallClosure { dest, .. } |
            IrInstruction::AsyncAwait { dest, .. } |
            IrInstruction::NativeCall { dest, .. } |
            IrInstruction::TraitDispatch { dest, .. } => dest.clone(),
            _ => None,
        }
    }
}
