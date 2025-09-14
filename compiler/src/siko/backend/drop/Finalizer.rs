use core::panic;
use std::collections::{BTreeMap, VecDeque};

use crate::siko::{
    backend::drop::{
        DeclarationStore::DeclarationStore,
        DropMetadataStore::DropMetadataStore,
        Path::{Path, PathSegment, SimplePath},
        ReferenceStore::ReferenceStore,
        Usage::getUsageInfo,
    },
    hir::{
        Block::BlockId,
        BlockBuilder::BlockBuilder,
        BodyBuilder::BodyBuilder,
        Function::Function,
        Instruction::{CallInfo, EnumCase, FieldId, FieldInfo, InstructionKind, Mutability},
        Program::Program,
        Type::Type,
        Variable::Variable,
    },
    qualifiedname::builtins::{getFalseName, getTrueName},
};

pub struct Finalizer<'a> {
    bodyBuilder: BodyBuilder,
    function: &'a Function,
    program: &'a Program,
    dropMetadataStore: &'a mut DropMetadataStore,
    declarationStore: &'a DeclarationStore,
    declaredDropFlags: BTreeMap<SimplePath, Variable>,
    referenceStore: &'a ReferenceStore,
}

impl<'a> Finalizer<'a> {
    pub fn new(
        f: &'a Function,
        program: &'a Program,
        dropMetadataStore: &'a mut DropMetadataStore,
        declarationStore: &'a DeclarationStore,
        referenceStore: &'a ReferenceStore,
    ) -> Finalizer<'a> {
        Finalizer {
            bodyBuilder: BodyBuilder::cloneFunction(f),
            function: f,
            program: program,
            dropMetadataStore,
            declarationStore,
            declaredDropFlags: BTreeMap::new(),
            referenceStore,
        }
    }

    fn declareDropFlags(&mut self) {
        let mut builder = self.bodyBuilder.iterator(BlockId::first());
        for (_, dropFlag) in &self.declaredDropFlags {
            builder.addInstruction(
                InstructionKind::DeclareVar(dropFlag.clone(), Mutability::Mutable),
                dropFlag.location().clone(),
            );
            builder.step();
            builder.addInstruction(
                InstructionKind::FunctionCall(dropFlag.clone(), CallInfo::new(getFalseName(), vec![])),
                dropFlag.location().clone(),
            );
            builder.step();
        }
    }

    fn getDropFlagForPath(&mut self, path: &SimplePath) -> Variable {
        match self.declaredDropFlags.get(path) {
            Some(v) => v.clone(),
            None => {
                let v = path.getDropFlag();
                self.declaredDropFlags.insert(path.clone(), v.clone());
                v
            }
        }
    }

    pub fn process(&mut self) -> Function {
        if self.function.body.is_none() {
            return self.function.clone();
        }

        self.dropMetadataStore.expandPathLists(self.program);

        // println!("Drop finalizer processing function: {}", self.function.name);
        // println!("{}\n", self.function);

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
                            InstructionKind::DropMetadata(name) => {
                                if let Some(declarationList) = self.dropMetadataStore.getPathList(name) {
                                    //println!("Processing DeclarationList: {}", name);
                                    for path in declarationList.paths() {
                                        //println!("Creating dropflag for path: {}", path);
                                        let dropFlag = path.getDropFlag();
                                        self.declaredDropFlags.insert(path, dropFlag.clone());
                                        builder.addInstruction(
                                            InstructionKind::FunctionCall(
                                                dropFlag,
                                                CallInfo::new(getFalseName(), vec![]),
                                            ),
                                            instruction.location.clone(),
                                        );
                                        builder.step();
                                    }
                                }
                                builder.removeInstruction();
                            }
                            InstructionKind::BlockEnd(id) => {
                                //println!("block end: {}", id);
                                if let Some(droppedValues) = self.declarationStore.getDeclarations(&id) {
                                    //println!(" {} drops {:?}", id, droppedValues);
                                    for var in droppedValues.iter().rev() {
                                        //println!("Generating drops for value {} in blockend", var);
                                        // println!("Generating drops for value {}", var);
                                        let rootPath = var.toPath().toSimplePath();
                                        let pathList = self.dropMetadataStore.getPathList(&var.name());
                                        if let Some(pathList) = pathList {
                                            for current in pathList.paths() {
                                                if current.contains(&rootPath) {
                                                    //println!("Adding drop path for {}", current);
                                                    self.addDropPath(&mut builder, &current, &var);
                                                }
                                            }
                                        }
                                    }
                                }
                                // println!("---------------------");
                                builder.step();
                            }
                            kind => {
                                let usageInfo = getUsageInfo(kind.clone(), self.referenceStore);
                                for usage in usageInfo.usages {
                                    if !usage.isMove() {
                                        continue;
                                    }
                                    self.disablePath(&usage.path, &mut builder);
                                }
                                if let Some(assignPath) = usageInfo.assign {
                                    // we are assigning to assignPath, need to enable dropflag for it and all other subpaths
                                    let root = assignPath.root.clone();
                                    if let Some(pathList) = self.dropMetadataStore.getPathList(&root.name()) {
                                        // println!("--------------------------");
                                        for path in pathList.paths() {
                                            if path.contains(&assignPath.toSimplePath()) {
                                                // println!(
                                                //     "Enabling dropflag for path: {} because {} is assigned",
                                                //     path, assignPath
                                                // );
                                                let generateDrop = if assignPath.isRootOnly() {
                                                    // if this is an implicit variable then the first assignment is the only assignment
                                                    // and we dont need to generate drop
                                                    self.declarationStore.explicitDeclarations.contains(&root.name())
                                                } else {
                                                    true
                                                };
                                                if generateDrop {
                                                    // println!("Adding drop path for {}", path);
                                                    self.addDropPath(&mut builder, &path, &root);
                                                }
                                                let dropFlag = self.getDropFlagForPath(&path);
                                                builder.addInstruction(
                                                    InstructionKind::FunctionCall(
                                                        dropFlag,
                                                        CallInfo::new(getTrueName(), vec![]),
                                                    ),
                                                    assignPath.location.clone(),
                                                );
                                                builder.step();
                                            }
                                        }
                                        // println!("---------------------");
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

        self.declareDropFlags();

        // let mut preDropSwitchResult = self.function.clone();
        // preDropSwitchResult.body = Some(self.bodyBuilder.build());
        // println!("Pre-drop switch function: {}", preDropSwitchResult);

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
                            let dropFlag = self.getDropFlagForPath(&dropVar.toPath().toSimplePath());
                            builder.replaceInstruction(
                                InstructionKind::EnumSwitch(dropFlag, cases),
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
                                let mut fields = Vec::new();
                                let mut dropVarTy = None;
                                for segment in &path.items {
                                    let fieldInfo = match segment {
                                        PathSegment::Named(name, ty) => {
                                            dropVarTy = Some(ty.clone());
                                            FieldInfo {
                                                name: FieldId::Named(name.clone()),
                                                location: instruction.location.clone(),
                                                ty: Some(ty.clone()),
                                            }
                                        }
                                        PathSegment::Indexed(index, ty) => {
                                            dropVarTy = Some(ty.clone());
                                            FieldInfo {
                                                name: FieldId::Indexed(*index),
                                                location: instruction.location.clone(),
                                                ty: Some(ty.clone()),
                                            }
                                        }
                                    };
                                    fields.push(fieldInfo);
                                }
                                let dropVar = self
                                    .bodyBuilder
                                    .createTempValueWithType(instruction.location.clone(), dropVarTy.unwrap());
                                let fieldAcess =
                                    InstructionKind::FieldRef(dropVar.useVarAsDrop(), path.root.clone(), fields);
                                dropBlock.addInstruction(fieldAcess, instruction.location.clone());
                                dropVar
                            };
                            let dropRes = self
                                .bodyBuilder
                                .createTempValueWithType(instruction.location.clone(), Type::getUnitType());
                            let dropInstruction = InstructionKind::Drop(dropRes, dropVar.useVarAsDrop());
                            dropBlock.addInstruction(dropInstruction, instruction.location.clone());
                            // when dropping a path we need to disable the drop flag for all sub-paths
                            self.disablePath(&path, &mut dropBlock.iterator());
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
                                InstructionKind::EnumSwitch(self.getDropFlagForPath(&path.toSimplePath()), cases),
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

    fn disablePath(&mut self, rootPath: &Path, builder: &mut BlockBuilder) {
        let pathList = self.dropMetadataStore.getPathList(&rootPath.root.name());
        if let Some(pathList) = pathList {
            // println!("--------------------------");
            for path in pathList.paths() {
                if path.sharesPrefixWith(&rootPath.toSimplePath()) {
                    // println!("Disabling dropflag for path: {} because {} is moved", path, rootPath);
                    let dropFlag = self.getDropFlagForPath(&path);
                    builder.addInstruction(
                        InstructionKind::FunctionCall(dropFlag, CallInfo::new(getFalseName(), vec![])),
                        rootPath.location.clone(),
                    );
                    builder.step();
                }
            }
            // println!("---------------------");
        }
    }

    fn addDropPath(&mut self, builder: &mut BlockBuilder, current: &SimplePath, var: &Variable) {
        let dropFlag = self.getDropFlagForPath(current);
        let mut path = Path::new(var.clone(), var.location().clone());
        path.items = current.items.clone();
        let drop = InstructionKind::DropPath(path);
        builder.addInstruction(drop, var.location().clone());
        builder.step();
        builder.addInstruction(
            InstructionKind::FunctionCall(dropFlag, CallInfo::new(getFalseName(), vec![])),
            var.location().clone(),
        );
        builder.step();
    }
}
