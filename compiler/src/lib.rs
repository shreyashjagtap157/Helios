//! Omni Compiler Library
//!
//! Provides the core compiler pipeline: lexer, parser, semantic analysis,
//! IR generation, code generation, and runtime.

pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod ir;
pub mod runtime;
pub mod codegen;
pub mod diagnostics;
pub mod enhancements;

// Re-export key types
pub use diagnostics::{ErrorCode, Diagnostic, DiagnosticLevel, QualityStandards, DiagnosticCollector};
pub use enhancements::{SIMDInfo, MemoryPool, VectorizationAnalyzer, SecurityHardening, PerformanceMetrics};

