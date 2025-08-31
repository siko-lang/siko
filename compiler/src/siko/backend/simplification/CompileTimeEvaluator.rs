use std::collections::BTreeMap;

use crate::siko::{
    hir::{
        Block::BlockId, BlockBuilder::BlockBuilder, BodyBuilder::BodyBuilder, Function::Function,
        Instruction::InstructionKind, Variable::VariableName,
    },
    qualifiedname::builtins::{getFalseName, getTrueName},
};

pub fn simplifyFunction(f: &Function) -> Option<Function> {
    let mut evaluator = CompileTimeEvaluator::new(f);
    evaluator.evaluate()
}

pub struct CompileTimeEvaluator<'a> {
    function: &'a Function,
    blockEnvs: BTreeMap<BlockId, Environment>,
    queue: Vec<BlockId>,
    modified: bool,
}

impl<'a> CompileTimeEvaluator<'a> {
    pub fn new(f: &'a Function) -> CompileTimeEvaluator<'a> {
        CompileTimeEvaluator {
            function: f,
            blockEnvs: BTreeMap::new(),
            queue: Vec::new(),
            modified: false,
        }
    }

    pub fn evaluate(&mut self) -> Option<Function> {
        if self.function.body.is_none() {
            return None; // No body to evaluate
        }

        // println!("CompileTimeEvaluator processing function: {}", self.function.name);
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
            self.evaluateBlock(&mut builder, false);
        }

        for (blockId, _) in self.blockEnvs.clone() {
            let mut builder = bodyBuilder.iterator(blockId);
            self.evaluateBlock(&mut builder, true);
        }

        if !self.modified {
            return None; // No modifications made
        }
        let mut f = self.function.clone();
        f.body = Some(bodyBuilder.build());
        Some(f)
    }

    fn getEnvForBlock(&mut self, blockId: BlockId) -> &mut Environment {
        self.blockEnvs
            .get_mut(&blockId)
            .expect("Environment for block should exist")
    }

    fn evaluateBlock(&mut self, builder: &mut BlockBuilder, modify: bool) {
        //println!("Evaluating block: {}", builder.getBlockId());
        let mut newEnv = self.getEnvForBlock(builder.getBlockId()).clone();
        loop {
            if let Some(instruction) = builder.getInstruction() {
                //println!("Processing instruction: {}", instruction);
                match instruction.kind {
                    InstructionKind::DeclareVar(var, _) => {
                        newEnv.set(var.name(), Value::Unknown);
                    }
                    InstructionKind::Assign(var, value) => {
                        let val = newEnv.get(&value.name());
                        newEnv.set(var.name(), val.clone());
                    }
                    InstructionKind::FunctionCall(dest, info) => {
                        if info.name == getTrueName() {
                            newEnv.set(dest.name(), Value::Bool(true));
                        } else if info.name == getFalseName() {
                            newEnv.set(dest.name(), Value::Bool(false));
                        }
                    }
                    InstructionKind::EnumSwitch(var, cases) => match newEnv.get(&var.name()) {
                        Value::Bool(true) => {
                            //println!("Enum switch to true branch");
                            let targetBranch = cases[1].branch;
                            let targetEnv = self.getEnvForBlock(targetBranch);
                            if targetEnv.merge(&newEnv) {
                                //println!("enum switch to true block {} with updated environment", targetBranch);
                                self.queue.push(targetBranch);
                            }
                            if modify {
                                let jumpVar = builder.getBodyBuilder().createTempValue(instruction.location.clone());
                                let kind = InstructionKind::Jump(jumpVar, targetBranch);
                                builder.replaceInstruction(kind, instruction.location.clone());
                                self.modified = true;
                            }
                        }
                        Value::Bool(false) => {
                            //println!("Enum switch to false branch");
                            let targetBranch = cases[0].branch;
                            let targetEnv = self.getEnvForBlock(targetBranch);
                            if targetEnv.merge(&newEnv) {
                                //println!("enum switch to false block {} with updated environment", targetBranch);
                                self.queue.push(targetBranch);
                            }
                            if modify {
                                let jumpVar = builder.getBodyBuilder().createTempValue(instruction.location.clone());
                                let kind = InstructionKind::Jump(jumpVar, targetBranch);
                                builder.replaceInstruction(kind, instruction.location.clone());
                                self.modified = true;
                            }
                        }
                        _ => {
                            //println!("Enum switch with unknown value, updating all branches");
                            for case in &cases {
                                let targetBranch = case.branch;
                                let targetEnv = self.getEnvForBlock(targetBranch);
                                if targetEnv.merge(&newEnv) {
                                    //println!("enum switch to block {} with updated environment", targetBranch);
                                    self.queue.push(targetBranch);
                                }
                            }
                        }
                    },
                    InstructionKind::Jump(_, target) => {
                        let targetEnv = self.getEnvForBlock(target);
                        if targetEnv.merge(&newEnv) {
                            //println!("Jump to block {} with updated environment", target);
                            self.queue.push(target);
                        }
                    }
                    InstructionKind::Return(_, _) => {}
                    k => {
                        if let Some(result) = k.getResultVar() {
                            newEnv.set(result.name(), Value::Unknown);
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
enum Value {
    Int(i64),
    Bool(bool),
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Environment {
    values: BTreeMap<VariableName, Value>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            values: BTreeMap::new(),
        }
    }

    pub fn get(&self, var: &VariableName) -> Value {
        self.values.get(var).cloned().unwrap_or(Value::Unknown)
    }

    pub fn set(&mut self, var: VariableName, value: Value) {
        //println!("Setting variable {} to {:?}", var, value);
        self.values.insert(var, value);
    }

    fn merge(&mut self, other: &Environment) -> bool {
        let mut changed = false;
        for (var, value) in &other.values {
            if let Some(existing) = self.values.get(var) {
                if existing != value {
                    self.set(var.clone(), Value::Unknown);
                    changed = true;
                }
            } else {
                self.set(var.clone(), value.clone());
                changed = true;
            }
        }
        changed
    }
}
