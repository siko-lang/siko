use std::collections::BTreeMap;

use crate::siko::hir::{
    BodyBuilder::BodyBuilder, Function::Function, Instruction::InstructionKind, Program::Program,
    Variable::VariableName,
};

pub fn eliminateUnusedVariable(f: &Function, program: &Program) -> Option<Function> {
    let mut eliminator = UnusedVariableEliminator::new(f, program);
    eliminator.process()
}

pub struct UnusedVariableEliminator<'a> {
    program: &'a Program,
    function: &'a Function,
    useCount: BTreeMap<VariableName, usize>,
}

impl<'a> UnusedVariableEliminator<'a> {
    pub fn new(f: &'a Function, program: &'a Program) -> UnusedVariableEliminator<'a> {
        UnusedVariableEliminator {
            function: f,
            program,
            useCount: BTreeMap::new(),
        }
    }

    fn process(&mut self) -> Option<Function> {
        if self.function.body.is_none() {
            return None;
        }

        //println!("UnusedVariableEliminator processing function: {}", self.function.name);
        //println!("{}", self.function);

        if let Some(body) = &self.function.body {
            for (_, block) in body.blocks.iter() {
                let inner = block.getInner();
                for i in inner.borrow().instructions.iter() {
                    let mut allVars = i.kind.collectVariables();
                    if let Some(v) = i.kind.getResultVar() {
                        self.useCount.entry(v.name()).or_insert(0);
                        allVars.retain(|var| var != &v);
                        if let InstructionKind::StorePtr(v1, v2) = &i.kind {
                            let c = self.useCount.entry(v1.name()).or_insert(0);
                            *c += 1;
                            let c = self.useCount.entry(v2.name()).or_insert(0);
                            *c += 1;
                        }
                    }
                    for v in allVars {
                        let count = self.useCount.entry(v.name()).or_insert(0);
                        *count += 1;
                    }
                }
            }
        }

        let mut needsRemoval = false;

        for (v, count) in &self.useCount {
            if v.isTemp() {
                if *count == 0 {
                    //println!("Unused variable: {}", v);
                    needsRemoval = true;
                }
            }
        }

        if !needsRemoval {
            return None; // No dead code found
        }

        let mut removed = false;

        let mut bodyBuilder = BodyBuilder::cloneFunction(self.function);

        let allblockIds = bodyBuilder.getAllBlockIds();
        for blockId in &allblockIds {
            let mut builder = bodyBuilder.iterator(*blockId);
            loop {
                if let Some(i) = builder.getInstruction() {
                    if let Some(v) = i.kind.getResultVar() {
                        if v.isTemp() {
                            if let Some(count) = self.useCount.get(&v.name()) {
                                if *count == 0 && self.canBeEliminated(&i.kind) {
                                    //println!("Removing unused variable: {} from {}", v.name, i);
                                    removed = true;
                                    builder.removeInstruction();
                                    continue;
                                }
                            }
                        }
                    }
                    builder.step();
                } else {
                    break;
                }
            }
            if builder.getBlockSize() == 0 {
                bodyBuilder.removeBlock(*blockId);
            }
        }

        if !removed {
            return None; // No dead code found
        }

        let mut f = self.function.clone();
        f.body = Some(bodyBuilder.build());
        Some(f)
    }

    fn canBeEliminated(&self, i: &InstructionKind) -> bool {
        match i {
            InstructionKind::DeclareVar(_, _) => true,
            InstructionKind::FieldRef(_, _, _) => true,
            InstructionKind::Assign(_, _) => true,
            InstructionKind::FunctionCall(_, info) => {
                let f = match self.program.getFunction(&info.name) {
                    Some(f) => f,
                    None => {
                        panic!("Function not found: {}", info.name);
                    }
                };
                f.isPure()
            }
            _ => false,
        }
    }
}
