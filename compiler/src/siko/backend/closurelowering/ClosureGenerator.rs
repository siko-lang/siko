use crate::siko::{
    backend::closurelowering::ClosureLowering::{ClosureInfo, ClosureInstanceInfo, ClosureKey},
    hir::{
        BodyBuilder::BodyBuilder,
        ConstraintContext::ConstraintContext,
        Data::{Enum, Field, Struct, Variant},
        Function::{Function, FunctionKind, Parameter},
        Instruction::{CallInfo, EnumCase, FieldId, FieldInfo, InstructionKind},
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
        let mut variants = Vec::new();
        let enumName = self.closure.name.clone();
        let enumTy = Type::Named(enumName.clone(), Vec::new());
        for (variantIndex, (instance, name)) in self.closure.instances.iter().enumerate() {
            let variant = self.generateVariant(instance, name, variantIndex, &enumTy, &location);
            variants.push(variant);
        }
        let enumDef = Enum {
            name: enumName.clone(),
            ty: enumTy.clone(),
            variants,
            location: location.clone(),
            methods: Vec::new(),
            ownership_info: None,
        };
        self.program.enums.insert(enumDef.name.clone(), enumDef);
        enumTy
    }

    fn generateVariantStruct(
        &mut self,
        closureInstanceInfo: &ClosureInstanceInfo,
        closureInstanceName: &QualifiedName,
        location: &Location,
    ) -> Type {
        let mut fields = Vec::new();
        for ty in &closureInstanceInfo.envTypes {
            fields.push(Field {
                name: format!("arg{}", fields.len()),
                ty: ty.clone(),
            });
        }
        let structName = QualifiedName::ClosureInstanceEnvStruct(Box::new(closureInstanceName.clone()));
        let structTy = Type::Named(structName.clone(), Vec::new());
        let variantStruct = Struct {
            name: structName.clone(),
            fields: fields,
            location: location.clone(),
            ty: structTy.clone(),
            methods: Vec::new(),
            ownership_info: None,
        };
        self.program.structs.insert(variantStruct.name.clone(), variantStruct);
        let mut structCtorParams = Vec::new();
        for (i, ty) in closureInstanceInfo.envTypes.iter().enumerate() {
            let argName = format!("arg{}", i);
            structCtorParams.push(Parameter::Named(argName, ty.clone(), false));
        }
        let structCtorFn = Function {
            name: structName.clone(),
            params: structCtorParams,
            result: Type::Named(structName, Vec::new()),
            body: None,
            constraintContext: ConstraintContext::new(),
            kind: FunctionKind::StructCtor,
        };
        self.program.functions.insert(structCtorFn.name.clone(), structCtorFn);
        structTy
    }

    fn generateVariant(
        &mut self,
        closureInstanceInfo: &ClosureInstanceInfo,
        closureInstanceName: &QualifiedName,
        variantIndex: usize,
        enumTy: &Type,
        location: &Location,
    ) -> Variant {
        let structTy = self.generateVariantStruct(closureInstanceInfo, closureInstanceName, location);
        let variant = Variant {
            name: closureInstanceName.clone(),
            items: vec![structTy],
        };
        let mut variantCtorParams = Vec::new();
        for (i, ty) in closureInstanceInfo.envTypes.iter().enumerate() {
            let argName = format!("arg{}", i);
            variantCtorParams.push(Parameter::Named(argName, ty.clone(), false));
        }
        let variantCtorFn = Function {
            name: closureInstanceName.clone(),
            params: variantCtorParams,
            result: enumTy.clone(),
            body: None,
            constraintContext: ConstraintContext::new(),
            kind: FunctionKind::VariantCtor(variantIndex as i64),
        };
        self.program.functions.insert(variantCtorFn.name.clone(), variantCtorFn);
        variant
    }

    fn generateClosureCallHandlerFunction(&mut self, enumTy: &Type, location: Location) {
        let mut handlerParams = Vec::new();
        let mut argVars = Vec::new();
        let mut bodyBuilder = BodyBuilder::new();
        let firstParamName = "closure".to_string();
        handlerParams.push(Parameter::Named(firstParamName.clone(), enumTy.clone(), false));
        let closureArg = Variable::newWithType(VariableName::Arg(firstParamName), location.clone(), enumTy.clone());
        argVars.push(closureArg.clone());
        for (it, ty) in self.key.args.iter().enumerate() {
            let paramName = format!("arg{}", it);
            handlerParams.push(Parameter::Named(paramName.clone(), ty.clone(), false));
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
        let mut cases = Vec::new();
        for (variantIndex, (instance, name)) in self.closure.instances.iter().enumerate() {
            let mut handlerArgs = Vec::new();
            let mut caseBlock = bodyBuilder.createBlock();
            let structTy = Type::Named(
                QualifiedName::ClosureInstanceEnvStruct(Box::new(name.clone())),
                Vec::new(),
            );
            let closureEnvVar = bodyBuilder.createTempValueWithType(location.clone(), structTy);
            let transform = InstructionKind::Transform(closureEnvVar.clone(), closureArg.clone(), variantIndex as u32);
            caseBlock.addInstruction(transform, location.clone());
            let mut envVars = Vec::new();
            for (i, ty) in instance.envTypes.iter().enumerate() {
                let varName = format!("env{}", i);
                let envVar = Variable::newWithType(VariableName::Arg(varName), location.clone(), ty.clone());
                let fieldInfo = FieldInfo {
                    name: FieldId::Named(format!("arg{}", i)),
                    location: location.clone(),
                    ty: Some(ty.clone()),
                };
                let fieldRef = InstructionKind::FieldRef(envVar.clone(), closureEnvVar.clone(), vec![fieldInfo]);
                caseBlock.addInstruction(fieldRef, location.clone());
                envVars.push(envVar);
            }
            handlerArgs.extend(envVars.iter().cloned());
            handlerArgs.extend(args.iter().skip(1).cloned());
            let callInfo = CallInfo {
                name: instance.handler.clone(),
                context: None,
                instanceRefs: Vec::new(),
                args: handlerArgs,
            };
            let callResult = bodyBuilder.createTempValueWithType(location.clone(), self.key.result.clone());
            let fnCall = InstructionKind::FunctionCall(callResult.clone(), callInfo);
            caseBlock.addInstruction(fnCall, location.clone());
            caseBlock.addReturn(callResult, location.clone());
            let case = EnumCase {
                index: variantIndex as u32,
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
            result: self.key.result.clone(),
            body: Some(body),
            constraintContext: ConstraintContext::new(),
            kind: FunctionKind::UserDefined,
        };
        //println!("handler fn {}", handlerFn);
        self.program.functions.insert(handlerFn.name.clone(), handlerFn);
    }
}
