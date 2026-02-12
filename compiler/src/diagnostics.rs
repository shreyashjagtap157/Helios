//! Comprehensive Error Handling and Quality Standards
//! Implements error codes, warning levels, diagnostics
//! Date: Feb 11, 2026, 15:35 UTC

use std::fmt;

/// Severity levels for compiler messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticLevel {
    /// Informational messages
    Info = 0,
    /// Warning - compilation may succeed but code may have issues
    Warning = 1,
    /// Error - compilation will fail
    Error = 2,
    /// Internal compiler error - should never happen
    Internal = 3,
}

/// Error codes for compiler diagnostics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    // Lexical errors (1000-1099)
    InvalidToken = 1001,
    InvalidStringLiteral = 1002,
    InvalidNumberLiteral = 1003,
    UnclosedComment = 1004,
    InvalidEscape = 1005,

    // Syntax errors (1100-1199)
    UnexpectedToken = 1101,
    MissingToken = 1102,
    InvalidPattern = 1103,
    MalformedExpression = 1104,
    MissingClosingBracket = 1105,

    // Semantic errors (1200-1299)
    UndefinedVariable = 1201,
    UndefinedFunction = 1202,
    TypeMismatch = 1203,
    BorrowError = 1204,
    LifetimeError = 1205,
    TraitNotFound = 1206,
    MethodNotFound = 1207,
    NonExhaustiveMatch = 1208,
    InvalidFieldAccess = 1209,

    // Type system errors (1300-1399)
    InvalidTypecast = 1301,
    GenericConstraintViolation = 1302,
    IncompatibleTypes = 1303,
    InvalidGenericArg = 1304,
    CircularTypeDefinition = 1305,

    // Runtime errors (1400-1499)
    StackOverflow = 1401,
    OutOfMemory = 1402,
    DivisionByZero = 1403,
    IndexOutOfBounds = 1404,
    NullPointerDereference = 1405,
    UseAfterFree = 1406,
    DoubleFreed = 1407,

    // Warnings (2000-2099)
    UnusedVariable = 2001,
    UnusedFunction = 2002,
    UnusedImport = 2003,
    DeprecatedFeature = 2004,
    PerformanceWarning = 2005,
    PotentialPanicWarning = 2006,

    // Other errors
    InternalCompilerError = 9000,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::InvalidToken => "E1001: Invalid token",
            ErrorCode::InvalidStringLiteral => "E1002: Invalid string literal",
            ErrorCode::InvalidNumberLiteral => "E1003: Invalid number literal",
            ErrorCode::UnclosedComment => "E1004: Unclosed comment",
            ErrorCode::InvalidEscape => "E1005: Invalid escape sequence",
            
            ErrorCode::UnexpectedToken => "E1101: Unexpected token",
            ErrorCode::MissingToken => "E1102: Missing required token",
            ErrorCode::InvalidPattern => "E1103: Invalid pattern",
            ErrorCode::MalformedExpression => "E1104: Malformed expression",
            ErrorCode::MissingClosingBracket => "E1105: Missing closing bracket",
            
            ErrorCode::UndefinedVariable => "E1201: Undefined variable",
            ErrorCode::UndefinedFunction => "E1202: Undefined function",
            ErrorCode::TypeMismatch => "E1203: Type mismatch",
            ErrorCode::BorrowError => "E1204: Borrow checker violation",
            ErrorCode::LifetimeError => "E1205: Lifetime error",
            ErrorCode::TraitNotFound => "E1206: Trait not found",
            ErrorCode::MethodNotFound => "E1207: Method not found",
            ErrorCode::NonExhaustiveMatch => "E1208: Non-exhaustive match",
            ErrorCode::InvalidFieldAccess => "E1209: Invalid field access",
            
            ErrorCode::InvalidTypecast => "E1301: Invalid type cast",
            ErrorCode::GenericConstraintViolation => "E1302: Generic constraint violation",
            ErrorCode::IncompatibleTypes => "E1303: Incompatible types",
            ErrorCode::InvalidGenericArg => "E1304: Invalid generic argument",
            ErrorCode::CircularTypeDefinition => "E1305: Circular type definition",
            
            ErrorCode::StackOverflow => "E1401: Stack overflow",
            ErrorCode::OutOfMemory => "E1402: Out of memory",
            ErrorCode::DivisionByZero => "E1403: Division by zero",
            ErrorCode::IndexOutOfBounds => "E1404: Index out of bounds",
            ErrorCode::NullPointerDereference => "E1405: Null pointer dereference",
            ErrorCode::UseAfterFree => "E1406: Use after free",
            ErrorCode::DoubleFreed => "E1407: Double freed",
            
            ErrorCode::UnusedVariable => "W2001: Unused variable",
            ErrorCode::UnusedFunction => "W2002: Unused function",
            ErrorCode::UnusedImport => "W2003: Unused import",
            ErrorCode::DeprecatedFeature => "W2004: Deprecated feature",
            ErrorCode::PerformanceWarning => "W2005: Performance warning",
            ErrorCode::PotentialPanicWarning => "W2006: Potential panic",
            
            ErrorCode::InternalCompilerError => "ICE9000: Internal compiler error",
        }
    }

    pub fn as_code_number(&self) -> u32 {
        *self as u32
    }
}

/// Compiler diagnostic with location and message
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub code: ErrorCode,
    pub message: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub hint: Option<String>,
}

impl Diagnostic {
    pub fn new_error(code: ErrorCode, message: String, file: String, line: usize, column: usize) -> Self {
        Diagnostic {
            level: DiagnosticLevel::Error,
            code,
            message,
            file,
            line,
            column,
            hint: None,
        }
    }

    pub fn new_warning(code: ErrorCode, message: String, file: String, line: usize, column: usize) -> Self {
        Diagnostic {
            level: DiagnosticLevel::Warning,
            code,
            message,
            file,
            line,
            column,
            hint: None,
        }
    }

    pub fn with_hint(mut self, hint: String) -> Self {
        self.hint = Some(hint);
        self
    }

    pub fn format(&self) -> String {
        let level_str = match self.level {
            DiagnosticLevel::Info => "info",
            DiagnosticLevel::Warning => "warning",
            DiagnosticLevel::Error => "error",
            DiagnosticLevel::Internal => "internal error",
        };

        let mut output = format!(
            "{}:{}:{}: {}: {}",
            self.file, self.line, self.column, level_str, self.message
        );

        if let Some(hint) = &self.hint {
            output.push('\n');
            output.push_str("  hint: ");
            output.push_str(hint);
        }

        output
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

/// Quality standards and tolerance levels
#[derive(Debug, Clone)]
pub struct QualityStandards {
    /// Maximum allowed warnings per compilation
    pub max_warnings: usize,
    /// Maximum allowed line length
    pub max_line_length: usize,
    /// Required code coverage percentage (0-100)
    pub min_coverage_percent: usize,
    /// Maximum cyclomatic complexity per function
    pub max_cyclomatic_complexity: usize,
    /// Maximum function length (lines of code)
    pub max_function_lines: usize,
    /// Whether to treat warnings as errors
    pub warnings_as_errors: bool,
    /// Whether to enforce strict null safety
    pub strict_null_safety: bool,
    /// Whether to enforce strict bounds checking
    pub strict_bounds_checking: bool,
}

impl Default for QualityStandards {
    fn default() -> Self {
        QualityStandards {
            max_warnings: 10,
            max_line_length: 120,
            min_coverage_percent: 80,
            max_cyclomatic_complexity: 10,
            max_function_lines: 200,
            warnings_as_errors: false,
            strict_null_safety: true,
            strict_bounds_checking: true,
        }
    }
}

impl QualityStandards {
    pub fn strict() -> Self {
        QualityStandards {
            max_warnings: 0,
            max_line_length: 100,
            min_coverage_percent: 95,
            max_cyclomatic_complexity: 7,
            max_function_lines: 100,
            warnings_as_errors: true,
            strict_null_safety: true,
            strict_bounds_checking: true,
        }
    }

    pub fn relaxed() -> Self {
        QualityStandards {
            max_warnings: 50,
            max_line_length: 150,
            min_coverage_percent: 50,
            max_cyclomatic_complexity: 15,
            max_function_lines: 500,
            warnings_as_errors: false,
            strict_null_safety: false,
            strict_bounds_checking: false,
        }
    }
}

/// Diagnostic collector for compilation
pub struct DiagnosticCollector {
    diagnostics: Vec<Diagnostic>,
    standards: QualityStandards,
}

impl DiagnosticCollector {
    pub fn new(standards: QualityStandards) -> Self {
        DiagnosticCollector {
            diagnostics: Vec::new(),
            standards,
        }
    }

    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn errors(&self) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.level == DiagnosticLevel::Error)
            .collect()
    }

    pub fn warnings(&self) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.level == DiagnosticLevel::Warning)
            .collect()
    }

    pub fn has_errors(&self) -> bool {
        !self.errors().is_empty()
    }

    pub fn check_standards(&self) -> Result<(), String> {
        let warnings = self.warnings().len();

        if warnings > self.standards.max_warnings {
            return Err(format!(
                "Too many warnings: {} > max {}",
                warnings, self.standards.max_warnings
            ));
        }

        if self.standards.warnings_as_errors && !self.warnings().is_empty() {
            return Err("Warnings treated as errors".to_string());
        }

        Ok(())
    }

    pub fn report(&self) {
        for diagnostic in &self.diagnostics {
            println!("{}", diagnostic.format());
        }

        let error_count = self.errors().len();
        let warning_count = self.warnings().len();

        if error_count > 0 || warning_count > 0 {
            println!(
                "\nCompilation: {} error(s), {} warning(s)",
                error_count, warning_count
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_string() {
        assert!(ErrorCode::InvalidToken.as_str().contains("1001"));
        assert!(ErrorCode::UnusedVariable.as_str().contains("2001"));
    }

    #[test]
    fn test_diagnostic_creation() {
        let diag = Diagnostic::new_error(
            ErrorCode::TypeMismatch,
            "Expected i32, got f64".to_string(),
            "test.omni".to_string(),
            42,
            10,
        ).with_hint("Use 'as i32' to cast".to_string());

        assert_eq!(diag.level, DiagnosticLevel::Error);
        assert!(diag.hint.is_some());
    }

    #[test]
    fn test_quality_standards() {
        let strict = QualityStandards::strict();
        let relaxed = QualityStandards::relaxed();

        assert!(strict.max_warnings < relaxed.max_warnings);
        assert!(strict.min_coverage_percent > relaxed.min_coverage_percent);
    }
}
