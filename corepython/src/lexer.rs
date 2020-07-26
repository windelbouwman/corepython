use super::token::Token;
use logos::{Lexer, Logos};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Logos, Debug, Clone)]
pub enum LogosToken {
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

    #[token("\\\n")]
    BackslashNewLine,

    #[token("\n")]
    NewLine,

    #[regex(" +", |l| l.slice().len())]
    WhiteSpace(usize),

    #[error]
    Error,
}

/// Location in the source file.
#[derive(Clone, Debug, Default)]
pub struct Location {
    pub row: usize,
    pub column: usize,
}

impl Location {
    pub fn get_text_for_user(&self) -> String {
        format!("<filename>:{}:{}", self.row, self.column)
    }
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
    keywords.insert("if".to_owned(), Token::KeywordIf);
    keywords.insert("in".to_owned(), Token::KeywordIn);
    keywords.insert("or".to_owned(), Token::KeywordOr);
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
        let span = self.inner.span();
        let spanned = (
            self.get_location(span.start),
            token,
            self.get_location(span.end),
        );
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
                    self.column_offset = self.inner.span().end;
                    self.spaces = 0;
                    self.row += 1;
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
                LogosToken::Number(value) => self.emit(Token::Number { value }),
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

            // Flush indentations:
            while self.indentations.len() > 1 {
                self.dedent();
            }
        }

        Ok(())
    }

    fn indent(&mut self, new_indentation: usize) {
        self.indentations.push(new_indentation);
        let spanned = (Default::default(), Token::Indent, Default::default());
        self.pending.push(spanned);
    }

    fn dedent(&mut self) {
        let spanned = (Default::default(), Token::Dedent, Default::default());
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
