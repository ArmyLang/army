mod ast;
mod lexer;
mod token;

use lexer::Lexer;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: army-lang <source.army>");
        process::exit(1);
    }

    let file_path = &args[1];
    let src = match fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", file_path, e);
            process::exit(1);
        }
    };

    let mut lexer = Lexer::new(&src);
    match lexer.scan() {
        Ok(tokens) => {
            println!("=== Tokens ({}) ===", tokens.len());
            for tok in &tokens {
                println!("  {}", tok);
            }
        }
        Err(e) => {
            eprintln!("Lexer error: {}", e);
            process::exit(1);
        }
    }
}
