use super::{parser::ast, wasm, CompilationError};
// use super::scope::{Scope, Symbol};
use super::analyze;

pub fn compile_ast(prog: ast::Program) -> Result<wasm::WasmModule, CompilationError> {
    info!("Compiling AST");

    let prog = analyze::analyze(prog)?;

    let c = Compiler::new();
    Ok(c.compile_prog(&prog))
}

/// Helper struct to compile a typed and resolved program to WebAssembly.
struct Compiler {
    code: Vec<wasm::Instruction>,
    module: wasm::WasmModule,
}

impl Compiler {
    fn new() -> Self {
        Compiler {
            code: vec![],
            module: wasm::WasmModule::new(),
        }
    }

    fn compile_prog(mut self, prog: &analyze::Program) -> wasm::WasmModule {
        for function in &prog.functions {
            self.compile_function(function);
        }
        self.module
    }

    fn get_type(&self, typ: &analyze::Type) -> wasm::Type {
        match typ {
            analyze::Type::BaseType(basetype) => match basetype {
                analyze::BaseType::Float => wasm::Type::F64,
                analyze::BaseType::Integer => wasm::Type::I32,
                analyze::BaseType::Bool => {
                    wasm::Type::I32
                    // unimplemented!("Ugh, what now?")
                }
            },
            // analyze::Type::Unknown => {
            //     panic!("Cannot compile partially typed program");
            //     // wasm::Type::I32
            // }
        }
    }

    fn compile_function(&mut self, function: &analyze::Function) {
        debug!("Compiling function {}", function.name);

        let mut params = vec![];
        for parameter in &function.parameters {
            match parameter.as_ref() {
                analyze::Symbol::Parameter {
                    name: _,
                    typ,
                    index: _,
                } => {
                    let param_type = self.get_type(typ);
                    params.push(param_type);
                }
                _ => {
                    panic!("Invalid parameter!");
                }
            }
        }

        let mut results = vec![];
        if let Some(t) = &function.return_type {
            results.push(self.get_type(t));
        }

        let mut locals: Vec<wasm::Type> = vec![];
        for local in &function.locals {
            match local.as_ref() {
                analyze::Symbol::Local { typ, index: _ } => {
                    let local_type = self.get_type(typ);
                    locals.push(local_type);
                }
                _ => {
                    panic!("Invalid local!");
                }
            }
        }

        self.compile_suite(&function.body);

        if let Some(t) = &function.return_type {
            match t {
                analyze::Type::BaseType(b) => {
                    match b {
                        analyze::BaseType::Integer | analyze::BaseType::Bool => {
                            // Implicit return 0:
                            self.emit(wasm::Instruction::I32Const(0));
                        }
                        analyze::BaseType::Float => {
                            // Implicit return 0:
                            self.emit(wasm::Instruction::F64Const(0.0));
                        }
                    }
                }
            }
        }

        // hmm, mem.replace??
        let code = std::mem::replace(&mut self.code, vec![]);

        self.module
            .add_function(function.name.clone(), params, results, locals, code);
    }

    fn compile_suite(&mut self, suite: &[analyze::Statement]) {
        for statement in suite {
            self.compile_statement(statement);
        }
    }

    fn compile_statement(&mut self, statement: &analyze::Statement) {
        match statement {
            analyze::Statement::Return { value } => {
                self.compile_expression(value);
                self.emit(wasm::Instruction::Return);
            }
            analyze::Statement::Expression(expr) => {
                self.compile_expression(expr);
                self.emit(wasm::Instruction::Drp);
            }
            analyze::Statement::If {
                condition,
                suite,
                else_suite,
            } => {
                self.compile_expression(condition);
                self.emit(wasm::Instruction::If);
                self.compile_suite(suite);
                self.emit(wasm::Instruction::Else);
                self.compile_suite(else_suite);
                self.emit(wasm::Instruction::End);
            }
            analyze::Statement::While { condition, suite } => {
                self.emit(wasm::Instruction::Block);
                self.emit(wasm::Instruction::Loop);
                self.compile_expression(condition);
                self.emit(wasm::Instruction::I32Eqz); // Invert condition, and branch if not good.
                self.emit(wasm::Instruction::BrIf(1));
                self.compile_suite(suite);
                self.emit(wasm::Instruction::End);
                self.emit(wasm::Instruction::End);
            }
            analyze::Statement::Assignment { target, value } => {
                self.compile_expression(value);
                let typ = value.get_type();
                self.store_value(target, typ);
            }
        }
    }

    fn compile_expression(&mut self, expression: &analyze::Expression) {
        match expression {
            analyze::Expression::Number(value) => {
                self.emit(wasm::Instruction::I32Const(*value));
            }
            analyze::Expression::Float(value) => {
                self.emit(wasm::Instruction::F64Const(*value));
            }
            analyze::Expression::Identifier(value) => {
                self.get_local(value);
            }
            analyze::Expression::BinaryOperation { a, op, b, typ: _ } => {
                self.compile_expression(a);
                self.compile_expression(b);
                let typ = self.get_type(a.get_type());
                match op {
                    analyze::BinaryOperation::ArithmaticOperation(op) => {
                        self.emit_arithmatic_operator(op, typ);
                    }
                    analyze::BinaryOperation::Comparison(op) => {
                        self.emit_comparison(op, typ);
                    }
                    analyze::BinaryOperation::Boolean(op) => match op {
                        ast::BooleanOperator::And => {
                            self.emit(wasm::Instruction::I32And);
                        }
                        ast::BooleanOperator::Or => {
                            self.emit(wasm::Instruction::I32Or);
                        }
                    },
                }
            }
            analyze::Expression::Call {
                callee,
                typ: _,
                arguments,
            } => {
                for argument in arguments {
                    self.compile_expression(argument);
                }

                let func: usize = match callee.as_ref() {
                    analyze::Symbol::Function { index, .. } => *index,
                    _ => {
                        panic!("Cannot call this!");
                    }
                };
                self.emit(wasm::Instruction::Call(func));
            }
        }
    }

    fn emit_comparison(&mut self, op: &ast::Comparison, typ: wasm::Type) {
        match typ {
            wasm::Type::I32 => match op {
                ast::Comparison::Lt => {
                    self.emit(wasm::Instruction::I32LtS);
                }
                ast::Comparison::Gt => {
                    self.emit(wasm::Instruction::I32GtS);
                }
                ast::Comparison::Le => {
                    self.emit(wasm::Instruction::I32LeS);
                }
                ast::Comparison::Ge => {
                    self.emit(wasm::Instruction::I32GeS);
                }
                ast::Comparison::Equal => {
                    self.emit(wasm::Instruction::I32Eq);
                }
                ast::Comparison::NotEqual => {
                    self.emit(wasm::Instruction::I32Ne);
                }
            },
            wasm::Type::F64 => match op {
                ast::Comparison::Lt => {
                    self.emit(wasm::Instruction::F64Lt);
                }
                ast::Comparison::Gt => {
                    self.emit(wasm::Instruction::F64Gt);
                }
                ast::Comparison::Le => {
                    self.emit(wasm::Instruction::F64Le);
                }
                ast::Comparison::Ge => {
                    self.emit(wasm::Instruction::F64Ge);
                }
                ast::Comparison::Equal | ast::Comparison::NotEqual => {
                    unimplemented!(
                        "Dubiously, equality and floating point are not always a good idea..."
                    );
                }
            },
        }
    }

    fn emit_arithmatic_operator(&mut self, op: &ast::BinaryOperation, typ: wasm::Type) {
        match typ {
            wasm::Type::I32 => match op {
                ast::BinaryOperation::Add => {
                    self.emit(wasm::Instruction::I32Add);
                }
                ast::BinaryOperation::Sub => {
                    self.emit(wasm::Instruction::I32Sub);
                }
                ast::BinaryOperation::Mul => {
                    self.emit(wasm::Instruction::I32Mul);
                }
                ast::BinaryOperation::Div => {
                    self.emit(wasm::Instruction::I32DivS);
                }
            },
            wasm::Type::F64 => match op {
                ast::BinaryOperation::Add => {
                    self.emit(wasm::Instruction::F64Add);
                }
                ast::BinaryOperation::Sub => {
                    self.emit(wasm::Instruction::F64Sub);
                }
                ast::BinaryOperation::Mul => {
                    self.emit(wasm::Instruction::F64Mul);
                }
                ast::BinaryOperation::Div => {
                    self.emit(wasm::Instruction::F64Div);
                }
            },
        }
    }

    fn store_value(&mut self, symbol: &analyze::Symbol, typ: &analyze::Type) {
        match symbol {
            analyze::Symbol::Local {
                typ: local_typ,
                index,
            }
            | analyze::Symbol::Parameter {
                name: _,
                typ: local_typ,
                index,
            } => {
                assert!(typ == local_typ);
                self.emit(wasm::Instruction::LocalSet(*index));
            }
            analyze::Symbol::Function { .. } => {
                panic!("Cannot store to function");
            }
        }
    }

    fn get_local(&mut self, symbol: &analyze::Symbol) {
        match symbol {
            analyze::Symbol::Local { typ: _, index }
            | analyze::Symbol::Parameter {
                name: _,
                typ: _,
                index,
            } => {
                self.emit(wasm::Instruction::LocalGet(*index));
            }
            analyze::Symbol::Function { .. } => {
                panic!("Cannot load from function");
            }
        }
    }

    fn emit(&mut self, opcode: wasm::Instruction) {
        // info!("Emit: {:?}", opcode);
        self.code.push(opcode);
    }
}
