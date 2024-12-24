use std::{cell::RefCell, rc::Rc};

use crate::siko::location::Location::Location;

use super::{
    BlockBuilder::{BlockBuilder, Mode},
    Function::{Block, BlockId, Body, Function, Instruction, InstructionKind, Variable, VariableName},
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

    fn cloneFunction(function: &Function) -> Builder {
        let body = match function.body {
            Some(ref body) => body.clone(),
            None => Body::new(),
        };
        let blockCount = body.blocks.len();
        Builder {
            body: body,
            nextBlockId: blockCount as u32,
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

    pub fn getTypeInBody(&self, var: &Variable) -> Option<Type> {
        self.body.getType(var)
    }

    pub fn setTargetBlockId(&mut self, id: BlockId) {
        //println!("Setting target block {} => {}", self.targetBlockId, id);
        self.targetBlockId = id;
    }

    pub fn getTargetBlockId(&mut self) -> BlockId {
        self.targetBlockId
    }

    pub fn getInstruction(&self, id: BlockId, index: usize) -> Option<Instruction> {
        let irBlock = &self.body.blocks[id.id as usize];
        match irBlock.instructions.get(index) {
            Some(instruction) => Some(instruction.clone()),
            None => None,
        }
    }

    pub fn addInstruction(&mut self, id: BlockId, instruction: InstructionKind, location: Location, implicit: bool) {
        let irBlock = &mut self.body.blocks[id.id as usize];
        return irBlock.add(instruction, location, implicit);
    }

    pub fn insertInstruction(
        &mut self,
        id: BlockId,
        index: usize,
        instruction: InstructionKind,
        location: Location,
        implicit: bool,
    ) {
        let irBlock = &mut self.body.blocks[id.id as usize];
        return irBlock.insert(index, instruction, location, implicit);
    }

    pub fn replaceInstruction(
        &mut self,
        id: BlockId,
        index: usize,
        instruction: InstructionKind,
        location: Location,
        implicit: bool,
    ) {
        let irBlock = &mut self.body.blocks[id.id as usize];
        return irBlock.replace(index, instruction, location, implicit);
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

    pub fn cloneFunction(function: &Function) -> BodyBuilder {
        let bodyBuilder = Builder::cloneFunction(function);
        BodyBuilder {
            bodyBuilder: Rc::new(RefCell::new(bodyBuilder)),
        }
    }

    pub fn createBlock(&mut self) -> BlockBuilder {
        let mut bodyBuilder = self.bodyBuilder.borrow_mut();
        let blockId = bodyBuilder.createBlock();
        BlockBuilder::new(blockId, self.clone(), Mode::Append)
    }

    pub fn current(&mut self) -> BlockBuilder {
        let blockId = self.getTargetBlockId();
        BlockBuilder::new(blockId, self.clone(), Mode::Append)
    }

    pub fn block(&mut self, blockId: BlockId) -> BlockBuilder {
        BlockBuilder::new(blockId, self.clone(), Mode::Append)
    }

    pub fn iterator(&mut self, blockId: BlockId) -> BlockBuilder {
        BlockBuilder::new(blockId, self.clone(), Mode::Iterator(0))
    }

    pub fn build(&self) -> Body {
        let bodyBuilder = self.bodyBuilder.borrow();
        bodyBuilder.body.clone()
    }

    pub fn setTypeInBody(&mut self, var: Variable, ty: Type) {
        let mut bodyBuilder = self.bodyBuilder.borrow_mut();
        bodyBuilder.setTypeInBody(var, ty);
    }

    pub fn getTypeInBody(&mut self, var: &Variable) -> Option<Type> {
        let bodyBuilder = self.bodyBuilder.borrow();
        bodyBuilder.getTypeInBody(var)
    }

    pub fn setTargetBlockId(&mut self, id: BlockId) {
        let mut bodyBuilder = self.bodyBuilder.borrow_mut();
        bodyBuilder.setTargetBlockId(id);
    }

    pub fn getTargetBlockId(&mut self) -> BlockId {
        let mut bodyBuilder = self.bodyBuilder.borrow_mut();
        bodyBuilder.getTargetBlockId()
    }

    pub fn addInstruction(&mut self, id: BlockId, instruction: InstructionKind, location: Location, implicit: bool) {
        let mut bodyBuilder = self.bodyBuilder.borrow_mut();
        bodyBuilder.addInstruction(id, instruction, location, implicit);
    }

    pub fn insertInstruction(
        &mut self,
        id: BlockId,
        index: usize,
        instruction: InstructionKind,
        location: Location,
        implicit: bool,
    ) {
        let mut bodyBuilder = self.bodyBuilder.borrow_mut();
        bodyBuilder.insertInstruction(id, index, instruction, location, implicit);
    }

    pub fn replaceInstruction(
        &mut self,
        id: BlockId,
        index: usize,
        instruction: InstructionKind,
        location: Location,
        implicit: bool,
    ) {
        let mut bodyBuilder = self.bodyBuilder.borrow_mut();
        bodyBuilder.replaceInstruction(id, index, instruction, location, implicit);
    }

    pub fn sortBlocks(&mut self) {
        let mut bodyBuilder = self.bodyBuilder.borrow_mut();
        bodyBuilder.sortBlocks();
    }

    pub fn createValue(&mut self, name: &str, location: Location) -> Variable {
        let mut bodyBuilder = self.bodyBuilder.borrow_mut();
        bodyBuilder.createValue(name, location)
    }

    pub fn getInstruction(&self, id: BlockId, index: usize) -> Option<Instruction> {
        let bodyBuilder = self.bodyBuilder.borrow();
        bodyBuilder.getInstruction(id, index)
    }
}
