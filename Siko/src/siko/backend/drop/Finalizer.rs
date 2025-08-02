use std::collections::VecDeque;

use crate::siko::{
    backend::drop::{DeclarationStore::DeclarationStore, DropList::DropListHandler},
    hir::{
        BodyBuilder::BodyBuilder,
        Function::{BlockId, Function},
        Instruction::{EnumCase, InstructionKind},
        Program::Program,
        Type::Type,
    },
};

pub struct Finalizer<'a> {
    bodyBuilder: BodyBuilder,
    function: &'a Function,
    program: &'a Program,
    dropListHandler: &'a mut DropListHandler,
    declarationStore: &'a DeclarationStore,
}

impl<'a> Finalizer<'a> {
    pub fn new(
        f: &'a Function,
        program: &'a Program,
        dropListHandler: &'a mut DropListHandler,
        declarationStore: &'a DeclarationStore,
    ) -> Finalizer<'a> {
        Finalizer {
            bodyBuilder: BodyBuilder::cloneFunction(f),
            function: f,
            program: program,
            dropListHandler,
            declarationStore,
        }
    }

    pub fn process(&mut self) -> Function {
        if self.function.body.is_none() {
            return self.function.clone();
        }
        //println!("Drop initializer processing function: {}", self.function.name);
        //println!("{}\n", self.function);

        let allBlocksIds = self.bodyBuilder.getAllBlockIds();
        for blockId in &allBlocksIds {
            let mut builder = self.bodyBuilder.iterator(*blockId);
            loop {
                match builder.getInstruction() {
                    Some(instruction) => {
                        // Process the instruction
                        match &instruction.kind {
                            InstructionKind::DropListPlaceholder(_) => {
                                // println!("Processing DropListPlaceholder at index: {}", index);
                                // let dropList = self.dropListHandler.getDropList(*index);
                                // for p in dropList.paths() {
                                //     println!("Dropping path: {} {:?}", p, p);
                                // }
                                builder.removeInstruction();
                            }
                            InstructionKind::BlockEnd(id) => {
                                if let Some(droppedValues) = self.declarationStore.getDeclarations(&id) {
                                    for var in droppedValues {
                                        let mut dropRes =
                                            self.bodyBuilder.createTempValue(instruction.location.clone());
                                        dropRes.ty = Some(Type::getUnitType());
                                        let drop = InstructionKind::Drop(dropRes, var.clone());
                                        builder.addInstruction(drop, var.location.clone());
                                        builder.step();
                                    }
                                }
                            }
                            _ => {}
                        }
                        builder.step();
                    }
                    None => break,
                }
            }
        }

        let mut queue = self.bodyBuilder.getAllBlockIds();
        loop {
            match queue.pop_front() {
                Some(blockId) => {
                    // Process the block
                    self.processBlock(blockId, &mut queue);
                }
                None => break,
            }
        }

        let mut result = self.function.clone();
        result.body = Some(self.bodyBuilder.build());
        result
    }

    fn processBlock(&mut self, blockId: BlockId, queue: &mut VecDeque<BlockId>) {
        let mut builder = self.bodyBuilder.iterator(blockId);
        loop {
            match builder.getInstruction() {
                Some(instruction) => {
                    // Process the instruction
                    match &instruction.kind {
                        InstructionKind::Drop(_, dropVar) => {
                            let newBlockId = builder.cutBlock(1);
                            let mut dropBlock = self.bodyBuilder.createBlock();
                            dropBlock.addInstruction(instruction.kind.clone(), instruction.location.clone());
                            dropBlock.addJump(newBlockId, instruction.location.clone());
                            let cases = vec![
                                EnumCase {
                                    index: 0,
                                    branch: newBlockId,
                                },
                                EnumCase {
                                    index: 1,
                                    branch: dropBlock.getBlockId(),
                                },
                            ];
                            builder.replaceInstruction(
                                InstructionKind::EnumSwitch(dropVar.getDropFlag(), cases),
                                instruction.location.clone(),
                            );
                            queue.push_back(newBlockId);
                            return;
                        }
                        _ => {}
                    }
                    builder.step();
                }
                None => break,
            }
        }
    }
}
