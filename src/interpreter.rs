use crate::ast::{Command, Value};

/// PostFix interpreter with data and command stacks
pub struct PostFix {
    data_stack: Vec<Value>,
    command_stack: Vec<Command>,
}

impl PostFix {
    /// Creates a new PostFix interpreter with initial parameters and commands
    pub fn new(params: Vec<i32>, commands: Vec<Command>) -> Self {
        let mut data_stack: Vec<Value> = params.into_iter().map(Value::Number).collect();
        data_stack.reverse();

        let mut command_stack = commands;
        command_stack.reverse();

        PostFix {
            data_stack,
            command_stack,
        }
    }

    /// Executes the program and returns the final result
    pub fn run(&mut self) -> Result<i32, String> {
        eprintln!(
            "DEBUG: Starting execution with {} commands",
            self.command_stack.len()
        );
        eprintln!("DEBUG: Initial data stack: {:?}", self.data_stack);

        while let Some(cmd) = self.command_stack.pop() {
            eprintln!("DEBUG: Executing: {:?}", cmd);
            eprintln!("DEBUG: Stack before: {:?}", self.data_stack);
            self.execute(cmd)?;
            eprintln!("DEBUG: Stack after: {:?}", self.data_stack);
            eprintln!("DEBUG: Remaining commands: {:?}", self.command_stack);
        }

        match self.data_stack.last() {
            Some(Value::Number(n)) => Ok(*n),
            Some(Value::Sequence(_)) => Err("Final value is not an integer".to_string()),
            None => Err("Empty stack at end".to_string()),
        }
    }

    /// Executes a single command
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
                // Select: if v3 != 0 then v2 else v1
                // Stack before: v1 v2 v3
                // Stack after: result
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
                // Copy nth value from stack (1-indexed from top)
                // nget 1 copies top, nget 2 copies second, etc.
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
                // Execute a sequence: push commands in reverse order
                // so they execute in correct order when popped
                if let Some(Value::Sequence(cmds)) = self.data_stack.pop() {
                    eprintln!(
                        "DEBUG: Executing sequence with {} commands: {:?}",
                        cmds.len(),
                        cmds
                    );
                    eprintln!("DEBUG: Current command stack: {:?}", self.command_stack);

                    for c in cmds.into_iter().rev() {
                        self.command_stack.push(c);
                    }

                    eprintln!("DEBUG: Command stack after exec: {:?}", self.command_stack);
                } else {
                    return Err("exec requires a sequence".to_string());
                }
            }
        }
        Ok(())
    }

    /// Pop a number from the data stack
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
