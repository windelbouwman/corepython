use super::ast;
use super::lexer::MyLexer;
use crate::corepython;

pub fn parse_python(filename: &str) -> ast::Program {
    info!("Reading {}", filename);

    let source = std::fs::read_to_string(filename).unwrap();
    // let toks = tokenize(&source);
    let lexer = MyLexer::new(&source);

    let res = corepython::ProgramParser::new().parse(lexer).unwrap();

    res
}
