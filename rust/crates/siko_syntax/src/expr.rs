use crate::pattern::PatternId;
use siko_constants::BuiltinOperator;
use siko_location_info::location_id::LocationId;
use siko_util::format_list;
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

impl fmt::Display for Case {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} -> {}", self.pattern_id, self.body)
    }
}

#[derive(Debug, Clone)]
pub struct RecordConstructionItem {
    pub field_name: String,
    pub body: ExprId,
    pub location_id: LocationId,
}

impl fmt::Display for RecordConstructionItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} -> {}", self.field_name, self.body)
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Lambda(Vec<(String, LocationId)>, ExprId),
    FunctionCall(ExprId, Vec<ExprId>),
    Builtin(BuiltinOperator),
    If(ExprId, ExprId, ExprId),
    Tuple(Vec<ExprId>),
    List(Vec<ExprId>),
    Path(String),
    IntegerLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    CharLiteral(char),
    Do(Vec<ExprId>),
    Bind(PatternId, ExprId),
    FieldAccess(String, ExprId),
    TupleFieldAccess(usize, ExprId),
    Formatter(String, Vec<ExprId>),
    CaseOf(ExprId, Vec<Case>),
    RecordInitialization(String, Vec<RecordConstructionItem>),
    RecordUpdate(String, Vec<RecordConstructionItem>),
    Return(ExprId),
    Loop(PatternId, ExprId, Vec<ExprId>),
    Continue(ExprId),
    Break(ExprId),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::Lambda(args, body) => {
                let args: Vec<_> = args.iter().map(|arg| &arg.0).collect();
                write!(f, "Lambda({}, {})", format_list(&args[..]), body)
            }
            Expr::FunctionCall(expr, args) => {
                write!(f, "FunctionCall({}, {})", expr, format_list(args))
            }
            Expr::Builtin(op) => write!(f, "Op({:?})", op),

            Expr::If(cond, true_branch, false_branch) => {
                write!(f, "If({}, {}, {})", cond, true_branch, false_branch)
            }
            Expr::Tuple(items) => write!(f, "Tuple({})", format_list(items)),
            Expr::List(items) => write!(f, "[{}]", format_list(items)),
            Expr::Path(path) => write!(f, "Path({})", path),
            Expr::IntegerLiteral(v) => write!(f, "Integer({})", v),
            Expr::FloatLiteral(v) => write!(f, "Float({})", v),
            Expr::StringLiteral(v) => write!(f, "String({})", v),
            Expr::CharLiteral(v) => write!(f, "Char({})", v),
            Expr::Do(items) => write!(f, "Do({})", format_list(items)),
            Expr::Bind(t, expr) => write!(f, "Bind({}, {})", t, expr),
            Expr::FieldAccess(name, expr) => write!(f, "FieldAccess({}, {})", name, expr),
            Expr::TupleFieldAccess(index, expr) => {
                write!(f, "TupleFieldAccess({}, {})", index, expr)
            }
            Expr::Formatter(fmt, items) => write!(f, "Formatter({}, {})", fmt, format_list(items)),
            Expr::CaseOf(body, cases) => write!(f, "CaseOf({}, {})", body, format_list(cases)),
            Expr::RecordInitialization(name, items) => {
                write!(f, "RecordInitialization({}, {})", name, format_list(items))
            }
            Expr::RecordUpdate(name, items) => {
                write!(f, "RecordUpdate({}, {})", name, format_list(items))
            }
            Expr::Return(expr) => write!(f, "Return({})", expr),
            Expr::Loop(pattern, start, block) => {
                write!(f, "Loop({}, {}, {})", pattern, start, format_list(block))
            }
            Expr::Continue(expr) => write!(f, "Continue({})", expr),
            Expr::Break(expr) => write!(f, "Break({})", expr),
        }
    }
}
