use std::{
    collections::{BTreeMap, BTreeSet},
    iter::zip,
};

use crate::siko::{
    hir::{
        Data::{Class, Enum},
        Function::{Body, Function, Instruction, InstructionId, InstructionKind, Parameter, ValueKind},
        Program::Program,
        Substitution::{instantiateClass, instantiateEnum, Apply, Substitution},
        TraitMethodSelector::TraitMethodSelector,
        Type::Type,
        TypeVarAllocator::TypeVarAllocator,
        Unification::unify,
    },
    location::{Location::Location, Report::ReportContext},
    qualifiedname::QualifiedName,
};

use super::Error::TypecheckerError;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum TypedId {
    Instruction(InstructionId),
    Value(String),
}

fn reportError(ctx: &ReportContext, ty1: Type, ty2: Type, location: Location) {
    TypecheckerError::TypeMismatch(format!("{}", ty1), format!("{}", ty2), location).report(ctx)
}

pub struct Typechecker<'a> {
    ctx: &'a ReportContext,
    program: &'a Program,
    traitMethodSelector: &'a TraitMethodSelector,
    allocator: TypeVarAllocator,
    substitution: Substitution<Type>,
    methodSources: BTreeMap<InstructionId, QualifiedName>,
    methodCalls: BTreeMap<InstructionId, InstructionId>,
    instructionSwaps: Substitution<InstructionId>,
    types: BTreeMap<TypedId, Type>,
    selfType: Option<Type>,
}

impl<'a> Typechecker<'a> {
    pub fn new(ctx: &'a ReportContext, program: &'a Program, traitMethodSelector: &'a TraitMethodSelector) -> Typechecker<'a> {
        Typechecker {
            ctx: ctx,
            program: program,
            traitMethodSelector: traitMethodSelector,
            allocator: TypeVarAllocator::new(),
            substitution: Substitution::new(),
            methodSources: BTreeMap::new(),
            methodCalls: BTreeMap::new(),
            instructionSwaps: Substitution::new(),
            types: BTreeMap::new(),
            selfType: None,
        }
    }

    pub fn run(&mut self, f: &Function) -> Function {
        self.initialize(f);
        //self.dump(f);
        self.check(f);
        self.verify(f);
        //self.dump(f);
        self.generate(f)
    }

    pub fn initialize(&mut self, f: &Function) {
        //println!("Initializing {}", f.name);
        for param in &f.params {
            match &param {
                Parameter::Named(name, ty, _) => {
                    self.types.insert(TypedId::Value(name.clone()), ty.clone());
                }
                Parameter::SelfParam(_, ty) => {
                    self.types.insert(TypedId::Value(format!("self")), ty.clone());
                    self.selfType = Some(ty.clone());
                }
            }
        }
        if let Some(body) = &f.body {
            for block in &body.blocks {
                for instruction in &block.instructions {
                    match &instruction.kind {
                        InstructionKind::Jump(_) | InstructionKind::Return(_) => {
                            self.types.insert(TypedId::Instruction(instruction.id), Type::Never);
                        }
                        _ => {
                            let ty = self.allocator.next();
                            self.types.insert(TypedId::Instruction(instruction.id), ty.clone());
                        }
                    }
                    match &instruction.kind {
                        InstructionKind::DeclareVar(name) => {
                            self.types.insert(TypedId::Value(name.to_string()), self.allocator.next());
                        }
                        InstructionKind::Bind(name, _) => {
                            self.types.insert(TypedId::Value(name.to_string()), self.allocator.next());
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn getType(&self, id: &TypedId) -> Type {
        self.types.get(id).expect("No type found!").clone()
    }

    fn getInstructionType(&self, id: InstructionId) -> Type {
        self.types.get(&TypedId::Instruction(id)).expect("not type for instruction").clone()
    }

    fn getValueType(&self, v: &String) -> Type {
        self.types.get(&TypedId::Value(v.clone())).expect("not type for value").clone()
    }

    fn unify(&mut self, ty1: Type, ty2: Type, location: Location) {
        //println!("UNIFY {} {}", ty1, ty2);
        if let Err(_) = unify(&mut self.substitution, &ty1, &ty2) {
            reportError(self.ctx, ty1.apply(&self.substitution), ty2.apply(&self.substitution), location);
        }
    }

    fn instantiateType(&mut self, ty: Type) -> Type {
        let vars = ty.collectVars(BTreeSet::new());
        let mut sub = Substitution::new();
        for var in &vars {
            sub.add(Type::Var(var.clone()), self.allocator.next());
        }
        ty.apply(&sub)
    }

    fn instantiateEnum(&mut self, e: &Enum, ty: &Type) -> Enum {
        instantiateEnum(&mut self.allocator, e, ty)
    }

    fn instantiateClass(&mut self, c: &Class, ty: &Type) -> Class {
        instantiateClass(&mut self.allocator, c, ty)
    }

    fn checkFunctionCall(&mut self, args: &Vec<InstructionId>, body: &Body, instruction: &Instruction, fnType: Type) {
        //println!("checkFunctionCall: {}", fnType);
        let fnType = self.instantiateType(fnType);
        let (fnArgs, mut fnResult) = match fnType.splitFnType() {
            Some((fnArgs, fnResult)) => (fnArgs, fnResult),
            None => return,
        };
        if args.len() != fnArgs.len() {
            TypecheckerError::ArgCountMismatch(fnArgs.len() as u32, args.len() as u32, instruction.location.clone()).report(self.ctx);
        }
        if fnArgs.len() > 0 {
            fnResult = fnResult.changeSelfType(fnArgs[0].clone());
        }
        for (arg, fnArg) in zip(args, fnArgs) {
            self.unify(self.getInstructionType(*arg), fnArg, body.getInstruction(*arg).location.clone());
        }
        self.unify(self.getInstructionType(instruction.id), fnResult, instruction.location.clone());
    }

    fn check(&mut self, f: &Function) {
        let body = if let Some(body) = &f.body {
            body
        } else {
            return;
        };
        for instruction in f.instructions() {
            //println!("Type checking {}", instruction);
            if let Some(ty) = &instruction.ty {
                self.unify(self.getInstructionType(instruction.id), ty.clone(), instruction.location.clone());
            }
            match &instruction.kind {
                InstructionKind::FunctionCall(name, args) => {
                    let f = self.program.functions.get(name).expect("Function not found");
                    let fnType = f.getType();
                    self.checkFunctionCall(args, body, instruction, fnType);
                }
                InstructionKind::DynamicFunctionCall(callable, args) => match self.methodSources.get(callable) {
                    Some(name) => {
                        let f = self.program.functions.get(&name).expect("Function not found");
                        let fnType = f.getType();
                        let mut newArgs = Vec::new();
                        newArgs.push(*callable);
                        newArgs.extend(args);
                        self.checkFunctionCall(&newArgs, body, instruction, fnType);
                        self.methodCalls.insert(instruction.id, *callable);
                    }
                    None => {
                        let fnType = self.getInstructionType(*callable);
                        self.checkFunctionCall(&args, body, instruction, fnType);
                    }
                },
                InstructionKind::ValueRef(value) => {
                    let receiverType = match &value {
                        ValueKind::Arg(name, _) => self.getValueType(name),
                        ValueKind::Value(name) => self.getValueType(name),
                    };
                    self.unify(receiverType, self.getInstructionType(instruction.id), instruction.location.clone());
                }
                InstructionKind::Bind(name, rhs) => {
                    self.unify(self.getValueType(name), self.getInstructionType(*rhs), instruction.location.clone());
                    self.unify(self.getInstructionType(instruction.id), Type::getUnitType(), instruction.location.clone());
                }
                InstructionKind::Tuple(args) => {
                    let mut argTypes = Vec::new();
                    for arg in args {
                        argTypes.push(self.getInstructionType(*arg));
                    }
                    self.unify(
                        self.getInstructionType(instruction.id),
                        Type::Tuple(argTypes),
                        instruction.location.clone(),
                    );
                }
                InstructionKind::StringLiteral(_) => {
                    self.unify(
                        self.getInstructionType(instruction.id),
                        Type::getStringType(),
                        instruction.location.clone(),
                    );
                }
                InstructionKind::IntegerLiteral(_) => {
                    self.unify(self.getInstructionType(instruction.id), Type::getIntType(), instruction.location.clone());
                }
                InstructionKind::CharLiteral(_) => {
                    self.unify(self.getInstructionType(instruction.id), Type::getCharType(), instruction.location.clone());
                }
                InstructionKind::Return(arg) => {
                    let mut result = f.result.clone();
                    if let Some(selfType) = self.selfType.clone() {
                        result = result.changeSelfType(selfType);
                    }
                    self.unify(result, self.getInstructionType(*arg), instruction.location.clone());
                    self.unify(self.getInstructionType(instruction.id), Type::Never, instruction.location.clone());
                }
                InstructionKind::Ref(arg) => {
                    let arg_type = self.getInstructionType(*arg);
                    self.unify(
                        self.getInstructionType(instruction.id),
                        Type::Reference(Box::new(arg_type), None),
                        instruction.location.clone(),
                    );
                }
                InstructionKind::Drop(_) => {
                    self.unify(self.getInstructionType(instruction.id), Type::getUnitType(), instruction.location.clone());
                }
                InstructionKind::Jump(_) => {}
                InstructionKind::Assign(name, rhs) => {
                    self.unify(self.getValueType(name), self.getInstructionType(*rhs), instruction.location.clone());
                    self.unify(self.getInstructionType(instruction.id), Type::getUnitType(), instruction.location.clone());
                }
                InstructionKind::DeclareVar(var) => {
                    self.unify(
                        self.getInstructionType(instruction.id),
                        self.getValueType(var),
                        instruction.location.clone(),
                    );
                }
                InstructionKind::Transform(root, index, _) => {
                    let rootTy = self.getInstructionType(*root);
                    let rootTy = rootTy.apply(&self.substitution);
                    match rootTy.getName() {
                        Some(name) => {
                            let e = self.program.enums.get(&name).expect("not an enum in transform!");
                            let e = self.instantiateEnum(e, &rootTy);
                            let v = &e.variants[*index as usize];
                            self.unify(
                                self.getInstructionType(instruction.id),
                                Type::Tuple(v.items.clone()),
                                instruction.location.clone(),
                            );
                        }
                        None => {
                            TypecheckerError::TypeAnnotationNeeded(instruction.location.clone()).report(self.ctx);
                        }
                    };
                }
                InstructionKind::EnumSwitch(_root, _cases) => {
                    self.unify(self.getInstructionType(instruction.id), Type::getUnitType(), instruction.location.clone());
                }
                InstructionKind::IntegerSwitch(_root, _cases) => {
                    self.unify(self.getInstructionType(instruction.id), Type::getUnitType(), instruction.location.clone());
                }
                InstructionKind::StringSwitch(_root, _cases) => {
                    self.unify(self.getInstructionType(instruction.id), Type::getUnitType(), instruction.location.clone());
                }
                InstructionKind::FieldRef(receiver, fieldName) => {
                    let receiverType = self.getInstructionType(*receiver);
                    let receiverType = receiverType.apply(&self.substitution);
                    match receiverType.unpackRef() {
                        Type::Named(name, _, _) => {
                            if let Some(classDef) = self.program.classes.get(&name) {
                                let classDef = self.instantiateClass(classDef, &receiverType);
                                let mut found = false;
                                for f in &classDef.fields {
                                    if f.name == *fieldName {
                                        self.unify(self.getInstructionType(instruction.id), f.ty.clone(), instruction.location.clone());
                                        found = true;
                                    }
                                }
                                if !found {
                                    for m in &classDef.methods {
                                        if m.name == *fieldName {
                                            found = true;
                                            self.methodSources.insert(instruction.id, m.fullName.clone());
                                            self.instructionSwaps.add(instruction.id, *receiver);
                                            break;
                                        }
                                    }
                                }
                                if !found {
                                    if let Some(methodName) = self.traitMethodSelector.get(&fieldName) {
                                        found = true;
                                        self.methodSources.insert(instruction.id, methodName);
                                    }
                                }
                                if !found {
                                    TypecheckerError::FieldNotFound(fieldName.clone(), instruction.location.clone()).report(self.ctx);
                                }
                            } else {
                                TypecheckerError::TypeAnnotationNeeded(instruction.location.clone()).report(self.ctx);
                            }
                        }
                        _ => {
                            TypecheckerError::TypeAnnotationNeeded(instruction.location.clone()).report(self.ctx);
                        }
                    }
                }
                InstructionKind::TupleIndex(receiver, index) => {
                    let receiverType = self.getInstructionType(*receiver);
                    let receiverType = receiverType.apply(&self.substitution);
                    match receiverType {
                        Type::Tuple(t) => {
                            if *index as usize >= t.len() {
                                TypecheckerError::FieldNotFound(format!(".{}", index), instruction.location.clone()).report(&self.ctx);
                            }
                            let fieldType = t[*index as usize].clone();
                            self.unify(self.getInstructionType(instruction.id), fieldType, instruction.location.clone());
                        }
                        _ => TypecheckerError::TypeAnnotationNeeded(instruction.location.clone()).report(self.ctx),
                    }
                }
                InstructionKind::Noop => {
                    self.unify(self.getInstructionType(instruction.id), Type::getUnitType(), instruction.location.clone());
                }
            }
        }
    }

    pub fn verify(&self, f: &Function) {
        if let Some(body) = &f.body {
            let fnType = f.getType();
            let publicVars = fnType.collectVars(BTreeSet::new());
            for block in &body.blocks {
                for instruction in &block.instructions {
                    let ty = self.getType(&TypedId::Instruction(instruction.id));
                    let ty = ty.apply(&self.substitution);
                    let vars = ty.collectVars(BTreeSet::new());
                    if !vars.is_empty() && vars != publicVars {
                        self.dump(f);
                        println!("{} {}", instruction, ty);
                        TypecheckerError::TypeAnnotationNeeded(instruction.location.clone()).report(self.ctx);
                    }
                }
            }
        }
    }

    pub fn dump(&self, f: &Function) {
        println!("Dumping {}", f.name);
        if let Some(body) = &f.body {
            for block in &body.blocks {
                for instruction in &block.instructions {
                    let ty = self.getType(&TypedId::Instruction(instruction.id));
                    let ty = ty.apply(&self.substitution);
                    println!("{} : {}", instruction, ty);
                }
            }
        }
    }

    pub fn generate(&self, f: &Function) -> Function {
        //println!("Generating {}", f.name);
        let mut result = f.clone();
        if let Some(selfType) = self.selfType.clone() {
            result.result = result.result.changeSelfType(selfType);
        }
        if let Some(body) = &mut result.body {
            for block in &mut body.blocks {
                for instruction in &mut block.instructions {
                    if self.methodSources.contains_key(&instruction.id) {
                        match &instruction.kind {
                            InstructionKind::FieldRef(_, _) => {
                                instruction.kind = InstructionKind::Noop;
                            }
                            kind => panic!("Unexpected instruction kind for method source while rewriting! {}", kind.dump()),
                        }
                    }
                    if let Some(source) = self.methodCalls.get(&instruction.id) {
                        match &instruction.kind {
                            InstructionKind::DynamicFunctionCall(_, args) => {
                                let name = self.methodSources.get(source).expect("Method not found for call!");
                                let mut newArgs = Vec::new();
                                newArgs.push(*source);
                                newArgs.extend(args);
                                instruction.kind = InstructionKind::FunctionCall(name.clone(), newArgs);
                            }
                            kind => panic!("Unexpected instruction kind for method call while rewriting! {}", kind.dump()),
                        }
                    }
                    let ty = self.getType(&TypedId::Instruction(instruction.id));
                    let ty = ty.apply(&self.substitution);
                    //println!("{} : {}", instruction, ty);
                    instruction.ty = Some(ty.clone());
                    if let InstructionKind::Transform(_, _, oldTy) = &mut instruction.kind {
                        *oldTy = ty;
                    }
                    instruction.kind = instruction.kind.apply(&self.instructionSwaps);
                }
            }
        }
        result
    }
}

impl Apply<InstructionId> for InstructionId {
    fn apply(&self, sub: &Substitution<InstructionId>) -> Self {
        sub.get(self.clone())
    }
}

impl Apply<InstructionId> for InstructionKind {
    fn apply(&self, sub: &Substitution<InstructionId>) -> Self {
        match self {
            InstructionKind::FunctionCall(name, args) => InstructionKind::FunctionCall(name.clone(), args.apply(sub)),
            InstructionKind::DynamicFunctionCall(receiver, args) => InstructionKind::DynamicFunctionCall(receiver.apply(sub), args.apply(sub)),
            InstructionKind::ValueRef(value) => InstructionKind::ValueRef(value.clone()),
            InstructionKind::FieldRef(root, field) => InstructionKind::FieldRef(root.apply(sub), field.clone()),
            InstructionKind::TupleIndex(root, index) => InstructionKind::TupleIndex(root.apply(sub), *index),
            InstructionKind::Bind(var, rhs) => InstructionKind::Bind(var.clone(), rhs.apply(sub)),
            InstructionKind::Tuple(vec) => InstructionKind::Tuple(vec.apply(sub)),
            InstructionKind::StringLiteral(lit) => InstructionKind::StringLiteral(lit.clone()),
            InstructionKind::IntegerLiteral(lit) => InstructionKind::IntegerLiteral(lit.clone()),
            InstructionKind::CharLiteral(ch) => InstructionKind::CharLiteral(*ch),
            InstructionKind::Return(instruction_id) => InstructionKind::Return(instruction_id.apply(sub)),
            InstructionKind::Ref(instruction_id) => InstructionKind::Ref(instruction_id.apply(sub)),
            InstructionKind::Drop(args) => InstructionKind::Drop(args.clone()),
            InstructionKind::Jump(block_id) => InstructionKind::Jump(*block_id),
            InstructionKind::Assign(var, rhs) => InstructionKind::Assign(var.clone(), rhs.apply(sub)),
            InstructionKind::DeclareVar(var) => InstructionKind::DeclareVar(var.clone()),
            InstructionKind::Transform(root, op, args) => InstructionKind::Transform(root.apply(sub), *op, args.clone()),
            InstructionKind::EnumSwitch(root, cases) => InstructionKind::EnumSwitch(root.apply(sub), cases.clone()),
            InstructionKind::IntegerSwitch(root, cases) => InstructionKind::IntegerSwitch(root.apply(sub), cases.clone()),
            InstructionKind::StringSwitch(root, cases) => InstructionKind::StringSwitch(root.apply(sub), cases.clone()),
            InstructionKind::Noop => InstructionKind::Noop,
        }
    }
}
