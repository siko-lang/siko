use std::{cell::RefCell, rc::Rc};

use crate::siko::location::Location::Location;

use super::{
    BlockBuilder::BlockBuilder,
    Function::{Block, BlockId, Body, InstructionKind, Variable, VariableName},
    Type::Type,
};

struct Builder {
    body: Body,
    nextBlockId: u32,
    targetBlockId: BlockId,
    valueId: u32,
}

impl Builder {
    fn new() -> Builder {
        Builder {
            body: Body::new(),
            nextBlockId: 0,
            targetBlockId: BlockId::first(),
            valueId: 0,
        }
    }

    fn createBlock(&mut self) -> BlockId {
        let blockId = BlockId { id: self.nextBlockId };
        self.nextBlockId += 1;
        let irBlock = Block::new(blockId);
        self.body.addBlock(irBlock);
        blockId
    }

    fn setTypeInBody(&mut self, var: Variable, ty: Type) {
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

    pub fn setImplicit(&mut self) {
        let irBlock = &mut self.body.blocks[self.targetBlockId.id as usize];
        irBlock.instructions.last_mut().unwrap().implicit = true;
    }
}

#[derive(Clone)]
pub struct BodyBuilder {
    bodyBuilder: Rc<RefCell<Builder>>,
}

impl BodyBuilder {
    pub fn new() -> BodyBuilder {
        BodyBuilder {
            bodyBuilder: Rc::new(RefCell::new(Builder::new())),
        }
    }

    pub fn createBlock(&mut self) -> BlockId {
        let mut bodyBuilder = self.bodyBuilder.borrow_mut();
        let blockId = bodyBuilder.createBlock();
        blockId
    }

    pub fn createBlock2(&mut self) -> BlockBuilder {
        let id = self.createBlock();
        BlockBuilder::new(id, self.clone())
    }

    pub fn current(&mut self) -> BlockBuilder {
        let blockId = self.getTargetBlockId();
        BlockBuilder::new(blockId, self.clone())
    }

    pub fn block(&mut self, blockId: BlockId) -> BlockBuilder {
        BlockBuilder::new(blockId, self.clone())
    }

    pub fn build(self) -> Body {
        let bodyBuilder = self.bodyBuilder.borrow();
        bodyBuilder.body.clone()
    }

    pub fn setTypeInBody(&mut self, var: Variable, ty: Type) {
        let mut bodyBuilder = self.bodyBuilder.borrow_mut();
        bodyBuilder.setTypeInBody(var, ty);
    }

    pub fn setTargetBlockId(&mut self, id: BlockId) {
        let mut bodyBuilder = self.bodyBuilder.borrow_mut();
        bodyBuilder.setTargetBlockId(id);
    }

    pub fn getTargetBlockId(&mut self) -> BlockId {
        let mut bodyBuilder = self.bodyBuilder.borrow_mut();
        bodyBuilder.getTargetBlockId()
    }

    pub fn addInstruction(&mut self, instruction: InstructionKind, location: Location) {
        let mut bodyBuilder = self.bodyBuilder.borrow_mut();
        bodyBuilder.addInstruction(instruction, location);
    }

    pub fn addImplicitInstruction(&mut self, instruction: InstructionKind, location: Location) {
        let mut bodyBuilder = self.bodyBuilder.borrow_mut();
        bodyBuilder.addImplicitInstruction(instruction, location);
    }

    pub fn addInstructionToBlock(&mut self, id: BlockId, instruction: InstructionKind, location: Location, implicit: bool) {
        let mut bodyBuilder = self.bodyBuilder.borrow_mut();
        bodyBuilder.addInstructionToBlock(id, instruction, location, implicit);
    }

    pub fn sortBlocks(&mut self) {
        let mut bodyBuilder = self.bodyBuilder.borrow_mut();
        bodyBuilder.sortBlocks();
    }

    pub fn createValue(&mut self, name: &str, location: Location) -> Variable {
        let mut bodyBuilder = self.bodyBuilder.borrow_mut();
        bodyBuilder.createValue(name, location)
    }

    pub fn setImplicit(&mut self) {
        let mut bodyBuilder = self.bodyBuilder.borrow_mut();
        bodyBuilder.setImplicit();
    }
}
