use crate::data::TypeDefId;
use crate::expr::ExprId;
use crate::type_signature::TypeSignatureId;
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
    Tuple(Vec<PatternId>),
    Record(TypeDefId, Vec<PatternId>),
    Variant(TypeDefId, usize, Vec<PatternId>),
    Guarded(PatternId, ExprId),
    Wildcard,
    IntegerLiteral(i64),
    StringLiteral(String),
    CharLiteral(char),
    CharRange(char, char, RangeKind),
    Typed(PatternId, TypeSignatureId),
}

#[derive(Debug, Clone)]
pub struct BindGroup {
    pub patterns: Vec<PatternId>,
}
