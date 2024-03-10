use super::{Identifier::Identifier, Pattern::Pattern, Statement::Block};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BinaryOp {
    And,
    Or,
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug)]
pub struct Branch {
    pub pattern: Pattern,
    pub body: Expr,
}

#[derive(Debug)]
pub enum Expr {
    Value(Identifier),
    SelfValue,
    Name(Identifier),
    FieldAccess(Box<Expr>, Identifier),
    Call(Box<Expr>, Vec<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    For(Pattern, Box<Expr>, Block),
    BinaryOp(BinaryOp, Box<Expr>, Box<Expr>),
    Match(Box<Expr>, Vec<Branch>),
    Block(Block),
    Tuple(Vec<Expr>),
}
