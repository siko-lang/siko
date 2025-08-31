use std::{
    collections::{BTreeMap, VecDeque},
    fmt::Display,
};

use crate::siko::hir::{
    Block::{Block, BlockId},
    Instruction::Instruction,
    Variable::CopyMap,
    VariableAllocator::VariableAllocator,
};

#[derive(Debug, Clone)]
pub struct Body {
    pub blocks: BTreeMap<BlockId, Block>,
    pub varAllocator: VariableAllocator,
}

impl Body {
    pub fn new() -> Body {
        Body {
            blocks: BTreeMap::new(),
            varAllocator: VariableAllocator::new(),
        }
    }

    pub fn copy(&self) -> Body {
        let mut copyMap = CopyMap::new();
        let mut blocks = BTreeMap::new();
        for (id, block) in &self.blocks {
            blocks.insert(*id, block.copy(&mut copyMap));
        }
        Body {
            blocks,
            varAllocator: self.varAllocator.copy(),
        }
    }

    pub fn addBlock(&mut self, block: Block) {
        self.blocks.insert(block.getId(), block);
    }

    pub fn getBlockById(&self, id: BlockId) -> &Block {
        &self.blocks.get(&id).expect("Block not found")
    }

    pub fn dump(&self) {
        for (_, block) in &self.blocks {
            block.dump();
        }
    }

    pub fn getInstruction(&self, blockId: BlockId, index: usize) -> Option<Instruction> {
        if let Some(block) = self.blocks.get(&blockId) {
            return block.getInstructionOpt(index);
        }
        None
    }

    pub fn getAllBlockIds(&self) -> VecDeque<BlockId> {
        let mut ids = VecDeque::new();
        for (id, _) in &self.blocks {
            ids.push_back(*id);
        }
        ids
    }

    pub fn cutBlock(&mut self, blockId: BlockId, index: usize, newBlockId: BlockId) {
        let block = self.blocks.get_mut(&blockId).expect("Block not found");
        let inner = block.getInner();
        let otherInstructions = inner.borrow_mut().instructions.split_off(index);
        let newBlock = self.blocks.get_mut(&newBlockId).expect("New block not found");
        let newInner = newBlock.getInner();
        newInner.borrow_mut().instructions = otherInstructions;
    }

    pub fn getBlockSize(&self, blockId: BlockId) -> usize {
        self.blocks
            .get(&blockId)
            .expect("Block not found")
            .getInner()
            .borrow()
            .instructions
            .len()
    }

    pub fn removeBlock(&mut self, block_id: BlockId) {
        self.blocks.remove(&block_id);
    }

    pub fn mergeBlocks(&mut self, sourceBlockId: BlockId, targetBlockId: BlockId) {
        let targetBlock = self.blocks.remove(&targetBlockId).expect("Target block not found");
        let sourceBlock = self.blocks.get_mut(&sourceBlockId).expect("Source block not found");
        let srcInner = sourceBlock.getInner();
        srcInner.borrow_mut().instructions.pop();
        srcInner
            .borrow_mut()
            .instructions
            .append(&mut targetBlock.getInner().borrow_mut().instructions);
    }
}

impl Display for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (_, block) in &self.blocks {
            write!(f, "{}", block)?;
        }
        Ok(())
    }
}
