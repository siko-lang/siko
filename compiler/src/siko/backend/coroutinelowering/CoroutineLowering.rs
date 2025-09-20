use std::{collections::BTreeMap, fmt::Debug, fmt::Display};

use crate::siko::{
    backend::coroutinelowering::{CoroutineGenerator::CoroutineGenerator, CoroutineTransformer::CoroutineTransformer},
    hir::{Function::Function, FunctionGroupBuilder::FunctionGroupBuilder, Program::Program, Type::Type},
    location::Location::Location,
    qualifiedname::QualifiedName,
};

#[derive(Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct CoroutineKey {
    pub yieldedTy: Type,
    pub returnTy: Type,
}

impl Display for CoroutineKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "coroutineKey({}, {})", self.yieldedTy, self.returnTy)
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
    pub stateMachineEnumTy: Type,
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

    pub fn getCoroutineType(&self) -> Type {
        Type::Coroutine(
            Box::new(self.key.yieldedTy.clone()),
            Box::new(self.key.returnTy.clone()),
        )
    }
}

pub struct CoroutineStore<'a> {
    pub coroutines: BTreeMap<CoroutineKey, CoroutineInfo>,
    pub program: &'a mut Program,
}

impl<'a> CoroutineStore<'a> {
    pub fn new(program: &'a mut Program) -> Self {
        Self {
            coroutines: BTreeMap::new(),
            program,
        }
    }

    pub fn process(mut self) {
        let functionGroupBuilder = FunctionGroupBuilder::new(self.program);
        let functionGroupInfo = functionGroupBuilder.process();
        for group in &functionGroupInfo.groups {
            //println!("CoroutineStore: processing function group: {:?}", group.items);
            for fnName in &group.items {
                let func = self.program.functions.get(&fnName).unwrap().clone();
                if self.isCoroutineFunction(&func) {
                    let mut transformer = CoroutineTransformer::new(&func, self.program);
                    let (f, coroutineInstanceInfo) = transformer.transform();
                    self.program.functions.insert(f.name.clone(), f);
                    let key = coroutineInstanceInfo.name.getCoroutineKey();
                    let coroutineKey = CoroutineKey {
                        yieldedTy: key.0,
                        returnTy: key.1,
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
            let mut generator = CoroutineGenerator::new(coroutine, self.program);
            generator.generateEnumForCoroutine(&Location::empty());
            let f = generator.generateResumeFunctionForCoroutine();
            self.program.functions.insert(f.name.clone(), f);
        }
    }

    fn isCoroutineFunction(&mut self, f: &Function) -> bool {
        f.result.isCoroutine()
    }
}
