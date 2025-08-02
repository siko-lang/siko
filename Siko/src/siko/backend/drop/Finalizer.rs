use crate::siko::{
    backend::drop::{DeclarationStore::DeclarationStore, DropList::DropListHandler},
    hir::{BodyBuilder::BodyBuilder, Function::Function, Instruction::InstructionKind, Program::Program, Type::Type},
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
                                        let ty = var.ty.clone().expect("no type for variable in drop");
                                        if ty.isNever() || ty.isPtr() || ty.isReference() || ty.isUnit() {
                                            continue;
                                        }
                                        if true {
                                            // NYI: without drop flags this just double frees everything, disable for now
                                            continue;
                                        }
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

        let mut result = self.function.clone();
        result.body = Some(self.bodyBuilder.build());
        result
    }
}
