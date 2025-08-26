use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use crate::siko::{
    backend::drop::{
        BlockProcessor::BlockProcessor,
        Context::Context,
        DeclarationStore::DeclarationStore,
        DropMetadataStore::DropMetadataStore,
        Error::{reportErrors, Error},
        Event::Collision,
        Finalizer::Finalizer,
        Initializer::Initializer,
        Path::Path,
    },
    hir::{
        BodyBuilder::BodyBuilder,
        Function::{BlockId, Function},
        FunctionCallResolver::FunctionCallResolver,
        Graph::GraphBuilder,
        InstanceResolver::InstanceResolver,
        Instruction::{CallInfo, InstructionKind},
        Program::Program,
        TypeVarAllocator::TypeVarAllocator,
        Unifier::Unifier,
    },
    location::Report::ReportContext,
    typechecker::ConstraintExpander::ConstraintExpander,
};

pub fn checkDrops(ctx: &ReportContext, program: Program) -> Program {
    let mut result = program.clone();
    for (name, f) in &program.functions {
        let mut dropMetadataStore = DropMetadataStore::new();
        let mut declarationStore = DeclarationStore::new();
        let mut initializer = Initializer::new(f, &program, &mut dropMetadataStore, &mut declarationStore);
        let f = initializer.process();
        //declarationStore.dump();
        let mut checker = DropChecker::new(&f, ctx, &program, &mut dropMetadataStore);
        //println!("Checking drops for {}", name);
        let f = checker.process();
        let mut finalizer = Finalizer::new(&f, &program, &mut dropMetadataStore, &declarationStore);
        let f = finalizer.process();
        if false {
            let graph = GraphBuilder::new(&f).withPostfix("dropcheck").build();
            graph.printDot();
        }
        result.functions.insert(name.clone(), f);
    }
    result
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Case {
    blockId: BlockId,
    context: Context,
}

impl Display for Case {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Case: BlockId: {}, Context: {}", self.blockId, self.context)
    }
}

pub struct DropChecker<'a> {
    ctx: &'a ReportContext,
    bodyBuilder: BodyBuilder,
    function: &'a Function,
    program: &'a Program,
    visited: BTreeMap<BlockId, BTreeSet<Context>>,
    dropMetadataStore: &'a mut DropMetadataStore,
    implResolver: InstanceResolver<'a>,
    fnCallResolver: FunctionCallResolver<'a>,
}

impl<'a> DropChecker<'a> {
    pub fn new(
        f: &'a Function,
        ctx: &'a ReportContext,
        program: &'a Program,
        dropMetadataStore: &'a mut DropMetadataStore,
    ) -> DropChecker<'a> {
        let instanceStore = program
            .instanceStores
            .get(&f.name.module())
            .expect("No impl store for module");
        let allocator = TypeVarAllocator::new();
        let implResolver = InstanceResolver::new(allocator.clone(), instanceStore, program);
        let expander = ConstraintExpander::new(program, allocator.clone(), f.constraintContext.clone());
        let knownConstraints = expander.expandKnownConstraints();
        let unifier = Unifier::new(ctx);
        let fnCallResolver = FunctionCallResolver::new(
            program,
            allocator.clone(),
            ctx,
            instanceStore,
            knownConstraints.clone(),
            unifier.clone(),
        );
        DropChecker {
            ctx: ctx,
            bodyBuilder: BodyBuilder::cloneFunction(f),
            function: f,
            program: program,
            visited: BTreeMap::new(),
            dropMetadataStore,
            implResolver,
            fnCallResolver,
        }
    }

    fn process(&mut self) -> Function {
        if self.function.body.is_none() {
            return self.function.clone();
        }
        // println!("Processing function: {}", self.function.name);
        // println!("Function: {}", self.function);

        // let mut graph = GraphBuilder::new(self.function);
        // graph = graph.withPostfix("dropcheck");
        // graph.build().printDot();

        let mut visited = BTreeSet::new();
        let mut queue = Vec::new();
        queue.push(Case {
            blockId: BlockId::first(),
            context: Context::new(),
        });
        let mut allCollisions = Vec::new();
        loop {
            let Some(case) = queue.pop() else { break };
            if !visited.insert(case.clone()) {
                continue;
            }
            //println!("Adding case {} to visited", case);
            //println!("Processed {} cases", visited.len());
            let builder = self.bodyBuilder.iterator(case.blockId);
            let mut blockProcessor = BlockProcessor::new(self.dropMetadataStore);
            let (context, jumpTargets) = blockProcessor.process(builder, case.context);
            let collisions = context.validate();
            allCollisions.extend(collisions);
            let jumpContext = context.compress();
            for jumpTarget in jumpTargets {
                queue.push(Case {
                    blockId: jumpTarget,
                    context: jumpContext.clone(),
                });
            }
        }

        let (allCollisions, implicitClones) = self.processImplicitClones(allCollisions);
        // println!(
        //     "Found {} collisions and {} implicit clones in function {}",
        //     allCollisions.len(),
        //     implicitClones.len(),
        //     self.function.name
        // );
        if !allCollisions.is_empty() {
            let mut errors = Vec::new();
            for collision in allCollisions {
                let err = Error::AlreadyMoved {
                    path: collision.path,
                    prevMove: collision.prev,
                };
                errors.push(err);
            }
            reportErrors(self.ctx, errors);
        }

        self.applyImplicitClones(implicitClones);

        let mut result = self.function.clone();
        result.body = Some(self.bodyBuilder.build());
        result
    }

    fn applyImplicitClones(&mut self, implicitClones: BTreeMap<BlockId, Vec<u32>>) {
        for (blockId, mut instructions) in implicitClones {
            instructions.sort();
            let mut builder = self.bodyBuilder.iterator(blockId);
            //println!("Processing {} implicit clones in block {}", instructions.len(), blockId);
            let mut index = 0;
            loop {
                if instructions.is_empty() {
                    break;
                }
                if instructions.contains(&(index as u32)) {
                    //println!("Processing implicit clone at index {} in block {}", index, blockId);
                    let instruction = builder
                        .getInstruction()
                        .expect(&format!("No instruction at index {}", index));
                    if let InstructionKind::FieldRef(dest, receiver, fields) = instruction.kind {
                        let implicitCloneVar = self
                            .bodyBuilder
                            .createTempValueWithType(dest.location().clone(), dest.getType().asRef());
                        let implicitCloneVarRef = self
                            .bodyBuilder
                            .createTempValueWithType(dest.location().clone(), receiver.getType().asRef());
                        let (fnName, instanceRefs) = self
                            .fnCallResolver
                            .resolveCloneCall(implicitCloneVar.clone(), dest.clone());
                        let mut info = CallInfo::new(fnName, vec![implicitCloneVar.clone()]);
                        info.instanceRefs.extend(instanceRefs);
                        let implicitClone = InstructionKind::FunctionCall(dest.clone(), info);
                        let implicitCloneRef = InstructionKind::Ref(implicitCloneVarRef.clone(), receiver.clone());
                        let updatedKind =
                            InstructionKind::FieldRef(implicitCloneVar.clone(), implicitCloneVarRef.clone(), fields);
                        builder.addInstruction(implicitCloneRef, dest.location().clone());
                        builder.step();
                        builder.replaceInstruction(updatedKind, dest.location().clone());
                        builder.step();
                        builder.addInstruction(implicitClone, dest.location().clone());
                        builder.step();
                    } else {
                        let mut vars = instruction.kind.collectVariables();
                        if let Some(result) = instruction.kind.getResultVar() {
                            vars.retain(|v| *v != result);
                        }
                        if vars.len() != 1 {
                            if let InstructionKind::FieldAssign(_, value, _) = &instruction.kind {
                                vars.retain(|v| *v != *value);
                            } else {
                                println!("Instruction: {}", instruction);
                                panic!(
                                    "Implicit clone should have exactly one non result variable, found: {}",
                                    vars.len()
                                );
                            }
                        }
                        let input = vars[0].clone();
                        let implicitCloneVar = self
                            .bodyBuilder
                            .createTempValueWithType(input.location().clone(), input.getType().clone());
                        let implicitCloneVarRef = self
                            .bodyBuilder
                            .createTempValueWithType(input.location().clone(), input.getType().asRef());
                        let (fnName, instanceRefs) = self
                            .fnCallResolver
                            .resolveCloneCall(implicitCloneVarRef.clone(), implicitCloneVar.clone());
                        let mut info = CallInfo::new(fnName, vec![implicitCloneVarRef.clone()]);
                        info.instanceRefs.extend(instanceRefs);
                        let implicitClone = InstructionKind::FunctionCall(implicitCloneVar.clone(), info);
                        let implicitCloneRef = InstructionKind::Ref(implicitCloneVarRef.clone(), input.clone());
                        let updatedKind = instruction.kind.replaceVar(input.clone(), implicitCloneVar);
                        builder.addInstruction(implicitCloneRef, input.location().clone());
                        builder.step();
                        builder.addInstruction(implicitClone, input.location().clone());
                        builder.step();
                        builder.replaceInstruction(updatedKind, input.location().clone());
                        builder.step();
                    }
                    instructions.retain(|&x| x != index);
                    index += 1;
                } else {
                    index += 1;
                    builder.step();
                }
            }
        }
    }

    fn processImplicitClones(
        &mut self,
        mut allCollisions: Vec<Collision>,
    ) -> (Vec<Collision>, BTreeMap<BlockId, Vec<u32>>) {
        let mut potentialImplicitClones = BTreeSet::new();
        let mut implicitClones = BTreeMap::new();
        for collision in &allCollisions {
            potentialImplicitClones.insert(collision.prev.clone());
        }
        for path in &potentialImplicitClones {
            // println!(
            //     "Checking potential implicit clone for: {} {} {}",
            //     path.userPath(),
            //     path.location,
            //     path.instructionRef
            // );
            let canBeClone = self.canBeImplicitClone(path);
            if canBeClone {
                //println!("Path can be implicit clone: {}", path);
                implicitClones
                    .entry(path.instructionRef.blockId)
                    .or_insert_with(Vec::new)
                    .push(path.instructionRef.instructionId.clone());
                allCollisions.retain(|c| c.prev != *path);
            }
        }
        (allCollisions, implicitClones)
    }

    fn canBeImplicitClone(&self, path: &Path) -> bool {
        let block = self.function.getBlockById(path.instructionRef.blockId);
        let instruction = block.instructions[path.instructionRef.instructionId as usize].clone();
        if path.isRootOnly() {
            let ty = path.root.getType();
            assert!(!ty.isReference(), "path root should not be a reference for a move!",);
            self.implResolver.isCopy(&ty)
        } else {
            let resultVar = instruction.kind.getResultVar().expect("no result var");
            let resulTy = resultVar.getType();
            assert!(
                !resulTy.isReference(),
                "result type should not be a reference for a move!",
            );
            self.implResolver.isCopy(&resulTy)
        }
    }
}
