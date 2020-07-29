use super::lexer::MyLexer;
use super::{ast, corepython};
use crate::CompilationError;

pub fn parse_python(source: &str) -> Result<ast::Program, CompilationError> {
    let lexer = MyLexer::new(&source);

    corepython::ProgramParser::new()
        .parse(lexer)
        .map_err(|err| match err {
            lalrpop_util::ParseError::UnrecognizedEOF { location, expected } => CompilationError {
                location: Some(location),
                message: format!("Unexpected end of file, expected: {}", expected.join(", ")),
            },
            lalrpop_util::ParseError::UnrecognizedToken { token, expected } => CompilationError {
                location: Some(token.0),
                message: format!("Got {:?}, expected {} ", token.1, expected.join(", ")),
            },
            lalrpop_util::ParseError::InvalidToken { location } => CompilationError {
                location: Some(location),
                message: "Invalid token.".to_string(),
            },
            lalrpop_util::ParseError::ExtraToken { token } => CompilationError {
                location: Some(token.0),
                message: format!("Got an extra token {:?}", token.1),
            },
            lalrpop_util::ParseError::User { error } => CompilationError {
                location: Some(error.location),
                message: error.msg,
            },
        })
}
