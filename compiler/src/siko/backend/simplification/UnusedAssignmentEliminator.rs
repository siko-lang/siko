use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::siko::{
    backend::simplification::Utils,
    hir::{
        Block::BlockId,
        BlockBuilder::{BlockBuilder, InstructionRef},
        BlockGroupBuilder::{BlockGroupBuilder, BlockGroupInfo},
        BodyBuilder::BodyBuilder,
        Function::Function,
        Instruction::InstructionKind,
        Program::Program,
        Variable::{Variable, VariableName},
    },
    util::Runner::Runner,
};

type DefinitionMap = BTreeMap<VariableName, BTreeSet<InstructionRef>>;

pub fn simplifyFunction(f: &Function, program: &Program, runner: Runner) -> Option<Function> {
    let mut eliminator = UnusedAssignmentEliminator::new(f, program, runner);
    eliminator.process()
}

pub struct UnusedAssignmentEliminator<'a> {
    program: &'a Program,
    function: &'a Function,
    modified: bool,
    assigned: BTreeSet<InstructionRef>,
    used: BTreeSet<InstructionRef>,
    runner: Runner,
    traceEnabled: bool,
}

impl<'a> UnusedAssignmentEliminator<'a> {
    pub fn new(f: &'a Function, program: &'a Program, runner: Runner) -> UnusedAssignmentEliminator<'a> {
        let traceEnabled = runner.getConfig().dumpCfg.unusedAssignmentEliminatorTraceEnabled;
        UnusedAssignmentEliminator {
            function: f,
            program,
            assigned: BTreeSet::new(),
            used: BTreeSet::new(),
            modified: false,
            runner,
            traceEnabled,
        }
    }

    pub fn process(&mut self) -> Option<Function> {
        if self.function.body.is_none() {
            return None; // No body to evaluate
        }

        if self.traceEnabled {
            println!(
                "[UAE] ===== Function {} =====",
                self.function.name
            );
        }

        // println!("UnusedAssignmentEliminator processing function: {}", self.function.name);
        // println!("{}", self.function);

        let mut bodyBuilder = BodyBuilder::cloneFunction(self.function);
        let allBlockIds = bodyBuilder.getAllBlockIds();

        let mut blockSummaries: BTreeMap<BlockId, BlockSummary> = BTreeMap::new();
        let analyzeRunner = self.runner.child("analyze_blocks");
        analyzeRunner.run(|| {
            for blockId in &allBlockIds {
                let mut builder = bodyBuilder.iterator(*blockId);
                let summary = self.analyzeBlock(&mut builder);
                blockSummaries.insert(*blockId, summary);
            }
        });

        let blockGroupBuilder = BlockGroupBuilder::new(self.function);
        let groupInfo = blockGroupBuilder.process();

        let mut incoming: BTreeMap<BlockId, DefinitionMap> = BTreeMap::new();
        let mut outgoing: BTreeMap<BlockId, DefinitionMap> = BTreeMap::new();

        let propagationRunner = self.runner.child("propagate");
        propagationRunner.run(|| {
            for group in &groupInfo.groups {
                let scc_size = group.items.len();
                {
                    let mut stats = self.runner.statistics.borrow_mut();
                    if stats.maxSCCSizeInUnusedAssignmentEliminator < scc_size {
                        stats.maxSCCSizeInUnusedAssignmentEliminator = scc_size;
                    }
                }
                if self.traceEnabled {
                    let block_list: Vec<String> = group
                        .items
                        .iter()
                        .map(|bid| format!("{}", bid))
                        .collect();
                    println!("[UAE] SCC start (size {}): {}", scc_size, block_list.join(", "));
                }
                let groupSet: BTreeSet<BlockId> = group.items.iter().cloned().collect();
                let mut queue: VecDeque<BlockId> = VecDeque::new();
                let mut queued: BTreeSet<BlockId> = BTreeSet::new();
                let blockRunner = propagationRunner.child("block");
                for blockId in &group.items {
                    if queued.insert(*blockId) {
                        queue.push_back(*blockId);
                    }
                }
                let mut iterations: u32 = 0;
                while let Some(blockId) = queue.pop_front() {
                    iterations = iterations.saturating_add(1);
                    queued.remove(&blockId);
                    let summary = blockSummaries.get(&blockId).expect("Missing block summary");
                    if self.traceEnabled {
                        println!("[UAE]   Iteration {} processing block {}", iterations, blockId);
                    }
                    blockRunner.run(|| {
                        self.process_propagation_block(
                            blockId,
                            summary,
                            &groupInfo,
                            &groupSet,
                            &mut incoming,
                            &mut outgoing,
                            &mut queue,
                            &mut queued,
                            self.traceEnabled,
                        );
                    });
                }
                {
                    let mut stats = self.runner.statistics.borrow_mut();
                    if stats.maxFixPointIterationCountInAssignmentEliminator < iterations {
                        stats.maxFixPointIterationCountInAssignmentEliminator = iterations;
                    }
                }
                if self.traceEnabled {
                    println!("[UAE] SCC finished after {} iterations", iterations);
                }
            }
        });

        let mut unused = BTreeMap::new();
        for assigned in &self.assigned {
            if !self.used.contains(assigned) {
                unused.entry(assigned.blockId).or_insert_with(Vec::new).push(assigned);
            }
        }

        for (blockId, instructions) in unused {
            let mut builder = bodyBuilder.iterator(blockId);
            for iRef in instructions.iter().rev() {
                let i = builder
                    .getInstructionAt(iRef.instructionId as usize)
                    .expect("Failed to get instruction at index");
                if Utils::canBeEliminated(self.program, &i.kind) {
                    //println!("Eliminating instruction: {}", i);
                    if self.traceEnabled {
                        println!(
                            "[UAE] Removing unused instruction {:?} from block {} in {}",
                            i.kind, blockId, self.function.name
                        );
                    }
                    builder.removeInstructionAt(iRef.instructionId as usize);
                    self.modified = true;
                }
            }
            if builder.getBlockSize() == 0 {
                bodyBuilder.removeBlock(blockId);
            }
        }

        if !self.modified {
            return None; // No modifications made
        }
        let mut f = self.function.clone();
        f.body = Some(bodyBuilder.build());

        if self.traceEnabled {
            println!("[UAE] ===== Function {} finished =====", self.function.name);
        }

        Some(f)
    }

    fn useVar(
        &mut self,
        var: &Variable,
        env: &DefinitionMap,
        externalReads: &mut BTreeSet<VariableName>,
    ) {
        let name = var.name();
        if let Some(defs) = env.get(&name) {
            for def in defs {
                self.used.insert(*def);
            }
        } else {
            externalReads.insert(name);
        }
    }

    fn analyzeBlock(&mut self, builder: &mut BlockBuilder) -> BlockSummary {
        let mut env: DefinitionMap = BTreeMap::new();
        let mut externalReads: BTreeSet<VariableName> = BTreeSet::new();
        let mut blockAssigned: BTreeSet<InstructionRef> = BTreeSet::new();

        loop {
            if let Some(instruction) = builder.getInstruction() {
                match instruction.kind {
                    InstructionKind::DeclareVar(_, _) => {}
                    InstructionKind::EnumSwitch(var, _) => {
                        self.useVar(&var, &env, &mut externalReads);
                    }
                    InstructionKind::IntegerSwitch(var, _) => {
                        self.useVar(&var, &env, &mut externalReads);
                    }
                    InstructionKind::Jump(_, _) => {}
                    InstructionKind::Return(_, v) => {
                        self.useVar(&v, &env, &mut externalReads);
                    }
                    InstructionKind::StorePtr(v1, v2) => {
                        self.useVar(&v1, &env, &mut externalReads);
                        self.useVar(&v2, &env, &mut externalReads);
                    }
                    k => {
                        let mut vars = k.collectVariables();
                        if let Some(v) = k.getResultVar() {
                            if v.isUserDefined() {
                                self.used.insert(builder.getInstructionRef());
                            }
                            let iRef = builder.getInstructionRef();
                            self.assigned.insert(iRef);
                            blockAssigned.insert(iRef);
                            let varName = v.name();
                            env.entry(varName.clone())
                                .or_insert_with(BTreeSet::new)
                                .insert(iRef);
                            vars.retain(|x| x != &v);
                        }
                        for variable in vars.iter() {
                            self.useVar(variable, &env, &mut externalReads);
                        }
                    }
                }
                builder.step();
            } else {
                break;
            }
        }

        let mut exportedDefs: DefinitionMap = BTreeMap::new();
        for (var, defs) in &env {
            for def in defs {
                if blockAssigned.contains(def) {
                    exportedDefs
                        .entry(var.clone())
                        .or_insert_with(BTreeSet::new)
                        .insert(*def);
                }
            }
        }

        BlockSummary {
            externalReads,
            exportedDefs,
        }
    }

    fn process_propagation_block(
        &mut self,
        blockId: BlockId,
        summary: &BlockSummary,
        groupInfo: &BlockGroupInfo,
        groupSet: &BTreeSet<BlockId>,
        incoming: &mut BTreeMap<BlockId, DefinitionMap>,
        outgoing: &mut BTreeMap<BlockId, DefinitionMap>,
        queue: &mut VecDeque<BlockId>,
        queued: &mut BTreeSet<BlockId>,
        traceEnabled: bool,
    ) {
        let entry = incoming.entry(blockId).or_insert_with(BTreeMap::new);

        let mut newEntry: DefinitionMap = BTreeMap::new();
        if let Some(preds) = groupInfo.deps.get(&blockId) {
            for pred in preds {
                if let Some(out) = outgoing.get(pred) {
                    for (var, defs) in out {
                        newEntry
                            .entry(var.clone())
                            .or_insert_with(BTreeSet::new)
                            .extend(defs.iter().copied());
                    }
                }
            }
        }

        let mut addedDefs = Vec::new();
        for (var, defs) in &newEntry {
            let entrySet = entry.entry(var.clone()).or_insert_with(BTreeSet::new);
            for def in defs {
                if entrySet.insert(*def) {
                    addedDefs.push((var.clone(), *def));
                }
            }
        }

        let existingVars: Vec<VariableName> = entry.keys().cloned().collect();
        for var in existingVars {
            if let Some(newDefs) = newEntry.get(&var) {
                if let Some(entrySet) = entry.get_mut(&var) {
                    entrySet.retain(|def| newDefs.contains(def));
                    if entrySet.is_empty() {
                        entry.remove(&var);
                        if traceEnabled {
                            println!("[UAE]     Block {}: cleared incoming {}", blockId, var);
                        }
                    }
                }
            } else {
                entry.remove(&var);
                if traceEnabled {
                    println!("[UAE]     Block {}: cleared incoming {}", blockId, var);
                }
            }
        }

        if traceEnabled {
            if addedDefs.is_empty() {
                println!("[UAE]     Block {}: no new incoming definitions", blockId);
            } else {
                println!("[UAE]     Block {}: new incoming definitions", blockId);
                for (var, def) in &addedDefs {
                    println!("[UAE]       {} <= {}", var, def);
                }
            }
        }

        for (var, def) in &addedDefs {
            if summary.externalReads.contains(var) {
                self.used.insert(*def);
            }
        }

        let mut newOutgoing = entry.clone();
        for (var, defs) in &summary.exportedDefs {
            newOutgoing
                .entry(var.clone())
                .or_insert_with(BTreeSet::new)
                .extend(defs.iter().copied());
        }

        let outgoingEntry = outgoing.entry(blockId).or_insert_with(BTreeMap::new);
        if *outgoingEntry != newOutgoing {
            *outgoingEntry = newOutgoing;
            if let Some(succs) = groupInfo.inverseDeps.get(&blockId) {
                if traceEnabled && !succs.is_empty() {
                    let succ_list: Vec<String> = succs.iter().map(|bid| format!("{}", bid)).collect();
                    println!("[UAE]     Block {} changed, enqueue successors: {}", blockId, succ_list.join(", "));
                }
                for succ in succs {
                    if groupSet.contains(succ) {
                        if queued.insert(*succ) {
                            if traceEnabled {
                                println!("[UAE]       queued block {}", succ);
                            }
                            queue.push_back(*succ);
                        }
                    }
                }
            }
        } else if traceEnabled {
            println!("[UAE]     Block {} outgoing unchanged", blockId);
        }
    }

}

struct BlockSummary {
    externalReads: BTreeSet<VariableName>,
    exportedDefs: DefinitionMap,
}
