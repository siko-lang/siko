use crate::siko::{
    backend::{coroutinelowering::CoroutineLowering::CoroutineInstanceInfo, BuilderUtils::EnumBuilder},
    hir::{
        Block::BlockId,
        BodyBuilder::BodyBuilder,
        ConstraintContext::ConstraintContext,
        Function::{Attributes, Function, FunctionKind, Parameter, ResultKind},
        Instruction::InstructionKind,
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
}

impl<'a> CoroutineTransformer<'a> {
    pub fn new(f: &'a Function, program: &'a mut Program) -> Self {
        Self {
            f,
            queue: Vec::new(),
            entryPoints: Vec::new(),
            program,
        }
    }

    pub fn transform(&mut self) -> (Function, CoroutineInstanceInfo) {
        println!("Before transformation: {}", self.f);
        let coroutineName = self.f.result.getCoroutineName();
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
        let mut f = self.f.clone();
        f.body = Some(bodyBuilder.build());
        println!("after transformation: {}", f);
        for entry in &self.entryPoints {
            println!(
                "Entry point at block {} for variables {:?}",
                entry.blockId, entry.variables
            );
        }
        let enumTy = self.generateCoroutineStateMachineEnum(f.kind.getLocation());
        self.generateInstanceResumeFunction(&enumTy, &f.kind.getLocation());
        let coroutineInstanceInfo = CoroutineInstanceInfo {
            name: QualifiedName::CoroutineInstance(
                Box::new(coroutineName),
                Box::new(QualifiedName::CoroutineStateMachineEnum(Box::new(f.name.clone()))),
            ),
            resumeFnName: QualifiedName::CoroutineStateMachineResume(Box::new(f.name.clone())),
            stateMachineEnumTy: enumTy,
        };
        (f, coroutineInstanceInfo)
    }

    pub fn processBlock(&mut self, blockId: BlockId, bodyBuilder: &mut BodyBuilder) {
        println!("Processing block: {:?}", blockId);
        let mut builder = bodyBuilder.iterator(blockId);
        loop {
            if let Some(instr) = builder.getInstruction() {
                match &instr.kind {
                    InstructionKind::Yield(yieldVar, value) => {
                        let retVar = bodyBuilder.createTempValueWithType(instr.location.clone(), Type::getNeverType());
                        let newBlock = builder.splitBlock(0);
                        println!("Split at yield, created new block: {:?}", newBlock);
                        self.entryPoints.push(EntryPoint {
                            blockId: newBlock,
                            variables: vec![yieldVar.clone()],
                        });
                        let mut newBuilder = bodyBuilder.iterator(newBlock);
                        newBuilder.removeInstruction();
                        self.queue.push(newBlock);
                        builder.addInstruction(InstructionKind::Return(retVar, value.clone()), instr.location.clone());
                    }
                    _ => {}
                };
                builder.step();
            } else {
                break;
            }
        }
    }

    fn generateCoroutineStateMachineEnum(&mut self, location: Location) -> Type {
        let mut enumBuilder = EnumBuilder::new(self.f.name.clone(), self.program, location.clone());
        for (variantIndex, entry) in self.entryPoints.clone().iter().enumerate() {
            let variantName =
                QualifiedName::CoroutineStateMachineVariant(Box::new(self.f.name.clone()), variantIndex as u32);
            let fieldTypes: Vec<Type> = entry.variables.iter().map(|v| v.getType()).collect();
            enumBuilder.generateVariant(&variantName, &fieldTypes, variantIndex);
        }
        enumBuilder.generateEnum(&location);
        enumBuilder.getEnumType()
    }

    fn generateInstanceResumeFunction(&mut self, enumTy: &Type, location: &Location) {
        let resumeFnName = QualifiedName::CoroutineStateMachineResume(Box::new(self.f.name.clone()));
        let mut params = Vec::new();
        params.push(Parameter::Named("self".to_string(), enumTy.clone(), false));
        let mut bodyBuilder = BodyBuilder::new();
        let mut mainBuilder = bodyBuilder.createBlock();
        let coroutineArg = bodyBuilder.createTempValueWithType(location.clone(), enumTy.clone());
        let assign = InstructionKind::Assign(
            coroutineArg.clone(),
            Variable::newWithType(VariableName::Arg("self".to_string()), location.clone(), enumTy.clone()),
        );
        mainBuilder.addInstruction(assign, location.clone());
        let resumeFn = Function {
            name: resumeFnName.clone(),
            params,
            result: ResultKind::SingleReturn(Type::getNeverType()),
            body: Some(bodyBuilder.build()),
            constraintContext: ConstraintContext::new(),
            kind: FunctionKind::UserDefined(location.clone()),
            attributes: Attributes::new(),
        };
        println!("Generated resume function: {}", resumeFn);
        self.program.functions.insert(resumeFn.name.clone(), resumeFn.clone());
    }
}
