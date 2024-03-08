use crate::siko::location::Location::Span;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BracketKind {
    Paren,
    Curly,
    Square,
}

#[derive(Debug, PartialEq, Eq, Clone)]
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum KeywordKind {
    Module,
    Where,
    Class,
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
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ArrowKind {
    Left,
    Right,
    DoubleRight,
    DoubleLeft,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RangeKind {
    Exclusive,
    Inclusive,
}

#[derive(Debug, PartialEq, Eq, Clone)]
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
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenKind {
    VarIdentifier,
    TypeIdentifier,
    LeftParen(BracketKind),
    RightParen(BracketKind),
    StringLiteral,
    IntegerLiteral,
    CharLiteral,
    Keyword(KeywordKind),
    Arrow(ArrowKind),
    Range(RangeKind),
    Misc(MiscKind),
    Op(OperatorKind),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TokenInfo {
    pub token: Token,
    pub span: Span,
}
