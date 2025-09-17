use crate::siko::location::Location::Location;

use super::{Identifier::Identifier, Pattern::Pattern, Statement::Block};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BinaryOp {
    And,
    Or,
    Add,
    Sub,
    Mul,
    Div,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UnaryOp {
    Not,
    Neg,
    Deref,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branch {
    pub pattern: Pattern,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextHandler {
    pub name: Identifier,
    pub handler: Identifier,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct With {
    pub handlers: Vec<ContextHandler>,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    pub expr: SimpleExpr,
    pub location: Location,
}

impl Expr {
    pub fn doesNotReturn(&self) -> bool {
        self.expr.doesNotReturn()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimpleExpr {
    Value(Identifier),
    SelfValue,
    Name(Identifier),
    FieldAccess(Box<Expr>, Identifier),
    TupleIndex(Box<Expr>, String),
    Call(Box<Expr>, Vec<Expr>),
    MethodCall(Box<Expr>, Identifier, Vec<Expr>),
    Loop(Pattern, Box<Expr>, Box<Expr>),
    BinaryOp(BinaryOp, Box<Expr>, Box<Expr>),
    UnaryOp(UnaryOp, Box<Expr>),
    Match(Box<Expr>, Vec<Branch>),
    Block(Block),
    Tuple(Vec<Expr>),
    StringLiteral(String),
    IntegerLiteral(String),
    CharLiteral(String),
    Return(Option<Box<Expr>>),
    Break(Option<Box<Expr>>),
    Continue(Option<Box<Expr>>),
    Ref(Box<Expr>),
    List(Vec<Expr>),
    With(Box<With>),
    Lambda(Vec<Pattern>, Box<Expr>),
    Yield(Box<Expr>),
    SpawnCoroutine(Box<Expr>),
}

impl SimpleExpr {
    pub fn doesNotReturn(&self) -> bool {
        match self {
            SimpleExpr::Return(_) | SimpleExpr::Break(_) | SimpleExpr::Continue(_) => true,
            SimpleExpr::FieldAccess(expr, _) => expr.doesNotReturn(),
            SimpleExpr::TupleIndex(expr, _) => expr.doesNotReturn(),
            SimpleExpr::Call(func, args) => func.doesNotReturn() || args.iter().any(|arg| arg.doesNotReturn()),
            SimpleExpr::MethodCall(obj, _, args) => obj.doesNotReturn() || args.iter().any(|arg| arg.doesNotReturn()),
            SimpleExpr::BinaryOp(_, left, right) => left.doesNotReturn() || right.doesNotReturn(),
            SimpleExpr::UnaryOp(_, expr) => expr.doesNotReturn(),
            SimpleExpr::Match(expr, branches) => {
                expr.doesNotReturn() || branches.iter().all(|branch| branch.body.doesNotReturn())
            }
            SimpleExpr::Block(block) => block.doesNotReturn(),
            SimpleExpr::Tuple(exprs) => exprs.iter().any(|expr| expr.doesNotReturn()),
            SimpleExpr::Ref(expr) => expr.doesNotReturn(),
            SimpleExpr::List(exprs) => exprs.iter().any(|expr| expr.doesNotReturn()),
            SimpleExpr::Yield(expr) => expr.doesNotReturn(),
            _ => false,
        }
    }
}
