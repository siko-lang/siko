use std::fmt;

use crate::siko::{hir::Type::formatTypes, qualifiedname::QualifiedName};

use super::{ConstraintContext::ConstraintContext, Type::Type};

#[derive(Debug)]
pub struct MethodInfo {
    pub name: String,
    pub fullName: QualifiedName,
}

impl fmt::Display for MethodInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fn {} => ({})", self.name, self.fullName)
    }
}

#[derive(Debug)]
pub struct Trait {
    pub name: QualifiedName,
    pub params: Vec<Type>,
    pub associatedTypes: Vec<String>,
    pub methods: Vec<MethodInfo>,
}

impl Trait {
    pub fn new(name: QualifiedName, params: Vec<Type>, associatedTypes: Vec<String>) -> Trait {
        Trait {
            name: name,
            params: params,
            associatedTypes: associatedTypes,
            methods: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct AssociatedType {
    pub name: String,
    pub ty: Type,
}

impl fmt::Display for AssociatedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type {} = {}", self.name, self.ty)
    }
}

#[derive(Debug)]
pub struct Instance {
    pub id: u64,
    pub traitName: QualifiedName,
    pub types: Vec<Type>,
    pub associatedTypes: Vec<AssociatedType>,
    pub constraintContext: ConstraintContext,
    pub methods: Vec<MethodInfo>,
}

impl Instance {
    pub fn new(
        id: u64,
        traitName: QualifiedName,
        types: Vec<Type>,
        associatedTypes: Vec<AssociatedType>,
        constraintContext: ConstraintContext,
    ) -> Instance {
        Instance {
            id: id,
            traitName: traitName,
            types: types,
            associatedTypes: associatedTypes,
            constraintContext: constraintContext,
            methods: Vec::new(),
        }
    }
}

impl fmt::Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let methods = self.methods.iter().map(|m| format!("{}", m)).collect::<Vec<_>>().join(",\n    ");
        let associatedTypes = self.associatedTypes.iter().map(|m| format!("{}", m)).collect::<Vec<_>>().join(",\n    ");
        write!(
            f,
            "instance #{} of {} [{}] {} {{\n    {}\n    {}\n}}",
            self.id,
            self.traitName,
            formatTypes(&self.types),
            self.constraintContext,
            associatedTypes,
            methods
        )
    }
}
