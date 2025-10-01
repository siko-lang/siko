use crate::siko::syntax::{Attributes::Attributes, Expr::Expr, Identifier::Identifier, Type::Type};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Global {
    pub name: Identifier,
    pub ty: Type,
    pub value: Expr,
    pub public: bool,
    pub attributes: Attributes,
}
