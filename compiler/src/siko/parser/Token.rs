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
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ShiftLeft,
    ShiftRight,
    BitAnd,
    BitOr,
    BitXor,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum KeywordKind {
    Module,
    Struct,
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
    Implicit,
    With,
    Using,
    Let,
    Derive,
    Type,
    Pub,
    Void,
    Not,
    Yield,
    Co,
    Raw,
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
    ThreeDots,
    Equal,
    Comma,
    Colon,
    Semicolon,
    ExclamationMark,
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
    CharLiteral(String),
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
            _ => write!(f, "{}", self.kind()),
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
            TokenKind::Keyword(KeywordKind::As) => write!(f, "as"),
            TokenKind::Keyword(KeywordKind::Break) => write!(f, "break"),
            TokenKind::Keyword(KeywordKind::Struct) => write!(f, "struct"),
            TokenKind::Keyword(KeywordKind::Continue) => write!(f, "continue"),
            TokenKind::Keyword(KeywordKind::Derive) => write!(f, "derive"),
            TokenKind::Keyword(KeywordKind::Effect) => write!(f, "effect"),
            TokenKind::Keyword(KeywordKind::Else) => write!(f, "else"),
            TokenKind::Keyword(KeywordKind::Enum) => write!(f, "enum"),
            TokenKind::Keyword(KeywordKind::Extern) => write!(f, "extern"),
            TokenKind::Keyword(KeywordKind::Fn) => write!(f, "fn"),
            TokenKind::Keyword(KeywordKind::For) => write!(f, "for"),
            TokenKind::Keyword(KeywordKind::Hiding) => write!(f, "hiding"),
            TokenKind::Keyword(KeywordKind::If) => write!(f, "if"),
            TokenKind::Keyword(KeywordKind::Implicit) => write!(f, "implicit"),
            TokenKind::Keyword(KeywordKind::Import) => write!(f, "import"),
            TokenKind::Keyword(KeywordKind::In) => write!(f, "in"),
            TokenKind::Keyword(KeywordKind::Instance) => write!(f, "instance"),
            TokenKind::Keyword(KeywordKind::Let) => write!(f, "let"),
            TokenKind::Keyword(KeywordKind::Loop) => write!(f, "loop"),
            TokenKind::Keyword(KeywordKind::Match) => write!(f, "match"),
            TokenKind::Keyword(KeywordKind::Module) => write!(f, "module"),
            TokenKind::Keyword(KeywordKind::Mut) => write!(f, "mut"),
            TokenKind::Keyword(KeywordKind::Pub) => write!(f, "pub"),
            TokenKind::Keyword(KeywordKind::Return) => write!(f, "return"),
            TokenKind::Keyword(KeywordKind::Trait) => write!(f, "trait"),
            TokenKind::Keyword(KeywordKind::Try) => write!(f, "try"),
            TokenKind::Keyword(KeywordKind::Type) => write!(f, "type"),
            TokenKind::Keyword(KeywordKind::Using) => write!(f, "using"),
            TokenKind::Keyword(KeywordKind::ValueSelf) => write!(f, "self"),
            TokenKind::Keyword(KeywordKind::TypeSelf) => write!(f, "Self"),
            TokenKind::Keyword(KeywordKind::While) => write!(f, "while"),
            TokenKind::Keyword(KeywordKind::With) => write!(f, "with"),
            TokenKind::Keyword(KeywordKind::Then) => write!(f, "then"),
            TokenKind::Keyword(KeywordKind::Void) => write!(f, "void"),
            TokenKind::Keyword(KeywordKind::Not) => write!(f, "not"),
            TokenKind::Keyword(KeywordKind::Yield) => write!(f, "yield"),
            TokenKind::Keyword(KeywordKind::Co) => write!(f, "co"),
            TokenKind::Keyword(KeywordKind::Raw) => write!(f, "raw"),
            TokenKind::Arrow(ArrowKind::DoubleLeft) => write!(f, "<="),
            TokenKind::Arrow(ArrowKind::DoubleRight) => write!(f, "=>"),
            TokenKind::Arrow(ArrowKind::Left) => write!(f, "<-"),
            TokenKind::Arrow(ArrowKind::Right) => write!(f, "->"),
            TokenKind::Range(RangeKind::Exclusive) => write!(f, ".."),
            TokenKind::Range(RangeKind::Inclusive) => write!(f, "..="),
            TokenKind::Misc(MiscKind::At) => write!(f, "@"),
            TokenKind::Misc(MiscKind::Backslash) => write!(f, "\\"),
            TokenKind::Misc(MiscKind::Colon) => write!(f, ":"),
            TokenKind::Misc(MiscKind::Comma) => write!(f, ","),
            TokenKind::Misc(MiscKind::Dot) => write!(f, "."),
            TokenKind::Misc(MiscKind::ThreeDots) => write!(f, "..."),
            TokenKind::Misc(MiscKind::Equal) => write!(f, "="),
            TokenKind::Misc(MiscKind::ExclamationMark) => write!(f, "!"),
            TokenKind::Misc(MiscKind::Percent) => write!(f, "%"),
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
            TokenKind::Op(OperatorKind::AddAssign) => write!(f, "+="),
            TokenKind::Op(OperatorKind::SubAssign) => write!(f, "-="),
            TokenKind::Op(OperatorKind::MulAssign) => write!(f, "*="),
            TokenKind::Op(OperatorKind::DivAssign) => write!(f, "/="),
            TokenKind::Op(OperatorKind::ShiftLeft) => write!(f, "<<"),
            TokenKind::Op(OperatorKind::ShiftRight) => write!(f, ">>"),
            TokenKind::Op(OperatorKind::BitAnd) => write!(f, "&"),
            TokenKind::Op(OperatorKind::BitOr) => write!(f, "|"),
            TokenKind::Op(OperatorKind::BitXor) => write!(f, "^"),
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
