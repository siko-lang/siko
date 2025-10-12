use crate::siko::{
    backend::{
        closurelowering::ClosureLowering::{ClosureInfo, ClosureKey},
        BuilderUtils::EnumBuilder,
    },
    hir::{
        BodyBuilder::BodyBuilder,
        ConstraintContext::ConstraintContext,
        Function::{Attributes, Function, FunctionKind, ParamInfo, Parameter, ResultKind},
        Instruction::{CallInfo, EnumCase, FieldAccessInfo, FieldId, FieldInfo, InstructionKind, TransformInfo},
        Program::Program,
        Type::Type,
        Variable::{Variable, VariableName},
    },
    location::Location::Location,
    qualifiedname::QualifiedName,
};

pub struct ClosureGenerator<'a> {
    program: &'a mut Program,
    key: &'a ClosureKey,
    closure: &'a ClosureInfo,
}

impl ClosureGenerator<'_> {
    pub fn new<'a>(program: &'a mut Program, key: &'a ClosureKey, closure: &'a ClosureInfo) -> ClosureGenerator<'a> {
        ClosureGenerator { program, key, closure }
    }

    pub fn generateClosure(&mut self) {
        let location = Location::empty();
        let enumTy = self.generateEnum(location.clone());
        self.generateClosureCallHandlerFunction(&enumTy, location);
    }

    fn generateEnum(&mut self, location: Location) -> Type {
        let mut enumBuilder = EnumBuilder::new(self.closure.name.clone(), self.program, location.clone());
        for (variantIndex, (instance, name)) in self.closure.instances.iter().enumerate() {
            enumBuilder.generateVariant(name, &instance.envTypes, variantIndex);
        }
        enumBuilder.generateEnum(&location);
        enumBuilder.getEnumType()
    }

    fn generateClosureCallHandlerFunction(&mut self, enumTy: &Type, location: Location) {
        let mut handlerParams = Vec::new();
        let mut argVars = Vec::new();
        let mut bodyBuilder = BodyBuilder::new();
        let firstParamName = "closure".to_string();
        handlerParams.push(Parameter::Named(
            firstParamName.clone(),
            enumTy.clone(),
            ParamInfo::new(),
        ));
        let closureArg = Variable::newWithType(VariableName::Arg(firstParamName), location.clone(), enumTy.clone());
        argVars.push(closureArg.clone());
        for (it, ty) in self.key.args.iter().enumerate() {
            let paramName = format!("arg{}", it);
            handlerParams.push(Parameter::Named(paramName.clone(), ty.clone(), ParamInfo::new()));
            let var = Variable::newWithType(VariableName::Arg(paramName), location.clone(), ty.clone());
            argVars.push(var);
        }
        let mut mainBuilder = bodyBuilder.createBlock();
        let mut args = Vec::new();
        for arg in argVars {
            let tmp = bodyBuilder.createTempValueWithType(location.clone(), arg.getType().clone());
            mainBuilder.addAssign(tmp.clone(), arg, location.clone());
            args.push(tmp);
        }
        let closureArg = args[0].clone();
        let mut cases = Vec::new();
        for (variantIndex, (instance, _)) in self.closure.instances.iter().enumerate() {
            let mut handlerArgs = Vec::new();
            let mut caseBlock = bodyBuilder.createBlock();
            let tupleTy = Type::Tuple(instance.envTypes.clone());
            let closureEnvVar = bodyBuilder.createTempValueWithType(location.clone(), tupleTy);
            let transform = InstructionKind::Transform(
                closureEnvVar.clone(),
                closureArg.clone(),
                TransformInfo {
                    variantIndex: variantIndex as u32,
                },
            );
            caseBlock.addInstruction(transform, location.clone());
            let mut envVars = Vec::new();
            for (i, ty) in instance.envTypes.iter().enumerate() {
                let envVar = bodyBuilder.createTempValueWithType(location.clone(), ty.clone());
                let fieldInfo = FieldInfo {
                    name: FieldId::Indexed(i as u32),
                    location: location.clone(),
                    ty: Some(ty.clone()),
                };
                let fieldRef = InstructionKind::FieldAccess(
                    envVar.clone(),
                    FieldAccessInfo {
                        receiver: closureEnvVar.clone(),
                        fields: vec![fieldInfo],
                        isRef: false,
                    },
                );
                caseBlock.addInstruction(fieldRef, location.clone());
                envVars.push(envVar);
            }
            handlerArgs.extend(envVars.iter().cloned());
            handlerArgs.extend(args.iter().skip(1).cloned());
            let callInfo = CallInfo::new(instance.handler.clone(), handlerArgs);
            let callResult = bodyBuilder.createTempValueWithType(location.clone(), self.key.result.clone());
            let fnCall = InstructionKind::FunctionCall(callResult.clone(), callInfo);
            caseBlock.addInstruction(fnCall, location.clone());
            caseBlock.addReturn(callResult, location.clone());
            let case = EnumCase {
                index: Some(variantIndex as u32),
                branch: caseBlock.getBlockId(),
            };
            cases.push(case);
        }
        let body = bodyBuilder.build();
        let enumSwitch = InstructionKind::EnumSwitch(closureArg.clone(), cases);
        mainBuilder.addInstruction(enumSwitch, location.clone());
        let handlerFn = Function {
            name: QualifiedName::ClosureCallHandler(Box::new(self.closure.name.clone())),
            params: handlerParams,
            result: ResultKind::SingleReturn(self.key.result.clone()),
            body: Some(body),
            constraintContext: ConstraintContext::new(),
            kind: FunctionKind::UserDefined(location.clone()),
            attributes: Attributes::new(),
        };
        //println!("handler fn {}", handlerFn);
        self.program.functions.insert(handlerFn.name.clone(), handlerFn);
    }
}
