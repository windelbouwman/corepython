pub struct Program {
    pub functions: Vec<FunctionDef>,
}

pub struct FunctionDef {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub result: Expression,
    pub body: Vec<Statement>,
}

pub struct Parameter {
    pub name: String,
    pub typ: Expression,
}

pub enum Statement {
    Return(Expression),
}

pub enum Expression {
    Number(i32),
    Identifier(String),
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
