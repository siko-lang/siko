use crate::siko::{
    backend::{coroutinelowering::CoroutineLowering::CoroutineInfo, BuilderUtils::EnumBuilder, RemoveTuples::getTuple},
    hir::{
        BodyBuilder::BodyBuilder,
        ConstraintContext::ConstraintContext,
        Function::{Attributes, Function, FunctionKind, Parameter, ResultKind},
        Instruction::{CallInfo, EnumCase, InstructionKind},
        Program::Program,
        Type::Type,
        Variable::{Variable, VariableName},
    },
    location::Location::Location,
    qualifiedname::builtins::{getCoroutineCoResumeName, getCoroutineCoResumeResultName},
};

pub struct CoroutineGenerator<'a> {
    pub coroutineInfo: &'a CoroutineInfo,
    pub program: &'a mut Program,
}

impl<'a> CoroutineGenerator<'a> {
    pub fn new(coroutineInfo: &'a CoroutineInfo, program: &'a mut Program) -> Self {
        Self { coroutineInfo, program }
    }

    pub fn generateResumeFunctionForCoroutine(&self) -> Function {
        let ctx = self.coroutineInfo.getContext();
        let resumeName = getCoroutineCoResumeName().monomorphized(ctx.clone());
        let location = Location::empty();
        let coroutineTy = self.coroutineInfo.getCoroutineType();
        let resultTy = Type::Named(getCoroutineCoResumeResultName().monomorphized(ctx), vec![]);
        let tupleStructName = getTuple(&Type::Tuple(vec![coroutineTy.clone(), resultTy.clone()]));
        let finalResumeTupleTy = Type::Named(tupleStructName, vec![]);

        let mut params = Vec::new();
        params.push(Parameter::Named("coro".to_string(), coroutineTy.clone(), false));
        params.push(Parameter::Named(
            "resumedValue".to_string(),
            self.coroutineInfo.key.resumedTy.clone(),
            false,
        ));

        let mut bodyBuilder = BodyBuilder::new();
        let mut mainBuilder = bodyBuilder.createBlock();
        let coroutineArg = bodyBuilder.createTempValueWithType(location.clone(), coroutineTy.clone());
        let resumedArg =
            bodyBuilder.createTempValueWithType(location.clone(), self.coroutineInfo.key.resumedTy.clone());
        let coroutineAssign = InstructionKind::Assign(
            coroutineArg.clone(),
            Variable::newWithType(
                VariableName::Arg("coro".to_string()),
                location.clone(),
                coroutineTy.clone(),
            ),
        );
        mainBuilder.addInstruction(coroutineAssign, location.clone());
        let resumeAssign = InstructionKind::Assign(
            resumedArg.clone(),
            Variable::newWithType(
                VariableName::Arg("resumedValue".to_string()),
                location.clone(),
                self.coroutineInfo.key.resumedTy.clone(),
            ),
        );
        mainBuilder.addInstruction(resumeAssign, location.clone());
        let mut cases = Vec::new();
        for (variantIndex, instance) in self.coroutineInfo.instances.values().enumerate() {
            let mut caseBuilder = bodyBuilder.createBlock();
            let resumeResult = bodyBuilder.createTempValueWithType(location.clone(), resultTy.clone());
            let callInfo = CallInfo {
                name: instance.resumeFnName.clone(),
                args: vec![coroutineArg.useVar(), resumedArg.useVar()],
                context: None,
                instanceRefs: Vec::new(),
                coroutineSpawn: false,
            };
            let resumeCall = InstructionKind::FunctionCall(resumeResult.clone(), callInfo);
            caseBuilder.addInstruction(resumeCall, location.clone());
            caseBuilder.addReturn(resumeResult, location.clone());
            cases.push(EnumCase {
                index: variantIndex as u32,
                branch: caseBuilder.getBlockId(),
            });
        }
        let enumSwitch = InstructionKind::EnumSwitch(coroutineArg, cases);
        mainBuilder.addInstruction(enumSwitch, location.clone());

        let f = Function {
            name: resumeName,
            params,
            result: ResultKind::SingleReturn(finalResumeTupleTy),
            body: Some(bodyBuilder.build()),
            kind: FunctionKind::UserDefined(location.clone()),
            constraintContext: ConstraintContext::new(),
            attributes: Attributes::new(),
        };
        println!(
            "Generated resume function for coroutine {}\n{}",
            self.coroutineInfo.key, f
        );
        f
    }

    pub fn generateEnumForCoroutine(&mut self, location: &Location) -> Type {
        let mut builder = EnumBuilder::new(self.coroutineInfo.getCoroutineName(), self.program, location.clone());
        for (variantIndex, (name, instance)) in self.coroutineInfo.instances.iter().enumerate() {
            let fieldTypes = vec![instance.stateMachineEnumTy.clone()];
            builder.generateVariant(name, &fieldTypes, variantIndex);
        }
        builder.generateEnum(location);
        builder.getEnumType()
    }
}
