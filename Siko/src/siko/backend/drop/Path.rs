use std::fmt::Display;

use crate::siko::{
    hir::{Function::BlockId, Variable::Variable},
    location::Location::Location,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstructionRef {
    pub blockId: BlockId,
    pub instructionId: u32,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Path {
    pub root: Variable,
    pub items: Vec<String>,
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

    pub fn add(&self, item: String, location: Location) -> Path {
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
            format!("{}.{}", self.root.value.visibleName(), self.items.join("."))
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
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.items.is_empty() {
            write!(f, "{}", self.root.value.visibleName())
        } else {
            write!(f, "{}.{}", self.root.value.visibleName(), self.items.join("."))
        }
    }
}
