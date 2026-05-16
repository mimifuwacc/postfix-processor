use crate::ast::{Command, Value};

pub struct PostFix {
    data_stack: Vec<Value>,
    command_stack: Vec<Command>,
    args: Vec<i32>,
    original_commands: Vec<Command>,
}

// --- formatting helpers ---

fn fmt_cmd(cmd: &Command) -> String {
    match cmd {
        Command::Number(n) => n.to_string(),
        Command::Add => "add".into(),
        Command::Sub => "sub".into(),
        Command::Mul => "mul".into(),
        Command::Div => "div".into(),
        Command::Rem => "rem".into(),
        Command::Lt => "lt".into(),
        Command::Gt => "gt".into(),
        Command::Eq => "eq".into(),
        Command::Pop => "pop".into(),
        Command::Swap => "swap".into(),
        Command::Sel => "sel".into(),
        Command::Nget => "nget".into(),
        Command::Exec => "exec".into(),
        Command::Sequence(cmds) => fmt_seq(cmds),
    }
}

fn fmt_seq(cmds: &[Command]) -> String {
    let parts: Vec<_> = cmds.iter().map(fmt_cmd).collect();
    format!("({})", parts.join(" "))
}

fn fmt_val(val: &Value) -> String {
    match val {
        Value::Number(n) => n.to_string(),
        Value::Sequence(cmds) => fmt_seq(cmds),
    }
}

// command_stack is stored reversed (pop from end = next to execute)
fn fmt_queue(cmd_stack: &[Command]) -> String {
    let parts: Vec<_> = cmd_stack.iter().rev().map(fmt_cmd).collect();
    format!("({})", parts.join(" "))
}

// data_stack is stored [bottom..top]; display top first
fn fmt_data(data_stack: &[Value]) -> String {
    let parts: Vec<_> = data_stack.iter().rev().map(fmt_val).collect();
    format!("[{}]", parts.join(", "))
}

fn fmt_config(cmd_stack: &[Command], data_stack: &[Value]) -> String {
    format!("⟨{}, {}⟩", fmt_queue(cmd_stack), fmt_data(data_stack))
}

// Determine the rewrite rule label before executing the command.
fn rule_name(cmd: &Command, data_stack: &[Value]) -> &'static str {
    match cmd {
        Command::Number(_) => "num",
        Command::Sequence(_) => "seq",
        Command::Pop => "pop",
        Command::Nget => "nget",
        Command::Swap => "swap",
        Command::Exec => "execute",
        Command::Add | Command::Sub | Command::Mul | Command::Div | Command::Rem => "arithop",
        Command::Sel => {
            // Stack layout (top first): Vfalse · Vtrue · Ntest · S
            // Ntest is at data_stack[len-3]
            let n = data_stack.len();
            if n >= 3 {
                match &data_stack[n - 3] {
                    Value::Number(0) => "sel-false",
                    _ => "sel-true",
                }
            } else {
                "sel-false"
            }
        }
        Command::Lt => relop_rule(data_stack, |v2, v1| v2 < v1),
        Command::Gt => relop_rule(data_stack, |v2, v1| v2 > v1),
        Command::Eq => relop_rule(data_stack, |v2, v1| v2 == v1),
    }
}

fn relop_rule(data_stack: &[Value], cmp: impl Fn(i32, i32) -> bool) -> &'static str {
    // Stack layout: N1 (top) · N2 · S  →  compare R N2 N1
    let n = data_stack.len();
    if n >= 2 {
        if let (Value::Number(v1), Value::Number(v2)) =
            (&data_stack[n - 1], &data_stack[n - 2])
        {
            if cmp(*v2, *v1) { "relop-true" } else { "relop-false" }
        } else {
            "relop-false"
        }
    } else {
        "relop-false"
    }
}

impl PostFix {
    pub fn new(params: Vec<i32>, commands: Vec<Command>) -> Self {
        let args = params.clone();
        let original_commands = commands.clone();

        let mut data_stack: Vec<Value> = params.into_iter().map(Value::Number).collect();
        data_stack.reverse();

        let mut command_stack = commands;
        command_stack.reverse();

        PostFix { data_stack, command_stack, args, original_commands }
    }

    pub fn run(&mut self) -> Result<i32, String> {
        let n = self.args.len();
        let prog = format!(
            "(postfix {} {})",
            n,
            self.original_commands.iter().map(fmt_cmd).collect::<Vec<_>>().join(" ")
        );
        let args_str = format!(
            "[{}]",
            self.args.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ")
        );

        println!("(IF ⟨{}, {}⟩)", prog, args_str);
        println!("= {}", fmt_config(&self.command_stack, &self.data_stack));

        while let Some(cmd) = self.command_stack.pop() {
            let rule = rule_name(&cmd, &self.data_stack);
            self.execute(cmd)?;
            let config = fmt_config(&self.command_stack, &self.data_stack);
            if self.command_stack.is_empty() {
                println!("⇒ {} ∈ FC  [{}]", config, rule);
            } else {
                println!("⇒ {}  [{}]", config, rule);
            }
        }

        match self.data_stack.last() {
            Some(Value::Number(n)) => {
                println!("(OF {}) = {}", fmt_config(&self.command_stack, &self.data_stack), n);
                Ok(*n)
            }
            Some(Value::Sequence(_)) => Err("Final value is not an integer".to_string()),
            None => Err("Empty stack at end".to_string()),
        }
    }

    fn execute(&mut self, cmd: Command) -> Result<(), String> {
        match cmd {
            Command::Number(n) => {
                self.data_stack.push(Value::Number(n));
            }
            Command::Add => {
                let v1 = self.pop_int()?;
                let v2 = self.pop_int()?;
                self.data_stack.push(Value::Number(v2 + v1));
            }
            Command::Sub => {
                let v1 = self.pop_int()?;
                let v2 = self.pop_int()?;
                self.data_stack.push(Value::Number(v2 - v1));
            }
            Command::Mul => {
                let v1 = self.pop_int()?;
                let v2 = self.pop_int()?;
                self.data_stack.push(Value::Number(v2 * v1));
            }
            Command::Div => {
                let v1 = self.pop_int()?;
                let v2 = self.pop_int()?;
                if v1 == 0 {
                    return Err("Division by zero".to_string());
                }
                self.data_stack.push(Value::Number(v2 / v1));
            }
            Command::Rem => {
                let v1 = self.pop_int()?;
                let v2 = self.pop_int()?;
                if v1 == 0 {
                    return Err("Remainder by zero".to_string());
                }
                self.data_stack.push(Value::Number(v2 % v1));
            }
            Command::Lt => {
                let v1 = self.pop_int()?;
                let v2 = self.pop_int()?;
                self.data_stack.push(Value::Number((v2 < v1) as i32));
            }
            Command::Gt => {
                let v1 = self.pop_int()?;
                let v2 = self.pop_int()?;
                self.data_stack.push(Value::Number((v2 > v1) as i32));
            }
            Command::Eq => {
                let v1 = self.pop_int()?;
                let v2 = self.pop_int()?;
                self.data_stack.push(Value::Number((v2 == v1) as i32));
            }
            Command::Pop => {
                if self.data_stack.is_empty() {
                    return Err("pop: stack underflow".to_string());
                }
                self.data_stack.pop();
            }
            Command::Swap => {
                if self.data_stack.len() < 2 {
                    return Err("swap: stack underflow (need 2 values)".to_string());
                }
                let v1 = self.data_stack.pop().unwrap();
                let v2 = self.data_stack.pop().unwrap();
                self.data_stack.push(v1);
                self.data_stack.push(v2);
            }
            Command::Sel => {
                if self.data_stack.len() < 3 {
                    return Err("sel: stack underflow (need 3 values)".to_string());
                }
                let v1 = self.data_stack.pop().unwrap();
                let v2 = self.data_stack.pop().unwrap();
                let v3 = self.pop_int()?;
                let result = if v3 != 0 { v2 } else { v1 };
                self.data_stack.push(result);
            }
            Command::Nget => {
                if self.data_stack.is_empty() {
                    return Err("nget: stack underflow".to_string());
                }
                let n = self.pop_int()?;
                if n <= 0 || n as usize > self.data_stack.len() {
                    return Err(format!(
                        "nget: index out of bounds (requested {}, but stack size is {})",
                        n,
                        self.data_stack.len()
                    ));
                }
                let value = self.data_stack[self.data_stack.len() - n as usize].clone();
                self.data_stack.push(value);
            }
            Command::Sequence(cmds) => {
                self.data_stack.push(Value::Sequence(cmds));
            }
            Command::Exec => {
                if let Some(Value::Sequence(cmds)) = self.data_stack.pop() {
                    for c in cmds.into_iter().rev() {
                        self.command_stack.push(c);
                    }
                } else {
                    return Err("exec requires a sequence".to_string());
                }
            }
        }
        Ok(())
    }

    fn pop_int(&mut self) -> Result<i32, String> {
        match self.data_stack.pop() {
            Some(Value::Number(n)) => Ok(n),
            Some(Value::Sequence(_)) => {
                Err("Expected number but got executable sequence".to_string())
            }
            None => Err("Stack underflow".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic_operations() {
        let commands = vec![Command::Number(3), Command::Number(4), Command::Add];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 7);

        let commands = vec![Command::Number(10), Command::Number(3), Command::Sub];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 7);

        let commands = vec![Command::Number(5), Command::Number(6), Command::Mul];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 30);

        let commands = vec![Command::Number(20), Command::Number(4), Command::Div];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 5);

        let commands = vec![Command::Number(17), Command::Number(5), Command::Rem];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 2);
    }

    #[test]
    fn test_comparison_operations() {
        let commands = vec![Command::Number(3), Command::Number(5), Command::Lt];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 1);

        let commands = vec![Command::Number(5), Command::Number(3), Command::Gt];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 1);

        let commands = vec![Command::Number(4), Command::Number(4), Command::Eq];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 1);

        let commands = vec![Command::Number(3), Command::Number(5), Command::Eq];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 0);
    }

    #[test]
    fn test_stack_operations() {
        let commands = vec![
            Command::Number(1),
            Command::Number(2),
            Command::Number(3),
            Command::Pop,
        ];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 2);

        let commands = vec![Command::Number(1), Command::Number(2), Command::Swap];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 1);
    }

    #[test]
    fn test_nget() {
        let commands = vec![
            Command::Number(10),
            Command::Number(20),
            Command::Number(30),
            Command::Number(1),
            Command::Nget,
        ];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 30);

        let commands = vec![
            Command::Number(10),
            Command::Number(20),
            Command::Number(30),
            Command::Number(2),
            Command::Nget,
        ];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 20);
    }

    #[test]
    fn test_sel() {
        let commands = vec![
            Command::Number(0),
            Command::Number(20),
            Command::Number(10),
            Command::Sel,
        ];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 10);

        let commands = vec![
            Command::Number(0),
            Command::Number(20),
            Command::Number(5),
            Command::Sel,
        ];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 5);
    }

    #[test]
    fn test_exec() {
        let seq = vec![Command::Number(5), Command::Mul];
        let commands = vec![Command::Number(2), Command::Sequence(seq), Command::Exec];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 10);
    }

    #[test]
    fn test_with_parameters() {
        let commands = vec![Command::Add];
        let mut postfix = PostFix::new(vec![3, 4], commands);
        assert_eq!(postfix.run().unwrap(), 7);
    }

    #[test]
    fn test_division_by_zero() {
        let commands = vec![Command::Number(10), Command::Number(0), Command::Div];
        let mut postfix = PostFix::new(vec![], commands);
        assert!(postfix.run().is_err());
    }

    #[test]
    fn test_stack_underflow() {
        let commands = vec![Command::Pop];
        let mut postfix = PostFix::new(vec![], commands);
        assert!(postfix.run().is_err());

        let commands = vec![Command::Number(5), Command::Add];
        let mut postfix = PostFix::new(vec![], commands);
        assert!(postfix.run().is_err());
    }

    #[test]
    fn test_complex_expression() {
        let add_seq = vec![Command::Number(4), Command::Add];
        let commands = vec![
            Command::Number(3),
            Command::Sequence(add_seq),
            Command::Exec,
            Command::Number(5),
            Command::Mul,
        ];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 35);
    }

    #[test]
    fn test_conditional_execution() {
        let seq1 = vec![Command::Number(10)];
        let seq2 = vec![Command::Number(20)];
        let commands = vec![
            Command::Number(0),
            Command::Sequence(seq1),
            Command::Sequence(seq2),
            Command::Sel,
            Command::Exec,
        ];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 20);
    }
}
