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
mod token;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(
    #[allow(clippy::all)]
    corepython,
    "/parser/corepython.rs"
);

pub use location::Location;

use crate::CompilationError;
use lexer::MyLexer;

pub fn parse_python(source: &str) -> Result<ast::Program, CompilationError> {
    let lexer = MyLexer::new(&source);

    corepython::ProgramParser::new()
        .parse(lexer)
        .map_err(|err| match err {
            lalrpop_util::ParseError::UnrecognizedEOF { location, expected } => {
                CompilationError::new(
                    &location,
                    &format!("Unexpected end of file, expected: {}", expected.join(", ")),
                )
            }
            lalrpop_util::ParseError::UnrecognizedToken { token, expected } => {
                CompilationError::new(
                    &token.0,
                    &format!("Got {:?}, expected {} ", token.1, expected.join(", ")),
                )
            }
            lalrpop_util::ParseError::InvalidToken { location } => {
                CompilationError::new(&location, "Invalid token.")
            }
            lalrpop_util::ParseError::ExtraToken { token } => {
                CompilationError::new(&token.0, &format!("Got an extra token {:?}", token.1))
            }
            lalrpop_util::ParseError::User { error } => {
                CompilationError::new(&error.location, &error.msg)
            }
        })
}

#[cfg(test)]
mod tests {
    use super::parse_python;

    #[test]
    fn test_parse_empty() {
        parse_python("").expect("Ok");
    }

    #[test]
    fn test_bad_indentation() {
        let source = r###"
def foo():
   pass
    return 2
        "###;
        let error = parse_python(source).expect_err("Indentation");
        assert_eq!(error.location.unwrap().row, 4);
    }
}
