//! Error recovery and diagnostic enhancement
//!
//! Provides:
//! - Detailed error messages with suggestions
//! - Error recovery strategies
//! - Error context tracking
//! - Diagnostic hints

use std::collections::HashMap;
use std::fmt;

/// Detailed semantic error
#[derive(Debug, Clone)]
pub struct SemanticError {
    pub code: ErrorCode,
    pub message: String,
    pub context: ErrorContext,
    pub suggestions: Vec<String>,
    pub related_errors: Vec<Box<SemanticError>>,
}

/// Error codes for categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    // Type errors
    TypeMismatch,
    UnboundTypeVar,
    AmbiguousType,
    InvalidTypeApplication,
    
    // Lifetime errors
    LifetimeOutlivesMismatch,
    MissingLifetime,
    AmbiguousLifetime,
    LifetimeInference,
    
    // Trait errors
    TraitNotFound,
    TraitBoundNotSatisfied,
    IncoherentImpl,
    InvalidTraitObject,
    
    // Associated type errors
    AssocTypeNotFound,
    AssocTypeBindingMismatch,
    
    // Where clause errors
    WhereBoundNotSatisfied,
    InvalidWhereBound,
    
    // Const generic errors
    ConstGenericNotFound,
    ConstEvaluationError,
    ConstBindingMismatch,
    
    // Generic errors
    GenericAritryMismatch,
    InvalidGenericBound,
    
    // Other errors
    UnknownError,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TypeMismatch => write!(f, "E001: Type Mismatch"),
            Self::UnboundTypeVar => write!(f, "E002: Unbound Type Variable"),
            Self::LifetimeOutlivesMismatch => write!(f, "E101: Lifetime Outlives Mismatch"),
            Self::MissingLifetime => write!(f, "E102: Missing Lifetime"),
            Self::TraitNotFound => write!(f, "E201: Trait Not Found"),
            Self::TraitBoundNotSatisfied => write!(f, "E202: Trait Bound Not Satisfied"),
            Self::AssocTypeNotFound => write!(f, "E301: Associated Type Not Found"),
            Self::WhereBoundNotSatisfied => write!(f, "E401: Where Bound Not Satisfied"),
            Self::ConstGenericNotFound => write!(f, "E501: Const Generic Not Found"),
            Self::ConstEvaluationError => write!(f, "E502: Const Evaluation Error"),
            _ => write!(f, "E999: Unknown Error"),
        }
    }
}

/// Error context information
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub source_snippet: String,
    pub variable_types: HashMap<String, String>,
}

impl ErrorContext {
    pub fn new(file: String, line: usize, column: usize, snippet: String) -> Self {
        Self {
            file,
            line,
            column,
            source_snippet: snippet,
            variable_types: HashMap::new(),
        }
    }

    pub fn with_variable(mut self, name: String, ty: String) -> Self {
        self.variable_types.insert(name, ty);
        self
    }
}

impl SemanticError {
    pub fn new(code: ErrorCode, message: String, context: ErrorContext) -> Self {
        Self {
            code,
            message,
            context,
            suggestions: Vec::new(),
            related_errors: Vec::new(),
        }
    }

    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions.extend(suggestions);
        self
    }

    pub fn with_related(mut self, error: SemanticError) -> Self {
        self.related_errors.push(Box::new(error));
        self
    }

    pub fn format_detailed(&self) -> String {
        let mut output = format!(
            "{}\n  at {}:{}:{}\n\n{}",
            self.code,
            self.context.file,
            self.context.line,
            self.context.column,
            self.context.source_snippet
        );

        if !self.suggestions.is_empty() {
            output.push_str("\n\nSuggestions:\n");
            for (i, suggestion) in self.suggestions.iter().enumerate() {
                output.push_str(&format!("  {}. {}\n", i + 1, suggestion));
            }
        }

        if !self.context.variable_types.is_empty() {
            output.push_str("\n\nVariable Types:\n");
            for (var, ty) in &self.context.variable_types {
                output.push_str(&format!("  {} : {}\n", var, ty));
            }
        }

        output
    }
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

/// Error recovery strategies
pub struct ErrorRecovery {
    /// Recoverable error patterns
    recovery_patterns: HashMap<String, Vec<String>>,
}

impl ErrorRecovery {
    pub fn new() -> Self {
        let mut recovery_patterns = HashMap::new();

        // Type mismatch recovery
        recovery_patterns.insert(
            "type_mismatch".to_string(),
            vec![
                "Try explicit type casting".to_string(),
                "Check if types are compatible".to_string(),
                "Consider using a wrapper type".to_string(),
            ],
        );

        // Lifetime errors recovery
        recovery_patterns.insert(
            "lifetime_error".to_string(),
            vec![
                "Add explicit lifetime annotations".to_string(),
                "Check if borrowed values live long enough".to_string(),
                "Consider using `'static` lifetime".to_string(),
            ],
        );

        // Trait bound errors recovery
        recovery_patterns.insert(
            "trait_bound".to_string(),
            vec![
                "Ensure the type implements the required trait".to_string(),
                "Add trait implementation or derive".to_string(),
                "Check where clause constraints".to_string(),
            ],
        );

        // Associated type errors recovery
        recovery_patterns.insert(
            "assoc_type".to_string(),
            vec![
                "Specify the associated type explicitly".to_string(),
                "Check if the trait defines this associated type".to_string(),
                "Ensure type binding is correct".to_string(),
            ],
        );

        Self { recovery_patterns }
    }

    /// Get recovery suggestions for error pattern
    pub fn suggest_recovery(&self, pattern: &str) -> Vec<String> {
        self.recovery_patterns
            .get(pattern)
            .cloned()
            .unwrap_or_else(Vec::new)
    }

    /// Try to recover from error
    pub fn attempt_recovery(&self, error: &SemanticError) -> Option<String> {
        match error.code {
            ErrorCode::TypeMismatch => Some(
                "Consider checking type compatibility".to_string(),
            ),
            ErrorCode::UnboundTypeVar => Some(
                "Add explicit type annotation".to_string(),
            ),
            ErrorCode::TraitBoundNotSatisfied => Some(
                "Implement the required trait for this type".to_string(),
            ),
            ErrorCode::LifetimeOutlivesMismatch => Some(
                "Review lifetime constraints in your code".to_string(),
            ),
            _ => None,
        }
    }
}

/// Diagnostic information collector
pub struct DiagnosticCollector {
    errors: Vec<SemanticError>,
    warnings: Vec<String>,
    notes: Vec<String>,
}

impl DiagnosticCollector {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            notes: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: SemanticError) {
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn add_note(&mut self, note: String) {
        self.notes.push(note);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn get_errors(&self) -> &[SemanticError] {
        &self.errors
    }

    pub fn format_report(&self) -> String {
        let mut report = String::new();

        if !self.errors.is_empty() {
            report.push_str(&format!("=== {} Errors ===\n\n", self.errors.len()));
            for error in &self.errors {
                report.push_str(&error.format_detailed());
                report.push('\n');
            }
        }

        if !self.warnings.is_empty() {
            report.push_str(&format!("=== {} Warnings ===\n\n", self.warnings.len()));
            for warning in &self.warnings {
                report.push_str(&format!("  - {}\n", warning));
            }
        }

        if !self.notes.is_empty() {
            report.push_str(&format!("=== {} Notes ===\n\n", self.notes.len()));
            for note in &self.notes {
                report.push_str(&format!("  - {}\n", note));
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let ctx = ErrorContext::new(
            "test.omni".to_string(),
            10,
            5,
            "let x: i32 = \"hello\";".to_string(),
        );
        let error = SemanticError::new(
            ErrorCode::TypeMismatch,
            "Expected i32, found String".to_string(),
            ctx,
        );
        assert_eq!(error.code, ErrorCode::TypeMismatch);
    }

    #[test]
    fn test_error_suggestions() {
        let recovery = ErrorRecovery::new();
        let suggestions = recovery.suggest_recovery("type_mismatch");
        assert!(!suggestions.is_empty());
    }

    #[test]
    fn test_diagnostic_collector() {
        let mut collector = DiagnosticCollector::new();
        let ctx = ErrorContext::new("test.omni".to_string(), 1, 1, "code".to_string());
        let error = SemanticError::new(
            ErrorCode::TypeMismatch,
            "Test error".to_string(),
            ctx,
        );
        collector.add_error(error);
        assert!(collector.has_errors());
        assert_eq!(collector.error_count(), 1);
    }

    #[test]
    fn test_error_code_display() {
        assert_eq!(ErrorCode::TypeMismatch.to_string(), "E001: Type Mismatch");
        assert_eq!(ErrorCode::TraitNotFound.to_string(), "E201: Trait Not Found");
    }
}
