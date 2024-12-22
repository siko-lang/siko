use crate::siko::{location::Location::Location, qualifiedname::QualifiedName};

use super::{
    Function::{Block, BlockId, Body, InstructionKind, Variable, VariableName},
    Type::Type,
};

pub struct BodyBuilder {
    body: Body,
    currentBlock: BlockId,
    nextBlockId: u32,
    targetBlockId: BlockId,
    valueId: u32,
}

impl BodyBuilder {
    pub fn new() -> BodyBuilder {
        BodyBuilder {
            body: Body::new(),
            currentBlock: BlockId::first(),
            nextBlockId: 0,
            targetBlockId: BlockId::first(),
            valueId: 0,
        }
    }

    pub fn createBlock(&mut self) -> BlockId {
        let blockId = BlockId { id: self.nextBlockId };
        self.nextBlockId += 1;
        let irBlock = Block::new(blockId);
        self.body.addBlock(irBlock);
        blockId
    }

    pub fn build(self) -> Body {
        self.body
    }

    pub fn setTypeInBody(&mut self, var: Variable, ty: Type) {
        self.body.setType(var, ty);
    }

    pub fn setTargetBlockId(&mut self, id: BlockId) {
        //println!("Setting target block {} => {}", self.targetBlockId, id);
        self.targetBlockId = id;
    }

    pub fn getTargetBlockId(&mut self) -> BlockId {
        self.targetBlockId
    }

    pub fn addInstruction(&mut self, instruction: InstructionKind, location: Location) {
        self.addInstructionToBlock(self.targetBlockId, instruction, location, false)
    }

    pub fn addImplicitInstruction(&mut self, instruction: InstructionKind, location: Location) {
        self.addInstructionToBlock(self.targetBlockId, instruction, location, true)
    }

    pub fn addInstructionToBlock(&mut self, id: BlockId, instruction: InstructionKind, location: Location, implicit: bool) {
        let irBlock = &mut self.body.blocks[id.id as usize];
        return irBlock.addWithImplicit(instruction, location, implicit);
    }

    pub fn sortBlocks(&mut self) {
        self.body.blocks.sort_by(|a, b| a.id.cmp(&b.id));
    }

    pub fn createValue(&mut self, name: &str, location: Location) -> Variable {
        let valueId = self.valueId;
        self.valueId += 1;
        Variable {
            value: VariableName::Local(name.to_string(), valueId),
            location: location,
            ty: None,
            index: 0,
        }
    }

    pub fn addFunctionCall(&mut self, functionName: QualifiedName, args: Vec<Variable>, location: Location) -> Variable {
        self.addFunctionCallToBlock(self.targetBlockId, functionName, args, location)
    }

    pub fn addFunctionCallToBlock(&mut self, blockId: BlockId, functionName: QualifiedName, args: Vec<Variable>, location: Location) -> Variable {
        let result = self.createValue("call", location.clone());
        self.addInstructionToBlock(
            blockId,
            InstructionKind::FunctionCall(result.clone(), functionName, args),
            location,
            false,
        );
        result
    }
}
