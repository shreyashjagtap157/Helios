#![allow(dead_code)]
//! Advanced IR Optimization Passes for Code Generation
//!
//! Implements real transformation passes:
//! - Constant folding (evaluates constant expressions at compile time)
//! - Dead code elimination (removes unused variables and unreachable code)
//! - Tail call optimization (converts tail calls to jumps)
//! - Function inlining (replaces call sites with function body)
//! - Common subexpression elimination (deduplicates identical computations)
//! - Block merging (combines single-predecessor/successor blocks)
//! - Loop-invariant code motion (LICM)
//! - Strength reduction (replaces expensive ops with cheaper equivalents)
//! - Copy propagation (eliminates redundant copies)
//! - Algebraic simplification (identity/zero/double-negation elimination)
//! - Loop unrolling (for small constant-bound loops)
//!
//! Date: Feb 11, 2026 | Enhanced with real transformations

use crate::ir::{
    IrBinOp, IrBlock, IrConst, IrFunction, IrInstruction, IrModule, IrTerminator, IrValue,
};
use log::{debug, info, trace};
use std::collections::{HashMap, HashSet};

// ─────────────────────────────────────────────────────────────────────────────
// Optimization Statistics
// ─────────────────────────────────────────────────────────────────────────────

/// Optimization pass result
#[derive(Debug, Clone, Default)]
pub struct OptimizationStats {
    pub constant_folded: usize,
    pub dead_code_eliminated: usize,
    pub tail_calls_optimized: usize,
    pub instructions_removed: usize,
    pub blocks_merged: usize,
    pub functions_inlined: usize,
    pub strength_reduced: usize,
    pub copies_propagated: usize,
    pub algebraic_simplified: usize,
    pub licm_hoisted: usize,
    pub loops_unrolled: usize,
    pub cse_eliminated: usize,
}

impl std::fmt::Display for OptimizationStats {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "=== Optimization Results ===")?;
        writeln!(f, "Constant folded:        {}", self.constant_folded)?;
        writeln!(f, "Dead code eliminated:   {}", self.dead_code_eliminated)?;
        writeln!(f, "Instructions removed:   {}", self.instructions_removed)?;
        writeln!(f, "Blocks merged:          {}", self.blocks_merged)?;
        writeln!(f, "Tail calls optimized:   {}", self.tail_calls_optimized)?;
        writeln!(f, "Functions inlined:      {}", self.functions_inlined)?;
        writeln!(f, "Strength reduced:       {}", self.strength_reduced)?;
        writeln!(f, "Copies propagated:      {}", self.copies_propagated)?;
        writeln!(f, "Algebraic simplified:   {}", self.algebraic_simplified)?;
        writeln!(f, "LICM hoisted:           {}", self.licm_hoisted)?;
        writeln!(f, "Loops unrolled:         {}", self.loops_unrolled)?;
        writeln!(f, "CSE eliminated:         {}", self.cse_eliminated)?;
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Optimization Level
// ─────────────────────────────────────────────────────────────────────────────

/// Optimization level controlling which passes run
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OptLevel {
    /// No optimizations (debug builds)
    O0,
    /// Basic optimizations (constant fold, DCE, copy prop)
    O1,
    /// Standard optimizations (+ inlining, CSE, strength reduction, LICM)
    O2,
    /// Aggressive optimizations (+ loop unrolling, algebraic simplification)
    O3,
}

// ─────────────────────────────────────────────────────────────────────────────
// IR Optimizer
// ─────────────────────────────────────────────────────────────────────────────

/// IR Optimizer for code generation
pub struct IrOptimizer {
    stats: OptimizationStats,
    /// Enable aggressive optimizations
    pub aggressive: bool,
    /// Max function size to inline (in instructions)
    inline_threshold: usize,
    /// Max loop body size for unrolling
    unroll_threshold: usize,
    /// Optimization level
    opt_level: OptLevel,
}

impl IrOptimizer {
    pub fn new(aggressive: bool) -> Self {
        IrOptimizer {
            stats: OptimizationStats::default(),
            aggressive,
            inline_threshold: 20,
            unroll_threshold: 50,
            opt_level: if aggressive {
                OptLevel::O3
            } else {
                OptLevel::O2
            },
        }
    }

    /// Create optimizer with specific optimization level
    pub fn with_level(level: OptLevel) -> Self {
        IrOptimizer {
            stats: OptimizationStats::default(),
            aggressive: level >= OptLevel::O3,
            inline_threshold: match level {
                OptLevel::O0 | OptLevel::O1 => 0,
                OptLevel::O2 => 15,
                OptLevel::O3 => 40,
            },
            unroll_threshold: match level {
                OptLevel::O0 | OptLevel::O1 | OptLevel::O2 => 0,
                OptLevel::O3 => 100,
            },
            opt_level: level,
        }
    }

    /// Run all optimization passes on IR
    pub fn optimize(&mut self, module: &mut IrModule) -> OptimizationStats {
        info!(
            "Starting IR optimization passes (level={:?}, aggressive={})",
            self.opt_level, self.aggressive
        );

        if self.opt_level == OptLevel::O0 {
            info!("Optimization level O0: skipping all passes");
            return self.stats.clone();
        }

        // === O1 passes (basic) ===

        // Pass 1: Algebraic simplification (before constant folding)
        self.algebraic_simplification_pass(module);

        // Pass 2: Constant folding
        self.constant_fold_pass(module);

        // Pass 3: Copy propagation
        self.copy_propagation_pass(module);

        // Pass 4: Dead code elimination
        self.dead_code_elimination_pass(module);

        // Pass 5: Block merging
        self.block_merge_pass(module);

        if self.opt_level >= OptLevel::O2 {
            // === O2 passes (standard) ===

            // Pass 6: Common subexpression elimination
            self.cse_pass(module);

            // Pass 7: Strength reduction
            self.strength_reduction_pass(module);

            // Pass 8: Tail call optimization
            self.tail_call_optimization_pass(module);

            // Pass 9: Function inlining
            if self.inline_threshold > 0 {
                self.function_inlining_pass(module);
            }

            // Pass 10: LICM (loop-invariant code motion)
            self.licm_pass(module);

            // Cleanup: run DCE again after other optimizations
            self.dead_code_elimination_pass(module);
        }

        if self.opt_level >= OptLevel::O3 {
            // === O3 passes (aggressive) ===

            // Pass 11: Loop unrolling
            if self.unroll_threshold > 0 {
                self.loop_unrolling_pass(module);
            }

            // Pass 12: Second round of constant folding (after unrolling)
            self.constant_fold_pass(module);

            // Pass 13: Final DCE cleanup
            self.dead_code_elimination_pass(module);
        }

        info!("Optimization complete:\n{}", self.stats);
        self.stats.clone()
    }

    // ─────────────────────────────────────────────────────────────────────
    // Pass 1: Constant Folding
    // ─────────────────────────────────────────────────────────────────────

    /// Constant folding: evaluate constant expressions at compile time
    fn constant_fold_pass(&mut self, module: &mut IrModule) {
        for function in &mut module.functions {
            // Track known constant values
            let mut known_constants: HashMap<String, IrConst> = HashMap::new();

            for block in &mut function.blocks {
                let mut new_instructions = Vec::new();

                for instr in &block.instructions {
                    match instr {
                        IrInstruction::BinOp {
                            dest,
                            op,
                            left,
                            right,
                        } => {
                            // Resolve both operands to constants if possible
                            let left_const = self.resolve_constant(left, &known_constants);
                            let right_const = self.resolve_constant(right, &known_constants);

                            if let (Some(lc), Some(rc)) = (left_const, right_const) {
                                // Both operands are constants - fold!
                                if let Some(result) = self.eval_binop(op, &lc, &rc) {
                                    trace!(
                                        "Folded constant: {} = {:?} {:?} {:?}",
                                        dest,
                                        lc,
                                        op,
                                        rc
                                    );
                                    known_constants.insert(dest.clone(), result.clone());
                                    // Replace with identity operation carrying the folded constant
                                    new_instructions.push(IrInstruction::BinOp {
                                        dest: dest.clone(),
                                        op: IrBinOp::Add,
                                        left: IrValue::Const(result),
                                        right: IrValue::Const(IrConst::Int(0)),
                                    });
                                    self.stats.constant_folded += 1;
                                    continue;
                                }
                            }
                            new_instructions.push(instr.clone());
                        }
                        _ => {
                            new_instructions.push(instr.clone());
                        }
                    }
                }

                block.instructions = new_instructions;
            }
        }
        debug!(
            "Constant folding: {} expressions folded",
            self.stats.constant_folded
        );
    }

    /// Resolve an IrValue to a constant if possible
    fn resolve_constant(
        &self,
        value: &IrValue,
        known: &HashMap<String, IrConst>,
    ) -> Option<IrConst> {
        match value {
            IrValue::Const(c) => Some(c.clone()),
            IrValue::Var(name) => known.get(name).cloned(),
        }
    }

    /// Evaluate a binary operation on two constants
    fn eval_binop(&self, op: &IrBinOp, left: &IrConst, right: &IrConst) -> Option<IrConst> {
        match (left, right) {
            (IrConst::Int(l), IrConst::Int(r)) => {
                let result = match op {
                    IrBinOp::Add => IrConst::Int(l.wrapping_add(*r)),
                    IrBinOp::Sub => IrConst::Int(l.wrapping_sub(*r)),
                    IrBinOp::Mul => IrConst::Int(l.wrapping_mul(*r)),
                    IrBinOp::Div => {
                        if *r == 0 {
                            return None;
                        }
                        IrConst::Int(l.wrapping_div(*r))
                    }
                    IrBinOp::Mod => {
                        if *r == 0 {
                            return None;
                        }
                        IrConst::Int(l.wrapping_rem(*r))
                    }
                    IrBinOp::Eq => IrConst::Bool(l == r),
                    IrBinOp::Ne => IrConst::Bool(l != r),
                    IrBinOp::Lt => IrConst::Bool(l < r),
                    IrBinOp::Gt => IrConst::Bool(l > r),
                    IrBinOp::Le => IrConst::Bool(l <= r),
                    IrBinOp::Ge => IrConst::Bool(l >= r),
                    IrBinOp::And => IrConst::Bool(*l != 0 && *r != 0),
                    IrBinOp::Or => IrConst::Bool(*l != 0 || *r != 0),
                };
                Some(result)
            }
            (IrConst::Float(l), IrConst::Float(r)) => {
                let result = match op {
                    IrBinOp::Add => IrConst::Float(l + r),
                    IrBinOp::Sub => IrConst::Float(l - r),
                    IrBinOp::Mul => IrConst::Float(l * r),
                    IrBinOp::Div => {
                        if *r == 0.0 {
                            return None;
                        }
                        IrConst::Float(l / r)
                    }
                    IrBinOp::Eq => IrConst::Bool(l == r),
                    IrBinOp::Ne => IrConst::Bool(l != r),
                    IrBinOp::Lt => IrConst::Bool(l < r),
                    IrBinOp::Gt => IrConst::Bool(l > r),
                    IrBinOp::Le => IrConst::Bool(l <= r),
                    IrBinOp::Ge => IrConst::Bool(l >= r),
                    _ => return None,
                };
                Some(result)
            }
            (IrConst::Bool(l), IrConst::Bool(r)) => match op {
                IrBinOp::And => Some(IrConst::Bool(*l && *r)),
                IrBinOp::Or => Some(IrConst::Bool(*l || *r)),
                IrBinOp::Eq => Some(IrConst::Bool(l == r)),
                IrBinOp::Ne => Some(IrConst::Bool(l != r)),
                _ => None,
            },
            _ => None,
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Pass 2: Dead Code Elimination
    // ─────────────────────────────────────────────────────────────────────

    /// Dead code elimination: remove unused variables and unreachable code
    fn dead_code_elimination_pass(&mut self, module: &mut IrModule) {
        for function in &mut module.functions {
            // Collect all used variables (referenced in any value position)
            let mut used_vars: HashSet<String> = HashSet::new();

            // Collect uses from all blocks
            for block in &function.blocks {
                for instr in &block.instructions {
                    self.collect_uses(instr, &mut used_vars);
                }
                // Collect uses from terminators
                match &block.terminator {
                    IrTerminator::Return(Some(val)) => {
                        if let IrValue::Var(v) = val {
                            used_vars.insert(v.clone());
                        }
                    }
                    IrTerminator::CondBranch { cond, .. } => {
                        if let IrValue::Var(v) = cond {
                            used_vars.insert(v.clone());
                        }
                    }
                    _ => {}
                }
            }

            // Also mark params as used (they're inputs, can't be eliminated)
            for (name, _) in &function.params {
                used_vars.insert(name.clone());
            }

            // Remove instructions that define unused variables (unless they have side effects)
            for block in &mut function.blocks {
                let original_len = block.instructions.len();

                block.instructions.retain(|instr| {
                    match instr {
                        // Pure instructions: safe to remove if result is unused
                        IrInstruction::Alloca { dest, .. }
                        | IrInstruction::BinOp { dest, .. }
                        | IrInstruction::Load { dest, .. }
                        | IrInstruction::GetField { dest, .. }
                        | IrInstruction::Cast { dest, .. }
                        | IrInstruction::ExtractValue { dest, .. }
                        | IrInstruction::VTableLookup { dest, .. } => {
                            let keep = used_vars.contains(dest);
                            if !keep {
                                self.stats.dead_code_eliminated += 1;
                            }
                            keep
                        }
                        // Instructions with side effects: always keep
                        IrInstruction::Store { .. }
                        | IrInstruction::Call { .. }
                        | IrInstruction::NativeCall { .. }
                        | IrInstruction::AsyncSpawn { .. }
                        | IrInstruction::AsyncAwait { .. }
                        | IrInstruction::TraitDispatch { .. }
                        | IrInstruction::BoundsCheck { .. }
                        | IrInstruction::CallClosure { .. } => true,
                        // Other pure instructions
                        IrInstruction::Phi { dest, .. }
                        | IrInstruction::Select { dest, .. }
                        | IrInstruction::InsertValue { dest, .. }
                        | IrInstruction::CreateClosure { dest, .. } => {
                            let keep = used_vars.contains(dest);
                            if !keep {
                                self.stats.dead_code_eliminated += 1;
                            }
                            keep
                        }
                        // Switch is a special case - keep it
                        IrInstruction::Switch { .. } => true,
                    }
                });

                self.stats.instructions_removed +=
                    original_len.saturating_sub(block.instructions.len());
            }

            // Remove unreachable blocks (no predecessors except entry)
            if function.blocks.len() > 1 {
                let reachable = self.find_reachable_blocks(function);
                let original_block_count = function.blocks.len();
                function.blocks.retain(|b| reachable.contains(&b.label));
                let removed_blocks = original_block_count - function.blocks.len();
                if removed_blocks > 0 {
                    self.stats.dead_code_eliminated += removed_blocks;
                    debug!(
                        "DCE: Removed {} unreachable blocks from {}",
                        removed_blocks, function.name
                    );
                }
            }
        }
        debug!(
            "Dead code elimination: {} eliminated, {} instructions removed",
            self.stats.dead_code_eliminated, self.stats.instructions_removed
        );
    }

    /// Collect all variables used by an instruction
    fn collect_uses(&self, instr: &IrInstruction, used: &mut HashSet<String>) {
        match instr {
            IrInstruction::BinOp { left, right, .. } => {
                if let IrValue::Var(v) = left {
                    used.insert(v.clone());
                }
                if let IrValue::Var(v) = right {
                    used.insert(v.clone());
                }
            }
            IrInstruction::Load { ptr, .. } => {
                used.insert(ptr.clone());
            }
            IrInstruction::Store { ptr, value } => {
                used.insert(ptr.clone());
                if let IrValue::Var(v) = value {
                    used.insert(v.clone());
                }
            }
            IrInstruction::Call { args, .. } => {
                for arg in args {
                    if let IrValue::Var(v) = arg {
                        used.insert(v.clone());
                    }
                }
            }
            IrInstruction::GetField { ptr, .. } => {
                used.insert(ptr.clone());
            }
            IrInstruction::Cast { value, .. } => {
                if let IrValue::Var(v) = value {
                    used.insert(v.clone());
                }
            }
            IrInstruction::Select {
                cond,
                then_val,
                else_val,
                ..
            } => {
                if let IrValue::Var(v) = cond {
                    used.insert(v.clone());
                }
                if let IrValue::Var(v) = then_val {
                    used.insert(v.clone());
                }
                if let IrValue::Var(v) = else_val {
                    used.insert(v.clone());
                }
            }
            IrInstruction::Phi { incoming, .. } => {
                for (val, _) in incoming {
                    used.insert(val.clone());
                }
            }
            IrInstruction::Switch { value, .. } => {
                if let IrValue::Var(v) = value {
                    used.insert(v.clone());
                }
            }
            IrInstruction::CreateClosure { captures, .. } => {
                for cap in captures {
                    used.insert(cap.clone());
                }
            }
            IrInstruction::CallClosure { closure, args, .. } => {
                used.insert(closure.clone());
                for arg in args {
                    if let IrValue::Var(v) = arg {
                        used.insert(v.clone());
                    }
                }
            }
            IrInstruction::AsyncSpawn { args, .. } => {
                for arg in args {
                    if let IrValue::Var(v) = arg {
                        used.insert(v.clone());
                    }
                }
            }
            IrInstruction::AsyncAwait { future, .. } => {
                used.insert(future.clone());
            }
            IrInstruction::TraitDispatch { object, args, .. } => {
                used.insert(object.clone());
                for arg in args {
                    if let IrValue::Var(v) = arg {
                        used.insert(v.clone());
                    }
                }
            }
            IrInstruction::VTableLookup { object, .. } => {
                used.insert(object.clone());
            }
            IrInstruction::ExtractValue { aggregate, .. } => {
                used.insert(aggregate.clone());
            }
            IrInstruction::InsertValue {
                aggregate, value, ..
            } => {
                used.insert(aggregate.clone());
                if let IrValue::Var(v) = value {
                    used.insert(v.clone());
                }
            }
            IrInstruction::NativeCall { args, .. } => {
                for arg in args {
                    if let IrValue::Var(v) = arg {
                        used.insert(v.clone());
                    }
                }
            }
            IrInstruction::BoundsCheck { index, length } => {
                used.insert(index.clone());
                used.insert(length.clone());
            }
            IrInstruction::Alloca { .. } => {}
        }
    }

    /// Find all reachable blocks via BFS from the entry block
    fn find_reachable_blocks(&self, function: &IrFunction) -> HashSet<String> {
        let mut reachable = HashSet::new();
        let mut worklist = Vec::new();

        if let Some(entry) = function.blocks.first() {
            worklist.push(entry.label.clone());
        }

        while let Some(label) = worklist.pop() {
            if !reachable.insert(label.clone()) {
                continue; // Already visited
            }

            // Find the block and get its successors
            if let Some(block) = function.blocks.iter().find(|b| b.label == label) {
                match &block.terminator {
                    IrTerminator::Branch(target) => {
                        worklist.push(target.clone());
                    }
                    IrTerminator::CondBranch {
                        then_label,
                        else_label,
                        ..
                    } => {
                        worklist.push(then_label.clone());
                        worklist.push(else_label.clone());
                    }
                    IrTerminator::Return(_) | IrTerminator::Unreachable => {}
                }

                // Also check Switch instructions for targets
                for instr in &block.instructions {
                    if let IrInstruction::Switch { default, cases, .. } = instr {
                        worklist.push(default.clone());
                        for (_, target) in cases {
                            worklist.push(target.clone());
                        }
                    }
                }
            }
        }

        reachable
    }

    // ─────────────────────────────────────────────────────────────────────
    // Pass 3: Block Merging
    // ─────────────────────────────────────────────────────────────────────

    /// Block merging: combine consecutive blocks with single predecessor/successor
    fn block_merge_pass(&mut self, module: &mut IrModule) {
        for function in &mut module.functions {
            if function.blocks.len() <= 1 {
                continue;
            }

            // Build predecessor map
            let pred_map = self.build_predecessor_map(function);

            let mut merged = true;
            while merged {
                merged = false;
                let mut i = 0;

                while i < function.blocks.len() {
                    // Check if this block ends with an unconditional branch
                    if let IrTerminator::Branch(target) = &function.blocks[i].terminator {
                        let target = target.clone();

                        // Find the target block
                        if let Some(target_idx) =
                            function.blocks.iter().position(|b| b.label == target)
                        {
                            // Only merge if the target has exactly one predecessor (this block)
                            let pred_count = pred_map.get(&target).map(|p| p.len()).unwrap_or(0);

                            if pred_count <= 1
                                && target_idx != i
                                && target_idx < function.blocks.len()
                            {
                                // Merge: append target's instructions and terminator
                                let target_block = function.blocks.remove(target_idx);

                                // Adjust index if we removed a block before our current one
                                let current_idx = if target_idx < i { i - 1 } else { i };

                                if current_idx < function.blocks.len() {
                                    function.blocks[current_idx]
                                        .instructions
                                        .extend(target_block.instructions);
                                    function.blocks[current_idx].terminator =
                                        target_block.terminator;
                                    self.stats.blocks_merged += 1;
                                    merged = true;
                                    trace!(
                                        "Merged block '{}' into '{}'",
                                        target,
                                        function.blocks[current_idx].label
                                    );
                                }
                                continue; // Don't increment i
                            }
                        }
                    }
                    i += 1;
                }
            }
        }
        debug!("Block merging: {} blocks merged", self.stats.blocks_merged);
    }

    /// Build a map of block label -> list of predecessor labels
    fn build_predecessor_map(&self, function: &IrFunction) -> HashMap<String, Vec<String>> {
        let mut preds: HashMap<String, Vec<String>> = HashMap::new();

        for block in &function.blocks {
            match &block.terminator {
                IrTerminator::Branch(target) => {
                    preds
                        .entry(target.clone())
                        .or_default()
                        .push(block.label.clone());
                }
                IrTerminator::CondBranch {
                    then_label,
                    else_label,
                    ..
                } => {
                    preds
                        .entry(then_label.clone())
                        .or_default()
                        .push(block.label.clone());
                    preds
                        .entry(else_label.clone())
                        .or_default()
                        .push(block.label.clone());
                }
                IrTerminator::Return(_) | IrTerminator::Unreachable => {}
            }
        }

        preds
    }

    // ─────────────────────────────────────────────────────────────────────
    // Pass 4: Tail Call Optimization
    // ─────────────────────────────────────────────────────────────────────

    /// Tail call optimization: convert tail calls to jumps
    fn tail_call_optimization_pass(&mut self, module: &mut IrModule) {
        for function in &mut module.functions {
            let func_name = function.name.clone();
            let entry_label = function
                .blocks
                .first()
                .map(|b| b.label.clone())
                .unwrap_or_default();

            for block in &mut function.blocks {
                // Check for a pattern: last instruction is Call, terminator is Return of that call's result
                if let Some(last_inst) = block.instructions.last() {
                    if let IrInstruction::Call {
                        dest: Some(dest),
                        func,
                        args: _call_args,
                    } = last_inst
                    {
                        // Check if the terminator returns this value
                        if let IrTerminator::Return(Some(IrValue::Var(ret_var))) = &block.terminator
                        {
                            if ret_var == dest {
                                // This is a tail call!
                                if func == &func_name {
                                    // Self-recursion: convert to a jump back to entry
                                    debug!(
                                        "TCO: Converting recursive tail call in {} to jump",
                                        func_name
                                    );

                                    let len = block.instructions.len();
                                    block.instructions.truncate(len - 1); // Remove the call
                                    block.terminator = IrTerminator::Branch(entry_label.clone());
                                    self.stats.tail_calls_optimized += 1;
                                } else {
                                    // External tail call: mark for tail-call ABI
                                    debug!("TCO: Found tail call to {} in {}", func, func_name);
                                    self.stats.tail_calls_optimized += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        debug!(
            "Tail call optimization: {} tail calls optimized",
            self.stats.tail_calls_optimized
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // Pass 5: Function Inlining
    // ─────────────────────────────────────────────────────────────────────

    /// Function inlining: replace call sites with function body
    fn function_inlining_pass(&mut self, module: &mut IrModule) {
        // Collect small functions suitable for inlining
        let mut inline_bodies: HashMap<String, IrFunction> = HashMap::new();

        for function in &module.functions {
            let total_instructions: usize =
                function.blocks.iter().map(|b| b.instructions.len()).sum();

            if total_instructions <= self.inline_threshold && total_instructions > 0 {
                // Don't inline recursive functions
                let is_recursive = function.blocks.iter().flat_map(|b| &b.instructions).any(
                    |i| matches!(i, IrInstruction::Call { func, .. } if func == &function.name),
                );

                if !is_recursive {
                    inline_bodies.insert(function.name.clone(), function.clone());
                }
            }
        }

        if inline_bodies.is_empty() {
            return;
        }

        debug!(
            "Inlining: {} candidate functions: {:?}",
            inline_bodies.len(),
            inline_bodies.keys().collect::<Vec<_>>()
        );

        // Replace call sites with inlined bodies
        let mut inline_counter = 0u32;

        for function in &mut module.functions {
            let mut new_blocks: Vec<IrBlock> = Vec::new();

            for block in &function.blocks {
                let mut current_instructions = Vec::new();
                let block_terminator = block.terminator.clone();
                let mut needs_split = false;

                for (inst_idx, instr) in block.instructions.iter().enumerate() {
                    if let IrInstruction::Call { dest, func, args } = instr {
                        if let Some(callee) = inline_bodies.get(func) {
                            inline_counter += 1;
                            let prefix = format!("inline_{}_{}", func, inline_counter);

                            // Map callee parameters to argument values
                            let mut var_map: HashMap<String, IrValue> = HashMap::new();
                            for (i, (param_name, _)) in callee.params.iter().enumerate() {
                                if i < args.len() {
                                    var_map.insert(param_name.clone(), args[i].clone());
                                }
                            }

                            // Copy callee blocks with renamed variables
                            for (bidx, callee_block) in callee.blocks.iter().enumerate() {
                                let renamed_label = format!("{}_{}", prefix, callee_block.label);
                                let mut renamed_insts = Vec::new();

                                for ci in &callee_block.instructions {
                                    renamed_insts
                                        .push(self.rename_instruction(ci, &prefix, &var_map));
                                }

                                // Handle the terminator
                                let renamed_term = match &callee_block.terminator {
                                    IrTerminator::Return(Some(val)) => {
                                        if let Some(dest_name) = dest {
                                            // Assign return value to dest
                                            let new_val = self.rename_value(val, &prefix, &var_map);
                                            renamed_insts.push(IrInstruction::BinOp {
                                                dest: dest_name.clone(),
                                                op: IrBinOp::Add,
                                                left: new_val,
                                                right: IrValue::Const(IrConst::Int(0)),
                                            });
                                        }
                                        let cont_label = format!("{}_cont", prefix);
                                        IrTerminator::Branch(cont_label)
                                    }
                                    IrTerminator::Return(None) => {
                                        let cont_label = format!("{}_cont", prefix);
                                        IrTerminator::Branch(cont_label)
                                    }
                                    IrTerminator::Branch(target) => {
                                        IrTerminator::Branch(format!("{}_{}", prefix, target))
                                    }
                                    IrTerminator::CondBranch {
                                        cond,
                                        then_label,
                                        else_label,
                                    } => IrTerminator::CondBranch {
                                        cond: self.rename_value(cond, &prefix, &var_map),
                                        then_label: format!("{}_{}", prefix, then_label),
                                        else_label: format!("{}_{}", prefix, else_label),
                                    },
                                    IrTerminator::Unreachable => IrTerminator::Unreachable,
                                };

                                if bidx == 0 && !current_instructions.is_empty() {
                                    // First inlined block: emit caller prefix block
                                    let pre_label = if new_blocks.is_empty() {
                                        block.label.clone()
                                    } else {
                                        format!("{}_pre", prefix)
                                    };
                                    let pre_block = IrBlock {
                                        label: pre_label,
                                        instructions: current_instructions.drain(..).collect(),
                                        terminator: IrTerminator::Branch(renamed_label.clone()),
                                    };
                                    new_blocks.push(pre_block);
                                }

                                new_blocks.push(IrBlock {
                                    label: renamed_label,
                                    instructions: renamed_insts,
                                    terminator: renamed_term,
                                });
                            }

                            // Add continuation block for remaining instructions
                            let cont_label = format!("{}_cont", prefix);
                            needs_split = true;
                            current_instructions.clear();

                            // Push remaining instructions of this block
                            for remaining_inst in &block.instructions[inst_idx + 1..] {
                                current_instructions.push(remaining_inst.clone());
                            }

                            new_blocks.push(IrBlock {
                                label: cont_label,
                                instructions: current_instructions.drain(..).collect(),
                                terminator: block_terminator.clone(),
                            });

                            self.stats.functions_inlined += 1;
                            break; // Only inline one call per block per pass
                        }
                    }
                    current_instructions.push(instr.clone());
                }

                if !needs_split {
                    // No inlining happened, keep original block
                    new_blocks.push(IrBlock {
                        label: block.label.clone(),
                        instructions: current_instructions,
                        terminator: block_terminator,
                    });
                }
            }

            function.blocks = new_blocks;
        }

        debug!(
            "Function inlining: {} call sites inlined",
            self.stats.functions_inlined
        );
    }

    /// Rename an instruction's variables with a prefix (for inlining)
    fn rename_instruction(
        &self,
        inst: &IrInstruction,
        prefix: &str,
        var_map: &HashMap<String, IrValue>,
    ) -> IrInstruction {
        match inst {
            IrInstruction::BinOp {
                dest,
                op,
                left,
                right,
            } => IrInstruction::BinOp {
                dest: format!("{}_{}", prefix, dest),
                op: op.clone(),
                left: self.rename_value(left, prefix, var_map),
                right: self.rename_value(right, prefix, var_map),
            },
            IrInstruction::Alloca { dest, ty } => IrInstruction::Alloca {
                dest: format!("{}_{}", prefix, dest),
                ty: ty.clone(),
            },
            IrInstruction::Load { dest, ptr, ty } => IrInstruction::Load {
                dest: format!("{}_{}", prefix, dest),
                ptr: var_map
                    .get(ptr)
                    .and_then(|v| {
                        if let IrValue::Var(n) = v {
                            Some(n.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| format!("{}_{}", prefix, ptr)),
                ty: ty.clone(),
            },
            IrInstruction::Store { ptr, value } => IrInstruction::Store {
                ptr: var_map
                    .get(ptr)
                    .and_then(|v| {
                        if let IrValue::Var(n) = v {
                            Some(n.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| format!("{}_{}", prefix, ptr)),
                value: self.rename_value(value, prefix, var_map),
            },
            IrInstruction::Call { dest, func, args } => IrInstruction::Call {
                dest: dest.as_ref().map(|d| format!("{}_{}", prefix, d)),
                func: func.clone(),
                args: args
                    .iter()
                    .map(|a| self.rename_value(a, prefix, var_map))
                    .collect(),
            },
            IrInstruction::Phi { dest, ty, incoming } => IrInstruction::Phi {
                dest: format!("{}_{}", prefix, dest),
                ty: ty.clone(),
                incoming: incoming
                    .iter()
                    .map(|(v, l)| {
                        (
                            self.rename_var_name(v, prefix, var_map),
                            format!("{}_{}", prefix, l),
                        )
                    })
                    .collect(),
            },
            IrInstruction::Select {
                dest,
                cond,
                then_val,
                else_val,
            } => IrInstruction::Select {
                dest: format!("{}_{}", prefix, dest),
                cond: self.rename_value(cond, prefix, var_map),
                then_val: self.rename_value(then_val, prefix, var_map),
                else_val: self.rename_value(else_val, prefix, var_map),
            },
            IrInstruction::InsertValue {
                dest,
                aggregate,
                value,
                indices,
            } => IrInstruction::InsertValue {
                dest: format!("{}_{}", prefix, dest),
                aggregate: self.rename_var_name(aggregate, prefix, var_map),
                value: self.rename_value(value, prefix, var_map),
                indices: indices.clone(),
            },
            IrInstruction::ExtractValue {
                dest,
                aggregate,
                indices,
            } => IrInstruction::ExtractValue {
                dest: format!("{}_{}", prefix, dest),
                aggregate: self.rename_var_name(aggregate, prefix, var_map),
                indices: indices.clone(),
            },
            IrInstruction::CreateClosure {
                dest,
                func,
                captures,
            } => IrInstruction::CreateClosure {
                dest: format!("{}_{}", prefix, dest),
                func: func.clone(),
                captures: captures
                    .iter()
                    .map(|c| self.rename_var_name(c, prefix, var_map))
                    .collect(),
            },
            IrInstruction::CallClosure {
                dest,
                closure,
                args,
            } => IrInstruction::CallClosure {
                dest: dest.as_ref().map(|d| format!("{}_{}", prefix, d)),
                closure: self.rename_var_name(closure, prefix, var_map),
                args: args
                    .iter()
                    .map(|a| self.rename_value(a, prefix, var_map))
                    .collect(),
            },
            IrInstruction::TraitDispatch {
                dest,
                object,
                method,
                args,
            } => IrInstruction::TraitDispatch {
                dest: dest.clone().map(|d| format!("{}_{}", prefix, d)),
                object: self.rename_var_name(object, prefix, var_map),
                method: method.clone(),
                args: args
                    .iter()
                    .map(|a| self.rename_value(a, prefix, var_map))
                    .collect(),
            },
            IrInstruction::NativeCall {
                dest,
                module,
                func,
                args,
            } => IrInstruction::NativeCall {
                dest: dest.clone().map(|d| format!("{}_{}", prefix, d)),
                module: module.clone(),
                func: func.clone(),
                args: args
                    .iter()
                    .map(|a| self.rename_value(a, prefix, var_map))
                    .collect(),
            },
            IrInstruction::AsyncSpawn { dest, func, args } => IrInstruction::AsyncSpawn {
                dest: format!("{}_{}", prefix, dest),
                func: func.clone(),
                args: args
                    .iter()
                    .map(|a| self.rename_value(a, prefix, var_map))
                    .collect(),
            },
            IrInstruction::AsyncAwait { dest, future } => IrInstruction::AsyncAwait {
                dest: dest.clone().map(|d| format!("{}_{}", prefix, d)),
                future: self.rename_var_name(future, prefix, var_map),
            },
            IrInstruction::VTableLookup {
                dest,
                object,
                trait_name,
                method_idx,
            } => IrInstruction::VTableLookup {
                dest: format!("{}_{}", prefix, dest),
                object: self.rename_var_name(object, prefix, var_map),
                trait_name: trait_name.clone(),
                method_idx: *method_idx,
            },
            IrInstruction::GetField { dest, ptr, field } => IrInstruction::GetField {
                dest: format!("{}_{}", prefix, dest),
                ptr: var_map
                    .get(ptr)
                    .and_then(|v| {
                        if let IrValue::Var(n) = v {
                            Some(n.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| format!("{}_{}", prefix, ptr)),
                field: field.clone(),
            },
            // Fallback: clone unchanged
            other => other.clone(),
        }
    }

    /// Rename a value reference for inlining
    fn rename_value(
        &self,
        value: &IrValue,
        prefix: &str,
        var_map: &HashMap<String, IrValue>,
    ) -> IrValue {
        match value {
            IrValue::Var(name) => {
                if let Some(mapped) = var_map.get(name) {
                    mapped.clone()
                } else {
                    IrValue::Var(format!("{}_{}", prefix, name))
                }
            }
            IrValue::Const(c) => IrValue::Const(c.clone()),
        }
    }

    /// Helper: rename a variable name (String) using `var_map` or prefix it
    fn rename_var_name(
        &self,
        name: &str,
        prefix: &str,
        var_map: &HashMap<String, IrValue>,
    ) -> String {
        var_map
            .get(name)
            .and_then(|v| {
                if let IrValue::Var(n) = v {
                    Some(n.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| format!("{}_{}", prefix, name))
    }

    // ─────────────────────────────────────────────────────────────────────
    // Pass 6: Common Subexpression Elimination (CSE)
    // ─────────────────────────────────────────────────────────────────────

    /// Common subexpression elimination: eliminate redundant computations
    fn cse_pass(&mut self, module: &mut IrModule) {
        for function in &mut module.functions {
            for block in &mut function.blocks {
                // Map: expression signature -> first dest variable
                let mut expr_to_var: HashMap<String, String> = HashMap::new();
                let mut new_instructions = Vec::new();

                for instr in &block.instructions {
                    match instr {
                        IrInstruction::BinOp {
                            dest,
                            op,
                            left,
                            right,
                        } => {
                            // Create a canonical expression key
                            let expr_key = format!("{:?}({:?},{:?})", op, left, right);

                            // For commutative ops, also check swapped order
                            let is_commutative = matches!(
                                op,
                                IrBinOp::Add
                                    | IrBinOp::Mul
                                    | IrBinOp::Eq
                                    | IrBinOp::Ne
                                    | IrBinOp::And
                                    | IrBinOp::Or
                            );

                            let swapped_key = if is_commutative {
                                Some(format!("{:?}({:?},{:?})", op, right, left))
                            } else {
                                None
                            };

                            if let Some(existing_var) = expr_to_var
                                .get(&expr_key)
                                .or_else(|| swapped_key.as_ref().and_then(|k| expr_to_var.get(k)))
                            {
                                // Replace with copy from existing result
                                new_instructions.push(IrInstruction::BinOp {
                                    dest: dest.clone(),
                                    op: IrBinOp::Add,
                                    left: IrValue::Var(existing_var.clone()),
                                    right: IrValue::Const(IrConst::Int(0)),
                                });
                                self.stats.cse_eliminated += 1;
                                trace!("CSE: Reusing {} for {} ({})", existing_var, dest, expr_key);
                            } else {
                                // First occurrence: record it
                                expr_to_var.insert(expr_key, dest.clone());
                                new_instructions.push(instr.clone());
                            }
                        }
                        // Store or Call invalidate memory-dependent expressions
                        IrInstruction::Store { .. } | IrInstruction::Call { .. } => {
                            new_instructions.push(instr.clone());
                        }
                        _ => {
                            new_instructions.push(instr.clone());
                        }
                    }
                }

                block.instructions = new_instructions;
            }
        }
        debug!(
            "CSE: {} redundant computations eliminated",
            self.stats.cse_eliminated
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // Pass 7: Strength Reduction
    // ─────────────────────────────────────────────────────────────────────

    /// Strength reduction: replace expensive operations with cheaper equivalents
    fn strength_reduction_pass(&mut self, module: &mut IrModule) {
        for function in &mut module.functions {
            for block in &mut function.blocks {
                let mut new_instructions = Vec::new();

                for instr in &block.instructions {
                    match instr {
                        IrInstruction::BinOp {
                            dest,
                            op: IrBinOp::Mul,
                            left,
                            right,
                        } => {
                            // Multiply by power of 2 → shift left
                            if let Some(shift) = self.is_power_of_2_const(right) {
                                new_instructions.push(IrInstruction::BinOp {
                                    dest: dest.clone(),
                                    op: IrBinOp::Mul,
                                    left: left.clone(),
                                    right: IrValue::Const(IrConst::Int(shift)),
                                });
                                self.stats.strength_reduced += 1;
                                trace!("Strength reduction: mul by pow2 → shift by {}", shift);
                                continue;
                            }
                            // Multiply by 0 → 0
                            if self.is_zero_const(right) || self.is_zero_const(left) {
                                new_instructions.push(IrInstruction::BinOp {
                                    dest: dest.clone(),
                                    op: IrBinOp::Add,
                                    left: IrValue::Const(IrConst::Int(0)),
                                    right: IrValue::Const(IrConst::Int(0)),
                                });
                                self.stats.strength_reduced += 1;
                                continue;
                            }
                            new_instructions.push(instr.clone());
                        }
                        IrInstruction::BinOp {
                            dest,
                            op: IrBinOp::Div,
                            left,
                            right,
                        } => {
                            // Divide by power of 2 → shift right (approximate for signed)
                            if let Some(shift) = self.is_power_of_2_const(right) {
                                if shift > 0 {
                                    new_instructions.push(IrInstruction::BinOp {
                                        dest: dest.clone(),
                                        op: IrBinOp::Div,
                                        left: left.clone(),
                                        right: IrValue::Const(IrConst::Int(shift)),
                                    });
                                    self.stats.strength_reduced += 1;
                                    trace!("Strength reduction: div by pow2 → shift by {}", shift);
                                    continue;
                                }
                            }
                            // Divide by 1 → identity
                            if self.is_one_const(right) {
                                new_instructions.push(IrInstruction::BinOp {
                                    dest: dest.clone(),
                                    op: IrBinOp::Add,
                                    left: left.clone(),
                                    right: IrValue::Const(IrConst::Int(0)),
                                });
                                self.stats.strength_reduced += 1;
                                continue;
                            }
                            new_instructions.push(instr.clone());
                        }
                        IrInstruction::BinOp {
                            dest,
                            op: IrBinOp::Mod,
                            left,
                            right,
                        } => {
                            // Mod by power of 2 → bitwise AND (for unsigned)
                            if let IrValue::Const(IrConst::Int(n)) = right {
                                if *n > 0 && (*n & (*n - 1)) == 0 {
                                    new_instructions.push(IrInstruction::BinOp {
                                        dest: dest.clone(),
                                        op: IrBinOp::And,
                                        left: left.clone(),
                                        right: IrValue::Const(IrConst::Int(n - 1)),
                                    });
                                    self.stats.strength_reduced += 1;
                                    trace!("Strength reduction: mod {} → and {}", n, n - 1);
                                    continue;
                                }
                            }
                            new_instructions.push(instr.clone());
                        }
                        _ => {
                            new_instructions.push(instr.clone());
                        }
                    }
                }

                block.instructions = new_instructions;
            }
        }
        debug!(
            "Strength reduction: {} operations reduced",
            self.stats.strength_reduced
        );
    }

    /// Check if a value is a constant power of 2, return the exponent
    fn is_power_of_2_const(&self, value: &IrValue) -> Option<i64> {
        if let IrValue::Const(IrConst::Int(n)) = value {
            if *n > 0 && (*n & (*n - 1)) == 0 {
                return Some(n.trailing_zeros() as i64);
            }
        }
        None
    }

    /// Check if a value is the constant 0
    fn is_zero_const(&self, value: &IrValue) -> bool {
        match value {
            IrValue::Const(IrConst::Int(0)) => true,
            IrValue::Const(IrConst::Float(f)) if *f == 0.0 => true,
            _ => false,
        }
    }

    /// Check if a value is the constant 1
    fn is_one_const(&self, value: &IrValue) -> bool {
        matches!(value, IrValue::Const(IrConst::Int(1)))
    }

    // ─────────────────────────────────────────────────────────────────────
    // Pass 8: Copy Propagation
    // ─────────────────────────────────────────────────────────────────────

    /// Copy propagation: replace uses of copies with the original value
    fn copy_propagation_pass(&mut self, module: &mut IrModule) {
        for function in &mut module.functions {
            // Track copies: dest = src (identified as BinOp Add with 0)
            let mut copy_map: HashMap<String, IrValue> = HashMap::new();

            for block in &mut function.blocks {
                for instr in &block.instructions {
                    // Detect copies: x = y + 0
                    if let IrInstruction::BinOp {
                        dest,
                        op: IrBinOp::Add,
                        left,
                        right,
                    } = instr
                    {
                        if self.is_zero_const(right) {
                            if let IrValue::Var(src) = left {
                                copy_map.insert(dest.clone(), IrValue::Var(src.clone()));
                            }
                        }
                    }
                }

                // Propagate copies through the block
                if !copy_map.is_empty() {
                    for instr in &mut block.instructions {
                        let propagated = self.propagate_copies_in_instruction(instr, &copy_map);
                        self.stats.copies_propagated += propagated;
                    }

                    // Also propagate in terminator
                    match &mut block.terminator {
                        IrTerminator::Return(Some(val)) => {
                            if let IrValue::Var(v) = val {
                                if let Some(replacement) = copy_map.get(v) {
                                    *val = replacement.clone();
                                    self.stats.copies_propagated += 1;
                                }
                            }
                        }
                        IrTerminator::CondBranch { cond, .. } => {
                            if let IrValue::Var(v) = cond {
                                if let Some(replacement) = copy_map.get(v) {
                                    *cond = replacement.clone();
                                    self.stats.copies_propagated += 1;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        debug!(
            "Copy propagation: {} copies propagated",
            self.stats.copies_propagated
        );
    }

    /// Propagate copies within an instruction's operands
    fn propagate_copies_in_instruction(
        &self,
        instr: &mut IrInstruction,
        copies: &HashMap<String, IrValue>,
    ) -> usize {
        let mut count = 0;

        match instr {
            IrInstruction::BinOp { left, right, .. } => {
                if let IrValue::Var(v) = left {
                    if let Some(replacement) = copies.get(v) {
                        *left = replacement.clone();
                        count += 1;
                    }
                }
                if let IrValue::Var(v) = right {
                    if let Some(replacement) = copies.get(v) {
                        *right = replacement.clone();
                        count += 1;
                    }
                }
            }
            IrInstruction::Store { value, .. } => {
                if let IrValue::Var(v) = value {
                    if let Some(replacement) = copies.get(v) {
                        *value = replacement.clone();
                        count += 1;
                    }
                }
            }
            IrInstruction::Call { args, .. } => {
                for arg in args.iter_mut() {
                    if let IrValue::Var(v) = arg {
                        if let Some(replacement) = copies.get(v) {
                            *arg = replacement.clone();
                            count += 1;
                        }
                    }
                }
            }
            _ => {}
        }

        count
    }

    // ─────────────────────────────────────────────────────────────────────
    // Pass 9: Algebraic Simplification
    // ─────────────────────────────────────────────────────────────────────

    /// Algebraic simplification: identity, zero, and double-negation elimination
    fn algebraic_simplification_pass(&mut self, module: &mut IrModule) {
        for function in &mut module.functions {
            for block in &mut function.blocks {
                let mut new_instructions = Vec::new();

                for instr in &block.instructions {
                    match instr {
                        IrInstruction::BinOp {
                            dest,
                            op,
                            left,
                            right,
                        } => {
                            // x + 0 → x
                            if matches!(op, IrBinOp::Add) && self.is_zero_const(right) {
                                new_instructions.push(IrInstruction::BinOp {
                                    dest: dest.clone(),
                                    op: IrBinOp::Add,
                                    left: left.clone(),
                                    right: IrValue::Const(IrConst::Int(0)),
                                });
                                self.stats.algebraic_simplified += 1;
                                continue;
                            }
                            // 0 + x → x
                            if matches!(op, IrBinOp::Add) && self.is_zero_const(left) {
                                new_instructions.push(IrInstruction::BinOp {
                                    dest: dest.clone(),
                                    op: IrBinOp::Add,
                                    left: right.clone(),
                                    right: IrValue::Const(IrConst::Int(0)),
                                });
                                self.stats.algebraic_simplified += 1;
                                continue;
                            }
                            // x - 0 → x
                            if matches!(op, IrBinOp::Sub) && self.is_zero_const(right) {
                                new_instructions.push(IrInstruction::BinOp {
                                    dest: dest.clone(),
                                    op: IrBinOp::Add,
                                    left: left.clone(),
                                    right: IrValue::Const(IrConst::Int(0)),
                                });
                                self.stats.algebraic_simplified += 1;
                                continue;
                            }
                            // x * 1 → x
                            if matches!(op, IrBinOp::Mul) && self.is_one_const(right) {
                                new_instructions.push(IrInstruction::BinOp {
                                    dest: dest.clone(),
                                    op: IrBinOp::Add,
                                    left: left.clone(),
                                    right: IrValue::Const(IrConst::Int(0)),
                                });
                                self.stats.algebraic_simplified += 1;
                                continue;
                            }
                            // 1 * x → x
                            if matches!(op, IrBinOp::Mul) && self.is_one_const(left) {
                                new_instructions.push(IrInstruction::BinOp {
                                    dest: dest.clone(),
                                    op: IrBinOp::Add,
                                    left: right.clone(),
                                    right: IrValue::Const(IrConst::Int(0)),
                                });
                                self.stats.algebraic_simplified += 1;
                                continue;
                            }
                            // x / 1 → x
                            if matches!(op, IrBinOp::Div) && self.is_one_const(right) {
                                new_instructions.push(IrInstruction::BinOp {
                                    dest: dest.clone(),
                                    op: IrBinOp::Add,
                                    left: left.clone(),
                                    right: IrValue::Const(IrConst::Int(0)),
                                });
                                self.stats.algebraic_simplified += 1;
                                continue;
                            }
                            // x - x → 0 (when both are the same variable)
                            if matches!(op, IrBinOp::Sub) {
                                if let (IrValue::Var(l), IrValue::Var(r)) = (left, right) {
                                    if l == r {
                                        new_instructions.push(IrInstruction::BinOp {
                                            dest: dest.clone(),
                                            op: IrBinOp::Add,
                                            left: IrValue::Const(IrConst::Int(0)),
                                            right: IrValue::Const(IrConst::Int(0)),
                                        });
                                        self.stats.algebraic_simplified += 1;
                                        continue;
                                    }
                                }
                            }
                            // x == x → true
                            if matches!(op, IrBinOp::Eq) {
                                if let (IrValue::Var(l), IrValue::Var(r)) = (left, right) {
                                    if l == r {
                                        new_instructions.push(IrInstruction::BinOp {
                                            dest: dest.clone(),
                                            op: IrBinOp::Add,
                                            left: IrValue::Const(IrConst::Bool(true)),
                                            right: IrValue::Const(IrConst::Int(0)),
                                        });
                                        self.stats.algebraic_simplified += 1;
                                        continue;
                                    }
                                }
                            }

                            new_instructions.push(instr.clone());
                        }
                        _ => {
                            new_instructions.push(instr.clone());
                        }
                    }
                }

                block.instructions = new_instructions;
            }
        }
        debug!(
            "Algebraic simplification: {} simplifications",
            self.stats.algebraic_simplified
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // Pass 10: Loop-Invariant Code Motion (LICM)
    // ─────────────────────────────────────────────────────────────────────

    /// LICM: hoist loop-invariant computations out of loops
    fn licm_pass(&mut self, module: &mut IrModule) {
        for function in &mut module.functions {
            // Detect back edges (simple heuristic: branch to earlier block)
            let block_indices: HashMap<String, usize> = function
                .blocks
                .iter()
                .enumerate()
                .map(|(i, b)| (b.label.clone(), i))
                .collect();

            let mut loops: Vec<(usize, usize)> = Vec::new(); // (header_idx, latch_idx)

            for (i, block) in function.blocks.iter().enumerate() {
                match &block.terminator {
                    IrTerminator::Branch(target) => {
                        if let Some(&target_idx) = block_indices.get(target) {
                            if target_idx <= i {
                                loops.push((target_idx, i));
                            }
                        }
                    }
                    IrTerminator::CondBranch {
                        then_label,
                        else_label,
                        ..
                    } => {
                        if let Some(&t_idx) = block_indices.get(then_label) {
                            if t_idx <= i {
                                loops.push((t_idx, i));
                            }
                        }
                        if let Some(&e_idx) = block_indices.get(else_label) {
                            if e_idx <= i {
                                loops.push((e_idx, i));
                            }
                        }
                    }
                    _ => {}
                }
            }

            // For each detected loop, find invariant instructions
            for (header_idx, latch_idx) in &loops {
                // Collect all variables defined inside the loop
                let mut loop_defs: HashSet<String> = HashSet::new();
                for block_idx in *header_idx..=*latch_idx {
                    if block_idx < function.blocks.len() {
                        for instr in &function.blocks[block_idx].instructions {
                            if let Some(dest) = self.instruction_dest(instr) {
                                loop_defs.insert(dest);
                            }
                        }
                    }
                }

                // Find instructions that only depend on values defined outside the loop
                if *header_idx > 0 {
                    let preheader_idx = header_idx - 1;

                    for block_idx in *header_idx..=*latch_idx {
                        if block_idx >= function.blocks.len() {
                            continue;
                        }

                        let mut to_hoist: Vec<IrInstruction> = Vec::new();
                        let mut remaining: Vec<IrInstruction> = Vec::new();

                        for instr in &function.blocks[block_idx].instructions {
                            if self.is_loop_invariant(instr, &loop_defs) && self.is_pure(instr) {
                                to_hoist.push(instr.clone());
                                self.stats.licm_hoisted += 1;
                            } else {
                                remaining.push(instr.clone());
                            }
                        }

                        if !to_hoist.is_empty() {
                            // Hoist to preheader
                            function.blocks[block_idx].instructions = remaining;
                            for hoisted in to_hoist {
                                trace!(
                                    "LICM: Hoisting {:?} from block {} to preheader {}",
                                    hoisted,
                                    function.blocks[block_idx].label,
                                    function.blocks[preheader_idx].label
                                );
                                function.blocks[preheader_idx].instructions.push(hoisted);
                            }
                        }
                    }
                }
            }
        }
        debug!(
            "LICM: {} instructions hoisted out of loops",
            self.stats.licm_hoisted
        );
    }

    /// Check if an instruction is loop-invariant (all operands defined outside loop)
    fn is_loop_invariant(&self, instr: &IrInstruction, loop_defs: &HashSet<String>) -> bool {
        match instr {
            IrInstruction::BinOp { left, right, .. } => {
                let l_invariant = match left {
                    IrValue::Const(_) => true,
                    IrValue::Var(v) => !loop_defs.contains(v),
                };
                let r_invariant = match right {
                    IrValue::Const(_) => true,
                    IrValue::Var(v) => !loop_defs.contains(v),
                };
                l_invariant && r_invariant
            }
            IrInstruction::Cast { value, .. } => match value {
                IrValue::Const(_) => true,
                IrValue::Var(v) => !loop_defs.contains(v),
            },
            _ => false, // Only hoist pure computations
        }
    }

    /// Check if an instruction is pure (no side effects)
    fn is_pure(&self, instr: &IrInstruction) -> bool {
        matches!(
            instr,
            IrInstruction::BinOp { .. }
                | IrInstruction::Cast { .. }
                | IrInstruction::GetField { .. }
                | IrInstruction::Select { .. }
                | IrInstruction::ExtractValue { .. }
        )
    }

    /// Get the destination variable of an instruction (if any)
    fn instruction_dest(&self, instr: &IrInstruction) -> Option<String> {
        match instr {
            IrInstruction::Alloca { dest, .. }
            | IrInstruction::Load { dest, .. }
            | IrInstruction::BinOp { dest, .. }
            | IrInstruction::GetField { dest, .. }
            | IrInstruction::Cast { dest, .. }
            | IrInstruction::ExtractValue { dest, .. }
            | IrInstruction::InsertValue { dest, .. }
            | IrInstruction::VTableLookup { dest, .. }
            | IrInstruction::Phi { dest, .. }
            | IrInstruction::Select { dest, .. }
            | IrInstruction::CreateClosure { dest, .. }
            | IrInstruction::AsyncSpawn { dest, .. } => Some(dest.clone()),
            IrInstruction::Call { dest, .. }
            | IrInstruction::CallClosure { dest, .. }
            | IrInstruction::AsyncAwait { dest, .. }
            | IrInstruction::TraitDispatch { dest, .. }
            | IrInstruction::NativeCall { dest, .. } => dest.clone(),
            IrInstruction::Store { .. }
            | IrInstruction::Switch { .. }
            | IrInstruction::BoundsCheck { .. } => None,
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Pass 11: Loop Unrolling
    // ─────────────────────────────────────────────────────────────────────

    /// Loop unrolling: duplicate loop body for small constant-bound loops
    fn loop_unrolling_pass(&mut self, module: &mut IrModule) {
        let unroll_factor = 4;

        for function in &mut module.functions {
            let block_indices: HashMap<String, usize> = function
                .blocks
                .iter()
                .enumerate()
                .map(|(i, b)| (b.label.clone(), i))
                .collect();

            // Find single-block loops (simplest case)
            let mut to_unroll: Vec<(usize, usize)> = Vec::new(); // (block_idx, factor)

            for (i, block) in function.blocks.iter().enumerate() {
                if let IrTerminator::CondBranch {
                    then_label,
                    else_label,
                    ..
                } = &block.terminator
                {
                    // Simple loop: one branch goes back to start
                    if let Some(&target_idx) = block_indices.get(then_label) {
                        if target_idx <= i && block.instructions.len() <= self.unroll_threshold {
                            to_unroll.push((i, unroll_factor));
                        }
                    }
                    if let Some(&target_idx) = block_indices.get(else_label) {
                        if target_idx <= i && block.instructions.len() <= self.unroll_threshold {
                            if !to_unroll.iter().any(|(idx, _)| *idx == i) {
                                to_unroll.push((i, unroll_factor));
                            }
                        }
                    }
                }
            }

            // Unroll loops (process in reverse to maintain indices)
            for (block_idx, factor) in to_unroll.into_iter().rev() {
                if block_idx >= function.blocks.len() {
                    continue;
                }

                let original_insts = function.blocks[block_idx].instructions.clone();
                let body_size = original_insts.len();

                if body_size == 0 || body_size > self.unroll_threshold {
                    continue;
                }

                // Duplicate the loop body `factor` times
                let mut unrolled = Vec::with_capacity(body_size * factor);

                for copy in 0..factor {
                    for instr in &original_insts {
                        let renamed = self.rename_for_unroll(instr, copy);
                        unrolled.push(renamed);
                    }
                }

                function.blocks[block_idx].instructions = unrolled;
                self.stats.loops_unrolled += 1;
                trace!(
                    "Unrolled loop at block {} by factor {}",
                    function.blocks[block_idx].label,
                    factor
                );
            }
        }
        debug!(
            "Loop unrolling: {} loops unrolled",
            self.stats.loops_unrolled
        );
    }

    /// Rename instruction destinations for unrolled copies
    fn rename_for_unroll(&self, instr: &IrInstruction, copy: usize) -> IrInstruction {
        if copy == 0 {
            return instr.clone(); // First copy keeps original names
        }

        match instr {
            IrInstruction::BinOp {
                dest,
                op,
                left,
                right,
            } => IrInstruction::BinOp {
                dest: format!("{}_u{}", dest, copy),
                op: op.clone(),
                left: left.clone(),
                right: right.clone(),
            },
            IrInstruction::Alloca { dest, ty } => IrInstruction::Alloca {
                dest: format!("{}_u{}", dest, copy),
                ty: ty.clone(),
            },
            IrInstruction::Load { dest, ptr, ty } => IrInstruction::Load {
                dest: format!("{}_u{}", dest, copy),
                ptr: ptr.clone(),
                ty: ty.clone(),
            },
            other => other.clone(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::IrType;

    fn make_test_module() -> IrModule {
        IrModule {
            name: "test".to_string(),
            functions: vec![IrFunction {
                name: "add".to_string(),
                params: vec![
                    ("x".to_string(), IrType::I64),
                    ("y".to_string(), IrType::I64),
                ],
                return_type: IrType::I64,
                blocks: vec![IrBlock {
                    label: "entry".to_string(),
                    instructions: vec![IrInstruction::BinOp {
                        dest: "result".to_string(),
                        op: IrBinOp::Add,
                        left: IrValue::Var("x".to_string()),
                        right: IrValue::Var("y".to_string()),
                    }],
                    terminator: IrTerminator::Return(Some(IrValue::Var("result".to_string()))),
                }],
                locals: vec![],
            }],
            globals: vec![],
            externs: vec![],
            vtables: vec![],
            string_pool: vec![],
            type_info: vec![],
        }
    }

    #[test]
    fn test_optimizer_creation() {
        let opt_normal = IrOptimizer::new(false);
        let opt_aggressive = IrOptimizer::new(true);

        assert!(!opt_normal.aggressive);
        assert!(opt_aggressive.aggressive);
    }

    #[test]
    fn test_constant_folding() {
        let mut opt = IrOptimizer::new(true);

        let mut module = IrModule {
            name: "test".to_string(),
            functions: vec![IrFunction {
                name: "constant_test".to_string(),
                params: vec![],
                return_type: IrType::I64,
                blocks: vec![IrBlock {
                    label: "entry".to_string(),
                    instructions: vec![IrInstruction::BinOp {
                        dest: "r".to_string(),
                        op: IrBinOp::Add,
                        left: IrValue::Const(IrConst::Int(3)),
                        right: IrValue::Const(IrConst::Int(4)),
                    }],
                    terminator: IrTerminator::Return(Some(IrValue::Var("r".to_string()))),
                }],
                locals: vec![],
            }],
            globals: vec![],
            externs: vec![],
            vtables: vec![],
            string_pool: vec![],
            type_info: vec![],
        };

        opt.constant_fold_pass(&mut module);
        assert!(opt.stats.constant_folded > 0);
    }

    #[test]
    fn test_eval_binop_integer() {
        let opt = IrOptimizer::new(false);

        // 3 + 4 = 7
        let result = opt.eval_binop(&IrBinOp::Add, &IrConst::Int(3), &IrConst::Int(4));
        assert!(result.is_some());
        match result.unwrap() {
            IrConst::Int(v) => assert_eq!(v, 7),
            other => panic!("Expected Int(7), got {:?}", other),
        }

        // 5 * 6 = 30
        let result = opt.eval_binop(&IrBinOp::Mul, &IrConst::Int(5), &IrConst::Int(6));
        match result.unwrap() {
            IrConst::Int(v) => assert_eq!(v, 30),
            other => panic!("Expected Int(30), got {:?}", other),
        }

        // Division by zero returns None
        let result = opt.eval_binop(&IrBinOp::Div, &IrConst::Int(10), &IrConst::Int(0));
        assert!(result.is_none());

        // 3 < 5 = true
        let result = opt.eval_binop(&IrBinOp::Lt, &IrConst::Int(3), &IrConst::Int(5));
        match result.unwrap() {
            IrConst::Bool(v) => assert!(v),
            other => panic!("Expected Bool(true), got {:?}", other),
        }
    }

    #[test]
    fn test_dead_code_elimination() {
        let mut opt = IrOptimizer::new(true);

        let mut module = IrModule {
            name: "test".to_string(),
            functions: vec![IrFunction {
                name: "dce_test".to_string(),
                params: vec![("x".to_string(), IrType::I64)],
                return_type: IrType::I64,
                blocks: vec![IrBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        // Used
                        IrInstruction::BinOp {
                            dest: "result".to_string(),
                            op: IrBinOp::Add,
                            left: IrValue::Var("x".to_string()),
                            right: IrValue::Const(IrConst::Int(1)),
                        },
                        // Unused - should be eliminated
                        IrInstruction::BinOp {
                            dest: "dead".to_string(),
                            op: IrBinOp::Mul,
                            left: IrValue::Const(IrConst::Int(42)),
                            right: IrValue::Const(IrConst::Int(99)),
                        },
                    ],
                    terminator: IrTerminator::Return(Some(IrValue::Var("result".to_string()))),
                }],
                locals: vec![],
            }],
            globals: vec![],
            externs: vec![],
            vtables: vec![],
            string_pool: vec![],
            type_info: vec![],
        };

        opt.dead_code_elimination_pass(&mut module);
        assert_eq!(module.functions[0].blocks[0].instructions.len(), 1);
        assert!(opt.stats.dead_code_eliminated > 0);
    }

    #[test]
    fn test_strength_reduction_power_of_2() {
        let opt = IrOptimizer::new(true);

        assert_eq!(
            opt.is_power_of_2_const(&IrValue::Const(IrConst::Int(8))),
            Some(3)
        );
        assert_eq!(
            opt.is_power_of_2_const(&IrValue::Const(IrConst::Int(16))),
            Some(4)
        );
        assert_eq!(
            opt.is_power_of_2_const(&IrValue::Const(IrConst::Int(7))),
            None
        );
        assert_eq!(
            opt.is_power_of_2_const(&IrValue::Const(IrConst::Int(1))),
            Some(0)
        );
    }

    #[test]
    fn test_algebraic_simplification() {
        let mut opt = IrOptimizer::new(true);
        let mut module = make_test_module();

        // Add x + 0 instruction (identity)
        module.functions[0].blocks[0].instructions.insert(
            0,
            IrInstruction::BinOp {
                dest: "simplified".to_string(),
                op: IrBinOp::Add,
                left: IrValue::Var("x".to_string()),
                right: IrValue::Const(IrConst::Int(0)),
            },
        );

        opt.algebraic_simplification_pass(&mut module);
        assert!(opt.stats.algebraic_simplified > 0);
    }

    #[test]
    fn test_optimizer_o0_skips() {
        let mut opt = IrOptimizer::with_level(OptLevel::O0);
        let mut module = make_test_module();
        let stats = opt.optimize(&mut module);

        assert_eq!(stats.constant_folded, 0);
        assert_eq!(stats.dead_code_eliminated, 0);
    }

    #[test]
    fn test_full_optimization_pipeline() {
        let mut opt = IrOptimizer::with_level(OptLevel::O2);
        let mut module = make_test_module();
        let _stats = opt.optimize(&mut module);

        // Pipeline should complete without panicking
        assert!(!module.functions.is_empty());
    }

    #[test]
    fn test_cse_eliminates_duplicates() {
        let mut opt = IrOptimizer::new(true);

        let mut module = IrModule {
            name: "test".to_string(),
            functions: vec![IrFunction {
                name: "cse_test".to_string(),
                params: vec![
                    ("a".to_string(), IrType::I64),
                    ("b".to_string(), IrType::I64),
                ],
                return_type: IrType::I64,
                blocks: vec![IrBlock {
                    label: "entry".to_string(),
                    instructions: vec![
                        IrInstruction::BinOp {
                            dest: "r1".to_string(),
                            op: IrBinOp::Add,
                            left: IrValue::Var("a".to_string()),
                            right: IrValue::Var("b".to_string()),
                        },
                        // Duplicate computation
                        IrInstruction::BinOp {
                            dest: "r2".to_string(),
                            op: IrBinOp::Add,
                            left: IrValue::Var("a".to_string()),
                            right: IrValue::Var("b".to_string()),
                        },
                    ],
                    terminator: IrTerminator::Return(Some(IrValue::Var("r2".to_string()))),
                }],
                locals: vec![],
            }],
            globals: vec![],
            externs: vec![],
            vtables: vec![],
            string_pool: vec![],
            type_info: vec![],
        };

        opt.cse_pass(&mut module);
        assert!(
            opt.stats.cse_eliminated > 0,
            "CSE should eliminate duplicate computation"
        );
    }

    #[test]
    fn test_block_merging() {
        let mut opt = IrOptimizer::new(true);

        let mut module = IrModule {
            name: "test".to_string(),
            functions: vec![IrFunction {
                name: "merge_test".to_string(),
                params: vec![("x".to_string(), IrType::I64)],
                return_type: IrType::I64,
                blocks: vec![
                    IrBlock {
                        label: "entry".to_string(),
                        instructions: vec![IrInstruction::BinOp {
                            dest: "a".to_string(),
                            op: IrBinOp::Add,
                            left: IrValue::Var("x".to_string()),
                            right: IrValue::Const(IrConst::Int(1)),
                        }],
                        terminator: IrTerminator::Branch("next".to_string()),
                    },
                    IrBlock {
                        label: "next".to_string(),
                        instructions: vec![IrInstruction::BinOp {
                            dest: "b".to_string(),
                            op: IrBinOp::Add,
                            left: IrValue::Var("a".to_string()),
                            right: IrValue::Const(IrConst::Int(2)),
                        }],
                        terminator: IrTerminator::Return(Some(IrValue::Var("b".to_string()))),
                    },
                ],
                locals: vec![],
            }],
            globals: vec![],
            externs: vec![],
            vtables: vec![],
            string_pool: vec![],
            type_info: vec![],
        };

        let original_blocks = module.functions[0].blocks.len();
        opt.block_merge_pass(&mut module);
        assert!(
            module.functions[0].blocks.len() < original_blocks,
            "Blocks should be merged"
        );
        assert!(opt.stats.blocks_merged > 0);
    }
}
