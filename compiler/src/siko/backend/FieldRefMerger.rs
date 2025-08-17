use std::collections::BTreeMap;

use crate::siko::{
    backend::simplification::UnusedVariableEliminator,
    hir::{
        BodyBuilder::BodyBuilder,
        Function::Function,
        Instruction::{FieldInfo, InstructionKind},
        Program::Program,
        Variable::Variable,
    },
};

pub fn mergeFieldRefs(program: Program) -> Program {
    let mut result = program.clone();
    for (name, f) in &program.functions {
        let mut merger = FieldRefMerger::new(f);
        let mut f = merger.process();
        if let Some(updatedF) = UnusedVariableEliminator::eliminateUnusedVariable(&f) {
            f = updatedF;
        }
        result.functions.insert(name.clone(), f);
    }
    result
}

struct FieldRefInfo {
    dest: Variable,
    receiver: Variable,
    fields: Vec<FieldInfo>,
}

pub struct FieldRefMerger<'a> {
    function: &'a Function,
}

impl<'a> FieldRefMerger<'a> {
    pub fn new(f: &'a Function) -> FieldRefMerger<'a> {
        FieldRefMerger { function: f }
    }

    fn process(&mut self) -> Function {
        if self.function.body.is_none() {
            return self.function.clone();
        }

        //println!("Processing function: {}", self.function);

        let mut bodyBuilder = BodyBuilder::cloneFunction(self.function);

        let mut fieldRefInfos = BTreeMap::new();

        let allBlockIds = bodyBuilder.getAllBlockIds();
        for blockId in &allBlockIds {
            let mut builder = bodyBuilder.iterator(*blockId);
            loop {
                if let Some(instruction) = builder.getInstruction() {
                    if let InstructionKind::FieldRef(dest, receiver, fields) = instruction.kind {
                        if dest.isTemp() {
                            let info = FieldRefInfo {
                                dest: dest.clone(),
                                receiver: receiver.clone(),
                                fields: fields,
                            };
                            //println!("Found field ref: {} -> {}", receiver, dest);
                            fieldRefInfos.insert(dest, info);
                        }
                    }
                    builder.step();
                } else {
                    break;
                }
            }
        }
        for blockId in &allBlockIds {
            let mut builder = bodyBuilder.iterator(*blockId);
            loop {
                if let Some(instruction) = builder.getInstruction() {
                    //println!("Processing instruction: {}", instruction);
                    if let InstructionKind::FieldRef(dest, receiver, fields) = &instruction.kind {
                        if let Some(info) = fieldRefInfos.get(&receiver) {
                            let mut fields = fields.clone();
                            fields.insert(0, info.fields[0].clone());
                            let kind = InstructionKind::FieldRef(dest.clone(), info.receiver.clone(), fields);
                            builder.replaceInstruction(kind, instruction.location.clone());
                            continue;
                        }
                        if !receiver.isTemp() {
                            builder.step();
                            continue;
                        }
                    }
                    builder.step();
                } else {
                    break;
                }
            }
        }
        let mut f = self.function.clone();
        f.body = Some(bodyBuilder.build());
        f
    }
}
