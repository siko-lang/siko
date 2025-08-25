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
        Function::Function,
        Instruction::{InstructionKind, SyntaxBlockId},
        Program::Program,
        SyntaxBlockIterator::SyntaxBlockIterator,
        Variable::Variable,
    },
};

pub struct Initializer<'a> {
    bodyBuilder: BodyBuilder,
    function: &'a Function,
    program: &'a Program,
    dropMetadataStore: &'a mut DropMetadataStore,
    declarationStore: &'a mut DeclarationStore,
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
        }
    }

    fn declareVar(&mut self, var: &Variable, syntaxBlock: &SyntaxBlockId, builder: &mut BlockBuilder, explicit: bool) {
        if var.hasTrivialDrop() || var.isArg() {
            return;
        }
        if !explicit
            && self
                .declarationStore
                .explicitDeclarations
                .contains(&var.name())
        {
            return;
        }
        self.declarationStore.declare(var.clone(), syntaxBlock.clone());
        self.dropMetadataStore
            .addVariable(var.name(), var.getType().clone());
        let kind = MetadataKind::DeclarationList(var.name());
        builder.addInstruction(InstructionKind::DropMetadata(kind), var.location().clone());
        builder.step();
    }

    fn collectExplicitDeclarations(&mut self) {
        for blockId in self.bodyBuilder.getAllBlockIds() {
            let mut builder = self.bodyBuilder.iterator(blockId);
            loop {
                if let Some(instruction) = builder.getInstruction() {
                    if let InstructionKind::DeclareVar(v, _) = &instruction.kind {
                        self.declarationStore.explicitDeclarations.insert(v.name());
                    }
                    builder.step();
                } else {
                    break;
                }
            }
        }
    }

    fn buildDeclarationStore(&mut self) {
        let mut syntaxBlockIterator = SyntaxBlockIterator::new(self.bodyBuilder.clone());

        syntaxBlockIterator.iterate(|instruction, syntaxBlock, builder| {
            match &instruction.kind {
                InstructionKind::DeclareVar(var, _) => {
                    self.declareVar(var, &syntaxBlock, builder, true);
                }
                kind => {
                    let usageInfo = getUsageInfo(kind.clone());
                    if let Some(assignPath) = usageInfo.assign {
                        if assignPath.isRootOnly() {
                            // println!("Declaring variable for assignPath: {}", assignPath);
                            self.declareVar(&assignPath.root, &syntaxBlock, builder, false);
                        }
                    }
                }
            }
        });
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
