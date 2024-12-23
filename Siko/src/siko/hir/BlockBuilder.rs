use crate::siko::{location::Location::Location, qualifiedname::QualifiedName};

use super::{
    BodyBuilder::BodyBuilder,
    Function::{BlockId, InstructionKind, Variable},
};

pub struct BlockBuilder {
    bodyBuilder: BodyBuilder,
    blockId: BlockId,
    isImplicit: bool,
}

impl BlockBuilder {
    pub fn new(blockId: BlockId, bodyBuilder: BodyBuilder) -> BlockBuilder {
        BlockBuilder {
            bodyBuilder: bodyBuilder,
            blockId,
            isImplicit: false,
        }
    }

    pub fn current(&mut self) {
        self.bodyBuilder.setTargetBlockId(self.blockId);
    }

    pub fn implicit(&self) -> BlockBuilder {
        BlockBuilder {
            bodyBuilder: self.bodyBuilder.clone(),
            blockId: self.blockId,
            isImplicit: true,
        }
    }

    pub fn addInstruction(&mut self, instruction: InstructionKind, location: Location) {
        self.bodyBuilder
            .addInstructionToBlock(self.blockId, instruction, location, self.isImplicit)
    }

    pub fn addAssign(&mut self, target: Variable, source: Variable, location: Location) {
        self.addInstruction(InstructionKind::Assign(target, source), location);
    }

    pub fn addReturn(&mut self, value: Variable, location: Location) -> Variable {
        let retValue = self.bodyBuilder.createValue("ret", location.clone());
        self.addInstruction(InstructionKind::Return(retValue.clone(), value), location);
        retValue
    }

    pub fn addRef(&mut self, arg: Variable, location: Location) -> Variable {
        let value = self.bodyBuilder.createValue("ref", location.clone());
        self.addInstruction(InstructionKind::Ref(value.clone(), arg), location.clone());
        value
    }

    pub fn addFunctionCall(&mut self, functionName: QualifiedName, args: Vec<Variable>, location: Location) -> Variable {
        let result = self.bodyBuilder.createValue("call", location.clone());
        self.addInstruction(InstructionKind::FunctionCall(result.clone(), functionName, args), location);
        result
    }
}
