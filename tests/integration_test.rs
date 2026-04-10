// Integration tests for the PostFix interpreter
// These test the full pipeline: parsing -> execution

fn parse_and_execute(program: &str, args: &[i32]) -> Result<i32, String> {
    let commands = postfix_processor::parser::parse_program(program, args)?;
    let mut interpreter = postfix_processor::interpreter::PostFix::new(args.to_vec(), commands);
    interpreter.run()
}

#[test]
fn test_simple_arithmetic() {
    // 3 4 add = 7
    let program = "(postfix 0 (3 4 add))";
    let args = [];
    assert_eq!(parse_and_execute(program, &args).unwrap(), 7);
}

#[test]
fn test_with_arguments() {
    // add two arguments
    let program = "(postfix 2 (add))";
    let args = [5, 3];
    assert_eq!(parse_and_execute(program, &args).unwrap(), 8);
}

#[test]
fn test_complex_arithmetic() {
    // (3 + 4) * 5 = 35
    let program = "(postfix 0 (3 4 add 5 mul))";
    let args = [];
    assert_eq!(parse_and_execute(program, &args).unwrap(), 35);
}

#[test]
fn test_comparison() {
    // 5 < 10 = 1 (true)
    let program = "(postfix 0 (5 10 lt))";
    let args = [];
    assert_eq!(parse_and_execute(program, &args).unwrap(), 1);
}

#[test]
fn test_conditional() {
    // 0 20 10 sel -> v1=10, v2=20, v3=0 -> v3==0, so returns v1=10
    let program = "(postfix 0 (0 20 10 sel))";
    let args = [];
    assert_eq!(parse_and_execute(program, &args).unwrap(), 10);

    // 5 20 10 sel -> v1=10, v2=20, v3=5 -> v3!=0, so returns v2=20
    let program2 = "(postfix 0 (5 20 10 sel))";
    assert_eq!(parse_and_execute(program2, &args).unwrap(), 20);
}

#[test]
fn test_exec_simple() {
    // Execute a simple sequence
    let program = "(postfix 0 (2 (5 mul) exec))";
    let args = [];
    assert_eq!(parse_and_execute(program, &args).unwrap(), 10);
}

#[test]
fn test_nested_sequences() {
    // More complex execution with sequences
    let program = "(postfix 0 (3 (4 add) exec 5 mul))";
    let args = [];
    assert_eq!(parse_and_execute(program, &args).unwrap(), 35);
}

#[test]
fn test_nget_operation() {
    // Copy and multiply
    let program = "(postfix 0 (10 20 1 nget mul))";
    let args = [];
    assert_eq!(parse_and_execute(program, &args).unwrap(), 400); // 20 * 20
}

#[test]
fn test_swap_operation() {
    // 3 5 swap -> returns 3
    let program = "(postfix 0 (3 5 swap))";
    let args = [];
    assert_eq!(parse_and_execute(program, &args).unwrap(), 3);
}

#[test]
fn test_pop_operation() {
    // 1 2 3 pop -> returns 2
    let program = "(postfix 0 (1 2 3 pop))";
    let args = [];
    assert_eq!(parse_and_execute(program, &args).unwrap(), 2);
}

#[test]
fn test_factorial() {
    // Simple factorial-like computation: 5! = 120
    // Using: 5 4 3 2 1 mul mul mul mul
    let program = "(postfix 0 (5 4 3 2 1 mul mul mul mul))";
    let args = [];
    assert_eq!(parse_and_execute(program, &args).unwrap(), 120);
}

#[test]
fn test_division_and_remainder() {
    // 17 / 5 = 3, 17 % 5 = 2
    let program = "(postfix 0 (17 5 div))";
    let args = [];
    assert_eq!(parse_and_execute(program, &args).unwrap(), 3);

    let program = "(postfix 0 (17 5 rem))";
    let args = [];
    assert_eq!(parse_and_execute(program, &args).unwrap(), 2);
}

#[test]
fn test_argument_count_validation() {
    // Should fail: expects 2 args but provides 1
    let program = "(postfix 2 (add))";
    let args = [5];
    assert!(parse_and_execute(program, &args).is_err());
}

#[test]
fn test_division_by_zero_error() {
    let program = "(postfix 0 (10 0 div))";
    let args = [];
    assert!(parse_and_execute(program, &args).is_err());
}

#[test]
fn test_stack_underflow_error() {
    let program = "(postfix 0 (add))"; // add needs 2 values
    let args = [];
    assert!(parse_and_execute(program, &args).is_err());
}

#[test]
fn test_empty_program() {
    let program = "(postfix 0 ())";
    let args = [];
    assert!(parse_and_execute(program, &args).is_err()); // Should fail: empty stack at end
}

#[test]
fn test_multiple_operations() {
    // (10 - 3) + (5 * 2) = 7 + 10 = 17
    let program = "(postfix 0 (10 3 sub 5 2 mul add))";
    let args = [];
    assert_eq!(parse_and_execute(program, &args).unwrap(), 17);
}
