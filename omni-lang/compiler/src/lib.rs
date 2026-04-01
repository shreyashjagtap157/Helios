// Copyright 2024 Shreyash Jagtap
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
#[path = "codegen/mod.rs"]
pub mod codegen;
pub mod diagnostics;
pub mod enhancements;
#[path = "ir/mod.rs"]
pub mod ir;
#[path = "lexer/mod.rs"]
pub mod lexer;
pub mod modes;
pub mod monitor;
pub mod optimizer;
#[path = "parser/mod.rs"]
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
