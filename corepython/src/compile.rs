use super::analyze;
use super::{parser::ast, wasm, CompilationError};

pub fn compile_ast(prog: ast::Program) -> Result<wasm::WasmModule, CompilationError> {
    info!("Compiling AST");

    let prog = analyze::analyze(prog)?;

    let c = Compiler::new();
    Ok(c.compile_prog(&prog))
}

/// Helper struct to compile a typed and resolved program to WebAssembly.
struct Compiler {
    code: Vec<wasm::Instruction>,
    func_offset: usize,
    module: wasm::WasmModule,
}

impl Compiler {
    fn new() -> Self {
        Compiler {
            code: vec![],
            func_offset: 0,
            module: wasm::WasmModule::new(),
        }
    }

    fn compile_prog(mut self, prog: &analyze::Program) -> wasm::WasmModule {
        for import in &prog.imports {
            let params: Vec<wasm::Type> = import
                .parameter_types
                .iter()
                .map(|t| self.get_type(t))
                .collect();
            let results: Vec<wasm::Type> = if let Some(t) = &import.return_type {
                vec![self.get_type(t)]
            } else {
                vec![]
            };
            self.module
                .add_import(&import.modname, &import.name, params, results);
        }
        self.func_offset += prog.imports.len();

        for function in &prog.functions {
            self.compile_function(function);
        }
        self.module
    }

    fn get_type(&self, typ: &analyze::Type) -> wasm::Type {
        match typ {
            analyze::Type::Float => wasm::Type::F64,
            analyze::Type::Integer => wasm::Type::I32,
            analyze::Type::Str | analyze::Type::Bytes => {
                // unimplemented!();
                // Assume pointer to bytes or string object in memory
                wasm::Type::I32
            }
            analyze::Type::Bool => {
                wasm::Type::I32
                // unimplemented!("Ugh, what now?")
            }
            analyze::Type::List(_) | analyze::Type::Tuple(_) => {
                // Assume pointer to some data structure in wasm memory.
                wasm::Type::I32
                // unimplemented!("TODO: lists");
            } // analyze::Type::Unknown => {
              //     panic!("Cannot compile partially typed program");
              //     // wasm::Type::I32
              // }
        }
    }

    fn compile_function(&mut self, function: &analyze::Function) {
        debug!("Compiling function {}", function.name);

        let mut params = vec![];
        for parameter in &function.parameters {
            let param_type = self.get_type(&parameter.typ);
            params.push(param_type);
        }

        let mut results = vec![];
        if let Some(t) = &function.return_type {
            results.push(self.get_type(t));
        }

        let mut locals: Vec<wasm::Type> = vec![];
        for local in &function.locals {
            let local_type = self.get_type(&local.typ);
            locals.push(local_type);
        }

        self.compile_suite(&function.body);

        if let Some(t) = &function.return_type {
            match t {
                analyze::Type::Integer | analyze::Type::Bool => {
                    // Implicit return 0:
                    self.emit(wasm::Instruction::I32Const(0));
                }
                analyze::Type::Float => {
                    // Implicit return 0:
                    self.emit(wasm::Instruction::F64Const(0.0));
                }
                analyze::Type::Str | analyze::Type::Bytes => {
                    unimplemented!();
                }
                analyze::Type::List(_) | analyze::Type::Tuple(_) => {
                    unimplemented!("TODO!");
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
                self.emit(wasm::Instruction::Br(0));
                self.emit(wasm::Instruction::End);
                self.emit(wasm::Instruction::End);
            }
            analyze::Statement::For {
                loop_var, // counts from 0 to len
                iter_var, // points to iterable (list)
                target,
                iter,
                suite,
            } => {
                let int_type = analyze::Type::Integer;

                info!("Loop var {:?}", loop_var);
                info!("Iter var {:?}", iter_var);

                // Evaluate iterator:
                self.compile_expression(iter);

                // Store iterator var:
                self.store_value(iter_var, &int_type);

                // Init loop var:
                self.emit(wasm::Instruction::I32Const(0));
                self.store_value(loop_var, &int_type);

                self.emit(wasm::Instruction::Loop);

                // Load current element from iter var:
                match iter.get_type() {
                    analyze::Type::List(element_type) => {
                        self.get_local(iter_var);
                        self.get_local(loop_var);
                        self.builtin_list_index(&element_type);
                        self.store_value(target, &element_type);
                    }
                    _ => {
                        unimplemented!();
                    }
                }

                // Execute body:
                self.compile_suite(suite);

                // i++ (update loop variable)
                // self.emit(wasm::Instruction::LocalGet(loop_var_index));
                self.get_local(loop_var);
                self.emit(wasm::Instruction::I32Const(1));
                self.emit(wasm::Instruction::I32Add);

                // self.emit(wasm::Instruction::LocalTee(loop_var));
                self.store_value(loop_var, &int_type);
                self.get_local(loop_var);

                // Get length
                self.get_local(iter_var);
                self.builtin_list_len();

                // Are we done?
                self.emit(wasm::Instruction::I32LtS);
                self.emit(wasm::Instruction::BrIf(0));

                self.emit(wasm::Instruction::End);
            }
            analyze::Statement::Assignment { target, value } => {
                self.compile_expression(value);
                let typ = value.get_type();
                self.store_value(target, typ);
            }
        }
    }

    /// Given a list as top of stack, retrieve its length.
    fn builtin_list_len(&mut self) {
        self.emit(wasm::Instruction::I32Load(2, 0));
    }

    /// Given a list and an index as top of stack, index the list
    /// List element is at top of stack.
    fn builtin_list_index(&mut self, element_type: &analyze::Type) {
        let element_size = self.get_sizeof(element_type);
        let element_wasm_typ = self.get_type(element_type);
        let header_size = 4; // i32 for length of list
        let data_start = round_to_multiple_of(header_size, element_size);

        self.emit(wasm::Instruction::I32Const(element_size as i32)); // sizeof int32 .....
        self.emit(wasm::Instruction::I32Mul);
        self.emit(wasm::Instruction::I32Add);
        self.read_mem(data_start, &element_wasm_typ);
    }

    fn get_sizeof(&self, element_type: &analyze::Type) -> usize {
        match element_type {
            analyze::Type::Integer => 4,
            analyze::Type::Float => 8,
            analyze::Type::List(_) => 4,
            _ => {
                unimplemented!();
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
            analyze::Expression::String(_) => {
                unimplemented!("TODO");
            }
            analyze::Expression::List {
                elements,
                typ: _,
                helper_local,
            } => {
                // Hmm, okay, list. Now what.
                // Plan:
                // Layout list literal in memory. Grab some memory, store all
                // elements sequentially.
                // Another idea: evaluate all elements and finally push the length
                // on the stack. Popping the elements will be in reversed order..
                let int_type = analyze::Type::Integer;

                // TODO: arbitrary size of list:
                assert!(!elements.is_empty());

                let element_typ = elements.first().expect("At least 1 element").get_type();
                let element_size = self.get_sizeof(element_typ);
                let element_wasm_type = self.get_type(element_typ);

                let header_size = 4; // i32 for length of list
                let data_start = round_to_multiple_of(header_size, element_size);

                // TODO: determine size of element:
                let mem_size = element_size * elements.len() + data_start;

                // Grab memory:
                self.allocate(mem_size);
                self.store_value(helper_local, &int_type);

                // Write len in header:
                self.get_local(helper_local);
                self.emit(wasm::Instruction::I32Const(elements.len() as i32));
                self.write_mem(0, &wasm::Type::I32);

                // Store:
                let mut offset = data_start;
                for element in elements {
                    self.get_local(helper_local);
                    self.compile_expression(element);
                    self.write_mem(offset, &element_wasm_type);
                    offset += element_size;
                }

                self.get_local(helper_local);
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

                match callee.as_ref() {
                    analyze::Symbol::Function { index, .. } => {
                        let func = *index + self.func_offset;
                        self.emit(wasm::Instruction::Call(func));
                    }
                    analyze::Symbol::ExternFunction { index, .. } => {
                        let func = *index;
                        self.emit(wasm::Instruction::Call(func));
                    }
                    analyze::Symbol::Builtin(builtin) => match builtin {
                        analyze::Builtin::Ord => {
                            unimplemented!();
                        }
                        analyze::Builtin::Len => {
                            self.builtin_list_len();
                        }
                    },
                    _ => {
                        panic!("Cannot call this!");
                    }
                };
            }
            analyze::Expression::Indexed { base, index, typ } => {
                self.compile_expression(base);
                self.compile_expression(index);
                self.builtin_list_index(typ);
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
            analyze::Symbol::Local { local: _, index } => {
                // assert!(typ == &local.typ);
                self.emit(wasm::Instruction::LocalSet(*index));
            }
            analyze::Symbol::Parameter { parameter, index } => {
                assert!(typ == &parameter.typ);
                self.emit(wasm::Instruction::LocalSet(*index));
            }
            analyze::Symbol::Function { .. }
            | analyze::Symbol::ExternFunction { .. }
            | analyze::Symbol::Builtin(..) => {
                panic!("Cannot store to this");
            }
        }
    }

    fn get_local(&mut self, symbol: &analyze::Symbol) {
        match symbol {
            analyze::Symbol::Local { local: _, index } => {
                self.emit(wasm::Instruction::LocalGet(*index));
            }
            analyze::Symbol::Parameter {
                parameter: _,
                index,
            } => {
                self.emit(wasm::Instruction::LocalGet(*index));
            }
            analyze::Symbol::Function { .. }
            | analyze::Symbol::ExternFunction { .. }
            | analyze::Symbol::Builtin(..) => {
                panic!("Cannot load from this");
            }
        }
    }

    /// Emit code to allocate some memory, and leave pointer on stack.
    fn allocate(&mut self, amount: usize) {
        debug!("Allocating {} bytes", amount);
        let amount = round_to_multiple_of(amount, 8);

        // First simple inline implementation of malloc which only increments memory:

        // Current value on stack:
        self.emit(wasm::Instruction::I32Const(0));
        self.emit(wasm::Instruction::I32Load(2, 0));

        // Increment free memory base:
        self.emit(wasm::Instruction::I32Const(0));
        self.emit(wasm::Instruction::I32Const(0));
        self.emit(wasm::Instruction::I32Load(2, 0));
        self.emit(wasm::Instruction::I32Const(amount as i32));
        self.emit(wasm::Instruction::I32Add);
        self.emit(wasm::Instruction::I32Store(2, 0));

        // Add header size (aligned to 8 bytes):
        let header_size = 4;
        self.emit(wasm::Instruction::I32Const(
            round_to_multiple_of(header_size, 8) as i32,
        ));
        self.emit(wasm::Instruction::I32Add);
    }

    /// Write top of stack (TOS) to memory at TOS[-1] + offset
    fn write_mem(&mut self, offset: usize, typ: &wasm::Type) {
        match typ {
            wasm::Type::F64 => {
                self.emit(wasm::Instruction::F64Store(3, offset));
            }
            wasm::Type::I32 => {
                self.emit(wasm::Instruction::I32Store(2, offset));
            }
        }
    }

    fn read_mem(&mut self, offset: usize, typ: &wasm::Type) {
        match typ {
            wasm::Type::F64 => {
                self.emit(wasm::Instruction::F64Load(3, offset));
            }
            wasm::Type::I32 => {
                self.emit(wasm::Instruction::I32Load(2, offset));
            }
        }
    }

    fn emit(&mut self, opcode: wasm::Instruction) {
        // info!("Emit: {:?}", opcode);
        self.code.push(opcode);
    }
}

fn round_to_multiple_of(value: usize, alignment: usize) -> usize {
    let remaining = value % alignment;
    if remaining > 0 {
        value + alignment - remaining
    } else {
        value
    }
}

#[cfg(test)]
mod tests {

    use super::round_to_multiple_of;

    #[test]
    fn test_round_to_multiple_of() {
        assert_eq!(16, round_to_multiple_of(9, 8));
        assert_eq!(8, round_to_multiple_of(8, 4));
        assert_eq!(21, round_to_multiple_of(20, 7));
        assert_eq!(0, round_to_multiple_of(0, 3));
    }
}
