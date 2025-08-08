use std::collections::{BTreeSet, VecDeque};

use crate::siko::{
    backend::drop::{
        DeclarationStore::DeclarationStore,
        DropMetadataStore::{DropMetadataStore, MetadataKind},
    },
    hir::{
        BodyBuilder::BodyBuilder,
        Function::{BlockId, Function},
        Instruction::{EnumCase, InstructionKind, Mutability},
        Program::Program,
        Type::Type,
        Variable::Variable,
    },
    qualifiedname::getFalseName,
};

pub struct Finalizer<'a> {
    bodyBuilder: BodyBuilder,
    function: &'a Function,
    program: &'a Program,
    dropMetadataStore: &'a mut DropMetadataStore,
    declarationStore: &'a DeclarationStore,
    declaredDropFlags: BTreeSet<Variable>,
}

impl<'a> Finalizer<'a> {
    pub fn new(
        f: &'a Function,
        program: &'a Program,
        dropMetadataStore: &'a mut DropMetadataStore,
        declarationStore: &'a DeclarationStore,
    ) -> Finalizer<'a> {
        Finalizer {
            bodyBuilder: BodyBuilder::cloneFunction(f),
            function: f,
            program: program,
            dropMetadataStore,
            declarationStore,
            declaredDropFlags: BTreeSet::new(),
        }
    }

    fn declareDropFlags(&mut self) {
        let mut builder = self.bodyBuilder.iterator(BlockId::first());
        for dropFlag in &self.declaredDropFlags {
            builder.addInstruction(
                InstructionKind::DeclareVar(dropFlag.clone(), Mutability::Mutable),
                dropFlag.location.clone(),
            );
            builder.step();
            builder.addInstruction(
                InstructionKind::FunctionCall(dropFlag.clone(), getFalseName(), vec![]),
                dropFlag.location.clone(),
            );
            builder.step();
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
                                // let dropList = self.dropMetadataStore.getDropList(*index);
                                // for p in dropList.paths() {
                                //     println!("Dropping path: {} {:?}", p, p);
                                // }
                                builder.removeInstruction();
                            }
                            InstructionKind::DropMetadata(kind) => {
                                match kind {
                                    MetadataKind::DeclarationList(name) => {
                                        if let Some(declarationList) = self.dropMetadataStore.getDeclarationList(name) {
                                            //println!("Processing DeclarationList: {}", name);
                                            for path in declarationList.paths() {
                                                //println!("Creating dropflag for path: {}", path);
                                                let dropFlag = path.getDropFlag();
                                                self.declaredDropFlags.insert(dropFlag.clone());
                                                builder.addInstruction(
                                                    InstructionKind::FunctionCall(dropFlag, getFalseName(), vec![]),
                                                    instruction.location.clone(),
                                                );
                                                builder.step();
                                            }
                                        }
                                    }
                                }
                                builder.removeInstruction();
                            }
                            InstructionKind::BlockStart(_) => {
                                builder.removeInstruction();
                            }
                            InstructionKind::BlockEnd(id) => {
                                if let Some(droppedValues) = self.declarationStore.getDeclarations(&id) {
                                    for var in droppedValues {
                                        let mut dropRes =
                                            self.bodyBuilder.createTempValue(instruction.location.clone());
                                        dropRes.ty = Some(Type::getUnitType());
                                        let dropFlag = var.getDropFlag();
                                        let drop = InstructionKind::Drop(dropRes, var.clone());
                                        builder.addInstruction(drop, var.location.clone());
                                        builder.step();
                                        builder.addInstruction(
                                            InstructionKind::FunctionCall(dropFlag, getFalseName(), vec![]),
                                            instruction.location.clone(),
                                        );
                                        builder.step();
                                    }
                                }
                                builder.removeInstruction();
                            }
                            _ => {
                                builder.step();
                            }
                        }
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

        self.declareDropFlags();

        let mut result = self.function.clone();
        result.body = Some(self.bodyBuilder.build());

        //println!("Finalized function: {}", result);
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
