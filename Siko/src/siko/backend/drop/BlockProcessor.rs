use core::panic;
use std::collections::BTreeMap;

use crate::siko::{
    backend::drop::{
        Context::Context,
        Path::{InstructionRef, Path, PathSegment},
        Usage::{Usage, UsageKind},
    },
    hir::{
        Function::{Block, BlockId},
        Instruction::{FieldId, InstructionKind},
        Variable::Variable,
    },
};

pub struct BlockProcessor {
    receiverPaths: BTreeMap<Variable, Path>,
}

impl BlockProcessor {
    pub fn new() -> BlockProcessor {
        BlockProcessor {
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
                    let mut path = Path::new(dest.clone(), dest.location.clone());
                    path = path.setInstructionRef(instructionRef);
                    context.addAssign(path.clone());
                }
                InstructionKind::Return(_, arg) => {
                    context.useVar(arg, instructionRef);
                }
                InstructionKind::FieldRef(dest, receiver, names) => {
                    let destTy = dest.getType();
                    let mut path = Path::new(receiver.clone(), dest.location.clone());
                    for field in names {
                        match &field.name {
                            FieldId::Named(name) => {
                                path = path.add(PathSegment::Named(name.clone()), dest.location.clone());
                            }
                            FieldId::Indexed(index) => {
                                path = path.add(PathSegment::Indexed(*index), dest.location.clone());
                            }
                        }
                    }
                    path = path.setInstructionRef(instructionRef);
                    if destTy.isReference() || destTy.isPtr() {
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
                InstructionKind::FieldAssign(dest, receiver, fields) => {
                    context.useVar(receiver, instructionRef);
                    let mut path = Path::new(dest.clone(), dest.location.clone());
                    for field in fields {
                        match &field.name {
                            FieldId::Named(name) => {
                                path = path.add(PathSegment::Named(name.clone()), dest.location.clone());
                            }
                            FieldId::Indexed(index) => {
                                path = path.add(PathSegment::Indexed(*index), dest.location.clone());
                            }
                        }
                    }
                    path = path.setInstructionRef(instructionRef);
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
}
