use super::{
    Identifier::Identifier,
    Statement::Block,
    Type::{Type, TypeParameterDeclaration},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Parameter {
    Named(Identifier, Type, bool),
    SelfParam(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: Identifier,
    pub typeParams: Option<TypeParameterDeclaration>,
    pub params: Vec<Parameter>,
    pub result: Option<Type>,
    pub body: Option<Block>,
    pub isExtern: bool,
}
