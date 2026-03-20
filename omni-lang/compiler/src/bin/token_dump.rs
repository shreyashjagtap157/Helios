use std::env;
use std::fs;
use omni_compiler::lexer::tokenize;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: token_dump <path.omni>");
        std::process::exit(1);
    }
    let path = &args[1];
    let src = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Failed to read {}: {}", path, e);
        std::process::exit(1);
    });

    let tokens = tokenize(&src).unwrap_or_else(|e| {
        eprintln!("Lexing failed: {}", e);
        std::process::exit(1);
    });

    for t in tokens {
        println!("{}:{} {:?} {}", t.line, t.column, t.kind, t.lexeme);
    }
}
