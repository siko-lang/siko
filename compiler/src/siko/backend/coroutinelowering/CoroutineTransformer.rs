use crate::siko::{
    backend::{
        coroutinelowering::{
            CoroutineLowering::CoroutineInstanceInfo,
            Utils::{getLoweredCoroutineName, getMonomorphizedContext, getResumeResultType, getStateMachineEnumName},
        },
        BuilderUtils::{EnumBuilder, StructBuilder},
        RemoveTuples::getTuple,
    },
    hir::{
        Block::BlockId,
        BlockBuilder::BlockBuilder,
        BodyBuilder::BodyBuilder,
        Function::{Function, Parameter, ResultKind},
        Instruction::{CallInfo, EnumCase, InstructionKind},
        Program::Program,
        Type::Type,
        Variable::{Variable, VariableName},
    },
    location::Location::Location,
    monomorphizer::Context::Context,
    qualifiedname::{
        builtins::{
            getCoroutineCoResumeResultCompletedName, getCoroutineCoResumeResultReturnedName,
            getCoroutineCoResumeResultYieldedName,
        },
        QualifiedName,
    },
};

#[derive(Clone)]
pub struct EntryPoint {
    blockId: BlockId,
    variables: Vec<Variable>,
}

pub struct CoroutineTransformer<'a> {
    f: &'a Function,
    queue: Vec<BlockId>,
    entryPoints: Vec<EntryPoint>,
    program: &'a mut Program,
    enumName: QualifiedName,
    enumTy: Type,
    coroutineTy: Type,
    ctx: Context,
    resumeResultTy: Type,
    resumeResultTupleName: QualifiedName,
    resumeResultTupleTy: Type,
}

impl<'a> CoroutineTransformer<'a> {
    pub fn new(f: &'a Function, program: &'a mut Program) -> CoroutineTransformer<'a> {
        let enumName = getStateMachineEnumName(&f.name);
        let enumTy = Type::Named(enumName.clone(), vec![]);
        let coroutineTy = f.result.getCoroutineType();
        let ctx = getMonomorphizedContext(&coroutineTy);
        let resumeResultTy = getResumeResultType(&coroutineTy);
        let resumeResultTupleTy = Type::Tuple(vec![enumTy.clone(), resumeResultTy.clone()]);
        let resumeResultTupleName = getTuple(&resumeResultTupleTy);
        let resumeResultTupleTy = Type::Named(resumeResultTupleName.clone(), vec![]);
        let mut structBuilder = StructBuilder::new(program, f.kind.getLocation().clone());
        structBuilder.generateStruct(&vec![enumTy.clone(), resumeResultTy.clone()], &resumeResultTupleName);
        CoroutineTransformer {
            f,
            queue: Vec::new(),
            entryPoints: Vec::new(),
            program,
            enumName,
            enumTy,
            coroutineTy,
            ctx,
            resumeResultTy,
            resumeResultTupleName,
            resumeResultTupleTy,
        }
    }

    pub fn transform(&mut self) -> (Function, CoroutineInstanceInfo) {
        //println!("Before transformation: {}", self.f);
        let coroutineName = getLoweredCoroutineName(&self.coroutineTy);
        let mut bodyBuilder = BodyBuilder::cloneFunction(self.f);
        let mut mainEntryPoint = EntryPoint {
            blockId: BlockId::first(),
            variables: Vec::new(),
        };
        for param in &self.f.params {
            let var = Variable::newWithType(
                VariableName::Arg(param.getName()),
                self.f.kind.getLocation(),
                param.getType(),
            );
            mainEntryPoint.variables.push(var);
        }
        self.entryPoints.push(mainEntryPoint);
        let yieldCount = self.getYieldCount(&mut bodyBuilder);
        let allBlockIds = bodyBuilder.getAllBlockIds();
        self.queue.extend(allBlockIds);
        while let Some(blockId) = self.queue.pop() {
            self.processBlock(blockId, &mut bodyBuilder, yieldCount);
        }
        // for entry in &self.entryPoints {
        //     println!(
        //         "Entry point at block {} for variables {:?}",
        //         entry.blockId, entry.variables
        //     );
        // }
        self.generateCoroutineStateMachineEnum(self.f.kind.getLocation());
        let coroVar = Variable::newWithType(
            VariableName::Arg("coro".to_string()),
            self.f.kind.getLocation(),
            self.enumTy.clone(),
        );
        let mut mainBlockBuilder = bodyBuilder.iterator(BlockId::first());
        let newMain = mainBlockBuilder.splitBlock(0);
        self.entryPoints[0].blockId = newMain;
        let mut cases = Vec::new();
        for (variantIndex, entryPoint) in self.entryPoints.clone().iter().enumerate() {
            let enumCase = EnumCase {
                index: variantIndex as u32,
                branch: entryPoint.blockId,
            };
            cases.push(enumCase);
        }
        let completedBlockBuilder = bodyBuilder.createBlock();
        let mut completedBlockBuilder = bodyBuilder.iterator(completedBlockBuilder.getBlockId());
        self.generateCompletedReturn(
            &mut bodyBuilder,
            &mut completedBlockBuilder,
            self.f.kind.getLocation(),
            self.entryPoints.len(),
        );
        let enumCase = EnumCase {
            index: self.entryPoints.len() as u32,
            branch: completedBlockBuilder.getBlockId(),
        };
        cases.push(enumCase);
        let enumSwitch = InstructionKind::EnumSwitch(coroVar.useVar(), cases);
        mainBlockBuilder.addInstruction(enumSwitch, self.f.kind.getLocation());
        let mut f = self.f.clone();
        f.params = vec![Parameter::Named("coro".to_string(), self.enumTy.clone(), false)];
        f.result = ResultKind::SingleReturn(self.resumeResultTupleTy.clone());
        f.body = Some(bodyBuilder.build());
        // println!("after transformation: {}", f);

        let coroutineInstanceInfo = CoroutineInstanceInfo {
            name: QualifiedName::CoroutineInstance(
                Box::new(coroutineName),
                Box::new(QualifiedName::CoroutineStateMachineEnum(Box::new(f.name.clone()))),
            ),
            resumeFnName: self.f.name.clone(),
            stateMachineEnumTy: self.enumTy.clone(),
        };
        (f, coroutineInstanceInfo)
    }

    fn getYieldCount(&self, bodyBuilder: &mut BodyBuilder) -> usize {
        let mut count = 0;
        let allBlockIds = bodyBuilder.getAllBlockIds();
        for blockId in allBlockIds {
            let mut builder = bodyBuilder.iterator(blockId);
            loop {
                if let Some(instr) = builder.getInstruction() {
                    if let InstructionKind::Yield(_, _) = &instr.kind {
                        count += 1;
                    }
                    builder.step();
                } else {
                    break;
                }
            }
        }
        count
    }

    fn processBlock(&mut self, blockId: BlockId, bodyBuilder: &mut BodyBuilder, yieldCount: usize) {
        //println!("Processing block: {:?}", blockId);
        let mut builder = bodyBuilder.iterator(blockId);
        loop {
            if let Some(instr) = builder.getInstruction() {
                match &instr.kind {
                    InstructionKind::Yield(_, value) => {
                        let newBlock = builder.splitBlock(0);
                        //println!("Split at yield, created new block: {:?}", newBlock);
                        self.entryPoints.push(EntryPoint {
                            blockId: newBlock,
                            variables: vec![],
                        });
                        let mut newBuilder = bodyBuilder.iterator(newBlock);
                        newBuilder.removeInstruction();
                        self.queue.push(newBlock);
                        let resultCtorName = getCoroutineCoResumeResultYieldedName().monomorphized(self.ctx.clone());
                        self.generateReturn(
                            bodyBuilder,
                            &mut builder,
                            value,
                            instr.location.clone(),
                            self.entryPoints.len() - 1,
                            resultCtorName,
                        );
                    }
                    InstructionKind::Return(_, value) => {
                        builder.removeInstruction();
                        let resultCtorName = getCoroutineCoResumeResultReturnedName().monomorphized(self.ctx.clone());
                        self.generateReturn(
                            bodyBuilder,
                            &mut builder,
                            value,
                            instr.location.clone(),
                            yieldCount + 1,
                            resultCtorName,
                        );
                    }
                    _ => {}
                };
                builder.step();
            } else {
                break;
            }
        }
    }

    fn generateReturn(
        &mut self,
        bodyBuilder: &mut BodyBuilder,
        builder: &mut BlockBuilder,
        value: &Variable,
        location: Location,
        variantIndex: usize,
        resultCtorName: QualifiedName,
    ) {
        let resultVar = bodyBuilder.createTempValueWithType(location.clone(), self.resumeResultTy.clone());
        let callInfo = CallInfo {
            name: resultCtorName,
            args: vec![value.useVar()],
            context: None,
            instanceRefs: Vec::new(),
            coroutineSpawn: false,
        };
        let resultCtorCall = InstructionKind::FunctionCall(resultVar.clone(), callInfo);
        builder.addInstruction(resultCtorCall, location.clone());
        builder.step();
        let variantVar = bodyBuilder.createTempValueWithType(location.clone(), self.enumTy.clone());
        let variantName = getVariantName(&self.f.name, variantIndex);
        let callInfo = CallInfo {
            name: variantName,
            args: Vec::new(),
            context: None,
            instanceRefs: Vec::new(),
            coroutineSpawn: false,
        };
        let variantCtorCall = InstructionKind::FunctionCall(variantVar.clone(), callInfo);
        builder.addInstruction(variantCtorCall, location.clone());
        builder.step();
        let tupleVar = bodyBuilder.createTempValueWithType(location.clone(), self.resumeResultTupleTy.clone());
        let tupleCallInfo = CallInfo {
            name: self.resumeResultTupleName.clone(),
            args: vec![variantVar.useVar(), resultVar.useVar()],
            context: None,
            instanceRefs: Vec::new(),
            coroutineSpawn: false,
        };
        let tupleCtorCall = InstructionKind::FunctionCall(tupleVar.clone(), tupleCallInfo);
        builder.addInstruction(tupleCtorCall, location.clone());
        builder.step();
        builder.addReturn(tupleVar, location.clone());
    }

    fn generateCompletedReturn(
        &mut self,
        bodyBuilder: &mut BodyBuilder,
        builder: &mut BlockBuilder,
        location: Location,
        variantIndex: usize,
    ) {
        let resultVar = bodyBuilder.createTempValueWithType(location.clone(), self.resumeResultTy.clone());
        let callInfo = CallInfo {
            name: getCoroutineCoResumeResultCompletedName().monomorphized(self.ctx.clone()),
            args: vec![],
            context: None,
            instanceRefs: Vec::new(),
            coroutineSpawn: false,
        };
        let resultCtorCall = InstructionKind::FunctionCall(resultVar.clone(), callInfo);
        builder.addInstruction(resultCtorCall, location.clone());
        builder.step();
        let variantVar = bodyBuilder.createTempValueWithType(location.clone(), self.enumTy.clone());
        let variantName = getVariantName(&self.f.name, variantIndex);
        let callInfo = CallInfo {
            name: variantName,
            args: Vec::new(),
            context: None,
            instanceRefs: Vec::new(),
            coroutineSpawn: false,
        };
        let variantCtorCall = InstructionKind::FunctionCall(variantVar.clone(), callInfo);
        builder.addInstruction(variantCtorCall, location.clone());
        builder.step();
        let tupleVar = bodyBuilder.createTempValueWithType(location.clone(), self.resumeResultTupleTy.clone());
        let tupleCallInfo = CallInfo {
            name: self.resumeResultTupleName.clone(),
            args: vec![variantVar.useVar(), resultVar.useVar()],
            context: None,
            instanceRefs: Vec::new(),
            coroutineSpawn: false,
        };
        let tupleCtorCall = InstructionKind::FunctionCall(tupleVar.clone(), tupleCallInfo);
        builder.addInstruction(tupleCtorCall, location.clone());
        builder.step();
        builder.addReturn(tupleVar, location.clone());
    }

    fn generateCoroutineStateMachineEnum(&mut self, location: Location) {
        let mut enumBuilder = EnumBuilder::new(self.enumName.clone(), self.program, location.clone());
        for (variantIndex, entryPoint) in self.entryPoints.clone().iter().enumerate() {
            let variantName = getVariantName(&self.f.name, variantIndex);
            let fieldTypes = entryPoint.variables.iter().map(|v| v.getType()).collect();
            enumBuilder.generateVariant(&variantName, &fieldTypes, variantIndex);
        }
        let variantName = getVariantName(&self.f.name, self.entryPoints.len());
        enumBuilder.generateVariant(&variantName, &Vec::new(), self.entryPoints.len());
        enumBuilder.generateEnum(&location);
    }
}

fn getVariantName(functionName: &QualifiedName, index: usize) -> QualifiedName {
    QualifiedName::CoroutineStateMachineVariant(Box::new(functionName.clone()), index as u32)
}
