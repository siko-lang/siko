use std::collections::{BTreeSet, VecDeque};

use crate::siko::{
    backend::drop::{
        DeclarationStore::DeclarationStore,
        DropMetadataStore::{DropMetadataStore, MetadataKind},
        Path::{Path, PathSegment, SimplePath},
        Usage::getUsageInfo,
    },
    hir::{
        BlockBuilder::BlockBuilder,
        BodyBuilder::BodyBuilder,
        Function::{BlockId, Function},
        Instruction::{EnumCase, FieldId, FieldInfo, InstructionKind, Mutability},
        Program::Program,
        Type::Type,
        Variable::Variable,
    },
    qualifiedname::{getFalseName, getTrueName},
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
                            InstructionKind::DropPath(_) => {
                                panic!("drop path found in first pass of finalizer")
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
                                        let rootPath = var.toPath().toSimplePath();
                                        let declarationList = self.dropMetadataStore.getDeclarationList(&var.name);
                                        if let Some(declarationList) = declarationList {
                                            for current in declarationList.paths() {
                                                if current.contains(&rootPath) {
                                                    addDropPath(&mut builder, &current, &var);
                                                }
                                            }
                                        }
                                    }
                                }
                                builder.removeInstruction();
                            }
                            kind => {
                                let usageInfo = getUsageInfo(kind.clone());
                                for usage in usageInfo.usages {
                                    if !usage.isMove() {
                                        continue;
                                    }
                                    let declarationList =
                                        self.dropMetadataStore.getDeclarationList(&usage.path.root.name);
                                    if let Some(declarationList) = declarationList {
                                        //println!("--------------------------");
                                        for path in declarationList.paths() {
                                            // we are moving usage.path, need to disable dropflag for it
                                            // and all other paths that share prefix with it
                                            if path.sharesPrefixWith(&usage.path.toSimplePath()) {
                                                // println!(
                                                //     "Disabling dropflag for path: {} because {} is moved",
                                                //     path, usage.path
                                                // );
                                                let dropFlag = path.getDropFlag();
                                                builder.addInstruction(
                                                    InstructionKind::FunctionCall(dropFlag, getFalseName(), vec![]),
                                                    usage.path.location.clone(),
                                                );
                                                builder.step();
                                            }
                                        }
                                    }
                                }
                                if let Some(assignPath) = usageInfo.assign {
                                    // we are assigning to assignPath, need to enable dropflag for it and all other subpaths
                                    let root = assignPath.root.clone();
                                    let declarationList = self.dropMetadataStore.getDeclarationList(&root.name);
                                    if let Some(declarationList) = declarationList {
                                        for path in declarationList.paths() {
                                            if path.contains(&assignPath.toSimplePath()) {
                                                // println!(
                                                //     "Enabling dropflag for path: {} because {} is assigned",
                                                //     path, assignPath
                                                // );
                                                addDropPath(&mut builder, &path, &root);
                                                let dropFlag = path.getDropFlag();
                                                builder.addInstruction(
                                                    InstructionKind::FunctionCall(dropFlag, getTrueName(), vec![]),
                                                    assignPath.location.clone(),
                                                );
                                                builder.step();
                                            }
                                        }
                                    }
                                }
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
                        InstructionKind::DropPath(path) => {
                            let newBlockId = builder.cutBlock(1);
                            let mut dropBlock = self.bodyBuilder.createBlock();
                            let dropVar = if path.isRootOnly() {
                                path.root.clone()
                            } else {
                                let mut dropVar = self.bodyBuilder.createTempValue(instruction.location.clone());
                                let mut fields = Vec::new();
                                for segment in &path.items {
                                    let fieldInfo = match segment {
                                        PathSegment::Named(name, ty) => {
                                            dropVar.ty = Some(ty.clone());
                                            FieldInfo {
                                                name: FieldId::Named(name.clone()),
                                                location: instruction.location.clone(),
                                                ty: Some(ty.clone()),
                                            }
                                        }
                                        PathSegment::Indexed(index, ty) => {
                                            dropVar.ty = Some(ty.clone());
                                            FieldInfo {
                                                name: FieldId::Indexed(*index),
                                                location: instruction.location.clone(),
                                                ty: Some(ty.clone()),
                                            }
                                        }
                                    };
                                    fields.push(fieldInfo);
                                }
                                let fieldAcess = InstructionKind::FieldRef(dropVar.clone(), path.root.clone(), fields);
                                dropBlock.addInstruction(fieldAcess, instruction.location.clone());
                                dropVar
                            };
                            let mut dropRes = self.bodyBuilder.createTempValue(instruction.location.clone());
                            dropRes.ty = Some(Type::getUnitType());
                            let dropInstruction = InstructionKind::Drop(dropRes, dropVar);
                            dropBlock.addInstruction(dropInstruction, instruction.location.clone());
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
                                InstructionKind::EnumSwitch(path.toSimplePath().getDropFlag(), cases),
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

fn addDropPath(builder: &mut BlockBuilder, current: &SimplePath, var: &Variable) {
    let dropFlag = current.getDropFlag();
    let mut path = Path::new(var.clone(), var.location.clone());
    path.items = current.items.clone();
    let drop = InstructionKind::DropPath(path);
    builder.addInstruction(drop, var.location.clone());
    builder.step();
    builder.addInstruction(
        InstructionKind::FunctionCall(dropFlag, getFalseName(), vec![]),
        var.location.clone(),
    );
    builder.step();
}
