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
        let mut allDeps: BTreeMap<BlockId, Vec<BlockId>> = BTreeMap::new();
        if let Some(body) = &self.f.body {
            for (id, block) in &body.blocks {
                allDeps.entry(id.clone()).or_insert_with(Vec::new);
                let inner = block.getInner();
                let b = inner.borrow();
                for instruction in &b.instructions {
                    match &instruction.kind {
                        InstructionKind::Jump(_, target) => {
                            allDeps.entry(id.clone()).or_insert_with(Vec::new).push(target.clone());
                        }
                        InstructionKind::EnumSwitch(_, cases) => {
                            for c in cases {
                                allDeps
                                    .entry(id.clone())
                                    .or_insert_with(Vec::new)
                                    .push(c.branch.clone());
                            }
                        }
                        InstructionKind::IntegerSwitch(_, cases) => {
                            for c in cases {
                                allDeps
                                    .entry(id.clone())
                                    .or_insert_with(Vec::new)
                                    .push(c.branch.clone());
                            }
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
