use std::fmt;

use crate::siko::location::Location::Location;

use super::{
    Function::Function,
    Identifier::Identifier,
    Type::{Constraint, Type, TypeParameterDeclaration},
};

#[derive(Debug, PartialEq, Eq)]
pub struct Trait {
    pub name: Identifier,
    pub params: Vec<Identifier>,
    pub typeParams: Option<TypeParameterDeclaration>,
    pub associatedTypes: Vec<AssociatedTypeDeclaration>,
    pub methods: Vec<Function>,
    pub public: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AssociatedTypeDeclaration {
    pub name: Identifier,
    pub constraints: Vec<Constraint>,
}

impl fmt::Display for AssociatedTypeDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.constraints.is_empty() {
            write!(f, "type {}", self.name)
        } else {
            let constraints = self
                .constraints
                .iter()
                .map(|constraint| format!("{}", constraint))
                .collect::<Vec<_>>()
                .join(", ");
            write!(f, "type {}: {}", self.name, constraints)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct AssociatedType {
    pub name: Identifier,
    pub ty: Type,
}

impl fmt::Display for AssociatedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type {} = {}", self.name, self.ty)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Instance {
    pub public: bool,
    pub name: Option<Identifier>,
    pub typeParams: Option<TypeParameterDeclaration>,
    pub traitName: Identifier,
    pub types: Vec<Type>,
    pub associatedTypes: Vec<AssociatedType>,
    pub methods: Vec<Function>,
    pub location: Location,
}
