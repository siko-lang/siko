use std::collections::{BTreeMap, BTreeSet};

use crate::siko::{
    backend::drop::{
        DeclarationStore::DeclarationStore,
        DropList::{DropListHandler, Kind},
        Path::Path,
        Util::{buildFieldPath, HasTrivialDrop},
    },
    hir::{
        BlockBuilder::BlockBuilder,
        BodyBuilder::BodyBuilder,
        Function::Function,
        Instruction::{InstructionKind, Mutability, SyntaxBlockId},
        Program::Program,
        Variable::Variable,
    },
    qualifiedname::{getFalseName, getTrueName},
};

pub struct Initializer<'a> {
    bodyBuilder: BodyBuilder,
    function: &'a Function,
    program: &'a Program,
    assignDestinations: BTreeSet<Variable>,
    implicitDestinations: BTreeSet<Variable>,
    destCounts: BTreeMap<Variable, usize>,
    dropListHandler: &'a mut DropListHandler,
    declarationStore: &'a mut DeclarationStore,
}

impl<'a> Initializer<'a> {
    pub fn new(
        f: &'a Function,
        program: &'a Program,
        dropListHandler: &'a mut DropListHandler,
        declarationStore: &'a mut DeclarationStore,
    ) -> Initializer<'a> {
        Initializer {
            bodyBuilder: BodyBuilder::cloneFunction(f),
            function: f,
            program: program,
            assignDestinations: BTreeSet::new(),
            implicitDestinations: BTreeSet::new(),
            destCounts: BTreeMap::new(),
            dropListHandler,
            declarationStore,
        }
    }

    fn addDest(&mut self, var: &Variable) {
        let count = self.destCounts.entry(var.clone()).or_insert(0);
        *count += 1;
    }

    fn declareVar(&mut self, var: &Variable, syntaxBlock: &SyntaxBlockId, builder: &mut BlockBuilder) {
        if var.hasTrivialDrop() || var.isArg() {
            return;
        }
        if self.declarationStore.declare(var.clone(), syntaxBlock.clone()) {
            let dropFlag = var.getDropFlag();
            builder.addInstruction(
                InstructionKind::DeclareVar(dropFlag.clone(), Mutability::Mutable),
                var.location.clone(),
            );
            builder.step();
            builder.addInstruction(
                InstructionKind::FunctionCall(dropFlag, getFalseName(), vec![]),
                var.location.clone(),
            );
            builder.step();
        }
    }

    fn useVar(&mut self, var: &Variable, builder: &mut BlockBuilder) {
        if var.hasTrivialDrop() || var.isArg() {
            return;
        }
        let dropFlag = var.getDropFlag();
        builder.addInstruction(
            InstructionKind::FunctionCall(dropFlag, getFalseName(), vec![]),
            var.location.clone(),
        );
        builder.step();
    }

    pub fn process(&mut self) -> Function {
        if self.function.body.is_none() {
            return self.function.clone();
        }
        //println!("Drop initializer processing function: {}", self.function.name);

        let allBlocksIds = self.bodyBuilder.getAllBlockIds();
        let mut placeHolderIndex = 0;

        let mut blockSyntaxBlocks = BTreeMap::new();
        for blockId in &allBlocksIds {
            blockSyntaxBlocks.insert(*blockId, SyntaxBlockId::new());
        }

        for blockId in &allBlocksIds {
            let mut currentSyntaxBlock = blockSyntaxBlocks.get(blockId).expect("Block not found").clone();

            let mut builder = self.bodyBuilder.iterator(*blockId);
            loop {
                match builder.getInstruction() {
                    Some(instruction) => {
                        // Process the instruction
                        match &instruction.kind {
                            InstructionKind::BlockStart(blockId) => {
                                currentSyntaxBlock = blockId.clone();
                            }
                            InstructionKind::BlockEnd(blockId) => {
                                currentSyntaxBlock = blockId.getParent();
                            }
                            InstructionKind::Assign(dest, src) => {
                                self.declareVar(dest, &currentSyntaxBlock, &mut builder);
                                self.useVar(src, &mut builder);
                                if !dest.hasTrivialDrop() {
                                    builder.addInstruction(
                                        InstructionKind::FunctionCall(dest.getDropFlag(), getTrueName(), vec![]),
                                        instruction.location.clone(),
                                    );
                                    builder.step();
                                }
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
                                builder.step();
                                placeHolderIndex += 1;
                            }
                            InstructionKind::FieldAssign(dest, _, fields) => {
                                self.declareVar(dest, &currentSyntaxBlock, &mut builder);
                                let path = buildFieldPath(dest, fields);
                                self.dropListHandler
                                    .createDropList(placeHolderIndex, Kind::FieldAssign(path.toSimplePath()));
                                self.dropListHandler.addPath(placeHolderIndex, path);
                                builder.addInstruction(
                                    InstructionKind::DropListPlaceholder(placeHolderIndex),
                                    instruction.location.clone(),
                                );
                                builder.step();
                                placeHolderIndex += 1;
                            }
                            InstructionKind::DeclareVar(var, _) => {
                                self.declareVar(var, &currentSyntaxBlock, &mut builder);
                            }
                            InstructionKind::Jump(_, targetBlock) => {
                                blockSyntaxBlocks.insert(*targetBlock, currentSyntaxBlock.clone());
                            }
                            InstructionKind::EnumSwitch(_, cases) => {
                                for case in cases {
                                    blockSyntaxBlocks.insert(case.branch, currentSyntaxBlock.clone());
                                }
                            }
                            InstructionKind::StringSwitch(_, cases) => {
                                for case in cases {
                                    blockSyntaxBlocks.insert(case.branch, currentSyntaxBlock.clone());
                                }
                            }
                            InstructionKind::IntegerSwitch(_, cases) => {
                                for case in cases {
                                    blockSyntaxBlocks.insert(case.branch, currentSyntaxBlock.clone());
                                }
                            }
                            InstructionKind::Return(_, _) => {
                                // No targets to propagate to
                            }
                            kind => {
                                let mut allUsedVars = kind.collectVariables();
                                if let Some(result) = kind.getResultVar() {
                                    allUsedVars.retain(|var| var != &result);
                                    self.declareVar(&result, &currentSyntaxBlock, &mut builder);
                                    if !result.isTemp() && !result.isDropFlag() {
                                        panic!(
                                            "Implicit destination should be a temporary variable, but found: {}",
                                            result
                                        );
                                    }
                                    self.addDest(&result);
                                }
                                for var in allUsedVars {
                                    self.useVar(&var, &mut builder);
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
