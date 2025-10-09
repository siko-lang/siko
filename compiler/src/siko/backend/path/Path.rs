use std::fmt::Debug;
use std::fmt::Display;

use crate::siko::backend::path::SimplePath::PathSegment;
use crate::siko::backend::path::SimplePath::SimplePath;
use crate::siko::hir::Block::BlockId;
use crate::siko::hir::BlockBuilder::InstructionRef;
use crate::siko::{hir::Variable::Variable, location::Location::Location};

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
            self.root.name().visibleName()
        } else {
            let items = self.items.iter().map(|i| i.to_string()).collect::<Vec<_>>();
            format!("{}.{}", self.root.name().visibleName(), items.join("."))
        }
    }

    pub fn sharesPrefixWith(&self, other: &Path) -> bool {
        if self.root.name() != other.root.name() {
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
        if self.root.name() != other.root.name() {
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
        if self.root.name() != other.root.name() {
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
            root: self.root.name(),
            items: self.items.clone(),
        }
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.items.is_empty() {
            write!(f, "{}/{}", self.root.name().visibleName(), self.instructionRef)
        } else {
            let items = self.items.iter().map(|i| i.to_string()).collect::<Vec<_>>();
            write!(
                f,
                "{}.{}/{}",
                self.root.name().visibleName(),
                items.join("."),
                self.instructionRef
            )
        }
    }
}
