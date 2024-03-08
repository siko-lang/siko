use super::{Expr::Expr, Pattern::Pattern};

#[derive(Debug)]
pub enum Statement {
    Expr(Expr),
    Assign(Expr, Expr),
    Let(Pattern, Expr),
}
