use std::collections::BTreeMap;

use crate::siko::{
    backend::{
        drop::{Context::Context, DropMetadataStore::DropMetadataStore},
        path::{Path::Path, ReferenceStore::ReferenceStore, Usage::getUsageInfo},
    },
    hir::{Block::BlockId, BlockBuilder::BlockBuilder, Instruction::InstructionKind, Variable::Variable},
};

pub struct BlockProcessor<'a> {
    receiverPaths: BTreeMap<Variable, Path>,
    dropMetadataStore: &'a DropMetadataStore,
    referenceStore: &'a ReferenceStore,
}

impl<'a> BlockProcessor<'a> {
    pub fn new(dropMetadataStore: &'a DropMetadataStore, referenceStore: &'a ReferenceStore) -> BlockProcessor<'a> {
        BlockProcessor {
            receiverPaths: BTreeMap::new(),
            dropMetadataStore,
            referenceStore,
        }
    }

    pub fn process(&mut self, mut builder: BlockBuilder, mut context: Context) -> (Context, Vec<BlockId>) {
        // println!("Processing block: {}", builder.getBlockId());
        // println!("starting context: {}", context);
        // println!("--------------");
        let mut jumpTargets = Vec::new();
        loop {
            if let Some(instruction) = builder.getInstruction() {
                // println!(
                //     "Processing instruction: {} {} {}",
                //     instruction.kind, instruction.location, instructionRef
                // );
                match &instruction.kind {
                    InstructionKind::Jump(_, blockId) => {
                        jumpTargets.push(blockId.clone());
                    }
                    InstructionKind::EnumSwitch(_, cases) => {
                        // enum switch does not 'use' the variable, transform does
                        for case in cases {
                            jumpTargets.push(case.branch.clone());
                        }
                    }
                    InstructionKind::IntegerSwitch(var, cases) => {
                        context.useVar(var, builder.getInstructionRef());
                        for case in cases {
                            jumpTargets.push(case.branch.clone());
                        }
                    }
                    InstructionKind::With(_, info) => {
                        jumpTargets.push(info.blockId.clone());
                    }
                    kind => {
                        let usageinfo = getUsageInfo(kind.clone(), self.referenceStore);
                        // println!("Instruction: {}", instruction.kind);
                        // println!("Usage info: {}", usageinfo);
                        for mut usage in usageinfo.usages {
                            usage.path.instructionRef = builder.getInstructionRef();
                            context.addUsage(usage);
                        }
                        if let Some(mut assignPath) = usageinfo.assign {
                            assignPath.instructionRef = builder.getInstructionRef();
                            context.addAssign(assignPath.clone());
                        }
                    }
                }
                builder.step();
            } else {
                break;
            }
        }
        for (name, events) in context.usages.iter() {
            if let Some(declarationList) = self.dropMetadataStore.getPathList(name) {
                for path in events.getAllWritePaths() {
                    let mut simplePath = path.toSimplePath();
                    loop {
                        declarationList.addPath(simplePath.clone());
                        if let Some(parent) = simplePath.getParent() {
                            simplePath = parent;
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        //println!("Final context: {}", context);
        (context, jumpTargets)
    }
}
