use crate::data::TypeDefId;
use crate::function::FunctionId;
use crate::pattern::PatternId;
use crate::types::PartialFunctionCallId;
use std::fmt;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct ExprId {
    pub id: usize,
}

impl fmt::Display for ExprId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{}", self.id)
    }
}

impl From<usize> for ExprId {
    fn from(id: usize) -> ExprId {
        ExprId { id: id }
    }
}

#[derive(Debug, Clone)]
pub struct Case {
    pub pattern_id: PatternId,
    pub body: ExprId,
}

#[derive(Debug, Clone)]
pub enum Expr {
    ArgRef(usize),
    Bind(PatternId, ExprId),
    CaseOf(ExprId, Vec<Case>),
    CharLiteral(char),
    Clone(ExprId),
    Deref(ExprId),
    Do(Vec<ExprId>),
    DynamicFunctionCall(ExprId, Vec<ExprId>),
    ExprValue(ExprId, PatternId),
    FieldAccess(usize, ExprId),
    FloatLiteral(f64),
    Formatter(String, Vec<ExprId>),
    If(ExprId, ExprId, ExprId),
    IntegerLiteral(i64),
    List(Vec<ExprId>),
    RecordInitialization(TypeDefId, Vec<(ExprId, usize)>),
    RecordUpdate(ExprId, Vec<(ExprId, usize)>),
    StaticFunctionCall(FunctionId, Vec<ExprId>),
    PartialFunctionCall(PartialFunctionCallId, Vec<ExprId>),
    StringLiteral(String),
    Return(ExprId),
}
