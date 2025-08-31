use std::collections::{BTreeMap, VecDeque};

use crate::siko::hir::{
    Block::BlockId,
    BlockBuilder::BlockBuilder,
    BodyBuilder::BodyBuilder,
    Instruction::{Instruction, InstructionKind, SyntaxBlockId},
};

pub struct SyntaxBlockIterator {
    bodyBuilder: BodyBuilder,
    queue: VecDeque<(BlockId, SyntaxBlockId)>,
}

impl SyntaxBlockIterator {
    pub fn new(bodyBuilder: BodyBuilder) -> SyntaxBlockIterator {
        SyntaxBlockIterator {
            bodyBuilder,
            queue: VecDeque::new(),
        }
    }

    pub fn iterate<F>(&mut self, mut callback: F)
    where
        F: FnMut(&Instruction, &SyntaxBlockId, &mut BlockBuilder),
    {
        let mut blockSyntaxBlocks = BTreeMap::new();

        self.addToQueue(BlockId::first(), SyntaxBlockId::new());

        while let Some((blockId, initialSyntaxBlock)) = self.queue.pop_front() {
            if blockSyntaxBlocks.contains_key(&blockId) {
                let existingSyntaxBlock = blockSyntaxBlocks.get(&blockId).unwrap();
                if *existingSyntaxBlock != initialSyntaxBlock {
                    panic!(
                        "Inconsistent syntax block for block {:?}: existing {} vs new {}",
                        blockId, existingSyntaxBlock, initialSyntaxBlock
                    );
                }
                continue;
            }

            blockSyntaxBlocks.insert(blockId, initialSyntaxBlock.clone());
            self.processBlock(blockId, initialSyntaxBlock, &mut callback);
        }
    }

    fn processBlock<F>(&mut self, blockId: BlockId, initialSyntaxBlock: SyntaxBlockId, callback: &mut F)
    where
        F: FnMut(&Instruction, &SyntaxBlockId, &mut BlockBuilder),
    {
        let mut currentSyntaxBlock = initialSyntaxBlock;

        let mut builder = self.bodyBuilder.iterator(blockId);
        loop {
            match builder.getInstruction() {
                Some(instruction) => {
                    // Update the current syntax block based on the instruction
                    match &instruction.kind {
                        InstructionKind::BlockStart(blockId) => {
                            currentSyntaxBlock = blockId.clone();
                        }
                        InstructionKind::BlockEnd(blockId) => {
                            currentSyntaxBlock = blockId.getParent();
                        }
                        InstructionKind::Jump(_, targetBlock) => {
                            self.addToQueue(*targetBlock, currentSyntaxBlock.clone());
                        }
                        InstructionKind::EnumSwitch(_, cases) => {
                            for case in cases {
                                self.addToQueue(case.branch, currentSyntaxBlock.clone());
                            }
                        }
                        InstructionKind::IntegerSwitch(_, cases) => {
                            for case in cases {
                                self.addToQueue(case.branch, currentSyntaxBlock.clone());
                            }
                        }
                        InstructionKind::With(_, info) => {
                            self.addToQueue(info.blockId, currentSyntaxBlock.clone());
                        }
                        _ => {}
                    }

                    // Call the callback with the instruction and current syntax block
                    callback(&instruction, &currentSyntaxBlock, &mut builder);

                    builder.step();
                }
                None => break,
            }
        }
    }

    fn addToQueue(&mut self, blockId: BlockId, syntaxBlock: SyntaxBlockId) {
        self.queue.push_back((blockId, syntaxBlock));
    }
}
