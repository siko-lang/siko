use super::Pattern::Pattern;

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
    pattern: Pattern,
    body: Expr,
}

#[derive(Debug)]
pub enum Expr {
    Call(Box<Expr>, Vec<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    BinaryOp(BinaryOp, Box<Expr>, Box<Expr>),
    Match(Box<Expr>, Vec<Branch>),
}
