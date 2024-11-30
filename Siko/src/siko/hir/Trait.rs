use crate::siko::qualifiedname::QualifiedName;

use super::Type::Type;

#[derive(Debug)]
pub struct MethodInfo {
    pub name: String,
    pub fullName: QualifiedName,
}

#[derive(Debug)]
pub struct Trait {
    pub name: QualifiedName,
    pub params: Vec<Type>,
    pub methods: Vec<MethodInfo>,
}

impl Trait {
    pub fn new(name: QualifiedName, params: Vec<Type>) -> Trait {
        Trait {
            name: name,
            params: params,
            methods: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct Instance {
    pub id: u64,
    pub traitName: QualifiedName,
    pub types: Vec<Type>,
    pub methods: Vec<MethodInfo>,
}

impl Instance {
    pub fn new(id: u64, traitName: QualifiedName, types: Vec<Type>) -> Instance {
        Instance {
            id: id,
            traitName: traitName,
            types: types,
            methods: Vec::new(),
        }
    }
}
