use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    rc::Rc,
};

use crate::siko::hir::{
    Block::BlockId,
    BlockBuilder::InstructionRef,
    BodyBuilder::BodyBuilder,
    Function::Function,
    Instruction::InstructionKind,
    Variable::{Variable, VariableName},
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

#[derive(Clone)]
pub struct YieldInfo {
    pub yieldKey: YieldKey,
    pub id: InstructionRef,
    pub savedVariables: BTreeSet<Variable>,
}

impl Display for YieldInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} = {{id: {}, savedVars: [{}]}}",
            self.yieldKey,
            self.id,
            self.savedVariables
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[derive(Clone)]
struct YieldInfoStore {
    infos: Rc<RefCell<BTreeMap<YieldKey, YieldInfo>>>,
}

impl YieldInfoStore {
    fn new() -> Self {
        YieldInfoStore {
            infos: Rc::new(RefCell::new(BTreeMap::new())),
        }
    }

    fn get(&self, key: &YieldKey) -> Option<YieldInfo> {
        self.infos.borrow().get(key).cloned()
    }

    fn insert(&self, info: YieldInfo) {
        self.infos.borrow_mut().insert(info.yieldKey.clone(), info);
    }

    fn addSavedVariable(&self, y: &&YieldKey, var: &Variable) {
        let mut infos = self.infos.borrow_mut();
        let info = infos.get_mut(y).expect("YieldInfo not found");
        info.savedVariables.insert(var.clone());
    }
}

pub struct CoroutineStateProcessor<'a> {
    f: &'a Function,
    bodyBuilder: BodyBuilder,
    blockEnvs: BTreeMap<BlockId, Environment>,
    queue: Vec<BlockId>,
    yieldInfos: YieldInfoStore,
}

impl<'a> CoroutineStateProcessor<'a> {
    pub fn new(f: &'a Function) -> CoroutineStateProcessor<'a> {
        CoroutineStateProcessor {
            f,
            bodyBuilder: BodyBuilder::cloneFunction(f),
            blockEnvs: BTreeMap::new(),
            queue: Vec::new(),
            yieldInfos: YieldInfoStore::new(),
        }
    }

    fn getBlockEnv(&self, blockId: BlockId) -> Environment {
        self.blockEnvs
            .get(&blockId)
            .expect("Block environment not found")
            .clone()
    }

    pub fn process(&mut self) {
        // println!("Processing coroutine state for function: {}", self.f.name);
        // println!("{}", self.f);
        self.queue.push(BlockId::first());
        let allBlockIds = self.bodyBuilder.getAllBlockIds();
        for blockId in allBlockIds {
            self.blockEnvs.insert(blockId, Environment::new());
        }

        while let Some(blockId) = self.queue.pop() {
            self.processBlock(blockId);
        }

        // println!("Coroutine state processing complete.");
        // println!("Yield infos:");

        // for (_, info) in self.yieldInfos.infos.borrow().iter() {
        //     println!("Yield info: {}", info);
        // }
    }

    pub fn getYieldInfo(&self, key: &YieldKey) -> YieldInfo {
        self.yieldInfos.get(key).expect("YieldInfo not found")
    }

    fn processJump(&mut self, targetBlock: BlockId, sourceEnv: &Environment) {
        let targetEnv = self.getBlockEnv(targetBlock);
        // println!("Merging envs for jump to block {}", targetBlock);
        // println!("  Source env:\n{}", sourceEnv);
        // println!("  Target env before merge:\n{}", targetEnv);
        if targetEnv.merge(sourceEnv) {
            self.queue.push(targetBlock);
        }
    }

    fn processBlock(&mut self, blockId: BlockId) {
        //println!("Processing block: {}", blockId);
        let env = self.getBlockEnv(blockId);
        let mut builder = self.bodyBuilder.iterator(blockId);
        loop {
            if let Some(instr) = builder.getInstruction() {
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
                        let mut infos = self.yieldInfos.infos.borrow_mut();
                        let info = infos.entry(yieldKey.clone()).or_insert(YieldInfo {
                            yieldKey: yieldKey.clone(),
                            id: builder.getInstructionRef(),
                            savedVariables: BTreeSet::new(),
                        });
                        for (_, values) in env.liveValues.borrow_mut().iter_mut() {
                            values.addYield(&yieldKey);
                        }
                        // we assume that the yield vars uniquely identify the yield instruction
                        // so asserting that the ids are the same
                        assert_eq!(info.id, builder.getInstructionRef());
                        env.addLiveValue(dest);
                        builder.step();
                        continue;
                    }
                    _ => {}
                }
                let mut vars = instr.kind.collectVariables();
                if let Some(resultVar) = instr.kind.getResultVar() {
                    env.addLiveValue(&resultVar);
                    vars.retain(|v| v != &resultVar);
                }
                for var in vars {
                    env.useValue(&var, &self.yieldInfos);
                }
                builder.step();
            } else {
                break;
            }
        }
    }
}

struct ValueInfo {
    var: Variable,
    yields: BTreeSet<YieldKey>,
    used: bool,
}

impl ValueInfo {
    fn new(var: Variable) -> Self {
        ValueInfo {
            var,
            yields: BTreeSet::new(),
            used: false,
        }
    }
}

impl Display for ValueInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: used={}, yields=[{}]",
            self.var,
            self.used,
            self.yields.iter().map(|y| y.to_string()).collect::<Vec<_>>().join(", ")
        )
    }
}

struct LiveValueStore {
    values: BTreeMap<Variable, ValueInfo>,
}

impl LiveValueStore {
    fn new() -> Self {
        LiveValueStore {
            values: BTreeMap::new(),
        }
    }

    fn addValue(&mut self, var: &Variable) {
        self.values
            .entry(var.clone())
            .or_insert_with(|| ValueInfo::new(var.clone()));
    }

    fn useValue(&mut self, var: &Variable, yieldInfoStore: &YieldInfoStore) {
        //println!("   Using variable: {}", var);
        for (v, info) in self.values.iter_mut() {
            if v.name() != var.name() {
                continue;
            }
            if info.yields.is_empty() {
                return;
            }
            // println!("   Using variable: {}", v);
            // println!("    Yield infos:");
            for y in &info.yields {
                yieldInfoStore.addSavedVariable(&y, var);
                //println!("      {}", y);
            }
            info.used = true;
        }
    }

    fn merge(&mut self, other: &LiveValueStore) -> bool {
        let mut changed = false;
        let startLen = self.values.len();
        for (var, otherInfo) in other.values.iter() {
            let selfInfo = self
                .values
                .entry(var.clone())
                .or_insert_with(|| ValueInfo::new(var.clone()));
            let beforeLen = selfInfo.yields.len();
            selfInfo.yields.extend(otherInfo.yields.iter().cloned());
            if selfInfo.yields.len() > beforeLen {
                changed = true;
            }
        }
        if self.values.len() > startLen {
            changed = true;
        }
        changed
    }

    fn addYield(&mut self, yield_key: &YieldKey) {
        for info in self.values.values_mut() {
            //println!("   Adding yield {} to variable {}", yield_key, info.var);
            info.yields.insert(yield_key.clone());
        }
    }
}

impl Display for LiveValueStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "LiveValueStore:")?;
        for (_, info) in &self.values {
            writeln!(f, "  {}", info)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
struct Environment {
    liveValues: Rc<RefCell<BTreeMap<VariableName, LiveValueStore>>>,
}

impl Environment {
    fn new() -> Self {
        Environment {
            liveValues: Rc::new(RefCell::new(BTreeMap::new())),
        }
    }

    fn addLiveValue(&self, var: &Variable) {
        //println!("   Adding live variable: {}", var);
        let mut values = self.liveValues.borrow_mut();
        let mut store = LiveValueStore::new();
        store.addValue(var);
        values.insert(var.name().clone(), store);
    }

    fn useValue(&self, var: &Variable, yieldInfoStore: &YieldInfoStore) {
        let mut values = self.liveValues.borrow_mut();
        if let Some(vs) = values.get_mut(&var.name()) {
            vs.useValue(var, yieldInfoStore);
        }
    }

    fn merge(&self, other: &Environment) -> bool {
        let mut changed = false;
        let mut selfValues = self.liveValues.borrow_mut();
        let otherValues = other.liveValues.borrow();
        for (name, otherStore) in otherValues.iter() {
            let selfStore = selfValues.entry(name.clone()).or_insert(LiveValueStore::new());
            if selfStore.merge(otherStore) {
                changed = true;
            }
        }
        changed
    }
}

impl Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Environment:")?;
        let values = self.liveValues.borrow();
        for (name, store) in values.iter() {
            writeln!(f, "  {}: {}", name, store)?;
        }
        Ok(())
    }
}
