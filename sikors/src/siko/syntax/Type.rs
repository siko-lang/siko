use super::Identifier::Identifier;

pub enum Type {
    Named(Identifier, Vec<Type>),
    Tuple(Vec<Type>),
    Function(Vec<Type>, Box<Type>),
    SelfType,
}

pub struct TypeParameter {
    pub name: Identifier,
    pub constraints: Vec<Type>,
}
pub struct TypeParameterDeclaration {
    pub params: Vec<TypeParameter>,
    pub constraints: Vec<Type>,
}
