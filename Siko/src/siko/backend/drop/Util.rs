use crate::siko::{
    backend::drop::Path::{Path, PathSegment},
    hir::{
        Instruction::{FieldId, FieldInfo},
        Variable::Variable,
    },
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
