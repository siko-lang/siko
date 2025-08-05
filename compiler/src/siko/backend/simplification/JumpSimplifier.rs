use std::collections::BTreeMap;

use crate::siko::hir::{
    BodyBuilder::BodyBuilder,
    Function::{BlockId, Function},
    Instruction::InstructionKind,
};

pub fn simplifyFunction(f: &Function) -> Option<Function> {
    let mut simplifier = JumpSimplifier::new(&f);
    return simplifier.process();
}

pub struct JumpSimplifier<'a> {
    function: &'a Function,
    jumps: BTreeMap<BlockId, BlockId>,
}

impl<'a> JumpSimplifier<'a> {
    pub fn new(f: &'a Function) -> JumpSimplifier<'a> {
        JumpSimplifier {
            function: f,
            jumps: BTreeMap::new(),
        }
    }

    fn replace(&mut self, old: BlockId) -> Option<BlockId> {
        let mut current = old;
        while let Some(new) = self.jumps.get(&current) {
            current = *new;
        }
        if current != old {
            Some(current)
        } else {
            None
        }
    }

    fn process(&mut self) -> Option<Function> {
        if self.function.body.is_none() {
            return None;
        }

        // println!("JumpSimplifier processing function: {}", self.function.name);
        // println!("{}", self.function);

        let mut bodyBuilder = BodyBuilder::cloneFunction(self.function);

        let allBlockIds = bodyBuilder.getAllBlockIds();
        for blockId in &allBlockIds {
            let builder = bodyBuilder.iterator(*blockId);
            if builder.getBlockSize() == 1 {
                if let Some(instruction) = builder.getInstruction() {
                    if let InstructionKind::Jump(_, target) = &instruction.kind {
                        self.jumps.insert(*blockId, *target);
                    }
                }
            }
        }

        if self.jumps.is_empty() {
            return None; // No jumps to simplify
        }

        for (blockId, _) in &self.jumps {
            bodyBuilder.removeBlock(*blockId);
            //println!("Removing block {} with jump to {}", blockId, target);
        }

        let allBlockIds = bodyBuilder.getAllBlockIds();
        for blockId in &allBlockIds {
            let mut builder = bodyBuilder.iterator(*blockId);
            loop {
                if let Some(instruction) = builder.getInstruction() {
                    if let InstructionKind::Jump(result, target) = &instruction.kind {
                        if let Some(newTarget) = self.replace(*target) {
                            builder.replaceInstruction(
                                InstructionKind::Jump(result.clone(), newTarget),
                                instruction.location.clone(),
                            );
                        }
                    }
                    if let InstructionKind::EnumSwitch(var, cases) = &instruction.kind {
                        let mut newCases = cases.clone();
                        for case in &mut newCases {
                            if let Some(newTarget) = self.replace(case.branch) {
                                case.branch = newTarget;
                            }
                        }
                        builder.replaceInstruction(
                            InstructionKind::EnumSwitch(var.clone(), newCases),
                            instruction.location.clone(),
                        );
                    }
                    if let InstructionKind::StringSwitch(var, cases) = &instruction.kind {
                        let mut newCases = cases.clone();
                        for case in &mut newCases {
                            if let Some(newTarget) = self.replace(case.branch) {
                                case.branch = newTarget;
                            }
                        }
                        builder.replaceInstruction(
                            InstructionKind::StringSwitch(var.clone(), newCases),
                            instruction.location.clone(),
                        );
                    }
                    if let InstructionKind::IntegerSwitch(var, cases) = &instruction.kind {
                        let mut newCases = cases.clone();
                        for case in &mut newCases {
                            if let Some(newTarget) = self.replace(case.branch) {
                                case.branch = newTarget;
                            }
                        }
                        builder.replaceInstruction(
                            InstructionKind::IntegerSwitch(var.clone(), newCases),
                            instruction.location.clone(),
                        );
                    }
                    builder.step();
                } else {
                    break;
                }
            }
        }

        let mut f = self.function.clone();
        f.body = Some(bodyBuilder.build());
        Some(f)
    }
}
