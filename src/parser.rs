use pest::Parser;
use pest_derive::Parser;

use crate::ast::Command;

/// Pest parser for PostFix grammar
#[derive(Parser)]
#[grammar = "./../postfix.pest"]
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

    // If there's a single outer sequence, unwrap it for execution
    let commands = if commands.len() == 1 {
        match &commands[0] {
            Command::Sequence(cmds) => cmds.clone(),
            _ => commands,
        }
    } else {
        commands
    };

    Ok(commands)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_number() {
        let program = "(postfix 0 (42))";
        let args = [];
        let result = parse_program(program, &args).unwrap();
        assert_eq!(result.len(), 1);
        // Outer sequence is now unwrapped
        assert!(matches!(result[0], Command::Number(42)));
    }

    #[test]
    fn test_parse_arithmetic() {
        let program = "(postfix 0 (add sub mul div rem))";
        let args = [];
        let result = parse_program(program, &args).unwrap();
        // Outer sequence is unwrapped, so we get 5 commands directly
        assert_eq!(result.len(), 5);
        assert!(matches!(result[0], Command::Add));
        assert!(matches!(result[1], Command::Sub));
        assert!(matches!(result[2], Command::Mul));
        assert!(matches!(result[3], Command::Div));
        assert!(matches!(result[4], Command::Rem));
    }

    #[test]
    fn test_parse_comparisons() {
        let program = "(postfix 0 (lt gt eq))";
        let args = [];
        let result = parse_program(program, &args).unwrap();
        assert_eq!(result.len(), 3);
        assert!(matches!(result[0], Command::Lt));
        assert!(matches!(result[1], Command::Gt));
        assert!(matches!(result[2], Command::Eq));
    }

    #[test]
    fn test_parse_stack_ops() {
        let program = "(postfix 0 (pop swap sel nget exec))";
        let args = [];
        let result = parse_program(program, &args).unwrap();
        assert_eq!(result.len(), 5);
        assert!(matches!(result[0], Command::Pop));
        assert!(matches!(result[1], Command::Swap));
        assert!(matches!(result[2], Command::Sel));
        assert!(matches!(result[3], Command::Nget));
        assert!(matches!(result[4], Command::Exec));
    }

    #[test]
    fn test_parse_sequence() {
        let program = "(postfix 0 ((1 2 add)))";
        let args = [];
        let result = parse_program(program, &args).unwrap();
        // Outer sequence is unwrapped, inner sequence remains
        assert_eq!(result.len(), 1);
        match &result[0] {
            Command::Sequence(inner_cmds) => {
                assert_eq!(inner_cmds.len(), 3);
                assert!(matches!(inner_cmds[0], Command::Number(1)));
                assert!(matches!(inner_cmds[1], Command::Number(2)));
                assert!(matches!(inner_cmds[2], Command::Add));
            }
            _ => panic!("Expected inner sequence"),
        }
    }

    #[test]
    fn test_arg_count_validation() {
        let program = "(postfix 2 (add))";
        let args = [1];
        let result = parse_program(program, &args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Arg count mismatch"));
    }

    #[test]
    fn test_correct_arg_count() {
        let program = "(postfix 2 (add))";
        let args = [1, 2];
        let result = parse_program(program, &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_nested_sequences() {
        let program = "(postfix 0 (((1 2 add) (3 4 mul))))";
        let args = [];
        let result = parse_program(program, &args).unwrap();
        // Outer sequence is unwrapped, middle sequence remains
        assert_eq!(result.len(), 1);
        match &result[0] {
            Command::Sequence(middle_cmds) => {
                // Middle sequence containing two sequences
                assert_eq!(middle_cmds.len(), 2);
                match &middle_cmds[0] {
                    Command::Sequence(inner_cmds) => {
                        assert_eq!(inner_cmds.len(), 3);
                        assert!(matches!(inner_cmds[2], Command::Add));
                    }
                    _ => panic!("Expected inner sequence"),
                }
            }
            _ => panic!("Expected middle sequence"),
        }
    }
}
