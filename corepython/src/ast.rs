pub struct Program {
    pub functions: Vec<FunctionDef>,
}

pub struct FunctionDef {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub result: Expression,
    pub body: Suite,
}

type Suite = Vec<Statement>;

pub struct Parameter {
    pub name: String,
    pub typ: Expression,
}

pub enum Statement {
    Return(Expression),
    If {
        condition: Box<Expression>,
        suite: Box<Suite>,
        else_suite: Box<Suite>,
    },
    While {
        condition: Box<Expression>,
        suite: Box<Suite>,
    },
    For {
        target: String,
        iter: Box<Expression>,
        suite: Box<Suite>,
    },
    Break,
    Continue,
}

pub enum Expression {
    Number(i32),
    Identifier(String),
    Comparison {
        a: Box<Expression>,
        op: Comparison,
        b: Box<Expression>,
    },
    BinaryOperation {
        a: Box<Expression>,
        op: BinaryOperation,
        b: Box<Expression>,
    },
}

pub enum BinaryOperation {
    Add,
    Sub,
    Mul,
}

pub enum Comparison {
    Lt,
    Gt,
    Le,
    Ge,
    Equal,
    NotEqual,
}
