use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use crate::siko::location::Location::Location;

use super::{
    BlockBuilder::{BlockBuilder, Mode},
    Function::{Block, BlockId, Body, Function},
    Instruction::{Instruction, InstructionKind},
    Type::Type,
    Variable::{Variable, VariableName},
};

struct Builder {
    body: Body,
    nextBlockId: u32,
    targetBlockId: BlockId,
    nextId: u32,
}

impl Builder {
    fn new() -> Builder {
        Builder {
            body: Body::new(),
            nextBlockId: 0,
            targetBlockId: BlockId::first(),
            nextId: 0,
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
            nextId: 0,
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

    pub fn getAllBlockIds(&self) -> VecDeque<BlockId> {
        self.body.getAllBlockIds()
    }

    pub fn setTargetBlockId(&mut self, id: BlockId) {
        //println!("Setting target block {} => {}", self.targetBlockId, id);
        self.targetBlockId = id;
    }

    pub fn getTargetBlockId(&self) -> BlockId {
        self.targetBlockId
    }

    pub fn getInstruction(&self, id: BlockId, index: usize) -> Option<Instruction> {
        let irBlock = self.body.getBlockById(id);
        match irBlock.instructions.get(index) {
            Some(instruction) => Some(instruction.clone()),
            None => None,
        }
    }

    pub fn addInstruction(&mut self, id: BlockId, instruction: InstructionKind, location: Location, implicit: bool) {
        let irBlock = self.body.blocks.get_mut(&id).expect("Block not found");
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
        let irBlock = self.body.blocks.get_mut(&id).expect("Block not found");
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
        let irBlock = self.body.blocks.get_mut(&id).expect("Block not found");
        return irBlock.replace(index, instruction, location, implicit);
    }

    pub fn removeInstruction(&mut self, id: BlockId, index: usize) {
        let irBlock = self.body.blocks.get_mut(&id).expect("Block not found");
        irBlock.remove(index);
    }

    pub fn getNextId(&mut self) -> u32 {
        let id = self.nextId;
        self.nextId += 1;
        id
    }

    pub fn createLocalValue(&mut self, name: &str, location: Location) -> Variable {
        let valueId = self.getNextId();
        Variable {
            name: VariableName::Local(name.to_string(), valueId),
            location: location,
            ty: None,
        }
    }

    pub fn createTempValue(&mut self, location: Location) -> Variable {
        self.body.varAllocator.allocate(location)
    }

    pub fn getLastInstruction(&self, block_id: BlockId) -> Option<Instruction> {
        let block = self.body.getBlockById(block_id);
        block.instructions.last().cloned()
    }

    pub fn isValid(&self, blockId: BlockId) -> bool {
        self.body.blocks.contains_key(&blockId)
    }

    fn withBody(body: Body) -> Builder {
        Builder {
            body,
            nextBlockId: 0,
            targetBlockId: BlockId::first(),
            nextId: 0,
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

    pub fn withBody(body: Body) -> BodyBuilder {
        BodyBuilder {
            bodyBuilder: Rc::new(RefCell::new(Builder::withBody(body))),
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

    pub fn getTargetBlockId(&self) -> BlockId {
        let bodyBuilder = self.bodyBuilder.borrow();
        bodyBuilder.getTargetBlockId()
    }

    pub fn getAllBlockIds(&self) -> VecDeque<BlockId> {
        let bodyBuilder = self.bodyBuilder.borrow();
        bodyBuilder.getAllBlockIds()
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

    pub fn removeInstruction(&mut self, id: BlockId, index: usize) {
        let mut bodyBuilder = self.bodyBuilder.borrow_mut();
        bodyBuilder.removeInstruction(id, index);
    }

    pub fn createLocalValue(&mut self, name: &str, location: Location) -> Variable {
        let mut bodyBuilder = self.bodyBuilder.borrow_mut();
        bodyBuilder.createLocalValue(name, location)
    }

    pub fn createTempValue(&mut self, location: Location) -> Variable {
        let mut bodyBuilder = self.bodyBuilder.borrow_mut();
        bodyBuilder.createTempValue(location)
    }

    pub fn getInstruction(&self, id: BlockId, index: usize) -> Option<Instruction> {
        let bodyBuilder = self.bodyBuilder.borrow();
        bodyBuilder.getInstruction(id, index)
    }

    pub fn cutBlock(&self, blockId: BlockId, index: usize) -> BlockId {
        let mut bodyBuilder = self.bodyBuilder.borrow_mut();
        let newBlock = bodyBuilder.createBlock();
        bodyBuilder.body.cutBlock(blockId, index, newBlock);
        newBlock
    }

    pub fn getBlockSize(&self, blockId: BlockId) -> usize {
        let bodyBuilder = self.bodyBuilder.borrow();
        bodyBuilder.body.getBlockSize(blockId)
    }

    pub fn removeBlock(&mut self, blockId: BlockId) {
        let mut bodyBuilder = self.bodyBuilder.borrow_mut();
        bodyBuilder.body.removeBlock(blockId);
    }

    pub fn mergeBlocks(&mut self, sourceBlockId: BlockId, targetBlockId: BlockId) {
        let mut bodyBuilder = self.bodyBuilder.borrow_mut();
        bodyBuilder.body.mergeBlocks(sourceBlockId, targetBlockId);
    }

    pub fn getLastInstruction(&self, blockId: BlockId) -> Option<Instruction> {
        let bodyBuilder = self.bodyBuilder.borrow();
        bodyBuilder.getLastInstruction(blockId)
    }

    pub fn isValid(&self, blockId: BlockId) -> bool {
        let bodyBuilder = self.bodyBuilder.borrow();
        bodyBuilder.isValid(blockId)
    }
}
