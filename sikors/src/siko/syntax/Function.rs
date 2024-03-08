use super::{Identifier::Identifier, Statement::Block, Type::Type};

pub struct Parameter {
    pub name: Identifier,
    pub ty: Type,
}
pub struct Function {
    pub name: Identifier,
    pub params: Vec<Parameter>,
    pub result: Option<Type>,
    pub body: Option<Block>,
}
