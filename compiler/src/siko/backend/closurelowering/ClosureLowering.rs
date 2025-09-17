use core::panic;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::fmt::Display;
use std::rc::Rc;

use crate::siko::backend::closurelowering::ClosureGenerator::ClosureGenerator;
use crate::siko::hir::Body::Body;
use crate::siko::hir::Data::Struct;
use crate::siko::hir::Function::Parameter;
use crate::siko::hir::Instruction::CallInfo;
use crate::siko::hir::Instruction::FieldInfo;
use crate::siko::hir::Instruction::InstructionKind;
use crate::siko::hir::Type::formatTypes;
use crate::siko::hir::Variable::Variable;
use crate::siko::hir::{Data::Enum, Program::Program, Type::Type};
use crate::siko::qualifiedname::QualifiedName;

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct ClosureKey {
    pub args: Vec<Type>,
    pub result: Type,
}

impl Display for ClosureKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", formatTypes(&self.args), self.result)
    }
}

impl Debug for ClosureKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct ClosureInstanceInfo {
    pub envTypes: Vec<Type>,
    pub handler: QualifiedName,
}

impl Display for ClosureInstanceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ClosureInstance({}, {})", self.handler, formatTypes(&self.envTypes))
    }
}

impl Debug for ClosureInstanceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone)]
struct ClosureInfoPtr {
    info: Rc<RefCell<ClosureInfo>>,
}

pub struct ClosureInfo {
    pub name: QualifiedName,
    pub instances: BTreeMap<ClosureInstanceInfo, QualifiedName>,
}

impl ClosureInfo {
    pub fn new(name: QualifiedName) -> ClosureInfo {
        ClosureInfo {
            name,
            instances: BTreeMap::new(),
        }
    }

    fn addInstance(&mut self, instance: ClosureInstanceInfo) -> QualifiedName {
        let index = self.instances.len() as u32;
        let name = QualifiedName::ClosureInstance(Box::new(self.name.clone()), index);
        self.instances.insert(instance, name.clone());
        name
    }
}

struct ClosureStore {
    closures: BTreeMap<ClosureKey, ClosureInfoPtr>,
}

impl ClosureStore {
    pub fn new() -> ClosureStore {
        ClosureStore {
            closures: BTreeMap::new(),
        }
    }

    fn lower(&mut self, mut program: Program) -> Program {
        for (_, e) in &mut program.enums {
            //println!("Lowering enum: {}", e.name);
            e.lower(self);
        }
        for (_, s) in &mut program.structs {
            //println!("Lowering struct: {}", s.name);
            s.lower(self);
        }
        for (_, f) in &mut program.functions {
            // println!("Lowering function: {}", f.name);
            // println!("Lowering function: {}", f);
            f.params.lower(self);
            if let Some(body) = &mut f.body {
                body.lower(self);
            }
            //println!("Lowered function: {}", f);
        }
        for (key, c) in &self.closures {
            //println!("Generating closure {}", key);
            let closure = c.info.borrow();
            let mut generator = ClosureGenerator::new(&mut program, key, &*closure);
            generator.generateClosure();
        }
        program
    }

    fn getClosureInfo(&mut self, args: Vec<Type>, result: Type) -> ClosureInfoPtr {
        assert!(!result.isUnit());
        let entry = self.closures.entry(ClosureKey {
            args: args.clone(),
            result: result.clone(),
        });
        let entry = entry.or_insert_with(|| {
            let name = QualifiedName::Closure(args.clone(), Box::new(result.clone()));
            ClosureInfoPtr {
                info: Rc::new(RefCell::new(ClosureInfo::new(name))),
            }
        });
        entry.clone()
    }

    fn getClosureName(&mut self, args: Vec<Type>, result: Type) -> QualifiedName {
        assert!(!result.isUnit());
        let ptr = self.getClosureInfo(args, result);
        let info = ptr.info.borrow();
        info.name.clone()
    }

    fn getClosureInstanceName(
        &mut self,
        args: Vec<Type>,
        result: Type,
        instance: ClosureInstanceInfo,
    ) -> QualifiedName {
        let ptr = self.getClosureInfo(args, result);
        let mut info = ptr.info.borrow_mut();
        //println!("Created closure type: {}", info.name);
        info.addInstance(instance)
    }
}

pub fn process(program: Program) -> Program {
    ClosureStore::new().lower(program)
}

trait ClosureLowering {
    fn lower(&mut self, closureStore: &mut ClosureStore);
}

impl ClosureLowering for Type {
    fn lower(&mut self, closureStore: &mut ClosureStore) {
        match self {
            Type::Function(params, ret) => {
                let paramTypes: Vec<Type> = params.iter().map(|p| p.clone()).collect();
                let name = closureStore.getClosureName(paramTypes, *ret.clone());
                *self = Type::Named(name, Vec::new());
            }
            _ => {}
        }
    }
}

impl ClosureLowering for Enum {
    fn lower(&mut self, closureStore: &mut ClosureStore) {
        self.ty.lower(closureStore);
        for variant in &mut self.variants {
            for item in &mut variant.items {
                item.lower(closureStore);
            }
        }
    }
}

impl ClosureLowering for Struct {
    fn lower(&mut self, closureStore: &mut ClosureStore) {
        self.ty.lower(closureStore);
        for field in &mut self.fields {
            field.ty.lower(closureStore);
        }
    }
}

impl ClosureLowering for Body {
    fn lower(&mut self, closureStore: &mut ClosureStore) {
        for (_, block) in &mut self.blocks {
            let inner = block.getInner();
            let mut b = inner.borrow_mut();
            for instruction in &mut b.instructions {
                instruction.kind.lower(closureStore);
            }
        }
    }
}

impl ClosureLowering for InstructionKind {
    fn lower(&mut self, closureStore: &mut ClosureStore) {
        match self {
            InstructionKind::FunctionCall(dest, info) => {
                dest.lower(closureStore);
                info.args.lower(closureStore);
            }
            InstructionKind::Converter(dest, src) => {
                dest.lower(closureStore);
                src.lower(closureStore);
            }
            InstructionKind::MethodCall(_, _, _, _) => {
                panic!("MethodCall instruction found in closure lowering");
            }
            InstructionKind::DynamicFunctionCall(dest, closure, args) => {
                dest.lower(closureStore);
                closure.lower(closureStore);
                args.lower(closureStore);
                let closureType = closure.getType();
                let closureName = closureType
                    .getName()
                    .expect("Dynamic function call destination must have a name");
                let mut callArgs = Vec::new();
                callArgs.push(closure.clone());
                callArgs.extend(args.iter().cloned());
                let callInfo = CallInfo {
                    name: QualifiedName::ClosureCallHandler(Box::new(closureName)),
                    args: callArgs,
                    context: None,
                    instanceRefs: Vec::new(),
                };
                let kind = InstructionKind::FunctionCall(dest.clone(), callInfo);
                *self = kind;
            }
            InstructionKind::FieldRef(dest, receiver, infos) => {
                dest.lower(closureStore);
                receiver.lower(closureStore);
                infos.lower(closureStore);
            }
            InstructionKind::Bind(lhs, rhs, _) => {
                lhs.lower(closureStore);
                rhs.lower(closureStore);
            }
            InstructionKind::Tuple(_, _) => {
                panic!("Tuple instruction found in closure lowering");
            }
            InstructionKind::StringLiteral(v, _) => {
                v.lower(closureStore);
            }
            InstructionKind::IntegerLiteral(v, _) => {
                v.lower(closureStore);
            }
            InstructionKind::CharLiteral(v, _) => {
                v.lower(closureStore);
            }
            InstructionKind::Return(v, arg) => {
                v.lower(closureStore);
                arg.lower(closureStore);
            }
            InstructionKind::Ref(dest, arg) => {
                dest.lower(closureStore);
                arg.lower(closureStore);
            }
            InstructionKind::PtrOf(dest, arg) => {
                dest.lower(closureStore);
                arg.lower(closureStore);
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
                v1.lower(closureStore);
            }
            InstructionKind::Assign(lhs, rhs) => {
                lhs.lower(closureStore);
                rhs.lower(closureStore);
            }
            InstructionKind::FieldAssign(dest, rhs, infos) => {
                dest.lower(closureStore);
                rhs.lower(closureStore);
                infos.lower(closureStore);
            }
            InstructionKind::AddressOfField(dest, receiver, infos) => {
                dest.lower(closureStore);
                receiver.lower(closureStore);
                infos.lower(closureStore);
            }
            InstructionKind::DeclareVar(v, _) => {
                v.lower(closureStore);
            }
            InstructionKind::Transform(dest, v, _) => {
                dest.lower(closureStore);
                v.lower(closureStore);
            }
            InstructionKind::EnumSwitch(v, _) => {
                v.lower(closureStore);
            }
            InstructionKind::IntegerSwitch(v, _) => {
                v.lower(closureStore);
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
                v.lower(closureStore);
                v2.lower(closureStore);
            }
            InstructionKind::StorePtr(v1, v2) => {
                v1.lower(closureStore);
                v2.lower(closureStore);
            }
            InstructionKind::CreateClosure(dest, info) => {
                info.closureParams.lower(closureStore);
                let closureInstance = ClosureInstanceInfo {
                    envTypes: info.closureParams.iter().map(|p| p.getType()).collect(),
                    handler: info.name.clone(),
                };
                let (args, resTy) = dest
                    .getType()
                    .splitFnType()
                    .expect("create closure result is not fn type");
                let closureInstanceName = closureStore.getClosureInstanceName(args, resTy, closureInstance);
                dest.lower(closureStore);
                //println!("closure params {:?} ", info.closureParams);
                let callInfo = CallInfo {
                    name: closureInstanceName,
                    args: info.closureParams.clone(),
                    context: None,
                    instanceRefs: Vec::new(),
                };
                let kind = InstructionKind::FunctionCall(dest.clone(), callInfo);
                *self = kind;
            }
            InstructionKind::ClosureReturn(_, _, _) => {
                panic!("ClosureReturn instruction found in closure lowering");
            }
            InstructionKind::IntegerOp(_, _, _, _) => {}
            InstructionKind::Yield(v, a) => {
                v.lower(closureStore);
                a.lower(closureStore);
            }
            InstructionKind::SpawnCoroutine(v, a) => {
                v.lower(closureStore);
                a.lower(closureStore);
            }
        }
    }
}

impl ClosureLowering for Variable {
    fn lower(&mut self, closureStore: &mut ClosureStore) {
        let mut ty = self.getType();
        ty.lower(closureStore);
        self.setType(ty);
    }
}

impl ClosureLowering for FieldInfo {
    fn lower(&mut self, closureStore: &mut ClosureStore) {
        if let Some(ref mut ty) = self.ty {
            ty.lower(closureStore);
        }
    }
}

impl<T: ClosureLowering> ClosureLowering for Vec<T> {
    fn lower(&mut self, closureStore: &mut ClosureStore) {
        for v in self {
            v.lower(closureStore);
        }
    }
}

impl ClosureLowering for Parameter {
    fn lower(&mut self, closureStore: &mut ClosureStore) {
        match self {
            Parameter::Named(_, ref mut ty, _) | Parameter::SelfParam(_, ref mut ty) => {
                ty.lower(closureStore);
            }
        }
    }
}
