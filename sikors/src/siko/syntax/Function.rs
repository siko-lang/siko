use super::{
    Identifier::Identifier,
    Statement::Block,
    Type::{Type, TypeParameterDeclaration},
};

pub enum Parameter {
    Named(Identifier, Type),
    SelfParam,
}

pub struct Function {
    pub name: Identifier,
    pub typeParams: Option<TypeParameterDeclaration>,
    pub params: Vec<Parameter>,
    pub result: Option<Type>,
    pub body: Option<Block>,
}
