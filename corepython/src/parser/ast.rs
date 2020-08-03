use super::Location;

#[derive(Debug)]
pub struct Program {
    pub top_levels: Vec<TopLevel>,
}

#[derive(Debug)]
pub enum TopLevel {
    Import { module: String, name: String },
    FunctionDef(FunctionDef),
    ClassDef(ClassDef),
}

#[derive(Debug)]
pub struct FunctionDef {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub result: Option<Expression>,
    pub body: Suite,
}

#[derive(Debug)]
pub struct ClassDef {
    pub name: String,
    pub methods: Vec<FunctionDef>,
}

type Suite = Vec<Statement>;

#[derive(Debug)]
pub struct Parameter {
    pub name: String,
    pub typ: Expression,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Expression {
    pub location: Location,
    pub kind: ExpressionType,
}

#[derive(Debug)]
pub enum ExpressionType {
    Number(i32),
    Float(f64),
    Str(String),
    // Bool(bool),
    Identifier(String),
    List {
        elements: Vec<Expression>,
    },
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

#[derive(Clone, Debug)]
pub enum BooleanOperator {
    And,
    Or,
}

#[derive(Clone, Debug)]
pub enum BinaryOperation {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, Debug)]
pub enum Comparison {
    Lt,
    Gt,
    Le,
    Ge,
    Equal,
    NotEqual,
}
