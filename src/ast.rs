/// Represents a PostFix command that can be executed
#[derive(Debug, Clone)]
pub enum Command {
    Number(i32),
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Gt,
    Lt,
    Pop,
    Swap,
    Sel,
    Nget,
    Exec,
    Sequence(Vec<Command>),
}

/// Represents a value on the data stack
#[derive(Debug, Clone)]
pub enum Value {
    Number(i32),
    Sequence(Vec<Command>),
}
