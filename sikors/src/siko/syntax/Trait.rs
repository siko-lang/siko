use super::{Function::Function, Identifier::Identifier, Type::TypeParameterDeclaration};

pub struct Trait {
    pub name: Identifier,
    pub typeParams: Option<TypeParameterDeclaration>,
    pub members: Vec<Function>,
}

pub struct Instance {}
