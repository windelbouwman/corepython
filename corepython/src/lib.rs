//! A single function library, to turn opiniated python code into WebAssembly.
//!
//! Call the function python_to_wasm to generate some WebAssembly bytecodes.

#[macro_use]
extern crate log;

mod analyze;
mod compile;
mod error;
mod parser;
mod wasm;

use compile::compile_ast;
use error::CompilationError;
use parser::parse_python;
use wasm::write_wasm;

/// Single exposed function.
///
/// This library function takes python-ish sourcecode and transforms it into WebAssembly.
pub fn python_to_wasm<W>(source: &str, dest: &mut W) -> Result<(), CompilationError>
where
    W: std::io::Write,
{
    let ast = parse_python(source)?;
    let wasm_module = compile_ast(ast)?;
    write_wasm(wasm_module, dest).map_err(|e| CompilationError {
        location: None,
        message: e.to_string(),
    })?;
    Ok(())
}
