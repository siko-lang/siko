use crate::siko::{
    backend::drop::DropList::DropListHandler,
    hir::{BodyBuilder::BodyBuilder, Function::Function, Instruction::InstructionKind, Program::Program},
};

pub struct Finalizer<'a> {
    bodyBuilder: BodyBuilder,
    function: &'a Function,
    program: &'a Program,
    dropListHandler: &'a mut DropListHandler,
}

impl<'a> Finalizer<'a> {
    pub fn new(f: &'a Function, program: &'a Program, dropListHandler: &'a mut DropListHandler) -> Finalizer<'a> {
        Finalizer {
            bodyBuilder: BodyBuilder::cloneFunction(f),
            function: f,
            program: program,
            dropListHandler,
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
                            InstructionKind::DropListPlaceholder(index) => {
                                // println!("Processing DropListPlaceholder at index: {}", index);
                                // let dropList = self.dropListHandler.getDropList(*index);
                                // for p in dropList.paths() {
                                //     println!("Dropping path: {} {:?}", p, p);
                                // }
                                builder.removeInstruction();
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
