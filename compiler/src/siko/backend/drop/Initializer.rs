use std::collections::{BTreeMap, VecDeque};

use crate::siko::{
    backend::drop::{
        DeclarationStore::DeclarationStore,
        DropMetadataStore::{DropMetadataStore, MetadataKind},
        Usage::getUsageInfo,
        Util::HasTrivialDrop,
    },
    hir::{
        BlockBuilder::BlockBuilder,
        BodyBuilder::BodyBuilder,
        Function::{BlockId, Function},
        Instruction::{InstructionKind, SyntaxBlockId},
        Program::Program,
        Variable::Variable,
    },
};

pub struct Initializer<'a> {
    bodyBuilder: BodyBuilder,
    function: &'a Function,
    program: &'a Program,
    dropMetadataStore: &'a mut DropMetadataStore,
    declarationStore: &'a mut DeclarationStore,
    queue: VecDeque<(BlockId, SyntaxBlockId)>,
}

impl<'a> Initializer<'a> {
    pub fn new(
        f: &'a Function,
        program: &'a Program,
        dropMetadataStore: &'a mut DropMetadataStore,
        declarationStore: &'a mut DeclarationStore,
    ) -> Initializer<'a> {
        Initializer {
            bodyBuilder: BodyBuilder::cloneFunction(f),
            function: f,
            program: program,
            dropMetadataStore,
            declarationStore,
            queue: VecDeque::new(),
        }
    }

    fn declareVar(&mut self, var: &Variable, syntaxBlock: &SyntaxBlockId, builder: &mut BlockBuilder, explicit: bool) {
        if var.hasTrivialDrop() || var.isArg() {
            return;
        }
        if !explicit && self.declarationStore.explicitDeclarations.contains(&var.name) {
            return;
        }
        self.declarationStore.declare(var.clone(), syntaxBlock.clone());
        self.dropMetadataStore
            .addVariable(var.name.clone(), var.getType().clone());
        let kind = MetadataKind::DeclarationList(var.name.clone());
        builder.addInstruction(InstructionKind::DropMetadata(kind), var.location.clone());
        builder.step();
    }

    fn processBlock(&mut self, blockId: BlockId, initialSyntaxBlock: SyntaxBlockId) {
        let mut currentSyntaxBlock = initialSyntaxBlock;
        // println!(
        //     "Processing block {:?}, initial currentSyntaxBlock: {}",
        //     blockId, currentSyntaxBlock
        // );

        let mut builder = self.bodyBuilder.iterator(blockId);
        loop {
            match builder.getInstruction() {
                Some(instruction) => {
                    // Process the instruction
                    match &instruction.kind {
                        InstructionKind::BlockStart(blockId) => {
                            // println!(
                            //     "BlockStart: {}, updating currentSyntaxBlock from {} to {}",
                            //     blockId, currentSyntaxBlock, blockId
                            // );
                            currentSyntaxBlock = blockId.clone();
                        }
                        InstructionKind::BlockEnd(blockId) => {
                            // println!(
                            //     "BlockEnd: {}, updating currentSyntaxBlock from {} to {}",
                            //     blockId,
                            //     currentSyntaxBlock,
                            //     blockId.getParent()
                            // );
                            currentSyntaxBlock = blockId.getParent();
                        }
                        InstructionKind::Jump(_, targetBlock) => {
                            self.addToQueue(*targetBlock, currentSyntaxBlock.clone());
                        }
                        InstructionKind::EnumSwitch(_, cases) => {
                            for case in cases {
                                self.addToQueue(case.branch, currentSyntaxBlock.clone());
                            }
                        }
                        InstructionKind::StringSwitch(_, cases) => {
                            for case in cases {
                                self.addToQueue(case.branch, currentSyntaxBlock.clone());
                            }
                        }
                        InstructionKind::IntegerSwitch(_, cases) => {
                            for case in cases {
                                self.addToQueue(case.branch, currentSyntaxBlock.clone());
                            }
                        }
                        InstructionKind::Return(_, _) => {}
                        InstructionKind::DeclareVar(var, _) => {
                            self.declareVar(var, &currentSyntaxBlock, &mut builder, true);
                        }
                        kind => {
                            let usageInfo = getUsageInfo(kind.clone());
                            if let Some(assignPath) = usageInfo.assign {
                                if assignPath.isRootOnly() {
                                    // println!("Declaring variable for assignPath: {}", assignPath);
                                    self.declareVar(&assignPath.root, &currentSyntaxBlock, &mut builder, false);
                                }
                            }
                        } // InstructionKind::Assign(dest, src) => {
                          //     self.declareVar(dest, &currentSyntaxBlock, &mut builder, false);
                          //     self.useVar(src, &mut builder);
                          //     if !dest.hasTrivialDrop() {
                          //         //println!("Dropflag set to true for variable {} at {}", dest, instruction.location);
                          //         builder.addInstruction(
                          //             InstructionKind::FunctionCall(dest.getDropFlag(), getTrueName(), vec![]),
                          //             instruction.location.clone(),
                          //         );
                          //         builder.step();
                          //     }
                          //     let index = self.dropMetadataStore.createDropList(Kind::VariableAssign(
                          //         Path::new(dest.clone(), instruction.location.clone()).toSimplePath(),
                          //     ));
                          //     builder.addInstruction(
                          //         InstructionKind::DropListPlaceholder(index),
                          //         instruction.location.clone(),
                          //     );
                          //     builder.step();
                          // }
                          // InstructionKind::FieldAssign(dest, _, fields) => {
                          //     self.declareVar(dest, &currentSyntaxBlock, &mut builder, false);
                          //     let path = buildFieldPath(dest, fields);
                          //     let index = self
                          //         .dropMetadataStore
                          //         .createDropList(Kind::FieldAssign(path.toSimplePath()));
                          //     builder.addInstruction(
                          //         InstructionKind::DropListPlaceholder(index),
                          //         instruction.location.clone(),
                          //     );
                          //     builder.step();
                          // }
                          // InstructionKind::DeclareVar(var, _) => {
                          //     // println!(
                          //     //     "Processing DeclareVar instruction for {} with currentSyntaxBlock: {}",
                          //     //     var, currentSyntaxBlock
                          //     // );
                          //     self.declareVar(var, &currentSyntaxBlock, &mut builder, true);
                          // }
                          // InstructionKind::Ref(dest, _) => {
                          //     self.declareVar(dest, &currentSyntaxBlock, &mut builder, false);
                          // }

                          // kind => {
                          //     let mut allUsedVars = kind.collectVariables();
                          //     if let Some(dest) = kind.getResultVar() {
                          //         allUsedVars.retain(|var| var != &dest);
                          //         self.declareVar(&dest, &currentSyntaxBlock, &mut builder, false);
                          //         if !dest.isTemp() && !dest.isDropFlag() {
                          //             panic!(
                          //                 "Implicit destination should be a temporary variable, but found: {}",
                          //                 dest
                          //             );
                          //         }
                          //         builder.step();
                          //         builder.addInstruction(
                          //             InstructionKind::FunctionCall(dest.getDropFlag(), getTrueName(), vec![]),
                          //             instruction.location.clone(),
                          //         );
                          //     }
                          //     for var in allUsedVars {
                          //         self.useVar(&var, &mut builder);
                          //     }
                          // }
                    }
                    builder.step();
                }
                None => break,
            }
        }
    }

    fn addToQueue(&mut self, blockId: BlockId, syntaxBlock: SyntaxBlockId) {
        //println!("Adding to queue: blockId = {}, syntaxBlock = {}", blockId, syntaxBlock);
        self.queue.push_back((blockId, syntaxBlock));
    }

    fn collectExplicitDeclarations(&mut self) {
        for blockId in self.bodyBuilder.getAllBlockIds() {
            let mut builder = self.bodyBuilder.iterator(blockId);
            loop {
                if let Some(instruction) = builder.getInstruction() {
                    if let InstructionKind::DeclareVar(v, _) = &instruction.kind {
                        self.declarationStore.explicitDeclarations.insert(v.name.clone());
                    }
                    builder.step();
                } else {
                    break;
                }
            }
        }
    }

    fn buildDeclarationStore(&mut self) {
        let mut blockSyntaxBlocks = BTreeMap::new();

        self.addToQueue(BlockId::first(), SyntaxBlockId::new());

        while let Some((blockId, initialSyntaxBlock)) = self.queue.pop_front() {
            if blockSyntaxBlocks.contains_key(&blockId) {
                let existingSyntaxBlock = blockSyntaxBlocks.get(&blockId).unwrap();
                if *existingSyntaxBlock != initialSyntaxBlock {
                    println!("Function: {}", self.function);

                    panic!(
                        "Inconsistent syntax block for block {:?}: existing {} vs new {}",
                        blockId, existingSyntaxBlock, initialSyntaxBlock
                    );
                }
                continue;
            }

            blockSyntaxBlocks.insert(blockId, initialSyntaxBlock.clone());
            self.processBlock(blockId, initialSyntaxBlock);
        }
    }

    pub fn process(&mut self) -> Function {
        if self.function.body.is_none() {
            return self.function.clone();
        }
        //println!("Drop initializer processing function: {}", self.function.name);

        // let graph = GraphBuilder::new(self.function).withPostfix("drop").build();
        // graph.printDot();

        self.collectExplicitDeclarations();

        self.buildDeclarationStore();

        // self.declarationStore.dump();

        let mut result = self.function.clone();
        result.body = Some(self.bodyBuilder.build());

        // println!("Drop initializer completed for function: {}", self.function.name);
        // println!("result {}", result);

        // let graph = GraphBuilder::new(&result).withPostfix("initializer_end").build();
        // graph.printDot();
        result
    }
}
