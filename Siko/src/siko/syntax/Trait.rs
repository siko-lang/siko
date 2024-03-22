use crate::siko::location::Location::Location;

use super::{
    Function::Function,
    Identifier::Identifier,
    Type::{Type, TypeParameterDeclaration},
};

#[derive(Debug, PartialEq, Eq)]
pub struct Trait {
    pub name: Identifier,
    pub params: Vec<Identifier>,
    pub deps: Vec<Identifier>,
    pub typeParams: Option<TypeParameterDeclaration>,
    pub methods: Vec<Function>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Instance {
    pub id: u64,
    pub typeParams: Option<TypeParameterDeclaration>,
    pub ty: Type,
    pub methods: Vec<Function>,
    pub location: Location,
}
