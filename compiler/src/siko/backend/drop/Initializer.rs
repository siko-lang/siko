use std::collections::{BTreeMap, BTreeSet, VecDeque};

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
        Function::{BlockId, Function},
        Graph::GraphBuilder,
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
    queue: VecDeque<(BlockId, SyntaxBlockId)>,
    placeHolderIndex: u32,
    declaredDropFlags: BTreeSet<Variable>,
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
            queue: VecDeque::new(),
            placeHolderIndex: 0,
            declaredDropFlags: BTreeSet::new(),
        }
    }

    fn addDest(&mut self, var: &Variable) {
        let count = self.destCounts.entry(var.clone()).or_insert(0);
        *count += 1;
    }

    fn declareVar(&mut self, var: &Variable, syntaxBlock: &SyntaxBlockId, builder: &mut BlockBuilder) {
        //println!("declareVar called for {} with syntaxBlock: {}", var, syntaxBlock);
        if var.hasTrivialDrop() || var.isArg() {
            //println!("  -> skipping {} (trivial drop or arg)", var);
            return;
        }
        if self.declarationStore.declare(var.clone(), syntaxBlock.clone()) {
            //println!("  -> declared {} in syntax block {}", var, syntaxBlock);
            let dropFlag = var.getDropFlag();

            self.declaredDropFlags.insert(dropFlag.clone());
            builder.addInstruction(
                InstructionKind::FunctionCall(dropFlag, getFalseName(), vec![]),
                var.location.clone(),
            );
            builder.step();
        } else {
            //println!("  -> {} already declared, skipping", var);
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
                        InstructionKind::Assign(dest, src) => {
                            self.declareVar(dest, &currentSyntaxBlock, &mut builder);
                            self.useVar(src, &mut builder);
                            if !dest.hasTrivialDrop() {
                                //println!("Dropflag set to true for variable {} at {}", dest, instruction.location);
                                builder.addInstruction(
                                    InstructionKind::FunctionCall(dest.getDropFlag(), getTrueName(), vec![]),
                                    instruction.location.clone(),
                                );
                                builder.step();
                            } else {
                                // println!(
                                //     "Skipping drop flag for variable {} at {} because it has a trivial drop",
                                //     dest, instruction.location
                                // );
                            }
                            self.dropListHandler.createDropList(
                                self.placeHolderIndex,
                                Kind::VariableAssign(
                                    Path::new(dest.clone(), instruction.location.clone()).toSimplePath(),
                                ),
                            );
                            builder.addInstruction(
                                InstructionKind::DropListPlaceholder(self.placeHolderIndex),
                                instruction.location.clone(),
                            );
                            builder.step();
                            self.placeHolderIndex += 1;
                        }
                        InstructionKind::FieldAssign(dest, _, fields) => {
                            self.declareVar(dest, &currentSyntaxBlock, &mut builder);
                            let path = buildFieldPath(dest, fields);
                            self.dropListHandler
                                .createDropList(self.placeHolderIndex, Kind::FieldAssign(path.toSimplePath()));
                            self.dropListHandler.addPath(self.placeHolderIndex, path);
                            builder.addInstruction(
                                InstructionKind::DropListPlaceholder(self.placeHolderIndex),
                                instruction.location.clone(),
                            );
                            builder.step();
                            self.placeHolderIndex += 1;
                        }
                        InstructionKind::DeclareVar(var, _) => {
                            // println!(
                            //     "Processing DeclareVar instruction for {} with currentSyntaxBlock: {}",
                            //     var, currentSyntaxBlock
                            // );
                            self.declareVar(var, &currentSyntaxBlock, &mut builder);
                        }
                        InstructionKind::Ref(dest, _) => {
                            self.declareVar(dest, &currentSyntaxBlock, &mut builder);
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
                        kind => {
                            let mut allUsedVars = kind.collectVariables();
                            if let Some(dest) = kind.getResultVar() {
                                allUsedVars.retain(|var| var != &dest);
                                self.declareVar(&dest, &currentSyntaxBlock, &mut builder);
                                if !dest.isTemp() && !dest.isDropFlag() {
                                    panic!(
                                        "Implicit destination should be a temporary variable, but found: {}",
                                        dest
                                    );
                                }
                                builder.step();
                                builder.addInstruction(
                                    InstructionKind::FunctionCall(dest.getDropFlag(), getTrueName(), vec![]),
                                    instruction.location.clone(),
                                );
                                self.addDest(&dest);
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

    pub fn addToQueue(&mut self, blockId: BlockId, syntaxBlock: SyntaxBlockId) {
        //println!("Adding to queue: blockId = {}, syntaxBlock = {}", blockId, syntaxBlock);
        self.queue.push_back((blockId, syntaxBlock));
    }

    pub fn process(&mut self) -> Function {
        if self.function.body.is_none() {
            return self.function.clone();
        }
        //println!("Drop initializer processing function: {}", self.function.name);

        // let graph = GraphBuilder::new(self.function).withPostfix("drop").build();
        // graph.printDot();

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

        for (var, count) in &self.destCounts {
            if *count > 1 {
                panic!(
                    "Variable {} is assigned more than once, but is temporary and should be only assigned once.",
                    var
                );
            }
        }

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

        // self.declarationStore.dump();

        let mut result = self.function.clone();
        result.body = Some(self.bodyBuilder.build());

        // println!("Drop initializer completed for function: {}", self.function.name);

        // let graph = GraphBuilder::new(&result).withPostfix("initializer_end").build();
        // graph.printDot();
        result
    }
}
