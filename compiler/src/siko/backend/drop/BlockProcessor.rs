use core::panic;
use std::collections::BTreeMap;

use crate::siko::{
    backend::drop::{
        Context::Context,
        DropList::DropListHandler,
        Path::{InstructionRef, Path, PathSegment},
        Usage::{Usage, UsageKind},
        Util::buildFieldPath,
    },
    hir::{
        Function::{Block, BlockId},
        Instruction::{FieldId, InstructionKind},
        Variable::Variable,
    },
};

pub struct BlockProcessor<'a> {
    receiverPaths: BTreeMap<Variable, Path>,
    dropListHandler: &'a mut DropListHandler,
}

impl<'a> BlockProcessor<'a> {
    pub fn new(dropListHandler: &'a mut DropListHandler) -> BlockProcessor<'a> {
        BlockProcessor {
            receiverPaths: BTreeMap::new(),
            dropListHandler,
        }
    }

    pub fn process(&mut self, block: &Block, mut context: Context) -> (Context, Vec<BlockId>) {
        // println!("Processing block: {}", block.id);

        // println!("starting context: {}", context);
        // println!("--------------");
        let mut jumpTargets = Vec::new();
        let mut instructionRef = InstructionRef {
            blockId: block.id,
            instructionId: 0,
        };
        for instruction in &block.instructions {
            // println!(
            //     "Processing instruction: {} {} {}",
            //     instruction.kind, instruction.location, instructionRef
            // );
            match &instruction.kind {
                InstructionKind::DeclareVar(var, _) => {
                    context.addLive(Path::new(var.clone(), var.location.clone()));
                }
                InstructionKind::BlockStart(_) => {}
                InstructionKind::BlockEnd(_) => {}
                InstructionKind::FunctionCall(dest, _, args) => {
                    for arg in args {
                        context.useVar(arg, instructionRef);
                    }
                    let mut path = Path::new(dest.clone(), dest.location.clone());
                    path = path.setInstructionRef(instructionRef);
                    context.addAssign(path.clone());
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
                    let mut path = Path::new(dest.clone(), dest.location.clone());
                    path = path.setInstructionRef(instructionRef);
                    context.addAssign(path.clone());
                }
                InstructionKind::FieldAssign(dest, receiver, fields) => {
                    context.useVar(receiver, instructionRef);
                    let mut path = buildFieldPath(dest, fields);
                    path = path.setInstructionRef(instructionRef);
                    context.addAssign(path.clone());
                }
                InstructionKind::Tuple(dest, args) => {
                    let mut path = Path::new(dest.clone(), dest.location.clone());
                    path = path.setInstructionRef(instructionRef);
                    context.addAssign(path.clone());
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
                InstructionKind::Ref(_, var) => {
                    context.addUsage(Usage {
                        path: Path::new(var.clone(), var.location.clone()).setInstructionRef(instructionRef),
                        kind: UsageKind::Ref,
                    });
                }
                InstructionKind::DropListPlaceholder(_) => {}
                InstructionKind::Drop(_, _) => {
                    panic!("Drop instruction found in block processor");
                }
                InstructionKind::Jump(_, blockId) => {
                    jumpTargets.push(blockId.clone());
                }
                InstructionKind::Transform(_, src, _) => {
                    context.useVar(src, instructionRef);
                }
                InstructionKind::EnumSwitch(_, cases) => {
                    // enum switch does not 'use' the variable, transform does
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
        let allDropListIds = self.dropListHandler.getDropListIds();
        for id in allDropListIds {
            let dropList = self.dropListHandler.getDropList(id);
            let rootPath = dropList.getRoot();
            for (name, events) in context.usages.iter() {
                if name.visibleName() == rootPath.root {
                    for path in events.getAllPaths() {
                        if path.toSimplePath().sharesPrefixWith(&rootPath) {
                            self.dropListHandler.addPath(id, path.clone());
                        }
                    }
                }
            }
        }
        //println!("Final context: {}", context);
        (context, jumpTargets)
    }
}
