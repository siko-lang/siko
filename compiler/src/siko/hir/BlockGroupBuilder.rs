use std::collections::BTreeMap;

use crate::siko::{
    hir::{Block::BlockId, Function::Function, Instruction::InstructionKind},
    util::DependencyProcessor::{processDependencies, DependencyGroup},
};

pub struct BlockGroupInfo {
    pub groups: Vec<DependencyGroup<BlockId>>,
}

pub struct BlockGroupBuilder<'a> {
    f: &'a Function,
}

impl<'a> BlockGroupBuilder<'a> {
    pub fn new(f: &'a Function) -> Self {
        BlockGroupBuilder { f }
    }

    pub fn process(&self) -> BlockGroupInfo {
        // println!("Building block groups for function: {}", self.f.name);
        // println!("Function: {}", self.f);
        let mut allDeps: BTreeMap<BlockId, Vec<BlockId>> = BTreeMap::new();
        if let Some(body) = &self.f.body {
            for (id, block) in &body.blocks {
                allDeps.entry(id.clone()).or_insert_with(Vec::new);
                let inner = block.getInner();
                let b = inner.borrow();
                for instruction in &b.instructions {
                    match &instruction.kind {
                        InstructionKind::Jump(_, target) => {
                            allDeps.entry(target.clone()).or_insert_with(Vec::new).push(id.clone());
                        }
                        InstructionKind::EnumSwitch(_, cases) => {
                            for c in cases {
                                allDeps
                                    .entry(c.branch.clone())
                                    .or_insert_with(Vec::new)
                                    .push(id.clone());
                            }
                        }
                        InstructionKind::IntegerSwitch(_, cases) => {
                            for c in cases {
                                allDeps
                                    .entry(c.branch.clone())
                                    .or_insert_with(Vec::new)
                                    .push(id.clone());
                            }
                        }
                        InstructionKind::With(_, info) => {
                            allDeps
                                .entry(info.blockId.clone())
                                .or_insert_with(Vec::new)
                                .push(id.clone());
                        }
                        _ => {}
                    }
                }
            }
        }
        let groups = processDependencies(&allDeps);
        BlockGroupInfo { groups }
    }
}
