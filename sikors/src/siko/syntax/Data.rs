use super::{
    Identifier::Identifier,
    Module::Derive,
    Type::{Type, TypeParameterDeclaration},
};

pub struct Class {
    pub name: Identifier,
    pub typeParams: Option<TypeParameterDeclaration>,
    pub isExtern: bool,
    pub fields: Vec<Field>,
    pub derives: Vec<Derive>,
}

pub struct Enum {
    pub name: Identifier,
    pub variants: Vec<Variant>,
    pub derives: Vec<Derive>,
}

pub struct Variant {
    pub name: Identifier,
    pub items: Vec<Type>,
}

pub struct Field {
    pub name: Identifier,
    pub ty: Type,
}
