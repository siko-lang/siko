use crate::siko::hir::Variable::Variable;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveKind {
    Fully(Variable),
    Partially,
    NotMoved,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PossibleCollision {
    pub first: Variable,
    pub second: Variable,
}
