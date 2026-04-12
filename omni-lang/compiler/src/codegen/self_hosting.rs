//! Self-Hosting Compiler Architecture for Omni
//!
#![allow(dead_code)]

//! This module establishes the framework and design patterns for implementing
//! the Omni compiler IN Omni itself, enabling true self-hosting.
//!
//! Current Status: Reference architecture for future Omni-based compiler
//! Bootstrap Path:
//! 1. Rust omnc compiles this module + minimal Omni stdlib
//! 2. Omni compiler (written in Omni) reaches feature parity with Rust omnc
//! 3. Omni compiler self-compiles (omnc_omni compiles itself)
//! 4. Eventually deprecate Rust omnc in favor of pure Omni implementation

use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ─── Bootstrap Runner  ──────────────────────────────────────────────────

/// Runs the self-hosting bootstrap pipeline:
/// Stage 0 (Rust omnc) → compiles main.omni → Stage 1 binary
/// Stage 1 binary → compiles main.omni → Stage 2 binary
/// Verify: Stage 1 == Stage 2 (fixpoint)
pub struct BootstrapRunner {
    /// Path to the Rust-compiled omnc binary (Stage 0)
    pub omnc_stage0: PathBuf,
    /// Path to the Omni compiler source root (contains main.omni)
    pub source_root: PathBuf,
    /// Working directory for bootstrap artifacts
    pub work_dir: PathBuf,
}

impl BootstrapRunner {
    /// Run Stage 0 → Stage 1: compile the Omni compiler source with Rust omnc.
    pub fn run_stage1(&self) -> Result<PathBuf, String> {
        std::fs::create_dir_all(&self.work_dir)
            .map_err(|e| format!("Cannot create work dir: {}", e))?;

        let main_omni = self.source_root.join("main.omni");
        if !main_omni.exists() {
            return Err(format!("Source file not found: {}", main_omni.display()));
        }

        let stage1_output = self.work_dir.join("omnc_stage1.ovm");

        let output = std::process::Command::new(&self.omnc_stage0)
            .arg(&main_omni)
            .arg("-o")
            .arg(&stage1_output)
            .output()
            .map_err(|e| format!("Failed to run stage0 omnc: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Stage 1 compilation failed: {}", stderr));
        }

        if !stage1_output.exists() {
            return Err("Stage 1 output not produced".to_string());
        }

        Ok(stage1_output)
    }

    /// Run Stage 1 → Stage 2: compile the Omni compiler source with Stage 1.
    pub fn run_stage2(&self, stage1: &Path) -> Result<PathBuf, String> {
        let main_omni = self.source_root.join("main.omni");
        let stage2_output = self.work_dir.join("omnc_stage2.ovm");

        let output = std::process::Command::new(&self.omnc_stage0)
            .arg("--run")
            .arg(stage1)
            .arg("--")
            .arg(&main_omni)
            .arg("-o")
            .arg(&stage2_output)
            .output()
            .map_err(|e| format!("Failed to run stage1: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Stage 2 compilation failed: {}", stderr));
        }

        if !stage2_output.exists() {
            return Err("Stage 2 output not produced".to_string());
        }

        Ok(stage2_output)
    }

    /// Verify that Stage 1 and Stage 2 binaries are identical (fixpoint).
    pub fn verify_fixpoint(&self, stage1: &Path, stage2: &Path) -> Result<(), String> {
        let h1 = sha256_file(stage1)?;
        let h2 = sha256_file(stage2)?;
        if h1 != h2 {
            return Err(format!(
                "Bootstrap FAILED: stage1 hash {} != stage2 hash {}",
                h1, h2
            ));
        }
        Ok(())
    }

    /// Get a summary of bootstrap readiness.
    pub fn readiness_check(&self) -> BootstrapReadiness {
        let stage0_exists = self.omnc_stage0.exists();
        let source_exists = self.source_root.join("main.omni").exists();
        let work_dir_writable = std::fs::create_dir_all(&self.work_dir).is_ok();

        BootstrapReadiness {
            stage0_binary: stage0_exists,
            source_available: source_exists,
            work_dir_writable,
            ready: stage0_exists && source_exists && work_dir_writable,
        }
    }
}

/// Result of a bootstrap readiness check.
#[derive(Debug, Clone)]
pub struct BootstrapReadiness {
    pub stage0_binary: bool,
    pub source_available: bool,
    pub work_dir_writable: bool,
    pub ready: bool,
}

/// Compute SHA-256 hash of a file, returned as a hex string.
pub fn sha256_file(path: &Path) -> Result<String, String> {
    use std::io::Read;
    let mut file =
        std::fs::File::open(path).map_err(|e| format!("Cannot open {}: {}", path.display(), e))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| format!("Read error: {}", e))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.hex_digest())
}

/// Minimal SHA-256 implementation (no external dependency).
struct Sha256 {
    state: [u32; 8],
    buffer: Vec<u8>,
    total_len: u64,
}

impl Sha256 {
    fn new() -> Self {
        Sha256 {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
                0x5be0cd19,
            ],
            buffer: Vec::new(),
            total_len: 0,
        }
    }

    fn update(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
        self.total_len += data.len() as u64;

        while self.buffer.len() >= 64 {
            let block: Vec<u8> = self.buffer.drain(..64).collect();
            self.process_block(&block);
        }
    }

    fn hex_digest(mut self) -> String {
        // Padding
        let bit_len = self.total_len * 8;
        self.buffer.push(0x80);
        while (self.buffer.len() % 64) != 56 {
            self.buffer.push(0);
        }
        self.buffer.extend_from_slice(&bit_len.to_be_bytes());

        // Process remaining blocks
        while self.buffer.len() >= 64 {
            let block: Vec<u8> = self.buffer.drain(..64).collect();
            self.process_block(&block);
        }

        // Format as hex
        self.state
            .iter()
            .map(|w| format!("{:08x}", w))
            .collect::<Vec<_>>()
            .join("")
    }

    fn process_block(&mut self, block: &[u8]) {
        const K: [u32; 64] = [
            0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
            0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
            0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
            0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
            0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
            0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
            0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
            0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
            0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
            0xc67178f2,
        ];

        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                block[i * 4],
                block[i * 4 + 1],
                block[i * 4 + 2],
                block[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
        self.state[5] = self.state[5].wrapping_add(f);
        self.state[6] = self.state[6].wrapping_add(g);
        self.state[7] = self.state[7].wrapping_add(h);
    }
}

// ─── Self-Hosting Compiler Architecture ──────────────────────────────────

/// Design document for bootstrapping Omni's self-hosted compiler
#[derive(Debug, Clone)]
pub struct SelfHostingPlan {
    pub phase: u32,
    pub description: String,
    pub target_completion: String,
    pub required_features: Vec<String>,
}

impl SelfHostingPlan {
    pub fn bootstrap_phases() -> Vec<Self> {
        vec![
            SelfHostingPlan {
                phase: 1,
                description: "Omni lexer implementation in Omni".to_string(),
                target_completion: "Q2 2026".to_string(),
                required_features: vec![
                    "Pattern matching".to_string(),
                    "String manipulation".to_string(),
                    "Enum types".to_string(),
                ],
            },
            SelfHostingPlan {
                phase: 2,
                description: "Omni parser implementation in Omni".to_string(),
                target_completion: "Q3 2026".to_string(),
                required_features: vec![
                    "Recursive data structures".to_string(),
                    "Error handling".to_string(),
                    "Trait objects".to_string(),
                ],
            },
            SelfHostingPlan {
                phase: 3,
                description: "Omni semantic analyzer in Omni".to_string(),
                target_completion: "Q4 2026".to_string(),
                required_features: vec![
                    "Type inference".to_string(),
                    "Generic types".to_string(),
                    "Trait bounds".to_string(),
                ],
            },
            SelfHostingPlan {
                phase: 4,
                description: "Omni IR generator in Omni".to_string(),
                target_completion: "Q1 2027".to_string(),
                required_features: vec![
                    "SSA construction".to_string(),
                    "Dominance analysis".to_string(),
                    "CFG manipulation".to_string(),
                ],
            },
            SelfHostingPlan {
                phase: 5,
                description: "Omni code generators (OVM, LLVM) in Omni".to_string(),
                target_completion: "Q2 2027".to_string(),
                required_features: vec![
                    "Binary code generation".to_string(),
                    "Machine instruction emission".to_string(),
                    "Linker integration".to_string(),
                ],
            },
            SelfHostingPlan {
                phase: 6,
                description: "Self-compilation: omnc_omni compiles itself".to_string(),
                target_completion: "Q3 2027".to_string(),
                required_features: vec![
                    "Full compiler feature parity".to_string(),
                    "Performance parity with Rust omnc".to_string(),
                    "Binary compatibility".to_string(),
                ],
            },
        ]
    }

    pub fn print_plan() {
        println!("\n=== OMNI SELF-HOSTING COMPILER BOOTSTRAP PLAN ===\n");
        for plan in Self::bootstrap_phases() {
            println!(
                "Phase {}: {}\n  Target: {}\n  Features: {}\n",
                plan.phase,
                plan.description,
                plan.target_completion,
                plan.required_features.join(", ")
            );
        }
    }
}

// ─── Omni Compiler Module Structure (for Omni implementation) ───────────

/// Blueprint for the lexer module when implemented in Omni
pub struct LexerModuleSpec;

impl LexerModuleSpec {
    pub fn omni_source() -> &'static str {
        r#"
// omni/compiler/lexer.omni
module compiler::lexer

pub enum TokenKind
  | Keyword(String)
  | Identifier(String)
  | Literal(LiteralKind)
  | Operator(String)
  | Punctuation(Char)
  | Eof

pub struct Token
  | kind: TokenKind
  | lexeme: String
  | line: Int
  | column: Int

pub fn tokenize(source: String) -> Result[Vec[Token], LexError]
  // Scan source character by character
  // Match keywords, identifiers, operators, literals
  // Track line/column for error reporting
  // Return token stream
"#
    }
}

/// Blueprint for the parser module when implemented in Omni
pub struct ParserModuleSpec;

impl ParserModuleSpec {
    pub fn omni_source() -> &'static str {
        r#"
// omni/compiler/parser.omni
module compiler::parser

use compiler::lexer

pub enum Expr
  | BinOp { left: Box[Expr], op: Op, right: Box[Expr] }
  | Call { func: Box[Expr], args: Vec[Expr] }
  | Literal(LiteralValue)
  | Variable(String)

pub enum Stmt
  | FnDef { name: String, params: Vec[Param], body: Vec[Stmt] }
  | Let { name: String, value: Expr }
  | Expr(Expr)

pub fn parse(tokens: Vec[Token]) -> Result[Vec[Stmt], ParseError]
  // Recursive descent parser
  // Convert token stream to AST
  // Validate syntax
"#
    }
}

/// Blueprint for the IR generator when implemented in Omni
pub struct IrGeneratorSpec;

impl IrGeneratorSpec {
    pub fn omni_source() -> &'static str {
        r#"
// omni/compiler/ir_gen.omni
module compiler::ir_gen

use compiler::parser

pub enum IrInst
  | BinOp { dest: String, op: Op, left: IrValue, right: IrValue }
  | Call { dest: Option[String], func: String, args: Vec[IrValue] }
  | Return(Option[IrValue])
  | Jump(String)
  | CondBranch { cond: IrValue, then_label: String, else_label: String }

pub fn generate(ast: Vec[Stmt]) -> Result[IrModule, IrError]
  // Walk AST
  // Generate SSA form
  // Perform initial optimizations
  // Return IR module
"#
    }
}

/// Blueprint for the code generator when implemented in Omni
pub struct CodegenSpec;

impl CodegenSpec {
    pub fn omni_source() -> &'static str {
        r#"
// omni/compiler/codegen.omni
module compiler::codegen

use compiler::ir_gen

pub enum Target
  | Ovm
  | X86_64
  | Arm64
  | Wasm

pub fn compile(ir: IrModule, target: Target) -> Result[Vec[u8], CodegenError]
  // Select appropriate code generator
  // Emit machine code or bytecode
  // Apply optimizations
  // Perform linking
"#
    }
}

// ─── Feature Checklist for Self-Hosting ──────────────────────────────────

/// Tracks which language features are needed for self-hosting
pub struct SelfHostingFeatures {
    pub features: HashMap<String, FeatureStatus>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FeatureStatus {
    NotImplemented,
    PartiallyImplemented,
    FullyImplemented,
    Optimized,
}

impl SelfHostingFeatures {
    pub fn new() -> Self {
        let mut features = HashMap::new();

        // Phase 1: Lexer features
        features.insert(
            "Pattern matching".to_string(),
            FeatureStatus::FullyImplemented,
        );
        features.insert(
            "String manipulation".to_string(),
            FeatureStatus::FullyImplemented,
        );
        features.insert("Enum types".to_string(), FeatureStatus::FullyImplemented);

        // Phase 2: Parser features
        features.insert(
            "Recursive data structures".to_string(),
            FeatureStatus::FullyImplemented,
        );
        features.insert(
            "Error handling".to_string(),
            FeatureStatus::FullyImplemented,
        );
        features.insert(
            "Trait objects".to_string(),
            FeatureStatus::PartiallyImplemented,
        );

        // Phase 3: Semantic analysis features
        // NOTE: As of Feb 2026, semantic analyzer is ~50% complete
        // - Type inference partial (no constraint solving)
        // - Generic monomorphization skeleton only (typed_body empty)
        // - Trait bounds basic implementation (no associated types)
        // - ZERO unit tests (critical gap)
        features.insert(
            "Type inference".to_string(),
            FeatureStatus::PartiallyImplemented,
        );
        features.insert(
            "Generic types".to_string(),
            FeatureStatus::PartiallyImplemented,
        );
        features.insert(
            "Trait bounds".to_string(),
            FeatureStatus::PartiallyImplemented,
        );

        // Phase 4: IR features
        features.insert(
            "SSA construction".to_string(),
            FeatureStatus::NotImplemented,
        );
        features.insert(
            "Dominance analysis".to_string(),
            FeatureStatus::NotImplemented,
        );
        features.insert(
            "CFG manipulation".to_string(),
            FeatureStatus::NotImplemented,
        );

        // Phase 5: Codegen features
        features.insert(
            "Binary code generation".to_string(),
            FeatureStatus::NotImplemented,
        );
        features.insert(
            "Machine instruction emission".to_string(),
            FeatureStatus::NotImplemented,
        );
        features.insert(
            "Linker integration".to_string(),
            FeatureStatus::NotImplemented,
        );

        Self { features }
    }

    pub fn check_phase_readiness(&self, phase: u32) -> bool {
        let required_features = match phase {
            1 => vec!["Pattern matching", "String manipulation", "Enum types"],
            2 => vec!["Recursive data structures", "Error handling"],
            3 => vec!["Type inference", "Generic types"],
            4 => vec!["SSA construction", "Dominance analysis", "CFG manipulation"],
            5 => vec!["Binary code generation", "Machine instruction emission"],
            _ => return false,
        };

        required_features.iter().all(|feat| {
            matches!(
                self.features.get(*feat),
                Some(FeatureStatus::FullyImplemented | FeatureStatus::Optimized)
            )
        })
    }

    pub fn report() -> String {
        let features = Self::new();
        let mut report = String::from("\n=== SELF-HOSTING READINESS REPORT ===\n\n");

        for phase in 1..=6 {
            let ready = features.check_phase_readiness(phase);
            report.push_str(&format!(
                "Phase {}: {}\n",
                phase,
                if ready { "✅ READY" } else { "⏳ PENDING" }
            ));
        }

        report.push_str("\nFeature Implementation Status:\n");
        for (name, status) in &features.features {
            let status_str = match status {
                FeatureStatus::NotImplemented => "❌ Not Implemented",
                FeatureStatus::PartiallyImplemented => "⚠️  Partial",
                FeatureStatus::FullyImplemented => "✅ Full",
                FeatureStatus::Optimized => "🚀 Optimized",
            };
            report.push_str(&format!("  {}: {}\n", name, status_str));
        }

        report
    }
}

// ─── Integration Points with Current Rust Compiler ──────────────────────

/// How the Omni compiler (once self-hosted) will integrate
pub struct SelfHostingIntegration {
    pub rust_omnc_status: String,
    pub omni_omnc_status: String,
    pub bootstrap_strategy: String,
}

impl SelfHostingIntegration {
    pub fn current_architecture() -> Self {
        Self {
            rust_omnc_status: "CURRENT (v1.0) - Stable production compiler".to_string(),
            omni_omnc_status: "IN DEVELOPMENT - Bootstrapping in phases".to_string(),
            bootstrap_strategy: "Dual compilation: Rust omnc compiles both Rust and Omni sources until feature parity"
                .to_string(),
        }
    }

    pub fn future_architecture() -> Self {
        Self {
            rust_omnc_status: "MAINTAINED (legacy) - Used only for initial bootstrap".to_string(),
            omni_omnc_status: "PRIMARY (v2.0) - Self-hosting production compiler".to_string(),
            bootstrap_strategy:
                "Omni omnc self-compiles: omnc_omni compiles itself and all Omni code".to_string(),
        }
    }

    pub fn print_architecture() {
        println!("\n=== CURRENT ARCHITECTURE ===");
        let current = Self::current_architecture();
        println!("Rust omnc: {}", current.rust_omnc_status);
        println!("Omni omnc: {}", current.omni_omnc_status);
        println!("Strategy: {}\n", current.bootstrap_strategy);

        println!("=== FUTURE ARCHITECTURE (Target 2027) ===");
        let future = Self::future_architecture();
        println!("Rust omnc: {}", future.rust_omnc_status);
        println!("Omni omnc: {}", future.omni_omnc_status);
        println!("Strategy: {}\n", future.bootstrap_strategy);
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bootstrap_phases() {
        let phases = SelfHostingPlan::bootstrap_phases();
        assert_eq!(phases.len(), 6);
        assert_eq!(phases[0].phase, 1);
        assert_eq!(phases[5].phase, 6);
    }

    #[test]
    fn test_phase_descriptions() {
        let phases = SelfHostingPlan::bootstrap_phases();
        assert!(phases[0].description.contains("lexer"));
        assert!(phases[1].description.contains("parser"));
        assert!(phases[2].description.contains("semantic"));
        assert!(phases[3].description.contains("IR"));
        assert!(phases[4].description.contains("code"));
        assert!(phases[5].description.contains("Self-compilation"));
    }

    #[test]
    fn test_phase_requirements() {
        let phases = SelfHostingPlan::bootstrap_phases();
        assert!(!phases[0].required_features.is_empty());
        assert!(!phases[1].required_features.is_empty());
    }

    #[test]
    fn test_selfhosting_features() {
        let features = SelfHostingFeatures::new();
        assert!(features.features.len() > 0);
    }

    #[test]
    fn test_phase_1_readiness() {
        let features = SelfHostingFeatures::new();
        assert!(features.check_phase_readiness(1)); // Phase 1 is ready
    }

    #[test]
    fn test_phase_3_not_ready() {
        let features = SelfHostingFeatures::new();
        assert!(!features.check_phase_readiness(3)); // Phase 3 not ready (type inference incomplete)
    }

    #[test]
    fn test_feature_report() {
        let report = SelfHostingFeatures::report();
        assert!(report.contains("Phase"));
        assert!(report.contains("Feature Implementation Status"));
    }

    #[test]
    fn test_lexer_module_spec() {
        let source = LexerModuleSpec::omni_source();
        assert!(source.contains("module compiler::lexer"));
        assert!(source.contains("TokenKind"));
        assert!(source.contains("fn tokenize"));
    }

    #[test]
    fn test_parser_module_spec() {
        let source = ParserModuleSpec::omni_source();
        assert!(source.contains("module compiler::parser"));
        assert!(source.contains("Expr"));
        assert!(source.contains("fn parse"));
    }

    #[test]
    fn test_ir_generator_spec() {
        let source = IrGeneratorSpec::omni_source();
        assert!(source.contains("module compiler::ir_gen"));
        assert!(source.contains("IrInst"));
        assert!(source.contains("fn generate"));
    }

    #[test]
    fn test_codegen_spec() {
        let source = CodegenSpec::omni_source();
        assert!(source.contains("module compiler::codegen"));
        assert!(source.contains("Target"));
        assert!(source.contains("fn compile"));
    }

    #[test]
    fn test_current_architecture() {
        let arch = SelfHostingIntegration::current_architecture();
        assert!(arch.rust_omnc_status.contains("CURRENT"));
        assert!(arch.omni_omnc_status.contains("DEVELOPMENT"));
    }

    #[test]
    fn test_future_architecture() {
        let arch = SelfHostingIntegration::future_architecture();
        assert!(arch.rust_omnc_status.contains("MAINTAINED"));
        assert!(arch.omni_omnc_status.contains("PRIMARY"));
    }

    // ─── BootstrapRunner tests ──────────────────────────────────────────

    #[test]
    fn test_bootstrap_runner_creation() {
        let runner = BootstrapRunner {
            omnc_stage0: PathBuf::from("/usr/bin/omnc"),
            source_root: PathBuf::from("/opt/omni/compiler"),
            work_dir: PathBuf::from("/tmp/bootstrap"),
        };
        assert_eq!(runner.omnc_stage0, PathBuf::from("/usr/bin/omnc"));
        assert_eq!(runner.source_root, PathBuf::from("/opt/omni/compiler"));
        assert_eq!(runner.work_dir, PathBuf::from("/tmp/bootstrap"));
    }

    #[test]
    fn test_fixpoint_verification_identical() {
        // Create two identical temp files and verify fixpoint passes
        let dir = std::env::temp_dir().join("omni_bootstrap_test");
        let _ = std::fs::create_dir_all(&dir);
        let f1 = dir.join("stage1.bin");
        let f2 = dir.join("stage2.bin");
        let data = b"hello bootstrap world";
        std::fs::write(&f1, data).unwrap();
        std::fs::write(&f2, data).unwrap();

        let runner = BootstrapRunner {
            omnc_stage0: PathBuf::from("omnc"),
            source_root: PathBuf::from("."),
            work_dir: dir.clone(),
        };
        assert!(runner.verify_fixpoint(&f1, &f2).is_ok());

        // cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_fixpoint_verification_different() {
        let dir = std::env::temp_dir().join("omni_bootstrap_diff");
        let _ = std::fs::create_dir_all(&dir);
        let f1 = dir.join("s1.bin");
        let f2 = dir.join("s2.bin");
        std::fs::write(&f1, b"aaa").unwrap();
        std::fs::write(&f2, b"bbb").unwrap();

        let runner = BootstrapRunner {
            omnc_stage0: PathBuf::from("omnc"),
            source_root: PathBuf::from("."),
            work_dir: dir.clone(),
        };
        let result = runner.verify_fixpoint(&f1, &f2);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("FAILED"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_sha256_known_vector() {
        // SHA-256 of empty string = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        let dir = std::env::temp_dir().join("omni_sha_test");
        let _ = std::fs::create_dir_all(&dir);
        let f = dir.join("empty.bin");
        std::fs::write(&f, b"").unwrap();
        let hash = sha256_file(&f).unwrap();
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_readiness_check_missing() {
        let runner = BootstrapRunner {
            omnc_stage0: PathBuf::from("/nonexistent/omnc"),
            source_root: PathBuf::from("/nonexistent/src"),
            work_dir: std::env::temp_dir().join("omni_readiness"),
        };
        let check = runner.readiness_check();
        assert!(!check.stage0_binary);
        assert!(!check.source_available);
        assert!(!check.ready);
    }
}
