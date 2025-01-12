use std::fmt::Debug;
use std::fmt::Display;

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
    While,
    For,
    Continue,
    Break,
    Match,
    Effect,
    With,
    Using,
    Let,
    Derive,
    Implicit,
    Type,
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
    Equal,
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

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::VarIdentifier(v) => write!(f, "{}", v),
            Token::TypeIdentifier(v) => write!(f, "{}", v),
            Token::LeftBracket(BracketKind::Curly) => write!(f, "{{"),
            Token::LeftBracket(BracketKind::Paren) => write!(f, "("),
            Token::LeftBracket(BracketKind::Square) => write!(f, "["),
            Token::RightBracket(BracketKind::Curly) => write!(f, "}}"),
            Token::RightBracket(BracketKind::Paren) => write!(f, ")"),
            Token::RightBracket(BracketKind::Square) => write!(f, "]"),
            Token::StringLiteral(v) => write!(f, "\"{}\"", v),
            Token::IntegerLiteral(v) => write!(f, "{}", v),
            Token::CharLiteral(v) => write!(f, "'{}'", v),
            Token::Keyword(k) => write!(f, "{:?}", k),
            Token::Arrow(ArrowKind::DoubleLeft) => write!(f, "<="),
            Token::Arrow(ArrowKind::DoubleRight) => write!(f, "=>"),
            Token::Arrow(ArrowKind::Left) => write!(f, "<-"),
            Token::Arrow(ArrowKind::Right) => write!(f, "->"),
            Token::Range(RangeKind::Exclusive) => write!(f, ".."),
            Token::Range(RangeKind::Inclusive) => write!(f, "..="),
            Token::Misc(MiscKind::Ampersand) => write!(f, "&"),
            Token::Misc(MiscKind::At) => write!(f, "@"),
            Token::Misc(MiscKind::Backslash) => write!(f, "\\"),
            Token::Misc(MiscKind::Colon) => write!(f, ":"),
            Token::Misc(MiscKind::Comma) => write!(f, ","),
            Token::Misc(MiscKind::Dot) => write!(f, "."),
            Token::Misc(MiscKind::Equal) => write!(f, "="),
            Token::Misc(MiscKind::ExclamationMark) => write!(f, "!"),
            Token::Misc(MiscKind::Percent) => write!(f, "%"),
            Token::Misc(MiscKind::Pipe) => write!(f, "|"),
            Token::Misc(MiscKind::Semicolon) => write!(f, ";"),
            Token::Misc(MiscKind::Tilde) => write!(f, "~"),
            Token::Misc(MiscKind::Wildcard) => write!(f, "_"),
            Token::Op(OperatorKind::Add) => write!(f, "+"),
            Token::Op(OperatorKind::And) => write!(f, "&&"),
            Token::Op(OperatorKind::Div) => write!(f, "/"),
            Token::Op(OperatorKind::Equal) => write!(f, "=="),
            Token::Op(OperatorKind::GreaterThan) => write!(f, ">"),
            Token::Op(OperatorKind::GreaterThanOrEqual) => write!(f, ">="),
            Token::Op(OperatorKind::LessThan) => write!(f, "<"),
            Token::Op(OperatorKind::LessThanOrEqual) => write!(f, "<="),
            Token::Op(OperatorKind::Mul) => write!(f, "*"),
            Token::Op(OperatorKind::NotEqual) => write!(f, "!="),
            Token::Op(OperatorKind::Or) => write!(f, "||"),
            Token::Op(OperatorKind::Sub) => write!(f, "-"),
            Token::EOF => write!(f, "EOF"),
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

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::VarIdentifier => write!(f, "identifier"),
            TokenKind::TypeIdentifier => write!(f, "type identifier"),
            TokenKind::LeftBracket(BracketKind::Curly) => write!(f, "{{"),
            TokenKind::LeftBracket(BracketKind::Paren) => write!(f, "("),
            TokenKind::LeftBracket(BracketKind::Square) => write!(f, "["),
            TokenKind::RightBracket(BracketKind::Curly) => write!(f, "}}"),
            TokenKind::RightBracket(BracketKind::Paren) => write!(f, ")"),
            TokenKind::RightBracket(BracketKind::Square) => write!(f, "]"),
            TokenKind::StringLiteral => write!(f, "string literal"),
            TokenKind::IntegerLiteral => write!(f, "integer literal"),
            TokenKind::CharLiteral => write!(f, "char literal"),
            TokenKind::Keyword(k) => write!(f, "{:?}", k),
            TokenKind::Arrow(ArrowKind::DoubleLeft) => write!(f, "<="),
            TokenKind::Arrow(ArrowKind::DoubleRight) => write!(f, "=>"),
            TokenKind::Arrow(ArrowKind::Left) => write!(f, "<-"),
            TokenKind::Arrow(ArrowKind::Right) => write!(f, "->"),
            TokenKind::Range(RangeKind::Exclusive) => write!(f, ".."),
            TokenKind::Range(RangeKind::Inclusive) => write!(f, "..="),
            TokenKind::Misc(MiscKind::Ampersand) => write!(f, "&"),
            TokenKind::Misc(MiscKind::At) => write!(f, "@"),
            TokenKind::Misc(MiscKind::Backslash) => write!(f, "\\"),
            TokenKind::Misc(MiscKind::Colon) => write!(f, ":"),
            TokenKind::Misc(MiscKind::Comma) => write!(f, ","),
            TokenKind::Misc(MiscKind::Dot) => write!(f, "."),
            TokenKind::Misc(MiscKind::Equal) => write!(f, "="),
            TokenKind::Misc(MiscKind::ExclamationMark) => write!(f, "!"),
            TokenKind::Misc(MiscKind::Percent) => write!(f, "%"),
            TokenKind::Misc(MiscKind::Pipe) => write!(f, "|"),
            TokenKind::Misc(MiscKind::Semicolon) => write!(f, ";"),
            TokenKind::Misc(MiscKind::Tilde) => write!(f, "~"),
            TokenKind::Misc(MiscKind::Wildcard) => write!(f, "_"),
            TokenKind::Op(OperatorKind::Add) => write!(f, "+"),
            TokenKind::Op(OperatorKind::And) => write!(f, "&&"),
            TokenKind::Op(OperatorKind::Div) => write!(f, "/"),
            TokenKind::Op(OperatorKind::Equal) => write!(f, "=="),
            TokenKind::Op(OperatorKind::GreaterThan) => write!(f, ">"),
            TokenKind::Op(OperatorKind::GreaterThanOrEqual) => write!(f, ">="),
            TokenKind::Op(OperatorKind::LessThan) => write!(f, "<"),
            TokenKind::Op(OperatorKind::LessThanOrEqual) => write!(f, "<="),
            TokenKind::Op(OperatorKind::Mul) => write!(f, "*"),
            TokenKind::Op(OperatorKind::NotEqual) => write!(f, "!="),
            TokenKind::Op(OperatorKind::Or) => write!(f, "||"),
            TokenKind::Op(OperatorKind::Sub) => write!(f, "-"),
            TokenKind::EOF => write!(f, "EOF"),
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct TokenInfo {
    pub token: Token,
    pub span: Span,
}

impl TokenInfo {
    pub fn kind(&self) -> TokenKind {
        self.token.kind()
    }
}

impl Display for TokenInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.token)
    }
}

impl Debug for TokenInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.token)
    }
}
