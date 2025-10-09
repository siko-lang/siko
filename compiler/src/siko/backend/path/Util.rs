use crate::siko::{
    backend::path::{Path::Path, SimplePath::buildSegments},
    hir::{Instruction::FieldInfo, Type::Type, Variable::Variable},
};

pub fn buildFieldPath(root: &Variable, fields: &Vec<FieldInfo>) -> Path {
    let mut path = Path::new(root.clone(), root.location().clone());
    let segments = buildSegments(fields);
    for segment in segments {
        path = path.add(segment);
    }
    path
}

pub trait HasTrivialDrop {
    fn hasTrivialDrop(&self) -> bool;
}

impl HasTrivialDrop for Type {
    fn hasTrivialDrop(&self) -> bool {
        match self {
            _ => {
                self.isNever()
                    || self.isPtr()
                    || self.isReference()
                    || self.isUnit()
                    || self.isVoid()
                    || self.isVoidPtr()
                    || self.isFunctionPtr()
            }
        }
    }
}

impl HasTrivialDrop for Variable {
    fn hasTrivialDrop(&self) -> bool {
        self.getType().hasTrivialDrop()
    }
}
