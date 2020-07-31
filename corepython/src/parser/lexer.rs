//! Python sourcecode lexer.
//!
//! This file processes python source text and produces tokens.
//!
//! Strategy is to use a lexer generator, and then post process
//! those tokens into our own tokens.
//!
//! Post processing steps:
//! - White space handling: create indent and dedent tokens.
//! - Newline counting (for proper source locations)
//! - Detection of keywords

use super::location::Location;
use super::token::Token;
use logos::{Lexer, Logos};
use std::collections::HashMap;
use std::str::FromStr;

fn shrink_string(txt: &str) -> String {
    let mut txt: String = txt.to_owned();
    txt.pop();
    txt.remove(0);
    txt
}

fn hex_number(txt: &str) -> i32 {
    i32::from_str_radix(&txt[2..], 16).unwrap()
}

#[derive(Logos, Debug, Clone)]
pub enum LogosToken {
    #[regex("[a-zA-Z]+", |l| l.slice().to_string())]
    Identifier(String),

    #[regex("[0-9]+", |l| i32::from_str(l.slice()))]
    Number(i32),

    #[regex("0x[0-9a-fA-F]+", |l| hex_number(l.slice()))]
    HexNumber(i32),

    #[regex(r"[0-9]+\.[0-9]+", |l| f64::from_str(l.slice()))]
    Float(f64),

    #[regex(r#"""".+""""#, |l| l.slice().to_string())]
    LongString(String),

    #[regex("'[^']+'", |l| shrink_string(l.slice()))]
    SmallString(String),

    #[regex(r#"#.+\n"#, |l| l.slice().to_string())]
    Comment(String),

    #[token(":")]
    Colon,

    #[token("(")]
    OpeningParenthesis,

    #[token(")")]
    ClosingParenthesis,

    #[token("[")]
    OpeningBracket,

    #[token("]")]
    ClosingBracket,

    #[token("{")]
    OpeningBrace,

    #[token("}")]
    ClosingBrace,

    #[token(",")]
    Comma,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Asterix,

    #[token("/")]
    Slash,

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

    #[token("=")]
    Equal,

    #[token("->")]
    Arrow,

    #[token("\\\n")]
    BackslashNewLine,

    #[token("\n")]
    NewLine,

    #[regex(" +", |l| l.slice().len())]
    WhiteSpace(usize),

    #[error]
    Error,
}

#[derive(Debug)]
pub struct LexicalError {
    pub msg: String,
    pub location: Location,
}

type Spanned<Tok, Error> = Result<(Location, Tok, Location), Error>;

pub struct MyLexer<'t> {
    inner: Lexer<'t, LogosToken>,
    pending: Vec<(Location, Token, Location)>,
    parenthesis_level: usize,
    spaces: usize,
    row: usize,
    column_offset: usize,
    indentations: Vec<usize>,
    at_end: bool,
    at_bol: bool, // begin of line
    keywords: HashMap<String, Token>,
}

fn get_keyword_map() -> HashMap<String, Token> {
    let mut keywords = HashMap::new();
    keywords.insert("and".to_owned(), Token::KeywordAnd);
    keywords.insert("break".to_owned(), Token::KeywordBreak);
    keywords.insert("class".to_owned(), Token::KeywordClass);
    keywords.insert("continue".to_owned(), Token::KeywordContinue);
    keywords.insert("def".to_owned(), Token::KeywordDef);
    keywords.insert("else".to_owned(), Token::KeywordElse);
    keywords.insert("for".to_owned(), Token::KeywordFor);
    keywords.insert("from".to_owned(), Token::KeywordFrom);
    keywords.insert("if".to_owned(), Token::KeywordIf);
    keywords.insert("import".to_owned(), Token::KeywordImport);
    keywords.insert("in".to_owned(), Token::KeywordIn);
    keywords.insert("or".to_owned(), Token::KeywordOr);
    keywords.insert("pass".to_owned(), Token::KeywordPass);
    keywords.insert("return".to_owned(), Token::KeywordReturn);
    keywords.insert("while".to_owned(), Token::KeywordWhile);
    keywords
}

impl<'t> MyLexer<'t> {
    pub fn new(txt: &'t str) -> Self {
        let inner = LogosToken::lexer(txt);
        let keywords = get_keyword_map();

        MyLexer {
            inner,
            pending: vec![],
            parenthesis_level: 0,
            spaces: 0,
            row: 1,
            column_offset: 0,
            indentations: vec![0],
            at_end: false,
            at_bol: true,
            keywords,
        }
    }

    fn get_location(&self, offset: usize) -> Location {
        let column = offset - self.column_offset + 1;
        Location {
            row: self.row,
            column,
        }
    }

    fn emit(&mut self, token: Token) {
        if self.at_bol && self.parenthesis_level == 0 {
            self.at_bol = false;

            let new_indentation: usize = self.spaces;
            self.update_indentation(new_indentation);
        }

        // Emit current span:
        self.emit_spanned(token, self.inner.span());
    }

    fn emit_spanned(&mut self, token: Token, span: logos::Span) {
        let spanned = (
            self.get_location(span.start),
            token,
            self.get_location(span.end),
        );
        self.pending.push(spanned);
    }

    /// Do indent / dedent book keepings
    fn update_indentation(&mut self, new_indentation: usize) {
        use std::cmp::Ordering;

        let location = Location {
            row: self.row,
            column: 1,
        };

        match new_indentation.cmp(&self.get_current_indentation()) {
            Ordering::Greater => {
                self.indent(new_indentation, location);
            }
            Ordering::Less => {
                while new_indentation < self.get_current_indentation() {
                    self.dedent(location.clone());
                }

                if new_indentation != self.get_current_indentation() {
                    panic!("Indentation error.");
                }
            }
            Ordering::Equal => {}
        }
    }

    fn get_current_indentation(&self) -> usize {
        *self.indentations.last().unwrap()
    }

    /// Process a single inner token.
    fn process(&mut self) -> Result<(), LexicalError> {
        let t = self.inner.next();
        if let Some(t) = t {
            // debug!("Logos Tok: {:?} --> '{}'", t, self.inner.slice());

            match t {
                LogosToken::Colon => self.emit(Token::Colon),
                LogosToken::Comma => self.emit(Token::Comma),
                LogosToken::Minus => self.emit(Token::Minus),
                LogosToken::Plus => self.emit(Token::Plus),
                LogosToken::Arrow => self.emit(Token::Arrow),
                LogosToken::Asterix => self.emit(Token::Asterix),
                LogosToken::Slash => self.emit(Token::Slash),
                LogosToken::Less => self.emit(Token::Less),
                LogosToken::Greater => self.emit(Token::Greater),
                LogosToken::LessEqual => self.emit(Token::LessEqual),
                LogosToken::GreaterEqual => self.emit(Token::GreaterEqual),
                LogosToken::EqualEqual => self.emit(Token::EqualEqual),
                LogosToken::NotEqual => self.emit(Token::NotEqual),
                LogosToken::Equal => self.emit(Token::Equal),
                LogosToken::OpeningBracket => {
                    self.parenthesis_level += 1;
                    self.emit(Token::OpeningBracket);
                }
                LogosToken::ClosingBracket => {
                    self.parenthesis_level -= 1;
                    self.emit(Token::ClosingBracket);
                }
                LogosToken::OpeningParenthesis => {
                    self.parenthesis_level += 1;
                    self.emit(Token::OpeningParenthesis);
                }
                LogosToken::ClosingParenthesis => {
                    self.parenthesis_level -= 1;
                    self.emit(Token::ClosingParenthesis);
                }
                LogosToken::OpeningBrace => {
                    self.parenthesis_level += 1;
                    self.emit(Token::OpeningBrace);
                }
                LogosToken::ClosingBrace => {
                    self.parenthesis_level += 1;
                    self.emit(Token::ClosingBrace);
                }
                LogosToken::NewLine => {
                    self.newline();
                }
                LogosToken::BackslashNewLine => {
                    // Hmm, special action when backslash at end of line  .....
                    self.column_offset = self.inner.span().end;
                    self.row += 1;
                }
                LogosToken::WhiteSpace(w) => {
                    if self.at_bol {
                        self.spaces += w;
                    }
                }
                LogosToken::Identifier(value) => {
                    if self.keywords.contains_key(&value) {
                        self.emit(self.keywords[&value].clone());
                    } else {
                        self.emit(Token::Identifier { value })
                    }
                }
                LogosToken::Number(value) | LogosToken::HexNumber(value) => {
                    self.emit(Token::Number { value })
                }
                LogosToken::Float(value) => self.emit(Token::Float { value }),
                LogosToken::LongString(value) | LogosToken::SmallString(value) => {
                    // pha, ignore for now. Assume docstring
                    // warn!("Ignoring assumed doc-string {}", value);
                    self.emit(Token::Str { value })
                }
                LogosToken::Comment(_) => {
                    // info!("Skipping comment {}", value);
                    self.newline();
                }
                LogosToken::Error => {
                    let location = self.get_location(self.inner.span().start);
                    return Err(LexicalError {
                        msg: "Invalid character!".to_owned(),
                        location,
                    });
                }
            };
        } else {
            self.at_end = true;
            let location = Location {
                row: self.row,
                column: 1,
            };

            // Flush indentations:
            while self.indentations.len() > 1 {
                self.dedent(location.clone());
            }
        }

        Ok(())
    }

    fn newline(&mut self) {
        if !self.at_bol {
            let span = self.inner.span();
            self.emit_spanned(Token::NewLine, span);
        }
        self.at_bol = true;
        self.column_offset = self.inner.span().end;
        self.spaces = 0;
        self.row += 1;
    }

    fn indent(&mut self, new_indentation: usize, location: Location) {
        self.indentations.push(new_indentation);
        let spanned = (location.clone(), Token::Indent, location);
        self.pending.push(spanned);
    }

    fn dedent(&mut self, location: Location) {
        let spanned = (location.clone(), Token::Dedent, location);
        self.pending.push(spanned);
        self.indentations.pop();
    }
}

impl<'t> std::iter::Iterator for MyLexer<'t> {
    type Item = Spanned<Token, LexicalError>;

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
