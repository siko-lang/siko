use core::panic;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use crate::siko::{
    backend::drop::{
        BlockProcessor::BlockProcessor,
        Context::Context,
        Error::{reportErrors, Error},
        Misc::{MoveKind, PossibleCollision},
        Path::Path,
        SingleUseVariables::{SingleUseVariableInfo, SingleUseVariables},
        SyntaxBlock::SyntaxBlock,
        Usage::{Usage, UsageKind},
    },
    hir::{
        Apply::instantiateStruct,
        BlockBuilder::BlockBuilder,
        BodyBuilder::BodyBuilder,
        Function::{BlockId, Function},
        Instruction::InstructionKind,
        Program::Program,
        Type::Type,
        TypeVarAllocator::TypeVarAllocator,
        Variable::{Variable, VariableName},
    },
    location::{
        Location::Location,
        Report::{Entry, Report, ReportContext},
    },
    qualifiedname::getCloneFnName,
};

pub fn checkDrops(ctx: &ReportContext, program: Program) -> Program {
    let mut result = program.clone();
    for (name, f) in &program.functions {
        let mut s = SingleUseVariables::new(f);
        let singleUseInfo = s.process();
        let mut checker = DropChecker::new(f, ctx, &program, singleUseInfo);
        //println!("Checking drops for {}", name);
        checker.processDeps();
        // let f = checker.process();
        // result.functions.insert(name.clone(), f);
    }
    result
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
struct InstructionId {
    block: usize,
    id: usize,
}
enum Result {
    AlreadyMoved(Path, Usage),
}

struct DropList {
    paths: Vec<Path>,
}

impl DropList {
    fn new() -> DropList {
        DropList { paths: Vec::new() }
    }

    fn add(&mut self, path: Path) {
        self.paths.push(path);
    }
}

impl Display for DropList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]",
            self.paths
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

struct ValueInfo {
    var: Variable,
    block: String,
}

impl Display for ValueInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.var.value, self.block)
    }
}

fn getUsageKind(var: &Variable) -> UsageKind {
    let ty = var.getType();
    if ty.isReference() || ty.isPtr() {
        UsageKind::Ref
    } else {
        UsageKind::Move
    }
}

pub struct DropChecker<'a> {
    ctx: &'a ReportContext,
    bodyBuilder: BodyBuilder,
    function: &'a Function,
    program: &'a Program,
    visited: BTreeMap<BlockId, BTreeSet<Context>>,
    paths: BTreeMap<VariableName, Path>,
    implicitClones: BTreeSet<Variable>,
    values: BTreeMap<VariableName, ValueInfo>,
    dropLists: BTreeMap<String, DropList>,
    terminalContexts: BTreeMap<BlockId, Context>,
    collisions: BTreeSet<PossibleCollision>,
    usages: BTreeMap<Variable, Usage>,
    assigns: BTreeMap<Variable, DropList>,
    returns: BTreeMap<Variable, DropList>,
    singleUseInfo: SingleUseVariableInfo,
}

impl<'a> DropChecker<'a> {
    pub fn new(
        f: &'a Function,
        ctx: &'a ReportContext,
        program: &'a Program,
        singleUseInfo: SingleUseVariableInfo,
    ) -> DropChecker<'a> {
        DropChecker {
            ctx: ctx,
            bodyBuilder: BodyBuilder::cloneFunction(f),
            function: f,
            program: program,
            visited: BTreeMap::new(),
            paths: BTreeMap::new(),
            implicitClones: BTreeSet::new(),
            values: BTreeMap::new(),
            dropLists: BTreeMap::new(),
            terminalContexts: BTreeMap::new(),
            collisions: BTreeSet::new(),
            usages: BTreeMap::new(),
            assigns: BTreeMap::new(),
            returns: BTreeMap::new(),
            singleUseInfo: singleUseInfo,
        }
    }

    fn addImplicitClone(&mut self) {
        // println!(
        //     "Adding implicit clones for {} {:?}",
        //     self.function.name, self.implicitClones
        // );
        let allblocksIds = self.bodyBuilder.getAllBlockIds();
        for block in allblocksIds {
            let mut builder = self.bodyBuilder.iterator(block);
            loop {
                if let Some(mut instruction) = builder.getInstruction() {
                    if let Some(resultVar) = instruction.kind.getResultVar() {
                        let mut variables = instruction.kind.collectVariables();
                        variables.retain(|v| v.value != resultVar.value);
                        let mut implicitClones = Vec::new();
                        for v in &variables {
                            if self.implicitClones.contains(v) {
                                implicitClones.push(v.clone());
                            }
                        }
                        if !implicitClones.is_empty() {
                            match instruction.kind {
                                InstructionKind::FieldRef(dest, receiver, name) => {
                                    assert_eq!(implicitClones.len(), 1);
                                    //println!("Adding implicit clone for {}", dest);
                                    let mut implicitRefDest = self.bodyBuilder.createTempValue(
                                        VariableName::DropImplicitCloneRef,
                                        instruction.location.clone(),
                                    );
                                    let ty = Type::Reference(Box::new(receiver.getType().clone()), None);
                                    implicitRefDest.ty = Some(ty);
                                    let mut implicitFieldRefDest = self.bodyBuilder.createTempValue(
                                        VariableName::DropImplicitCloneRef,
                                        instruction.location.clone(),
                                    );
                                    implicitFieldRefDest.ty =
                                        Some(Type::Reference(Box::new(dest.getType().clone()), None));
                                    let implicitRefKind =
                                        InstructionKind::Ref(implicitRefDest.clone(), receiver.clone());
                                    let implicitCloneKind = InstructionKind::FunctionCall(
                                        dest.clone(),
                                        getCloneFnName(),
                                        vec![implicitFieldRefDest.clone()],
                                    );
                                    builder.addInstruction(implicitRefKind.clone(), instruction.location.clone());
                                    builder.step();
                                    instruction.kind =
                                        InstructionKind::FieldRef(implicitFieldRefDest.clone(), implicitRefDest, name);
                                    builder.replaceInstruction(instruction.kind, instruction.location.clone());
                                    builder.step();
                                    builder.addInstruction(implicitCloneKind, instruction.location.clone());
                                    builder.step();
                                }
                                _ => {
                                    for c in implicitClones {
                                        //println!("Adding implicit clone for {}", dest);
                                        let mut implicitRefDest = self.bodyBuilder.createTempValue(
                                            VariableName::DropImplicitCloneRef,
                                            instruction.location.clone(),
                                        );
                                        let ty = Type::Reference(Box::new(c.getType().clone()), None);
                                        implicitRefDest.ty = Some(ty);
                                        let implicitRefKind = InstructionKind::Ref(implicitRefDest.clone(), c.clone());

                                        let mut implicitCloneDest = self.bodyBuilder.createTempValue(
                                            VariableName::DropImplicitClone,
                                            instruction.location.clone(),
                                        );
                                        implicitCloneDest.ty = c.ty.clone();
                                        let implicitCloneKind = InstructionKind::FunctionCall(
                                            implicitCloneDest.clone(),
                                            getCloneFnName(),
                                            vec![implicitRefDest],
                                        );
                                        builder.addInstruction(implicitRefKind, instruction.location.clone());
                                        builder.step();
                                        builder.addInstruction(implicitCloneKind, instruction.location.clone());
                                        builder.step();
                                        let newKind = instruction.kind.replaceVar(c, implicitCloneDest.clone());
                                        builder.replaceInstruction(newKind, instruction.location.clone());
                                    }
                                }
                            }
                        }
                    }
                    builder.step();
                } else {
                    break;
                }
            }
        }
    }

    fn processDeps(&mut self) {
        let allblocksIds = self.bodyBuilder.getAllBlockIds();
        //let mut blockDeps = BTreeMap::new();
        // for blockId in &allblocksIds {
        //     blockDeps.insert(*blockId, Vec::new());
        // }
        for blockId in allblocksIds {
            let block = self.function.getBlockById(blockId);
            let mut blockProcessor = BlockProcessor::new(&self.singleUseInfo);
            let mut context = Context::new();
            context = blockProcessor.process(&block, context);
            let collisions = context.validate();
            let mut errors = Vec::new();
            for c in collisions {
                let err = Error::AlreadyMoved {
                    path: c.path,
                    prevMove: c.prev,
                };
                errors.push(err);
            }
            reportErrors(self.ctx, errors);
            // let mut builder = self.bodyBuilder.iterator(blockId);
            // loop {
            //     if let Some(instruction) = builder.getInstruction() {
            //         builder.step();
            //         match instruction.kind {
            //             InstructionKind::Jump(_, id, _) => {
            //                 blockDeps.entry(id).or_default().push(blockId);
            //             }
            //             InstructionKind::EnumSwitch(_, cases) => {
            //                 for case in cases {
            //                     blockDeps.entry(case.branch).or_default().push(blockId);
            //                 }
            //             }
            //             InstructionKind::IntegerSwitch(_, cases) => {
            //                 for case in cases {
            //                     blockDeps.entry(case.branch).or_default().push(blockId);
            //                 }
            //             }
            //             InstructionKind::StringSwitch(_, cases) => {
            //                 for case in cases {
            //                     blockDeps.entry(case.branch).or_default().push(blockId);
            //                 }
            //             }
            //             _ => {}
            //         }
            //     } else {
            //         break;
            //     }
            // }
        }
        // let groups = DependencyProcessor::processDependencies(&mut blockDeps);
        // //println!("all deps {:?}", blockDeps);
        // //println!("groups {:?}", groups);
        // for g in &groups {
        //     //println!("groups {:?}", g);
        //     self.processGroup(&g.items, &blockDeps);
        // }
    }

    // fn processCollision(&mut self) {
    //     let mut failed = false;
    //     for collision in &self.collisions {
    //         let mut prevUsage = self.usages.get(&collision.first).unwrap().clone();
    //         let currentUsage = self.usages.get(&collision.second).unwrap().clone();

    //         if !prevUsage.isMove() {
    //             continue;
    //         }

    //         //println!("collision {} {}", prevUsage.var, currentUsage.var);
    //         if self.program.instanceResolver.isCopy(prevUsage.var.getType()) {
    //             //println!("implict clone for {}", prevUsage.var);
    //             self.implicitClones.insert(prevUsage.var.clone());
    //             prevUsage.kind = UsageKind::Ref;
    //             self.usages.insert(prevUsage.var.clone(), prevUsage);
    //             continue;
    //         }

    //         failed = true;

    //         if prevUsage.var == currentUsage.var {
    //             let slogan = format!(
    //                 "Value {} moved in previous iteration of loop",
    //                 self.ctx.yellow(&currentUsage.path.userPath())
    //             );
    //             //let slogan = format!("Value {} already moved", self.ctx.yellow(&currentPath.to_string()));
    //             let mut entries = Vec::new();
    //             entries.push(Entry::new(None, currentUsage.var.location.clone()));
    //             let r = Report::build(self.ctx, slogan, entries);
    //             r.print();
    //         } else {
    //             let slogan = format!("Value {} already moved", self.ctx.yellow(&currentUsage.path.userPath()));
    //             //let slogan = format!("Value {} already moved", self.ctx.yellow(&currentPath.to_string()));
    //             let mut entries = Vec::new();
    //             entries.push(Entry::new(None, currentUsage.var.location.clone()));
    //             entries.push(Entry::new(
    //                 Some(format!("NOTE: previously moved here")),
    //                 prevUsage.var.location.clone(),
    //             ));
    //             let r = Report::build(self.ctx, slogan, entries);
    //             r.print();
    //         }
    //     }

    //     if failed {
    //         std::process::exit(1);
    //     }
    // }

    // fn process(&mut self) -> Function {
    //     if self.function.body.is_none() {
    //         return self.function.clone();
    //     }
    //     self.processBlock(BlockId::first(), Context::new());
    //     self.processCollision();
    //     self.addImplicitClone();
    //     self.generateDrops();
    //     let mut result = self.function.clone();
    //     result.body = Some(self.bodyBuilder.build());
    //     result
    // }

    fn processDropList(&mut self, dropList: DropList, builder: &mut BlockBuilder, location: Location) {
        let mut kinds = Vec::new();
        for path in &dropList.paths {
            // create FieldRef instructionsfor the path and drop the dest of the fieldref
            let mut receiver = path.root.clone();
            let mut currentTy = receiver.getType().clone();

            for item in &path.items {
                if let Some(structName) = currentTy.getName() {
                    if let Some(structDef) = self.program.getStruct(&structName) {
                        let mut allocator = TypeVarAllocator::new();
                        let structInstance = instantiateStruct(&mut allocator, &structDef, &currentTy);
                        for field in &structInstance.fields {
                            if field.name == *item {
                                currentTy = field.ty.clone();
                                break;
                            }
                        }
                    }
                }

                let mut dest = self
                    .bodyBuilder
                    .createTempValue(VariableName::DropVar, location.clone());
                dest.ty = Some(currentTy.clone());

                let fieldRefKind = InstructionKind::FieldRef(dest.clone(), receiver, item.clone());
                kinds.push(fieldRefKind);
                receiver = dest;
            }

            let mut dropRes = self
                .bodyBuilder
                .createTempValue(VariableName::AutoDropResult, location.clone());
            dropRes.ty = Some(Type::getUnitType());

            let dropKind = InstructionKind::Drop(dropRes, receiver.clone());

            kinds.push(dropKind);
        }
        for k in kinds.into_iter().rev() {
            builder.addInstruction(k, location.clone());
        }
    }

    fn generateDrops(&mut self) {
        let allBlocks = self.bodyBuilder.getAllBlockIds();
        for blockId in allBlocks {
            let mut builder = self.bodyBuilder.iterator(blockId);
            loop {
                if let Some(instruction) = builder.getInstruction() {
                    match &instruction.kind {
                        InstructionKind::Assign(dest, _) => {
                            if let Some(dropList) = self.assigns.remove(&dest) {
                                self.processDropList(dropList, &mut builder, instruction.location.clone());
                            }
                        }
                        InstructionKind::BlockEnd(id) => {
                            if let Some(dropList) = self.dropLists.remove(&id.id) {
                                self.processDropList(dropList, &mut builder, instruction.location.clone());
                            }
                        }
                        InstructionKind::Return(dest, _) => {
                            if let Some(dropList) = self.returns.remove(&dest) {
                                self.processDropList(dropList, &mut builder, instruction.location.clone());
                            }
                        }
                        _ => {}
                    }
                    builder.step();
                } else {
                    break;
                }
            }
        }
    }

    // fn checkMove(&mut self, context: &mut Context, var: &Variable, kind: UsageKind) {
    //     context.addUsage(&self.paths, var, kind, &mut self.collisions, &mut self.usages)
    // }

    // fn declareValue(&mut self, var: &Variable, context: &mut Context) {
    //     context.addLive(var);
    //     self.values.insert(
    //         var.value.clone(),
    //         ValueInfo {
    //             var: var.clone(),
    //             block: context.rootBlock.getCurrentBlockId(),
    //         },
    //     );
    // }

    // fn processBlock(&mut self, blockId: BlockId, mut context: Context) -> bool {
    //     let contexts = self.visited.entry(blockId).or_insert(BTreeSet::new());
    //     if contexts.is_empty() {
    //         contexts.insert(context.clone());
    //     } else {
    //         let first = contexts.first().unwrap();
    //         context.rootBlock = first.rootBlock.clone();
    //         if !contexts.insert(context.clone()) {
    //             //println!("already visited {}", context);
    //             return false;
    //         }
    //     }
    //     let mut builder = self.bodyBuilder.iterator(blockId);
    //     loop {
    //         if let Some(instruction) = builder.getInstruction() {
    //             //println!("PROCESSING {}", instruction.kind);
    //             match &instruction.kind {
    //                 InstructionKind::FunctionCall(dest, _, args) => {
    //                     for arg in args {
    //                         self.checkMove(&mut context, arg, getUsageKind(arg));
    //                     }
    //                     self.declareValue(dest, &mut context);
    //                 }
    //                 InstructionKind::Converter(dest, source) => {
    //                     unreachable!("converter in Drop checker");
    //                 }
    //                 InstructionKind::MethodCall(_, _, _, _) => unreachable!("method call in Drop checker"),
    //                 InstructionKind::DynamicFunctionCall(_, _, _) => {}
    //                 InstructionKind::FieldRef(dest, receiver, fieldName) => {
    //                     if let Some(path) = self.paths.get(&receiver.value) {
    //                         self.paths.insert(dest.value.clone(), path.add(fieldName.clone()));
    //                     } else {
    //                         self.paths
    //                             .insert(dest.value.clone(), Path::new(receiver.clone()).add(fieldName.clone()));
    //                     }
    //                 }
    //                 InstructionKind::TupleIndex(dest, src, index) => {
    //                     let fieldName = format!("{}", index);
    //                     if let Some(path) = self.paths.get(&src.value) {
    //                         self.paths.insert(dest.value.clone(), path.add(fieldName.clone()));
    //                     } else {
    //                         self.paths
    //                             .insert(dest.value.clone(), Path::new(src.clone()).add(fieldName.clone()));
    //                     }
    //                 }
    //                 InstructionKind::Tuple(dest, args) => {
    //                     for arg in args {
    //                         self.checkMove(&mut context, arg, getUsageKind(arg));
    //                     }
    //                     self.declareValue(dest, &mut context);
    //                 }
    //                 InstructionKind::StringLiteral(dest, _) => {
    //                     self.declareValue(dest, &mut context);
    //                 }
    //                 InstructionKind::IntegerLiteral(dest, _) => {
    //                     self.declareValue(dest, &mut context);
    //                 }
    //                 InstructionKind::CharLiteral(dest, _) => {
    //                     self.declareValue(dest, &mut context);
    //                 }
    //                 InstructionKind::Return(dest, src) => {
    //                     self.checkMove(&mut context, src, getUsageKind(src));
    //                     let mut dropList = DropList::new();
    //                     self.dropValues(&mut context, format!("0"), &mut dropList);
    //                     //println!("drop list {}", dropList);
    //                     let mut collisions = BTreeSet::new();
    //                     for p in &dropList.paths {
    //                         context.addUsage(&self.paths, &p.root, UsageKind::Move, &mut collisions, &mut self.usages);
    //                     }
    //                     self.returns.insert(dest.clone(), dropList);
    //                 }
    //                 InstructionKind::Ref(dest, value) => {
    //                     self.checkMove(&mut context, value, UsageKind::Ref);
    //                     self.declareValue(dest, &mut context);
    //                 }
    //                 InstructionKind::Drop(_, _) => {}
    //                 InstructionKind::Jump(_, _, _) => {}
    //                 InstructionKind::Assign(dest, src) => {
    //                     self.checkMove(&mut context, src, getUsageKind(src));
    //                     if !dest.getType().isReference() {
    //                         if context.isLive(&dest.value) {
    //                             let mut dropList = DropList::new();
    //                             self.dropPath(&Path::new(dest.clone()), &dest.getType(), &context, &mut dropList);
    //                             //println!("drop list {}", dropList);
    //                             self.assigns.insert(dest.clone(), dropList);
    //                         }
    //                     }
    //                     context.removeSpecificMoveByRoot(dest);
    //                 }
    //                 InstructionKind::Bind(_, _, _) => {
    //                     panic!("Bind instruction found in DropChecker, this should not happen");
    //                 }
    //                 InstructionKind::FieldAssign(dest, src, fields) => {
    //                     self.checkMove(&mut context, src, getUsageKind(src));
    //                     let ty = fields.last().unwrap().ty.clone().unwrap();
    //                     if !ty.isReference() {
    //                         let mut dropList = DropList::new();
    //                         let mut path = Path::new(dest.clone());
    //                         for f in fields {
    //                             path = path.add(f.name.clone());
    //                         }
    //                         let mut parent = path.clone();
    //                         parent.items.pop();
    //                         for usage in &context.usages {
    //                             //println!("checking {}/{} and {}/{}", usage.var, usage.path, currentPath.root, var);
    //                             if usage.path.shares_prefix_with(&parent) && !usage.path.same(&path) && usage.isMove() {
    //                                 self.collisions.insert(PossibleCollision {
    //                                     first: usage.var.clone(),
    //                                     second: dest.clone(),
    //                                 });
    //                             }
    //                         }
    //                         self.usages.insert(
    //                             dest.clone(),
    //                             Usage {
    //                                 var: dest.clone(),
    //                                 path: parent.clone(),
    //                                 kind: UsageKind::Move,
    //                             },
    //                         );
    //                         self.dropPath(&path, &dest.getType(), &context, &mut dropList);
    //                         //println!("drop list {}", dropList);
    //                         context.removeSpecificMoveByPath(&path);
    //                     }
    //                 }
    //                 InstructionKind::DeclareVar(_, _) => {}
    //                 InstructionKind::Transform(dest, _, _) => {
    //                     self.declareValue(dest, &mut context);
    //                 }
    //                 InstructionKind::EnumSwitch(root, _) => {
    //                     self.checkMove(&mut context, root, getUsageKind(root));
    //                 }
    //                 InstructionKind::IntegerSwitch(root, _) => {
    //                     self.checkMove(&mut context, root, getUsageKind(root));
    //                 }
    //                 InstructionKind::StringSwitch(root, _) => {
    //                     self.checkMove(&mut context, root, getUsageKind(root));
    //                 }
    //                 InstructionKind::BlockStart(name) => {
    //                     let block = SyntaxBlock::new(name.id.clone());
    //                     context.rootBlock.addBlock(block);
    //                     //println!("block start {}", context.rootBlock.getCurrentBlockId());
    //                 }
    //                 InstructionKind::BlockEnd(endId) => {
    //                     //println!("block end {}", context.rootBlock.getCurrentBlockId());
    //                     let current = context.rootBlock.getCurrentBlockId();
    //                     let mut dropList = DropList::new();
    //                     self.dropValues(&mut context, current, &mut dropList);
    //                     //println!("drop list {}", dropList);
    //                     let mut collisions = BTreeSet::new();
    //                     for p in &dropList.paths {
    //                         context.addUsage(&self.paths, &p.root, UsageKind::Move, &mut collisions, &mut self.usages);
    //                     }
    //                     self.dropLists.insert(endId.id.clone(), dropList);
    //                     context.rootBlock.endBlock();
    //                 }
    //             }
    //             builder.step();
    //         } else {
    //             break;
    //         }
    //     }
    //     let old = self.terminalContexts.insert(blockId, context.clone());
    //     match old {
    //         Some(oldContext) => oldContext != context,
    //         None => {
    //             return true;
    //         }
    //     }
    // }

    // fn dropPath(&self, rootPath: &Path, ty: &Type, context: &Context, dropList: &mut DropList) {
    //     match context.isMoved(&&rootPath) {
    //         MoveKind::NotMoved => {
    //             //println!("not moved - drop {}", rootPath);
    //             dropList.add(rootPath.clone());
    //         }
    //         MoveKind::Partially => {
    //             //println!("partially moved {}", rootPath);
    //             //println!("already moved (maybe partially?) {}", rootPath);
    //             if let Some(structName) = ty.getName() {
    //                 if let Some(structDef) = self.program.getStruct(&structName) {
    //                     let mut allocator = TypeVarAllocator::new();
    //                     let structInstance = instantiateStruct(&mut allocator, &structDef, ty);
    //                     for field in &structInstance.fields {
    //                         let path = rootPath.add(field.name.clone());
    //                         self.dropPath(&path, &field.ty, context, dropList);
    //                     }
    //                 }
    //             }
    //         }
    //         MoveKind::Fully(var) => {
    //             //println!("already moved {} by {}", rootPath, var);
    //         }
    //     }
    // }

    // fn dropValues(&mut self, context: &mut Context, block: String, dropList: &mut DropList) {
    //     //println!("Dropping in {}", block);
    //     for var in &context.live {
    //         if let Some(info) = self.values.get(&var.value) {
    //             if info.block.starts_with(&block) {
    //                 //println!("live {}", var.value);
    //                 if !var.getType().isReference() {
    //                     self.dropPath(&Path::new(var.clone()), &var.getType(), context, dropList);
    //                 }
    //             }
    //         }
    //     }
    //     //println!("---");
    // }

    // fn processGroup(&mut self, items: &Vec<BlockId>, blockDeps: &BTreeMap<BlockId, Vec<BlockId>>) {
    //     loop {
    //         let mut changed = false;
    //         for item in items {
    //             //println!("processing block {}", item);
    //             let deps = blockDeps.get(item).expect("deps not found");
    //             let mut mergedContext = Context::new();
    //             let mut empty = !deps.is_empty();
    //             for dep in deps {
    //                 if let Some(terminalContext) = self.terminalContexts.get(dep) {
    //                     mergedContext.merge(terminalContext);
    //                     empty = false;
    //                 } else {
    //                     if !items.contains(dep) {
    //                         panic!("terminal context not found for {}", dep);
    //                     }
    //                 }
    //             }
    //             if empty {
    //                 continue;
    //             }
    //             //println!("merged context {}", mergedContext);
    //             if self.processBlock(item.clone(), mergedContext) {
    //                 changed = true;
    //             }
    //         }
    //         if !changed {
    //             break;
    //         }
    //     }
    // }
}
