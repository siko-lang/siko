use super::{Function::Function, Identifier::Identifier};

pub struct Trait {
    pub name: Identifier,
    pub members: Vec<Function>,
}

pub struct Instance {}
