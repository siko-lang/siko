use std::collections::BTreeMap;

use crate::siko::hir::{
    BodyBuilder::BodyBuilder, Function::Function, Instruction::InstructionKind, Variable::VariableName,
};

pub fn eliminateUnusedVariable(f: &Function) -> Option<Function> {
    let mut eliminator = UnusedVariableEliminator::new(f);
    eliminator.process()
}

pub struct UnusedVariableEliminator<'a> {
    function: &'a Function,
    useCount: BTreeMap<VariableName, usize>,
}

impl<'a> UnusedVariableEliminator<'a> {
    pub fn new(f: &'a Function) -> UnusedVariableEliminator<'a> {
        UnusedVariableEliminator {
            function: f,
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
                for i in block.instructions.iter() {
                    let mut allVars = i.kind.collectVariables();
                    if let Some(v) = i.kind.getResultVar() {
                        self.useCount.entry(v.name.clone()).or_insert(0);
                        allVars.retain(|var| var != &v);
                    }
                    for v in allVars {
                        let count = self.useCount.entry(v.name.clone()).or_insert(0);
                        *count += 1;
                    }
                }
            }
        }

        let mut needsRemoval = false;

        for (_, count) in &self.useCount {
            if *count == 0 {
                //println!("Unused variable: {}", name);
                needsRemoval = true;
            }
        }

        if !needsRemoval {
            return None; // No dead code found
        }

        let mut bodyBuilder = BodyBuilder::cloneFunction(self.function);

        let allblockIds = bodyBuilder.getAllBlockIds();
        for blockId in &allblockIds {
            let mut builder = bodyBuilder.iterator(*blockId);
            loop {
                if let Some(i) = builder.getInstruction() {
                    if let Some(v) = i.kind.getResultVar() {
                        if let Some(count) = self.useCount.get(&v.name) {
                            if *count == 0 && canBeEliminated(&i.kind) {
                                //println!("Removing unused variable: {} from {}", v.name, i);
                                builder.removeInstruction();
                                continue;
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

        let mut f = self.function.clone();
        f.body = Some(bodyBuilder.build());
        Some(f)
    }
}

fn canBeEliminated(i: &InstructionKind) -> bool {
    match i {
        InstructionKind::DeclareVar(_, _) => false,
        InstructionKind::FieldRef(_, _, _) => true,
        InstructionKind::Assign(_, _) => false,
        _ => false,
    }
}
