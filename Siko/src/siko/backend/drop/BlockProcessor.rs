use core::panic;
use std::collections::BTreeMap;

use crate::siko::{
    backend::drop::{
        Context::Context,
        Path::{InstructionRef, Path},
        SingleUseVariables::SingleUseVariableInfo,
        Usage::{Usage, UsageKind},
    },
    hir::{
        Function::{Block, BlockId},
        Instruction::InstructionKind,
        Variable::Variable,
    },
};

pub struct BlockProcessor<'a> {
    singleUseVars: &'a SingleUseVariableInfo,
    receiverPaths: BTreeMap<Variable, Path>,
}

impl<'a> BlockProcessor<'a> {
    pub fn new(singleUseVars: &'a SingleUseVariableInfo) -> BlockProcessor<'a> {
        BlockProcessor {
            singleUseVars,
            receiverPaths: BTreeMap::new(),
        }
    }

    pub fn process(&mut self, block: &Block, mut context: Context) -> (Context, Vec<BlockId>) {
        //println!("Processing block: {}", block.id);
        let mut jumpTargets = Vec::new();
        let mut instructionRef = InstructionRef {
            blockId: block.id,
            instructionId: 0,
        };
        for instruction in &block.instructions {
            match &instruction.kind {
                InstructionKind::DeclareVar(var, _) => {
                    context.addLive(Path::new(var.clone(), var.location.clone()));
                }
                InstructionKind::BlockStart(_) => {}
                InstructionKind::BlockEnd(_) => {}
                InstructionKind::FunctionCall(_, _, args) => {
                    for arg in args {
                        context.useVar(arg, instructionRef);
                    }
                }
                InstructionKind::Assign(dest, src) => {
                    context.useVar(src, instructionRef);
                    let path = Path::new(dest.clone(), dest.location.clone());
                    context.addAssign(path.clone());
                }
                InstructionKind::Return(_, arg) => {
                    context.useVar(arg, instructionRef);
                }
                InstructionKind::FieldRef(dest, receiver, name) => {
                    self.processFieldRef(dest, receiver, name.clone(), &mut context);
                }
                InstructionKind::FieldAssign(dest, receiver, fields) => {
                    context.useVar(receiver, instructionRef);
                    let mut path = Path::new(dest.clone(), dest.location.clone());
                    for field in fields {
                        path = path.add(field.name.clone(), dest.location.clone());
                    }
                    context.addAssign(path.clone());
                }
                InstructionKind::Tuple(_, args) => {
                    for arg in args {
                        context.useVar(arg, instructionRef);
                    }
                }
                InstructionKind::Converter(_, _) => {
                    panic!("Converter instruction found in block processor");
                }
                InstructionKind::MethodCall(_, _, _, _) => {
                    panic!("Method call instruction found in block processor");
                }
                InstructionKind::DynamicFunctionCall(_, _, _) => {
                    panic!("Dynamic function call found in block processor");
                }
                InstructionKind::TupleIndex(dest, receiver, index) => {
                    self.processFieldRef(dest, receiver, format!(".{}", index), &mut context);
                }
                InstructionKind::Bind(_, _, _) => {
                    panic!("Bind instruction found in block processor");
                }
                InstructionKind::StringLiteral(_, _) => {}
                InstructionKind::IntegerLiteral(_, _) => {}
                InstructionKind::CharLiteral(_, _) => {}
                InstructionKind::Ref(_, src) => {
                    context.useVar(src, instructionRef);
                }
                InstructionKind::Drop(_, _) => {
                    panic!("Drop instruction found in block processor");
                }
                InstructionKind::Jump(_, blockId, _) => {
                    jumpTargets.push(blockId.clone());
                }
                InstructionKind::Transform(_, src, _) => {
                    context.useVar(src, instructionRef);
                }
                InstructionKind::EnumSwitch(var, cases) => {
                    context.useVar(var, instructionRef);
                    for case in cases {
                        jumpTargets.push(case.branch.clone());
                    }
                }
                InstructionKind::IntegerSwitch(var, cases) => {
                    context.useVar(var, instructionRef);
                    for case in cases {
                        jumpTargets.push(case.branch.clone());
                    }
                }
                InstructionKind::StringSwitch(var, cases) => {
                    context.useVar(var, instructionRef);
                    for case in cases {
                        jumpTargets.push(case.branch.clone());
                    }
                }
            }
            instructionRef.instructionId += 1;
        }
        (context, jumpTargets)
    }

    fn processFieldRef(&mut self, dest: &Variable, receiver: &Variable, name: String, context: &mut Context) {
        let destTy = dest.getType();
        let mut path = Path::new(receiver.clone(), receiver.location.clone()).add(name.clone(), dest.location.clone());
        if self.singleUseVars.isSingleUse(&dest.value) && self.singleUseVars.isReceiver(&dest.value) {
            self.receiverPaths.insert(dest.clone(), path.clone());
        } else {
            if let Some(origPath) = self.receiverPaths.get(receiver) {
                path = origPath.add(name.clone(), dest.location.clone());
            }
            if destTy.isReference() {
                context.addUsage(Usage {
                    path,
                    kind: UsageKind::Ref,
                });
            } else {
                context.addUsage(Usage {
                    path,
                    kind: UsageKind::Move,
                });
            }
        }
    }
}
