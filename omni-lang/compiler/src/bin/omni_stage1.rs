#![allow(dead_code)]

#[path = "../ast.rs"]
mod ast;
#[path = "../ir.rs"]
mod ir;
#[path = "../lexer.rs"]
mod lexer;
#[path = "../parser.rs"]
mod parser;
#[path = "../semantics.rs"]
mod semantics;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use ir::IRGenerator;
use lexer::Lexer;
use parser::Parser;
use semantics::TypeChecker;

fn main() {
    if let Err(err) = run() {
        eprintln!("omni_stage1 error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        print_usage();
        return Err("missing required arguments".to_string());
    }

    let command = args[1].as_str();
    let input_path = PathBuf::from(&args[2]);
    let source = fs::read_to_string(&input_path)
        .map_err(|e| format!("failed to read {}: {}", input_path.display(), e))?;

    let ir_module = compile_to_ir(&source)?;

    match command {
        "check" => {
            println!("ok: {} passed lexer/parser/semantic/ir pipeline", input_path.display());
            Ok(())
        }
        "emit-ir" => {
            let output = parse_output_path(&args[3..], &input_path)?;
            let ir_text = format!("{:#?}\n", ir_module);
            fs::write(&output, ir_text)
                .map_err(|e| format!("failed to write {}: {}", output.display(), e))?;
            println!("ok: wrote IR to {}", output.display());
            Ok(())
        }
        _ => {
            print_usage();
            Err(format!("unsupported command: {command}"))
        }
    }
}

fn compile_to_ir(source: &str) -> Result<ir::IRModule, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;

    let mut type_checker = TypeChecker::new();
    let typed_ast = type_checker.check(&ast)?;

    let mut ir_generator = IRGenerator::new();
    ir_generator.generate(&typed_ast)
}

fn parse_output_path(extra_args: &[String], input_path: &Path) -> Result<PathBuf, String> {
    if extra_args.len() == 2 && extra_args[0] == "-o" {
        return Ok(PathBuf::from(&extra_args[1]));
    }

    if extra_args.is_empty() {
        let stem = input_path
            .file_stem()
            .ok_or_else(|| "input file has no stem".to_string())?
            .to_string_lossy();
        return Ok(PathBuf::from(format!("{stem}.oir")));
    }

    Err("emit-ir accepts optional '-o <path>'".to_string())
}

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  omni_stage1 check <file.omni>");
    eprintln!("  omni_stage1 emit-ir <file.omni> [-o output.oir]");
}
