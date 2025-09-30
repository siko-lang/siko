use crate::siko::syntax::{Attributes::Attributes, Expr::Expr};

use super::{
    Identifier::Identifier,
    Statement::Block,
    Type::{Type, TypeParameterDeclaration},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Parameter {
    Named(Identifier, Type, bool, Option<Expr>),
    SelfParam,
    MutSelfParam,
    RefSelfParam,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionExternKind {
    Builtin,
    C(Option<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResultKind {
    SingleReturn(Type),
    Coroutine(Type),
}

impl ResultKind {
    pub fn assertSingleReturn(&self) -> &Type {
        match self {
            ResultKind::SingleReturn(ty) => ty,
            ResultKind::Coroutine(_) => {
                panic!("Expected single return type, found coroutine type.")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: Identifier,
    pub typeParams: Option<TypeParameterDeclaration>,
    pub params: Vec<Parameter>,
    pub result: ResultKind,
    pub body: Option<Block>,
    pub externKind: Option<FunctionExternKind>,
    pub public: bool,
    pub attributes: Attributes,
}
