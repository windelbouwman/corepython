use super::Location;

pub struct Program {
    pub top_levels: Vec<TopLevel>,
}

pub enum TopLevel {
    Import { module: String, name: String },
    FunctionDef(FunctionDef),
    ClassDef(ClassDef),
}

pub struct FunctionDef {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub result: Option<Expression>,
    pub body: Suite,
}

pub struct ClassDef {
    pub name: String,
    pub methods: Vec<FunctionDef>,
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
    Assignment {
        target: String,
        value: Box<Expression>,
    },
    Expression(Expression),
    // AugmentAssignment {
    //     target: String,
    //     op: BinaryOperation,
    //     value: Box<Expression>,
    // },
    Break,
    Continue,
    Pass,
}

pub struct Expression {
    pub location: Location,
    pub kind: ExpressionType,
}

pub enum ExpressionType {
    Number(i32),
    Float(f64),
    Str(String),
    // Bool(bool),
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
    BoolOp {
        a: Box<Expression>,
        op: BooleanOperator,
        b: Box<Expression>,
    },
    Call {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
    },
    Indexed {
        base: Box<Expression>,
        index: Box<Expression>,
    },
}

#[derive(Clone)]
pub enum BooleanOperator {
    And,
    Or,
}

#[derive(Clone)]
pub enum BinaryOperation {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone)]
pub enum Comparison {
    Lt,
    Gt,
    Le,
    Ge,
    Equal,
    NotEqual,
}
