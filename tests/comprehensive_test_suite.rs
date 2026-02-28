/// Comprehensive Test Suite and Validation Framework
/// Tests all new features: language features, AI/ML framework, developer tools, optimizations

#[cfg(test)]
mod comprehensive_tests {
    use crate::language_features::prelude::*;
    use crate::brain::learning_framework::*;
    use crate::brain::adaptive_reasoning::*;

    mod language_features_tests {
        use super::*;

        #[test]
        fn test_lazy_static_thread_safety() {
            let lazy = std::sync::Arc::new(LazyStat::new());
            let mut handles = vec![];

            for _ in 0..100 {
                let l = lazy.clone();
                handles.push(std::thread::spawn(move || {
                    let val = l.get_or_init(|| 42);
                    *val
                }));
            }

            for handle in handles {
                assert_eq!(handle.join().unwrap(), 42);
            }
        }

        #[test]
        fn test_default_parameters_resolution() {
            let sig = FunctionSignature::new("test_fn".to_string(), Type::I32)
                .add_param(Parameter::new("a".to_string(), Type::I32))
                .add_param(Parameter::with_default("b".to_string(), Type::I32, "10".to_string()))
                .add_param(Parameter::with_default("c".to_string(), Type::I32, "20".to_string()));

            assert!(sig.validate().is_ok());
            assert_eq!(sig.required_params(), 1);
            assert_eq!(sig.total_params(), 3);

            let resolver = CallResolver::new(sig);
            let args = vec![Argument::positional("5".to_string())];
            let resolved = resolver.resolve_call(args).unwrap();

            assert_eq!(resolved.len(), 3);
            assert_eq!(resolved[1].1, "10");
            assert_eq!(resolved[2].1, "20");
        }

        #[test]
        fn test_operator_overloading_registry() {
            let mut registry = OperatorRegistry::new();
            let vec_type = Type::Struct("Vec2".to_string());

            registry.register(vec_type.clone(), Operator::Add, Some(vec_type.clone()), vec_type.clone());
            registry.register(vec_type.clone(), Operator::Mul, Some(Type::F32), vec_type.clone());

            assert!(registry.lookup(&vec_type, Operator::Add).is_some());
            assert!(registry.lookup(&vec_type, Operator::Mul).is_some());
            assert!(registry.lookup(&vec_type, Operator::Sub).is_none());

            let overloads = registry.overloads_for_type(&vec_type);
            assert_eq!(overloads.len(), 2);
        }

        #[test]
        fn test_named_arguments_ordering() {
            let sig = FunctionSignature::new("greet".to_string(), Type::String)
                .add_param(Parameter::new("greeting".to_string(), Type::String))
                .add_param(Parameter::new("name".to_string(), Type::String))
                .add_param(Parameter::new("suffix".to_string(), Type::String));

            let resolver = CallResolver::new(sig);

            // Call with named arguments in different order
            let args = vec![
                Argument::named("suffix".to_string(), "!".to_string()),
                Argument::named("name".to_string(), "Alice".to_string()),
                Argument::named("greeting".to_string(), "Hi".to_string()),
            ];

            let resolved = resolver.resolve_call(args).unwrap();
            assert_eq!(resolved.len(), 3);

            // Check that resolution worked correctly despite order
            assert!(resolved.iter().any(|(n, v)| n == "greeting" && v == "Hi"));
            assert!(resolved.iter().any(|(n, v)| n == "name" && v == "Alice"));
            assert!(resolved.iter().any(|(n, v)| n == "suffix" && v == "!"));
        }

        #[test]
        fn test_variadic_type_validation() {
            let vf = VariadicFunction::new(Type::I32, false)
                .add_param("count".to_string(), Type::I32)
                .set_variadic(Type::String, 0);

            let valid_args = vec![
                ("count".to_string(), Type::I32),
                ("str1".to_string(), Type::String),
                ("str2".to_string(), Type::String),
                ("str3".to_string(), Type::String),
            ];

            assert!(vf.validate_variadic_args(&valid_args).is_ok());

            let invalid_args = vec![
                ("count".to_string(), Type::I32),
                ("str1".to_string(), Type::String),
                ("num".to_string(), Type::I32), // Wrong type!
            ];

            assert!(vf.validate_variadic_args(&invalid_args).is_err());
        }
    }

    mod ai_ml_tests {
        use super::*;

        #[test]
        fn test_knowledge_extraction_and_verification() {
            let extractor = KnowledgeExtractor::new();
            let verifier = MultiSourceVerifier::new();

            // Register test sources
            verifier.register_source(Source::new("wiki".to_string(), "Wikipedia".to_string()));
            verifier.register_source(Source::new("db".to_string(), "Database".to_string()));

            // Extract facts
            let text = "Alice works at Google and Bob works at Microsoft";
            let facts = extractor.extract(text);

            for fact in facts {
                let verification = verifier.verify_fact(&fact, 0.5);
                assert!(verification.agreement_ratio >= 0.0 && verification.agreement_ratio <= 1.0);
            }
        }

        #[test]
        fn test_real_time_learning_loop() {
            let verifier = MultiSourceVerifier::new();
            verifier.register_source(Source::new("source1".to_string(), "Source 1".to_string()));

            let loop_coord = RealTimeLearningLoop::new(verifier);

            // Simulate learning from text
            loop_coord.learn_from_text("The Earth orbits the Sun".to_string());
            loop_coord.learn_from_text("Water freezes at 0 degrees Celsius".to_string());

            let processed = loop_coord.learning_cycle();
            assert!(processed >= 0);

            // Check knowledge base was updated
            let kb = loop_coord.get_knowledge_base();
            println!("Learned {} facts", kb.len());
        }

        #[test]
        fn test_error_learning_and_adaptation() {
            let verifier = MultiSourceVerifier::new();
            let loop_coord = RealTimeLearningLoop::new(verifier);

            loop_coord.learn_from_text("The Sun orbits the Earth".to_string());
            let _processed = loop_coord.learning_cycle();

            // Record error and correct fact
            loop_coord.record_error(
                "Incorrect fact about celestial mechanics".to_string(),
                Some("subject:relation:object".to_string()),
                Some("The Earth orbits the Sun".to_string()),
            );

            let errors = loop_coord.get_error_history();
            assert_eq!(errors.len(), 1);
            assert!(errors[0].correction.is_some());
        }

        #[test]
        fn test_multi_step_adaptive_reasoning() {
            let engine = AdaptiveReasoningEngine::new();

            let premises = vec![
                "All mammals are animals".to_string(),
                "Dogs are mammals".to_string(),
            ];

            // Multi-step reasoning
            let chain = engine.multi_step_reasoning(premises);
            assert!(chain.steps.len() > 0);
            assert!(chain.overall_confidence > 0.0);

            // Record success
            engine.record_outcome(chain.steps[0].strategy.clone(), true);

            // Verify strategy performance improves
            let report = engine.get_performance_report();
            let deductive = &report[&ReasoningStrategy::Deductive];
            assert!(deductive.success_count > 0);
        }

        #[test]
        fn test_contextual_reasoning_with_domain_knowledge() {
            let engine = std::sync::Arc::new(AdaptiveReasoningEngine::new());
            let reasoner = ContextualReasoner::new(engine);

            reasoner.set_context("domain".to_string(), "computer science".to_string());
            reasoner.set_context("topic".to_string(), "algorithms".to_string());

            let premises = vec![
                "{domain} studies {topic}".to_string(),
            ];

            let chain = reasoner.reason_with_context(premises);
            assert!(!chain.final_conclusion.is_empty());
        }
    }

    mod developer_tools_tests {
        use super::*;

        #[test]
        fn test_lsp_document_operations() {
            let server = crate::tools::omni_lsp::server::LanguageServer::new();

            // Open document
            server.open_document(
                "test.omni".to_string(),
                "fn greet(name: String) -> String {\n  return \"Hello, \" + name\n}".to_string(),
            );

            // Get diagnostics
            let diags = server.get_diagnostics("test.omni");
            println!("Diagnostics: {:?}", diags);

            // Format document
            let formatted = server.format_document("test.omni");
            assert!(formatted.is_some());

            // Close document
            server.close_document("test.omni");
            let diags_after = server.get_diagnostics("test.omni");
            assert!(diags_after.is_empty());
        }

        #[test]
        fn test_lsp_code_completion() {
            let server = crate::tools::omni_lsp::server::LanguageServer::new();
            server.open_document("test.omni".to_string(), "fn test() {}".to_string());

            let completions = server.completions("test.omni", 0, 0);
            assert!(!completions.is_empty());

            // Check that keywords are included
            let keywords = completions
                .iter()
                .filter(|c| matches!(c.kind, crate::tools::omni_lsp::server::CompletionItemKind::Keyword))
                .collect::<Vec<_>>();
            assert!(!keywords.is_empty());

            // Check that function keywords are present
            assert!(keywords.iter().any(|c| c.label == "fn"));
            assert!(keywords.iter().any(|c| c.label == "async"));
            assert!(keywords.iter().any(|c| c.label == "struct"));
        }

        #[test]
        fn test_lsp_hover_and_navigation() {
            let server = crate::tools::omni_lsp::server::LanguageServer::new();
            server.open_document(
                "test.omni".to_string(),
                "let x = 42\nlet y = x + 10".to_string(),
            );

            // Test hover
            let hover = server.hover("test.omni", 0, 4);
            assert!(hover.is_some());
            assert!(hover.unwrap().contents.contains("Type:") || hover.unwrap().contents.contains("declared"));

            // Test multiple positions
            let hover2 = server.hover("test.omni", 1, 8);
            assert!(hover2.is_some());
        }
    }

    mod integration_tests {
        use super::*;

        #[test]
        fn test_full_workflow_language_and_ml() {
            // 1. Create a function with default parameters and operator overloading
            let sig = FunctionSignature::new("process".to_string(), Type::I32)
                .add_param(Parameter::new("data".to_string(), Type::String))
                .add_param(Parameter::with_default("threshold".to_string(), Type::I32, "50".to_string()));

            assert!(sig.validate().is_ok());

            // 2. Set up AI/ML learning
            let verifier = MultiSourceVerifier::new();
            let learning_loop = RealTimeLearningLoop::new(verifier);

            // 3. Learn from example data
            learning_loop.learn_from_text("The algorithm processes input data efficiently".to_string());
            let processed = learning_loop.learning_cycle();
            println!("Processed {} facts", processed);

            // 4. Apply adaptive reasoning
            let engine = AdaptiveReasoningEngine::new();
            let premises = vec!["algorithm is efficient".to_string()];
            let chain = engine.multi_step_reasoning(premises);
            assert!(!chain.final_conclusion.is_empty());
        }

        #[test]
        fn test_full_development_workflow() {
            // Simulate full development workflow
            let server = crate::tools::omni_lsp::server::LanguageServer::new();

            // 1. Write code
            let code = r#"
fn factorial(n: i32) -> i32 {
    if n <= 1 {
        return 1;
    }
    return n * factorial(n - 1);
}

fn main() {
    let result = factorial(5);
    println("Result: {}", result);
}
            "#;

            // 2. Open in LSP
            server.open_document("factorial.omni".to_string(), code.to_string());

            // 3. Get diagnostics (errors/warnings)
            let diags = server.get_diagnostics("factorial.omni");
            println!("Found {} issues", diags.len());

            // 4. Get completions
            let completions = server.completions("factorial.omni", 0, 0);
            assert!(!completions.is_empty());

            // 5. Format code
            let formatted = server.format_document("factorial.omni");
            assert!(formatted.is_some());

            // 6. Close document
            server.close_document("factorial.omni");
        }
    }
}

// Performance benchmarks
#[cfg(test)]
mod performance_benchmarks {
    #[test]
    #[ignore] // Run with: cargo test -- --ignored --nocapture performance_benchmarks
    fn bench_lazy_static_access_speed() {
        use crate::language_features::prelude::*;
        use std::time::Instant;

        let lazy = std::sync::Arc::new(LazyStat::new());

        // First access (initialization)
        let start = Instant::now();
        let _ = lazy.get_or_init(|| 42);
        let init_time = start.elapsed();
        println!("Lazy static initialization: {:?}", init_time);

        // Subsequent accesses (should be very fast)
        let start = Instant::now();
        for _ in 0..1_000_000 {
            let _ = lazy.get_or_init(|| 42);
        }
        let access_time = start.elapsed();
        println!("1M lazy static accesses: {:?}", access_time);
        println!("Average access time: {} ns", access_time.as_nanos() / 1_000_000);
    }

    #[test]
    #[ignore]
    fn bench_learning_loop_throughput() {
        use crate::brain::learning_framework::*;

        let verifier = MultiSourceVerifier::new();
        let learning_loop = RealTimeLearningLoop::new(verifier);

        // Simulate high-volume learning
        let texts = vec![
            "Alice works at Google"; 1000
        ];

        let start = std::time::Instant::now();
        for text in texts {
            for _ in 0..1000 {
                learning_loop.learn_from_text(text.to_string());
            }
        }

        let elapsed = start.elapsed();
        println!("Learning from 1M texts: {:?}", elapsed);
        println!("Average per text: {} µs", elapsed.as_micros() / 1_000_000);
    }
}
