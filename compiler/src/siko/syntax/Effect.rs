use crate::siko::syntax::{Function::Function, Identifier::Identifier};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Effect {
    pub name: Identifier,
    pub methods: Vec<Function>,
    pub public: bool,
}
