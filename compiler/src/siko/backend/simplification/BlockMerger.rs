use std::collections::BTreeMap;

use crate::siko::hir::{Block::BlockId, BodyBuilder::BodyBuilder, Function::Function, Instruction::InstructionKind};

pub fn simplifyFunction(f: &Function) -> Option<Function> {
    let mut merger = BlockMerger::new(f);
    merger.process()
}

pub struct BlockMerger<'a> {
    function: &'a Function,
    jumpCounts: BTreeMap<BlockId, usize>,
}

impl<'a> BlockMerger<'a> {
    pub fn new(f: &'a Function) -> BlockMerger<'a> {
        BlockMerger {
            function: f,
            jumpCounts: BTreeMap::new(),
        }
    }

    fn process(&mut self) -> Option<Function> {
        if self.function.body.is_none() {
            return None;
        }

        // println!("BlockMerger processing function: {}", self.function.name);
        // println!("{}", self.function);

        // First pass: count all jumps to each block
        self.countJumps();

        // Second pass: merge blocks that have only one incoming jump
        self.mergeBlocks()
    }

    fn countJumps(&mut self) {
        if let Some(body) = &self.function.body {
            for (_, block) in body.blocks.iter() {
                let inner = block.getInner();
                let b = inner.borrow();
                if let Some(lastInstruction) = b.instructions.last() {
                    match &lastInstruction.kind {
                        InstructionKind::Jump(_, targetId) => {
                            *self.jumpCounts.entry(*targetId).or_insert(0) += 1;
                        }
                        InstructionKind::EnumSwitch(_, cases) => {
                            for case in cases {
                                *self.jumpCounts.entry(case.branch).or_insert(0) += 1;
                            }
                        }
                        InstructionKind::IntegerSwitch(_, cases) => {
                            for case in cases {
                                *self.jumpCounts.entry(case.branch).or_insert(0) += 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn mergeBlocks(&mut self) -> Option<Function> {
        let mut bodyBuilder = BodyBuilder::cloneFunction(self.function);
        let mut merged = false;

        let allBlockIds = bodyBuilder.getAllBlockIds();

        for blockId in allBlockIds {
            let builder = bodyBuilder.iterator(blockId);
            if builder.isValid() {
                loop {
                    if let Some(lastInstruction) = builder.getLastInstruction() {
                        if let InstructionKind::Jump(_, targetId) = &lastInstruction.kind {
                            // Check if target block has exactly one incoming jump
                            let count = *self.jumpCounts.get(targetId).expect("Target block not found");
                            if count == 1 {
                                //println!("Merging single jump target block {} into block {}", targetId, blockId);
                                bodyBuilder.mergeBlocks(blockId, *targetId);
                                merged = true;
                            } else {
                                break; // target block has multiple incoming jumps - cannot merge
                            }
                        } else {
                            break; // No jump instruction, nothing to merge
                        }
                    } else {
                        break; // No more instructions to process in this block
                    }
                }
            }
        }

        if merged {
            //println!("Block count {}", bodyBuilder.getAllBlockIds().len());
            let mut f = self.function.clone();
            f.body = Some(bodyBuilder.build());
            Some(f)
        } else {
            None
        }
    }
}
