use super::{
    Function::Function,
    Identifier::Identifier,
    Module::Derive,
    Type::{Type, TypeParameterDeclaration},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Struct {
    pub name: Identifier,
    pub typeParams: Option<TypeParameterDeclaration>,
    pub isExtern: bool,
    pub fields: Vec<Field>,
    pub methods: Vec<Function>,
    pub derives: Vec<Derive>,
    pub hasImplicitMember: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Enum {
    pub name: Identifier,
    pub typeParams: Option<TypeParameterDeclaration>,
    pub variants: Vec<Variant>,
    pub methods: Vec<Function>,
    pub derives: Vec<Derive>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variant {
    pub name: Identifier,
    pub items: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: Identifier,
    pub ty: Type,
}
