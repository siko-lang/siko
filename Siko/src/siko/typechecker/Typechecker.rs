use std::{
    collections::{BTreeMap, BTreeSet},
    iter::zip,
};

use crate::siko::{
    hir::{
        Apply::{instantiateClass, instantiateEnum, instantiateType, instantiateType2, Apply, ApplyVariable},
        ConstraintContext::{Constraint as HirConstraint, ConstraintContext},
        Data::{Class, Enum},
        Function::{Body, Function, Instruction, InstructionKind, Parameter, Variable},
        InstanceResolver::ResolutionResult,
        Program::Program,
        Substitution::{TypeSubstitution, VariableSubstitution},
        TraitMethodSelector::TraitMethodSelector,
        Type::{formatTypes, Type, TypeVar},
        TypeVarAllocator::TypeVarAllocator,
        Unification::unify,
    },
    location::{Location::Location, Report::ReportContext},
    qualifiedname::QualifiedName,
};

use super::Error::TypecheckerError;

fn reportError(ctx: &ReportContext, ty1: Type, ty2: Type, location: Location) {
    TypecheckerError::TypeMismatch(format!("{}", ty1), format!("{}", ty2), location).report(ctx)
}

struct MutableMethodCallInfo {
    receiver: Variable,
    baseType: Type,
    selfLessType: Type,
}

struct Constraint {
    ty: Type,
    traitName: QualifiedName,
    location: Location,
}

pub struct Typechecker<'a> {
    ctx: &'a ReportContext,
    program: &'a Program,
    traitMethodSelector: &'a TraitMethodSelector,
    allocator: TypeVarAllocator,
    substitution: TypeSubstitution,
    methodCalls: BTreeMap<Variable, QualifiedName>,
    types: BTreeMap<String, Type>,
    selfType: Option<Type>,
    mutables: BTreeSet<String>,
    implicitRefs: BTreeSet<Variable>,
    mutableMethodCalls: BTreeMap<Variable, MutableMethodCallInfo>,
    fieldTypes: BTreeMap<Variable, Vec<Type>>,
}

impl<'a> Typechecker<'a> {
    pub fn new(ctx: &'a ReportContext, program: &'a Program, traitMethodSelector: &'a TraitMethodSelector) -> Typechecker<'a> {
        Typechecker {
            ctx: ctx,
            program: program,
            traitMethodSelector: traitMethodSelector,
            allocator: TypeVarAllocator::new(),
            substitution: TypeSubstitution::new(),
            methodCalls: BTreeMap::new(),
            types: BTreeMap::new(),
            selfType: None,
            mutables: BTreeSet::new(),
            implicitRefs: BTreeSet::new(),
            mutableMethodCalls: BTreeMap::new(),
            fieldTypes: BTreeMap::new(),
        }
    }

    pub fn run(&mut self, f: &Function) -> Function {
        self.initialize(f);
        //self.dump(f);
        self.check(f);
        //self.dump(f);
        self.generate(f)
    }

    fn initializeVar(&mut self, var: &Variable, body: &Body) {
        match &var.ty {
            Some(ty) => {
                self.types.insert(var.value.clone(), ty.clone());
            }
            None => {
                if let Some(ty) = body.varTypes.get(&var.value) {
                    self.types.insert(var.value.clone(), ty.clone());
                } else {
                    let ty = self.allocator.next();
                    self.types.insert(var.value.clone(), ty.clone());
                }
            }
        }
    }

    pub fn initialize(&mut self, f: &Function) {
        //println!("Initializing {}", f.name);
        for param in &f.params {
            match &param {
                Parameter::Named(name, ty, mutable) => {
                    self.types.insert(name.clone(), ty.clone());
                    if *mutable {
                        self.mutables.insert(name.clone());
                    }
                }
                Parameter::SelfParam(mutable, ty) => {
                    let name = format!("self");
                    self.types.insert(name.clone(), ty.clone());
                    self.selfType = Some(ty.clone());
                    if *mutable {
                        self.mutables.insert(name);
                    }
                }
            }
        }
        if let Some(body) = &f.body {
            for block in &body.blocks {
                for instruction in &block.instructions {
                    match &instruction.kind {
                        InstructionKind::FunctionCall(var, _, _) => {
                            self.initializeVar(var, body);
                        }
                        InstructionKind::MethodCall(var, _, _, _) => {
                            self.initializeVar(var, body);
                        }
                        InstructionKind::DynamicFunctionCall(var, _, _) => {
                            self.initializeVar(var, body);
                        }
                        InstructionKind::ValueRef(var, _) => {
                            self.initializeVar(var, body);
                        }
                        InstructionKind::FieldRef(var, _, _) => {
                            self.initializeVar(var, body);
                        }
                        InstructionKind::TupleIndex(var, _, _) => {
                            self.initializeVar(var, body);
                        }
                        InstructionKind::Bind(var, _, mutable) => {
                            self.initializeVar(var, body);
                            if *mutable {
                                self.mutables.insert(var.value.clone());
                            }
                        }
                        InstructionKind::Tuple(var, _) => {
                            self.initializeVar(var, body);
                        }
                        InstructionKind::StringLiteral(var, _) => {
                            self.initializeVar(var, body);
                        }
                        InstructionKind::IntegerLiteral(var, _) => {
                            self.initializeVar(var, body);
                        }
                        InstructionKind::CharLiteral(var, _) => {
                            self.initializeVar(var, body);
                        }
                        InstructionKind::Return(var, _) => {
                            self.types.insert(var.value.clone(), Type::Never);
                        }
                        InstructionKind::Ref(var, _) => {
                            self.initializeVar(var, body);
                        }
                        InstructionKind::Drop(_) => {}
                        InstructionKind::Jump(var, _) => {
                            self.types.insert(var.value.clone(), Type::Never);
                        }
                        InstructionKind::Assign(_, _) => {}
                        InstructionKind::FieldAssign(_, _, _) => {}
                        InstructionKind::DeclareVar(var) => {
                            self.initializeVar(var, body);
                            self.mutables.insert(var.value.clone());
                        }
                        InstructionKind::Transform(var, _, _) => {
                            self.initializeVar(var, body);
                        }
                        InstructionKind::EnumSwitch(_, _) => {}
                        InstructionKind::IntegerSwitch(_, _) => {}
                        InstructionKind::StringSwitch(_, _) => {}
                    }
                }
            }
        }
    }

    fn getType(&self, var: &Variable) -> Type {
        match self.types.get(&var.value) {
            Some(ty) => ty.clone(),
            None => panic!("No type found for {}!", var),
        }
    }

    fn unify(&mut self, ty1: Type, ty2: Type, location: Location) {
        //println!("UNIFY {} {}", ty1, ty2);
        if let Err(_) = unify(&mut self.substitution, &ty1, &ty2, false) {
            reportError(self.ctx, ty1.apply(&self.substitution), ty2.apply(&self.substitution), location);
        }
    }

    fn instantiateType(&mut self, ty: Type) -> Type {
        instantiateType(&mut self.allocator, &ty)
    }

    fn instantiateEnum(&mut self, e: &Enum, ty: &Type) -> Enum {
        instantiateEnum(&mut self.allocator, e, ty)
    }

    fn instantiateClass(&mut self, c: &Class, ty: &Type) -> Class {
        instantiateClass(&mut self.allocator, c, ty)
    }

    fn checkFunctionCall(
        &mut self,
        args: &Vec<Variable>,
        resultVar: &Variable,
        fnType: Type,
        neededConstraints: &ConstraintContext,
        knownConstraints: &ConstraintContext,
    ) {
        //println!("checkFunctionCall: {} {}", fnType, constraintContext);
        let (fnType, sub) = instantiateType2(&mut self.allocator, &fnType);
        let constraintContext = neededConstraints.apply(&sub);
        let (fnArgs, mut fnResult) = match fnType.splitFnType() {
            Some((fnArgs, fnResult)) => (fnArgs, fnResult),
            None => return,
        };
        if args.len() != fnArgs.len() {
            TypecheckerError::ArgCountMismatch(fnArgs.len() as u32, args.len() as u32, resultVar.location.clone()).report(self.ctx);
        }
        if fnArgs.len() > 0 {
            fnResult = fnResult.changeSelfType(fnArgs[0].clone());
        }
        for (arg, fnArg) in zip(args, fnArgs) {
            let mut argTy = self.getType(arg);
            argTy = argTy.apply(&self.substitution);
            let fnArg = fnArg.apply(&self.substitution);
            if !argTy.isReference() && fnArg.isReference() {
                argTy = Type::Reference(Box::new(argTy), None);
                //println!("IMPLICIT REF FOR {}", arg);
                self.implicitRefs.insert(arg.clone());
            }
            self.unify(argTy, fnArg, arg.location.clone());
        }
        let constraints = constraintContext.apply(&self.substitution);
        self.checkConstraint(&constraints, knownConstraints, resultVar.location.clone());
        self.unify(self.getType(resultVar), fnResult, resultVar.location.clone());
    }

    fn checkTraitFunctionCall(
        &mut self,
        args: &Vec<Variable>,
        resultVar: &Variable,
        fnType: Type,
        traitName: QualifiedName,
        memberName: QualifiedName,
        constraintContext: &ConstraintContext,
    ) {
        println!("checkTraitFunctionCall: {} {}", fnType, constraintContext);
        let traitDef = self.program.getTrait(&traitName);
        //println!("trait {}", traitDef);
        let (fnType, sub) = instantiateType2(&mut self.allocator, &fnType);
        let traitDef = traitDef.apply(&sub);
        //println!("trait {}", traitDef);
        //println!("fnType: {}", fnType);
        let (fnArgs, mut fnResult) = match fnType.splitFnType() {
            Some((fnArgs, fnResult)) => (fnArgs, fnResult),
            None => return,
        };
        if args.len() != fnArgs.len() {
            TypecheckerError::ArgCountMismatch(fnArgs.len() as u32, args.len() as u32, resultVar.location.clone()).report(self.ctx);
        }
        if fnArgs.len() > 0 {
            fnResult = fnResult.changeSelfType(fnArgs[0].clone());
        }
        for (arg, fnArg) in zip(args, fnArgs) {
            let mut argTy = self.getType(arg);
            argTy = argTy.apply(&self.substitution);
            let fnArg = fnArg.apply(&self.substitution);
            if !argTy.isReference() && fnArg.isReference() {
                argTy = Type::Reference(Box::new(argTy), None);
                //println!("IMPLICIT REF FOR {}", arg);
                self.implicitRefs.insert(arg.clone());
            }
            self.unify(argTy, fnArg, arg.location.clone());
        }
        self.unify(self.getType(resultVar), fnResult, resultVar.location.clone());
        let traitDef = traitDef.apply(&self.substitution).unwrap();
        //println!("final trait {}", traitDef);
        let mut fullySpecified = true;
        for param in &traitDef.params {
            if !param.isConcrete() {
                fullySpecified = false;
            }
        }
        if fullySpecified {
            if let Some(instances) = self.program.instanceResolver.lookupInstances(&traitName) {
                let resolutionResult = instances.find(&mut self.allocator, &traitDef.params);
                match resolutionResult {
                    ResolutionResult::Winner(instance) => {
                        //println!("winner instance {}", instance);
                        for m in &instance.members {
                            let base = m.fullName.base();
                            if base == memberName {
                                self.unify(self.getType(resultVar), m.result.clone(), resultVar.location.clone());
                                break;
                            }
                        }
                    }
                    ResolutionResult::Ambiguous(_) => {
                        TypecheckerError::AmbiguousInstances(
                            traitName.toString(),
                            formatTypes(&traitDef.params),
                            resultVar.location.clone(),
                            Vec::new(),
                        )
                        .report(self.ctx);
                    }
                    ResolutionResult::NoInstanceFound => {
                        TypecheckerError::InstanceNotFound(traitName.toString(), formatTypes(&traitDef.params), resultVar.location.clone())
                            .report(self.ctx);
                    }
                }
            } else {
                TypecheckerError::InstanceNotFound(traitName.toString(), formatTypes(&traitDef.params), resultVar.location.clone()).report(self.ctx);
            }
        } else {
            let constraint = HirConstraint {
                traitName: traitName.clone(),
                args: traitDef.params.clone(),
                associatedTypes: Vec::new(),
            };
            //println!("{}", constraint);
            //println!("Available: constraints {}", constraintContext);
            if !constraintContext.contains(&constraint) {
                if let Some(instances) = self.program.instanceResolver.lookupInstances(&traitName) {
                    let resolutionResult = instances.find(&mut self.allocator, &traitDef.params);
                    match resolutionResult {
                        ResolutionResult::Winner(instance) => {
                            //println!("Winner {} for {}", instance, formatTypes(&traitDef.params));
                            for m in &instance.members {
                                let base = m.fullName.base();
                                if base == memberName {
                                    self.unify(self.getType(resultVar), m.result.clone(), resultVar.location.clone());
                                    break;
                                }
                            }
                        }
                        ResolutionResult::Ambiguous(_) => {
                            TypecheckerError::AmbiguousInstances(
                                traitName.toString(),
                                formatTypes(&traitDef.params),
                                resultVar.location.clone(),
                                Vec::new(),
                            )
                            .report(self.ctx);
                        }
                        ResolutionResult::NoInstanceFound => {
                            TypecheckerError::InstanceNotFound(traitName.toString(), formatTypes(&traitDef.params), resultVar.location.clone())
                                .report(self.ctx);
                        }
                    }
                } else {
                    TypecheckerError::InstanceNotFound(traitName.toString(), formatTypes(&traitDef.params), resultVar.location.clone())
                        .report(self.ctx);
                }
            }
        }
    }

    fn checkConstraint(&mut self, neededConstraints: &ConstraintContext, knownConstraints: &ConstraintContext, location: Location) {
        //println!("needed {}", neededConstraints);
        //println!("known {}", knownConstraints);
        for c in &neededConstraints.constraints {
            if !knownConstraints.contains(c) {
                if let Some(instances) = self.program.instanceResolver.lookupInstances(&c.traitName) {
                    let resolutionResult = instances.find(&mut self.allocator, &c.args);
                    match resolutionResult {
                        ResolutionResult::Winner(instance) => {
                            //println!("Winner {} for {}", instance, formatTypes(&c.args));
                            for ctxAssocTy in &c.associatedTypes {
                                for instanceAssocTy in &instance.associatedTypes {
                                    if instanceAssocTy.name == ctxAssocTy.name {
                                        if let Err(_) = unify(&mut self.substitution, &instanceAssocTy.ty, &ctxAssocTy.ty, false) {
                                            reportError(
                                                self.ctx,
                                                instanceAssocTy.ty.apply(&self.substitution),
                                                ctxAssocTy.ty.apply(&self.substitution),
                                                location.clone(),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        ResolutionResult::Ambiguous(_) => {
                            TypecheckerError::AmbiguousInstances(c.traitName.toString(), formatTypes(&c.args), location.clone(), Vec::new())
                                .report(self.ctx);
                        }
                        ResolutionResult::NoInstanceFound => {
                            TypecheckerError::InstanceNotFound(c.traitName.toString(), formatTypes(&c.args), location.clone()).report(self.ctx);
                        }
                    }
                } else {
                    TypecheckerError::InstanceNotFound(c.traitName.toString(), formatTypes(&c.args), location.clone()).report(self.ctx);
                }
            }
        }
    }

    fn lookupTraitMethod(&mut self, methodName: &String, location: Location) -> QualifiedName {
        if let Some(selections) = self.traitMethodSelector.get(methodName) {
            if selections.len() > 1 {
                TypecheckerError::MethodAmbiguous(methodName.clone(), location.clone()).report(self.ctx);
            }
            return selections[0].method.clone();
        }
        TypecheckerError::MethodNotFound(methodName.clone(), location.clone()).report(self.ctx);
    }

    fn lookupMethod(&mut self, receiverType: Type, methodName: &String, location: Location) -> QualifiedName {
        match receiverType.unpackRef() {
            Type::Named(name, _, _) => {
                if let Some(classDef) = self.program.classes.get(&name) {
                    let classDef = self.instantiateClass(classDef, receiverType.unpackRef());
                    for m in &classDef.methods {
                        if m.name == *methodName {
                            //println!("Added {} {}", dest, m.fullName);
                            return m.fullName.clone();
                        }
                    }
                    return self.lookupTraitMethod(methodName, location);
                } else if let Some(enumDef) = self.program.enums.get(&name) {
                    let enumDef = self.instantiateEnum(enumDef, receiverType.unpackRef());
                    for m in &enumDef.methods {
                        if m.name == *methodName {
                            return m.fullName.clone();
                        }
                    }
                    return self.lookupTraitMethod(methodName, location);
                } else {
                    TypecheckerError::TypeAnnotationNeeded(location.clone()).report(self.ctx);
                }
            }
            Type::Var(TypeVar::Named(_)) => {
                return self.lookupTraitMethod(methodName, location);
            }
            _ => {
                TypecheckerError::TypeAnnotationNeeded(location.clone()).report(self.ctx);
            }
        };
    }

    fn checkField(&mut self, receiverType: Type, fieldName: String, location: Location) -> Type {
        let receiverType = receiverType.apply(&self.substitution);
        match receiverType.unpackRef() {
            Type::Named(name, _, _) => {
                if let Some(classDef) = self.program.classes.get(&name) {
                    let classDef = self.instantiateClass(classDef, receiverType.unpackRef());
                    for f in &classDef.fields {
                        if f.name == *fieldName {
                            return f.ty.clone();
                        }
                    }
                    TypecheckerError::FieldNotFound(fieldName.clone(), location.clone()).report(self.ctx);
                } else {
                    TypecheckerError::TypeAnnotationNeeded(location.clone()).report(self.ctx);
                }
            }
            _ => {
                TypecheckerError::TypeAnnotationNeeded(location.clone()).report(self.ctx);
            }
        }
    }

    fn check(&mut self, f: &Function) {
        //println!("checking {}", f.name);
        if f.body.is_none() {
            return;
        };
        for instruction in f.instructions() {
            //println!("Type checking {}", instruction);
            match &instruction.kind {
                InstructionKind::FunctionCall(dest, name, args) => {
                    let targetFn = self.program.functions.get(name).expect("Function not found");
                    let fnType = targetFn.getType();
                    self.checkFunctionCall(args, dest, fnType, &targetFn.constraintContext, &f.constraintContext);
                }
                InstructionKind::MethodCall(dest, receiver, methodName, args) => {
                    let receiverType = self.getType(receiver);
                    let receiverType = receiverType.apply(&self.substitution);
                    //println!("METHOD {} {} {}", methodName, receiver, receiverType);
                    let name = self.lookupMethod(receiverType.clone(), methodName, instruction.location.clone());
                    self.methodCalls.insert(dest.clone(), name.clone());
                    let targetFn = self.program.functions.get(&name).expect("Function not found");
                    let mut fnType = targetFn.getType();
                    let mut args = args.clone();
                    args.insert(0, receiver.clone());
                    if self.mutables.contains(&receiver.value) && fnType.getResult().hasSelfType() {
                        let originalType = fnType.getResult();
                        fnType = fnType.changeMethodResult();
                        let baseType = originalType.changeSelfType(receiverType);
                        let selfLessType = originalType.getSelflessType(false);
                        //println!("MUT METHOD {} => {}", originalType, selfLessType);
                        //println!("MUT METHOD {} => {}", baseType, selfLessType);
                        self.mutableMethodCalls.insert(
                            dest.clone(),
                            MutableMethodCallInfo {
                                receiver: receiver.clone(),
                                baseType,
                                selfLessType,
                            },
                        );
                    }
                    //if let Some(traitName) = targetFn.kind.isTraitCall() {
                    //    self.checkTraitFunctionCall(&args, dest, fnType, traitName, name.clone(), &f.constraintContext);
                    //} else {
                    self.checkFunctionCall(&args, dest, fnType, &targetFn.constraintContext, &f.constraintContext);
                    //}
                }
                InstructionKind::DynamicFunctionCall(dest, callable, args) => {
                    let fnType = self.getType(callable);
                    self.checkFunctionCall(&args, dest, fnType, &ConstraintContext::new(), &f.constraintContext);
                }
                InstructionKind::ValueRef(dest, value) => {
                    let receiverType = self.getType(value);
                    self.unify(receiverType, self.getType(dest), instruction.location.clone());
                }
                InstructionKind::Bind(name, rhs, _) => {
                    self.unify(self.getType(name), self.getType(rhs), instruction.location.clone());
                }
                InstructionKind::Tuple(dest, args) => {
                    let mut argTypes = Vec::new();
                    for arg in args {
                        argTypes.push(self.getType(arg));
                    }
                    self.unify(self.getType(dest), Type::Tuple(argTypes), instruction.location.clone());
                }
                InstructionKind::StringLiteral(dest, _) => {
                    self.unify(self.getType(dest), Type::getStringType(), instruction.location.clone());
                }
                InstructionKind::IntegerLiteral(dest, _) => {
                    self.unify(self.getType(dest), Type::getIntType(), instruction.location.clone());
                }
                InstructionKind::CharLiteral(dest, _) => {
                    self.unify(self.getType(dest), Type::getCharType(), instruction.location.clone());
                }
                InstructionKind::Return(_, arg) => {
                    let mut result = f.result.clone();
                    if let Some(selfType) = self.selfType.clone() {
                        result = result.changeSelfType(selfType);
                    }
                    self.unify(result, self.getType(arg), instruction.location.clone());
                }
                InstructionKind::Ref(dest, arg) => {
                    let arg_type = self.getType(arg);
                    self.unify(
                        self.getType(dest),
                        Type::Reference(Box::new(arg_type), None),
                        instruction.location.clone(),
                    );
                }
                InstructionKind::Drop(_) => {}
                InstructionKind::Jump(_, _) => {}
                InstructionKind::Assign(name, rhs) => {
                    if !self.mutables.contains(&name.value) {
                        TypecheckerError::ImmutableAssign(instruction.location.clone()).report(self.ctx);
                    }
                    self.unify(self.getType(name), self.getType(rhs), instruction.location.clone());
                }
                InstructionKind::FieldAssign(name, rhs, fields) => {
                    if !self.mutables.contains(&name.value) {
                        TypecheckerError::ImmutableAssign(instruction.location.clone()).report(self.ctx);
                    }
                    let receiverType = self.getType(name);
                    let mut types = Vec::new();
                    let mut receiverType = receiverType.apply(&self.substitution);
                    for field in fields {
                        let fieldTy = self.checkField(receiverType, field.name.clone(), field.location.clone());
                        types.push(fieldTy.clone());
                        receiverType = fieldTy;
                    }
                    self.fieldTypes.insert(name.clone(), types);
                    self.unify(self.getType(rhs), receiverType, instruction.location.clone());
                }
                InstructionKind::DeclareVar(_) => {}
                InstructionKind::Transform(dest, root, index) => {
                    let rootTy = self.getType(root);
                    let rootTy = rootTy.apply(&self.substitution);
                    match rootTy.getName() {
                        Some(name) => {
                            let e = self.program.enums.get(&name).expect("not an enum in transform!");
                            let e = self.instantiateEnum(e, &rootTy);
                            let v = &e.variants[*index as usize];
                            self.unify(self.getType(dest), Type::Tuple(v.items.clone()), instruction.location.clone());
                        }
                        None => {
                            TypecheckerError::TypeAnnotationNeeded(instruction.location.clone()).report(self.ctx);
                        }
                    };
                }
                InstructionKind::EnumSwitch(_root, _cases) => {}
                InstructionKind::IntegerSwitch(_root, _cases) => {}
                InstructionKind::StringSwitch(_root, _cases) => {}
                InstructionKind::FieldRef(dest, receiver, fieldName) => {
                    let receiverType = self.getType(receiver);
                    let fieldTy = self.checkField(receiverType, fieldName.clone(), instruction.location.clone());
                    self.unify(self.getType(dest), fieldTy, instruction.location.clone());
                }
                InstructionKind::TupleIndex(dest, receiver, index) => {
                    let receiverType = self.getType(receiver);
                    let receiverType = receiverType.apply(&self.substitution);
                    match receiverType {
                        Type::Tuple(t) => {
                            if *index as usize >= t.len() {
                                TypecheckerError::FieldNotFound(format!(".{}", index), instruction.location.clone()).report(&self.ctx);
                            }
                            let fieldType = t[*index as usize].clone();
                            self.unify(self.getType(dest), fieldType, instruction.location.clone());
                        }
                        _ => TypecheckerError::TypeAnnotationNeeded(instruction.location.clone()).report(self.ctx),
                    }
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
                    if let Some(v) = instruction.kind.getResultVar() {
                        if let Some(ty) = v.ty {
                            let vars = ty.collectVars(BTreeSet::new());
                            if !publicVars.is_superset(&vars) {
                                self.dump(f);
                                println!("MISSING: {} {}", instruction, ty);
                                TypecheckerError::TypeAnnotationNeeded(v.location.clone()).report(self.ctx);
                            }
                        } else {
                            TypecheckerError::TypeAnnotationNeeded(v.location.clone()).report(self.ctx);
                        }
                    }
                }
            }
        }
    }

    pub fn dump(&self, f: &Function) {
        println!("Dumping {}", f.name);
        if let Some(body) = &f.body {
            for block in &body.blocks {
                println!("{}:", block.id);
                for instruction in &block.instructions {
                    match instruction.kind.getResultVar() {
                        Some(v) => match v.ty {
                            Some(ty) => {
                                println!("  {} : {}", instruction, ty);
                            }
                            None => {
                                let ty = self.getType(&v);
                                let ty = ty.apply(&self.substitution);
                                println!("  {} : {} inferred", instruction, ty);
                            }
                        },
                        None => {
                            println!("  {}", instruction);
                        }
                    }
                }
            }
        }
    }

    fn convertMethodCalls(&mut self, f: &mut Function) {
        let body = &mut f.body.as_mut().unwrap();

        for block in &mut body.blocks {
            for instruction in &mut block.instructions {
                if let InstructionKind::MethodCall(dest, root, _, args) = &mut instruction.kind {
                    if let Some(fnName) = self.methodCalls.get(&dest) {
                        let mut newArgs = Vec::new();
                        newArgs.push(root.clone());
                        newArgs.extend(args.clone());
                        instruction.kind = InstructionKind::FunctionCall(dest.clone(), fnName.clone(), newArgs);
                    }
                }
            }
        }
    }

    fn addImplicitRefs(&mut self, f: &mut Function) {
        let mut nextImplicitRef = 0;

        let body = &mut f.body.as_mut().unwrap();

        for block in &mut body.blocks {
            let mut index = 0;
            loop {
                if index >= block.instructions.len() {
                    break;
                }
                let mut instruction = block.instructions[index].clone();
                let vars = instruction.kind.collectVariables();
                let mut instructionIndex = index;
                for var in vars {
                    if self.implicitRefs.contains(&var) {
                        let mut dest = var.clone();
                        dest.value = format!("implicitRef{}", nextImplicitRef);
                        nextImplicitRef += 1;
                        let ty = Type::Reference(Box::new(self.getType(&var)), None);
                        self.types.insert(dest.value.clone(), ty);
                        let mut varSwap = VariableSubstitution::new();
                        varSwap.add(var.clone(), dest.clone());
                        let kind = InstructionKind::Ref(dest.clone(), var.clone());
                        let implicitRef = Instruction {
                            implicit: true,
                            kind: kind,
                            location: instruction.location.clone(),
                        };
                        instruction.kind = instruction.kind.applyVar(&varSwap);
                        block.instructions.insert(index, implicitRef);
                        instructionIndex += 1;
                        self.implicitRefs.remove(&var);
                    }
                }
                block.instructions[instructionIndex] = instruction;
                index += 1;
            }
        }
    }

    fn transformMutableMethodCalls(&mut self, f: &mut Function) {
        let mut nextImplicitResult = 0;

        let body = &mut f.body.as_mut().unwrap();

        for block in &mut body.blocks {
            let mut index = 0;
            loop {
                if index >= block.instructions.len() {
                    break;
                }
                let mut instruction = block.instructions[index].clone();
                let vars = instruction.kind.collectVariables();
                for var in vars {
                    if let Some(info) = self.mutableMethodCalls.get(&var) {
                        let mut dest = var.clone();
                        dest.value = format!("implicitResult{}", nextImplicitResult);
                        nextImplicitResult += 1;
                        self.types.insert(dest.value.clone(), info.baseType.clone());
                        let mut varSwap = VariableSubstitution::new();
                        varSwap.add(var.clone(), dest.clone());
                        let tupleTypes = info.selfLessType.getTupleTypes();
                        let mut currentIndex = index + 1;
                        match tupleTypes.len() {
                            0 => {
                                let kind = InstructionKind::Assign(info.receiver.clone(), dest.clone());
                                let selfIndex = Instruction {
                                    implicit: true,
                                    kind: kind,
                                    location: instruction.location.clone(),
                                };
                                block.instructions.insert(currentIndex, selfIndex);
                                let kind = InstructionKind::Tuple(var.clone(), Vec::new());
                                let assign = Instruction {
                                    implicit: true,
                                    kind: kind,
                                    location: instruction.location.clone(),
                                };
                                block.instructions.insert(index + 1, assign);
                            }
                            1 => {
                                let kind = InstructionKind::TupleIndex(info.receiver.clone(), dest.clone(), 0);
                                let selfIndex = Instruction {
                                    implicit: true,
                                    kind: kind,
                                    location: instruction.location.clone(),
                                };
                                block.instructions.insert(currentIndex, selfIndex);
                                let mut argVar = var.clone();
                                argVar.value = format!("argVar{}", nextImplicitResult);
                                nextImplicitResult += 1;
                                self.types.insert(argVar.value.clone(), tupleTypes[0].clone());
                                let kind = InstructionKind::TupleIndex(argVar.clone(), dest.clone(), 1);
                                let argIndex = Instruction {
                                    implicit: true,
                                    kind: kind,
                                    location: instruction.location.clone(),
                                };
                                block.instructions.insert(currentIndex, argIndex);
                                currentIndex += 1;
                                let kind = InstructionKind::Assign(var.clone(), argVar.clone());
                                let assign = Instruction {
                                    implicit: true,
                                    kind: kind,
                                    location: instruction.location.clone(),
                                };
                                block.instructions.insert(currentIndex, assign);
                            }
                            _ => {
                                let kind = InstructionKind::TupleIndex(info.receiver.clone(), dest.clone(), 0);
                                let selfIndex = Instruction {
                                    implicit: true,
                                    kind: kind,
                                    location: instruction.location.clone(),
                                };
                                block.instructions.insert(currentIndex, selfIndex);
                                let mut args = Vec::new();
                                for (argIndex, argType) in tupleTypes.iter().enumerate() {
                                    let mut argVar = var.clone();
                                    argVar.value = format!("argVar{}", nextImplicitResult);
                                    args.push(argVar.clone());
                                    nextImplicitResult += 1;
                                    self.types.insert(argVar.value.clone(), argType.clone());
                                    let kind = InstructionKind::TupleIndex(argVar.clone(), dest.clone(), (argIndex + 1) as i32);
                                    let tupleIndex = Instruction {
                                        implicit: true,
                                        kind: kind,
                                        location: instruction.location.clone(),
                                    };
                                    block.instructions.insert(currentIndex, tupleIndex);
                                    currentIndex += 1;
                                }
                                let kind = InstructionKind::Tuple(var.clone(), args);
                                let tuple = Instruction {
                                    implicit: true,
                                    kind: kind,
                                    location: instruction.location.clone(),
                                };
                                block.instructions.insert(currentIndex, tuple);
                            }
                        }
                        instruction.kind = instruction.kind.applyVar(&varSwap);
                        self.mutableMethodCalls.remove(&var);
                        break;
                    }
                }
                block.instructions[index] = instruction;
                index += 1;
            }
        }
    }

    fn addFieldTypes(&mut self, f: &mut Function) {
        let body = &mut f.body.as_mut().unwrap();

        for block in &mut body.blocks {
            for instruction in &mut block.instructions {
                if let InstructionKind::FieldAssign(dest, _, fields) = &mut instruction.kind {
                    let types = self.fieldTypes.get(&dest).expect("field types are missing");
                    for (index, ty) in types.iter().enumerate() {
                        fields[index].ty = Some(ty.apply(&self.substitution));
                    }
                }
            }
        }
    }

    fn addTypes(&mut self, f: &mut Function) {
        let mut varSwap = VariableSubstitution::new();

        let body = &mut f.body.as_mut().unwrap();

        for block in &mut body.blocks {
            for instruction in &mut block.instructions {
                for var in instruction.kind.collectVariables() {
                    let ty = self.getType(&var);
                    let ty = ty.apply(&self.substitution);
                    let mut newVar = var.clone();
                    newVar.ty = Some(ty.clone());
                    if newVar != var {
                        varSwap.add(var, newVar);
                    }
                }
            }
        }

        for block in &mut body.blocks {
            for instruction in &mut block.instructions {
                instruction.kind = instruction.kind.applyVar(&varSwap);
            }
        }
    }

    pub fn generate(&mut self, f: &Function) -> Function {
        //println!("Generating {}", f.name);
        if f.body.is_none() {
            return f.clone();
        }

        let mut result = f.clone();
        if let Some(selfType) = self.selfType.clone() {
            result.result = result.result.changeSelfType(selfType);
        }

        self.convertMethodCalls(&mut result);
        self.addImplicitRefs(&mut result);
        self.transformMutableMethodCalls(&mut result);
        self.addFieldTypes(&mut result);
        self.addTypes(&mut result);

        //self.dump(&result);
        self.verify(&result);
        result
    }
}
