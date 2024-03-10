use super::{
    Function::Function,
    Identifier::Identifier,
    Module::Derive,
    Type::{Type, TypeParameterDeclaration},
};

pub struct Class {
    pub name: Identifier,
    pub typeParams: Option<TypeParameterDeclaration>,
    pub isExtern: bool,
    pub fields: Vec<Field>,
    pub methods: Vec<Function>,
    pub derives: Vec<Derive>,
    pub hasImplicitMember: bool,
}

pub struct Enum {
    pub name: Identifier,
    pub typeParams: Option<TypeParameterDeclaration>,
    pub variants: Vec<Variant>,
    pub methods: Vec<Function>,
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
