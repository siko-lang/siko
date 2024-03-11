use crate::siko::qualifiedname::QualifiedName;

use super::Type::Type;

pub struct Field {
    pub name: String,
    pub ty: Type,
}
pub struct Class {
    pub name: QualifiedName,
    pub fields: Vec<Field>,
}
