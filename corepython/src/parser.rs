use super::ast;
use super::lexer::MyLexer;
use crate::corepython;

pub fn parse_python(source: &str) -> ast::Program {
    let lexer = MyLexer::new(&source);

    match corepython::ProgramParser::new().parse(lexer) {
        Ok(prog) => prog,
        Err(err) => {
            match err {
                lalrpop_util::ParseError::UnrecognizedEOF { location, expected } => {
                    error!(
                        "{}: Unexpected end of file, expected: {}",
                        location.get_text_for_user(),
                        expected.join(", ")
                    );
                }
                lalrpop_util::ParseError::UnrecognizedToken { token, expected } => {
                    error!(
                        "{}: Got {:?}, expected {} ",
                        token.0.get_text_for_user(),
                        token.1,
                        expected.join(", ")
                    );
                }
                lalrpop_util::ParseError::InvalidToken { location } => {
                    error!("{}: Invalid token.", location.get_text_for_user());
                }
                lalrpop_util::ParseError::ExtraToken { token } => {
                    error!("Got an extra token {:?}", token);
                }
                lalrpop_util::ParseError::User { error } => {
                    error!("{}: {}", error.location.get_text_for_user(), error.msg);
                }
            }
            panic!("Oh noes!");
        }
    }
}
