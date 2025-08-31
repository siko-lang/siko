use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use crate::siko::{
    backend::simplification::Utils,
    hir::{
        Block::BlockId,
        BlockBuilder::{BlockBuilder, InstructionRef},
        BodyBuilder::BodyBuilder,
        Function::Function,
        Instruction::InstructionKind,
        Program::Program,
        Variable::{Variable, VariableName},
    },
};

pub fn simplifyFunction(f: &Function, program: &Program) -> Option<Function> {
    let mut eliminator = UnusedAssignmentEliminator::new(f, program);
    eliminator.process()
}

pub struct UnusedAssignmentEliminator<'a> {
    program: &'a Program,
    function: &'a Function,
    blockEnvs: BTreeMap<BlockId, Environment>,
    queue: Vec<BlockId>,
    modified: bool,
    assigned: BTreeSet<InstructionRef>,
    used: BTreeSet<InstructionRef>,
}

impl<'a> UnusedAssignmentEliminator<'a> {
    pub fn new(f: &'a Function, program: &'a Program) -> UnusedAssignmentEliminator<'a> {
        UnusedAssignmentEliminator {
            function: f,
            program,
            blockEnvs: BTreeMap::new(),
            queue: Vec::new(),
            assigned: BTreeSet::new(),
            used: BTreeSet::new(),
            modified: false,
        }
    }

    pub fn process(&mut self) -> Option<Function> {
        if self.function.body.is_none() {
            return None; // No body to evaluate
        }

        // println!("UnusedAssignmentEliminator processing function: {}", self.function.name);
        // println!("{}", self.function);

        let mut bodyBuilder = BodyBuilder::cloneFunction(self.function);
        let allBlockIds = bodyBuilder.getAllBlockIds();
        for blockId in allBlockIds {
            let env = Environment::new();
            self.blockEnvs.insert(blockId, env);
        }
        self.queue.push(BlockId::first());

        while let Some(blockId) = self.queue.pop() {
            //println!("Queue length: {}", self.queue.len());
            let mut builder = bodyBuilder.iterator(blockId);
            self.processBlock(&mut builder);
        }

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
        Some(f)
    }

    fn useVar(&mut self, var: &Variable, env: &Environment) {
        let refs = env.get(&var.name());
        for iRef in refs {
            //println!("Using variable {} assigned at {:?}", var, iRef);
            self.used.insert(iRef);
        }
    }

    fn getEnvForBlock(&mut self, blockId: BlockId) -> &mut Environment {
        self.blockEnvs
            .get_mut(&blockId)
            .expect("Environment for block should exist")
    }

    fn processBlock(&mut self, builder: &mut BlockBuilder) {
        let mut env = self.getEnvForBlock(builder.getBlockId()).clone();
        //println!("---------- Evaluating block: {}/ {}", builder.getBlockId(), env);
        loop {
            if let Some(instruction) = builder.getInstruction() {
                //println!("Processing instruction: {}", instruction);
                match instruction.kind {
                    InstructionKind::DeclareVar(_, _) => {}
                    InstructionKind::EnumSwitch(var, cases) => {
                        self.useVar(&var, &env);
                        for case in &cases {
                            let targetBranch = case.branch;
                            let targetEnv = self.getEnvForBlock(targetBranch);
                            if targetEnv.merge(&env) {
                                //println!("enum switch to true block {} with updated environment", targetBranch);
                                self.queue.push(targetBranch);
                            }
                        }
                    }
                    InstructionKind::IntegerSwitch(var, cases) => {
                        self.useVar(&var, &env);
                        for case in &cases {
                            let targetBranch = case.branch;
                            let targetEnv = self.getEnvForBlock(targetBranch);
                            if targetEnv.merge(&env) {
                                //println!("enum switch to true block {} with updated environment", targetBranch);
                                self.queue.push(targetBranch);
                            }
                        }
                    }
                    InstructionKind::Jump(_, target) => {
                        let targetEnv = self.getEnvForBlock(target);
                        if targetEnv.merge(&env) {
                            //println!("Jumping to block {} with updated environment", target);
                            self.queue.push(target);
                        }
                    }
                    InstructionKind::Return(_, v) => {
                        self.useVar(&v, &env);
                    }
                    InstructionKind::StorePtr(v1, v2) => {
                        self.useVar(&v1, &env);
                        self.useVar(&v2, &env);
                    }
                    k => {
                        let mut vars = k.collectVariables();
                        if let Some(v) = k.getResultVar() {
                            if v.isUserDefined() {
                                //println!("Using variable {} assigned at {:?}", v, builder.getInstructionRef());
                                self.used.insert(builder.getInstructionRef());
                            }
                            let iRef = builder.getInstructionRef();
                            self.assigned.insert(iRef);
                            env.set(v.name(), iRef);
                            vars.retain(|x| x != &v);
                        }
                        for v in vars {
                            self.useVar(&v, &env);
                        }
                    }
                }
                builder.step();
            } else {
                break; // No more instructions
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Environment {
    values: BTreeMap<VariableName, Vec<InstructionRef>>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            values: BTreeMap::new(),
        }
    }

    pub fn get(&self, var: &VariableName) -> Vec<InstructionRef> {
        //println!("Getting variable {} from env", var);
        self.values.get(var).cloned().unwrap_or_default()
    }

    pub fn set(&mut self, var: VariableName, value: InstructionRef) {
        //println!("Setting variable {} to {:?}", var, value);
        self.values.entry(var).or_insert_with(Vec::new).push(value);
    }

    fn merge(&mut self, env: &Environment) -> bool {
        let mut changed = false;
        for (var, refs) in env.values.iter() {
            let entry = self.values.entry(var.clone()).or_insert_with(Vec::new);
            for r in refs {
                if !entry.contains(r) {
                    entry.push(*r);
                    changed = true;
                }
            }
        }
        changed
    }
}

impl Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.values)
    }
}
