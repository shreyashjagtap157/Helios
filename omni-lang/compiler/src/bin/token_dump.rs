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

use omni_compiler::lexer::tokenize;
use std::env;
use std::fs;

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
