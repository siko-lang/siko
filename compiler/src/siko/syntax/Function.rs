use super::{
    Identifier::Identifier,
    Statement::Block,
    Type::{Type, TypeParameterDeclaration},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Parameter {
    Named(Identifier, Type, bool),
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
pub struct Attributes {
    pub testEntry: bool,
}

impl Attributes {
    pub fn new() -> Self {
        Attributes { testEntry: false }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: Identifier,
    pub typeParams: Option<TypeParameterDeclaration>,
    pub params: Vec<Parameter>,
    pub result: Type,
    pub body: Option<Block>,
    pub externKind: Option<FunctionExternKind>,
    pub public: bool,
    pub attributes: Attributes,
}
