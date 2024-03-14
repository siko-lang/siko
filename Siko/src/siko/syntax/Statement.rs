use crate::siko::location::Location::Location;

use super::{Expr::Expr, Pattern::Pattern};

#[derive(Debug)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub location: Location,
}

#[derive(Debug)]
pub enum StatementKind {
    Expr(Expr),
    Assign(Expr, Expr),
    Let(Pattern, Expr),
}

#[derive(Debug)]
pub struct Statement {
    pub kind: StatementKind,
    pub hasSemicolon: bool,
}
