use std::collections::BTreeMap;

use crate::siko::{
    backend::{
        borrowcheck::usageprocessor::{Environment::Environment, Path::Path, Value::Value},
        path::{ReferenceStore::ReferenceStore, SimplePath::SimplePath, Usage::getUsageInfo},
    },
    hir::{
        Block::BlockId, BlockBuilder::InstructionRef, BlockGroupBuilder::BlockGroupBuilder, Function::Function,
        Instruction::InstructionKind, Type::Type, Variable::Variable,
    },
    location::Location::Location,
    util::Runner::Runner,
};

struct BorrowInfo {
    pub location: Location,
}

struct BorrowSet {
    paths: BTreeMap<Path, BorrowInfo>,
}

pub struct UsageProcessor<'a> {
    f: &'a Function,
    borrows: BTreeMap<Type, BorrowSet>,
    blockEnvs: BTreeMap<BlockId, Environment>,
    traceEnabled: bool,
    liveValues: BTreeMap<InstructionRef, BTreeMap<InstructionRef, Value>>,
    referenceStore: ReferenceStore,
}

impl<'a> UsageProcessor<'a> {
    pub fn new(f: &'a Function, runner: Runner) -> UsageProcessor<'a> {
        UsageProcessor {
            f,
            borrows: BTreeMap::new(),
            blockEnvs: BTreeMap::new(),
            traceEnabled: runner.getConfig().dumpCfg.usageProcessorTraceEnabled,
            liveValues: BTreeMap::new(),
            referenceStore: ReferenceStore::new(),
        }
    }

    pub fn process(&mut self) {
        if self.f.body.is_none() {
            return;
        }
        if self.traceEnabled {
            println!("Usage processor checking function: {}", self.f.name);
            println!("Function profile {}", self.f);
        }
        self.referenceStore.build(self.f);

        let blockGroupBuilder = BlockGroupBuilder::new(self.f);
        let groupInfo = blockGroupBuilder.process();
        //println!("Block groups: {:?}", groupInfo.groups);
        for group in groupInfo.groups {
            //println!("Processing block group {:?}", group.items);
            let mut queue = Vec::new();
            for blockId in &group.items {
                queue.push(blockId.clone());
                self.blockEnvs.entry(blockId.clone()).or_insert_with(Environment::new);
            }
            while let Some(blockId) = queue.pop() {
                let entryEnv = self.getEnvForBlock(blockId.clone());
                let env = entryEnv.snapshot();
                let jumpTargets = self.processBlock(blockId.clone(), env);
                for (target, targetEnv) in jumpTargets {
                    let mut updated = false;
                    let entry = self.blockEnvs.entry(target.clone()).or_insert_with(|| {
                        updated = true;
                        Environment::new()
                    });
                    if entry.merge(&targetEnv) {
                        updated = true;
                    }
                    if updated && group.items.contains(&target) {
                        queue.push(target);
                    }
                }
            }
        }
    }

    fn getEnvForBlock(&mut self, blockId: BlockId) -> Environment {
        let env = self.blockEnvs.entry(blockId).or_insert_with(Environment::new);
        env.clone()
    }

    fn processBlock(&mut self, blockId: BlockId, env: Environment) -> Vec<(BlockId, Environment)> {
        //println!(" Processing block: {}", blockId);
        let block = self.f.getBlockById(blockId);
        let inner = block.getInner();
        let b = inner.borrow();
        let mut jumpTargets = Vec::new();
        for (index, i) in b.instructions.iter().enumerate() {
            let id = InstructionRef {
                blockId,
                instructionId: index as u32,
            };
            //for value in env.values.values() {}
            match &i.kind {
                InstructionKind::Jump(_, target) => {
                    jumpTargets.push((target.clone(), env.snapshot()));
                }
                InstructionKind::EnumSwitch(_, cases) => {
                    for case in cases {
                        jumpTargets.push((case.branch.clone(), env.snapshot()));
                    }
                }
                InstructionKind::IntegerSwitch(_, cases) => {
                    for case in cases {
                        jumpTargets.push((case.branch.clone(), env.snapshot()));
                    }
                }
                _ => {
                    let usages = getUsageInfo(i.kind.clone(), &mut self.referenceStore);
                    for usage in usages.usages {
                        //println!("usage: {} for {}", usage, id);
                    }
                }
            }
        }
        jumpTargets
    }
}

fn varToPath(v: &Variable) -> Path {
    Path {
        p: SimplePath::new(v.name()),
    }
}

#[derive(Clone)]
pub struct DeathInfo {
    pub location: Location,
    pub isDrop: bool,
}
