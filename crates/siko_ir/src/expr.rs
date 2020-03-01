use crate::class::ClassMemberId;
use crate::data::TypeDefId;
use crate::function::FunctionId;
use crate::pattern::BindGroup;
use crate::pattern::PatternId;
use siko_util::format_list;
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct FunctionArgumentRef {
    pub captured: bool,
    pub id: FunctionId,
    pub index: usize,
}

impl FunctionArgumentRef {
    pub fn new(captured: bool, id: FunctionId, index: usize) -> FunctionArgumentRef {
        FunctionArgumentRef {
            captured: captured,
            id: id,
            index: index,
        }
    }
}

impl fmt::Display for FunctionArgumentRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ArgRef({}.{})", self.id, self.index)
    }
}

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
pub struct FieldAccessInfo {
    pub record_id: TypeDefId,
    pub index: usize,
}

impl fmt::Display for FieldAccessInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FieldAccessInfo{}:{}", self.record_id, self.index)
    }
}

#[derive(Debug, Clone)]
pub struct Case {
    pub pattern_id: PatternId,
    pub body: ExprId,
}

impl fmt::Display for Case {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} -> {}", self.pattern_id, self.body)
    }
}

#[derive(Debug, Clone)]
pub struct RecordFieldValueExpr {
    pub expr_id: ExprId,
    pub index: usize,
}

impl fmt::Display for RecordFieldValueExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} -> {}", self.expr_id, self.index)
    }
}

#[derive(Debug, Clone)]
pub struct RecordUpdateInfo {
    pub record_id: TypeDefId,
    pub items: Vec<RecordFieldValueExpr>,
}

impl fmt::Display for RecordUpdateInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} -> {}", self.record_id, format_list(&self.items))
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    StaticFunctionCall(FunctionId, Vec<ExprId>),
    DynamicFunctionCall(ExprId, Vec<ExprId>),
    ClassFunctionCall(ClassMemberId, Vec<ExprId>),
    If(ExprId, ExprId, ExprId),
    Tuple(Vec<ExprId>),
    List(Vec<ExprId>),
    IntegerLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    CharLiteral(char),
    Do(Vec<ExprId>),
    Bind(PatternId, ExprId),
    ArgRef(FunctionArgumentRef),
    ExprValue(ExprId, PatternId),
    FieldAccess(Vec<FieldAccessInfo>, ExprId),
    TupleFieldAccess(usize, ExprId),
    Formatter(String, Vec<ExprId>),
    CaseOf(ExprId, Vec<Case>, Vec<BindGroup>),
    RecordInitialization(TypeDefId, Vec<RecordFieldValueExpr>),
    RecordUpdate(ExprId, Vec<RecordUpdateInfo>),
    Return(ExprId),
    Loop(PatternId, ExprId, Vec<ExprId>, Vec<ExprId>, Vec<ExprId>),
    Continue(ExprId),
    Break(ExprId),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::StaticFunctionCall(id, args) => {
                write!(f, "StaticFunctionCall({}, {})", id, format_list(args))
            }
            Expr::DynamicFunctionCall(id_expr, args) => {
                write!(f, "DynamicFunctionCall({}, {})", id_expr, format_list(args))
            }
            Expr::ClassFunctionCall(class_member_id, args) => write!(
                f,
                "ClassFunctionCall({}, {})",
                class_member_id,
                format_list(args)
            ),
            Expr::If(cond, true_branch, false_branch) => {
                write!(f, "If({}, {}, {})", cond, true_branch, false_branch)
            }
            Expr::Tuple(items) => write!(f, "Tuple({})", format_list(items)),
            Expr::List(items) => write!(f, "[{}]", format_list(items)),
            Expr::IntegerLiteral(v) => write!(f, "Integer({})", v),
            Expr::FloatLiteral(v) => write!(f, "Float({})", v),
            Expr::StringLiteral(v) => write!(f, "String({})", v),
            Expr::CharLiteral(v) => write!(f, "Char({})", v),
            Expr::Do(items) => write!(f, "Do({})", format_list(items)),
            Expr::Bind(pattern_id, expr) => write!(f, "Bind({}, {})", pattern_id, expr),
            Expr::ArgRef(v) => write!(f, "{}", v),
            Expr::ExprValue(id, index) => write!(f, "ExprValue({}, {})", id, index),
            Expr::FieldAccess(accesses, expr) => {
                write!(f, "FieldAccess({}, {})", format_list(accesses), expr)
            }
            Expr::TupleFieldAccess(index, expr) => {
                write!(f, "TupleFieldAccess({}, {})", index, expr)
            }
            Expr::Formatter(fmt, items) => write!(f, "Formatter({}, {})", fmt, format_list(items)),
            Expr::CaseOf(body, cases, _) => write!(f, "CaseOf({}, {})", body, format_list(cases)),
            Expr::RecordInitialization(type_id, items) => write!(
                f,
                "RecordInitialization({}, {})",
                type_id,
                format_list(items)
            ),
            Expr::RecordUpdate(expr_id, items) => {
                write!(f, "RecordUpdate({}, {})", expr_id, format_list(items))
            }
            Expr::Return(expr_id) => write!(f, "Return({})", expr_id),
            Expr::Loop(pattern, start, block, _, _) => {
                write!(f, "Loop({}, {}, {})", pattern, start, format_list(block))
            }
            Expr::Continue(expr) => write!(f, "Continue({})", expr),
            Expr::Break(expr) => write!(f, "Break({})", expr),
        }
    }
}
