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

pub mod brain;
pub mod codegen;
pub mod diagnostics;
pub mod enhancements;
pub mod ir;
pub mod lexer;
pub mod modes;
pub mod monitor;
pub mod optimizer;
pub mod parser;
pub mod resolver;
pub mod runtime;
pub mod semantic;

// Re-export key types
pub use diagnostics::{
    Diagnostic, DiagnosticCollector, DiagnosticLevel, ErrorCode, QualityStandards,
};
pub use enhancements::{
    MemoryPool, PerformanceMetrics, SIMDInfo, SecurityHardening, VectorizationAnalyzer,
};
pub use modes::{
    allowed_zones, is_memory_op_allowed, Feature, MemoryOperation, MemoryZone, MemoryZoneChecker,
    ModuleChecker, ModuleMode, PackageManifest,
};
pub use resolver::{
    resolve_all, Annotations, ConcurrencyStrategy, ExecMode, MemoryStrategy, ResolverContext,
    ResolverDecision, ResolverLog,
};
