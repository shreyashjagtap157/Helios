#[cfg(test)]
mod tests {
    use crate::parser::ast::*;
    use crate::semantic::{Analyzer, SemanticError, TypedExprKind, TypedItem};

    #[test]
    fn test_analyzer_creation() {
        let _analyzer = Analyzer::new();
        // Basic instantiation test
    }

    #[test]
    fn test_lookup_builtin_println() {
        let analyzer = Analyzer::new();
        let result = analyzer.lookup_symbol("println");
        assert!(result.is_ok());
    }

    #[test]
    fn test_lookup_builtin_exit() {
        let analyzer = Analyzer::new();
        let result = analyzer.lookup_symbol("exit");
        assert!(result.is_ok());
    }

    #[test]
    fn test_lookup_undefined_fails() {
        let analyzer = Analyzer::new();
        let result = analyzer.lookup_symbol("undefined_xyz");
        assert!(result.is_err());
    }

    #[test]
    fn test_define_and_lookup_symbol() {
        let mut analyzer = Analyzer::new();
        let _ = analyzer.define_symbol("test_var".to_string(), Type::I64, false);
        let result = analyzer.lookup_symbol("test_var");
        assert!(result.is_ok());
    }

    #[test]
    fn test_unify_identical_types() {
        let mut analyzer = Analyzer::new();
        let result = analyzer.unify(&Type::I64, &Type::I64);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unify_different_types_fails() {
        let mut analyzer = Analyzer::new();
        let result = analyzer.unify(&Type::I64, &Type::Str);
        assert!(result.is_err());
    }

    #[test]
    fn test_static_declaration_analysis() {
        let mut analyzer = Analyzer::new();

        let static_item = Item::Static(StaticDecl {
            name: "g_count".to_string(),
            mutable: true,
            attributes: Vec::new(),
            ty: Type::I64,
            value: Expression::Literal(Literal::Int(0)),
        });

        let main_fn = Item::Function(Function {
            name: "main".to_string(),
            is_async: false,
            attributes: Vec::new(),
            params: Vec::new(),
            return_type: None,
            effect_row: None,
            body: Block {
                statements: Vec::new(),
            },
        });

        let module = Module {
            items: vec![static_item, main_fn],
        };

        let typed = analyzer.analyze(module).unwrap();
        assert!(typed
            .items
            .iter()
            .any(|item| matches!(item, TypedItem::Static(s) if s.name == "g_count")));
    }

    #[test]
    fn test_function_effect_annotation_analysis() {
        let mut analyzer = Analyzer::new();

        let fn_item = Item::Function(Function {
            name: "uses_io".to_string(),
            is_async: false,
            attributes: Vec::new(),
            params: Vec::new(),
            return_type: Some(Type::Named("()".to_string())),
            effect_row: Some(EffectRow {
                effects: vec![EffectSymbol {
                    name: "IO".to_string(),
                    param: None,
                }],
            }),
            body: Block {
                statements: vec![Statement::Expression(Expression::Call(
                    Box::new(Expression::Identifier("println".to_string())),
                    vec![Expression::Literal(Literal::String("hello".to_string()))],
                ))],
            },
        });

        let typed = analyzer
            .analyze(Module {
                items: vec![fn_item],
            })
            .expect("expected effect-annotated function to analyze");

        let typed_fn = typed
            .items
            .iter()
            .find_map(|item| match item {
                TypedItem::Function(function) if function.name == "uses_io" => Some(function),
                _ => None,
            })
            .expect("expected typed function");

        let effect_row = typed_fn.effect_row.as_ref().expect("expected effect row");
        let effect_names: Vec<_> = effect_row
            .iter()
            .map(|effect| effect.name.as_str())
            .collect();
        assert_eq!(effect_names, vec!["IO"]);
    }

    #[test]
    fn test_unify_numeric_coercion() {
        let mut analyzer = Analyzer::new();
        let result = analyzer.unify(&Type::I32, &Type::I64);
        // Should succeed with coercion
        assert!(result.is_ok());
    }

    #[test]
    fn test_literal_analysis_int() {
        let mut analyzer = Analyzer::new();
        let lit = Expression::Literal(Literal::Int(42));
        let result = analyzer.analyze_expression(&lit);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap().ty, Type::I64));
    }

    #[test]
    fn test_literal_analysis_float() {
        let mut analyzer = Analyzer::new();
        let lit = Expression::Literal(Literal::Float(3.14));
        let result = analyzer.analyze_expression(&lit);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap().ty, Type::F64));
    }

    #[test]
    fn test_literal_analysis_bool() {
        let mut analyzer = Analyzer::new();
        let lit = Expression::Literal(Literal::Bool(true));
        let result = analyzer.analyze_expression(&lit);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap().ty, Type::Bool));
    }

    #[test]
    fn test_literal_analysis_string() {
        let mut analyzer = Analyzer::new();
        let lit = Expression::Literal(Literal::String("test".to_string()));
        let result = analyzer.analyze_expression(&lit);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap().ty, Type::Str));
    }

    #[test]
    fn test_let_chain_expression_analysis() {
        let mut analyzer = Analyzer::new();
        let expr = Expression::LetChain {
            name: "x".to_string(),
            value: Box::new(Expression::Literal(Literal::Int(41))),
            body: Box::new(Expression::Binary(
                Box::new(Expression::Identifier("x".to_string())),
                BinaryOp::Add,
                Box::new(Expression::Literal(Literal::Int(1))),
            )),
        };

        let result = analyzer
            .analyze_expression(&expr)
            .expect("expected let-chain expression to analyze");

        assert!(matches!(result.ty, Type::I64));
        assert!(matches!(result.kind, TypedExprKind::LetChain { .. }));
    }

    #[test]
    fn test_binary_addition() {
        let mut analyzer = Analyzer::new();
        let expr = Expression::Binary(
            Box::new(Expression::Literal(Literal::Int(1))),
            BinaryOp::Add,
            Box::new(Expression::Literal(Literal::Int(2))),
        );
        let result = analyzer.analyze_expression(&expr);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap().ty, Type::I64));
    }

    #[test]
    fn test_binary_comparison() {
        let mut analyzer = Analyzer::new();
        let expr = Expression::Binary(
            Box::new(Expression::Literal(Literal::Int(1))),
            BinaryOp::Lt,
            Box::new(Expression::Literal(Literal::Int(2))),
        );
        let result = analyzer.analyze_expression(&expr);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap().ty, Type::Bool));
    }

    #[test]
    fn test_linear_parameter_must_be_consumed() {
        let mut analyzer = Analyzer::new();
        let result = analyzer.analyze(Module {
            items: vec![Item::Function(Function {
                name: "consume".to_string(),
                is_async: false,
                attributes: vec![],
                params: vec![Param {
                    name: "token".to_string(),
                    ty: Type::Named("Token".to_string()),
                    modifier: ParamModifier::Linear,
                }],
                return_type: None,
                effect_row: None,
                body: Block {
                    statements: vec![Statement::Pass],
                },
            })],
        });

        assert!(matches!(
            result,
            Err(SemanticError::LinearityError(message)) if message.contains("token")
        ));
    }

    #[test]
    fn test_linear_parameter_can_be_consumed_once() {
        let mut analyzer = Analyzer::new();
        let result = analyzer.analyze(Module {
            items: vec![Item::Function(Function {
                name: "consume".to_string(),
                is_async: false,
                attributes: vec![],
                params: vec![Param {
                    name: "token".to_string(),
                    ty: Type::Named("Token".to_string()),
                    modifier: ParamModifier::Linear,
                }],
                return_type: Some(Type::Named("Token".to_string())),
                effect_row: None,
                body: Block {
                    statements: vec![Statement::Return(Some(Expression::Identifier(
                        "token".to_string(),
                    )))],
                },
            })],
        });

        assert!(result.is_ok(), "linear parameter should be consumable once");
    }

    #[test]
    fn test_linear_parameter_cannot_be_borrowed() {
        let mut analyzer = Analyzer::new();
        let result = analyzer.analyze(Module {
            items: vec![Item::Function(Function {
                name: "consume".to_string(),
                is_async: false,
                attributes: vec![],
                params: vec![Param {
                    name: "token".to_string(),
                    ty: Type::Named("Token".to_string()),
                    modifier: ParamModifier::Linear,
                }],
                return_type: None,
                effect_row: None,
                body: Block {
                    statements: vec![Statement::Expression(Expression::Borrow {
                        mutable: false,
                        expr: Box::new(Expression::Identifier("token".to_string())),
                    })],
                },
            })],
        });

        assert!(matches!(
            result,
            Err(SemanticError::LinearityError(message)) if message.contains("borrowed")
        ));
    }

    #[test]
    fn test_linear_parameter_used_twice_errors() {
        let mut analyzer = Analyzer::new();
        let result = analyzer.analyze(Module {
            items: vec![Item::Function(Function {
                name: "consume".to_string(),
                is_async: false,
                attributes: vec![],
                params: vec![Param {
                    name: "token".to_string(),
                    ty: Type::Named("Token".to_string()),
                    modifier: ParamModifier::Linear,
                }],
                return_type: Some(Type::Named("Token".to_string())),
                effect_row: None,
                body: Block {
                    statements: vec![
                        Statement::Expression(Expression::Identifier("token".to_string())),
                        Statement::Return(Some(Expression::Identifier("token".to_string()))),
                    ],
                },
            })],
        });

        assert!(matches!(
            result,
            Err(SemanticError::LinearityError(message)) if message.contains("more than once")
        ));
    }

    #[test]
    fn test_monomorphize_function() {
        let mut analyzer = Analyzer::new();
        let result = analyzer.monomorphize("test_fn", &[Type::I64]);
        assert!(result.is_ok());
        let mangled = result.unwrap();
        assert!(mangled.contains("test_fn"));
    }

    #[test]
    fn test_monomorphize_different_types() {
        let mut analyzer = Analyzer::new();
        let name1 = analyzer.monomorphize("id", &[Type::I64]).unwrap();
        let name2 = analyzer.monomorphize("id", &[Type::Str]).unwrap();
        assert_ne!(name1, name2);
    }

    #[test]
    fn test_monomorphize_caching() {
        let mut analyzer = Analyzer::new();
        let name1 = analyzer.monomorphize("id", &[Type::I64]).unwrap();
        let name2 = analyzer.monomorphize("id", &[Type::I64]).unwrap();
        assert_eq!(name1, name2);
    }

    #[test]
    fn test_monomorphize_with_body_generation() {
        let mut analyzer = Analyzer::new();
        // Create a simple generic function and store it
        let generic_func = Function {
            name: "identity".to_string(),
            is_async: false,
            attributes: vec![],
            params: vec![Param {
                name: "x".to_string(),
                ty: Type::Named("T".to_string()),
                modifier: ParamModifier::Normal,
            }],
            return_type: Some(Type::Named("T".to_string())),
            effect_row: None,
            body: Block {
                statements: vec![Statement::Return(Some(Expression::Identifier(
                    "x".to_string(),
                )))],
            },
        };

        // Store the generic function
        analyzer
            .generic_functions
            .insert("identity".to_string(), generic_func);

        // Monomorphize with concrete type
        let mangled = analyzer.monomorphize("identity", &[Type::I64]).unwrap();

        // Verify it created a monomorphized entry
        assert!(analyzer.monomorphized.contains_key(&mangled));

        // The typed_body should be generated
        let mono_func = &analyzer.monomorphized[&mangled];
        assert_eq!(mono_func.original_name, "identity");
        assert_eq!(mono_func.type_args.len(), 1);
        assert!(matches!(mono_func.type_args[0], Type::I64));
    }

    #[test]
    fn test_trait_bound_copy_i64() {
        let analyzer = Analyzer::new();
        let result = analyzer.check_trait_bound(&Type::I64, "Copy");
        assert!(result.is_ok());
    }

    #[test]
    fn test_trait_bound_copy_str() {
        let analyzer = Analyzer::new();
        let result = analyzer.check_trait_bound(&Type::Str, "Copy");
        assert!(result.is_err());
    }

    #[test]
    fn test_type_implements_copy_primitives() {
        let analyzer = Analyzer::new();
        assert!(analyzer.type_implements_copy(&Type::I32));
        assert!(analyzer.type_implements_copy(&Type::F64));
        assert!(analyzer.type_implements_copy(&Type::Bool));
    }

    #[test]
    fn test_type_implements_copy_str() {
        let analyzer = Analyzer::new();
        assert!(!analyzer.type_implements_copy(&Type::Str));
    }

    #[test]
    fn test_type_to_string() {
        let analyzer = Analyzer::new();
        assert_eq!(analyzer.type_to_string(&Type::I64), "i64");
        assert_eq!(analyzer.type_to_string(&Type::Str), "str");
        assert_eq!(analyzer.type_to_string(&Type::Bool), "bool");
    }

    #[test]
    fn test_array_type_unification() {
        let mut analyzer = Analyzer::new();
        let arr1 = Type::Array(Box::new(Type::I64), None);
        let arr2 = Type::Array(Box::new(Type::I64), None);
        let result = analyzer.unify(&arr1, &arr2);
        assert!(result.is_ok());
    }

    #[test]
    fn test_array_type_mismatch() {
        let mut analyzer = Analyzer::new();
        let arr1 = Type::Array(Box::new(Type::I64), None);
        let arr2 = Type::Array(Box::new(Type::Str), None);
        let result = analyzer.unify(&arr1, &arr2);
        assert!(result.is_err());
    }

    #[test]
    fn test_generic_type_unification() {
        let mut analyzer = Analyzer::new();
        let gen1 = Type::Generic("T".to_string(), vec![Type::I64]);
        let gen2 = Type::Generic("T".to_string(), vec![Type::I64]);
        let result = analyzer.unify(&gen1, &gen2);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generic_type_mismatch() {
        let mut analyzer = Analyzer::new();
        let gen1 = Type::Generic("T".to_string(), vec![Type::I64]);
        let gen2 = Type::Generic("U".to_string(), vec![Type::I64]);
        let result = analyzer.unify(&gen1, &gen2);
        assert!(result.is_err());
    }

    #[test]
    fn test_nested_binary_operations() {
        let mut analyzer = Analyzer::new();
        let expr = Expression::Binary(
            Box::new(Expression::Binary(
                Box::new(Expression::Literal(Literal::Int(1))),
                BinaryOp::Add,
                Box::new(Expression::Literal(Literal::Int(2))),
            )),
            BinaryOp::Mul,
            Box::new(Expression::Literal(Literal::Int(3))),
        );
        let result = analyzer.analyze_expression(&expr);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap().ty, Type::I64));
    }

    #[test]
    fn test_mixed_numeric_types() {
        let mut analyzer = Analyzer::new();
        let expr = Expression::Binary(
            Box::new(Expression::Literal(Literal::Int(1))),
            BinaryOp::Add,
            Box::new(Expression::Literal(Literal::Float(2.5))),
        );
        let result = analyzer.analyze_expression(&expr);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap().ty, Type::F64));
    }
}
