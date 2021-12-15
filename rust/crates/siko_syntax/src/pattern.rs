use crate::expr::ExprId;
use crate::types::TypeSignatureId;
use siko_location_info::location_id::LocationId;
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
pub struct RecordFieldPattern {
    pub name: String,
    pub value: PatternId,
    pub location_id: LocationId,
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
    Constructor(String, Vec<PatternId>),
    Guarded(PatternId, ExprId),
    Wildcard,
    IntegerLiteral(i64),
    StringLiteral(String),
    CharLiteral(char),
    Typed(PatternId, TypeSignatureId),
    Record(String, Vec<RecordFieldPattern>),
    CharRange(char, char, RangeKind),
    Or(Vec<PatternId>),
}
