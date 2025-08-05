use crate::siko::{
    backend::drop::Path::{Path, PathSegment},
    hir::{
        Instruction::{FieldId, FieldInfo},
        Type::Type,
        Variable::Variable,
    },
    qualifiedname::{getBoolTypeName, getIntTypeName},
};

pub fn buildFieldPath(root: &Variable, fields: &Vec<FieldInfo>) -> Path {
    let mut path = Path::new(root.clone(), root.location.clone());
    for field in fields {
        match &field.name {
            FieldId::Named(name) => {
                path = path.add(PathSegment::Named(name.clone()), field.location.clone());
            }
            FieldId::Indexed(index) => {
                path = path.add(PathSegment::Indexed(*index), field.location.clone());
            }
        }
    }
    path
}

pub trait HasTrivialDrop {
    fn hasTrivialDrop(&self) -> bool;
}

impl HasTrivialDrop for Type {
    fn hasTrivialDrop(&self) -> bool {
        match self {
            Type::Named(name, _) => getIntTypeName() == *name || getBoolTypeName() == *name, // TODO: make this more generic
            _ => self.isNever() || self.isPtr() || self.isReference() || self.isUnit(),
        }
    }
}

impl HasTrivialDrop for Variable {
    fn hasTrivialDrop(&self) -> bool {
        self.ty
            .as_ref()
            .expect("Variable type should be present")
            .hasTrivialDrop()
    }
}
