use std::collections::BTreeMap;

use crate::siko::hir::{
    BodyBuilder::BodyBuilder,
    Function::Function,
    Instruction::{FieldInfo, InstructionKind},
    Program::Program,
    Variable::Variable,
};

pub fn mergeFieldRefs(program: Program) -> Program {
    let mut result = program.clone();
    for (name, f) in &program.functions {
        let mut merger = FieldRefMerger::new(f);
        let f = merger.process();
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
        let mut bodyBuilder = BodyBuilder::cloneFunction(self.function);

        let mut fieldRefInfos = BTreeMap::new();

        let allBlockIds = bodyBuilder.getAllBlockIds();
        for blockId in &allBlockIds {
            let mut builder = bodyBuilder.iterator(*blockId);
            loop {
                if let Some(instruction) = builder.getInstruction() {
                    if let InstructionKind::FieldRef(dest, receiver, fields) = instruction.kind {
                        if receiver.isTemp() {
                            let info = FieldRefInfo {
                                dest: dest,
                                receiver: receiver.clone(),
                                fields: fields,
                            };
                            fieldRefInfos.insert(receiver, info);
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
                    if let InstructionKind::FieldRef(dest, receiver, mut fields) = instruction.kind {
                        if let Some(info) = fieldRefInfos.get(&dest) {
                            fields.extend(info.fields.clone());
                            let kind = InstructionKind::FieldRef(info.dest.clone(), receiver.clone(), fields);
                            //println!("Replacing {}", kind);
                            builder.replaceInstruction(kind, instruction.location.clone());
                            fieldRefInfos.remove(&dest);
                            continue;
                        }
                        if !receiver.isTemp() {
                            builder.step();
                            continue;
                        }
                        if let None = fieldRefInfos.get(&receiver) {
                            //println!("Removing instruction {} {}", dest, receiver);
                            builder.removeInstruction();
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
