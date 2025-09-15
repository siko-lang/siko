use std::collections::BTreeSet;

use crate::siko::hir::{BodyBuilder::BodyBuilder, Function::Function, Instruction::InstructionKind, Type::Type};

pub fn simplifyFunction(f: &Function) -> Option<Function> {
    let mut simplifier = SwitchSimplifier::new(&f);
    return simplifier.process();
}

pub struct SwitchSimplifier<'a> {
    function: &'a Function,
}

impl<'a> SwitchSimplifier<'a> {
    pub fn new(f: &'a Function) -> SwitchSimplifier<'a> {
        SwitchSimplifier { function: f }
    }

    fn process(&mut self) -> Option<Function> {
        if self.function.body.is_none() {
            return None;
        }

        //println!("SwitchSimplifier processing function: {}", self.function.name);
        //println!("{}", self.function);

        let mut bodyBuilder = BodyBuilder::cloneFunction(self.function);

        let mut changed = false;
        let allBlockIds = bodyBuilder.getAllBlockIds();
        for blockId in &allBlockIds {
            let mut builder = bodyBuilder.iterator(*blockId);
            loop {
                if let Some(instruction) = builder.getInstruction() {
                    if let InstructionKind::EnumSwitch(_, cases) = &instruction.kind {
                        // If all cases go to the same block, replace with a jump
                        let mut targetBlocks = BTreeSet::new();
                        for case in cases {
                            targetBlocks.insert(case.branch);
                        }
                        //println!("SwitchSimplifier: EnumSwitch target blocks: {:?}", targetBlocks);
                        if targetBlocks.len() == 1 {
                            //println!("SwitchSimplifier: Replacing EnumSwitch with Jump");
                            let jumpVar =
                                bodyBuilder.createTempValueWithType(instruction.location.clone(), Type::getNeverType());
                            let kind = InstructionKind::Jump(jumpVar, targetBlocks.iter().next().unwrap().clone());
                            builder.replaceInstruction(kind, instruction.location.clone());
                            changed = true;
                        }
                    }
                    if let InstructionKind::IntegerSwitch(_, cases) = &instruction.kind {
                        // If all cases go to the same block, replace with a jump
                        let mut targetBlocks = BTreeSet::new();
                        for case in cases {
                            targetBlocks.insert(case.branch);
                        }
                        //println!("SwitchSimplifier: IntegerSwitch target blocks: {:?}", targetBlocks);
                        if targetBlocks.len() == 1 {
                            //println!("SwitchSimplifier: Replacing IntegerSwitch with Jump");
                            let jumpVar =
                                bodyBuilder.createTempValueWithType(instruction.location.clone(), Type::getNeverType());
                            let kind = InstructionKind::Jump(jumpVar, targetBlocks.iter().next().unwrap().clone());
                            builder.replaceInstruction(kind, instruction.location.clone());
                            changed = true;
                        }
                    }
                    builder.step();
                } else {
                    break;
                }
            }
        }

        if !changed {
            return None; // No switches to simplify
        }

        let mut f = self.function.clone();
        f.body = Some(bodyBuilder.build());

        //println!("After SwitchSimplifier:");
        //println!("{}", f);

        Some(f)
    }
}
