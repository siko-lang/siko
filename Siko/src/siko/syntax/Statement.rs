use crate::siko::location::Location::Location;

use super::{Expr::Expr, Pattern::Pattern};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub location: Location,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatementKind {
    Expr(Expr),
    Assign(Expr, Expr),
    Let(Pattern, Expr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Statement {
    pub kind: StatementKind,
    pub hasSemicolon: bool,
}
