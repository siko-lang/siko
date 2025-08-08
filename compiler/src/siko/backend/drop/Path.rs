use std::fmt::Debug;
use std::fmt::Display;

use crate::siko::hir::Type::Type;
use crate::siko::hir::Variable::VariableName;
use crate::siko::{
    hir::{Function::BlockId, Variable::Variable},
    location::Location::Location,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstructionRef {
    pub blockId: BlockId,
    pub instructionId: u32,
}

impl Display for InstructionRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.blockId, self.instructionId)
    }
}

impl Debug for InstructionRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PathSegment {
    Named(String),
    Indexed(u32),
}

impl Display for PathSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathSegment::Named(name) => write!(f, "{}", name),
            PathSegment::Indexed(index) => write!(f, "t{}", index),
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

    pub fn add(&self, item: PathSegment, location: Location) -> Path {
        let mut p = self.clone();
        p.items.push(item);
        p.location = location;
        p
    }

    pub fn setInstructionRef(&self, instructionRef: InstructionRef) -> Path {
        let mut p = self.clone();
        p.instructionRef = instructionRef;
        p
    }

    pub fn userPath(&self) -> String {
        if self.items.is_empty() {
            self.root.value.visibleName()
        } else {
            let items = self.items.iter().map(|i| i.to_string()).collect::<Vec<_>>();
            format!("{}.{}", self.root.value.visibleName(), items.join("."))
        }
    }

    pub fn sharesPrefixWith(&self, other: &Path) -> bool {
        if self.root.value != other.root.value {
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
        if self.root.value != other.root.value {
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
        if self.root.value != other.root.value {
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
            root: self.root.value.to_string(),
            items: self.items.clone(),
        }
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.items.is_empty() {
            write!(f, "{}", self.root.value.visibleName())
        } else {
            let items = self.items.iter().map(|i| i.to_string()).collect::<Vec<_>>();
            write!(f, "{}.{}", self.root.value.visibleName(), items.join("."))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimplePath {
    pub root: String,
    items: Vec<PathSegment>,
}

impl SimplePath {
    pub fn new(root: String) -> SimplePath {
        SimplePath {
            root,
            items: Vec::new(),
        }
    }

    pub fn add(&mut self, item: PathSegment) {
        self.items.push(item);
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

    pub fn getDropFlag(&self) -> Variable {
        Variable {
            value: VariableName::DropFlag(self.to_string()),
            location: Location::empty(), // Assuming a default location, adjust as needed
            ty: Some(Type::getBoolType()),
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
