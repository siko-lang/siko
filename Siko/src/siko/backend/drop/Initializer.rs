use std::collections::{BTreeMap, BTreeSet};

use crate::siko::{
    backend::drop::{
        DropList::{DropListHandler, Kind},
        Path::Path,
        Util::buildFieldPath,
    },
    hir::{
        BodyBuilder::BodyBuilder, Function::Function, Instruction::InstructionKind, Program::Program,
        Variable::Variable,
    },
};

pub struct Initializer<'a> {
    bodyBuilder: BodyBuilder,
    function: &'a Function,
    program: &'a Program,
    assignDestinations: BTreeSet<Variable>,
    implicitDestinations: BTreeSet<Variable>,
    destCounts: BTreeMap<Variable, usize>,
    dropListHandler: &'a mut DropListHandler,
}

impl<'a> Initializer<'a> {
    pub fn new(f: &'a Function, program: &'a Program, dropListHandler: &'a mut DropListHandler) -> Initializer<'a> {
        Initializer {
            bodyBuilder: BodyBuilder::cloneFunction(f),
            function: f,
            program: program,
            assignDestinations: BTreeSet::new(),
            implicitDestinations: BTreeSet::new(),
            destCounts: BTreeMap::new(),
            dropListHandler,
        }
    }

    fn addDest(&mut self, var: &Variable) {
        let count = self.destCounts.entry(var.clone()).or_insert(0);
        *count += 1;
    }

    pub fn process(&mut self) -> Function {
        if self.function.body.is_none() {
            return self.function.clone();
        }
        //println!("Drop initializer processing function: {}", self.function.name);

        let allBlocksIds = self.bodyBuilder.getAllBlockIds();
        let mut placeHolderIndex = 0;
        for blockId in &allBlocksIds {
            let mut builder = self.bodyBuilder.iterator(*blockId);
            loop {
                match builder.getInstruction() {
                    Some(instruction) => {
                        // Process the instruction
                        match &instruction.kind {
                            InstructionKind::BlockEnd(_) => {}
                            InstructionKind::Assign(dest, _) => {
                                self.dropListHandler.createDropList(
                                    placeHolderIndex,
                                    Kind::VariableAssign(
                                        Path::new(dest.clone(), instruction.location.clone()).toSimplePath(),
                                    ),
                                );
                                builder.addInstruction(
                                    InstructionKind::DropListPlaceholder(placeHolderIndex),
                                    instruction.location.clone(),
                                );
                                placeHolderIndex += 1;
                                builder.step();
                            }
                            InstructionKind::FieldAssign(dest, _, fields) => {
                                let path = buildFieldPath(dest, fields);
                                self.dropListHandler
                                    .createDropList(placeHolderIndex, Kind::FieldAssign(path.toSimplePath()));
                                self.dropListHandler.addPath(placeHolderIndex, path);
                                builder.addInstruction(
                                    InstructionKind::DropListPlaceholder(placeHolderIndex),
                                    instruction.location.clone(),
                                );
                                placeHolderIndex += 1;
                                builder.step();
                            }
                            InstructionKind::DeclareVar(_, _) => {}
                            kind => {
                                if let Some(result) = kind.getResultVar() {
                                    if !result.isTemp() {
                                        panic!(
                                            "Implicit destination should be a temporary variable, but found: {}",
                                            result
                                        );
                                    }
                                    self.addDest(&result);
                                }
                            }
                        }
                        builder.step();
                    }
                    None => break,
                }
            }
        }

        for (var, count) in &self.destCounts {
            if *count > 1 {
                panic!(
                    "Variable {} is assigned more than once, but is temporary and should be only assigned once.",
                    var
                );
            }
        }

        let mut result = self.function.clone();
        result.body = Some(self.bodyBuilder.build());
        result
    }
}
