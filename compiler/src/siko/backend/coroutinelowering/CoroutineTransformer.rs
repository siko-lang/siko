use crate::siko::{
    backend::{
        coroutinelowering::{
            CoroutineLowering::CoroutineInstanceInfo,
            Utils::{getLoweredCoroutineName, getResumeResultType},
        },
        BuilderUtils::EnumBuilder,
        RemoveTuples::getTuple,
    },
    hir::{
        Block::BlockId,
        BodyBuilder::BodyBuilder,
        Function::{Function, Parameter, ResultKind},
        Instruction::{CallInfo, EnumCase, InstructionKind},
        Program::Program,
        Type::Type,
        Variable::{Variable, VariableName},
    },
    location::Location::Location,
    qualifiedname::QualifiedName,
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
    resumeResultTy: Type,
    resumeResultTupleName: QualifiedName,
    resumeResultTupleTy: Type,
}

impl<'a> CoroutineTransformer<'a> {
    pub fn new(f: &'a Function, program: &'a mut Program) -> CoroutineTransformer<'a> {
        let enumName = QualifiedName::CoroutineStateMachineEnum(Box::new(f.name.clone()));
        let enumTy = Type::Named(enumName.clone(), vec![]);
        let coroutineTy = f.result.getCoroutineType();
        let resumeResultTy = getResumeResultType(&coroutineTy);
        let resumeResultTupleTy = Type::Tuple(vec![enumTy.clone(), resumeResultTy.clone()]);
        let resumeResultTupleName = getTuple(&resumeResultTupleTy);
        let resumeResultTupleTy = Type::Named(resumeResultTupleName.clone(), vec![]);
        CoroutineTransformer {
            f,
            queue: Vec::new(),
            entryPoints: Vec::new(),
            program,
            enumName,
            enumTy,
            coroutineTy,
            resumeResultTy,
            resumeResultTupleName,
            resumeResultTupleTy,
        }
    }

    pub fn transform(&mut self) -> (Function, CoroutineInstanceInfo) {
        println!("Before transformation: {}", self.f);
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
        let allBlockIds = bodyBuilder.getAllBlockIds();
        self.queue.extend(allBlockIds);
        while let Some(blockId) = self.queue.pop() {
            self.processBlock(blockId, &mut bodyBuilder);
        }
        for entry in &self.entryPoints {
            println!(
                "Entry point at block {} for variables {:?}",
                entry.blockId, entry.variables
            );
        }
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
        let enumSwitch = InstructionKind::EnumSwitch(coroVar.useVar(), cases);
        mainBlockBuilder.addInstruction(enumSwitch, self.f.kind.getLocation());
        let mut f = self.f.clone();
        f.params = vec![Parameter::Named("coro".to_string(), self.enumTy.clone(), false)];
        f.result = ResultKind::SingleReturn(self.resumeResultTupleTy.clone());
        f.body = Some(bodyBuilder.build());
        println!("after transformation: {}", f);

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

    pub fn processBlock(&mut self, blockId: BlockId, bodyBuilder: &mut BodyBuilder) {
        println!("Processing block: {:?}", blockId);
        let mut builder = bodyBuilder.iterator(blockId);
        loop {
            if let Some(instr) = builder.getInstruction() {
                match &instr.kind {
                    InstructionKind::Yield(_, value) => {
                        let retVar = bodyBuilder.createTempValueWithType(instr.location.clone(), Type::getNeverType());
                        let newBlock = builder.splitBlock(0);
                        println!("Split at yield, created new block: {:?}", newBlock);
                        self.entryPoints.push(EntryPoint {
                            blockId: newBlock,
                            variables: vec![],
                        });
                        let mut newBuilder = bodyBuilder.iterator(newBlock);
                        newBuilder.removeInstruction();
                        self.queue.push(newBlock);
                        let variantVar =
                            bodyBuilder.createTempValueWithType(instr.location.clone(), self.enumTy.clone());
                        let variantName = getVariantName(&self.f.name, self.entryPoints.len() - 1);
                        let callInfo = CallInfo {
                            name: variantName,
                            args: Vec::new(),
                            context: None,
                            instanceRefs: Vec::new(),
                            coroutineSpawn: false,
                        };
                        let variantCtorCall = InstructionKind::FunctionCall(variantVar.clone(), callInfo);
                        builder.addInstruction(variantCtorCall, instr.location.clone());
                        builder.step();
                        let tupleVar = bodyBuilder
                            .createTempValueWithType(instr.location.clone(), self.resumeResultTupleTy.clone());
                        let tupleCallInfo = CallInfo {
                            name: self.resumeResultTupleName.clone(),
                            args: vec![variantVar.useVar(), retVar.useVar()],
                            context: None,
                            instanceRefs: Vec::new(),
                            coroutineSpawn: false,
                        };
                        let tupleCtorCall = InstructionKind::FunctionCall(tupleVar.clone(), tupleCallInfo);
                        builder.addInstruction(tupleCtorCall, instr.location.clone());
                        builder.step();
                        builder
                            .addInstruction(InstructionKind::Return(tupleVar, value.clone()), instr.location.clone());
                    }
                    _ => {}
                };
                builder.step();
            } else {
                break;
            }
        }
    }

    fn generateCoroutineStateMachineEnum(&mut self, location: Location) {
        let mut enumBuilder = EnumBuilder::new(self.enumName.clone(), self.program, location.clone());
        for (variantIndex, entryPoint) in self.entryPoints.clone().iter().enumerate() {
            let variantName = getVariantName(&self.f.name, variantIndex);
            let fieldTypes = entryPoint.variables.iter().map(|v| v.getType()).collect();
            enumBuilder.generateVariant(&variantName, &fieldTypes, variantIndex);
        }
        enumBuilder.generateEnum(&location);
    }
}

fn getVariantName(functionName: &QualifiedName, index: usize) -> QualifiedName {
    QualifiedName::CoroutineStateMachineVariant(Box::new(functionName.clone()), index as u32)
}
