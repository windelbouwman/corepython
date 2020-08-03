//! Check a parsed AST for symbols and types.

use std::collections::HashMap;
use std::rc::Rc;

use super::parser::{ast, Location};
use super::CompilationError;

/// Analyze the given AST and construct a type checked and
/// symbolically resolved program from it.
///
/// Analyze also checks for some simple optimizations,
/// such as ord('A') --> 65
pub fn analyze(prog: ast::Program) -> Result<Program, CompilationError> {
    let a = Analyzer::new();
    a.analyze_program(&prog)
}

pub enum Symbol {
    Parameter {
        parameter: Rc<Parameter>,
        index: usize,
    },

    Local {
        local: Rc<Local>,
        index: usize,
    },
    Function {
        function: Rc<Function>,
        index: usize,
    },
    ExternFunction {
        index: usize,
    },
    Builtin(Builtin),
    // TODO:
    // Type {
    //     typ: Type,
    // },
    // Unresolved,
}

pub enum Builtin {
    Ord,
    Len,
}

impl Symbol {
    fn get_type(&self) -> &Type {
        match self {
            Symbol::Parameter { parameter, .. } => &parameter.typ,
            Symbol::Local { local, .. } => &local.typ,
            Symbol::Builtin(..) => {
                unimplemented!("TODO!");
            }
            Symbol::Function { .. } => {
                unimplemented!("TODO!");
                // &function.as_ref().return_type.unwrap()
            }
            Symbol::ExternFunction { .. } => {
                unimplemented!();
            }
        }
    }
}

pub struct Parameter {
    pub name: String,
    pub typ: Type,
}

pub struct Local {
    pub name: String,
    pub typ: Type,
}

pub struct Program {
    pub functions: Vec<Rc<Function>>,
    pub imports: Vec<Import>,
}

pub struct Import {
    pub modname: String,
    pub name: String,
}

pub struct Function {
    pub name: String,
    pub parameters: Vec<Rc<Parameter>>,
    pub locals: Vec<Rc<Local>>,
    pub body: Suite,
    pub return_type: Option<Type>,
}

type Suite = Vec<Statement>;

pub enum Statement {
    Assignment {
        target: Rc<Symbol>,
        value: Expression,
    },
    Expression(Expression),
    If {
        condition: Expression,
        suite: Suite,
        else_suite: Suite,
    },
    While {
        condition: Expression,
        suite: Suite,
    },
    Return {
        value: Expression,
    },
}

pub enum Expression {
    Number(i32),
    Float(f64),
    String(String),
    Identifier(Rc<Symbol>),
    BinaryOperation {
        a: Box<Expression>,
        op: BinaryOperation,
        b: Box<Expression>,
        typ: Type,
    },
    Call {
        callee: Rc<Symbol>,
        arguments: Vec<Expression>,
        typ: Type,
    },
}

impl Expression {
    pub fn get_type(&self) -> &Type {
        match self {
            Expression::Number(_) => &Type::BaseType(BaseType::Integer),
            Expression::Float(_) => &Type::BaseType(BaseType::Float),
            Expression::String(_) => &Type::BaseType(BaseType::Str),
            Expression::Identifier(symbol) => symbol.get_type(),
            Expression::BinaryOperation { typ, .. } => typ,
            Expression::Call { typ, .. } => typ,
        }
    }
}

pub enum BinaryOperation {
    ArithmaticOperation(ast::BinaryOperation),
    Comparison(ast::Comparison),
    Boolean(ast::BooleanOperator),
}

#[derive(Clone, PartialEq)]
pub enum BaseType {
    Integer,
    Float,
    Bool,
    Str,
}

#[derive(Clone, PartialEq)]
pub enum Type {
    BaseType(BaseType),
    // TODO: user type / function type

    // We do not know the type yet.
    // Unknown,
    /// A list of certain types
    List(Box<Type>),
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::BaseType(base_type) => match base_type {
                BaseType::Float => write!(f, "float"),
                BaseType::Integer => write!(f, "int"),
                BaseType::Bool => write!(f, "bool"),
                BaseType::Str => write!(f, "str"),
            },
            Type::List(element) => write!(f, "list[{}]", element),
        }
    }
}

enum TypeConstructor {
    List,
}

pub struct Scope {
    pub variables: HashMap<String, Rc<Symbol>>,
    pub locals: Vec<Rc<Local>>,
    // parent: Option<Rc<Scope>>,
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            variables: Default::default(),
            locals: vec![],
            // parent: None,
        }
    }

    pub fn contains(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    pub fn lookup(&self, name: &str) -> &Rc<Symbol> {
        self.variables.get(name).unwrap()
    }

    pub fn register(&mut self, name: &str, symbol: Rc<Symbol>) {
        self.variables.insert(name.to_owned(), symbol);
    }
}

struct Analyzer {
    scopes: Vec<Scope>,
}

impl Analyzer {
    fn new() -> Self {
        Analyzer { scopes: vec![] }
    }

    fn analyze_program(mut self, prog: &ast::Program) -> Result<Program, CompilationError> {
        self.define_intrinsics();
        self.enter_scope();
        let mut imports = vec![];
        let mut functions = vec![];
        for top_level in &prog.top_levels {
            match top_level {
                ast::TopLevel::FunctionDef(function_def) => {
                    let function = self.analyze_function_def(function_def)?;
                    let function = Rc::new(function);
                    let index = functions.len();
                    functions.push(function.clone());
                    let symbol = Symbol::Function { function, index };
                    self.define(&function_def.name, Rc::new(symbol));
                }
                ast::TopLevel::Import { module, name } => {
                    // arg
                    // TODO!
                    info!("Importing {}.{}", module, name);
                    let index = imports.len();
                    imports.push(Import {
                        modname: module.clone(),
                        name: name.clone(),
                    });
                    let symbol = Symbol::ExternFunction { index };
                    self.define(name, Rc::new(symbol));
                }
                ast::TopLevel::ClassDef { .. } => {
                    unimplemented!("TODO");
                }
            }
        }
        self.leave_scope();
        Ok(Program { imports, functions })
    }

    /// Define internal functions such as len and ord.
    fn define_intrinsics(&mut self) {
        self.enter_scope();
        let symbol = Symbol::Builtin(Builtin::Ord);
        self.define("ord", Rc::new(symbol));

        let symbol = Symbol::Builtin(Builtin::Len);
        self.define("len", Rc::new(symbol));
    }

    /// Determine the type given a parsed expression
    fn get_type(&self, typ: &ast::Expression) -> Result<Type, CompilationError> {
        match &typ.kind {
            ast::ExpressionType::Identifier(name) => match name.as_str() {
                "float" => Ok(Type::BaseType(BaseType::Float)),
                "int" => Ok(Type::BaseType(BaseType::Integer)),
                name => Err(new_error(
                    typ,
                    &format!("Invalid type identifier: {}", name),
                )),
            },
            ast::ExpressionType::Indexed { base, index } => {
                let base = self.get_type_constructor(base)?;
                let index = self.get_type(index)?;
                let typ = self.apply(base, index);
                Ok(typ)
            }
            _ => Err(new_error(typ, "Invalid type expression")),
        }
    }

    fn get_type_constructor(
        &self,
        con: &ast::Expression,
    ) -> Result<TypeConstructor, CompilationError> {
        match &con.kind {
            ast::ExpressionType::Identifier(name) => match name.as_str() {
                "list" => Ok(TypeConstructor::List),
                name => Err(new_error(
                    con,
                    &format!("No such type constructor {}", name),
                )),
            },
            _ => Err(new_error(con, "Invalid type constructor")),
        }
    }

    fn apply(&self, con: TypeConstructor, arg: Type) -> Type {
        match con {
            TypeConstructor::List => Type::List(Box::new(arg)),
        }
    }

    fn analyze_function_def(
        &mut self,
        function_def: &ast::FunctionDef,
    ) -> Result<Function, CompilationError> {
        debug!("Analyzing function {}", function_def.name);
        self.enter_scope();
        let mut parameters = vec![];
        for (index, parameter) in function_def.parameters.iter().enumerate() {
            let param_type = self.get_type(&parameter.typ)?;
            let param = Rc::new(Parameter {
                name: parameter.name.clone(),
                typ: param_type,
            });
            let symbol = Rc::new(Symbol::Parameter {
                parameter: param.clone(),
                index,
            });
            self.define(&parameter.name, symbol.clone());
            parameters.push(param);
        }

        let return_type = match &function_def.result {
            Some(e) => Some(self.get_type(e)?),
            None => None,
        };
        let body = self.analyze_suite(&function_def.body)?;

        let locals = self.leave_scope().locals;
        Ok(Function {
            name: function_def.name.clone(),
            parameters,
            locals,
            body,
            return_type,
        })
    }

    fn analyze_suite(&mut self, suite: &[ast::Statement]) -> Result<Suite, CompilationError> {
        let mut statements: Suite = vec![];
        for statement in suite {
            let statement = self.analyze_statement(statement)?;
            statements.push(statement);
        }
        Ok(statements)
    }

    fn analyze_statement(
        &mut self,
        statement: &ast::Statement,
    ) -> Result<Statement, CompilationError> {
        match statement {
            ast::Statement::Return(e) => {
                let value = self.analyze_expression(e)?;
                // TODO: Type check type with return type of function
                Ok(Statement::Return { value })
            }
            ast::Statement::If {
                condition,
                suite,
                else_suite,
            } => {
                let condition = self.analyze_expression(condition)?;
                let body = self.analyze_suite(suite)?;
                let else_suite = self.analyze_suite(else_suite)?;

                Ok(Statement::If {
                    condition,
                    suite: body,
                    else_suite,
                })
            }
            ast::Statement::While { condition, suite } => {
                let condition = self.analyze_expression(condition)?;
                let suite = self.analyze_suite(suite)?;

                Ok(Statement::While { condition, suite })
            }
            ast::Statement::For {
                target: _,
                iter: _,
                suite: _,
            } => {
                // self.compile_suite(suite)?;
                unimplemented!();
            }
            ast::Statement::Pass => {
                unimplemented!();
            }
            ast::Statement::Break => {
                unimplemented!();
            }
            ast::Statement::Continue => {
                unimplemented!();
            }
            ast::Statement::Expression(expr) => {
                let expr = self.analyze_expression(expr)?;
                Ok(Statement::Expression(expr))
            }
            ast::Statement::Assignment { target, value } => {
                let value = self.analyze_expression(value)?;
                let typ = value.get_type();
                // TODO: derive type!
                let target = self.store_value(target, typ);
                Ok(Statement::Assignment { target, value })
            } // ast::Statement::AugmentAssignment { .. } => {
              // self.get_local(target);
              // self.compile_expression(value);
              // // TODO: coerce!
              // self.emit_operator(op);

              // let typ = wasm::Type::I32; // TODO!
              // self.store_value(target, typ);
              // unimplemented!();
              // }
        }
    }

    // fn compile_condition(&self, condition: &ast::Expression) {
    //     self.compile_expression
    //     unimplemented!();
    // }

    fn analyze_expression(
        &mut self,
        expression: &ast::Expression,
    ) -> Result<Expression, CompilationError> {
        match &expression.kind {
            ast::ExpressionType::Number(value) => Ok(Expression::Number(*value)),
            ast::ExpressionType::Float(value) => Ok(Expression::Float(*value)),
            // ast::Expression::Bool(_) => {
            //     unimplemented!();
            // }
            ast::ExpressionType::Str(value) => {
                // unimplemented!("STR: {}", value);

                // println!("Str: {} {}", value, value.len());
                // TODO: how to represent strings?
                Ok(Expression::String(value.clone()))
            }
            ast::ExpressionType::Identifier(value) => {
                let symbol = self.get_local(value);
                Ok(Expression::Identifier(symbol))
            }
            ast::ExpressionType::Comparison { a, op, b } => {
                let a = self.analyze_expression(a)?;
                let b = self.analyze_expression(b)?;
                if a.get_type() != b.get_type() {
                    return Err(new_error(expression, "Type mismatch"));
                }
                let typ = Type::BaseType(BaseType::Bool);
                Ok(Expression::BinaryOperation {
                    a: Box::new(a),
                    op: BinaryOperation::Comparison(op.clone()),
                    b: Box::new(b),
                    typ,
                })
            }
            ast::ExpressionType::BinaryOperation { a, op, b } => {
                let a = self.analyze_expression(a)?;
                let b = self.analyze_expression(b)?;

                if a.get_type() != b.get_type() {
                    return Err(new_error(expression, "Type mismatch"));
                }

                let typ = a.get_type().clone();
                // TODO: type checking!
                Ok(Expression::BinaryOperation {
                    a: Box::new(a),
                    op: BinaryOperation::ArithmaticOperation(op.clone()),
                    b: Box::new(b),
                    typ,
                })
            }
            ast::ExpressionType::BoolOp { a, op, b } => {
                let a = self.analyze_expression(a)?;
                let b = self.analyze_expression(b)?;

                Ok(Expression::BinaryOperation {
                    a: Box::new(a),
                    op: BinaryOperation::Boolean(op.clone()),
                    b: Box::new(b),
                    typ: Type::BaseType(BaseType::Bool),
                })
            }
            ast::ExpressionType::Call { callee, arguments } => {
                let mut args = vec![];
                for argument in arguments {
                    let arg = self.analyze_expression(argument)?;
                    args.push(arg);
                }

                match &callee.kind {
                    ast::ExpressionType::Identifier(name) => {
                        // arg
                        // return Err(Self::new_error(callee, "TODO".to_owned()));
                        if let Some(callee) = self.lookup(name) {
                            // callee

                            match callee.as_ref() {
                                Symbol::Function { function, index: _ } => {
                                    // Check args now!!!
                                    let expected_types =
                                        function.parameters.iter().map(|p| p.typ.clone()).collect();
                                    self.check_arguments(
                                        &expression.location,
                                        &args,
                                        expected_types,
                                    )?;

                                    // Arg: TODO: determine type!
                                    let typ = Type::BaseType(BaseType::Integer);

                                    Ok(Expression::Call {
                                        callee,
                                        arguments: args,
                                        typ,
                                    })
                                }
                                Symbol::ExternFunction { index: _ } => {
                                    // TODO: Check args now!!!

                                    // Arg: TODO: determine type!
                                    let typ = Type::BaseType(BaseType::Integer);

                                    Ok(Expression::Call {
                                        callee,
                                        arguments: args,
                                        typ,
                                    })
                                }
                                Symbol::Builtin(builtin) => {
                                    self.analyze_builtin_call(&expression.location, builtin, args)
                                }
                                Symbol::Local { .. } => {
                                    Err(new_error(expression, "Cannot call local variable"))
                                }
                                Symbol::Parameter { .. } => {
                                    Err(new_error(expression, "Cannot call parameter"))
                                }
                            }
                        } else {
                            Err(new_error(callee, &format!("Undefined: {}", name)))
                        }
                    }
                    _ => Err(new_error(callee, "Cannot call")),
                }
            }
            ast::ExpressionType::Indexed { .. } => {
                unimplemented!();
            }
        }
    }

    fn analyze_builtin_call(
        &self,
        location: &Location,
        builtin: &Builtin,
        args: Vec<Expression>,
    ) -> Result<Expression, CompilationError> {
        match builtin {
            Builtin::Len => {
                unimplemented!();
            }
            Builtin::Ord => {
                self.check_arguments(location, &args, vec![Type::BaseType(BaseType::Str)])?;
                let arg = &args[0];

                match arg {
                    Expression::String(value) => {
                        // For now treat chars as strings of len = 1
                        if value.len() == 1 {
                            let value: i32 = value.chars().next().unwrap() as i32;

                            Ok(Expression::Number(value))
                        } else {
                            Err(CompilationError::new(
                                location,
                                "String passed to ord must be a single character",
                            ))
                        }
                    }
                    _ => {
                        unimplemented!();
                    }
                }
            }
        }
    }

    fn check_arguments(
        &self,
        location: &Location,
        actual_args: &Vec<Expression>,
        expected_types: Vec<Type>,
    ) -> Result<(), CompilationError> {
        if actual_args.len() != expected_types.len() {
            return Err(CompilationError::new(
                location,
                &format!(
                    "Expected {} arguments, but got {}",
                    expected_types.len(),
                    actual_args.len()
                ),
            ));
        }

        for (arg, typ) in actual_args.iter().zip(expected_types.iter()) {
            let arg_typ = arg.get_type();
            if arg_typ != typ {
                return Err(CompilationError::new(
                    location,
                    &format!("Expected {} but got {}", typ, arg_typ),
                ));
            }
        }

        // return Err(new_error(act, message: String))
        Ok(())
    }

    fn new_local(&mut self, name: &str, typ: Type) {
        let index = self.get_scope_mut().variables.len();
        let local = Rc::new(Local {
            name: name.to_string(),
            typ,
        });
        let symbol = Rc::new(Symbol::Local {
            local: local.clone(),
            index,
        });
        self.get_scope_mut().locals.push(local);
        self.define(name, symbol);
    }

    fn define(&mut self, name: &str, symbol: Rc<Symbol>) {
        self.get_scope_mut().register(name, symbol);
    }

    fn is_defined(&self, name: &str) -> bool {
        self.scopes.last().unwrap().contains(name)
    }

    fn lookup(&self, name: &str) -> Option<Rc<Symbol>> {
        for scope in self.scopes.iter().rev() {
            if scope.contains(name) {
                // println!("Got symbol!");
                return Some(scope.lookup(name).clone());
            } else {
                // println!("Looking further!");
            }
        }
        None
    }

    fn store_value(&mut self, name: &str, typ: &Type) -> Rc<Symbol> {
        if !self.is_defined(name) {
            self.new_local(name, typ.clone());
            // TODO: type deduction?
            // self.locals.push(typ.clone());
        }

        self.get_local(name)
    }

    fn get_local(&self, name: &str) -> Rc<Symbol> {
        self.scopes.last().unwrap().lookup(name).clone()
    }

    fn get_scope_mut(&mut self) -> &mut Scope {
        self.scopes.last_mut().unwrap()
    }

    fn enter_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    fn leave_scope(&mut self) -> Scope {
        self.scopes.pop().unwrap()
    }
}

fn new_error(expression: &ast::Expression, message: &str) -> CompilationError {
    CompilationError::new(&expression.location, message)
}
