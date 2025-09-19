use std::{collections::BTreeMap, fmt::Debug, fmt::Display};

use crate::siko::{
    backend::{coroutinelowering::CoroutineTransformer::CoroutineTransformer, RemoveTuples::getTuple},
    hir::{
        BodyBuilder::BodyBuilder,
        ConstraintContext::ConstraintContext,
        Function::{Attributes, Function, FunctionKind, Parameter, ResultKind},
        FunctionGroupBuilder::FunctionGroupBuilder,
        Instruction::{CallInfo, EnumCase, InstructionKind},
        Program::Program,
        Type::Type,
        Variable::{Variable, VariableName},
    },
    location::Location::Location,
    monomorphizer::Context::Context,
    qualifiedname::{
        builtins::{getCoroutineCoResumeName, getCoroutineCoResumeResultName},
        QualifiedName,
    },
};

#[derive(Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct CoroutineKey {
    pub yieldedTy: Type,
    pub resumedTy: Type,
    pub returnTy: Type,
}

impl Display for CoroutineKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "coroutineKey({}, {}, {})",
            self.yieldedTy, self.resumedTy, self.returnTy
        )
    }
}

impl Debug for CoroutineKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

pub struct CoroutineInstanceInfo {
    pub name: QualifiedName,
    pub resumeFnName: QualifiedName,
}

pub struct CoroutineInfo {
    pub key: CoroutineKey,
    pub instances: BTreeMap<QualifiedName, CoroutineInstanceInfo>,
}

impl CoroutineInfo {
    pub fn new(key: CoroutineKey) -> Self {
        Self {
            key,
            instances: BTreeMap::new(),
        }
    }
}

pub struct CoroutineStore {
    pub coroutines: BTreeMap<CoroutineKey, CoroutineInfo>,
}

impl CoroutineStore {
    pub fn new() -> Self {
        Self {
            coroutines: BTreeMap::new(),
        }
    }

    pub fn process(&mut self, mut program: Program) -> Program {
        let functionGroupBuilder = FunctionGroupBuilder::new(&program);
        let functionGroupInfo = functionGroupBuilder.process();
        for group in &functionGroupInfo.groups {
            //println!("CoroutineStore: processing function group: {:?}", group.items);
            for fnName in &group.items {
                let func = program.functions.get(&fnName).unwrap().clone();
                if self.isCoroutineFunction(&func) {
                    let mut transformer = CoroutineTransformer::new(&func, &mut program);
                    let (f, coroutineInstanceInfo) = transformer.transform();
                    program.functions.insert(f.name.clone(), f);
                    let key = coroutineInstanceInfo.name.getCoroutineKey();
                    let coroutineKey = CoroutineKey {
                        yieldedTy: key.0,
                        resumedTy: key.1,
                        returnTy: key.2,
                    };
                    let coroutineInfo = self
                        .coroutines
                        .entry(coroutineKey.clone())
                        .or_insert(CoroutineInfo::new(coroutineKey));
                    coroutineInfo
                        .instances
                        .insert(coroutineInstanceInfo.name.clone(), coroutineInstanceInfo);
                }
            }
        }
        for (_, coroutine) in &self.coroutines {
            let f = self.generateResumeFunctionForCoroutine(coroutine);
            program.functions.insert(f.name.clone(), f);
        }
        program
    }

    fn isCoroutineFunction(&mut self, f: &Function) -> bool {
        f.result.isCoroutine()
    }

    fn generateResumeFunctionForCoroutine(&self, coroutine: &CoroutineInfo) -> Function {
        let mut ctx = Context::new();
        ctx.args.push(coroutine.key.yieldedTy.clone());
        ctx.args.push(coroutine.key.resumedTy.clone());
        ctx.args.push(coroutine.key.returnTy.clone());
        let resumeName = getCoroutineCoResumeName().monomorphized(ctx.clone());
        let location = Location::empty();
        let coroutineTy = Type::Named(
            QualifiedName::Coroutine(
                coroutine.key.yieldedTy.clone().into(),
                coroutine.key.resumedTy.clone().into(),
                coroutine.key.returnTy.clone().into(),
            ),
            vec![],
        );
        let resultTy = Type::Named(getCoroutineCoResumeResultName().monomorphized(ctx), vec![]);
        let tupleStructName = getTuple(&Type::Tuple(vec![coroutineTy.clone(), resultTy.clone()]));
        let finalResumeTupleTy = Type::Named(tupleStructName, vec![]);

        let mut params = Vec::new();
        params.push(Parameter::Named("coro".to_string(), coroutineTy.clone(), false));
        params.push(Parameter::Named(
            "resumedValue".to_string(),
            coroutine.key.resumedTy.clone(),
            false,
        ));

        let mut bodyBuilder = BodyBuilder::new();
        let mut mainBuilder = bodyBuilder.createBlock();
        let coroutineArg = bodyBuilder.createTempValueWithType(location.clone(), coroutineTy.clone());
        let resumedArg = bodyBuilder.createTempValueWithType(location.clone(), coroutine.key.resumedTy.clone());
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
                coroutine.key.resumedTy.clone(),
            ),
        );
        mainBuilder.addInstruction(resumeAssign, location.clone());
        let mut cases = Vec::new();
        for (variantIndex, instance) in coroutine.instances.values().enumerate() {
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
        println!("Generated resume function for coroutine {}\n{}", coroutine.key, f);
        f
    }
}
