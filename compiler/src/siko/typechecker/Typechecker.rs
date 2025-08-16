use core::panic;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    iter::zip,
};

use crate::siko::{
    hir::{
        Apply::Apply,
        BlockBuilder::BlockBuilder,
        BodyBuilder::BodyBuilder,
        ConstraintContext::{Constraint as HirConstraint, ConstraintContext},
        Data::{Enum, Struct},
        Function::{BlockId, Function, Parameter},
        Instantiation::{instantiateEnum, instantiateStruct, instantiateTrait, instantiateTypes},
        Instruction::{FieldId, FieldInfo, Instruction, InstructionKind, Mutability},
        Program::Program,
        Substitution::Substitution,
        TraitMethodSelector::TraitMethodSelector,
        Type::{Type, TypeVar},
        TypeVarAllocator::TypeVarAllocator,
        Unification::unify,
        Variable::Variable,
    },
    location::{Location::Location, Report::ReportContext},
    qualifiedname::{
        builtins::{
            getCloneFnName, getImplicitConvertFnName, getNativePtrCloneName, getNativePtrIsNullName,
            getNativePtrLoadName, getNativePtrStoreName,
        },
        QualifiedName,
    },
    typechecker::ConstraintChecker::ConstraintChecker,
};

use super::Error::TypecheckerError;

pub fn typecheck(ctx: &ReportContext, mut program: Program) -> Program {
    let mut result = BTreeMap::new();
    for (_, f) in &program.functions {
        let moduleName = f.name.module();
        let traitMethodSelector = &program
            .traitMethodSelectors
            .get(&moduleName)
            .expect("Trait method selector not found");
        let mut typechecker = Typechecker::new(ctx, &program, &traitMethodSelector, f);
        let typedFn = typechecker.run();
        //typedFn.dump();
        result.insert(typedFn.name.clone(), typedFn);
    }
    program.functions = result;
    program
}

pub fn reportTypeMismatch(ctx: &ReportContext, ty1: Type, ty2: Type, location: Location) {
    TypecheckerError::TypeMismatch(format!("{}", ty1), format!("{}", ty2), location).report(ctx)
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
                self.types.insert(var.name.to_string(), ty.clone());
            }
            None => {
                if let Some(ty) = self.bodyBuilder.getTypeInBody(&var) {
                    self.types.insert(var.name.to_string(), ty.clone());
                } else {
                    let ty = self.allocator.next();
                    self.types.insert(var.name.to_string(), ty.clone());
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
            for (_, block) in &body.blocks {
                for instruction in &block.instructions {
                    match &instruction.kind {
                        InstructionKind::FunctionCall(dest, _, _) => {
                            self.initializeVar(dest);
                        }
                        InstructionKind::Converter(var, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::MethodCall(dest, _, _, _) => {
                            self.initializeVar(dest);
                        }
                        InstructionKind::DynamicFunctionCall(var, _, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::FieldRef(var, _, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::Bind(var, _, mutable) => {
                            self.initializeVar(var);
                            if *mutable {
                                self.mutables.insert(var.name.to_string(), Mutability::ExplicitMutable);
                            } else {
                                self.mutables.insert(var.name.to_string(), Mutability::Immutable);
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
                            self.types.insert(var.name.to_string(), Type::Never(false));
                        }
                        InstructionKind::Ref(var, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::DropPath(_) => {
                            panic!("DropListPlaceholder found in Typechecker, this should not happen");
                        }
                        InstructionKind::DropMetadata(_) => {
                            panic!("DropMetadata found in Typechecker, this should not happen");
                        }
                        InstructionKind::Drop(_, _) => {}
                        InstructionKind::Jump(var, _) => {
                            self.types.insert(var.name.to_string(), Type::Never(false));
                        }
                        InstructionKind::Assign(_, _) => {}
                        InstructionKind::FieldAssign(_, _, _) => {}
                        InstructionKind::AddressOfField(var, _, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::DeclareVar(var, mutability) => {
                            self.initializeVar(var);
                            self.mutables.insert(var.name.to_string(), mutability.clone());
                        }
                        InstructionKind::Transform(var, _, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::EnumSwitch(_, _) => {}
                        InstructionKind::IntegerSwitch(_, _) => {}
                        InstructionKind::BlockStart(_) => {}
                        InstructionKind::BlockEnd(_) => {}
                        InstructionKind::With(handlers, _) => {
                            for effectHandler in handlers {
                                let method = self
                                    .program
                                    .getFunction(&effectHandler.method)
                                    .expect("Method function not found");
                                let handlerFn = self
                                    .program
                                    .getFunction(&effectHandler.handler)
                                    .expect("Handler function not found");
                                let methodType = method.getType();
                                let handlerType = handlerFn.getType();
                                self.unify(methodType, handlerType, effectHandler.location.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    fn getType(&self, var: &Variable) -> Type {
        match self.types.get(&var.name.to_string()) {
            Some(ty) => ty.clone(),
            None => panic!("No type found for {}!", var),
        }
    }

    fn unify(&mut self, ty1: Type, ty2: Type, location: Location) {
        //println!("UNIFY {} {}", ty1, ty2);
        if let Err(_) = unify(&mut self.substitution, ty1.clone(), ty2.clone(), false) {
            reportTypeMismatch(
                self.ctx,
                ty1.apply(&self.substitution),
                ty2.apply(&self.substitution),
                location,
            );
        }
    }

    fn tryUnify(&mut self, ty1: Type, ty2: Type) -> bool {
        //println!("UNIFY {} {}", ty1, ty2);
        if let Err(_) = unify(&mut self.substitution, ty1.clone(), ty2.clone(), false) {
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

    fn updateConverterDestination(&mut self, dest: &Variable, target: &Type) {
        let destTy = self.getType(dest).apply(&self.substitution);
        let targetTy = target.clone().apply(&self.substitution);
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
            let targetTy = target.clone().apply(&self.substitution);
            self.types.insert(dest.name.to_string(), targetTy);
        }
    }

    fn checkFunctionCall(
        &mut self,
        args: &Vec<Variable>,
        resultVar: &Variable,
        fnType: Type,
        neededConstraints: &ConstraintContext,
    ) -> Type {
        //println!(
        //    "checkFunctionCall: {} {} {}",
        //    fnType, neededConstraints, self.knownConstraints
        //);
        let mut types = neededConstraints.typeParameters.clone();
        types.push(fnType.clone());
        let sub = instantiateTypes(&mut self.allocator, &types);
        let instantiatedFnType = fnType.apply(&sub);
        //println!("inst {}", instantiatedFnType);
        let constraintContext = neededConstraints.clone().apply(&sub);
        //println!("applied constraintContext {}", constraintContext);
        let (fnArgs, mut fnResult) = match instantiatedFnType.clone().splitFnType() {
            Some((fnArgs, fnResult)) => (fnArgs, fnResult),
            None => panic!("Function type {} is not a function type!", instantiatedFnType),
        };
        if args.len() != fnArgs.len() {
            TypecheckerError::ArgCountMismatch(fnArgs.len() as u32, args.len() as u32, resultVar.location.clone())
                .report(self.ctx);
        }
        if fnArgs.len() > 0 {
            fnResult = fnResult.changeSelfType(fnArgs[0].clone());
        }
        //println!("fnResult {}", fnResult);
        //println!("self.getType(resultVar) {}", self.getType(resultVar));
        let fnResult = fnResult.clone().apply(&self.substitution);
        //println!("fnResult {}", fnResult);
        self.unify(self.getType(resultVar), fnResult.clone(), resultVar.location.clone());
        let originalSub = self.substitution.clone();
        let mut helperSub = Substitution::new();
        loop {
            for (arg, fnArg) in zip(args, &fnArgs) {
                //println!("arg {} fnArg {}", arg, fnArg);
                let mut fnArg2 = fnArg.clone().apply(&self.substitution);
                //println!("fnArg2 {}", fnArg2);
                if !helperSub.empty() {
                    fnArg2 = fnArg2.apply(&helperSub);
                }
                //println!("Convert arg {} => fnArg {}", arg, fnArg2);
                self.updateConverterDestination(arg, &fnArg2);
            }
            let constraints = constraintContext.clone().apply(&self.substitution);

            let mut constraintChecker = ConstraintChecker::new(
                &self.allocator,
                self.ctx,
                self.program,
                &self.knownConstraints,
                &self.substitution,
            );

            match constraintChecker.checkConstraint(&constraints, resultVar.location.clone()) {
                Ok(()) => {
                    self.allocator = constraintChecker.allocator;
                    self.substitution = constraintChecker.substitution;
                    break;
                }
                Err(err) => {
                    if helperSub.empty() {
                        let newFnType = instantiatedFnType.clone().apply(&constraintChecker.substitution);
                        let (newFnArgs, _) = newFnType.splitFnType().expect("Failed to split function type");
                        for (arg, newFnArg) in zip(fnArgs.clone(), &newFnArgs) {
                            //println!("arg {} fnArg {}", arg, newFnArg);
                            if arg.isGeneric() && newFnArg.isReference() {
                                if let Type::Reference(inner, _) = newFnArg {
                                    if self.program.instanceResolver.isCopy(&inner) {
                                        //println!("Worth trying to clone {}", inner);
                                        helperSub.add(arg.clone(), *inner.clone());
                                    }
                                }
                            }
                        }
                        if !helperSub.empty() {
                            //println!("Applying helper substitution: {}", helperSub);
                            self.substitution = originalSub.clone();
                            continue;
                        }
                    }
                    err.report(self.ctx);
                }
            }
        }
        // println!("fnResult {}", fnResult);
        // println!("self.getType(resultVar) {}", self.getType(resultVar));
        let fnResult = fnResult.apply(&self.substitution);
        //println!("fnResult {}", fnResult);
        self.unify(self.getType(resultVar), fnResult, resultVar.location.clone());
        instantiatedFnType.apply(&self.substitution)
    }

    fn lookupTraitMethod(&mut self, receiverType: Type, methodName: &String, location: Location) -> QualifiedName {
        if let Some(selections) = self.traitMethodSelector.get(methodName) {
            if selections.len() > 1 {
                TypecheckerError::MethodAmbiguous(methodName.clone(), location.clone()).report(self.ctx);
            }
            return selections[0].method.clone();
        }
        TypecheckerError::MethodNotFound(methodName.clone(), receiverType.to_string(), location.clone())
            .report(self.ctx);
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
                    return self.lookupTraitMethod(receiverType, methodName, location);
                } else if let Some(enumDef) = self.program.enums.get(&name) {
                    let enumDef = self.instantiateEnum(enumDef, receiverType.unpackRef());
                    for m in &enumDef.methods {
                        if m.name == *methodName {
                            return m.fullName.clone();
                        }
                    }
                    return self.lookupTraitMethod(receiverType, methodName, location);
                } else {
                    TypecheckerError::MethodNotFound(methodName.clone(), receiverType.to_string(), location.clone())
                        .report(self.ctx);
                }
            }
            Type::Var(TypeVar::Named(_)) => {
                return self.lookupTraitMethod(receiverType, methodName, location);
            }
            Type::Ptr(_) => {
                if methodName == "isNull" {
                    // TODO: make this nicer, somehow??
                    return getNativePtrIsNullName();
                } else {
                    TypecheckerError::MethodNotFound(methodName.clone(), receiverType.to_string(), location.clone())
                        .report(self.ctx);
                }
            }
            _ => {
                TypecheckerError::MethodNotFound(methodName.clone(), receiverType.to_string(), location.clone())
                    .report(self.ctx);
            }
        };
    }

    fn checkField(&mut self, mut receiverType: Type, fieldId: &FieldId, location: Location) -> Type {
        let origType = receiverType.clone();
        receiverType = receiverType.apply(&self.substitution);
        if let Type::Reference(_, _) = &receiverType {
            TypecheckerError::FieldNotFound(fieldId.to_string(), origType.to_string(), location.clone())
                .report(self.ctx);
        }
        if let Type::Ptr(innerTy) = &receiverType {
            receiverType = *innerTy.clone();
        }
        match &fieldId {
            FieldId::Named(fieldName) => match &receiverType {
                Type::Named(name, _) => {
                    if let Some(structDef) = self.program.getStruct(name) {
                        let structDef = self.instantiateStruct(&structDef, &receiverType);
                        for f in &structDef.fields {
                            if f.name == *fieldName {
                                return f.ty.clone();
                            }
                        }
                        TypecheckerError::FieldNotFound(fieldName.clone(), origType.to_string(), location.clone())
                            .report(self.ctx);
                    } else {
                        TypecheckerError::FieldNotFound(fieldName.clone(), origType.to_string(), location.clone())
                            .report(self.ctx);
                    }
                }
                _ => {
                    TypecheckerError::FieldNotFound(fieldName.clone(), origType.to_string(), location.clone())
                        .report(self.ctx);
                }
            },
            FieldId::Indexed(index) => {
                let receiverType = receiverType.apply(&self.substitution);
                match receiverType.unpackRef() {
                    Type::Tuple(types) => {
                        if *index as usize >= types.len() {
                            TypecheckerError::FieldNotFound(
                                fieldId.to_string().clone(),
                                origType.to_string(),
                                location.clone(),
                            )
                            .report(self.ctx);
                        }
                        if receiverType.isReference() {
                            return Type::Reference(Box::new(types[*index as usize].clone()), None);
                        }
                        return types[*index as usize].clone();
                    }
                    _ => {
                        TypecheckerError::FieldNotFound(
                            fieldId.to_string().clone(),
                            origType.to_string(),
                            location.clone(),
                        )
                        .report(self.ctx);
                    }
                }
            }
        }
    }

    fn readField(&mut self, receiverType: Type, fieldName: String, location: Location) -> Type {
        let receiverType = receiverType.apply(&self.substitution);
        match receiverType.unpackRef() {
            Type::Named(name, _) => {
                if let Some(structDef) = self.program.structs.get(&name) {
                    let structDef = self.instantiateStruct(structDef, receiverType.unpackRef());
                    for f in &structDef.fields {
                        if f.name == *fieldName {
                            let mut result = f.ty.clone();
                            if receiverType.isReference() {
                                result = Type::Reference(Box::new(f.ty.clone()), None);
                            }
                            return result;
                        }
                    }
                    TypecheckerError::FieldNotFound(fieldName.clone(), receiverType.to_string(), location.clone())
                        .report(self.ctx);
                } else {
                    TypecheckerError::FieldNotFound(fieldName.clone(), receiverType.to_string(), location.clone())
                        .report(self.ctx);
                }
            }
            _ => {
                TypecheckerError::FieldNotFound(fieldName.clone(), receiverType.to_string(), location.clone())
                    .report(self.ctx);
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
                //println!("FunctionCall {} {} {:?}", dest, name, args);
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
                let mutableCall = self.mutables.get(&origReceiver.name.to_string())
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
                            let mut implicitResult = self.bodyBuilder.createTempValue(instruction.location.clone());
                            implicitResult.ty = Some(baseType.clone());
                            self.types.insert(implicitResult.name.to_string(), baseType.clone());
                            let kind = InstructionKind::FunctionCall(implicitResult.clone(), name, extendedArgs);
                            builder.replaceInstruction(kind, instruction.location.clone());
                            builder.step();
                            builder.addAssign(
                                origReceiver.clone(),
                                implicitResult.clone(),
                                instruction.location.clone(),
                            );
                            builder.addInstruction(
                                InstructionKind::Tuple(dest.clone(), vec![]),
                                instruction.location.clone(),
                            );
                        }
                        1 => {
                            let mut implicitResult = self.bodyBuilder.createTempValue(instruction.location.clone());
                            implicitResult.ty = Some(baseType.clone());
                            self.types.insert(implicitResult.name.to_string(), baseType.clone());
                            let fnCall = InstructionKind::FunctionCall(implicitResult.clone(), name, extendedArgs);
                            let mut implicitSelf = self.bodyBuilder.createTempValue(instruction.location.clone());
                            let receiverTy = self.getType(&origReceiver);
                            implicitSelf.ty = Some(receiverTy.clone());
                            self.types.insert(implicitSelf.name.to_string(), receiverTy.clone());
                            let implicitSelfIndex = InstructionKind::FieldRef(
                                implicitSelf.clone(),
                                implicitResult.clone(),
                                vec![FieldInfo {
                                    name: FieldId::Indexed(0),
                                    ty: Some(receiverTy.clone()),
                                    location: instruction.location.clone(),
                                }],
                            );
                            let assign2 = InstructionKind::Assign(origReceiver.clone(), implicitSelf.clone());
                            let mut resVar = self.bodyBuilder.createTempValue(instruction.location.clone());
                            let destTy = self.getType(dest);
                            resVar.ty = Some(destTy.clone());
                            self.types.insert(resVar.name.to_string(), destTy.clone());
                            let assign = InstructionKind::Assign(dest.clone(), resVar.clone());
                            let resIndex = InstructionKind::FieldRef(
                                resVar.clone(),
                                implicitResult.clone(),
                                vec![FieldInfo {
                                    name: FieldId::Indexed(1),
                                    ty: Some(destTy.clone()),
                                    location: instruction.location.clone(),
                                }],
                            );
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
                            self.types.insert(implicitResult.name.to_string(), baseType.clone());
                            let fnCall = InstructionKind::FunctionCall(implicitResult.clone(), name, extendedArgs);
                            let mut implicitSelf = self.bodyBuilder.createTempValue(instruction.location.clone());
                            let receiverTy = self.getType(&origReceiver);
                            implicitSelf.ty = Some(receiverTy.clone());
                            self.types.insert(implicitSelf.name.to_string(), receiverTy.clone());
                            let implicitSelfIndex = InstructionKind::FieldRef(
                                implicitSelf.clone(),
                                implicitResult.clone(),
                                vec![FieldInfo {
                                    name: FieldId::Indexed(0),
                                    ty: Some(receiverTy.clone()),
                                    location: instruction.location.clone(),
                                }],
                            );
                            let assign = InstructionKind::Assign(origReceiver.clone(), implicitSelf.clone());
                            let mut args = Vec::new();
                            let tupleTypes = selfLessType.getTupleTypes();
                            let mut tupleIndices = Vec::new();
                            for (argIndex, argType) in tupleTypes.iter().enumerate() {
                                let mut resVar = self.bodyBuilder.createTempValue(instruction.location.clone());
                                resVar.ty = Some(argType.clone());
                                args.push(resVar.clone());
                                self.types.insert(resVar.name.to_string(), argType.clone());
                                let tupleIndexN = InstructionKind::FieldRef(
                                    resVar.clone(),
                                    implicitResult.clone(),
                                    vec![FieldInfo {
                                        name: FieldId::Indexed((argIndex + 1) as u32),
                                        ty: Some(argType.clone()),
                                        location: instruction.location.clone(),
                                    }],
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
            InstructionKind::DropPath(_) => unreachable!("drop list placeholder in typechecker!"),
            InstructionKind::DropMetadata(_) => unreachable!("drop metadata in typechecker!"),
            InstructionKind::Drop(_, _) => unreachable!("drop in typechecker!"),
            InstructionKind::Jump(_, id) => {
                self.queue.push_back(*id);
            }
            InstructionKind::Assign(name, rhs) => {
                if self.mutables.get(&name.name.to_string()) == Some(&Mutability::Immutable) {
                    TypecheckerError::ImmutableAssign(instruction.location.clone()).report(self.ctx);
                }
                self.unify(self.getType(name), self.getType(rhs), instruction.location.clone());
            }
            InstructionKind::FieldAssign(receiver, rhs, fields) => {
                if self.mutables.get(&receiver.name.to_string()) == Some(&Mutability::Immutable) {
                    TypecheckerError::ImmutableAssign(instruction.location.clone()).report(self.ctx);
                }
                let receiverType = self.getType(receiver);
                let mut receiverType = receiverType.apply(&self.substitution);
                //println!("FieldAssign start {} {} {}", receiverType, name, instruction.location);
                let mut ptrReceiver = false;
                let mut newFields = Vec::new();
                for field in fields {
                    if receiverType.isPtr() {
                        ptrReceiver = true;
                    }
                    let fieldTy = self.checkField(receiverType, &field.name, field.location.clone());
                    let mut newField = field.clone();
                    newField.ty = Some(fieldTy.clone());
                    newFields.push(newField);
                    receiverType = fieldTy;
                    //println!("FieldAssign updated {} {} {}", receiverType, field.name, field.location);
                }
                if !ptrReceiver {
                    let kind = InstructionKind::FieldAssign(receiver.clone(), rhs.clone(), newFields);
                    builder.replaceInstruction(kind, instruction.location.clone());
                } else {
                    let mut addressOfVar = self.bodyBuilder.createTempValue(instruction.location.clone());
                    addressOfVar.ty = Some(Type::Ptr(Box::new(receiverType.clone())));
                    self.types
                        .insert(addressOfVar.name.to_string(), Type::Ptr(Box::new(receiverType.clone())));
                    let kind = InstructionKind::AddressOfField(addressOfVar.clone(), receiver.clone(), newFields);
                    builder.addInstruction(kind, instruction.location.clone());
                    builder.step();
                    let mut storeVar = self.bodyBuilder.createTempValue(instruction.location.clone());
                    storeVar.ty = Some(Type::getUnitType());
                    self.types.insert(storeVar.name.to_string(), Type::getUnitType());
                    let store = InstructionKind::FunctionCall(
                        storeVar,
                        getNativePtrStoreName(),
                        vec![addressOfVar, rhs.clone()],
                    );
                    builder.replaceInstruction(store, instruction.location.clone());
                }
                // println!(
                //     "FieldAssign check {} {} {}",
                //     self.getType(rhs).apply(&self.substitution),
                //     receiverType,
                //     instruction.location
                // );
                self.unify(self.getType(rhs), receiverType, instruction.location.clone());
            }
            InstructionKind::AddressOfField(dest, receiver, fields) => {
                let receiverType = self.getType(receiver);
                let mut receiverType = receiverType.apply(&self.substitution);
                let mut newFields = Vec::new();
                for field in fields {
                    let fieldTy = self.checkField(receiverType, &field.name, field.location.clone());
                    let mut newField = field.clone();
                    newField.ty = Some(fieldTy.clone());
                    newFields.push(newField);
                    receiverType = fieldTy;
                }
                receiverType = Type::Reference(Box::new(receiverType), None);
                let newKind = InstructionKind::AddressOfField(dest.clone(), receiver.clone(), newFields);
                builder.replaceInstruction(newKind, instruction.location.clone());
                self.unify(self.getType(dest), receiverType, instruction.location.clone());
            }
            InstructionKind::DeclareVar(_, _) => {}
            InstructionKind::Transform(dest, root, index) => {
                let rootTy = self.getType(root);
                let rootTy = rootTy.apply(&self.substitution);
                let isRef = rootTy.isReference();
                match rootTy.unpackRef().getName() {
                    Some(name) => {
                        let e = self.program.enums.get(&name).expect("not an enum in transform!");
                        let e = self.instantiateEnum(e, &rootTy.unpackRef());
                        let v = &e.variants[*index as usize];
                        let destType = if isRef {
                            Type::Reference(Box::new(Type::Tuple(v.items.clone())), None)
                        } else {
                            Type::Tuple(v.items.clone())
                        };
                        self.unify(self.getType(dest), destType, instruction.location.clone());
                    }
                    None => {
                        println!("Transform on non-enum type: {} {}", rootTy, instruction);
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
            InstructionKind::FieldRef(dest, receiver, fields) => {
                let receiver = receiver.clone();
                let mut receiverType = self.getType(&receiver);
                assert_eq!(fields.len(), 1, "FieldRef with multiple fields in typecheck!");
                receiverType = receiverType.apply(&self.substitution);
                if let Type::Reference(innerTy, _) = &receiverType {
                    receiverType = *innerTy.clone();
                }
                if let Type::Ptr(innerTy) = &receiverType {
                    let mut ptrLoadResultVar = self.bodyBuilder.createTempValue(instruction.location.clone());
                    ptrLoadResultVar.ty = Some(*innerTy.clone());
                    self.types.insert(ptrLoadResultVar.name.to_string(), *innerTy.clone());
                    builder.addInstruction(
                        InstructionKind::FunctionCall(
                            ptrLoadResultVar.clone(),
                            getNativePtrLoadName(),
                            vec![receiver.clone()],
                        ),
                        instruction.location.clone(),
                    );
                    builder.step();
                    let kind = instruction.kind.replaceVar(receiver.clone(), ptrLoadResultVar.clone());
                    builder.replaceInstruction(kind, instruction.location.clone());
                    receiverType = *innerTy.clone();
                } else {
                    receiverType = self.getType(&receiver);
                }
                let fieldName = fields[0].name.clone();
                match fieldName {
                    FieldId::Named(n) => {
                        let result = self.readField(receiverType, n, instruction.location.clone());
                        self.unify(self.getType(dest), result, instruction.location.clone());
                    }
                    FieldId::Indexed(index) => {
                        receiverType = receiverType.apply(&self.substitution);
                        let isRef = receiverType.isReference();
                        match receiverType.unpackRef() {
                            Type::Tuple(t) => {
                                if index as usize >= t.len() {
                                    TypecheckerError::FieldNotFound(
                                        fieldName.to_string().clone(),
                                        receiverType.to_string(),
                                        instruction.location.clone(),
                                    )
                                    .report(self.ctx);
                                }
                                let fieldType = if isRef {
                                    Type::Reference(Box::new(t[index as usize].clone()), None)
                                } else {
                                    t[index as usize].clone()
                                };
                                self.unify(self.getType(dest), fieldType, instruction.location.clone());
                            }
                            _ => {
                                println!("TupleIndex on non-tuple type: {} {}", receiverType, instruction);
                                TypecheckerError::TypeAnnotationNeeded(instruction.location.clone()).report(self.ctx);
                            }
                        }
                    }
                }
            }
            InstructionKind::BlockStart(_) => {}
            InstructionKind::BlockEnd(_) => {}
            InstructionKind::With(_, blockId) => {
                self.queue.push_back(*blockId);
            }
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
            for (_, block) in &body.blocks {
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
            for (_, block) in &body.blocks {
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
                            for field in &mut fields {
                                field.ty = Some(
                                    field
                                        .ty
                                        .clone()
                                        .expect("field type is missing")
                                        .apply(&self.substitution),
                                );
                            }
                            let kind = InstructionKind::FieldAssign(dest.clone(), root.clone(), fields);
                            builder.replaceInstruction(kind, instruction.location.clone());
                        }
                        if let InstructionKind::AddressOfField(dest, root, fields) = &instruction.kind {
                            let mut fields = fields.clone();
                            for field in &mut fields {
                                field.ty = Some(
                                    field
                                        .ty
                                        .clone()
                                        .expect("field type is missing")
                                        .apply(&self.substitution),
                                );
                            }
                            let kind = InstructionKind::AddressOfField(dest.clone(), root.clone(), fields);
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

        for (_, block) in &mut body.blocks {
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
                                            self.types.insert(newVar.name.to_string(), *inner.clone());
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
