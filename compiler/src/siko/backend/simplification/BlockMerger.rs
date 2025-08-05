use std::collections::BTreeMap;

use crate::siko::hir::{
    BodyBuilder::BodyBuilder,
    Function::{BlockId, Function},
    Instruction::InstructionKind,
};

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
                if let Some(lastInstruction) = block.instructions.last() {
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
                        InstructionKind::StringSwitch(_, cases) => {
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

        if let Some(body) = &self.function.body {
            for (blockId, block) in body.blocks.iter() {
                if let Some(lastInstruction) = block.instructions.last() {
                    if let InstructionKind::Jump(_, targetId) = &lastInstruction.kind {
                        // Check if target block has exactly one incoming jump
                        if let Some(&count) = self.jumpCounts.get(targetId) {
                            if count == 1 {
                                bodyBuilder.mergeBlocks(*blockId, *targetId);
                                merged = true;
                                break;
                            }
                        }
                    }
                }
            }
        }

        if merged {
            let mut f = self.function.clone();
            f.body = Some(bodyBuilder.build());
            Some(f)
        } else {
            None
        }
    }
}
