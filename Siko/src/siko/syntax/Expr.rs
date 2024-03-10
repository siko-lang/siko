use crate::siko::location::Location::Location;

use super::{Identifier::Identifier, Pattern::Pattern, Statement::Block};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BinaryOp {
    And,
    Or,
    Add,
    Sub,
    Mul,
    Div,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
}

#[derive(Debug)]
pub struct Branch {
    pub pattern: Pattern,
    pub body: Expr,
}

#[derive(Debug)]
pub struct Expr {
    pub expr: SimpleExpr,
    pub location: Location,
}

#[derive(Debug)]
pub enum SimpleExpr {
    Value(Identifier),
    SelfValue,
    Name(Identifier),
    FieldAccess(Box<Expr>, Identifier),
    Call(Box<Expr>, Vec<Expr>),
    If(Box<Expr>, Box<Expr>, Option<Box<Expr>>),
    For(Pattern, Box<Expr>, Box<Expr>),
    BinaryOp(BinaryOp, Box<Expr>, Box<Expr>),
    Match(Box<Expr>, Vec<Branch>),
    Block(Block),
    Tuple(Vec<Expr>),
    StringLiteral(String),
    IntegerLiteral(String),
    CharLiteral(char),
    Return(Option<Box<Expr>>),
    Break(Option<Box<Expr>>),
    Continue(Option<Box<Expr>>),
}
