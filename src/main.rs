// PostFix language interpreter
// Modules: AST definition, parser, and interpreter

mod ast;
mod interpreter;
mod parser;

use interpreter::PostFix;

/// Parse and execute a PostFix program with given arguments
fn process_program(program: &str, args: &[i32]) -> Result<i32, String> {
    let commands = parser::parse_program(program, args)?;
    let mut postfix = PostFix::new(args.to_vec(), commands);
    postfix.run()
}

/// Main entry point: read program file, parse arguments, and execute
fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <program_file> [arg1 arg2 ...]", args[0]);
        std::process::exit(1);
    }

    let program_file = &args[1];
    let program = std::fs::read_to_string(program_file).unwrap_or_else(|e| {
        eprintln!("Error reading file '{}': {}", program_file, e);
        std::process::exit(1);
    });

    let program = program.replace('\n', " ").trim().to_string();

    let program_args: Vec<i32> = args[2..]
        .iter()
        .map(|s| {
            s.parse::<i32>().unwrap_or_else(|_| {
                eprintln!("Error: '{}' is not a valid integer", s);
                std::process::exit(1);
            })
        })
        .collect();

    match process_program(&program, &program_args) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
