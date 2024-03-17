use std::fmt::Display;

use crate::siko::ownership::TypeVariableInfo::{GroupTypeVariable, TypeVariableInfo};

use super::{Borrow::BorrowId, TypeVariableInfo::OwnershipTypeVariable};

#[derive(Debug, PartialEq, Eq)]
pub struct Allocator {
    nextOwnershipVar: u32,
    nextGroupVar: u32,
    nextBorrowVar: u32,
}

impl Allocator {
    pub fn new() -> Allocator {
        Allocator {
            nextOwnershipVar: 0,
            nextGroupVar: 0,
            nextBorrowVar: 0,
        }
    }

    pub fn nextOwnershipVar(&mut self) -> OwnershipTypeVariable {
        let v = self.nextOwnershipVar;
        self.nextOwnershipVar += 1;
        OwnershipTypeVariable { value: v }
    }

    pub fn nextGroupVar(&mut self) -> GroupTypeVariable {
        let v = self.nextGroupVar;
        self.nextGroupVar += 1;
        GroupTypeVariable { value: v }
    }

    pub fn nextBorrowVar(&mut self) -> BorrowId {
        let v = self.nextBorrowVar;
        self.nextBorrowVar += 1;
        BorrowId { value: v }
    }

    pub fn nextTypeVariableInfo(&mut self) -> TypeVariableInfo {
        let mut tvInfo = TypeVariableInfo::new();
        tvInfo.owner = self.nextOwnershipVar();
        tvInfo.group = self.nextGroupVar();
        tvInfo
    }
}

impl Display for Allocator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "allocator({}/{}/{})",
            self.nextOwnershipVar, self.nextGroupVar, self.nextBorrowVar
        )
    }
}
