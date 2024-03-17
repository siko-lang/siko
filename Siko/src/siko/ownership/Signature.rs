use crate::siko::ownership::TypeVariableInfo::TypeVariableInfo;

use super::{Allocator::Allocator, MemberInfo::MemberInfo};

#[derive(Debug, PartialEq, Eq)]
pub struct FunctionOwnershipSignature {
    pub allocator: Allocator,
    pub args: Vec<TypeVariableInfo>,
    pub result: TypeVariableInfo,
    pub members: Vec<MemberInfo>,
}

impl FunctionOwnershipSignature {
    pub fn new() -> FunctionOwnershipSignature {
        let mut allocator = Allocator::new();
        let args = Vec::new();
        let result = allocator.nextTypeVariableInfo();
        FunctionOwnershipSignature {
            allocator: allocator,
            args: args,
            result: result,
            members: Vec::new(),
        }
    }
}
