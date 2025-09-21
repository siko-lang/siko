use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    rc::Rc,
};

use crate::siko::{
    hir::{
        Block::BlockId, BlockBuilder::InstructionRef, BodyBuilder::BodyBuilder, Function::Function,
        Instruction::InstructionKind, Variable::VariableName,
    },
    util::DependencyProcessor::processDependencies,
};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct YieldKey {
    pub destVar: VariableName,
    pub resultVar: VariableName,
}

impl Display for YieldKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Yield({},{})", self.destVar, self.resultVar)
    }
}

pub struct YieldInfo {
    pub yieldKey: YieldKey,
    pub id: InstructionRef,
    pub existingVariables: BTreeSet<VariableName>,
    pub savedVariables: BTreeSet<VariableName>,
}

impl Display for YieldInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} = {{id: {}, existingVars: [{}], savedVars: [{}]}}",
            self.yieldKey,
            self.id,
            self.existingVariables
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            self.savedVariables
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

pub struct CoroutineStateProcessor<'a> {
    f: &'a Function,
    bodyBuilder: BodyBuilder,
    blockEnvs: BTreeMap<BlockId, Environment>,
    queue: Vec<BlockId>,
    yieldInfos: BTreeMap<YieldKey, YieldInfo>,
}

impl CoroutineStateProcessor<'_> {
    pub fn new(f: &Function) -> CoroutineStateProcessor {
        CoroutineStateProcessor {
            f,
            bodyBuilder: BodyBuilder::cloneFunction(f),
            blockEnvs: BTreeMap::new(),
            queue: Vec::new(),
            yieldInfos: BTreeMap::new(),
        }
    }

    fn getBlockEnv(&self, blockId: BlockId) -> Environment {
        self.blockEnvs
            .get(&blockId)
            .expect("Block environment not found")
            .clone()
    }

    pub fn process(&mut self) {
        //println!("Processing coroutine state for function: {}", self.f.name);
        self.queue.push(BlockId::first());
        let allBlockIds = self.bodyBuilder.getAllBlockIds();
        let mut blockUses = BTreeMap::new();
        let mut blockDeps = BTreeMap::new();
        for blockId in allBlockIds {
            self.blockEnvs.insert(blockId, Environment::new());
            let block = self.f.getBlockById(blockId);
            let inner = block.getInner();
            let b = inner.borrow();
            for instr in &b.instructions {
                let vars = instr.kind.collectVariables();
                blockUses
                    .entry(blockId)
                    .or_insert(BTreeSet::new())
                    .extend(vars.iter().map(|v| v.name().clone()));
            }
            let last = block.getLastInstruction();
            let deps = match &last.kind {
                InstructionKind::Jump(_, target) => vec![*target],
                InstructionKind::EnumSwitch(_, cases) => cases.iter().map(|c| c.branch).collect(),
                InstructionKind::IntegerSwitch(_, cases) => cases.iter().map(|c| c.branch).collect(),
                _ => vec![],
            };
            blockDeps.insert(blockId, deps);
        }
        // block uses tells us which variables are used in a block
        // block deps tells us which blocks are reachable from a block
        // we can calculate for each block the set of variables that will be used after the end of the block
        let groups = processDependencies(&blockDeps);
        // we have calculated the SCCs of the block dependency graph
        // now we can process the groups in topological order
        let mut updatedUses: BTreeMap<BlockId, BTreeSet<VariableName>> = BTreeMap::new();
        for group in &groups {
            for item in &group.items {
                let mut uses = blockUses.get(&item).expect("Block uses not found").clone();
                for dep in blockDeps.get(&item).cloned().unwrap_or_default() {
                    if group.items.contains(&dep) {
                        continue;
                    }
                    if let Some(depUses) = updatedUses.get(&dep) {
                        uses.extend(depUses.iter().cloned());
                    } else {
                        panic!("Dependency uses not found for block {}, groups {:?}", dep, groups);
                    }
                }
                updatedUses.insert(item.clone(), uses.clone());
            }
        }

        while let Some(blockId) = self.queue.pop() {
            self.processBlock(blockId);
        }

        // for each yield instruction, we know the set of variables which existed before the yield
        // and we know the set of variables which will be used after the block containing the yield
        // we can combine these to get the set of variables which need to be stored in the coroutine state
        // we need variables which are used after the yield, but which are not defined after the yield
        for (_, info) in &mut self.yieldInfos {
            let blockDeps = blockDeps
                .get(&info.id.blockId)
                .expect("Block dependencies not found for block");
            // collect uses from all dependent blocks
            // plus the uses from the current block after the yield instruction
            let mut uses = BTreeSet::new();
            for dep in blockDeps {
                if let Some(depUses) = updatedUses.get(&dep) {
                    uses.extend(depUses.iter().cloned());
                } else {
                    panic!("Dependency uses not found for block {}, groups {:?}", dep, groups);
                }
            }

            // collect uses from the current block after the yield instruction
            let mut builder = self.bodyBuilder.iterator(info.id.blockId);
            builder.stepTo((info.id.instructionId + 1) as usize);
            loop {
                if let Some(instr) = builder.getInstruction() {
                    let vars = instr.kind.collectVariables();
                    uses.extend(vars.iter().map(|v| v.name().clone()));
                    builder.step();
                } else {
                    break;
                }
            }

            for var in uses {
                if info.existingVariables.contains(&var) {
                    info.savedVariables.insert(var.clone());
                }
            }
        }
        // println!("Coroutine state processing complete.");
        // println!("Yield infos:");

        // for (_, info) in &self.yieldInfos {
        //     println!("Yield info: {}", info);
        // }
    }

    fn processJump(&mut self, targetBlock: BlockId, sourceEnv: &Environment) {
        let targetEnv = self.getBlockEnv(targetBlock);
        if targetEnv.merge(sourceEnv) {
            self.queue.push(targetBlock);
        }
    }

    fn processBlock(&mut self, blockId: BlockId) {
        let env = self.getBlockEnv(blockId);
        let mut blockBuilder = self.bodyBuilder.iterator(blockId);
        loop {
            if let Some(instr) = blockBuilder.getInstruction() {
                match &instr.kind {
                    InstructionKind::Jump(_, targetBlock) => {
                        self.processJump(*targetBlock, &env);
                    }
                    InstructionKind::EnumSwitch(_, cases) => {
                        for c in cases {
                            self.processJump(c.branch, &env);
                        }
                    }
                    InstructionKind::IntegerSwitch(_, cases) => {
                        for c in cases {
                            self.processJump(c.branch, &env);
                        }
                    }
                    InstructionKind::Yield(dest, var) => {
                        let yieldKey = YieldKey {
                            destVar: dest.name().clone(),
                            resultVar: var.name().clone(),
                        };
                        let info = self.yieldInfos.entry(yieldKey.clone()).or_insert(YieldInfo {
                            yieldKey: yieldKey.clone(),
                            id: blockBuilder.getInstructionRef(),
                            existingVariables: BTreeSet::new(),
                            savedVariables: BTreeSet::new(),
                        });
                        // we assume that the yield vars uniquely identify the yield instruction
                        // so asserting that the ids are the same
                        assert_eq!(info.id, blockBuilder.getInstructionRef());
                        info.existingVariables
                            .extend(env.existingVariables.borrow().iter().cloned());
                        env.addVariable(&dest.name());
                        env.addVariable(&var.name());
                    }
                    _ => {
                        let vars = instr.kind.collectVariables();
                        for var in vars {
                            env.addVariable(&var.name());
                        }
                    }
                }
                blockBuilder.step();
            } else {
                break;
            }
        }
    }
}

#[derive(Clone)]
struct Environment {
    existingVariables: Rc<RefCell<BTreeSet<VariableName>>>,
}

impl Environment {
    fn new() -> Self {
        Environment {
            existingVariables: Rc::new(RefCell::new(BTreeSet::new())),
        }
    }

    fn addVariable(&self, var: &VariableName) {
        self.existingVariables.borrow_mut().insert(var.clone());
    }

    fn merge(&self, other: &Environment) -> bool {
        let mut self_vars = self.existingVariables.borrow_mut();
        let mut updated = false;
        for var in other.existingVariables.borrow().iter() {
            updated |= self_vars.insert(var.clone());
        }
        updated
    }
}
