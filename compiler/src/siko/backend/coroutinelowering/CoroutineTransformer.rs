use std::collections::BTreeMap;

use crate::siko::{
    backend::{
        coroutinelowering::{
            CoroutineLowering::CoroutineInstanceInfo,
            CoroutineStateProcessor::{CoroutineStateProcessor, YieldKey},
            Utils::{getLoweredCoroutineName, getMonomorphizedContext, getResumeResultType, getStateMachineEnumName},
        },
        BuilderUtils::EnumBuilder,
    },
    hir::{
        Block::BlockId,
        BlockBuilder::BlockBuilder,
        BodyBuilder::BodyBuilder,
        ConstraintContext::ConstraintContext,
        Function::{Attributes, Function, FunctionKind, Parameter, ResultKind},
        Instruction::{CallInfo, EnumCase, FieldId, FieldInfo, InstructionKind, TransformInfo},
        Program::Program,
        Type::Type,
        Variable::{Variable, VariableName},
    },
    location::Location::Location,
    monomorphizer::Context::Context,
    qualifiedname::{
        builtins::{
            getBoolTypeName, getCoroutineCoResumeResultCompletedName, getCoroutineCoResumeResultReturnedName,
            getCoroutineCoResumeResultYieldedName, getFalseName, getTrueName,
        },
        QualifiedName,
    },
};

#[derive(Clone)]
pub struct EntryPoint {
    index: usize,
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
    resumeResultTupleTy: Type,
    argReplacements: BTreeMap<VariableName, Variable>,
}

impl<'a> CoroutineTransformer<'a> {
    pub fn new(f: &'a Function, program: &'a mut Program) -> CoroutineTransformer<'a> {
        let enumName = getStateMachineEnumName(&f.name);
        let enumTy = Type::Named(enumName.clone(), vec![]);
        let coroutineTy = f.result.getCoroutineType();
        let ctx = getMonomorphizedContext(&coroutineTy);
        let resumeResultTy = getResumeResultType(&coroutineTy);
        let resumeResultTupleTy = Type::Tuple(vec![enumTy.clone(), resumeResultTy.clone()]);
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
            resumeResultTupleTy,
            argReplacements: BTreeMap::new(),
        }
    }

    pub fn transform(&mut self) -> (Function, Function, CoroutineInstanceInfo) {
        //println!("Before transformation: {}", self.f);
        let coroutineName = getLoweredCoroutineName(&self.coroutineTy);
        let mut bodyBuilder = BodyBuilder::cloneFunction(self.f);
        let mut mainEntryPoint = EntryPoint {
            index: 0,
            blockId: BlockId::first(),
            variables: Vec::new(),
        };
        for param in &self.f.params {
            let newArg = bodyBuilder.createTempValueWithType(self.f.kind.getLocation(), param.getType());
            self.argReplacements
                .insert(VariableName::Arg(param.getName()), newArg.useVar());
            mainEntryPoint.variables.push(newArg);
        }
        self.entryPoints.push(mainEntryPoint);
        let yieldCount = self.getYieldCount(&mut bodyBuilder);
        let mut stateProcessor = CoroutineStateProcessor::new(self.f);
        stateProcessor.process();
        let allBlockIds = bodyBuilder.getAllBlockIds();
        self.queue.extend(allBlockIds);
        while let Some(blockId) = self.queue.pop() {
            self.processBlock(blockId, &mut bodyBuilder, yieldCount, &stateProcessor);
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
        let firstEntryBlockId = mainBlockBuilder.splitBlock(0);
        self.entryPoints[0].blockId = firstEntryBlockId;

        self.addRestoreForArguments(&mut bodyBuilder, &coroVar, firstEntryBlockId);

        for entry in self.entryPoints.clone().iter().skip(1) {
            self.addRestoreForYieldVariables(&mut bodyBuilder, &coroVar, entry);
        }

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

        let isCompletedFn = self.generateIsCompletedFunction();

        let coroutineInstanceInfo = CoroutineInstanceInfo {
            name: QualifiedName::CoroutineInstance(
                Box::new(coroutineName),
                Box::new(QualifiedName::CoroutineStateMachineEnum(Box::new(f.name.clone()))),
            ),
            resumeFnName: self.f.name.clone(),
            isCompletedFnName: isCompletedFn.name.clone(),
            stateMachineEnumTy: self.enumTy.clone(),
            resumeTupleTy: self.resumeResultTupleTy.clone(),
        };

        (f, isCompletedFn, coroutineInstanceInfo)
    }

    fn addRestoreForArguments(&mut self, bodyBuilder: &mut BodyBuilder, coroVar: &Variable, newMain: BlockId) {
        let location = self.f.kind.getLocation();
        let mut builder = bodyBuilder.iterator(newMain);
        if !self.entryPoints[0].variables.is_empty() {
            let argumentTypes: Vec<Type> = self.entryPoints[0].variables.iter().map(|v| v.getType()).collect();
            let tupleType = Type::Tuple(argumentTypes);
            let transformVar = bodyBuilder.createTempValueWithType(location.clone(), tupleType.clone());
            let transform = InstructionKind::Transform(
                transformVar.clone(),
                coroVar.useVar(),
                TransformInfo { variantIndex: 0 },
            );
            builder.addInstruction(transform, location.clone());
            builder.step();

            for (argIndex, variable) in self.entryPoints[0].variables.iter().enumerate() {
                let fieldInfo = FieldInfo {
                    name: FieldId::Indexed(argIndex as u32),
                    location: location.clone(),
                    ty: Some(variable.getType()),
                };
                let extractedVar = bodyBuilder.createTempValueWithType(location.clone(), variable.getType());
                let fieldRef = InstructionKind::FieldRef(extractedVar.clone(), transformVar.useVar(), vec![fieldInfo]);
                builder.addInstruction(fieldRef, location.clone());
                builder.step();

                let assignInstruction = InstructionKind::Assign(variable.clone(), extractedVar);
                builder.addInstruction(assignInstruction, location.clone());
                builder.step();
            }
        }
    }

    fn addRestoreForYieldVariables(&mut self, bodyBuilder: &mut BodyBuilder, coroVar: &Variable, entry: &EntryPoint) {
        let location = self.f.kind.getLocation();
        let mut builder = bodyBuilder.iterator(entry.blockId);

        if !entry.variables.is_empty() {
            let variableTypes: Vec<Type> = entry.variables.iter().map(|v| v.getType()).collect();
            let tupleType = Type::Tuple(variableTypes);
            let transformVar = bodyBuilder.createTempValueWithType(location.clone(), tupleType.clone());
            let transform = InstructionKind::Transform(
                transformVar.clone(),
                coroVar.useVar(),
                TransformInfo {
                    variantIndex: entry.index as u32,
                },
            );
            builder.addInstruction(transform, location.clone());
            builder.step();

            for (varIndex, variable) in entry.variables.iter().enumerate() {
                let fieldInfo = FieldInfo {
                    name: FieldId::Indexed(varIndex as u32),
                    location: location.clone(),
                    ty: Some(variable.getType()),
                };
                let extractedVar = bodyBuilder.createTempValueWithType(location.clone(), variable.getType());
                let fieldRef = InstructionKind::FieldRef(extractedVar.clone(), transformVar.useVar(), vec![fieldInfo]);
                builder.addInstruction(fieldRef, location.clone());
                builder.step();

                let assignInstruction = InstructionKind::Assign(variable.clone(), extractedVar.useVar());
                builder.addInstruction(assignInstruction, location.clone());
                builder.step();
            }
        }
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

    fn processBlock(
        &mut self,
        blockId: BlockId,
        bodyBuilder: &mut BodyBuilder,
        yieldCount: usize,
        stateProcessor: &CoroutineStateProcessor,
    ) {
        //println!("Processing block: {:?}", blockId);
        let mut builder = bodyBuilder.iterator(blockId);
        loop {
            if let Some(instr) = builder.getInstruction() {
                match &instr.kind {
                    InstructionKind::Assign(var, value) => {
                        if let Some(replacement) = self.argReplacements.get(&value.name()) {
                            let newAssign = InstructionKind::Assign(var.clone(), replacement.clone());
                            builder.replaceInstruction(newAssign, instr.location.clone());
                        }
                    }
                    InstructionKind::Yield(dest, value) => {
                        let newBlock = builder.splitBlock(0);
                        //println!("Split at yield, created new block: {:?}", newBlock);

                        let yieldKey = YieldKey {
                            destVar: dest.name().clone(),
                            resultVar: value.name().clone(),
                        };
                        let yieldInfo = stateProcessor.getYieldInfo(&yieldKey);
                        let yieldVariables: Vec<_> = yieldInfo.savedVariables.iter().cloned().collect();

                        self.entryPoints.push(EntryPoint {
                            index: self.entryPoints.len(),
                            blockId: newBlock,
                            variables: yieldVariables.clone(),
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
                            yieldVariables,
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
                            Vec::new(),
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
        variables: Vec<Variable>,
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
            args: variables.iter().map(|v| v.useVar()).collect(),
            context: None,
            instanceRefs: Vec::new(),
            coroutineSpawn: false,
        };
        let variantCtorCall = InstructionKind::FunctionCall(variantVar.clone(), callInfo);
        builder.addInstruction(variantCtorCall, location.clone());
        builder.step();
        let tupleVar = bodyBuilder.createTempValueWithType(location.clone(), self.resumeResultTupleTy.clone());
        let tupleCtorCall = InstructionKind::Tuple(tupleVar.clone(), vec![variantVar.useVar(), resultVar.useVar()]);
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
        let tupleCtorCall = InstructionKind::Tuple(tupleVar.clone(), vec![variantVar.useVar(), resultVar.useVar()]);
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

    fn generateIsCompletedFunction(&self) -> Function {
        let location = Location::empty();
        let boolTy = Type::Named(getBoolTypeName(), vec![]);
        let isCompletedFnName = QualifiedName::CoroutineStateMachineIsCompleted(Box::new(self.f.name.clone()));

        let mut params = Vec::new();
        params.push(Parameter::Named("stateMachine".to_string(), self.enumTy.asRef(), false));

        let mut bodyBuilder = BodyBuilder::new();
        let mut mainBuilder = bodyBuilder.createBlock();
        let stateMachineArg = bodyBuilder.createTempValueWithType(location.clone(), self.enumTy.asRef());
        let stateMachineAssign = InstructionKind::Assign(
            stateMachineArg.clone(),
            Variable::newWithType(
                VariableName::Arg("stateMachine".to_string()),
                location.clone(),
                self.enumTy.asRef(),
            ),
        );
        mainBuilder.addInstruction(stateMachineAssign, location.clone());

        // The completed state is the last variant (highest index)
        let completedIndex = self.entryPoints.len() as u32;
        let mut cases = Vec::new();

        for i in 0..=completedIndex {
            let mut stateBlock = bodyBuilder.createBlock();
            let resultVar = bodyBuilder.createTempValueWithType(location.clone(), boolTy.clone());

            if i == completedIndex {
                let trueCall = InstructionKind::FunctionCall(
                    resultVar.clone(),
                    CallInfo {
                        name: getTrueName(),
                        args: vec![],
                        context: None,
                        instanceRefs: Vec::new(),
                        coroutineSpawn: false,
                    },
                );
                stateBlock.addInstruction(trueCall, location.clone());
            } else {
                let falseCall = InstructionKind::FunctionCall(
                    resultVar.clone(),
                    CallInfo {
                        name: getFalseName(),
                        args: vec![],
                        context: None,
                        instanceRefs: Vec::new(),
                        coroutineSpawn: false,
                    },
                );
                stateBlock.addInstruction(falseCall, location.clone());
            }
            stateBlock.addReturn(resultVar, location.clone());

            cases.push(EnumCase {
                index: i,
                branch: stateBlock.getBlockId(),
            });
        }

        let enumSwitch = InstructionKind::EnumSwitch(stateMachineArg, cases);
        mainBuilder.addInstruction(enumSwitch, location.clone());

        Function {
            name: isCompletedFnName,
            params,
            result: ResultKind::SingleReturn(boolTy),
            body: Some(bodyBuilder.build()),
            kind: FunctionKind::UserDefined(location.clone()),
            constraintContext: ConstraintContext::new(),
            attributes: Attributes::new(),
        }
    }
}

fn getVariantName(functionName: &QualifiedName, index: usize) -> QualifiedName {
    QualifiedName::CoroutineStateMachineVariant(Box::new(functionName.clone()), index as u32)
}
