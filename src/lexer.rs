use super::token::Token;
use logos::{Lexer, Logos};
use std::str::FromStr;

#[derive(Logos, Debug, Clone)]
pub enum LogosToken {
    #[token("def")]
    KeywordDef,

    #[token("return")]
    KeywordReturn,

    #[regex("[a-zA-Z]+", |l| l.slice().to_string())]
    Identifier(String),

    #[regex("[0-9]+", |l| i32::from_str(l.slice()))]
    Number(i32),

    #[token(":")]
    Colon,

    #[token("(")]
    OpeningParenthesis,

    #[token(")")]
    ClosingParenthesis,

    #[token(",")]
    Comma,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("->")]
    Arrow,

    #[token("\n")]
    NewLine,

    #[regex(" +", |l| l.slice().len())]
    WhiteSpace(usize),

    #[error]
    Error,
}

type Location = usize;
type Spanned<Tok, Error> = Result<(Location, Tok, Location), Error>;

pub struct MyLexer<'t> {
    inner: Lexer<'t, LogosToken>,
    pending: Vec<(usize, Token, usize)>,
    parenthesis_level: usize,
    spaces: usize,
    at_end: bool,
}

impl<'t> MyLexer<'t> {
    pub fn new(txt: &'t str) -> Self {
        let inner = LogosToken::lexer(txt);
        MyLexer {
            inner,
            pending: vec![],
            parenthesis_level: 0,
            spaces: 0,
            at_end: false,
        }
    }

    fn emit(&mut self, token: Token) {
        let span = self.inner.span();
        let spanned = (span.start, token, span.end);
        self.pending.push(spanned);
    }

    /// Process a single inner token.
    fn process(&mut self) -> Result<(), String> {
        let t = self.inner.next();
        if let Some(t) = t {
            // debug!("Logos Tok: {:?} --> '{}'", t, self.inner.slice());

            match t {
                LogosToken::Colon => self.emit(Token::Colon),
                LogosToken::Comma => self.emit(Token::Comma),
                LogosToken::KeywordDef => self.emit(Token::KeywordDef),
                LogosToken::KeywordReturn => self.emit(Token::KeywordReturn),
                LogosToken::Minus => self.emit(Token::Minus),
                LogosToken::Plus => self.emit(Token::Plus),
                LogosToken::Arrow => self.emit(Token::Arrow),
                LogosToken::OpeningParenthesis => {
                    self.parenthesis_level += 1;
                    self.emit(Token::OpeningParenthesis);
                }
                LogosToken::ClosingParenthesis => {
                    self.parenthesis_level -= 1;
                    self.emit(Token::ClosingParenthesis);
                }
                LogosToken::NewLine => {
                    // ?
                }
                LogosToken::WhiteSpace(w) => {
                    // ?
                    self.spaces += w;
                }
                LogosToken::Identifier(value) => self.emit(Token::Identifier { value }),
                LogosToken::Number(value) => self.emit(Token::Number { value }),
                LogosToken::Error => {
                    return Err("Lexical error!".to_owned());
                }
            };
        } else {
            self.at_end = true;
        }

        Ok(())
    }
}

impl<'t> std::iter::Iterator for MyLexer<'t> {
    type Item = Spanned<Token, String>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.pending.is_empty() && !self.at_end {
            if let Err(err) = self.process() {
                return Some(Err(err));
            }
        }

        if self.pending.is_empty() {
            None
        } else {
            Some(Ok(self.pending.remove(0)))
        }
    }
}

// pub fn tokenize(txt: &str) {
//     debug!("Tokenizing {}", txt);
//     let mut lex = Token::lexer(txt);

//     while let Some(t) = lex.next() {
//         info!("Tok: {:?} --> '{}'", t, lex.slice());

//         match t {
//             Token::Error => {
//                 error!("Ugh");
//             },
//             _ => {
//                 // ok
//             }
//         }
//     }
// }
