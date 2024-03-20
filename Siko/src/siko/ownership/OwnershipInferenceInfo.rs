use std::collections::BTreeMap;

use crate::siko::ir::Function::InstructionId;

use super::{
    Borrow::BorrowId,
    MemberInfo::MemberInfo,
    Signature::FunctionOwnershipSignature,
    TypeVariableInfo::{GroupTypeVariable, OwnershipTypeVariable, TypeVariableInfo},
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypedId {
    Instruction(InstructionId),
    Value(String),
}

pub struct OwnershipInferenceInfo {
    pub signature: FunctionOwnershipSignature,
    pub tvInfos: BTreeMap<TypedId, TypeVariableInfo>,
    pub members: BTreeMap<InstructionId, Vec<MemberInfo>>,
}

impl OwnershipInferenceInfo {
    pub fn new() -> OwnershipInferenceInfo {
        OwnershipInferenceInfo {
            signature: FunctionOwnershipSignature::new(),
            tvInfos: BTreeMap::new(),
            members: BTreeMap::new(),
        }
    }

    pub fn nextOwnershipVar(&mut self) -> OwnershipTypeVariable {
        self.signature.allocator.nextOwnershipVar()
    }

    pub fn nextGroupVar(&mut self) -> GroupTypeVariable {
        self.signature.allocator.nextGroupVar()
    }

    pub fn nextBorrowVar(&mut self) -> BorrowId {
        self.signature.allocator.nextBorrowVar()
    }

    pub fn nextTypeVariableInfo(&mut self) -> TypeVariableInfo {
        self.signature.allocator.nextTypeVariableInfo()
    }
}
