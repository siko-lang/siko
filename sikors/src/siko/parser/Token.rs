use crate::siko::location::Location::Span;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BracketKind {
    Paren,
    Curly,
    Square,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OperatorKind {
    Equal,
    DoubleEqual,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum KeywordKind {
    Module,
    Class,
    Fn,
    Enum,
    Trait,
    Instance,
    Extern,
    Import,
    Hiding,
    As,
    In,
    Mut,
    ValueSelf,
    TypeSelf,
    If,
    Then,
    Else,
    Return,
    Try,
    Loop,
    For,
    Continue,
    Break,
    Match,
    Effect,
    With,
    Using,
    Let,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ArrowKind {
    Left,
    Right,
    DoubleRight,
    DoubleLeft,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RangeKind {
    Exclusive,
    Inclusive,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MiscKind {
    Dot,
    Comma,
    Colon,
    Semicolon,
    ExclamationMark,
    Ampersand,
    Pipe,
    Percent,
    Backslash,
    Tilde,
    Wildcard,
    At,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    VarIdentifier(String),
    TypeIdentifier(String),
    LeftBracket(BracketKind),
    RightBracket(BracketKind),
    StringLiteral(String),
    IntegerLiteral(String),
    CharLiteral(char),
    Keyword(KeywordKind),
    Arrow(ArrowKind),
    Range(RangeKind),
    Misc(MiscKind),
    Op(OperatorKind),
    EOF,
}

impl Token {
    pub fn kind(&self) -> TokenKind {
        match self {
            Token::VarIdentifier(_) => TokenKind::VarIdentifier,
            Token::TypeIdentifier(_) => TokenKind::TypeIdentifier,
            Token::LeftBracket(k) => TokenKind::LeftBracket(*k),
            Token::RightBracket(k) => TokenKind::RightBracket(*k),
            Token::StringLiteral(_) => TokenKind::StringLiteral,
            Token::IntegerLiteral(_) => TokenKind::IntegerLiteral,
            Token::CharLiteral(_) => TokenKind::CharLiteral,
            Token::Keyword(k) => TokenKind::Keyword(*k),
            Token::Arrow(k) => TokenKind::Arrow(*k),
            Token::Range(k) => TokenKind::Range(*k),
            Token::Misc(k) => TokenKind::Misc(*k),
            Token::Op(k) => TokenKind::Op(*k),
            Token::EOF => TokenKind::EOF,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenKind {
    VarIdentifier,
    TypeIdentifier,
    LeftBracket(BracketKind),
    RightBracket(BracketKind),
    StringLiteral,
    IntegerLiteral,
    CharLiteral,
    Keyword(KeywordKind),
    Arrow(ArrowKind),
    Range(RangeKind),
    Misc(MiscKind),
    Op(OperatorKind),
    EOF,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TokenInfo {
    pub token: Token,
    pub span: Span,
}

impl TokenInfo {
    pub fn kind(&self) -> TokenKind {
        self.token.kind()
    }
}
