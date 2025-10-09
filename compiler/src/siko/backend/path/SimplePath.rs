use std::fmt::Display;

use crate::siko::hir::{
    Instruction::{FieldId, FieldInfo},
    Type::Type,
    Variable::VariableName,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PathSegment {
    Named(String, Type),
    Indexed(u32, Type),
}

impl PathSegment {
    pub fn getType(&self) -> &Type {
        match self {
            PathSegment::Named(_, ty) => ty,
            PathSegment::Indexed(_, ty) => ty,
        }
    }
}

impl Display for PathSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathSegment::Named(name, _) => write!(f, "{}", name),
            PathSegment::Indexed(index, _) => write!(f, "{}", index),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimplePath {
    pub root: VariableName,
    pub items: Vec<PathSegment>,
}

impl SimplePath {
    pub fn new(root: VariableName) -> SimplePath {
        SimplePath {
            root,
            items: Vec::new(),
        }
    }

    pub fn add(&self, item: PathSegment) -> SimplePath {
        let mut new_path = self.clone();
        new_path.items.push(item);
        new_path
    }

    pub fn sharesPrefixWith(&self, other: &SimplePath) -> bool {
        if self.root != other.root {
            return false;
        }
        for (i1, i2) in self.items.iter().zip(other.items.iter()) {
            if i1 != i2 {
                return false;
            }
        }
        true
    }

    pub fn contains(&self, other: &SimplePath) -> bool {
        if self.root != other.root {
            return false;
        }
        if self.items.len() < other.items.len() {
            return false;
        }
        for (i1, i2) in self.items.iter().zip(other.items.iter()) {
            if i1 != i2 {
                return false;
            }
        }
        true
    }

    pub fn getRootPath(&self) -> SimplePath {
        SimplePath {
            root: self.root.clone(),
            items: vec![],
        }
    }

    pub fn getParent(&self) -> Option<SimplePath> {
        if self.items.is_empty() {
            None
        } else {
            Some(SimplePath {
                root: self.root.clone(),
                items: self.items[..self.items.len() - 1].to_vec(),
            })
        }
    }
}

impl Display for SimplePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.items.is_empty() {
            write!(f, "{}", self.root)
        } else {
            let items = self.items.iter().map(|i| i.to_string()).collect::<Vec<_>>();
            write!(f, "{}.{}", self.root, items.join("."))
        }
    }
}

pub fn buildSegments(fields: &Vec<FieldInfo>) -> Vec<PathSegment> {
    let mut segments = Vec::new();
    for field in fields {
        match &field.name {
            FieldId::Named(name) => {
                segments.push(PathSegment::Named(
                    name.clone(),
                    field.ty.clone().expect("fieldid without type"),
                ));
            }
            FieldId::Indexed(index) => {
                segments.push(PathSegment::Indexed(
                    *index,
                    field.ty.clone().expect("fieldid without type"),
                ));
            }
        }
    }
    segments
}
