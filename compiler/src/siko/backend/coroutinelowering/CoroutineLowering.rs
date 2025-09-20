use core::panic;
use std::{collections::BTreeMap, fmt::Debug, fmt::Display};

use crate::siko::{
    backend::coroutinelowering::{
        CoroutineGenerator::CoroutineGenerator, CoroutineTransformer::CoroutineTransformer,
        Utils::getLoweredCoroutineType,
    },
    hir::{
        Block::Block,
        Body::Body,
        Data::{Enum, Struct},
        Function::{Function, Parameter, ResultKind},
        FunctionGroupBuilder::FunctionGroupBuilder,
        Instruction::{FieldInfo, Instruction, InstructionKind},
        Program::Program,
        Type::Type,
        Variable::Variable,
    },
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
                    let (mut f, coroutineInstanceInfo) = transformer.transform();
                    f.lower();
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
                } else {
                    let mut f = func.clone();
                    f.lower();
                    self.program.functions.insert(f.name.clone(), f);
                }
            }
        }
        //println!("CoroutineStore: found {} coroutines", self.coroutines.len());
        for (_, coroutine) in &self.coroutines {
            let mut generator = CoroutineGenerator::new(coroutine, self.program);
            generator.generateEnumForCoroutine(&Location::empty());
            let f = generator.generateResumeFunctionForCoroutine();
            self.program.functions.insert(f.name.clone(), f);
        }
        for (_, e) in &mut self.program.enums {
            //println!("Lowering enum: {}", e.name);
            e.lower();
        }
        for (_, s) in &mut self.program.structs {
            //println!("Lowering struct: {}", s.name);
            s.lower();
        }
    }

    fn isCoroutineFunction(&mut self, f: &Function) -> bool {
        f.result.isCoroutine()
    }
}

trait CoroutineLowering {
    fn lower(&mut self);
}

impl<T: CoroutineLowering> CoroutineLowering for Option<T> {
    fn lower(&mut self) {
        if let Some(inner) = self {
            inner.lower();
        }
    }
}

impl<T: CoroutineLowering> CoroutineLowering for Vec<T> {
    fn lower(&mut self) {
        for item in self {
            item.lower();
        }
    }
}

impl CoroutineLowering for Variable {
    fn lower(&mut self) {
        let mut ty = self.getType();
        ty.lower();
        self.setType(ty);
    }
}

impl CoroutineLowering for InstructionKind {
    fn lower(&mut self) {
        match self {
            InstructionKind::FunctionCall(dest, info) => {
                dest.lower();
                info.args.lower();
            }
            InstructionKind::Converter(dest, src) => {
                dest.lower();
                src.lower();
            }
            InstructionKind::MethodCall(_, _, _, _) => {
                panic!("MethodCall instruction found in coroutine lowering");
            }
            InstructionKind::DynamicFunctionCall(_, _, _) => {
                panic!("DynamicFunctionCall instruction found in coroutine lowering");
            }
            InstructionKind::FieldRef(dest, receiver, infos) => {
                dest.lower();
                receiver.lower();
                infos.lower();
            }
            InstructionKind::Bind(lhs, rhs, _) => {
                lhs.lower();
                rhs.lower();
            }
            InstructionKind::Tuple(_, _) => {
                panic!("Tuple instruction found in closure lowering");
            }
            InstructionKind::StringLiteral(v, _) => {
                v.lower();
            }
            InstructionKind::IntegerLiteral(v, _) => {
                v.lower();
            }
            InstructionKind::CharLiteral(v, _) => {
                v.lower();
            }
            InstructionKind::Return(v, arg) => {
                v.lower();
                arg.lower();
            }
            InstructionKind::Ref(dest, arg) => {
                dest.lower();
                arg.lower();
            }
            InstructionKind::PtrOf(dest, arg) => {
                dest.lower();
                arg.lower();
            }
            InstructionKind::DropPath(_) => {
                panic!("DropPath instruction found in closure lowering");
            }
            InstructionKind::DropMetadata(_) => {
                panic!("DropMetadata instruction found in closure lowering");
            }
            InstructionKind::Drop(_, _) => {
                panic!("Drop instruction found in closure lowering");
            }
            InstructionKind::Jump(v1, _) => {
                v1.lower();
            }
            InstructionKind::Assign(lhs, rhs) => {
                lhs.lower();
                rhs.lower();
            }
            InstructionKind::FieldAssign(dest, rhs, infos) => {
                dest.lower();
                rhs.lower();
                infos.lower();
            }
            InstructionKind::AddressOfField(dest, receiver, infos) => {
                dest.lower();
                receiver.lower();
                infos.lower();
            }
            InstructionKind::DeclareVar(v, _) => {
                v.lower();
            }
            InstructionKind::Transform(dest, v, _) => {
                dest.lower();
                v.lower();
            }
            InstructionKind::EnumSwitch(v, _) => {
                v.lower();
            }
            InstructionKind::IntegerSwitch(v, _) => {
                v.lower();
            }
            InstructionKind::BlockStart(_) => {}
            InstructionKind::BlockEnd(_) => {}
            InstructionKind::With(_, _) => {
                panic!("With instruction found in closure lowering");
            }
            InstructionKind::ReadImplicit(_, _) => {
                panic!("ReadImplicit instruction found in closure lowering");
            }
            InstructionKind::WriteImplicit(_, _) => {
                panic!("WriteImplicit instruction found in closure lowering");
            }
            InstructionKind::LoadPtr(v, v2) => {
                v.lower();
                v2.lower();
            }
            InstructionKind::StorePtr(v1, v2) => {
                v1.lower();
                v2.lower();
            }
            InstructionKind::CreateClosure(_, _) => {
                panic!("CreateClosure instruction found in coroutine lowering");
            }
            InstructionKind::ClosureReturn(_, _, _) => {
                panic!("ClosureReturn instruction found in closure lowering");
            }
            InstructionKind::IntegerOp(_, _, _, _) => {}
            InstructionKind::Yield(v, a) => {
                v.lower();
                a.lower();
            }
        }
    }
}

impl CoroutineLowering for Type {
    fn lower(&mut self) {
        match self {
            Type::Coroutine(_, _) => *self = getLoweredCoroutineType(self),
            _ => {}
        }
    }
}

impl CoroutineLowering for FieldInfo {
    fn lower(&mut self) {
        if let Some(ref mut ty) = self.ty {
            ty.lower();
        }
    }
}

impl CoroutineLowering for Instruction {
    fn lower(&mut self) {
        self.kind.lower();
    }
}

impl CoroutineLowering for Block {
    fn lower(&mut self) {
        let inner = self.getInner();
        let mut b = inner.borrow_mut();
        for instr in &mut b.instructions {
            instr.lower();
        }
    }
}

impl CoroutineLowering for Body {
    fn lower(&mut self) {
        for (_, block) in &mut self.blocks {
            block.lower();
        }
    }
}

impl CoroutineLowering for Parameter {
    fn lower(&mut self) {
        match self {
            Parameter::Named(_, ty, _) => {
                ty.lower();
            }
            Parameter::SelfParam(_, ty) => {
                ty.lower();
            }
        }
    }
}

impl CoroutineLowering for ResultKind {
    fn lower(&mut self) {
        match self {
            ResultKind::SingleReturn(ty) => {
                ty.lower();
            }
            ResultKind::Coroutine(_) => {
                panic!("ResultKind::Coroutine found in coroutine lowering, should have been transformed already");
            }
        }
    }
}

impl CoroutineLowering for Function {
    fn lower(&mut self) {
        self.body.lower();
        self.params.lower();
        self.result.lower();
    }
}

impl CoroutineLowering for Enum {
    fn lower(&mut self) {
        self.ty.lower();
        for variant in &mut self.variants {
            for item in &mut variant.items {
                item.lower();
            }
        }
    }
}

impl CoroutineLowering for Struct {
    fn lower(&mut self) {
        self.ty.lower();
        for field in &mut self.fields {
            field.ty.lower();
        }
    }
}
