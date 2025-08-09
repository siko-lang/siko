use std::fmt::Debug;
use std::fmt::Display;
use std::vec;

use crate::siko::hir::BlockBuilder::InstructionRef;
use crate::siko::hir::Type::Type;
use crate::siko::hir::Variable::VariableName;
use crate::siko::{
    hir::{Function::BlockId, Variable::Variable},
    location::Location::Location,
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
pub struct Path {
    pub root: Variable,
    pub items: Vec<PathSegment>,
    pub location: Location,
    pub instructionRef: InstructionRef,
}

impl Path {
    pub fn new(root: Variable, location: Location) -> Path {
        Path {
            root: root,
            items: Vec::new(),
            location: location,
            instructionRef: InstructionRef {
                blockId: BlockId::first(),
                instructionId: 0,
            },
        }
    }

    pub fn add(&self, item: PathSegment) -> Path {
        let mut p = self.clone();
        p.items.push(item);
        p
    }

    pub fn setInstructionRef(&self, instructionRef: InstructionRef) -> Path {
        let mut p = self.clone();
        p.instructionRef = instructionRef;
        p
    }

    pub fn userPath(&self) -> String {
        if self.items.is_empty() {
            self.root.name.visibleName()
        } else {
            let items = self.items.iter().map(|i| i.to_string()).collect::<Vec<_>>();
            format!("{}.{}", self.root.name.visibleName(), items.join("."))
        }
    }

    pub fn sharesPrefixWith(&self, other: &Path) -> bool {
        if self.root.name != other.root.name {
            return false;
        }
        for (i1, i2) in self.items.iter().zip(other.items.iter()) {
            if i1 != i2 {
                return false;
            }
        }
        true
    }

    pub fn same(&self, other: &Path) -> bool {
        if self.root.name != other.root.name {
            return false;
        }
        if self.items.len() != other.items.len() {
            return false;
        }
        for (i1, i2) in self.items.iter().zip(other.items.iter()) {
            if i1 != i2 {
                return false;
            }
        }
        true
    }

    pub fn contains(&self, other: &Path) -> bool {
        if self.root.name != other.root.name {
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

    pub fn isRootOnly(&self) -> bool {
        self.items.is_empty()
    }

    pub fn toSimplePath(&self) -> SimplePath {
        SimplePath {
            root: self.root.name.to_string(),
            items: self.items.clone(),
        }
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.items.is_empty() {
            write!(f, "{}", self.root.name.visibleName())
        } else {
            let items = self.items.iter().map(|i| i.to_string()).collect::<Vec<_>>();
            write!(f, "{}.{}", self.root.name.visibleName(), items.join("."))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimplePath {
    pub root: String,
    pub items: Vec<PathSegment>,
}

impl SimplePath {
    pub fn new(root: String) -> SimplePath {
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

    pub fn getDropFlag(&self) -> Variable {
        Variable {
            name: VariableName::DropFlag(self.to_string()),
            location: Location::empty(), // Assuming a default location, adjust as needed
            ty: Some(Type::getBoolType()),
        }
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
