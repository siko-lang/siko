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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UnaryOp {
    Not,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branch {
    pub pattern: Pattern,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    pub expr: SimpleExpr,
    pub location: Location,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimpleExpr {
    Value(Identifier),
    SelfValue,
    Name(Identifier),
    FieldAccess(Box<Expr>, Identifier),
    TupleIndex(Box<Expr>, String),
    Call(Box<Expr>, Vec<Expr>),
    MethodCall(Box<Expr>, Identifier, Vec<Expr>),
    For(Pattern, Box<Expr>, Box<Expr>),
    Loop(Pattern, Box<Expr>, Box<Expr>),
    BinaryOp(BinaryOp, Box<Expr>, Box<Expr>),
    UnaryOp(UnaryOp, Box<Expr>),
    Match(Box<Expr>, Vec<Branch>),
    Block(Block),
    Tuple(Vec<Expr>),
    StringLiteral(String),
    IntegerLiteral(String),
    CharLiteral(char),
    Return(Option<Box<Expr>>),
    Break(Option<Box<Expr>>),
    Continue(Option<Box<Expr>>),
    Ref(Box<Expr>),
}

impl SimpleExpr {
    pub fn doesNotReturn(&self) -> bool {
        match self {
            SimpleExpr::Return(_) => true,
            SimpleExpr::Break(_) => true,
            SimpleExpr::Continue(_) => true,
            _ => false,
        }
    }
}
