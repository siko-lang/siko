use super::{Identifier::Identifier, Type::Type};

pub struct Parameter {
    pub name: Identifier,
    pub ty: Type,
}
pub struct Function {
    pub name: Identifier,
    pub params: Vec<Parameter>,
    pub result: Option<Type>,
}
