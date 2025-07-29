use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    iter::zip,
};

use crate::siko::{
    hir::{
        Apply::{instantiateEnum, instantiateStruct, instantiateTrait, instantiateType4, Apply},
        BlockBuilder::BlockBuilder,
        BodyBuilder::BodyBuilder,
        ConstraintContext::{Constraint as HirConstraint, ConstraintContext},
        Data::{Enum, Struct},
        Function::{BlockId, Function, Parameter},
        InstanceResolver::ResolutionResult,
        Instruction::{Instruction, InstructionKind, Mutability},
        Program::Program,
        Substitution::Substitution,
        TraitMethodSelector::TraitMethodSelector,
        Type::{formatTypes, Type, TypeVar},
        TypeVarAllocator::TypeVarAllocator,
        Unification::unify,
        Variable::{Variable, VariableName},
    },
    location::{Location::Location, Report::ReportContext},
    qualifiedname::{getCloneFnName, getDerefGetName, getImplicitConvertFnName, getNativePtrCloneName, QualifiedName},
};

use super::Error::TypecheckerError;

fn reportError(ctx: &ReportContext, ty1: Type, ty2: Type, location: Location) {
    TypecheckerError::TypeMismatch(format!("{}", ty1), format!("{}", ty2), location).report(ctx)
}

struct ReadFieldResult {
    ty: Type,
    derefs: Vec<Type>,
}

struct Constraint {
    ty: Type,
    traitName: QualifiedName,
    location: Location,
}

pub struct Typechecker<'a> {
    ctx: &'a ReportContext,
    program: &'a Program,
    f: &'a Function,
    traitMethodSelector: &'a TraitMethodSelector,
    allocator: TypeVarAllocator,
    substitution: Substitution,
    types: BTreeMap<String, Type>,
    selfType: Option<Type>,
    mutables: BTreeMap<String, Mutability>,
    fieldTypes: BTreeMap<Variable, Vec<Type>>,
    bodyBuilder: BodyBuilder,
    visitedBlocks: BTreeSet<BlockId>,
    queue: VecDeque<BlockId>,
    knownConstraints: ConstraintContext,
    converterVars: BTreeMap<Variable, Variable>,
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
            substitution: Substitution::new(),
            types: BTreeMap::new(),
            selfType: None,
            mutables: BTreeMap::new(),
            fieldTypes: BTreeMap::new(),
            bodyBuilder: BodyBuilder::cloneFunction(f),
            visitedBlocks: BTreeSet::new(),
            queue: VecDeque::new(),
            knownConstraints: f.constraintContext.clone(),
            converterVars: BTreeMap::new(),
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
                        self.mutables.insert(name.clone(), Mutability::ExplicitMutable);
                    }
                }
                Parameter::SelfParam(mutable, ty) => {
                    let name = format!("self");
                    self.types.insert(name.clone(), ty.clone());
                    self.selfType = Some(ty.clone());
                    if *mutable {
                        self.mutables.insert(name, Mutability::ExplicitMutable);
                    }
                }
            }
        }
        if let Some(body) = &self.f.body {
            for block in &body.blocks {
                for instruction in &block.instructions {
                    match &instruction.kind {
                        InstructionKind::FunctionCall(_, _, _) => {}
                        InstructionKind::Converter(var, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::MethodCall(_, _, _, _) => {
                            //self.initializeVar(var);
                        }
                        InstructionKind::DynamicFunctionCall(var, _, _) => {
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
                                self.mutables.insert(var.value.to_string(), Mutability::ExplicitMutable);
                            } else {
                                self.mutables.insert(var.value.to_string(), Mutability::Immutable);
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
                        InstructionKind::DeclareVar(var, mutability) => {
                            self.initializeVar(var);
                            self.mutables.insert(var.value.to_string(), mutability.clone());
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

    fn tryUnify(&mut self, ty1: Type, ty2: Type) -> bool {
        //println!("UNIFY {} {}", ty1, ty2);
        if let Err(_) = unify(&mut self.substitution, &ty1, &ty2, false) {
            return false;
        }
        true
    }

    fn instantiateEnum(&mut self, e: &Enum, ty: &Type) -> Enum {
        instantiateEnum(&mut self.allocator, e, ty)
    }

    fn instantiateStruct(&mut self, c: &Struct, ty: &Type) -> Struct {
        instantiateStruct(&mut self.allocator, c, ty)
    }

    // fn handleImplicits(&mut self, input: &Variable, output: &Type, builder: &mut BlockBuilder) -> Type {
    //     let mut inputTy = self.getType(input).apply(&self.substitution);
    //     if inputTy.isReference() && !output.isReference() && !output.isGeneric() {
    //         if self.program.instanceResolver.isCopy(&output) {
    //             inputTy = inputTy.unpackRef().clone();
    //             //println!("IMPLICIT CLONE FOR {} {} {}", arg, argTy, fnArg);
    //             let tag = self.addImplicitCloneMarker(input);
    //             builder.addTag(tag);
    //         }
    //     }
    //     if inputTy.isReference()
    //         && output.isPtr()
    //         && ((inputTy.unpackRef() == output) || output.unpackPtr().isGeneric())
    //     {
    //         inputTy = inputTy.unpackRef().clone();
    //         //println!("IMPLICIT CLONE FOR {} {} {}", arg, argTy, fnArg);
    //         let tag = self.addImplicitCloneMarker(input);
    //         builder.addTag(tag);
    //     }
    //     if !inputTy.isGeneric() && !output.isGeneric() && inputTy != *output {
    //         if self.program.instanceResolver.isImplicitConvert(&inputTy, &output) {
    //             inputTy = output.clone();
    //             //println!("IMPLICIT CONVERT FOR {} {} {}", input, inputTy, output);
    //             let tag = self.addImplicitConvertMarker(input, ImplicitConvertInfo::Simple(output.clone()));
    //             builder.addTag(tag);
    //         }
    //     }
    //     inputTy
    // }

    fn updateConverterDestination(&mut self, dest: &Variable, target: &Type) {
        let destTy = self.getType(dest).apply(&self.substitution);
        let targetTy = target.apply(&self.substitution);
        if !self.tryUnify(destTy.clone(), targetTy.clone()) {
            match (destTy, targetTy.clone()) {
                (ty1, Type::Reference(ty2, _)) => {
                    self.tryUnify(ty1, *ty2.clone());
                }
                (Type::Reference(ty1, _), ty2) => {
                    self.tryUnify(*ty1.clone(), ty2);
                }
                (ty1, ty2) => {
                    self.tryUnify(ty1, ty2);
                }
            }
            let targetTy = target.apply(&self.substitution);
            self.types.insert(dest.value.to_string(), targetTy);
        }
    }

    fn checkFunctionCall(
        &mut self,
        args: &Vec<Variable>,
        resultVar: &Variable,
        fnType: Type,
        neededConstraints: &ConstraintContext,
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
            let fnArg2 = fnArg.apply(&self.substitution);
            // let mut argTy = self.handleImplicits(arg, &fnArg, builder);
            // //println!("ARG {} {} {}", arg, argTy, fnArg);
            // if !argTy.isReference() && fnArg.isReference() {
            //     let targetTy = fnArg.unpackRef().clone();
            //     if targetTy != argTy && self.program.instanceResolver.isImplicitConvert(&argTy, &targetTy) {
            //         //println!("IMPLICIT CONVERT REF FOR {} {} {}", arg, argTy, targetTy);
            //         argTy = Type::Reference(Box::new(targetTy.clone()), None);
            //         let tag = self.addImplicitConvertMarker(arg, ImplicitConvertInfo::Ref(targetTy));
            //         builder.addTag(tag);
            //     } else {
            //         argTy = Type::Reference(Box::new(argTy), None);
            //         //println!("IMPLICIT REF FOR {}", arg);
            //         let tag = self.addImplicitRefMarker(arg);
            //         builder.addTag(tag);
            //     }
            // }
            //self.unify(argTy, fnArg, arg.location.clone());
            self.updateConverterDestination(arg, &fnArg2);
            // if !self.tryUnify(argTy.clone(), fnArg2.clone()) {
            //     println!("ARG {} {} {}", arg, argTy, fnArg2);
            //     match (argTy, fnArg2) {
            //         (ty1, Type::Reference(ty2, _)) => {
            //             self.tryUnify(ty1, *ty2.clone());
            //         }
            //         (Type::Reference(ty1, _), ty2) => {
            //             self.tryUnify(*ty1.clone(), ty2);
            //         }
            //         (ty1, ty2) => {
            //             self.tryUnify(ty1, ty2);
            //         }
            //     }
            //     let fnArg = fnArg.apply(&self.substitution);
            //     self.types.insert(arg.value.to_string(), fnArg);
            // }
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
            Type::Named(name, _) => {
                if let Some(structDef) = self.program.structs.get(&name) {
                    let structDef = self.instantiateStruct(structDef, receiverType.unpackRef());
                    for m in &structDef.methods {
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
                    println!("Lookup method 1 on non-named type: {} {}", receiverType, methodName);
                    TypecheckerError::TypeAnnotationNeeded(location.clone()).report(self.ctx);
                }
            }
            Type::Var(TypeVar::Named(_)) => {
                return self.lookupTraitMethod(methodName, location);
            }
            _ => {
                println!("Lookup method 2 on non-named type: {} {}", receiverType, methodName);
                TypecheckerError::TypeAnnotationNeeded(location.clone()).report(self.ctx);
            }
        };
    }

    fn checkField(&mut self, receiverType: Type, fieldName: String, location: Location) -> Type {
        let receiverType = receiverType.apply(&self.substitution);
        match receiverType.unpackRef() {
            Type::Named(name, _) => {
                if let Some(structDef) = self.program.structs.get(&name) {
                    let structDef = self.instantiateStruct(structDef, receiverType.unpackRef());
                    for f in &structDef.fields {
                        if f.name == *fieldName {
                            if receiverType.isReference() {
                                return Type::Reference(Box::new(f.ty.clone()), None);
                            }
                            return f.ty.clone();
                        }
                    }
                    if let Some(i) = self.program.instanceResolver.isDeref(&receiverType) {
                        let receiverType = i.associatedTypes[0].ty.clone();
                        return self.checkField(receiverType, fieldName, location);
                    } else {
                        TypecheckerError::FieldNotFound(fieldName.clone(), location.clone()).report(self.ctx);
                    }
                } else {
                    //println!("ReadField 1 on non-named type: {} {}", receiverType, fieldName);
                    TypecheckerError::TypeAnnotationNeeded(location.clone()).report(self.ctx);
                }
            }
            _ => {
                //println!("ReadField 2 on non-named type: {} {}", receiverType, fieldName);
                TypecheckerError::TypeAnnotationNeeded(location.clone()).report(self.ctx);
            }
        }
    }

    fn readField(&mut self, receiverType: Type, fieldName: String, location: Location) -> ReadFieldResult {
        let receiverType = receiverType.apply(&self.substitution);
        match receiverType.unpackRef() {
            Type::Named(name, _) => {
                if let Some(structDef) = self.program.structs.get(&name) {
                    let structDef = self.instantiateStruct(structDef, receiverType.unpackRef());
                    for f in &structDef.fields {
                        if f.name == *fieldName {
                            let mut result = ReadFieldResult {
                                ty: f.ty.clone(),
                                derefs: Vec::new(),
                            };
                            if receiverType.isReference() {
                                result.ty = Type::Reference(Box::new(f.ty.clone()), None);
                            }
                            return result;
                        }
                    }
                    if let Some(i) = self.program.instanceResolver.isDeref(&receiverType.unpackRef()) {
                        let receiverType = i.associatedTypes[0].ty.clone();
                        let mut result = self.readField(receiverType.clone(), fieldName.clone(), location.clone());
                        if self.program.instanceResolver.isCopy(&result.ty) {
                            result.derefs.push(receiverType);
                            return result;
                        } else {
                            TypecheckerError::DerefNotCopy(result.ty.to_string(), fieldName, location.clone())
                                .report(self.ctx);
                        }
                    } else {
                        TypecheckerError::FieldNotFound(fieldName.clone(), location.clone()).report(self.ctx);
                    }
                } else {
                    //println!("ReadField 1 on non-named type: {} {}", receiverType, fieldName);
                    TypecheckerError::TypeAnnotationNeeded(location.clone()).report(self.ctx);
                }
            }
            _ => {
                //println!("ReadField 2 on non-named type: {} {}", receiverType, fieldName);
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
                self.checkFunctionCall(args, dest, fnType, &targetFn.constraintContext);
            }
            InstructionKind::Converter(dest, source) => {
                self.converterVars.insert(dest.clone(), source.clone());
                // println!("Converter {} {} {}", dest, source, instruction.location);
                // println!(
                //     "Converter {} {} {}",
                //     dest,
                //     self.getType(dest).apply(&self.substitution),
                //     self.getType(source).apply(&self.substitution)
                // );
                self.unify(self.getType(dest), self.getType(source), instruction.location.clone());
            }
            InstructionKind::MethodCall(dest, receiver, methodName, args) => {
                let receiverType = self.getType(receiver);
                let receiverType = receiverType.apply(&self.substitution);
                //println!("MethodCall {} {} {} {}", dest, receiver, methodName, receiverType);
                let name = self.lookupMethod(receiverType.clone(), methodName, instruction.location.clone());
                let mut extendedArgs = args.clone();
                extendedArgs.insert(0, receiver.clone());
                builder.replaceInstruction(
                    InstructionKind::FunctionCall(dest.clone(), name.clone(), extendedArgs.clone()),
                    instruction.location.clone(),
                );
                let targetFn = self.program.functions.get(&name).expect("Function not found");
                let mut fnType = targetFn.getType();
                let origReceiver = match self.converterVars.get(receiver) {
                    Some(var) => var.clone(),
                    None => receiver.clone(),
                };
                let mutableCall = self.mutables.get(&origReceiver.value.to_string())
                    == Some(&Mutability::ExplicitMutable)
                    && fnType.getResult().hasSelfType();
                if mutableCall {
                    fnType = fnType.changeMethodResult();
                }
                let fnType = self.checkFunctionCall(&extendedArgs, dest, fnType, &targetFn.constraintContext);
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
                    match selfLessType.getTupleTypes().len() {
                        0 => {
                            let kind = InstructionKind::FunctionCall(origReceiver.clone(), name, extendedArgs);
                            builder.replaceInstruction(kind, instruction.location.clone());
                        }
                        1 => {
                            let mut implicitResult = self.bodyBuilder.createTempValue(instruction.location.clone());
                            implicitResult.ty = Some(baseType.clone());
                            self.types.insert(implicitResult.value.to_string(), baseType.clone());
                            let fnCall = InstructionKind::FunctionCall(implicitResult.clone(), name, extendedArgs);
                            let mut implicitSelf = self.bodyBuilder.createTempValue(instruction.location.clone());
                            let receiverTy = self.getType(&origReceiver);
                            implicitSelf.ty = Some(receiverTy.clone());
                            self.types.insert(implicitSelf.value.to_string(), receiverTy.clone());
                            let implicitSelfIndex =
                                InstructionKind::TupleIndex(implicitSelf.clone(), implicitResult.clone(), 0);
                            let assign2 = InstructionKind::Assign(origReceiver.clone(), implicitSelf.clone());
                            let mut resVar = self.bodyBuilder.createTempValue(instruction.location.clone());
                            let destTy = self.getType(dest);
                            resVar.ty = Some(destTy.clone());
                            self.types.insert(resVar.value.to_string(), destTy.clone());
                            let assign = InstructionKind::Assign(dest.clone(), resVar.clone());
                            let resIndex = InstructionKind::TupleIndex(resVar.clone(), implicitResult.clone(), 1);
                            builder.replaceInstruction(fnCall, instruction.location.clone());
                            builder.step();
                            builder.addInstruction(assign, instruction.location.clone());
                            builder.addInstruction(assign2, instruction.location.clone());
                            builder.addInstruction(resIndex, instruction.location.clone());
                            builder.addInstruction(implicitSelfIndex, instruction.location.clone());
                        }
                        _ => {
                            let mut implicitResult = self.bodyBuilder.createTempValue(instruction.location.clone());
                            implicitResult.ty = Some(baseType.clone());
                            self.types.insert(implicitResult.value.to_string(), baseType.clone());
                            let fnCall = InstructionKind::FunctionCall(implicitResult.clone(), name, extendedArgs);
                            let mut implicitSelf = self.bodyBuilder.createTempValue(instruction.location.clone());
                            let receiverTy = self.getType(&origReceiver);
                            implicitSelf.ty = Some(receiverTy.clone());
                            self.types.insert(implicitSelf.value.to_string(), receiverTy.clone());
                            let implicitSelfIndex =
                                InstructionKind::TupleIndex(implicitSelf.clone(), implicitResult.clone(), 0);
                            let assign = InstructionKind::Assign(origReceiver.clone(), implicitSelf.clone());
                            let mut args = Vec::new();
                            let tupleTypes = selfLessType.getTupleTypes();
                            let mut tupleIndices = Vec::new();
                            for (argIndex, argType) in tupleTypes.iter().enumerate() {
                                let mut resVar = self.bodyBuilder.createTempValue(instruction.location.clone());
                                resVar.ty = Some(argType.clone());
                                args.push(resVar.clone());
                                self.types.insert(resVar.value.to_string(), argType.clone());
                                let tupleIndexN = InstructionKind::TupleIndex(
                                    resVar.clone(),
                                    implicitResult.clone(),
                                    (argIndex + 1) as i32,
                                );
                                tupleIndices.push(tupleIndexN);
                            }
                            let tuple = InstructionKind::Tuple(dest.clone(), args);
                            builder.replaceInstruction(fnCall, instruction.location.clone());
                            builder.step();
                            builder.addInstruction(tuple, instruction.location.clone());
                            builder.addInstruction(assign, instruction.location.clone());
                            builder.addInstruction(implicitSelfIndex, instruction.location.clone());
                            for i in tupleIndices {
                                builder.addInstruction(i, instruction.location.clone());
                            }
                        }
                    }
                }
                //println!("METHOD CALL {} {} {}", dest, name, fnType);
            }
            InstructionKind::DynamicFunctionCall(dest, callable, args) => {
                let fnType = self.getType(callable);
                self.checkFunctionCall(&args, dest, fnType, &ConstraintContext::new());
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
                self.updateConverterDestination(arg, &result);
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
                if self.mutables.get(&name.value.to_string()) == Some(&Mutability::Immutable) {
                    TypecheckerError::ImmutableAssign(instruction.location.clone()).report(self.ctx);
                }
                self.unify(self.getType(name), self.getType(rhs), instruction.location.clone());
            }
            InstructionKind::FieldAssign(name, rhs, fields) => {
                if self.mutables.get(&name.value.to_string()) == Some(&Mutability::Immutable) {
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
            InstructionKind::DeclareVar(_, _) => {}
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
                        //println!("Transform on non-enum type: {} {}", rootTy, instruction);
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
            InstructionKind::FieldRef(dest, receiver, fields) => {
                let receiverType = self.getType(receiver);
                assert_eq!(fields.len(), 1, "FieldRef with multiple fields in typecheck!");
                let fieldName = fields[0].name.clone();
                let result = self.readField(receiverType, fieldName.clone(), instruction.location.clone());
                if result.derefs.len() > 0 {
                    let mut rootVar = receiver.clone();
                    for deref in &result.derefs {
                        let mut derefInputVar = rootVar.clone();
                        let rootVarTy = self.getType(&rootVar).apply(&self.substitution);
                        let derefResultVar = self.bodyBuilder.createTempValue(instruction.location.clone());
                        if !rootVarTy.isReference() {
                            let refResultVar = self.bodyBuilder.createTempValue(instruction.location.clone());
                            self.types.insert(
                                refResultVar.value.to_string(),
                                Type::Reference(Box::new(rootVarTy.clone()), None),
                            );
                            let kind = InstructionKind::Ref(refResultVar.clone(), rootVar.clone());
                            builder.addInstruction(kind, instruction.location.clone());
                            builder.step();
                            derefInputVar = refResultVar;
                        }
                        let kind = InstructionKind::FunctionCall(
                            derefResultVar.clone(),
                            getDerefGetName(),
                            vec![derefInputVar.clone()],
                        );
                        self.types.insert(derefResultVar.value.to_string(), deref.clone());
                        builder.addInstruction(kind, instruction.location.clone());
                        builder.step();
                        rootVar = derefResultVar;
                    }
                    builder.replaceInstruction(
                        InstructionKind::FieldRef(dest.clone(), rootVar.clone(), fields.clone()),
                        instruction.location.clone(),
                    );
                }
                self.unify(self.getType(dest), result.ty, instruction.location.clone());
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
                    _ => {
                        //println!("TupleIndex on non-tuple type: {} {}", receiverType, instruction);
                        TypecheckerError::TypeAnnotationNeeded(instruction.location.clone()).report(self.ctx);
                    }
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
                        if let InstructionKind::FieldRef(dest, root, fields) = &instruction.kind {
                            assert_eq!(fields.len(), 1, "FieldRef with multiple fields in typecheck!");
                            let mut fields = fields.clone();
                            let destTy = self.getType(dest).apply(&self.substitution);
                            fields[0].ty = Some(destTy.clone());
                            let kind = InstructionKind::FieldRef(dest.clone(), root.clone(), fields);
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
        let body = &mut f.body.as_mut().unwrap();

        for block in &mut body.blocks {
            for instruction in &mut block.instructions {
                let vars = instruction.kind.collectVariables();
                for var in vars {
                    let ty = self.getType(&var);
                    let ty = ty.apply(&self.substitution);
                    let mut newVar = var.clone();
                    newVar.ty = Some(ty.clone());
                    instruction.kind = instruction.kind.replaceVar(var, newVar);
                }
            }
        }
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
        let mut sub = Substitution::new();
        for (arg, ctxArg) in zip(&traitDef.params, &c.args) {
            sub.add(arg.clone(), ctxArg.clone());
        }
        let traitDef = traitDef.apply(&sub);
        self.knownConstraints.constraints.push(c.clone());
        for c in traitDef.constraint.constraints {
            self.expandKnownConstraint(&c, processed);
        }
    }

    fn processConverters(&mut self) {
        // println!("processConverters {}", self.f.name);
        let allblocksIds = self.bodyBuilder.getAllBlockIds();
        for blockId in allblocksIds {
            let mut builder = self.bodyBuilder.iterator(blockId);
            loop {
                match builder.getInstruction() {
                    Some(instruction) => {
                        if let InstructionKind::Converter(dest, source) = &instruction.kind {
                            let destTy = self.getType(dest).apply(&self.substitution);
                            let sourceTy = self.getType(source).apply(&self.substitution);
                            //println!("Processing converter {} : {} -> {}", instruction, sourceTy, destTy);
                            match (&destTy, &sourceTy) {
                                (Type::Reference(inner, _), Type::Reference(src, _)) => {
                                    self.unify(*inner.clone(), *src.clone(), instruction.location.clone());
                                    builder.addDeclare(dest.clone(), instruction.location.clone());
                                    builder.step();
                                    let kind = InstructionKind::Assign(dest.clone(), source.clone());
                                    builder.replaceInstruction(kind, instruction.location.clone());
                                }
                                (destTy, Type::Reference(sourceInner, _)) => {
                                    self.unify(destTy.clone(), *sourceInner.clone(), instruction.location.clone());
                                    let kind = if destTy.isPtr() {
                                        InstructionKind::FunctionCall(
                                            dest.clone(),
                                            getNativePtrCloneName(),
                                            vec![source.clone()],
                                        )
                                    } else {
                                        if self.program.instanceResolver.isCopy(destTy) {
                                            InstructionKind::FunctionCall(
                                                dest.clone(),
                                                getCloneFnName(),
                                                vec![source.clone()],
                                            )
                                        } else {
                                            TypecheckerError::TypeMismatch(
                                                sourceTy.to_string(),
                                                destTy.to_string(),
                                                instruction.location.clone(),
                                            )
                                            .report(self.ctx);
                                        }
                                    };
                                    builder.replaceInstruction(kind, instruction.location.clone());
                                }
                                (Type::Reference(inner, _), src) => {
                                    let mut refSource = source.clone();
                                    if !self.tryUnify(*inner.clone(), src.clone()) {
                                        // check implicit conversion is implemented for these types
                                        if !self.program.instanceResolver.isImplicitConvert(&src, &inner) {
                                            TypecheckerError::TypeMismatch(
                                                destTy.to_string(),
                                                sourceTy.to_string(),
                                                instruction.location.clone(),
                                            )
                                            .report(self.ctx);
                                        } else {
                                            let mut newVar =
                                                self.bodyBuilder.createTempValue(instruction.location.clone());
                                            newVar.ty = Some(*inner.clone());
                                            self.types.insert(newVar.value.to_string(), *inner.clone());
                                            let kind = InstructionKind::FunctionCall(
                                                newVar.clone(),
                                                getImplicitConvertFnName(),
                                                vec![source.clone()],
                                            );
                                            builder.addInstruction(kind, instruction.location.clone());
                                            builder.step();
                                            refSource = newVar;
                                        }
                                    }
                                    let kind = InstructionKind::Ref(dest.clone(), refSource.clone());
                                    builder.replaceInstruction(kind, instruction.location.clone());
                                }
                                (t1, t2) => {
                                    if !self.tryUnify(t1.clone(), t2.clone()) {
                                        if !self.program.instanceResolver.isImplicitConvert(&t2, &t1) {
                                            TypecheckerError::TypeMismatch(
                                                destTy.to_string(),
                                                sourceTy.to_string(),
                                                instruction.location.clone(),
                                            )
                                            .report(self.ctx);
                                        } else {
                                            let kind = InstructionKind::FunctionCall(
                                                dest.clone(),
                                                getImplicitConvertFnName(),
                                                vec![source.clone()],
                                            );
                                            builder.replaceInstruction(kind, instruction.location.clone());
                                        }
                                    } else {
                                        builder.addDeclare(dest.clone(), instruction.location.clone());
                                        builder.step();
                                        let kind = InstructionKind::Assign(dest.clone(), source.clone());
                                        builder.replaceInstruction(kind, instruction.location.clone());
                                    }
                                }
                            }
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

    fn removeBinds(&mut self) {
        // println!("removeBinds {}", self.f.name);
        let allblocksIds = self.bodyBuilder.getAllBlockIds();
        for blockId in allblocksIds {
            let mut builder = self.bodyBuilder.iterator(blockId);
            loop {
                match builder.getInstruction() {
                    Some(instruction) => {
                        if let InstructionKind::Bind(dest, src, _) = &instruction.kind {
                            builder.addDeclare(dest.clone(), instruction.location.clone());
                            let kind = InstructionKind::Assign(dest.clone(), src.clone());
                            builder.step();
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

    pub fn generate(&mut self) -> Function {
        //println!("Generating {}", self.f.name);
        if self.f.body.is_none() {
            return self.f.clone();
        }

        self.processConverters();
        self.addFieldTypes();
        self.removeBinds();

        let mut result = self.f.clone();
        result.body = Some(self.bodyBuilder.build());
        if let Some(selfType) = self.selfType.clone() {
            result.result = result.result.changeSelfType(selfType);
        }

        self.addTypes(&mut result);
        //self.dump(&result);
        self.verify(&result);
        result
    }
}
