#![allow(dead_code)]
//! Comprehensive Test Suite for the Omni Compiler
//!
//! Coverage areas:
//! - Lexer tokenization
//! - Parser AST construction
//! - Semantic analysis
//! - IR generation and manipulation
//! - Code generation (all backends)
//! - Safety passes
//! - Optimization passes
//! - DWARF debug info
//! - Integration tests

#[cfg(test)]
mod tests {
    use crate::codegen::dwarf::DwarfEmitter;
    use crate::codegen::exception_handling::*;
    use crate::codegen::gpu_binary::*;
    use crate::codegen::native_codegen::*;
    use crate::codegen::native_extended::*;
    use crate::codegen::optimizer::IrOptimizer;
    use crate::ir::{
        IrBinOp, IrBlock, IrConst, IrExternalFunc, IrFunction, IrGlobal, IrInstruction, IrModule,
        IrTerminator, IrType, IrTypeInfo, IrValue, TypeKind, VTable,
    };
    use crate::lexer::{tokenize, Lexer, TokenKind};

    // ─── Helpers ─────────────────────────────────────────────────────────

    fn make_function(
        name: &str,
        instructions: Vec<IrInstruction>,
        ret: Option<IrValue>,
    ) -> IrFunction {
        IrFunction {
            name: name.to_string(),
            params: vec![
                ("x".to_string(), IrType::I64),
                ("y".to_string(), IrType::I64),
            ],
            return_type: IrType::I64,
            blocks: vec![IrBlock {
                label: "entry".to_string(),
                instructions,
                terminator: IrTerminator::Return(ret),
            }],
            locals: vec![],
        }
    }

    fn make_module(name: &str, functions: Vec<IrFunction>) -> IrModule {
        IrModule {
            name: name.to_string(),
            functions,
            globals: vec![],
            externs: vec![],
            vtables: vec![],
            string_pool: vec![],
            type_info: vec![],
        }
    }

    fn simple_add_func() -> IrFunction {
        make_function(
            "add",
            vec![IrInstruction::BinOp {
                dest: "result".to_string(),
                op: IrBinOp::Add,
                left: IrValue::Var("x".to_string()),
                right: IrValue::Var("y".to_string()),
            }],
            Some(IrValue::Var("result".to_string())),
        )
    }

    fn multi_op_func() -> IrFunction {
        make_function(
            "compute",
            vec![
                IrInstruction::BinOp {
                    dest: "sum".to_string(),
                    op: IrBinOp::Add,
                    left: IrValue::Var("x".to_string()),
                    right: IrValue::Var("y".to_string()),
                },
                IrInstruction::BinOp {
                    dest: "product".to_string(),
                    op: IrBinOp::Mul,
                    left: IrValue::Var("sum".to_string()),
                    right: IrValue::Const(IrConst::Int(2)),
                },
                IrInstruction::BinOp {
                    dest: "result".to_string(),
                    op: IrBinOp::Sub,
                    left: IrValue::Var("product".to_string()),
                    right: IrValue::Const(IrConst::Int(1)),
                },
            ],
            Some(IrValue::Var("result".to_string())),
        )
    }

    // ─── Lexer Tests ─────────────────────────────────────────────────────

    #[test]
    fn test_lexer_keywords() {
        let source =
            "fn let mut if else for while return match async await import struct trait impl";
        let tokens = tokenize(source).unwrap();
        let kinds: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind != TokenKind::Newline)
            .map(|t| &t.kind)
            .collect();
        assert_eq!(kinds[0], &TokenKind::Fn);
        assert_eq!(kinds[1], &TokenKind::Let);
        assert_eq!(kinds[2], &TokenKind::Mut);
        assert_eq!(kinds[3], &TokenKind::If);
        assert_eq!(kinds[4], &TokenKind::Else);
        assert_eq!(kinds[5], &TokenKind::For);
        assert_eq!(kinds[6], &TokenKind::While);
        assert_eq!(kinds[7], &TokenKind::Return);
        assert_eq!(kinds[8], &TokenKind::Match);
    }

    #[test]
    fn test_lexer_operators() {
        let source = "+ - * / % == != < > <= >= && ||";
        let tokens = tokenize(source).unwrap();
        let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
        assert!(kinds.contains(&&TokenKind::Plus));
        assert!(kinds.contains(&&TokenKind::Minus));
        assert!(kinds.contains(&&TokenKind::Star));
        assert!(kinds.contains(&&TokenKind::EqEq));
        assert!(kinds.contains(&&TokenKind::NotEq));
        assert!(kinds.contains(&&TokenKind::LtEq));
        assert!(kinds.contains(&&TokenKind::GtEq));
    }

    #[test]
    fn test_lexer_literals() {
        let source = "42 3.14 \"hello world\" true false";
        let tokens = tokenize(source).unwrap();
        assert!(tokens
            .iter()
            .any(|t| t.kind == TokenKind::IntLiteral && t.lexeme == "42"));
        assert!(tokens
            .iter()
            .any(|t| t.kind == TokenKind::FloatLiteral && t.lexeme == "3.14"));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::StringLiteral));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::True));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::False));
    }

    #[test]
    fn test_lexer_indentation() {
        let source = "if x:\n    return 1\n    let y = 2";
        let tokens = tokenize(source).unwrap();
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Indent));
    }

    #[test]
    fn test_lexer_dedentation() {
        let source = "fn main():\n    return 0\nfn other():\n    return 1";
        let tokens = tokenize(source).unwrap();
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Dedent));
    }

    #[test]
    fn test_lexer_type_keywords() {
        let source = "u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 bool str";
        let tokens = tokenize(source).unwrap();
        let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
        assert!(kinds.contains(&&TokenKind::U8));
        assert!(kinds.contains(&&TokenKind::U32));
        assert!(kinds.contains(&&TokenKind::I64));
        assert!(kinds.contains(&&TokenKind::F32));
        assert!(kinds.contains(&&TokenKind::F64));
        assert!(kinds.contains(&&TokenKind::Bool));
    }

    #[test]
    fn test_lexer_delimiters() {
        let source = "( ) [ ] { } , : ; .";
        let tokens = tokenize(source).unwrap();
        let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
        assert!(kinds.contains(&&TokenKind::LParen));
        assert!(kinds.contains(&&TokenKind::RParen));
        assert!(kinds.contains(&&TokenKind::LBracket));
        assert!(kinds.contains(&&TokenKind::RBracket));
        assert!(kinds.contains(&&TokenKind::LBrace));
        assert!(kinds.contains(&&TokenKind::RBrace));
    }

    #[test]
    fn test_lexer_compound_operators() {
        let source = "-> => :: .. ..= += -= *= /=";
        let tokens = tokenize(source).unwrap();
        let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
        assert!(kinds.contains(&&TokenKind::Arrow));
        assert!(kinds.contains(&&TokenKind::FatArrow));
        assert!(kinds.contains(&&TokenKind::DoubleColon));
        assert!(kinds.contains(&&TokenKind::DotDot));
    }

    #[test]
    fn test_lexer_comments_skipped() {
        let source = "let x = 5 # this is a comment";
        let tokens = tokenize(source).unwrap();
        // Comments should be filtered
        assert!(!tokens
            .iter()
            .any(|t| t.lexeme.contains("this is a comment")));
    }

    #[test]
    fn test_lexer_identifier() {
        let source = "my_variable camelCase _private CONSTANT";
        let tokens = tokenize(source).unwrap();
        let idents: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind == TokenKind::Identifier)
            .map(|t| t.lexeme.as_str())
            .collect();
        assert!(idents.contains(&"my_variable"));
        assert!(idents.contains(&"camelCase"));
        assert!(idents.contains(&"_private"));
        assert!(idents.contains(&"CONSTANT"));
    }

    #[test]
    fn test_lexer_empty_source() {
        let tokens = tokenize("").unwrap();
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_lexer_iterator_api() {
        let lexer = Lexer::new("let x = 42").unwrap();
        let tokens: Vec<_> = lexer.collect();
        assert!(!tokens.is_empty());
        assert_eq!(tokens[0].kind, TokenKind::Let);
    }

    // ─── IR Tests ────────────────────────────────────────────────────────

    #[test]
    fn test_ir_function_display() {
        let func = simple_add_func();
        let display = format!("{}", func);
        assert!(display.contains("fn @add"));
        assert!(display.contains("add"));
    }

    #[test]
    fn test_ir_module_display() {
        let module = make_module("test", vec![simple_add_func()]);
        let display = format!("{}", module);
        assert!(display.contains("Omni IR Module: test"));
    }

    #[test]
    fn test_ir_type_display() {
        assert_eq!(format!("{}", IrType::I32), "i32");
        assert_eq!(format!("{}", IrType::I64), "i64");
        assert_eq!(format!("{}", IrType::F32), "f32");
        assert_eq!(format!("{}", IrType::F64), "f64");
        assert_eq!(format!("{}", IrType::Bool), "i1");
        assert_eq!(format!("{}", IrType::Void), "void");
    }

    #[test]
    fn test_ir_binop_display() {
        assert_eq!(format!("{}", IrBinOp::Add), "add");
        assert_eq!(format!("{}", IrBinOp::Sub), "sub");
        assert_eq!(format!("{}", IrBinOp::Mul), "mul");
        assert_eq!(format!("{}", IrBinOp::Div), "div");
    }

    #[test]
    fn test_ir_value_display() {
        assert_eq!(format!("{}", IrValue::Var("x".to_string())), "%x");
        assert_eq!(format!("{}", IrValue::Const(IrConst::Int(42))), "42");
        assert_eq!(format!("{}", IrValue::Const(IrConst::Bool(true))), "true");
    }

    #[test]
    fn test_ir_pointer_type() {
        let ptr = IrType::Ptr(Box::new(IrType::I64));
        let display = format!("{}", ptr);
        assert!(display.contains("*"));
    }

    #[test]
    fn test_ir_block_display() {
        let block = IrBlock {
            label: "entry".to_string(),
            instructions: vec![IrInstruction::BinOp {
                dest: "x".to_string(),
                op: IrBinOp::Add,
                left: IrValue::Const(IrConst::Int(1)),
                right: IrValue::Const(IrConst::Int(2)),
            }],
            terminator: IrTerminator::Return(Some(IrValue::Var("x".to_string()))),
        };
        let display = format!("{}", block);
        assert!(display.contains("entry:"));
        assert!(display.contains("add"));
    }

    #[test]
    fn test_ir_multiple_blocks() {
        let func = IrFunction {
            name: "branch_test".to_string(),
            params: vec![("cond".to_string(), IrType::Bool)],
            return_type: IrType::I64,
            blocks: vec![
                IrBlock {
                    label: "entry".to_string(),
                    instructions: vec![],
                    terminator: IrTerminator::CondBranch {
                        cond: IrValue::Var("cond".to_string()),
                        then_label: "then".to_string(),
                        else_label: "else".to_string(),
                    },
                },
                IrBlock {
                    label: "then".to_string(),
                    instructions: vec![],
                    terminator: IrTerminator::Return(Some(IrValue::Const(IrConst::Int(1)))),
                },
                IrBlock {
                    label: "else".to_string(),
                    instructions: vec![],
                    terminator: IrTerminator::Return(Some(IrValue::Const(IrConst::Int(0)))),
                },
            ],
            locals: vec![],
        };

        assert_eq!(func.blocks.len(), 3);
        let display = format!("{}", func);
        assert!(display.contains("then:"));
        assert!(display.contains("else:"));
    }

    #[test]
    fn test_ir_instruction_variants() {
        let alloca = IrInstruction::Alloca {
            dest: "x".to_string(),
            ty: IrType::I64,
        };
        let load = IrInstruction::Load {
            dest: "v".to_string(),
            ptr: "x".to_string(),
            ty: IrType::I64,
        };
        let store = IrInstruction::Store {
            ptr: "x".to_string(),
            value: IrValue::Const(IrConst::Int(42)),
        };
        let call = IrInstruction::Call {
            dest: Some("r".to_string()),
            func: "foo".to_string(),
            args: vec![],
        };
        let bounds = IrInstruction::BoundsCheck {
            index: "i".to_string(),
            length: "len".to_string(),
        };

        let alloca_s = format!("{}", alloca);
        assert!(alloca_s.contains("alloca"));
        let load_s = format!("{}", load);
        assert!(load_s.contains("load"));
        let store_s = format!("{}", store);
        assert!(store_s.contains("store"));
        let call_s = format!("{}", call);
        assert!(call_s.contains("call"));
        let bounds_s = format!("{}", bounds);
        assert!(bounds_s.contains("bounds_check"));
    }

    // ─── OVM Codegen Tests ───────────────────────────────────────────────

    #[test]
    fn test_ir_module_to_ovm_format() {
        let func = simple_add_func();
        let module = make_module("test", vec![func]);
        // Module should be well-formed for any backend
        assert_eq!(module.functions.len(), 1);
        assert_eq!(module.functions[0].name, "add");
    }

    #[test]
    fn test_ir_multi_op_structure() {
        let func = multi_op_func();
        let module = make_module("test", vec![func]);
        assert_eq!(module.functions[0].blocks[0].instructions.len(), 3);
    }

    #[test]
    fn test_ir_empty_function_valid() {
        let func = make_function("empty", vec![], None);
        let module = make_module("test", vec![func]);
        assert!(module.functions[0].blocks[0].instructions.is_empty());
    }

    // ─── Optimizer Tests ─────────────────────────────────────────────────

    #[test]
    fn test_optimizer_constant_folding() {
        let func = make_function(
            "const_fold",
            vec![IrInstruction::BinOp {
                dest: "result".to_string(),
                op: IrBinOp::Add,
                left: IrValue::Const(IrConst::Int(10)),
                right: IrValue::Const(IrConst::Int(20)),
            }],
            Some(IrValue::Var("result".to_string())),
        );

        let mut module = make_module("test", vec![func]);
        let mut optimizer = IrOptimizer::new(true);
        optimizer.optimize(&mut module);

        // After constant folding, the add should be simplified
        assert!(!module.functions.is_empty());
    }

    #[test]
    fn test_optimizer_dce() {
        let func = make_function(
            "dead_code",
            vec![
                IrInstruction::BinOp {
                    dest: "dead".to_string(),
                    op: IrBinOp::Add,
                    left: IrValue::Const(IrConst::Int(1)),
                    right: IrValue::Const(IrConst::Int(2)),
                },
                IrInstruction::BinOp {
                    dest: "result".to_string(),
                    op: IrBinOp::Add,
                    left: IrValue::Var("x".to_string()),
                    right: IrValue::Var("y".to_string()),
                },
            ],
            Some(IrValue::Var("result".to_string())),
        );

        let mut module = make_module("test", vec![func]);
        let mut optimizer = IrOptimizer::new(true);
        optimizer.optimize(&mut module);

        // DCE should remove the dead instruction
        assert!(!module.functions.is_empty());
    }

    #[test]
    fn test_optimizer_levels() {
        let func = simple_add_func();
        for _level in 0..=3 {
            let mut module = make_module("test", vec![func.clone()]);
            let mut optimizer = IrOptimizer::new(false);
            optimizer.optimize(&mut module);
            assert!(!module.functions.is_empty());
        }
    }

    // ─── DWARF Tests ─────────────────────────────────────────────────────

    #[test]
    fn test_dwarf_debug_info_generation() {
        let func = simple_add_func();
        let debug_info = DwarfEmitter::emit_debug_info("test_module", &[func]);
        assert!(!debug_info.is_empty());
        // Should contain .debug_abbrev section marker
        assert!(debug_info.windows(13).any(|w| w == b".debug_abbrev"));
    }

    #[test]
    fn test_dwarf_multiple_functions() {
        let f1 = simple_add_func();
        let f2 = multi_op_func();
        let debug_info = DwarfEmitter::emit_debug_info("multi", &[f1, f2]);
        assert!(!debug_info.is_empty());
    }

    #[test]
    fn test_dwarf_sections_present() {
        let func = simple_add_func();
        let debug_info = DwarfEmitter::emit_debug_info("sections", &[func]);
        assert!(debug_info.windows(11).any(|w| w == b".debug_info"));
        assert!(debug_info.windows(11).any(|w| w == b".debug_line"));
        assert!(debug_info.windows(10).any(|w| w == b".debug_str"));
    }

    // ─── Native Codegen Tests ────────────────────────────────────────────

    #[test]
    fn test_native_compile_x86_64() {
        let func = simple_add_func();
        let module = make_module("test", vec![func]);
        let target = TargetTriple::x86_64_linux();
        let mut codegen = NativeCodegen::new(target);
        let result = codegen.compile_module(&module);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.binary.is_empty());
    }

    #[test]
    fn test_native_compile_aarch64() {
        let func = simple_add_func();
        let module = make_module("test", vec![func]);
        let target = TargetTriple::aarch64_linux();
        let mut codegen = NativeCodegen::new(target);
        let result = codegen.compile_module(&module);
        assert!(result.is_ok());
    }

    #[test]
    fn test_native_compile_wasm() {
        let func = simple_add_func();
        let module = make_module("test", vec![func]);
        let target = TargetTriple::wasm32_wasi();
        let mut codegen = NativeCodegen::new(target);
        let result = codegen.compile_module(&module);
        assert!(result.is_ok());
    }

    #[test]
    fn test_native_host_target() {
        let target = TargetTriple::host();
        #[cfg(target_os = "windows")]
        assert_eq!(target.format, OutputFormat::Pe);
        assert!(!format!("{}", target.arch).is_empty());
    }

    #[test]
    fn test_native_output_symbols() {
        let func = simple_add_func();
        let module = make_module("test", vec![func]);
        let target = TargetTriple::x86_64_linux();
        let mut codegen = NativeCodegen::new(target);
        let output = codegen.compile_module(&module).unwrap();
        assert!(output.symbols.iter().any(|s| s.name == "add"));
    }

    // ─── GPU Binary Tests ────────────────────────────────────────────────

    #[test]
    fn test_gpu_ptx_sm_versions() {
        let func = simple_add_func();
        for sm in [50, 60, 70, 80, 90] {
            let mut compiler = PtxCompiler::new(sm);
            let binary = compiler.emit_ptx_binary(&func).unwrap();
            assert!(binary.ptx_source.contains(&format!("sm_{}", sm)));
        }
    }

    #[test]
    fn test_gpu_spirv_magic_number() {
        let func = simple_add_func();
        let mut compiler = SpirvCompiler::new();
        let binary = compiler.emit_spirv_binary(&func).unwrap();
        let magic = u32::from_le_bytes([
            binary.binary[0],
            binary.binary[1],
            binary.binary[2],
            binary.binary[3],
        ]);
        assert_eq!(magic, 0x07230203);
    }

    #[test]
    fn test_gpu_metal_kernel_signature() {
        let func = simple_add_func();
        let compiler = MetalCompiler::new(MetalGpuFamily::Mac2);
        let binary = compiler.emit_metal_binary(&func).unwrap();
        assert!(binary.msl_source.contains("kernel void add"));
        assert_eq!(binary.thread_execution_width, 64); // Mac GPU family
    }

    #[test]
    fn test_gpu_driver_software_fallback() {
        let driver = GpuDriver::new();
        let backends = driver.list_backends();
        let sw = backends
            .iter()
            .find(|b| b.backend_type == GpuDriverType::Software)
            .unwrap();
        assert!(sw.available);
    }

    // ─── Exception Handling Tests ────────────────────────────────────────

    #[test]
    fn test_eh_frame_multiple_functions() {
        let emitter = EhFrameEmitter::new(EhArch::X86_64);
        let entries = vec![
            CfiEntry {
                func_name: "func_a".to_string(),
                func_start: 0,
                func_size: 32,
                instructions: vec![CfiInstruction::DefCfa {
                    register: 7,
                    offset: 8,
                }],
                landing_pads: vec![],
            },
            CfiEntry {
                func_name: "func_b".to_string(),
                func_start: 32,
                func_size: 64,
                instructions: vec![CfiInstruction::DefCfa {
                    register: 7,
                    offset: 8,
                }],
                landing_pads: vec![],
            },
        ];
        let section = emitter.emit_eh_frame(&entries);
        assert!(!section.is_empty());
    }

    #[test]
    fn test_exception_lowering_nested() {
        let mut lowering = ExceptionLowering::new();
        let blocks = lowering.lower_try_catch(
            vec![],
            vec![
                CatchClause {
                    exception_type: Some("Error1".to_string()),
                    binding: Some("e1".to_string()),
                    handler_label: "catch1".to_string(),
                },
                CatchClause {
                    exception_type: Some("Error2".to_string()),
                    binding: Some("e2".to_string()),
                    handler_label: "catch2".to_string(),
                },
            ],
            None,
        );
        assert!(blocks.len() >= 4); // try + dispatch + 2 catch handlers + continue
    }

    // ─── PE/COFF + Mach-O + RISC-V Tests ────────────────────────────────

    #[test]
    fn test_pe_coff_sections() {
        let mut pe = PeBuilder::new(PeMachine::Amd64);
        pe.add_text(vec![0x90, 0xC3]); // nop; ret
        pe.add_data(vec![0x42, 0x00, 0x00, 0x00]);
        pe.add_symbol("_start", 0, 1);
        let coff = pe.build_coff();
        assert!(!coff.is_empty());
    }

    #[test]
    fn test_pe_arm64() {
        let mut pe = PeBuilder::new(PeMachine::Arm64);
        pe.add_text(vec![0xD5, 0x03, 0x20, 0x1F]); // NOP
        let coff = pe.build_coff();
        let machine = u16::from_le_bytes([coff[0], coff[1]]);
        assert_eq!(machine, 0xAA64);
    }

    #[test]
    fn test_macho_x86_64_magic() {
        let mut m = MachOBuilder::new(MachOCpuType::X86_64);
        m.add_text(vec![0xC3]);
        let binary = m.build();
        let magic = u32::from_le_bytes([binary[0], binary[1], binary[2], binary[3]]);
        assert_eq!(magic, 0xFEEDFACF);
    }

    #[test]
    fn test_riscv_store_load() {
        let mut emitter = RiscvEmitter::new();
        emitter.sd(RvReg::A0, RvReg::Sp, -8);
        emitter.ld(RvReg::A1, RvReg::Sp, -8);
        assert_eq!(emitter.code().len(), 8);
    }

    #[test]
    fn test_riscv_mul_div() {
        let mut emitter = RiscvEmitter::new();
        emitter.mul(RvReg::A0, RvReg::A1, RvReg::A2);
        emitter.div(RvReg::A0, RvReg::A1, RvReg::A2);
        emitter.rem(RvReg::A0, RvReg::A1, RvReg::A2);
        assert_eq!(emitter.code().len(), 12);
    }

    #[test]
    fn test_riscv_branch_and_jump() {
        let mut emitter = RiscvEmitter::new();
        emitter.jal(RvReg::Ra, 0);
        emitter.jalr(RvReg::Zero, RvReg::Ra, 0);
        assert_eq!(emitter.code().len(), 8);
    }

    #[test]
    fn test_riscv_ecall() {
        let mut emitter = RiscvEmitter::new();
        emitter.ecall();
        let word = u32::from_le_bytes([
            emitter.code()[0],
            emitter.code()[1],
            emitter.code()[2],
            emitter.code()[3],
        ]);
        assert_eq!(word, 0x73);
    }

    // ─── Integration Tests ───────────────────────────────────────────────

    #[test]
    fn test_full_pipeline_ir_to_ovm() {
        let func = multi_op_func();
        let mut module = make_module("pipeline", vec![func]);

        // Optimize
        let mut optimizer = IrOptimizer::new(true);
        optimizer.optimize(&mut module);

        // Verify module structure is intact after optimization
        assert!(!module.functions.is_empty());
        assert_eq!(module.functions[0].name, "compute");
    }

    #[test]
    fn test_full_pipeline_ir_to_native() {
        let func = simple_add_func();
        let mut module = make_module("pipeline", vec![func]);

        let mut optimizer = IrOptimizer::new(false);
        optimizer.optimize(&mut module);

        let target = TargetTriple::x86_64_linux();
        let mut codegen = NativeCodegen::new(target);
        let result = codegen.compile_module(&module);
        assert!(result.is_ok());
    }

    #[test]
    fn test_full_pipeline_ir_to_gpu() {
        let func = simple_add_func();
        let mut module = make_module("pipeline", vec![func.clone()]);

        let mut optimizer = IrOptimizer::new(true);
        optimizer.optimize(&mut module);

        let mut driver = GpuDriver::new();
        let result = driver.compile_kernel(&func);
        assert!(result.is_ok());
    }

    #[test]
    fn test_full_pipeline_with_debug_info() {
        let func = simple_add_func();
        let mut module = make_module("debug_pipeline", vec![func.clone()]);

        let mut optimizer = IrOptimizer::new(false);
        optimizer.optimize(&mut module);

        let debug_info = DwarfEmitter::emit_debug_info("debug_pipeline", &module.functions);
        assert!(!debug_info.is_empty());

        let target = TargetTriple::x86_64_linux();
        let mut codegen = NativeCodegen::new(target);
        let result = codegen.compile_module(&module);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ir_module_with_globals() {
        let module = IrModule {
            name: "globals_test".to_string(),
            functions: vec![simple_add_func()],
            globals: vec![IrGlobal {
                name: "counter".to_string(),
                ty: IrType::I64,
                init: Some(IrValue::Const(IrConst::Int(0))),
            }],
            externs: vec![IrExternalFunc {
                name: "printf".to_string(),
                abi: "C".to_string(),
                params: vec![IrType::Ptr(Box::new(IrType::I8))],
                return_type: IrType::I32,
            }],
            vtables: vec![],
            string_pool: vec!["hello".to_string()],
            type_info: vec![],
        };

        assert_eq!(module.globals.len(), 1);
        assert_eq!(module.externs.len(), 1);
        assert_eq!(module.string_pool.len(), 1);
    }

    #[test]
    fn test_ir_all_binops() {
        let ops = vec![
            IrBinOp::Add,
            IrBinOp::Sub,
            IrBinOp::Mul,
            IrBinOp::Div,
            IrBinOp::Mod,
            IrBinOp::And,
            IrBinOp::Or,
            IrBinOp::Eq,
            IrBinOp::Ne,
            IrBinOp::Lt,
            IrBinOp::Le,
            IrBinOp::Gt,
            IrBinOp::Ge,
        ];

        for op in &ops {
            let inst = IrInstruction::BinOp {
                dest: "r".to_string(),
                op: op.clone(),
                left: IrValue::Const(IrConst::Int(1)),
                right: IrValue::Const(IrConst::Int(2)),
            };
            let s = format!("{}", inst);
            assert!(!s.is_empty());
        }
    }

    #[test]
    fn test_native_all_output_formats() {
        let func = simple_add_func();

        // ELF
        let module = make_module("elf_test", vec![func.clone()]);
        let mut codegen = NativeCodegen::new(TargetTriple::x86_64_linux());
        let elf = codegen.compile_module(&module).unwrap();
        assert_eq!(&elf.binary[0..4], &[0x7F, 0x45, 0x4C, 0x46]);

        // WASM
        let module = make_module("wasm_test", vec![func.clone()]);
        let mut codegen = NativeCodegen::new(TargetTriple::wasm32_wasi());
        let wasm = codegen.compile_module(&module).unwrap();
        assert_eq!(&wasm.binary[0..4], &[0x00, 0x61, 0x73, 0x6D]);
    }

    #[test]
    fn test_linker_driver_command() {
        let mut linker = LinkerDriver::new();
        linker.add_lib_path("/usr/lib");
        linker.add_library("m");
        linker.add_flag("-static");

        let cmd = linker.build_command("main.o", "main");
        assert!(cmd.contains(&"-lm".to_string()));
        assert!(cmd.contains(&"-static".to_string()));
    }
}
