use super::token::Token;
use logos::{Lexer, Logos};
use std::str::FromStr;

#[derive(Logos, Debug, Clone)]
pub enum LogosToken {
    #[token("def")]
    KeywordDef,

    #[token("else")]
    KeywordElse,

    #[token("return")]
    KeywordReturn,

    #[token("if")]
    KeywordIf,

    #[token("in")]
    KeywordIn,

    #[token("for")]
    KeywordFor,

    #[token("while")]
    KeywordWhile,

    #[token("break")]
    KeywordBreak,

    #[token("continue")]
    KeywordContinue,

    #[token("class")]
    KeywordClass,

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

    #[token("*")]
    Asterix,

    #[token("<")]
    Less,

    #[token(">")]
    Greater,

    #[token(">=")]
    GreaterEqual,

    #[token("<=")]
    LessEqual,

    #[token("==")]
    EqualEqual,

    #[token("!=")]
    NotEqual,

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
    indentations: Vec<usize>,
    at_end: bool,
    at_bol: bool, // begin of line
}

impl<'t> MyLexer<'t> {
    pub fn new(txt: &'t str) -> Self {
        let inner = LogosToken::lexer(txt);
        MyLexer {
            inner,
            pending: vec![],
            parenthesis_level: 0,
            spaces: 0,
            indentations: vec![0],
            at_end: false,
            at_bol: true,
        }
    }

    fn emit(&mut self, token: Token) {
        if self.at_bol && self.parenthesis_level == 0 {
            self.at_bol = false;

            let new_indentation: usize = self.spaces;
            self.update_indentation(new_indentation);
        }

        // Emit current span:
        let span = self.inner.span();
        let spanned = (span.start, token, span.end);
        self.pending.push(spanned);
    }

    /// Do indent / dedent book keepings
    fn update_indentation(&mut self, new_indentation: usize) {
        if new_indentation > self.get_current_indentation() {
            self.indent(new_indentation);
        } else if new_indentation < self.get_current_indentation() {
            while new_indentation < self.get_current_indentation() {
                self.dedent();
            }

            if new_indentation != self.get_current_indentation() {
                panic!("Indentation error.");
            }
        }
    }

    fn get_current_indentation(&self) -> usize {
        *self.indentations.last().unwrap()
    }

    /// Process a single inner token.
    fn process(&mut self) -> Result<(), String> {
        let t = self.inner.next();
        if let Some(t) = t {
            // debug!("Logos Tok: {:?} --> '{}'", t, self.inner.slice());

            match t {
                LogosToken::Colon => self.emit(Token::Colon),
                LogosToken::Comma => self.emit(Token::Comma),
                LogosToken::KeywordBreak => self.emit(Token::KeywordBreak),
                LogosToken::KeywordClass => self.emit(Token::KeywordClass),
                LogosToken::KeywordContinue => self.emit(Token::KeywordContinue),
                LogosToken::KeywordDef => self.emit(Token::KeywordDef),
                LogosToken::KeywordElse => self.emit(Token::KeywordElse),
                LogosToken::KeywordFor => self.emit(Token::KeywordFor),
                LogosToken::KeywordIf => self.emit(Token::KeywordIf),
                LogosToken::KeywordIn => self.emit(Token::KeywordIn),
                LogosToken::KeywordReturn => self.emit(Token::KeywordReturn),
                LogosToken::KeywordWhile => self.emit(Token::KeywordWhile),
                LogosToken::Minus => self.emit(Token::Minus),
                LogosToken::Plus => self.emit(Token::Plus),
                LogosToken::Arrow => self.emit(Token::Arrow),
                LogosToken::Asterix => self.emit(Token::Asterix),
                LogosToken::Less => self.emit(Token::Less),
                LogosToken::Greater => self.emit(Token::Greater),
                LogosToken::LessEqual => self.emit(Token::LessEqual),
                LogosToken::GreaterEqual => self.emit(Token::GreaterEqual),
                LogosToken::EqualEqual => self.emit(Token::EqualEqual),
                LogosToken::NotEqual => self.emit(Token::NotEqual),
                LogosToken::OpeningParenthesis => {
                    self.parenthesis_level += 1;
                    self.emit(Token::OpeningParenthesis);
                }
                LogosToken::ClosingParenthesis => {
                    self.parenthesis_level -= 1;
                    self.emit(Token::ClosingParenthesis);
                }
                LogosToken::NewLine => {
                    self.at_bol = true;
                    self.spaces = 0;
                }
                LogosToken::WhiteSpace(w) => {
                    if self.at_bol {
                        self.spaces += w;
                    }
                }
                LogosToken::Identifier(value) => self.emit(Token::Identifier { value }),
                LogosToken::Number(value) => self.emit(Token::Number { value }),
                LogosToken::Error => {
                    return Err("Lexical error!".to_owned());
                }
            };
        } else {
            self.at_end = true;

            // Flush indentations:
            while self.indentations.len() > 1 {
                self.dedent();
            }
        }

        Ok(())
    }

    fn indent(&mut self, new_indentation: usize) {
        self.indentations.push(new_indentation);
        let spanned = (0, Token::Indent, 0);
        self.pending.push(spanned);
    }

    fn dedent(&mut self) {
        let spanned = (0, Token::Dedent, 0);
        self.pending.push(spanned);
        self.indentations.pop();
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
            let tok = self.pending.remove(0);
            // debug!("TOK: {:?}", tok);
            Some(Ok(tok))
        }
    }
}
