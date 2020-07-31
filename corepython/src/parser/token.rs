#[derive(Debug, Clone)]
pub enum Token {
    Indent,
    Dedent,
    KeywordAnd,
    KeywordBreak,
    KeywordClass,
    KeywordContinue,
    KeywordDef,
    KeywordElse,
    KeywordFor,
    KeywordFrom,
    KeywordIf,
    KeywordIn,
    KeywordImport,
    KeywordOr,
    KeywordPass,
    KeywordReturn,
    KeywordWhile,
    Identifier { value: String },
    Number { value: i32 },
    Float { value: f64 },
    Str { value: String },
    Colon,
    OpeningParenthesis,
    ClosingParenthesis,
    OpeningBrace,
    ClosingBrace,
    OpeningBracket,
    ClosingBracket,
    Comma,
    Plus,
    Minus,
    Asterix,
    Slash,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    EqualEqual,
    NotEqual,
    Equal,
    Arrow,
    NewLine,
}
