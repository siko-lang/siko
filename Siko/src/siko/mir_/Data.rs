use crate::siko::qualifiedname::QualifiedName;

use super::Type::Type;

pub struct Class {
    pub name: QualifiedName,
    pub fields: Vec<Field>,
}

impl Class {
    pub fn new(name: QualifiedName) -> Class {
        Class {
            name: name,
            fields: Vec::new(),
        }
    }
}

pub struct Field {
    pub name: String,
    pub ty: Type,
}

impl Field {
    pub fn new(name: String, ty: Type) -> Field {
        Field { name: name, ty: ty }
    }
}
