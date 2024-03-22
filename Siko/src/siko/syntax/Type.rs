use super::Identifier::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Named(Identifier, Vec<Type>),
    Tuple(Vec<Type>),
    Function(Vec<Type>, Box<Type>),
    SelfType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeParameter {
    pub name: Identifier,
    pub constraints: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeParameterDeclaration {
    pub params: Vec<TypeParameter>,
    pub constraints: Vec<Type>,
}
