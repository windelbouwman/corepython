//! A single function library, to turn opiniated python code into WebAssembly.
//!
//! Call the function python_to_wasm to generate some WebAssembly bytecodes.

#[macro_use]
extern crate log;

mod ast;
mod compile;
mod lexer;
mod parser;
mod token;
mod wasm;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub corepython);

use compile::compile_ast;
use parser::parse_python;
use wasm::write_wasm;

/// Single function
pub fn python_to_wasm<W>(source: &str, dest: &mut W)
where
    W: std::io::Write,
{
    let ast = parse_python(source);
    let wasm_module = compile_ast(ast);
    write_wasm(wasm_module, dest).unwrap();
}
