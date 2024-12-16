use crate::siko::location::Location::Location;

use super::{Expr::Expr, Pattern::Pattern, Type::Type};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub location: Location,
}

impl Block {
    pub fn doesNotReturn(&self) -> bool {
        for s in &self.statements {
            if s.kind.doesNotReturn() {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatementKind {
    Expr(Expr),
    Assign(Expr, Expr),
    Let(Pattern, Expr, Option<Type>),
}

impl StatementKind {
    pub fn doesNotReturn(&self) -> bool {
        match self {
            StatementKind::Expr(expr) => expr.doesNotReturn(),
            StatementKind::Assign(_, rhs) => rhs.doesNotReturn(),
            StatementKind::Let(_, rhs, _) => rhs.doesNotReturn(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Statement {
    pub kind: StatementKind,
    pub hasSemicolon: bool,
}
