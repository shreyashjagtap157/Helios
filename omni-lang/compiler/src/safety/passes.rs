//! Compiler Safety Passes
//!
//! Static analysis for bounds check elimination and unsafe code auditing.
#![allow(dead_code)]

use crate::ir::{IrBinOp, IrConst, IrFunction, IrInstruction, IrValue};
use std::collections::{HashMap, HashSet};

/// Bounds Check Elimination (BCE)
///
/// Removes redundant array bounds checks when static analysis can prove
/// the index is always within [0, length).
pub struct BoundsCheckElimination;

impl BoundsCheckElimination {
    pub fn run(func: &mut IrFunction) -> usize {
        let mut eliminated = 0;
        let mut known_ranges: HashMap<String, (i64, i64)> = HashMap::new();
        let mut known_lengths: HashMap<String, i64> = HashMap::new();
        let mut constants: HashMap<String, i64> = HashMap::new();
        let mut checked: HashSet<(String, String)> = HashSet::new();

        // Pass 1: Collect constants and known values from instructions
        for block in &func.blocks {
            for inst in &block.instructions {
                match inst {
                    // Track constant assignments: %x = const 5
                    IrInstruction::Store { ptr, value } => {
                        if let IrValue::Const(IrConst::Int(v)) = value {
                            constants.insert(ptr.clone(), *v);
                            known_ranges.insert(ptr.clone(), (*v, *v));
                        }
                    }
                    // Track alloca sizes for arrays: alloca [N x T]
                    IrInstruction::Alloca { dest, ty } => {
                        if let crate::ir::IrType::Array(_, size) = ty {
                            known_lengths.insert(dest.clone(), *size as i64);
                        }
                    }
                    // Track PHI nodes (loop induction variables)
                    IrInstruction::Phi { dest, .. } => {
                        known_ranges.insert(dest.clone(), (0, i64::MAX)); // Conservative
                    }
                    // Track binary ops that refine ranges
                    IrInstruction::BinOp {
                        dest,
                        op,
                        left,
                        right,
                    } => {
                        let l_key = match left {
                            IrValue::Var(name) => Some(name.clone()),
                            _ => None,
                        };
                        let r_key = match right {
                            IrValue::Var(name) => Some(name.clone()),
                            _ => None,
                        };
                        let l_range = l_key.as_ref().and_then(|k| known_ranges.get(k)).copied();
                        let r_range = r_key.as_ref().and_then(|k| known_ranges.get(k)).copied();

                        if let (Some((l_lo, l_hi)), Some((r_lo, r_hi))) = (l_range, r_range) {
                            let new_range = match op {
                                IrBinOp::Add => {
                                    (l_lo.saturating_add(r_lo), l_hi.saturating_add(r_hi))
                                }
                                IrBinOp::Sub => {
                                    (l_lo.saturating_sub(r_hi), l_hi.saturating_sub(r_lo))
                                }
                                IrBinOp::Mul if l_lo >= 0 && r_lo >= 0 => {
                                    (l_lo.saturating_mul(r_lo), l_hi.saturating_mul(r_hi))
                                }
                                IrBinOp::Mod if r_lo > 0 => (0, r_hi - 1), // x % n is in [0, n-1]
                                IrBinOp::And if r_lo >= 0 => (0, r_hi),    // x & mask ≤ mask
                                _ => (i64::MIN, i64::MAX),
                            };
                            known_ranges.insert(dest.clone(), new_range);
                        } else if let Some(r_val) = r_key.as_ref().and_then(|k| constants.get(k)) {
                            if let Some((_l_lo, _l_hi)) = l_range {
                                let new_range = match op {
                                    IrBinOp::Mod if *r_val > 0 => (0, r_val - 1),
                                    IrBinOp::And if *r_val >= 0 => (0, *r_val),
                                    _ => (i64::MIN, i64::MAX),
                                };
                                known_ranges.insert(dest.clone(), new_range);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Pass 2: Eliminate provably-safe bounds checks
        for block in &mut func.blocks {
            let mut new_insts = Vec::new();

            for inst in &block.instructions {
                match inst {
                    IrInstruction::BoundsCheck { index, length } => {
                        if Self::is_safe(index, length, &known_ranges, &known_lengths, &constants) {
                            eliminated += 1;
                            // Skip emission — check is provably unnecessary
                        } else if checked.contains(&(index.clone(), length.clone())) {
                            // Already checked this exact (index, length) pair — deduplicate
                            eliminated += 1;
                        } else {
                            checked.insert((index.clone(), length.clone()));
                            new_insts.push(inst.clone());
                        }
                    }
                    _ => new_insts.push(inst.clone()),
                }
            }
            block.instructions = new_insts;
        }
        eliminated
    }

    fn is_safe(
        index: &str,
        length: &str,
        ranges: &HashMap<String, (i64, i64)>,
        known_lengths: &HashMap<String, i64>,
        constants: &HashMap<String, i64>,
    ) -> bool {
        // Get the index range
        let idx_range = if let Some(&range) = ranges.get(index) {
            range
        } else if let Some(&v) = constants.get(index) {
            (v, v)
        } else {
            return false; // Unknown index — keep the check
        };

        // Get the length value
        let len_val = if let Some(&v) = constants.get(length) {
            v
        } else if let Some(&v) = known_lengths.get(length) {
            v
        } else {
            return false; // Unknown length — keep the check
        };

        // Safe if: 0 ≤ index_min AND index_max < length
        idx_range.0 >= 0 && idx_range.1 < len_val
    }
}

/// Unsafe Code Auditor
/// Scans source files for `unsafe` blocks and reports locations.
pub struct UnsafeAuditor;

#[derive(Debug)]
pub struct UnsafeReport {
    pub file: String,
    pub line: usize,
    pub reason: String,
}

impl UnsafeAuditor {
    /// Walk the crate directory, find files with `unsafe` blocks, report them.
    pub fn audit_crate(root: &std::path::Path) -> Vec<UnsafeReport> {
        let mut reports = Vec::new();
        Self::walk_dir(root, &mut reports);
        reports
    }

    fn walk_dir(dir: &std::path::Path, reports: &mut Vec<UnsafeReport>) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Skip hidden directories and common non-source dirs
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !name.starts_with('.') && name != "target" && name != "node_modules" {
                    Self::walk_dir(&path, reports);
                }
            } else if path.extension().and_then(|e| e.to_str()) == Some("omni")
                || path.extension().and_then(|e| e.to_str()) == Some("rs")
            {
                Self::audit_file(&path, reports);
            }
        }
    }

    fn audit_file(path: &std::path::Path, reports: &mut Vec<UnsafeReport>) {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return,
        };
        let file_str = path.to_string_lossy().to_string();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Detect unsafe blocks
            if trimmed.contains("unsafe {")
                || trimmed.contains("unsafe:")
                || trimmed.starts_with("unsafe ")
            {
                reports.push(UnsafeReport {
                    file: file_str.clone(),
                    line: line_num + 1,
                    reason: format!(
                        "Unsafe block: {}",
                        trimmed.chars().take(80).collect::<String>()
                    ),
                });
            }

            // Detect raw pointer operations
            if trimmed.contains("*mut ") || trimmed.contains("*const ") {
                reports.push(UnsafeReport {
                    file: file_str.clone(),
                    line: line_num + 1,
                    reason: format!(
                        "Raw pointer usage: {}",
                        trimmed.chars().take(80).collect::<String>()
                    ),
                });
            }

            // Detect FFI declarations
            if trimmed.contains("extern \"C\"") || trimmed.contains("extern \"system\"") {
                reports.push(UnsafeReport {
                    file: file_str.clone(),
                    line: line_num + 1,
                    reason: format!(
                        "FFI boundary: {}",
                        trimmed.chars().take(80).collect::<String>()
                    ),
                });
            }
        }
    }
}
