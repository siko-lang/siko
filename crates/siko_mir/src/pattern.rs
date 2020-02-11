use crate::data::TypeDefId;
use crate::expr::ExprId;
use std::fmt;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct PatternId {
    pub id: usize,
}

impl fmt::Display for PatternId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{}", self.id)
    }
}

impl From<usize> for PatternId {
    fn from(id: usize) -> PatternId {
        PatternId { id: id }
    }
}

#[derive(Debug, Clone)]
pub enum RangeKind {
    Exclusive,
    Inclusive,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Binding(String),
    Record(TypeDefId, Vec<PatternId>),
    Variant(TypeDefId, usize, Vec<PatternId>),
    Guarded(PatternId, ExprId),
    Wildcard,
    IntegerLiteral(i64),
    CharLiteral(char),
    CharRange(char, char, RangeKind),
    StringLiteral(String),
}
