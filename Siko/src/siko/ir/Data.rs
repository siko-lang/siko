use crate::siko::qualifiedname::QualifiedName;

use super::Type::Type;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MethodInfo {
    pub name: String,
    pub fullName: QualifiedName,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Class {
    pub name: QualifiedName,
    pub ty: Type,
    pub fields: Vec<Field>,
    pub methods: Vec<MethodInfo>,
}

impl Class {
    pub fn new(name: QualifiedName, ty: Type) -> Class {
        Class {
            name: name,
            ty: ty,
            fields: Vec::new(),
            methods: Vec::new(),
        }
    }
}
#[derive(Clone, Debug)]
pub struct Variant {
    pub name: QualifiedName,
    pub items: Vec<Type>,
}

#[derive(Clone, Debug)]
pub struct Enum {
    pub name: QualifiedName,
    pub ty: Type,
    pub variants: Vec<Variant>,
    pub methods: Vec<MethodInfo>,
}

impl Enum {
    pub fn new(name: QualifiedName, ty: Type) -> Enum {
        Enum {
            name: name,
            ty: ty,
            variants: Vec::new(),
            methods: Vec::new(),
        }
    }
}
