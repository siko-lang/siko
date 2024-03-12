use crate::siko::qualifiedname::QualifiedName;

use super::Type::Type;

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug)]
pub struct MethodInfo {
    pub name: String,
    pub fullName: QualifiedName,
}

#[derive(Debug)]
pub struct Class {
    pub name: QualifiedName,
    pub fields: Vec<Field>,
    pub methods: Vec<MethodInfo>,
}

impl Class {
    pub fn new(name: QualifiedName) -> Class {
        Class {
            name: name,
            fields: Vec::new(),
            methods: Vec::new(),
        }
    }
}
#[derive(Debug)]
pub struct Variant {
    pub name: QualifiedName,
    pub items: Vec<Type>,
}

#[derive(Debug)]
pub struct Enum {
    pub name: QualifiedName,
    pub variants: Vec<Variant>,
    pub methods: Vec<MethodInfo>,
}

impl Enum {
    pub fn new(name: QualifiedName) -> Enum {
        Enum {
            name: name,
            variants: Vec::new(),
            methods: Vec::new(),
        }
    }
}
