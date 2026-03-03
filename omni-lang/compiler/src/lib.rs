//! Omni Compiler Library
//!
//! Provides the core compiler pipeline: lexer, parser, semantic analysis,
//! IR generation, code generation, and runtime.

#![allow(
    unused_imports,
    unused_variables,
    unused_mut,
    unused_parens,
    dead_code,
    non_snake_case,
    mismatched_lifetime_syntaxes
)]

pub mod codegen;
pub mod diagnostics;
pub mod enhancements;
pub mod ir;
pub mod lexer;
pub mod optimizer;
pub mod parser;
pub mod runtime;
pub mod semantic;

// Re-export key types
pub use diagnostics::{
    Diagnostic, DiagnosticCollector, DiagnosticLevel, ErrorCode, QualityStandards,
};
pub use enhancements::{
    MemoryPool, PerformanceMetrics, SIMDInfo, SecurityHardening, VectorizationAnalyzer,
};
