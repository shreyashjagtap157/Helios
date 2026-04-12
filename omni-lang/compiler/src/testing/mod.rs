//! Omni test annotations and runner
//!
//! Phase 11 adds a lightweight test harness that can discover annotated
//! functions from the parsed AST and execute them through the existing
//! interpreter.

#![allow(dead_code)]

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::parser::ast::{self, Function, Item, Module};
use crate::runtime::interpreter::{Interpreter, RuntimeValue};

/// Test annotation metadata attached to a function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Test {
    pub name: Option<String>,
    pub should_panic: bool,
    pub ignore: bool,
    pub timeout_ms: Option<u64>,
}

impl Default for Test {
    fn default() -> Self {
        Self {
            name: None,
            should_panic: false,
            ignore: false,
            timeout_ms: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestShouldPanic {
    pub expected: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestIgnore {
    pub reason: String,
}

pub trait EffectMock: Send + Sync {
    fn mock(&self, effect: &str, args: &[RuntimeValue]) -> RuntimeValue;
}

pub struct EffectTest {
    pub mock_effects: Vec<(String, Box<dyn EffectMock>)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requires {
    pub condition: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ensures {
    pub condition: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invariant {
    pub condition: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestAnnotation {
    Test,
    TestShouldPanic(TestShouldPanic),
    TestIgnore(TestIgnore),
    EffectTest,
    Requires(Requires),
    Ensures(Ensures),
    Invariant(Invariant),
    Timeout(u64),
    Custom { name: String, value: Option<String> },
}

#[derive(Debug, Clone)]
pub struct TestCase {
    pub order: usize,
    pub module: Arc<Module>,
    pub function: Function,
    pub test: Test,
    pub annotations: Vec<TestAnnotation>,
    pub effect_row: Option<ast::EffectRow>,
    pub ignore_reason: Option<String>,
    pub expected_panic: Option<String>,
}

impl TestCase {
    pub fn display_name(&self) -> &str {
        self.test.name.as_deref().unwrap_or(&self.function.name)
    }
}

#[derive(Debug, Clone)]
pub struct TestConfig {
    pub parallel: bool,
    pub workers: usize,
    pub filter: Option<String>,
    pub show_output: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            parallel: false,
            workers: 1,
            filter: None,
            show_output: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TestFailure {
    pub name: String,
    pub reason: String,
    pub should_panic: bool,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub failures: Vec<TestFailure>,
    pub duration: Duration,
}

impl Default for TestResult {
    fn default() -> Self {
        Self {
            total: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            failures: Vec::new(),
            duration: Duration::default(),
        }
    }
}

impl TestResult {
    pub fn is_success(&self) -> bool {
        self.failed == 0
    }

    pub fn summary(&self) -> String {
        let status = if self.failed == 0 { "ok" } else { "FAILED" };
        format!(
            "test result: {}. {} passed; {} failed; {} skipped; {} total; elapsed {} ms",
            status,
            self.passed,
            self.failed,
            self.skipped,
            self.total,
            self.duration.as_millis()
        )
    }
}

pub struct TestRunner {
    config: TestConfig,
}

impl TestRunner {
    pub fn new(config: TestConfig) -> Self {
        Self { config }
    }

    pub fn run_module(&self, module: &Module) -> TestResult {
        let tests = discover_tests(module);
        self.run_tests(&tests)
    }

    pub fn run_tests(&self, tests: &[TestCase]) -> TestResult {
        let started = Instant::now();

        let mut cases: Vec<TestCase> = tests
            .iter()
            .cloned()
            .filter(|case| match &self.config.filter {
                Some(filter) => case.display_name().contains(filter),
                None => true,
            })
            .collect();
        cases.sort_by_key(|case| case.order);

        let mut collected: Vec<(usize, TestOutcome, Duration)> = if self.config.parallel
            && self.config.workers > 1
            && cases.len() > 1
        {
            self.collect_parallel(&cases)
        } else {
            self.collect_sequential(&cases)
        };

        collected.sort_by_key(|(order, _, _)| *order);

        let mut result = TestResult::default();
        result.total = collected.len();

        for (case, (_, outcome, duration)) in cases.iter().zip(collected.into_iter()) {
            let mut outcome = outcome;

            if !matches!(outcome, TestOutcome::Skipped(_)) {
                if let Some(timeout_ms) = case.test.timeout_ms {
                    if duration > Duration::from_millis(timeout_ms) {
                        outcome = TestOutcome::Failed(format!(
                            "test exceeded timeout of {} ms (elapsed {} ms)",
                            timeout_ms,
                            duration.as_millis()
                        ));
                    }
                }
            }

            if self.config.show_output {
                match &outcome {
                    TestOutcome::Passed => {
                        println!("test {} ... ok", case.display_name());
                    }
                    TestOutcome::Skipped(reason) => {
                        println!("test {} ... ignored ({})", case.display_name(), reason);
                    }
                    TestOutcome::Failed(reason) => {
                        println!("test {} ... FAILED: {}", case.display_name(), reason);
                    }
                }
            }

            match outcome {
                TestOutcome::Passed => {
                    result.passed += 1;
                }
                TestOutcome::Skipped(_) => {
                    result.skipped += 1;
                }
                TestOutcome::Failed(reason) => {
                    result.failed += 1;
                    result.failures.push(TestFailure {
                        name: case.display_name().to_string(),
                        reason,
                        should_panic: case.test.should_panic,
                    });
                }
            }
        }

        result.duration = started.elapsed();

        if self.config.show_output {
            println!("{}", result.summary());
        }

        result
    }

    fn collect_sequential(&self, cases: &[TestCase]) -> Vec<(usize, TestOutcome, Duration)> {
        cases
            .iter()
            .map(|case| {
                let (outcome, duration) = execute_test_case(case);
                (case.order, outcome, duration)
            })
            .collect()
    }

    fn collect_parallel(&self, cases: &[TestCase]) -> Vec<(usize, TestOutcome, Duration)> {
        let (tx, rx) = std::sync::mpsc::channel();

        for case in cases.iter().cloned() {
            let tx = tx.clone();
            std::thread::spawn(move || {
                let (outcome, duration) = execute_test_case(&case);
                let _ = tx.send((case.order, outcome, duration));
            });
        }

        drop(tx);

        let mut collected = Vec::with_capacity(cases.len());
        for _ in 0..cases.len() {
            match rx.recv() {
                Ok(item) => collected.push(item),
                Err(_) => break,
            }
        }
        collected
    }
}

pub fn discover_tests(module: &Module) -> Vec<TestCase> {
    let shared_module = Arc::new(module.clone());
    let mut tests = Vec::new();

    for item in &module.items {
        if let Item::Function(function) = item {
            let annotations = parse_annotations(&function.attributes);
            if !is_test_candidate(&annotations) {
                continue;
            }

            let mut test = Test {
                name: Some(function.name.clone()),
                ..Test::default()
            };
            let mut ignore_reason = None;
            let mut expected_panic = None;

            for annotation in &annotations {
                match annotation {
                    TestAnnotation::TestShouldPanic(details) => {
                        test.should_panic = true;
                        if expected_panic.is_none() {
                            expected_panic = details.expected.clone();
                        }
                    }
                    TestAnnotation::TestIgnore(details) => {
                        test.ignore = true;
                        if ignore_reason.is_none() {
                            ignore_reason = Some(details.reason.clone());
                        }
                    }
                    TestAnnotation::Timeout(limit) => {
                        test.timeout_ms = Some(*limit);
                    }
                    _ => {}
                }
            }

            tests.push(TestCase {
                order: tests.len(),
                module: Arc::clone(&shared_module),
                function: function.clone(),
                test,
                annotations,
                effect_row: function.effect_row.clone(),
                ignore_reason,
                expected_panic,
            });
        }
    }

    tests
}

pub fn run_tests(tests: &[TestCase], config: &TestConfig) -> TestResult {
    TestRunner::new(config.clone()).run_tests(tests)
}

pub fn run_module_tests(module: &Module, config: &TestConfig) -> TestResult {
    TestRunner::new(config.clone()).run_module(module)
}

enum TestOutcome {
    Passed,
    Failed(String),
    Skipped(String),
}

fn execute_test_case(case: &TestCase) -> (TestOutcome, Duration) {
    if case.test.ignore {
        return (
            TestOutcome::Skipped(
                case.ignore_reason
                    .clone()
                    .unwrap_or_else(|| "ignored".to_string()),
            ),
            Duration::default(),
        );
    }

    let started = Instant::now();
    let execution = catch_unwind(AssertUnwindSafe(|| {
        let mut interpreter = Interpreter::new();
        interpreter.load_module_ast(case.module.as_ref());
        interpreter.call_function(&case.function.name, &[])
    }));
    let duration = started.elapsed();

    let outcome = match execution {
        Ok(Ok(_)) => {
            if case.test.should_panic {
                TestOutcome::Failed("expected panic, but the test completed successfully".to_string())
            } else {
                TestOutcome::Passed
            }
        }
        Ok(Err(error)) => {
            if case.test.should_panic {
                if let Some(expected) = &case.expected_panic {
                    if error.to_string().contains(expected) {
                        TestOutcome::Passed
                    } else {
                        TestOutcome::Failed(format!(
                            "panic message '{}' did not match expected '{}'",
                            error, expected
                        ))
                    }
                } else {
                    TestOutcome::Passed
                }
            } else {
                TestOutcome::Failed(error.to_string())
            }
        }
        Err(_) => {
            if case.test.should_panic {
                TestOutcome::Passed
            } else {
                TestOutcome::Failed("test panicked unexpectedly".to_string())
            }
        }
    };

    (outcome, duration)
}

fn is_test_candidate(annotations: &[TestAnnotation]) -> bool {
    annotations.iter().any(|annotation| {
        matches!(annotation, TestAnnotation::Test | TestAnnotation::EffectTest)
    })
}

fn parse_annotations(attributes: &[String]) -> Vec<TestAnnotation> {
    let mut annotations = Vec::new();

    for attribute in attributes {
        let Some((name, args)) = parse_attribute(attribute) else {
            continue;
        };

        match name.as_str() {
            "cfg" => continue,
            "test" => {
                annotations.push(TestAnnotation::Test);
                annotations.extend(parse_test_args(args.as_deref()));
            }
            "test_should_panic" => {
                annotations.push(TestAnnotation::Test);
                annotations.push(TestAnnotation::TestShouldPanic(TestShouldPanic {
                    expected: parse_expected(args.as_deref()),
                }));
            }
            "test_ignore" => {
                annotations.push(TestAnnotation::Test);
                annotations.push(TestAnnotation::TestIgnore(TestIgnore {
                    reason: parse_reason(args.as_deref()).unwrap_or_else(|| "ignored".to_string()),
                }));
            }
            "effect_test" => annotations.push(TestAnnotation::EffectTest),
            "should_panic" => {
                annotations.push(TestAnnotation::TestShouldPanic(TestShouldPanic {
                    expected: parse_expected(args.as_deref()),
                }));
            }
            "ignore" => {
                annotations.push(TestAnnotation::TestIgnore(TestIgnore {
                    reason: parse_reason(args.as_deref()).unwrap_or_else(|| "ignored".to_string()),
                }));
            }
            "requires" => {
                if let Some(condition) = args.clone() {
                    annotations.push(TestAnnotation::Requires(Requires {
                        condition: normalize_value(&condition),
                    }));
                }
            }
            "ensures" => {
                if let Some(condition) = args.clone() {
                    annotations.push(TestAnnotation::Ensures(Ensures {
                        condition: normalize_value(&condition),
                    }));
                }
            }
            "invariant" => {
                if let Some(condition) = args.clone() {
                    annotations.push(TestAnnotation::Invariant(Invariant {
                        condition: normalize_value(&condition),
                    }));
                }
            }
            "timeout" => {
                if let Some(value) = parse_timeout(args.as_deref()) {
                    annotations.push(TestAnnotation::Timeout(value));
                }
            }
            other => annotations.push(TestAnnotation::Custom {
                name: other.to_string(),
                value: args,
            }),
        }
    }

    annotations
}

fn parse_test_args(args: Option<&str>) -> Vec<TestAnnotation> {
    let Some(args) = args else {
        return Vec::new();
    };

    let mut annotations = Vec::new();
    for token in args.split(',').map(|part| part.trim()).filter(|part| !part.is_empty()) {
        let lower = token.to_ascii_lowercase();
        if lower == "should_panic" {
            annotations.push(TestAnnotation::TestShouldPanic(TestShouldPanic {
                expected: None,
            }));
        } else if lower == "ignore" {
            annotations.push(TestAnnotation::TestIgnore(TestIgnore {
                reason: "ignored".to_string(),
            }));
        } else if lower.starts_with("ignore=") {
            annotations.push(TestAnnotation::TestIgnore(TestIgnore {
                reason: normalize_value(token.split_once('=').map(|(_, value)| value).unwrap_or("ignored")),
            }));
        } else if lower.starts_with("expected=") {
            annotations.push(TestAnnotation::TestShouldPanic(TestShouldPanic {
                expected: Some(normalize_value(
                    token.split_once('=').map(|(_, value)| value).unwrap_or(""),
                )),
            }));
        } else if lower.starts_with("timeout=") {
            if let Some(limit) = parse_timeout(Some(token.split_once('=').map(|(_, value)| value).unwrap_or(""))) {
                annotations.push(TestAnnotation::Timeout(limit));
            }
        } else if lower == "effect" {
            annotations.push(TestAnnotation::EffectTest);
        } else if lower.starts_with("requires=") {
            annotations.push(TestAnnotation::Requires(Requires {
                condition: normalize_value(token.split_once('=').map(|(_, value)| value).unwrap_or("")),
            }));
        } else if lower.starts_with("ensures=") {
            annotations.push(TestAnnotation::Ensures(Ensures {
                condition: normalize_value(token.split_once('=').map(|(_, value)| value).unwrap_or("")),
            }));
        } else if lower.starts_with("invariant=") {
            annotations.push(TestAnnotation::Invariant(Invariant {
                condition: normalize_value(token.split_once('=').map(|(_, value)| value).unwrap_or("")),
            }));
        }
    }

    annotations
}

fn parse_attribute(attribute: &str) -> Option<(String, Option<String>)> {
    let trimmed = attribute.trim();
    let inner = trimmed
        .strip_prefix("#[")
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(trimmed);

    if inner.is_empty() {
        return None;
    }

    let (name, args) = match inner.find('(') {
        Some(start) if inner.ends_with(')') => (
            inner[..start].trim().to_ascii_lowercase(),
            Some(inner[start + 1..inner.len() - 1].trim().to_string()),
        ),
        _ => (inner.trim().to_ascii_lowercase(), None),
    };

    Some((name, args))
}

fn parse_expected(args: Option<&str>) -> Option<String> {
    args.and_then(|value| {
        value
            .split(',')
            .map(|part| part.trim())
            .find_map(|part| {
                let lower = part.to_ascii_lowercase();
                if lower.starts_with("expected=") {
                    Some(normalize_value(
                        part.split_once('=').map(|(_, rhs)| rhs).unwrap_or(""),
                    ))
                } else if lower.starts_with("message=") {
                    Some(normalize_value(
                        part.split_once('=').map(|(_, rhs)| rhs).unwrap_or(""),
                    ))
                } else {
                    None
                }
            })
    })
}

fn parse_reason(args: Option<&str>) -> Option<String> {
    let value = args?.trim();
    let value = value
        .split_once('=')
        .map(|(_, rhs)| rhs.trim())
        .unwrap_or(value);
    let value = normalize_value(value);

    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn parse_timeout(args: Option<&str>) -> Option<u64> {
    let value = args?.trim();
    let value = value
        .split_once('=')
        .map(|(_, rhs)| rhs.trim())
        .unwrap_or(value);
    value.parse::<u64>().ok()
}

fn normalize_value(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_module() -> Module {
        Module {
            items: vec![
                Item::Function(Function {
                    name: "passes".to_string(),
                    is_async: false,
                    attributes: vec!["#[test]".to_string()],
                    params: Vec::new(),
                    return_type: Some(ast::Type::I64),
                    effect_row: None,
                    body: ast::Block {
                        statements: vec![ast::Statement::Return(Some(ast::Expression::Literal(
                            ast::Literal::Int(1),
                        )))],
                    },
                }),
                Item::Function(Function {
                    name: "should_panic_case".to_string(),
                    is_async: false,
                    attributes: vec!["#[test(should_panic)]".to_string()],
                    params: Vec::new(),
                    return_type: None,
                    effect_row: None,
                    body: ast::Block {
                        statements: vec![ast::Statement::Expression(ast::Expression::Call(
                            Box::new(ast::Expression::Identifier("missing_fn".to_string())),
                            Vec::new(),
                        ))],
                    },
                }),
                Item::Function(Function {
                    name: "ignored_case".to_string(),
                    is_async: false,
                    attributes: vec!["#[test(ignore=\"skip\")]".to_string()],
                    params: Vec::new(),
                    return_type: Some(ast::Type::I64),
                    effect_row: None,
                    body: ast::Block {
                        statements: vec![ast::Statement::Return(Some(ast::Expression::Literal(
                            ast::Literal::Int(2),
                        )))],
                    },
                }),
            ],
        }
    }

    fn combo_module() -> Module {
        Module {
            items: vec![
                Item::Function(Function {
                    name: "passes_case".to_string(),
                    is_async: false,
                    attributes: vec!["#[test]".to_string()],
                    params: Vec::new(),
                    return_type: Some(ast::Type::I64),
                    effect_row: None,
                    body: ast::Block {
                        statements: vec![ast::Statement::Return(Some(ast::Expression::Literal(
                            ast::Literal::Int(1),
                        )))],
                    },
                }),
                Item::Function(Function {
                    name: "expected_panic_case".to_string(),
                    is_async: false,
                    attributes: vec!["#[test_should_panic(expected=\"Undefined function: missing_fn\")]".to_string()],
                    params: Vec::new(),
                    return_type: None,
                    effect_row: None,
                    body: ast::Block {
                        statements: vec![ast::Statement::Expression(ast::Expression::Call(
                            Box::new(ast::Expression::Identifier("missing_fn".to_string())),
                            Vec::new(),
                        ))],
                    },
                }),
                Item::Function(Function {
                    name: "ignored_case".to_string(),
                    is_async: false,
                    attributes: vec!["#[test_ignore(reason=\"skip combo\")]".to_string()],
                    params: Vec::new(),
                    return_type: Some(ast::Type::I64),
                    effect_row: None,
                    body: ast::Block {
                        statements: vec![ast::Statement::Return(Some(ast::Expression::Literal(
                            ast::Literal::Int(2),
                        )))],
                    },
                }),
                Item::Function(Function {
                    name: "effect_case".to_string(),
                    is_async: false,
                    attributes: vec!["#[effect_test]".to_string()],
                    params: Vec::new(),
                    return_type: Some(ast::Type::I64),
                    effect_row: None,
                    body: ast::Block {
                        statements: vec![ast::Statement::Return(Some(ast::Expression::Literal(
                            ast::Literal::Int(3),
                        )))],
                    },
                }),
                Item::Function(Function {
                    name: "combined_case".to_string(),
                    is_async: false,
                    attributes: vec!["#[test(should_panic, expected=\"Undefined function: missing_fn\", ignore=\"skip combined\", timeout=1, requires=\"io\", ensures=\"done\", invariant=\"stable\", effect)]".to_string()],
                    params: Vec::new(),
                    return_type: None,
                    effect_row: None,
                    body: ast::Block {
                        statements: vec![ast::Statement::Expression(ast::Expression::Call(
                            Box::new(ast::Expression::Identifier("missing_fn".to_string())),
                            Vec::new(),
                        ))],
                    },
                }),
            ],
        }
    }

    #[test]
    fn discover_tests_finds_test_functions() {
        let module = sample_module();
        let tests = discover_tests(&module);

        assert_eq!(tests.len(), 3);
        assert_eq!(tests[0].display_name(), "passes");
        assert!(tests.iter().any(|case| case.function.name == "should_panic_case"));
        assert!(tests.iter().any(|case| case.test.ignore));
    }

    #[test]
    fn run_module_tests_executes_passing_and_ignored_cases() {
        let module = sample_module();
        let config = TestConfig {
            parallel: false,
            workers: 1,
            filter: None,
            show_output: false,
        };

        let result = run_module_tests(&module, &config);

        assert_eq!(result.total, 3);
        assert_eq!(result.passed, 2);
        assert_eq!(result.skipped, 1);
        assert_eq!(result.failed, 0);
        assert!(result.is_success());
    }

    #[test]
    fn run_module_tests_parallel_path_works() {
        let module = sample_module();
        let config = TestConfig {
            parallel: true,
            workers: 2,
            filter: None,
            show_output: false,
        };

        let result = run_module_tests(&module, &config);

        assert_eq!(result.total, 3);
        assert_eq!(result.passed, 2);
        assert_eq!(result.skipped, 1);
        assert_eq!(result.failed, 0);
    }

    #[test]
    fn parse_attribute_normalizes_bracketed_and_bare_forms() {
        assert_eq!(
            parse_attribute("#[TEST( expected = \"boom\" )]"),
            Some((
                "test".to_string(),
                Some("expected = \"boom\"".to_string())
            ))
        );
        assert_eq!(
            parse_attribute("timeout(5)"),
            Some(("timeout".to_string(), Some("5".to_string())))
        );
        assert_eq!(parse_attribute("#[ignore]"), Some(("ignore".to_string(), None)));
    }

    #[test]
    fn parse_test_args_handles_combined_flags() {
        let annotations = parse_test_args(Some(
            "should_panic, ignore=\"later\", expected=\"boom\", timeout=12, effect, requires=\"alpha\", ensures=\"beta\", invariant=\"gamma\"",
        ));

        assert_eq!(
            annotations,
            vec![
                TestAnnotation::TestShouldPanic(TestShouldPanic { expected: None }),
                TestAnnotation::TestIgnore(TestIgnore {
                    reason: "later".to_string(),
                }),
                TestAnnotation::TestShouldPanic(TestShouldPanic {
                    expected: Some("boom".to_string()),
                }),
                TestAnnotation::Timeout(12),
                TestAnnotation::EffectTest,
                TestAnnotation::Requires(Requires {
                    condition: "alpha".to_string(),
                }),
                TestAnnotation::Ensures(Ensures {
                    condition: "beta".to_string(),
                }),
                TestAnnotation::Invariant(Invariant {
                    condition: "gamma".to_string(),
                }),
            ]
        );
    }

    #[test]
    fn parse_reason_extracts_explicit_value() {
        assert_eq!(
            parse_reason(Some("reason=\"skip combo\"")),
            Some("skip combo".to_string())
        );
        assert_eq!(
            parse_reason(Some("reason='skip combo'")),
            Some("skip combo".to_string())
        );
        assert_eq!(parse_reason(Some("skip combo")), Some("skip combo".to_string()));
    }

    #[test]
    fn parse_annotations_collects_combined_metadata() {
        let annotations = parse_annotations(&vec![
            "#[test(should_panic, expected=\"Undefined function: missing_fn\", ignore=\"skip\", timeout=1, effect, requires=\"io\", ensures=\"done\", invariant=\"stable\")]".to_string(),
            "#[custom(flag)]".to_string(),
        ]);

        assert_eq!(
            annotations,
            vec![
                TestAnnotation::Test,
                TestAnnotation::TestShouldPanic(TestShouldPanic { expected: None }),
                TestAnnotation::TestShouldPanic(TestShouldPanic {
                    expected: Some("Undefined function: missing_fn".to_string()),
                }),
                TestAnnotation::TestIgnore(TestIgnore {
                    reason: "skip".to_string(),
                }),
                TestAnnotation::Timeout(1),
                TestAnnotation::EffectTest,
                TestAnnotation::Requires(Requires {
                    condition: "io".to_string(),
                }),
                TestAnnotation::Ensures(Ensures {
                    condition: "done".to_string(),
                }),
                TestAnnotation::Invariant(Invariant {
                    condition: "stable".to_string(),
                }),
                TestAnnotation::Custom {
                    name: "custom".to_string(),
                    value: Some("flag".to_string()),
                },
            ]
        );
    }

    #[test]
    fn discover_tests_tracks_order_and_combined_metadata() {
        let module = combo_module();
        let tests = discover_tests(&module);

        let names: Vec<_> = tests.iter().map(|case| case.display_name()).collect();
        assert_eq!(
            names,
            vec![
                "passes_case",
                "expected_panic_case",
                "ignored_case",
                "effect_case",
                "combined_case",
            ]
        );

        let expected = tests
            .iter()
            .find(|case| case.function.name == "expected_panic_case")
            .expect("expected panic case should be discovered");
        assert!(expected.test.should_panic);
        assert_eq!(
            expected.expected_panic.as_deref(),
            Some("Undefined function: missing_fn")
        );

        let ignored = tests
            .iter()
            .find(|case| case.function.name == "ignored_case")
            .expect("ignored case should be discovered");
        assert!(ignored.test.ignore);
        assert_eq!(ignored.ignore_reason.as_deref(), Some("skip combo"));

        let combined = tests
            .iter()
            .find(|case| case.function.name == "combined_case")
            .expect("combined case should be discovered");
        assert!(combined.test.should_panic);
        assert!(combined.test.ignore);
        assert_eq!(combined.test.timeout_ms, Some(1));
        assert_eq!(
            combined.expected_panic.as_deref(),
            Some("Undefined function: missing_fn")
        );
        assert_eq!(combined.ignore_reason.as_deref(), Some("skip combined"));
        assert!(combined
            .annotations
            .iter()
            .any(|annotation| matches!(annotation, TestAnnotation::EffectTest)));
        assert!(combined.annotations.iter().any(|annotation| matches!(
            annotation,
            TestAnnotation::Requires(Requires { condition }) if condition == "io"
        )));
        assert!(combined.annotations.iter().any(|annotation| matches!(
            annotation,
            TestAnnotation::Ensures(Ensures { condition }) if condition == "done"
        )));
        assert!(combined.annotations.iter().any(|annotation| matches!(
            annotation,
            TestAnnotation::Invariant(Invariant { condition }) if condition == "stable"
        )));
    }

    #[test]
    fn run_tests_applies_filter_to_discovered_cases() {
        let module = combo_module();
        let tests = discover_tests(&module);
        let config = TestConfig {
            parallel: false,
            workers: 1,
            filter: Some("effect_case".to_string()),
            show_output: false,
        };

        let result = run_tests(&tests, &config);

        assert_eq!(result.total, 1);
        assert_eq!(result.passed, 1);
        assert_eq!(result.skipped, 0);
        assert_eq!(result.failed, 0);
        assert!(result.is_success());
    }

    #[test]
    fn test_result_summary_reports_success_and_failure() {
        let success = TestResult {
            total: 2,
            passed: 2,
            failed: 0,
            skipped: 0,
            failures: Vec::new(),
            duration: Duration::from_millis(14),
        };
        assert!(success.summary().contains("ok"));
        assert!(success.summary().contains("2 passed"));
        assert!(success.summary().contains("0 failed"));
        assert!(success.summary().contains("14 ms"));

        let failure = TestResult {
            total: 3,
            passed: 1,
            failed: 1,
            skipped: 1,
            failures: vec![TestFailure {
                name: "failing_case".to_string(),
                reason: "boom".to_string(),
                should_panic: false,
            }],
            duration: Duration::from_millis(9),
        };
        assert!(!failure.is_success());
        assert!(failure.summary().contains("FAILED"));
        assert!(failure.summary().contains("1 passed"));
        assert!(failure.summary().contains("1 failed"));
        assert!(failure.summary().contains("1 skipped"));
        assert!(failure.summary().contains("9 ms"));
    }
}
