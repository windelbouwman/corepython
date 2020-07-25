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

fn main() {
    simple_logger::init().unwrap();

    let filename = "demo.py";
    let ast = parse_python(filename);
    let wasm_module = compile_ast(ast);
    write_wasm(wasm_module, "demo.wasm").unwrap();
}
