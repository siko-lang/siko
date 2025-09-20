use crate::siko::{
    backend::{
        coroutinelowering::{
            CoroutineLowering::CoroutineInfo,
            Utils::{
                getLoweredCoroutineName, getLoweredCoroutineType, getMonomorphizedContext, getResumeResultType,
                getResumeTupleType,
            },
        },
        BuilderUtils::{getStructFieldName, EnumBuilder, StructBuilder},
    },
    hir::{
        BodyBuilder::BodyBuilder,
        ConstraintContext::ConstraintContext,
        Function::{Attributes, Function, FunctionKind, Parameter, ResultKind},
        Instruction::{CallInfo, EnumCase, FieldId, FieldInfo, InstructionKind, TransformInfo},
        Program::Program,
        Type::Type,
        Variable::{Variable, VariableName},
    },
    location::Location::Location,
    qualifiedname::builtins::getCoroutineCoResumeName,
};

pub struct CoroutineGenerator<'a> {
    pub coroutineInfo: &'a CoroutineInfo,
    pub program: &'a mut Program,
}

impl<'a> CoroutineGenerator<'a> {
    pub fn new(coroutineInfo: &'a CoroutineInfo, program: &'a mut Program) -> Self {
        Self { coroutineInfo, program }
    }

    pub fn generateResumeFunctionForCoroutine(&mut self) -> Function {
        let ctx = getMonomorphizedContext(&self.coroutineInfo.getCoroutineType());
        let resumeName = getCoroutineCoResumeName().monomorphized(ctx.clone());
        let location = Location::empty();
        let coroutineTy = getLoweredCoroutineType(&self.coroutineInfo.getCoroutineType());
        let resultTy = getResumeResultType(&self.coroutineInfo.getCoroutineType());
        let finalResumeTupleTy = getResumeTupleType(&self.coroutineInfo.getCoroutineType());
        let mut structBuilder = StructBuilder::new(self.program, location.clone());
        structBuilder.generateStruct(
            &vec![coroutineTy.clone(), resultTy.clone()],
            &finalResumeTupleTy.getName().expect("Failed to get name"),
        );
        let mut params = Vec::new();
        params.push(Parameter::Named("coro".to_string(), coroutineTy.clone(), false));
        let mut bodyBuilder = BodyBuilder::new();
        let mut mainBuilder = bodyBuilder.createBlock();
        let coroutineArg = bodyBuilder.createTempValueWithType(location.clone(), coroutineTy.clone());
        let coroutineAssign = InstructionKind::Assign(
            coroutineArg.clone(),
            Variable::newWithType(
                VariableName::Arg("coro".to_string()),
                location.clone(),
                coroutineTy.clone(),
            ),
        );
        mainBuilder.addInstruction(coroutineAssign, location.clone());
        let mut cases = Vec::new();
        for (variantIndex, (name, instance)) in self.coroutineInfo.instances.iter().enumerate() {
            let mut caseBuilder = bodyBuilder.createBlock();
            let structTy = Type::Named(name.clone(), Vec::new());
            let transformVar = bodyBuilder.createTempValueWithType(location.clone(), structTy.clone());
            let transform = InstructionKind::Transform(
                transformVar.clone(),
                coroutineArg.clone(),
                TransformInfo {
                    variantIndex: variantIndex as u32,
                },
            );
            caseBuilder.addInstruction(transform, location.clone());
            let fieldInfo = FieldInfo {
                name: FieldId::Named(getStructFieldName(0)),
                location: location.clone(),
                ty: Some(instance.stateMachineEnumTy.clone()),
            };
            let fieldRefVar =
                bodyBuilder.createTempValueWithType(location.clone(), instance.stateMachineEnumTy.clone());
            let fieldRef = InstructionKind::FieldRef(fieldRefVar.clone(), transformVar.clone(), vec![fieldInfo]);
            caseBuilder.addInstruction(fieldRef, location.clone());
            let resumeResult = bodyBuilder.createTempValueWithType(location.clone(), resultTy.clone());
            let callInfo = CallInfo {
                name: instance.resumeFnName.clone(),
                args: vec![fieldRefVar.useVar()],
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
        // println!(
        //     "Generated resume function for coroutine {}\n{}",
        //     self.coroutineInfo.key, f
        // );
        f
    }

    pub fn generateEnumForCoroutine(&mut self, location: &Location) -> Type {
        let enumName = getLoweredCoroutineName(&self.coroutineInfo.getCoroutineType());
        let mut builder = EnumBuilder::new(enumName.clone(), self.program, location.clone());
        for (variantIndex, (name, instance)) in self.coroutineInfo.instances.iter().enumerate() {
            let fieldTypes = vec![instance.stateMachineEnumTy.clone()];
            builder.generateVariant(name, &fieldTypes, variantIndex);
        }
        //println!("Generating coroutine enum: {}", enumName);
        builder.generateEnum(location);
        builder.getEnumType()
    }
}
