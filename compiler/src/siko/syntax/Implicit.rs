use crate::siko::syntax::{Identifier::Identifier, Type::Type};

#[derive(Debug, PartialEq, Eq)]
pub struct Implicit {
    pub name: Identifier,
    pub ty: Type,
    pub mutable: bool,
    pub public: bool,
}
