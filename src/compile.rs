use super::{ast, wasm};
use std::collections::HashMap;

pub fn compile_ast(prog: ast::Program) -> wasm::WasmModule {
    info!("Compiling AST");

    let c = Compiler::new();
    c.compile_prog(&prog)
}

struct Scope {
    variables: HashMap<String, usize>,
}

impl Scope {
    fn new() -> Self {
        Scope {
            variables: Default::default(),
        }
    }

    fn lookup(&self, name: &str) -> &usize {
        self.variables.get(name).unwrap()
    }

    fn register(&mut self, name: &str, index: usize) {
        self.variables.insert(name.to_owned(), index);
    }
}

struct Compiler {
    scopes: Vec<Scope>,
    code: Vec<wasm::Instruction>,
    module: wasm::WasmModule,
}

impl Compiler {
    fn new() -> Self {
        Compiler {
            scopes: vec![],
            code: vec![],
            module: wasm::WasmModule::new(),
        }
    }

    fn compile_prog(mut self, prog: &ast::Program) -> wasm::WasmModule {
        for function_def in &prog.functions {
            self.compile_function_def(function_def);
        }
        self.module
    }

    fn get_type(&self, typ: &ast::Expression) -> wasm::Type {
        match typ {
            ast::Expression::Identifier(name) => match name.as_str() {
                "float" => wasm::Type::F64,
                "int" => wasm::Type::I32,
                _ => unimplemented!(),
            },
            _ => {
                unimplemented!();
            }
        }
    }

    fn compile_function_def(&mut self, function_def: &ast::FunctionDef) {
        debug!("Compiling function {}", function_def.name);
        self.enter_scope();
        let mut params = vec![];
        for (index, parameter) in function_def.parameters.iter().enumerate() {
            self.get_scope_mut().register(&parameter.name, index);
            let param_type = self.get_type(&parameter.typ);
            params.push(param_type);
        }

        self.compile_suite(&function_def.body);

        // Implicit return 0:
        self.emit(wasm::Instruction::I32Const(0));
        self.leave_scope();

        // hmm, mem.replace??
        let code = std::mem::replace(&mut self.code, vec![]);

        self.module
            .add_function(function_def.name.clone(), params, code);
    }

    fn compile_suite(&mut self, suite: &[ast::Statement]) {
        for statement in suite {
            self.compile_statement(statement);
        }
    }

    fn compile_statement(&mut self, statement: &ast::Statement) {
        match statement {
            ast::Statement::Return(e) => {
                self.compile_expression(e);
                self.emit(wasm::Instruction::Return);
            }
            ast::Statement::If {
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
            ast::Statement::While { condition, suite } => {
                self.emit(wasm::Instruction::Block);
                self.emit(wasm::Instruction::Loop);
                self.compile_expression(condition);
                self.emit(wasm::Instruction::I32Eqz); // Invert condition, and branch if not good.
                self.emit(wasm::Instruction::BrIf(1));
                self.compile_suite(suite);
                self.emit(wasm::Instruction::End);
                self.emit(wasm::Instruction::End);
            }
            ast::Statement::For {
                target,
                iter,
                suite,
            } => {
                self.compile_suite(suite);
                unimplemented!();
            }
            ast::Statement::Break => {
                unimplemented!();
            }
            ast::Statement::Continue => {
                unimplemented!();
            }
        }
    }

    // fn compile_condition(&self, condition: &ast::Expression) {
    //     self.compile_expression
    //     unimplemented!();
    // }

    fn compile_expression(&mut self, expression: &ast::Expression) {
        match expression {
            ast::Expression::Number(value) => {
                self.emit(wasm::Instruction::I32Const(*value));
            }
            ast::Expression::Identifier(value) => {
                self.get_local(value);
            }
            ast::Expression::Comparison { a, op, b } => {
                self.compile_expression(a);
                self.compile_expression(b);
                match op {
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
                }
            }
            ast::Expression::BinaryOperation { a, op, b } => {
                self.compile_expression(a);
                self.compile_expression(b);
                match op {
                    ast::BinaryOperation::Add => {
                        self.emit(wasm::Instruction::I32Add);
                    }
                    ast::BinaryOperation::Sub => {
                        self.emit(wasm::Instruction::I32Sub);
                    }
                    ast::BinaryOperation::Mul => {
                        self.emit(wasm::Instruction::I32Mul);
                    }
                }
            }
        }
    }

    fn get_local(&mut self, name: &str) {
        let index = { *self.get_scope_mut().lookup(name) };
        self.emit(wasm::Instruction::LocalGet(index));
    }

    fn get_scope_mut(&mut self) -> &mut Scope {
        self.scopes.last_mut().unwrap()
    }

    fn enter_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    fn leave_scope(&mut self) {
        self.scopes.pop();
    }

    fn emit(&mut self, opcode: wasm::Instruction) {
        // info!("Emit: {:?}", opcode);
        self.code.push(opcode);
    }
}
