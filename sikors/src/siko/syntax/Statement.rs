use super::{Expr::Expr, Pattern::Pattern};

#[derive(Debug)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    Expr(Expr),
    Assign(Expr, Expr),
    Let(Pattern, Expr),
}
