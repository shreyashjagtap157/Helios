#[cfg(test)]
mod tests {
    use crate::codegen::opt::{
        const_prop::ConstantPropagation, dce::DeadCodeElimination, OptimizationPass,
    };
    use crate::ir::{
        IrBinOp, IrBlock, IrConst, IrFunction, IrInstruction, IrTerminator, IrType, IrValue,
    };

    fn create_test_function(name: &str, instructions: Vec<IrInstruction>) -> IrFunction {
        IrFunction {
            name: name.to_string(),
            params: vec![],
            return_type: IrType::Void,
            blocks: vec![IrBlock {
                label: "entry".to_string(),
                instructions,
                terminator: IrTerminator::Return(None),
            }],
            locals: vec![],
        }
    }

    #[test]
    fn test_constant_propagation() {
        // %1 = 5
        // %2 = 10
        // %3 = add %1, %2  -> should become 15
        let instructions = vec![
            IrInstruction::BinOp {
                dest: "v1".to_string(),
                op: IrBinOp::Add,
                left: IrValue::Const(IrConst::Int(2)),
                right: IrValue::Const(IrConst::Int(3)),
            }, // v1 = 5
            IrInstruction::BinOp {
                dest: "v2".to_string(),
                op: IrBinOp::Add,
                left: IrValue::Const(IrConst::Int(5)),
                right: IrValue::Const(IrConst::Int(5)),
            }, // v2 = 10
            IrInstruction::BinOp {
                dest: "v3".to_string(),
                op: IrBinOp::Add,
                left: IrValue::Var("v1".to_string()),
                right: IrValue::Var("v2".to_string()),
            }, // v3 = v1 + v2
        ];

        let mut func = create_test_function("test_const_prop", instructions);
        let pass = ConstantPropagation;
        let changed = pass.run(&mut func);

        assert!(
            changed,
            "Constant propagation should have changed the function"
        );

        // Check if v3 is now adding constants
        let block = &func.blocks[0];
        let v3_inst = &block.instructions[2];

        if let IrInstruction::BinOp { left, right, .. } = v3_inst {
            assert!(
                matches!(left, IrValue::Const(IrConst::Int(5))),
                "Left operand should be const 5"
            );
            assert!(
                matches!(right, IrValue::Const(IrConst::Int(10))),
                "Right operand should be const 10"
            );
        } else {
            panic!("Expected BinOp for v3");
        }
    }

    #[test]
    fn test_dead_code_elimination() {
        // v1 = 5 + 5
        // v2 = v1 + 10  (unused)
        // return
        let instructions = vec![
            IrInstruction::BinOp {
                dest: "v1".to_string(),
                op: IrBinOp::Add,
                left: IrValue::Const(IrConst::Int(5)),
                right: IrValue::Const(IrConst::Int(5)),
            },
            IrInstruction::BinOp {
                dest: "v2".to_string(),
                op: IrBinOp::Add,
                left: IrValue::Var("v1".to_string()),
                right: IrValue::Const(IrConst::Int(10)),
            },
        ];

        let mut func = create_test_function("test_dce", instructions);
        let pass = DeadCodeElimination;
        let changed = pass.run(&mut func);

        assert!(changed, "DCE should have removed unused instructions");

        // v2 should be gone. v1 should be gone (recursively), unless v1 is used in terminator?
        // In our create_test_function, terminator returns Void (None).
        // So both should be removed.

        assert_eq!(
            func.blocks[0].instructions.len(),
            0,
            "All instructions should be removed"
        );
    }

    #[test]
    fn test_dce_keeps_side_effects() {
        // call foo() (side effect)
        // v1 = 5 + 5 (unused)
        let instructions = vec![
            IrInstruction::Call {
                dest: None,
                func: "foo".to_string(),
                args: vec![],
            },
            IrInstruction::BinOp {
                dest: "v1".to_string(),
                op: IrBinOp::Add,
                left: IrValue::Const(IrConst::Int(5)),
                right: IrValue::Const(IrConst::Int(5)),
            },
        ];

        let mut func = create_test_function("test_dce_effects", instructions);
        let pass = DeadCodeElimination;
        pass.run(&mut func);

        assert_eq!(func.blocks[0].instructions.len(), 1, "Should keep the call");
        assert!(matches!(
            func.blocks[0].instructions[0],
            IrInstruction::Call { .. }
        ));
    }

    #[test]
    fn test_licm() {
        use crate::ir::{IrBlock, IrTerminator};

        // CFG: entry -> loop -> exit
        // loop has back-edge to itself

        let mut func = IrFunction {
            name: "test_licm".to_string(),
            params: vec![],
            return_type: IrType::Void,
            blocks: vec![
                IrBlock {
                    label: "entry".to_string(),
                    instructions: vec![],
                    terminator: IrTerminator::Branch("loop".to_string()),
                },
                IrBlock {
                    label: "loop".to_string(),
                    instructions: vec![
                        // Invariant: constant expression
                        IrInstruction::BinOp {
                            dest: "v1".to_string(),
                            op: IrBinOp::Add,
                            left: IrValue::Const(IrConst::Int(5)),
                            right: IrValue::Const(IrConst::Int(5)),
                        },
                        // Loop variant: depends on phi (simulated) or just not invariant
                        IrInstruction::BinOp {
                            dest: "v2".to_string(),
                            op: IrBinOp::Add,
                            left: IrValue::Var("v1".to_string()),
                            right: IrValue::Const(IrConst::Int(1)),
                        },
                    ],
                    terminator: IrTerminator::CondBranch {
                        cond: IrValue::Const(IrConst::Bool(true)), // infinite loop for test
                        then_label: "loop".to_string(),
                        else_label: "exit".to_string(),
                    },
                },
                IrBlock {
                    label: "exit".to_string(),
                    instructions: vec![],
                    terminator: IrTerminator::Return(None),
                },
            ],
            locals: vec![],
        };

        let pass = crate::codegen::opt::licm::LoopInvariantCodeMotion;
        let changed = pass.run(&mut func);

        assert!(changed, "LICM should have hoisted instructions");

        // Check entry block for hoisted instruction v1
        let entry_insts = &func.blocks[0].instructions;
        assert_eq!(
            entry_insts.len(),
            1,
            "Entry block should have 1 hoisted instruction"
        );
        if let IrInstruction::BinOp { dest, .. } = &entry_insts[0] {
            assert_eq!(dest, "v1", "v1 should be hoisted");
        } else {
            panic!("Expected BinOp hoisted in entry");
        }

        // Loop block should have v2 but not v1
        let loop_insts = &func.blocks[1].instructions;
        assert_eq!(
            loop_insts.len(),
            1,
            "Loop block should have 1 instruction left"
        );
        if let IrInstruction::BinOp { dest, .. } = &loop_insts[0] {
            assert_eq!(dest, "v2", "v2 should remain in loop");
        }
    }
}
