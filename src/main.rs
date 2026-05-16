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

/// Main entry point: read program file or inline program string, then execute.
/// Usage:
///   postfix <program_file> [arg1 arg2 ...]
///   postfix -e "(postfix N ...)" [arg1 arg2 ...]
fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  {} <program_file> [arg1 arg2 ...]", args[0]);
        eprintln!("  {} -e \"(postfix N ...)\" [arg1 arg2 ...]", args[0]);
        std::process::exit(1);
    }

    let (program, rest) = if args[1] == "-e" {
        if args.len() < 3 {
            eprintln!("Error: -e requires a program string");
            std::process::exit(1);
        }
        (args[2].clone(), &args[3..])
    } else {
        let src = std::fs::read_to_string(&args[1])
            .unwrap_or_else(|e| {
                eprintln!("Error reading file '{}': {}", args[1], e);
                std::process::exit(1);
            })
            .replace('\n', " ")
            .trim()
            .to_string();
        (src, &args[2..])
    };

    let program_args: Vec<i32> = rest
        .iter()
        .map(|s| {
            s.parse::<i32>().unwrap_or_else(|_| {
                eprintln!("Error: '{}' is not a valid integer", s);
                std::process::exit(1);
            })
        })
        .collect();

    match process_program(&program, &program_args) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
