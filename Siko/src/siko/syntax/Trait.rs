use super::{
    Function::Function,
    Identifier::Identifier,
    Type::{Type, TypeParameterDeclaration},
};

pub struct Trait {
    pub name: Identifier,
    pub typeParams: Option<TypeParameterDeclaration>,
    pub members: Vec<Function>,
}

pub struct Instance {
    pub typeParams: Option<TypeParameterDeclaration>,
    pub ty: Type,
    pub members: Vec<Function>,
}
