//! Python sourcecode parsing.
//!
//! This submodule handles these stages:
//! - lexing
//! - parsing
//!
//! The final result is an AST (abstract syntax tree)

pub mod ast;
mod lexer;
mod location;
mod parser;
mod token;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(
    #[allow(clippy::all)]
    corepython,
    "/parser/corepython.rs"
);

pub use location::Location;
pub use parser::parse_python;
