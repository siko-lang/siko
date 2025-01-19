use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    iter::zip,
};

use crate::siko::{
    hir::{
        Apply::{instantiateClass, instantiateEnum, instantiateTrait, instantiateType4, Apply, ApplyVariable},
        BlockBuilder::BlockBuilder,
        BodyBuilder::BodyBuilder,
        ConstraintContext::Constraint as HirConstraint,
        ConstraintContext::ConstraintContext,
        Data::{Class, Enum},
        Function::{BlockId, Function, Parameter, Variable, VariableName},
        InstanceResolver::ResolutionResult,
        Instruction::{Instruction, InstructionKind, Tag},
        Program::Program,
        Substitution::{TypeSubstitution, VariableSubstitution},
        TraitMethodSelector::TraitMethodSelector,
        Type::{formatTypes, Type, TypeVar},
        TypeVarAllocator::TypeVarAllocator,
        Unification::unify,
    },
    location::{Location::Location, Report::ReportContext},
    qualifiedname::{getCloneFnName, getImplicitConvertFnName, getNativePtrCloneName, QualifiedName},
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

enum MarkerInfo {
    ImplicitRef(Variable),
    ImplicitClone(Variable),
}

enum ImplicitConvertInfo {
    Simple(Type),
    Ref(Type),
}

pub struct Typechecker<'a> {
    ctx: &'a ReportContext,
    program: &'a Program,
    f: &'a Function,
    traitMethodSelector: &'a TraitMethodSelector,
    allocator: TypeVarAllocator,
    substitution: TypeSubstitution,
    types: BTreeMap<String, Type>,
    selfType: Option<Type>,
    mutables: BTreeSet<String>,
    implicitConverts: BTreeMap<Variable, ImplicitConvertInfo>,
    mutableMethodCalls: BTreeMap<Variable, MutableMethodCallInfo>,
    fieldTypes: BTreeMap<Variable, Vec<Type>>,
    bodyBuilder: BodyBuilder,
    visitedBlocks: BTreeSet<BlockId>,
    queue: VecDeque<BlockId>,
    markers: BTreeMap<Tag, MarkerInfo>,
    knownConstraints: ConstraintContext,
}

impl<'a> Typechecker<'a> {
    pub fn new(
        ctx: &'a ReportContext,
        program: &'a Program,
        traitMethodSelector: &'a TraitMethodSelector,
        f: &'a Function,
    ) -> Typechecker<'a> {
        Typechecker {
            ctx: ctx,
            program: program,
            f: f,
            traitMethodSelector: traitMethodSelector,
            allocator: TypeVarAllocator::new(),
            substitution: TypeSubstitution::new(),
            types: BTreeMap::new(),
            selfType: None,
            mutables: BTreeSet::new(),
            implicitConverts: BTreeMap::new(),
            mutableMethodCalls: BTreeMap::new(),
            fieldTypes: BTreeMap::new(),
            bodyBuilder: BodyBuilder::cloneFunction(f),
            visitedBlocks: BTreeSet::new(),
            queue: VecDeque::new(),
            markers: BTreeMap::new(),
            knownConstraints: f.constraintContext.clone(),
        }
    }

    pub fn run(&mut self) -> Function {
        self.initialize();
        self.expandKnownConstraints();
        //self.dump(self.f);
        self.check();
        //self.dump(self.f);
        self.generate()
    }

    fn addImplicitRefMarker(&mut self, var: &Variable) -> Tag {
        let tag = self.bodyBuilder.buildTag(Tag::ImplicitRef);
        self.markers.insert(tag, MarkerInfo::ImplicitRef(var.clone()));
        tag
    }

    fn addImplicitCloneMarker(&mut self, var: &Variable) -> Tag {
        let tag = self.bodyBuilder.buildTag(Tag::ImplicitClone);
        self.markers.insert(tag, MarkerInfo::ImplicitClone(var.clone()));
        tag
    }

    fn initializeVar(&mut self, var: &Variable) {
        match &var.ty {
            Some(ty) => {
                self.types.insert(var.value.to_string(), ty.clone());
            }
            None => {
                if let Some(ty) = self.bodyBuilder.getTypeInBody(&var) {
                    self.types.insert(var.value.to_string(), ty.clone());
                } else {
                    let ty = self.allocator.next();
                    self.types.insert(var.value.to_string(), ty.clone());
                }
            }
        }
    }

    pub fn initialize(&mut self) {
        //println!("Initializing {}", self.f.name);
        for param in &self.f.params {
            match &param {
                Parameter::Named(name, ty, mutable) => {
                    self.types.insert(format!("{}", name), ty.clone());
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
        if let Some(body) = &self.f.body {
            for block in &body.blocks {
                for instruction in &block.instructions {
                    match &instruction.kind {
                        InstructionKind::FunctionCall(var, _, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::MethodCall(var, _, _, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::DynamicFunctionCall(var, _, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::ValueRef(var, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::FieldRef(var, _, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::TupleIndex(var, _, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::Bind(var, _, mutable) => {
                            self.initializeVar(var);
                            if *mutable {
                                self.mutables.insert(var.value.to_string());
                            }
                        }
                        InstructionKind::Tuple(var, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::StringLiteral(var, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::IntegerLiteral(var, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::CharLiteral(var, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::Return(var, _) => {
                            self.types.insert(var.value.to_string(), Type::Never(false));
                        }
                        InstructionKind::Ref(var, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::Drop(_, _) => {}
                        InstructionKind::Jump(var, _, _) => {
                            self.types.insert(var.value.to_string(), Type::Never(false));
                        }
                        InstructionKind::Assign(_, _) => {}
                        InstructionKind::FieldAssign(_, _, _) => {}
                        InstructionKind::DeclareVar(var) => {
                            self.initializeVar(var);
                            self.mutables.insert(var.value.to_string());
                        }
                        InstructionKind::Transform(var, _, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::EnumSwitch(_, _) => {}
                        InstructionKind::IntegerSwitch(_, _) => {}
                        InstructionKind::StringSwitch(_, _) => {}
                        InstructionKind::BlockStart(_) => {}
                        InstructionKind::BlockEnd(_) => {}
                    }
                }
            }
        }
    }

    fn getType(&self, var: &Variable) -> Type {
        match self.types.get(&var.value.to_string()) {
            Some(ty) => ty.clone(),
            None => panic!("No type found for {}!", var),
        }
    }

    fn unify(&mut self, ty1: Type, ty2: Type, location: Location) {
        //println!("UNIFY {} {}", ty1, ty2);
        if let Err(_) = unify(&mut self.substitution, &ty1, &ty2, false) {
            reportError(
                self.ctx,
                ty1.apply(&self.substitution),
                ty2.apply(&self.substitution),
                location,
            );
        }
    }

    fn instantiateEnum(&mut self, e: &Enum, ty: &Type) -> Enum {
        instantiateEnum(&mut self.allocator, e, ty)
    }

    fn instantiateClass(&mut self, c: &Class, ty: &Type) -> Class {
        instantiateClass(&mut self.allocator, c, ty)
    }

    fn handleImplicits(&mut self, input: &Variable, output: &Type, builder: &mut BlockBuilder) -> Type {
        let mut inputTy = self.getType(input).apply(&self.substitution);
        if inputTy.isReference() && !output.isReference() && !output.isGeneric() {
            if self.program.instanceResolver.isCopy(&output) {
                inputTy = inputTy.unpackRef().clone();
                //println!("IMPLICIT CLONE FOR {} {} {}", arg, argTy, fnArg);
                let tag = self.addImplicitCloneMarker(input);
                builder.addTag(tag);
            }
        }
        if inputTy.isReference()
            && output.isPtr()
            && ((inputTy.unpackRef() == output) || output.unpackPtr().isGeneric())
        {
            inputTy = inputTy.unpackRef().clone();
            //println!("IMPLICIT CLONE FOR {} {} {}", arg, argTy, fnArg);
            let tag = self.addImplicitCloneMarker(input);
            builder.addTag(tag);
        }
        if !inputTy.isGeneric() && !output.isGeneric() && inputTy != *output {
            if self.program.instanceResolver.isImplicitConvert(&inputTy, &output) {
                inputTy = output.clone();
                //println!("IMPLICIT CONVERT FOR {} {} {}", input, inputTy, output);
                self.implicitConverts
                    .insert(input.clone(), ImplicitConvertInfo::Simple(output.clone()));
            }
        }
        inputTy
    }

    fn checkFunctionCall(
        &mut self,
        args: &Vec<Variable>,
        resultVar: &Variable,
        fnType: Type,
        neededConstraints: &ConstraintContext,
        builder: &mut BlockBuilder,
    ) -> Type {
        // println!(
        //     "checkFunctionCall: {} {} {}",
        //     fnType, neededConstraints, knownConstraints
        // );
        let mut contextArgs = BTreeSet::new();
        for arg in &neededConstraints.typeParameters {
            contextArgs = arg.collectVars(contextArgs);
        }
        let mut types = neededConstraints.typeParameters.clone();
        types.push(fnType.clone());
        let sub = instantiateType4(&mut self.allocator, &types);
        let fnType = fnType.apply(&sub);
        //println!("inst {}", fnType);
        let constraintContext = neededConstraints.apply(&sub);
        let (fnArgs, mut fnResult) = match fnType.clone().splitFnType() {
            Some((fnArgs, fnResult)) => (fnArgs, fnResult),
            None => return fnType,
        };
        if args.len() != fnArgs.len() {
            TypecheckerError::ArgCountMismatch(fnArgs.len() as u32, args.len() as u32, resultVar.location.clone())
                .report(self.ctx);
        }
        if fnArgs.len() > 0 {
            fnResult = fnResult.changeSelfType(fnArgs[0].clone());
        }
        for (arg, fnArg) in zip(args, fnArgs) {
            let fnArg = fnArg.apply(&self.substitution);
            let mut argTy = self.handleImplicits(arg, &fnArg, builder);
            //println!("ARG {} {} {}", arg, argTy, fnArg);
            if !argTy.isReference() && fnArg.isReference() {
                let targetTy = fnArg.unpackRef().clone();
                if targetTy != argTy && self.program.instanceResolver.isImplicitConvert(&argTy, &targetTy) {
                    //println!("IMPLICIT CONVERT REF FOR {} {} {}", arg, argTy, targetTy);
                    argTy = Type::Reference(Box::new(targetTy.clone()), None);
                    self.implicitConverts
                        .insert(arg.clone(), ImplicitConvertInfo::Ref(targetTy));
                } else {
                    argTy = Type::Reference(Box::new(argTy), None);
                    //println!("IMPLICIT REF FOR {}", arg);
                    let tag = self.addImplicitRefMarker(arg);
                    builder.addTag(tag);
                }
            }
            self.unify(argTy, fnArg, arg.location.clone());
        }
        let constraints = constraintContext.apply(&self.substitution);
        self.checkConstraint(&constraints, resultVar.location.clone());
        //println!("fnResult {}", fnResult);
        //println!("self.getType(resultVar) {}", self.getType(resultVar));
        let fnResult = fnResult.apply(&self.substitution);
        self.unify(self.getType(resultVar), fnResult, resultVar.location.clone());
        // let mut argTy = self.getType(resultVar);
        // argTy = argTy.apply(&self.substitution);
        // println!("ffff result {}", argTy);
        fnType.apply(&self.substitution)
    }

    fn checkConstraint(&mut self, neededConstraints: &ConstraintContext, location: Location) {
        //println!("needed {}", neededConstraints);
        //println!("known {}", self.knownConstraints);
        for c in &neededConstraints.constraints {
            //println!("knownConstraints.contains(c) {}", self.knownConstraints.contains(c));
            if !self.knownConstraints.contains(c) {
                if let Some(instances) = self.program.instanceResolver.lookupInstances(&c.traitName) {
                    //println!("c.args {}", formatTypes(&c.args));
                    let mut fullyGeneric = true;
                    for arg in &c.args {
                        if !arg.isTypeVar() {
                            fullyGeneric = false;
                            break;
                        }
                    }
                    if fullyGeneric {
                        continue;
                    }
                    let resolutionResult = instances.find(&mut self.allocator, &c.args);
                    match resolutionResult {
                        ResolutionResult::Winner(instance) => {
                            //println!("Base Winner {} for {}", instance, formatTypes(&c.args));
                            let instance = instance.apply(&self.substitution);
                            //println!("Winner {} for {}", instance, formatTypes(&c.args));
                            for ctxAssocTy in &c.associatedTypes {
                                for instanceAssocTy in &instance.associatedTypes {
                                    if instanceAssocTy.name == ctxAssocTy.name {
                                        //println!("ASSOC MATCH {} {}", instanceAssocTy.ty, ctxAssocTy.ty);
                                        if let Err(_) =
                                            unify(&mut self.substitution, &instanceAssocTy.ty, &ctxAssocTy.ty, false)
                                        {
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
                            TypecheckerError::AmbiguousInstances(
                                c.traitName.toString(),
                                formatTypes(&c.args),
                                location.clone(),
                                Vec::new(),
                            )
                            .report(self.ctx);
                        }
                        ResolutionResult::NoInstanceFound => {
                            TypecheckerError::InstanceNotFound(
                                c.traitName.toString(),
                                formatTypes(&c.args),
                                location.clone(),
                            )
                            .report(self.ctx);
                        }
                    }
                } else {
                    TypecheckerError::InstanceNotFound(c.traitName.toString(), formatTypes(&c.args), location.clone())
                        .report(self.ctx);
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
                            if receiverType.isReference() {
                                return Type::Reference(Box::new(f.ty.clone()), None);
                            }
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

    fn checkBlock(&mut self, blockId: BlockId) {
        let mut builder = self.bodyBuilder.iterator(blockId);
        loop {
            match builder.getInstruction() {
                Some(instruction) => {
                    self.checkInstruction(instruction, &mut builder);
                    builder.step();
                }
                None => {
                    break;
                }
            }
        }
    }

    fn checkInstruction(&mut self, instruction: Instruction, builder: &mut BlockBuilder) {
        //println!("checkInstruction {}", instruction);
        match &instruction.kind {
            InstructionKind::FunctionCall(dest, name, args) => {
                let Some(targetFn) = self.program.functions.get(name) else {
                    panic!("Function not found {}", name);
                };
                let fnType = targetFn.getType();
                self.checkFunctionCall(args, dest, fnType, &targetFn.constraintContext, builder);
            }
            InstructionKind::MethodCall(dest, receiver, methodName, args) => {
                let receiverType = self.getType(receiver);
                let receiverType = receiverType.apply(&self.substitution);
                //println!("METHOD {} {} {}", methodName, receiver, receiverType);
                let name = self.lookupMethod(receiverType.clone(), methodName, instruction.location.clone());
                let mut args = args.clone();
                args.insert(0, receiver.clone());
                builder.replaceInstruction(
                    InstructionKind::FunctionCall(dest.clone(), name.clone(), args.clone()),
                    instruction.location.clone(),
                );
                let targetFn = self.program.functions.get(&name).expect("Function not found");
                let mut fnType = targetFn.getType();

                let mutableCall =
                    self.mutables.contains(&receiver.value.to_string()) && fnType.getResult().hasSelfType();
                if mutableCall {
                    fnType = fnType.changeMethodResult();
                }
                let fnType = self.checkFunctionCall(&args, dest, fnType, &targetFn.constraintContext, builder);
                if mutableCall {
                    let result = fnType.getResult();
                    let (baseType, selfLessType) = if targetFn.getType().getResult().isTuple() {
                        let baseType = result.addSelfType(receiverType);
                        let selfLessType = baseType.getSelflessType(false);
                        (baseType, selfLessType)
                    } else {
                        (receiverType, Type::Tuple(Vec::new()))
                    };
                    //println!("MUT METHOD {} => {}", fnType, selfLessType);
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
            }
            InstructionKind::DynamicFunctionCall(dest, callable, args) => {
                let fnType = self.getType(callable);
                self.checkFunctionCall(&args, dest, fnType, &ConstraintContext::new(), builder);
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
                self.unify(
                    self.getType(dest),
                    Type::getStringLiteralType(),
                    instruction.location.clone(),
                );
            }
            InstructionKind::IntegerLiteral(dest, _) => {
                self.unify(self.getType(dest), Type::getIntType(), instruction.location.clone());
            }
            InstructionKind::CharLiteral(dest, _) => {
                self.unify(self.getType(dest), Type::getCharType(), instruction.location.clone());
            }
            InstructionKind::Return(_, arg) => {
                let mut result = self.f.result.clone();
                if let Some(selfType) = self.selfType.clone() {
                    result = result.changeSelfType(selfType);
                }
                let argTy = self.handleImplicits(arg, &result, builder);
                self.unify(result, argTy, instruction.location.clone());
            }
            InstructionKind::Ref(dest, arg) => {
                let arg_type = self.getType(arg);
                self.unify(
                    self.getType(dest),
                    Type::Reference(Box::new(arg_type), None),
                    instruction.location.clone(),
                );
            }
            InstructionKind::Drop(_, _) => unreachable!("drop in typechecker!"),
            InstructionKind::Jump(_, id, _) => {
                self.queue.push_back(*id);
            }
            InstructionKind::Assign(name, rhs) => {
                if !self.mutables.contains(&name.value.to_string()) {
                    TypecheckerError::ImmutableAssign(instruction.location.clone()).report(self.ctx);
                }
                self.unify(self.getType(name), self.getType(rhs), instruction.location.clone());
            }
            InstructionKind::FieldAssign(name, rhs, fields) => {
                if !self.mutables.contains(&name.value.to_string()) {
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
                match rootTy.unpackRef().getName() {
                    Some(name) => {
                        let e = self.program.enums.get(&name).expect("not an enum in transform!");
                        let e = self.instantiateEnum(e, &rootTy.unpackRef());
                        let v = &e.variants[*index as usize];
                        self.unify(
                            self.getType(dest),
                            Type::Tuple(v.items.clone()),
                            instruction.location.clone(),
                        );
                    }
                    None => {
                        TypecheckerError::TypeAnnotationNeeded(instruction.location.clone()).report(self.ctx);
                    }
                };
            }
            InstructionKind::EnumSwitch(_, cases) => {
                for case in cases {
                    self.queue.push_back(case.branch);
                }
            }
            InstructionKind::IntegerSwitch(_, cases) => {
                for case in cases {
                    self.queue.push_back(case.branch);
                }
            }
            InstructionKind::StringSwitch(_, cases) => {
                for case in cases {
                    self.queue.push_back(case.branch);
                }
            }
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
                            TypecheckerError::FieldNotFound(format!(".{}", index), instruction.location.clone())
                                .report(&self.ctx);
                        }
                        let fieldType = t[*index as usize].clone();
                        self.unify(self.getType(dest), fieldType, instruction.location.clone());
                    }
                    _ => TypecheckerError::TypeAnnotationNeeded(instruction.location.clone()).report(self.ctx),
                }
            }
            InstructionKind::BlockStart(_) => {}
            InstructionKind::BlockEnd(_) => {}
        }
    }

    fn check(&mut self) {
        //println!("checking {}", f.name);
        if self.f.body.is_none() {
            return;
        };
        // the double loop is needed to reach even the unreachable blocks
        let mut allblocksIds = self.bodyBuilder.getAllBlockIds();
        loop {
            if let Some(blockId) = allblocksIds.pop_front() {
                self.queue.push_back(blockId);
                loop {
                    if let Some(blockId) = self.queue.pop_front() {
                        if self.visitedBlocks.contains(&blockId) {
                            continue;
                        }
                        self.visitedBlocks.insert(blockId);
                        self.checkBlock(blockId);
                    } else {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    pub fn verify(&self, f: &Function) {
        if let Some(body) = &f.body {
            let fnType = f.getType();
            let publicVars = fnType.collectVars(BTreeSet::new());
            for block in &body.blocks {
                for instruction in &block.instructions {
                    let vars = instruction.kind.collectVariables();
                    for v in vars {
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

    fn addImplicitConverts(&mut self, f: &mut Function) {
        let mut nextImplicitConvert = 0;

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
                    if let Some(info) = self.implicitConverts.get(&var) {
                        match info {
                            ImplicitConvertInfo::Simple(targetTy) => {
                                let mut dest = var.clone();
                                dest.value = VariableName::Local(format!("implicitConvert"), nextImplicitConvert);
                                nextImplicitConvert += 1;
                                self.types.insert(dest.value.to_string(), targetTy.clone());
                                let mut varSwap = VariableSubstitution::new();
                                varSwap.add(var.clone(), dest.clone());
                                let kind = InstructionKind::FunctionCall(
                                    dest.clone(),
                                    getImplicitConvertFnName(),
                                    vec![var.clone()],
                                );
                                let implicitRef = Instruction {
                                    implicit: true,
                                    kind: kind,
                                    location: instruction.location.clone(),
                                    tags: Vec::new(),
                                };
                                instruction.kind = instruction.kind.applyVar(&varSwap);
                                block.instructions.insert(index, implicitRef);
                                instructionIndex += 1;
                                self.implicitConverts.remove(&var);
                            }
                            ImplicitConvertInfo::Ref(targetTy) => {
                                //println!("IMPLICIT CONVERT REF FOR {} {}", var, targetTy);
                                let mut implicitConvertDest = var.clone();
                                implicitConvertDest.value =
                                    VariableName::Local(format!("implicitConvert"), nextImplicitConvert);
                                nextImplicitConvert += 1;
                                self.types
                                    .insert(implicitConvertDest.value.to_string(), targetTy.clone());

                                let mut implicitRefDest = var.clone();
                                implicitRefDest.value =
                                    VariableName::Local(format!("implicitRef"), nextImplicitConvert);
                                nextImplicitConvert += 1;
                                self.types.insert(
                                    implicitRefDest.value.to_string(),
                                    Type::Reference(Box::new(targetTy.clone()), None),
                                );

                                let mut varSwap = VariableSubstitution::new();
                                varSwap.add(var.clone(), implicitRefDest.clone());
                                let kind = InstructionKind::FunctionCall(
                                    implicitConvertDest.clone(),
                                    getImplicitConvertFnName(),
                                    vec![var.clone()],
                                );
                                let implicitConvert = Instruction {
                                    implicit: true,
                                    kind: kind,
                                    location: instruction.location.clone(),
                                    tags: Vec::new(),
                                };
                                let kind = InstructionKind::Ref(implicitRefDest.clone(), implicitConvertDest.clone());
                                let implicitRef = Instruction {
                                    implicit: true,
                                    kind: kind,
                                    location: instruction.location.clone(),
                                    tags: Vec::new(),
                                };
                                instruction.kind = instruction.kind.applyVar(&varSwap);
                                block.instructions.insert(index, implicitRef);
                                block.instructions.insert(index, implicitConvert);
                                instructionIndex += 1;
                                instructionIndex += 1;
                                self.implicitConverts.remove(&var);
                            }
                        }
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
                        dest.value = VariableName::Local(format!("implicitResult"), nextImplicitResult);
                        nextImplicitResult += 1;
                        self.types.insert(dest.value.to_string(), info.baseType.clone());
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
                                    tags: Vec::new(),
                                };
                                block.instructions.insert(currentIndex, selfIndex);
                                let kind = InstructionKind::Tuple(var.clone(), Vec::new());
                                let assign = Instruction {
                                    implicit: true,
                                    kind: kind,
                                    location: instruction.location.clone(),
                                    tags: Vec::new(),
                                };
                                block.instructions.insert(index + 1, assign);
                            }
                            1 => {
                                let kind = InstructionKind::TupleIndex(info.receiver.clone(), dest.clone(), 0);
                                let selfIndex = Instruction {
                                    implicit: true,
                                    kind: kind,
                                    location: instruction.location.clone(),
                                    tags: Vec::new(),
                                };
                                block.instructions.insert(currentIndex, selfIndex);
                                let mut argVar = var.clone();
                                argVar.value = VariableName::Local(format!("argVar"), nextImplicitResult);
                                nextImplicitResult += 1;
                                self.types.insert(argVar.value.to_string(), tupleTypes[0].clone());
                                let kind = InstructionKind::TupleIndex(argVar.clone(), dest.clone(), 1);
                                let argIndex = Instruction {
                                    implicit: true,
                                    kind: kind,
                                    location: instruction.location.clone(),
                                    tags: Vec::new(),
                                };
                                block.instructions.insert(currentIndex, argIndex);
                                currentIndex += 1;
                                let kind = InstructionKind::Assign(var.clone(), argVar.clone());
                                let assign = Instruction {
                                    implicit: true,
                                    kind: kind,
                                    location: instruction.location.clone(),
                                    tags: Vec::new(),
                                };
                                block.instructions.insert(currentIndex, assign);
                            }
                            _ => {
                                let kind = InstructionKind::TupleIndex(info.receiver.clone(), dest.clone(), 0);
                                let selfIndex = Instruction {
                                    implicit: true,
                                    kind: kind,
                                    location: instruction.location.clone(),
                                    tags: Vec::new(),
                                };
                                block.instructions.insert(currentIndex, selfIndex);
                                let mut args = Vec::new();
                                for (argIndex, argType) in tupleTypes.iter().enumerate() {
                                    let mut argVar = var.clone();
                                    argVar.value = VariableName::Local(format!("argVar"), nextImplicitResult);
                                    args.push(argVar.clone());
                                    nextImplicitResult += 1;
                                    self.types.insert(argVar.value.to_string(), argType.clone());
                                    let kind = InstructionKind::TupleIndex(
                                        argVar.clone(),
                                        dest.clone(),
                                        (argIndex + 1) as i32,
                                    );
                                    let tupleIndex = Instruction {
                                        implicit: true,
                                        kind: kind,
                                        location: instruction.location.clone(),
                                        tags: Vec::new(),
                                    };
                                    block.instructions.insert(currentIndex, tupleIndex);
                                    currentIndex += 1;
                                }
                                let kind = InstructionKind::Tuple(var.clone(), args);
                                let tuple = Instruction {
                                    implicit: true,
                                    kind: kind,
                                    location: instruction.location.clone(),
                                    tags: Vec::new(),
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

    fn addFieldTypes(&mut self) {
        // same as addFieldTypes but uses the bodybuilder api
        let allblocksIds = self.bodyBuilder.getAllBlockIds();
        for blockId in allblocksIds {
            let mut builder = self.bodyBuilder.iterator(blockId);
            loop {
                match builder.getInstruction() {
                    Some(instruction) => {
                        if let InstructionKind::FieldAssign(dest, root, fields) = &instruction.kind {
                            let mut fields = fields.clone();
                            let types = self.fieldTypes.get(&dest).expect("field types are missing");
                            for (index, ty) in types.iter().enumerate() {
                                fields[index].ty = Some(ty.apply(&self.substitution));
                            }
                            let kind = InstructionKind::FieldAssign(dest.clone(), root.clone(), fields);

                            builder.replaceInstruction(kind, instruction.location.clone());
                        }
                        builder.step();
                    }
                    None => {
                        break;
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
                let vars = instruction.kind.collectVariables();
                for var in vars {
                    let ty = self.getType(&var);
                    let ty = ty.apply(&self.substitution);
                    let mut newVar = var.clone();
                    newVar.ty = Some(ty.clone());
                    varSwap.add(var, newVar);
                }
            }
        }

        for block in &mut body.blocks {
            for instruction in &mut block.instructions {
                instruction.kind = instruction.kind.applyVar(&varSwap);
            }
        }
    }

    pub fn addImplicitRefs(&mut self) {
        let allblocksIds = self.bodyBuilder.getAllBlockIds();
        for blockId in allblocksIds {
            let mut builder = self.bodyBuilder.iterator(blockId);
            loop {
                match builder.getInstruction() {
                    Some(instruction) => {
                        //println!("instruction {} at {} {:?}", instruction, blockId, instruction.tags);
                        if instruction.tags.is_empty() {
                            builder.step();
                            continue;
                        }
                        let mut sub = VariableSubstitution::new();
                        for tag in &instruction.tags {
                            if let Some(MarkerInfo::ImplicitRef(var)) = self.markers.get(tag) {
                                //println!("IMPLICIT REF FOR {}", var);
                                let mut dest = var.clone();
                                dest.value = VariableName::ImplicitRef(tag.getId());
                                let ty = Type::Reference(Box::new(self.getType(var)), None);
                                self.types.insert(dest.value.to_string(), ty);
                                let kind = InstructionKind::Ref(dest.clone(), var.clone());
                                sub.add(var.clone(), dest.clone());
                                builder.addInstruction(kind, instruction.location.clone());
                                builder.step();
                            }
                        }
                        builder.replaceInstruction(instruction.kind.applyVar(&sub), instruction.location.clone());
                        builder.step();
                    }
                    None => {
                        break;
                    }
                }
            }
        }
    }

    pub fn addImplicitClones(&mut self) {
        let allblocksIds = self.bodyBuilder.getAllBlockIds();
        for blockId in allblocksIds {
            let mut builder = self.bodyBuilder.iterator(blockId);
            loop {
                match builder.getInstruction() {
                    Some(instruction) => {
                        if instruction.tags.is_empty() {
                            builder.step();
                            continue;
                        }
                        let mut sub = VariableSubstitution::new();
                        for tag in &instruction.tags {
                            if let Some(MarkerInfo::ImplicitClone(var)) = self.markers.get(tag) {
                                let mut dest = var.clone();
                                dest.value = VariableName::ImplicitClone(tag.getId());
                                let ty = self.getType(&var).apply(&self.substitution).unpackRef().clone();
                                let kind = if ty.isPtr() {
                                    InstructionKind::FunctionCall(
                                        dest.clone(),
                                        getNativePtrCloneName(),
                                        vec![var.clone()],
                                    )
                                } else {
                                    InstructionKind::FunctionCall(dest.clone(), getCloneFnName(), vec![var.clone()])
                                };
                                self.types.insert(dest.value.to_string(), ty);
                                sub.add(var.clone(), dest.clone());
                                builder.addInstruction(kind, instruction.location.clone());
                                builder.step();
                            }
                        }
                        builder.replaceInstruction(instruction.kind.applyVar(&sub), instruction.location.clone());
                        builder.step();
                    }
                    None => {
                        break;
                    }
                }
            }
        }
    }

    pub fn generate(&mut self) -> Function {
        //println!("Generating {}", self.f.name);
        if self.f.body.is_none() {
            return self.f.clone();
        }

        self.addImplicitRefs();
        self.addImplicitClones();
        self.addFieldTypes();

        let mut result = self.f.clone();
        result.body = Some(self.bodyBuilder.build());
        if let Some(selfType) = self.selfType.clone() {
            result.result = result.result.changeSelfType(selfType);
        }

        self.addImplicitConverts(&mut result);
        self.transformMutableMethodCalls(&mut result);
        self.addTypes(&mut result);

        //self.dump(&result);
        self.verify(&result);
        result
    }

    fn expandKnownConstraints(&mut self) {
        //println!("expandKnownConstraints {}", self.f.name);
        let mut processed = Vec::new();
        let start = self.knownConstraints.constraints.clone();
        for c in &start {
            self.expandKnownConstraint(c, &mut processed);
        }
    }

    fn expandKnownConstraint(&mut self, c: &HirConstraint, processed: &mut Vec<HirConstraint>) {
        if processed.contains(c) {
            return;
        }
        //println!("expandKnownConstraint {}", c);
        processed.push(c.clone());
        let traitDef = self.program.getTrait(&c.traitName).expect("Trait not found");
        let traitDef = instantiateTrait(&mut self.allocator, &traitDef);
        let mut sub = TypeSubstitution::new();
        for (arg, ctxArg) in zip(&traitDef.params, &c.args) {
            sub.add(arg.clone(), ctxArg.clone());
        }
        let traitDef = traitDef.apply(&sub);
        self.knownConstraints.constraints.push(c.clone());
        for c in traitDef.constraint.constraints {
            self.expandKnownConstraint(&c, processed);
        }
    }
}
