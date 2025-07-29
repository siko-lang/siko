use std::collections::{BTreeMap, BTreeSet};

use crate::siko::{
    backend::drop::{
        BlockProcessor::BlockProcessor,
        Context::Context,
        Error::{reportErrors, Error},
        Event::Collision,
        Path::Path,
    },
    hir::{
        BodyBuilder::BodyBuilder,
        Function::{BlockId, Function},
        Instruction::InstructionKind,
        Program::Program,
        Type::Type,
    },
    location::Report::ReportContext,
    qualifiedname::getCloneFnName,
};

pub fn checkDrops(ctx: &ReportContext, program: Program) -> Program {
    let mut result = program.clone();
    for (name, f) in &program.functions {
        let mut checker = DropChecker::new(f, ctx, &program);
        //println!("Checking drops for {}", name);
        let f = checker.process();
        result.functions.insert(name.clone(), f);
    }
    result
}

pub struct DropChecker<'a> {
    ctx: &'a ReportContext,
    bodyBuilder: BodyBuilder,
    function: &'a Function,
    program: &'a Program,
    visited: BTreeMap<BlockId, BTreeSet<Context>>,
}

impl<'a> DropChecker<'a> {
    pub fn new(f: &'a Function, ctx: &'a ReportContext, program: &'a Program) -> DropChecker<'a> {
        DropChecker {
            ctx: ctx,
            bodyBuilder: BodyBuilder::cloneFunction(f),
            function: f,
            program: program,
            visited: BTreeMap::new(),
        }
    }

    fn process(&mut self) -> Function {
        if self.function.body.is_none() {
            return self.function.clone();
        }
        //println!("Processing function: {}", self.function.name);
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
        struct Case {
            blockId: BlockId,
            context: Context,
        }

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
            let block = self.function.getBlockById(case.blockId);
            let mut blockProcessor = BlockProcessor::new();
            let (context, jumpTargets) = blockProcessor.process(&block, case.context);
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

        println!(
            "Found {} collisions and {} implicit clones in function {}",
            allCollisions.len(),
            implicitClones.len(),
            self.function.name
        );
        for (blockId, mut instructions) in implicitClones {
            instructions.sort();
            let mut builder = self.bodyBuilder.iterator(blockId);
            println!("Processing {} implicit clones in block {}", instructions.len(), blockId);
            let mut index = 0;
            loop {
                if instructions.is_empty() {
                    break;
                }
                if instructions.contains(&(index as u32)) {
                    println!("Processing implicit clone at index {} in block {}", index, blockId);
                    let instruction = builder
                        .getInstruction()
                        .expect(&format!("No instruction at index {}", index));
                    if let InstructionKind::FieldRef(dest, receiver, fields) = instruction.kind {
                        let mut implicitCloneVar = self.bodyBuilder.createTempValue(dest.location.clone());
                        implicitCloneVar.ty = dest.ty.clone().map(|t| Type::Reference(Box::new(t), None));
                        let mut implicitCloneVarRef = self.bodyBuilder.createTempValue(dest.location.clone());
                        implicitCloneVarRef.ty = receiver.ty.clone().map(|t| Type::Reference(Box::new(t), None));
                        let implicitClone = InstructionKind::FunctionCall(
                            dest.clone(),
                            getCloneFnName(),
                            vec![implicitCloneVar.clone()],
                        );
                        let implicitCloneRef = InstructionKind::Ref(implicitCloneVarRef.clone(), receiver.clone());
                        let updatedKind =
                            InstructionKind::FieldRef(implicitCloneVar.clone(), implicitCloneVarRef.clone(), fields);
                        builder.addInstruction(implicitCloneRef, dest.location.clone());
                        builder.step();
                        builder.replaceInstruction(updatedKind, dest.location.clone());
                        builder.step();
                        builder.addInstruction(implicitClone, dest.location.clone());
                        builder.step();
                    } else {
                        let result = instruction
                            .kind
                            .getResultVar()
                            .expect("No result var for implicit clone");
                        let mut implicitCloneVar = self.bodyBuilder.createTempValue(result.location.clone());
                        implicitCloneVar.ty = result.ty.clone();
                        let mut implicitCloneVarRef = self.bodyBuilder.createTempValue(result.location.clone());
                        implicitCloneVarRef.ty = result.ty.clone().map(|t| Type::Reference(Box::new(t), None));
                        let implicitClone = InstructionKind::FunctionCall(
                            result.clone(),
                            getCloneFnName(),
                            vec![implicitCloneVar.clone()],
                        );
                        let implicitCloneRef = InstructionKind::Ref(implicitCloneVarRef.clone(), result.clone());
                        let updatedKind = instruction.kind.replaceVar(result.clone(), implicitCloneVarRef);
                        builder.addInstruction(implicitCloneRef, result.location.clone());
                        builder.step();
                        builder.replaceInstruction(updatedKind, result.location.clone());
                        builder.step();
                        builder.addInstruction(implicitClone, result.location.clone());
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

        let mut errors = Vec::new();
        for collision in allCollisions {
            let err = Error::AlreadyMoved {
                path: collision.path,
                prevMove: collision.prev,
            };
            errors.push(err);
        }
        reportErrors(self.ctx, errors);
        let mut result = self.function.clone();
        result.body = Some(self.bodyBuilder.build());
        result
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
        if path.isSimple() {
            let ty = path.root.getType();
            assert!(!ty.isReference(), "path root should not be a reference for a move!",);
            self.program.instanceResolver.isCopy(&ty)
        } else {
            let resultVar = instruction.kind.getResultVar().expect("no result var");
            let resulTy = resultVar.getType();
            assert!(
                !resulTy.isReference(),
                "result type should not be a reference for a move!",
            );
            //println!("Checking if result type is copy: {}", resulTy);
            self.program.instanceResolver.isCopy(&resulTy)
        }
    }
}
