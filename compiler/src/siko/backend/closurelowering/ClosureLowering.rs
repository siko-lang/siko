use core::panic;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::fmt::Display;
use std::rc::Rc;

use crate::siko::hir::Body::Body;
use crate::siko::hir::ConstraintContext::ConstraintContext;
use crate::siko::hir::Data::Field;
use crate::siko::hir::Data::Struct;
use crate::siko::hir::Data::Variant;
use crate::siko::hir::Function::Function;
use crate::siko::hir::Function::FunctionKind;
use crate::siko::hir::Function::Parameter;
use crate::siko::hir::Instruction::CallInfo;
use crate::siko::hir::Instruction::FieldInfo;
use crate::siko::hir::Instruction::InstructionKind;
use crate::siko::hir::Type::formatTypes;
use crate::siko::hir::Variable::Variable;
use crate::siko::hir::{Data::Enum, Program::Program, Type::Type};
use crate::siko::location::Location::Location;
use crate::siko::qualifiedname::QualifiedName;

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
struct ClosureKey {
    args: Vec<Type>,
    result: Type,
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
struct ClosureInstance {
    envTypes: Vec<Type>,
    handler: QualifiedName,
}

impl Display for ClosureInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ClosureInstance({}, {})", self.handler, formatTypes(&self.envTypes))
    }
}

impl Debug for ClosureInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone)]
struct ClosureInfoPtr {
    info: Rc<RefCell<ClosureInfo>>,
}

struct ClosureInfo {
    name: QualifiedName,
    instances: BTreeMap<ClosureInstance, QualifiedName>,
}

impl ClosureInfo {
    pub fn new(name: QualifiedName) -> ClosureInfo {
        ClosureInfo {
            name,
            instances: BTreeMap::new(),
        }
    }

    fn addInstance(&mut self, instance: ClosureInstance) -> QualifiedName {
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
            e.lower(self);
        }
        for (_, s) in &mut program.structs {
            s.lower(self);
        }
        for (_, f) in &mut program.functions {
            f.params.lower(self);
            if let Some(body) = &mut f.body {
                body.lower(self);
            }
            //println!("Lowered function: {}", f);
        }
        for (key, c) in &self.closures {
            println!("Generating closure {}", key);
            let closure = c.info.borrow();
            generateClosure(&mut program, key, closure);
        }
        program
    }

    fn getClosureInfo(&mut self, args: Vec<Type>, result: Type) -> ClosureInfoPtr {
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
        let ptr = self.getClosureInfo(args, result);
        let info = ptr.info.borrow();
        info.name.clone()
    }

    fn getClosureInstanceName(&mut self, args: Vec<Type>, result: Type, instance: ClosureInstance) -> QualifiedName {
        let ptr = self.getClosureInfo(args, result);
        let mut info = ptr.info.borrow_mut();
        //println!("Created closure type: {}", info.name);
        info.addInstance(instance)
    }
}

fn generateClosure(program: &mut Program, key: &ClosureKey, closure: std::cell::Ref<'_, ClosureInfo>) {
    let mut variants = Vec::new();
    let enumName = closure.name.clone();
    let enumTy = Type::Named(enumName.clone(), Vec::new());
    for (variantIndex, (instance, name)) in closure.instances.iter().enumerate() {
        let mut fields = Vec::new();
        for ty in &instance.envTypes {
            fields.push(Field {
                name: format!("arg{}", fields.len()),
                ty: ty.clone(),
            });
        }
        let structName = QualifiedName::ClosureEnvStruct(Box::new(name.clone()));
        let structTy = Type::Named(structName.clone(), Vec::new());
        let variantStruct = Struct {
            name: structName.clone(),
            fields: fields,
            location: Location::empty(),
            ty: structTy.clone(),
            methods: Vec::new(),
            ownership_info: None,
        };
        program.structs.insert(variantStruct.name.clone(), variantStruct);
        let mut structCtorParams = Vec::new();
        for (i, ty) in instance.envTypes.iter().enumerate() {
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
        program.functions.insert(structCtorFn.name.clone(), structCtorFn);
        let variant = Variant {
            name: name.clone(),
            items: vec![structTy],
        };
        variants.push(variant);
        let mut variantCtorParams = Vec::new();
        for (i, ty) in instance.envTypes.iter().enumerate() {
            let argName = format!("arg{}", i);
            variantCtorParams.push(Parameter::Named(argName, ty.clone(), false));
        }
        let variantCtorFn = Function {
            name: name.clone(),
            params: variantCtorParams,
            result: enumTy.clone(),
            body: None,
            constraintContext: ConstraintContext::new(),
            kind: FunctionKind::VariantCtor(variantIndex as i64),
        };
        program.functions.insert(variantCtorFn.name.clone(), variantCtorFn);
    }
    let enumDef = Enum {
        name: enumName.clone(),
        ty: enumTy.clone(),
        variants,
        location: Location::empty(),
        methods: Vec::new(),
        ownership_info: None,
    };
    program.enums.insert(enumDef.name.clone(), enumDef);
    let mut handlerParams = Vec::new();
    for (it, ty) in key.args.iter().enumerate() {
        let paramName = format!("arg{}", it);
        handlerParams.push(Parameter::Named(paramName, ty.clone(), false));
    }
    let handlerFn = Function {
        name: QualifiedName::ClosureCallHandler(Box::new(closure.name.clone())),
        params: handlerParams,
        result: key.result.clone(),
        body: None,
        constraintContext: ConstraintContext::new(),
        kind: FunctionKind::UserDefined,
    };
    program.functions.insert(handlerFn.name.clone(), handlerFn);
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
                let closureInstance = ClosureInstance {
                    envTypes: info.closureParams.iter().map(|p| p.getType()).collect(),
                    handler: info.name.clone(),
                };
                let (args, resTy) = dest
                    .getType()
                    .splitFnType()
                    .expect("create closure result is not fn type");
                let closureInstanceName = closureStore.getClosureInstanceName(args, resTy, closureInstance);
                dest.lower(closureStore);
                println!("closure params {:?} ", info.closureParams);
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
