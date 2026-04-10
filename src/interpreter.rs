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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic_operations() {
        // 3 4 add = 7
        let commands = vec![Command::Number(3), Command::Number(4), Command::Add];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 7);

        // 10 3 sub = 7
        let commands = vec![Command::Number(10), Command::Number(3), Command::Sub];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 7);

        // 5 6 mul = 30
        let commands = vec![Command::Number(5), Command::Number(6), Command::Mul];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 30);

        // 20 4 div = 5
        let commands = vec![Command::Number(20), Command::Number(4), Command::Div];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 5);

        // 17 5 rem = 2
        let commands = vec![Command::Number(17), Command::Number(5), Command::Rem];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 2);
    }

    #[test]
    fn test_comparison_operations() {
        // 3 5 lt = 1 (true)
        let commands = vec![Command::Number(3), Command::Number(5), Command::Lt];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 1);

        // 5 3 gt = 1 (true)
        let commands = vec![Command::Number(5), Command::Number(3), Command::Gt];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 1);

        // 4 4 eq = 1 (true)
        let commands = vec![Command::Number(4), Command::Number(4), Command::Eq];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 1);

        // 3 5 eq = 0 (false)
        let commands = vec![Command::Number(3), Command::Number(5), Command::Eq];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 0);
    }

    #[test]
    fn test_stack_operations() {
        // pop: remove top value
        // 1 2 3 pop -> should return 2
        let commands = vec![
            Command::Number(1),
            Command::Number(2),
            Command::Number(3),
            Command::Pop,
        ];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 2);

        // swap: swap top two values
        // 1 2 swap -> should return 1 (2 and 1 are swapped)
        let commands = vec![Command::Number(1), Command::Number(2), Command::Swap];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 1);
    }

    #[test]
    fn test_nget() {
        // nget: copy nth value from stack
        // 10 20 30 1 nget -> copies 30, returns 30
        let commands = vec![
            Command::Number(10),
            Command::Number(20),
            Command::Number(30),
            Command::Number(1),
            Command::Nget,
        ];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 30);

        // 10 20 30 2 nget -> copies 20, returns 20
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
        // sel: select based on v3 (3rd from top)
        // Stack: 0 20 10 sel -> v1=10, v2=20, v3=0 -> v3==0, so returns v1=10
        let commands = vec![
            Command::Number(0),
            Command::Number(20),
            Command::Number(10),
            Command::Sel,
        ];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 10);

        // Stack: 0 20 5 sel -> v1=5, v2=20, v3=0 -> v3==0, so returns v1=5
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
        // exec: execute a sequence
        // 2 ((5 mul) exec) -> should return 10
        let seq = vec![Command::Number(5), Command::Mul];
        let commands = vec![Command::Number(2), Command::Sequence(seq), Command::Exec];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 10);
    }

    #[test]
    fn test_with_parameters() {
        // Test with initial parameters: params=[3, 4], then add = 7
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
        // pop on empty stack
        let commands = vec![Command::Pop];
        let mut postfix = PostFix::new(vec![], commands);
        assert!(postfix.run().is_err());

        // add with insufficient values
        let commands = vec![Command::Number(5), Command::Add];
        let mut postfix = PostFix::new(vec![], commands);
        assert!(postfix.run().is_err());
    }

    #[test]
    fn test_complex_expression() {
        // 3 (4 add) exec 5 mul -> 3+4=7, 7*5=35
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
        // 0 (10) (20) sel exec -> v3=0, v2=(10), v1=(20) -> v3==0, so select and execute v1=(20)
        let seq1 = vec![Command::Number(10)];
        let seq2 = vec![Command::Number(20)];
        let commands = vec![
            Command::Number(0),      // v3 (condition)
            Command::Sequence(seq1), // v2 (true branch)
            Command::Sequence(seq2), // v1 (false branch)
            Command::Sel,
            Command::Exec,
        ];
        let mut postfix = PostFix::new(vec![], commands);
        assert_eq!(postfix.run().unwrap(), 20);
    }
}
