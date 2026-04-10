use pest::Parser;
use pest_derive::Parser;

use crate::ast::Command;

/// Pest parser for PostFix grammar
#[derive(Parser)]
#[grammar = "grammar.pest"]
struct PostfixParser;

/// Parse a single command from a pest pair
fn parse_command(pair: pest::iterators::Pair<Rule>) -> Result<Command, String> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::number => {
            let value: i32 = inner.as_str().parse().unwrap();
            Ok(Command::Number(value))
        }
        Rule::infix_op => match inner.as_str() {
            "add" => Ok(Command::Add),
            "sub" => Ok(Command::Sub),
            "mul" => Ok(Command::Mul),
            "div" => Ok(Command::Div),
            "rem" => Ok(Command::Rem),
            _ => Err(format!("Unknown infix op: {}", inner.as_str())),
        },
        Rule::comparison_op => match inner.as_str() {
            "lt" => Ok(Command::Lt),
            "gt" => Ok(Command::Gt),
            "eq" => Ok(Command::Eq),
            _ => Err(format!("Unknown comparison op: {}", inner.as_str())),
        },
        Rule::stack_op => match inner.as_str() {
            "pop" => Ok(Command::Pop),
            "swap" => Ok(Command::Swap),
            "sel" => Ok(Command::Sel),
            "nget" => Ok(Command::Nget),
            "exec" => Ok(Command::Exec),
            _ => Err(format!("Unknown stack op: {}", inner.as_str())),
        },
        Rule::executable_sequence => {
            let mut commands = Vec::new();
            for inner_cmd in inner.into_inner() {
                commands.push(parse_command(inner_cmd)?);
            }
            Ok(Command::Sequence(commands))
        }
        _ => Err(format!("Unknown command rule: {:?}", inner.as_rule())),
    }
}

/// Parse multiple commands from pest pairs
fn parse_commands(pairs: pest::iterators::Pairs<Rule>) -> Result<Vec<Command>, String> {
    pairs.map(|pair| parse_command(pair)).collect()
}

/// Parse a PostFix program and validate argument count
pub fn parse_program(program: &str, args: &[i32]) -> Result<Vec<Command>, String> {
    let pairs =
        PostfixParser::parse(Rule::program, program).map_err(|e| format!("Parse error: {}", e))?;
    let pair = pairs.into_iter().next().unwrap();
    let mut inner = pair.into_inner();

    let arg_count_pair = inner.find(|p| p.as_rule() == Rule::arg_count).unwrap();
    let arg_count: usize = arg_count_pair
        .into_inner()
        .next()
        .unwrap()
        .as_str()
        .parse()
        .map_err(|e| format!("Invalid arg count: {}", e))?;

    if arg_count != args.len() {
        return Err(format!(
            "Arg count mismatch: expected {} arguments, got {}",
            arg_count,
            args.len()
        ));
    }

    let commands_pair = inner.find(|p| p.as_rule() == Rule::commands).unwrap();
    let commands = parse_commands(commands_pair.into_inner())?;

    eprintln!("DEBUG: Parsed {} commands:", commands.len());
    for (i, cmd) in commands.iter().enumerate() {
        eprintln!("  [{}] {:?}", i, cmd);
    }

    Ok(commands)
}
