use std::collections::BTreeSet;

use crate::siko::hir::{
    BodyBuilder::BodyBuilder, Function::Function, Instruction::InstructionKind, Variable::VariableName,
};

pub struct ReferenceStore {
    references: BTreeSet<VariableName>,
}

impl ReferenceStore {
    pub fn new() -> Self {
        ReferenceStore {
            references: BTreeSet::new(),
        }
    }

    pub fn addReference(&mut self, var_name: VariableName) {
        self.references.insert(var_name);
    }

    pub fn isReference(&self, var_name: &VariableName) -> bool {
        self.references.contains(var_name)
    }

    pub fn build<'a>(&mut self, f: &'a Function) {
        let mut bodyBuilder = BodyBuilder::cloneFunction(f);
        for blockId in bodyBuilder.getAllBlockIds() {
            let mut builder = bodyBuilder.iterator(blockId);
            loop {
                if let Some(instruction) = builder.getInstruction() {
                    if let InstructionKind::Ref(_, src) = &instruction.kind {
                        self.references.insert(src.name());
                    }
                    builder.step();
                } else {
                    break;
                }
            }
        }
    }
}
