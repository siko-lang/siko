use core::panic;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt::Debug,
    fmt::Display,
    iter::zip,
};

use crate::siko::{
    hir::{
        Apply::Apply,
        BlockBuilder::BlockBuilder,
        BodyBuilder::BodyBuilder,
        ConstraintContext::ConstraintContext,
        Data::{Enum, Struct},
        Function::{BlockId, Function, Parameter},
        ImplementationResolver::ImplementationResolver,
        ImplementationStore::ImplementationStore,
        Instantiation::{instantiateEnum, instantiateImplementation, instantiateStruct, instantiateTypes},
        Instruction::{
            CallInfo, FieldId, FieldInfo, ImplementationReference, ImplicitIndex, Instruction, InstructionKind,
            Mutability, WithContext,
        },
        Program::Program,
        ProtocolMethodSelector::ProtocolMethodSelector,
        Substitution::Substitution,
        Trait::Implementation,
        TraitMethodSelector::TraitMethodSelector,
        Type::{Type, TypeVar},
        TypeVarAllocator::TypeVarAllocator,
        Unification::unify,
        Variable::Variable,
    },
    location::{Location::Location, Report::ReportContext},
    qualifiedname::{
        builtins::{getCloneFnName, getImplicitConvertFnName, getNativePtrCloneName, getNativePtrIsNullName},
        QualifiedName,
    },
    typechecker::{
        ConstraintChecker::ConstraintChecker,
        ConstraintExpander::ConstraintExpander,
        FunctionCallResolver::{FunctionCallResolver, VariableTypeHandler},
    },
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
        let protocolMethodSelector = &program
            .protocolMethodSelectors
            .get(&moduleName)
            .expect("Protocol method selector not found");
        let implementationStore = &program
            .implementationStores
            .get(&moduleName)
            .expect("Implementation store not found");
        let mut typechecker = Typechecker::new(
            ctx,
            &program,
            &traitMethodSelector,
            &protocolMethodSelector,
            implementationStore,
            f,
        );
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

#[derive(Clone)]
struct ReceiverChainEntry {
    source: Variable,
    dest: Variable,
    field: Option<FieldInfo>,
}

impl Display for ReceiverChainEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(field) = &self.field {
            write!(f, "{}.{} => {}", self.source, field.name, self.dest)
        } else {
            write!(f, "{} => {}", self.source, self.dest)
        }
    }
}

impl Debug for ReceiverChainEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

struct CheckFunctionCallResult {
    pub fnType: Type,
    pub fnName: QualifiedName,
    pub implRefs: Vec<ImplementationReference>,
}

pub struct Typechecker<'a> {
    ctx: &'a ReportContext,
    program: &'a Program,
    f: &'a Function,
    traitMethodSelector: &'a TraitMethodSelector,
    protocolMethodSelector: &'a ProtocolMethodSelector,
    implementationStore: &'a ImplementationStore,
    allocator: TypeVarAllocator,
    substitution: Substitution,
    types: BTreeMap<String, Type>,
    selfType: Option<Type>,
    mutables: BTreeMap<String, Mutability>,
    bodyBuilder: BodyBuilder,
    visitedBlocks: BTreeSet<BlockId>,
    queue: VecDeque<BlockId>,
    knownConstraints: ConstraintContext,
    receiverChains: BTreeMap<Variable, ReceiverChainEntry>,
    implResolver: ImplementationResolver<'a>,
}

impl<'a> Typechecker<'a> {
    pub fn new(
        ctx: &'a ReportContext,
        program: &'a Program,
        traitMethodSelector: &'a TraitMethodSelector,
        protocolMethodSelector: &'a ProtocolMethodSelector,
        implementationStore: &'a ImplementationStore,
        f: &'a Function,
    ) -> Typechecker<'a> {
        let allocator = TypeVarAllocator::new();
        let implResolver = ImplementationResolver::new(allocator.clone(), implementationStore, program);
        Typechecker {
            ctx: ctx,
            program: program,
            f: f,
            traitMethodSelector: traitMethodSelector,
            protocolMethodSelector: protocolMethodSelector,
            implementationStore: implementationStore,
            allocator: allocator,
            substitution: Substitution::new(),
            types: BTreeMap::new(),
            selfType: None,
            mutables: BTreeMap::new(),
            bodyBuilder: BodyBuilder::cloneFunction(f),
            visitedBlocks: BTreeSet::new(),
            queue: VecDeque::new(),
            knownConstraints: f.constraintContext.clone(),
            receiverChains: BTreeMap::new(),
            implResolver: implResolver,
        }
    }

    pub fn run(&mut self) -> Function {
        //println!("Typechecking function {}", self.f.name);
        self.initialize();
        self.knownConstraints = self
            .createConstraintExpander(self.knownConstraints.clone())
            .expandKnownConstraints();
        //self.dump(self.f);
        self.check();
        //self.dump(self.f);
        self.generate()
    }

    fn createConstraintExpander(&mut self, constraints: ConstraintContext) -> ConstraintExpander {
        let expander = ConstraintExpander::new(self.program, self.allocator.clone(), constraints);
        expander
    }

    fn initializeVar(&mut self, var: &Variable) {
        match var.getTypeOpt() {
            Some(ty) => {
                self.setType(var, ty.clone());
            }
            None => {
                if let Some(ty) = self.bodyBuilder.getTypeInBody(&var) {
                    self.setType(var, ty.clone());
                } else {
                    let ty = self.allocator.next();
                    self.setType(var, ty.clone());
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
                        InstructionKind::FunctionCall(dest, _) => {
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
                            self.setType(var, Type::Never(false));
                        }
                        InstructionKind::Ref(var, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::PtrOf(var, _) => {
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
                            self.setType(var, Type::Never(false));
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
                        InstructionKind::With(v, _) => {
                            self.setType(v, Type::Never(false));
                        }
                        InstructionKind::ReadImplicit(var, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::WriteImplicit(_, var) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::LoadPtr(dest, _) => {
                            self.initializeVar(dest);
                        }
                        InstructionKind::StorePtr(dest, _) => {
                            self.initializeVar(dest);
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

    fn setType(&mut self, var: &Variable, ty: Type) {
        self.types.insert(var.name.to_string(), ty);
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

    fn unifyVar(&mut self, var: &Variable, ty: Type) {
        self.unify(self.getType(var), ty, var.location.clone());
    }

    fn unifyVars(&mut self, var1: &Variable, var2: &Variable) {
        self.unify(self.getType(var1), self.getType(var2), var1.location.clone());
    }

    fn instantiateEnum(&mut self, e: &Enum, ty: &Type) -> Enum {
        instantiateEnum(&mut self.allocator, e, ty)
    }

    fn instantiateStruct(&mut self, c: &Struct, ty: &Type) -> Struct {
        instantiateStruct(&mut self.allocator, c, ty)
    }

    fn instantiateImplementation(&mut self, impl_def: &crate::siko::hir::Trait::Implementation) -> Implementation {
        instantiateImplementation(&mut self.allocator, impl_def)
    }

    fn updateConverterDestination(&mut self, dest: &Variable, target: &Type) {
        let destTy = self.getType(dest).apply(&self.substitution);
        let targetTy = target.clone().apply(&self.substitution);
        //println!("Updating converter destination: {} -> {}", destTy, targetTy);
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
            self.setType(dest, targetTy);
        }
    }

    fn checkFunctionCall(
        &mut self,
        targetFn: &Function,
        args: &Vec<Variable>,
        resultVar: &Variable,
        isProtocolMethod: bool,
    ) -> CheckFunctionCallResult {
        // println!(
        //     "Checking function call: {} {} {} args {:?}, result {}",
        //     targetFn.name, fnType, targetFn.constraintContext, args, resultVar
        // );
        let mut useProtocol = isProtocolMethod;
        for c in &targetFn.constraintContext.constraints {
            if self.program.isProtocol(&c.name) {
                useProtocol = true;
            }
        }
        if useProtocol {
            if targetFn.kind.isProtocolCall() {
                let checkResult = self.checkFunctionCall_new(targetFn, args, resultVar, resultVar.location.clone());
                let f = self
                    .program
                    .getFunction(&checkResult.fnName)
                    .expect("Function not found");
                let checkResult = self.checkFunctionCall_new(&f, args, resultVar, resultVar.location.clone());
                checkResult
            } else {
                let checkResult = self.checkFunctionCall_new(targetFn, args, resultVar, resultVar.location.clone());
                checkResult
            }
        } else {
            let fnType =
                self.checkFunctionCall_legacy(args, resultVar, targetFn.getType(), &targetFn.constraintContext);
            let checkResult = CheckFunctionCallResult {
                fnType: fnType,
                fnName: targetFn.name.clone(),
                implRefs: Vec::new(),
            };
            checkResult
        }
    }

    fn checkFunctionCall_new(
        &mut self,
        f: &Function,
        args: &Vec<Variable>,
        resultVar: &Variable,
        location: Location,
    ) -> CheckFunctionCallResult {
        let (result, sub) = FunctionCallResolver::new(
            self.program,
            self.allocator.clone(),
            self.ctx,
            self.implementationStore,
            self.knownConstraints.clone(),
            self.substitution.clone(),
            self,
        )
        .resolve(f, args, resultVar, location);
        self.substitution = sub;
        CheckFunctionCallResult {
            fnType: result.fnType,
            fnName: result.fnName,
            implRefs: result.implRefs,
        }
    }

    fn checkFunctionCall_legacy(
        &mut self,
        args: &Vec<Variable>,
        resultVar: &Variable,
        fnType: Type,
        neededConstraints: &ConstraintContext,
    ) -> Type {
        // println!(
        //     "checkFunctionCall(legacy): {} {} {}",
        //     fnType, neededConstraints, self.knownConstraints
        // );
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
        // println!(
        //     "self.getType(resultVar) {}",
        //     self.getType(resultVar).apply(&self.substitution)
        // );
        let fnResult = fnResult.clone().apply(&self.substitution);
        //println!("fnResult {}", fnResult);
        self.unifyVar(resultVar, fnResult.clone());
        loop {
            for (arg, fnArg) in zip(args, &fnArgs) {
                //println!("arg {} fnArg {}", arg, fnArg);
                let fnArg2 = fnArg.clone().apply(&self.substitution);
                //println!("fnArg2 {}", fnArg2);
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
                    err.report(self.ctx);
                }
            }
        }
        // println!("fnResult {}", fnResult);
        // println!(
        //     "self.getType(resultVar) {}",
        //     self.getType(resultVar).apply(&self.substitution)
        // );
        let fnResult = fnResult.apply(&self.substitution);
        //println!("fnResult {}", fnResult);
        self.unifyVar(resultVar, fnResult);
        // println!(
        //     "self.getType(resultVar) {}",
        //     self.getType(resultVar).apply(&self.substitution)
        // );
        instantiatedFnType.apply(&self.substitution)
    }

    fn lookupTraitMethod(
        &mut self,
        receiverType: Type,
        methodName: &String,
        location: Location,
    ) -> (QualifiedName, bool) {
        if let Some(selections) = self.traitMethodSelector.get(methodName) {
            if selections.len() > 1 {
                TypecheckerError::MethodAmbiguous(methodName.clone(), location.clone()).report(self.ctx);
            }
            return (selections[0].method.clone(), false);
        }
        return self.lookupProtocolMethod(receiverType, methodName, location);
    }

    fn lookupProtocolMethod(
        &mut self,
        receiverType: Type,
        methodName: &String,
        location: Location,
    ) -> (QualifiedName, bool) {
        if let Some(selections) = self.protocolMethodSelector.get(methodName) {
            if selections.len() > 1 {
                TypecheckerError::MethodAmbiguous(methodName.clone(), location.clone()).report(self.ctx);
            }
            return (selections[0].method.clone(), true);
        }
        TypecheckerError::MethodNotFound(methodName.clone(), receiverType.to_string(), location.clone())
            .report(self.ctx);
    }

    fn lookupMethod(&mut self, receiverType: Type, methodName: &String, location: Location) -> (QualifiedName, bool) {
        match receiverType.unpackRef() {
            Type::Named(name, _) => {
                if let Some(structDef) = self.program.structs.get(&name) {
                    let structDef = self.instantiateStruct(structDef, receiverType.unpackRef());
                    for m in &structDef.methods {
                        if m.name == *methodName {
                            //println!("Added {} {}", dest, m.fullName);
                            return (m.fullName.clone(), false);
                        }
                    }
                    return self.lookupTraitMethod(receiverType, methodName, location);
                } else if let Some(enumDef) = self.program.enums.get(&name) {
                    let enumDef = self.instantiateEnum(enumDef, receiverType.unpackRef());
                    for m in &enumDef.methods {
                        if m.name == *methodName {
                            return (m.fullName.clone(), false);
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
                    return (getNativePtrIsNullName(), false);
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
            InstructionKind::FunctionCall(dest, info) => {
                //println!("FunctionCall {} {} {:?}", dest, name, args);
                let Some(targetFn) = self.program.functions.get(&info.name) else {
                    panic!("Function not found {}", info.name);
                };
                let checkResult = self.checkFunctionCall(&targetFn, &info.args, dest, false);
                let mut info = info.clone();
                info.name = checkResult.fnName;
                info.implementations = checkResult.implRefs;
                let kind = InstructionKind::FunctionCall(dest.clone(), info);
                builder.replaceInstruction(kind, instruction.location.clone());
            }
            InstructionKind::Converter(dest, source) => {
                self.receiverChains.insert(
                    dest.clone(),
                    ReceiverChainEntry {
                        source: source.clone(),
                        dest: dest.clone(),
                        field: None,
                    },
                );
                // println!("Converter {} {} {}", dest, source, instruction.location);
                // println!(
                //     "Converter {} {} {}",
                //     dest,
                //     self.getType(dest).apply(&self.substitution),
                //     self.getType(source).apply(&self.substitution)
                // );
                self.unifyVars(dest, source);
            }
            InstructionKind::MethodCall(dest, receiver, methodName, args) => {
                self.handleMethodCall(&instruction, builder, dest, receiver, methodName, args);
            }
            InstructionKind::DynamicFunctionCall(_, _, _) => {
                unimplemented!("Dynamic function call not yet implemented")
            }
            InstructionKind::Bind(name, rhs, _) => {
                self.unifyVars(name, rhs);
            }
            InstructionKind::Tuple(dest, args) => {
                let mut argTypes = Vec::new();
                for arg in args {
                    argTypes.push(self.getType(arg));
                }
                self.unifyVar(dest, Type::Tuple(argTypes));
            }
            InstructionKind::StringLiteral(dest, _) => {
                self.unifyVar(dest, Type::getStringLiteralType());
            }
            InstructionKind::IntegerLiteral(dest, _) => {
                self.unifyVar(dest, Type::getIntType());
            }
            InstructionKind::CharLiteral(dest, _) => {
                self.unifyVar(dest, Type::getU8Type());
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
                self.unifyVar(dest, Type::Reference(Box::new(arg_type), None));
            }
            InstructionKind::PtrOf(dest, arg) => {
                let arg_type = self.getType(arg);
                self.unifyVar(dest, Type::Ptr(Box::new(arg_type)));
            }
            InstructionKind::DropPath(_) => {
                unreachable!("drop list placeholder in typechecker!")
            }
            InstructionKind::DropMetadata(_) => {
                unreachable!("drop metadata in typechecker!")
            }
            InstructionKind::Drop(_, _) => unreachable!("drop in typechecker!"),
            InstructionKind::Jump(_, id) => {
                self.queue.push_back(*id);
            }
            InstructionKind::Assign(name, rhs) => {
                if self.mutables.get(&name.name.to_string()) == Some(&Mutability::Immutable) {
                    TypecheckerError::ImmutableAssign(instruction.location.clone()).report(self.ctx);
                }
                self.unifyVars(name, rhs);
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
                    let addressOfVar = self
                        .bodyBuilder
                        .createTempValueWithType(instruction.location.clone(), receiverType.clone().asPtr());
                    self.types
                        .insert(addressOfVar.name.to_string(), Type::Ptr(Box::new(receiverType.clone())));
                    let kind = InstructionKind::AddressOfField(addressOfVar.clone(), receiver.clone(), newFields);
                    builder.addInstruction(kind, instruction.location.clone());
                    builder.step();
                    let store = InstructionKind::StorePtr(addressOfVar, rhs.clone());
                    builder.replaceInstruction(store, instruction.location.clone());
                }
                // println!(
                //     "FieldAssign check {} {} {}",
                //     self.getType(rhs).apply(&self.substitution),
                //     receiverType,
                //     instruction.location
                // );
                self.unifyVar(rhs, receiverType);
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
                self.unifyVar(dest, receiverType);
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
                        self.unifyVar(dest, destType);
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
                self.receiverChains.insert(
                    dest.clone(),
                    ReceiverChainEntry {
                        source: receiver.clone(),
                        dest: dest.clone(),
                        field: Some(fields[0].clone()),
                    },
                );
                receiverType = receiverType.apply(&self.substitution);
                if let Type::Reference(innerTy, _) = &receiverType {
                    receiverType = *innerTy.clone();
                }
                if let Type::Ptr(innerTy) = &receiverType {
                    let ptrLoadResultVar = self
                        .bodyBuilder
                        .createTempValueWithType(instruction.location.clone(), *innerTy.clone());
                    self.setType(&ptrLoadResultVar, *innerTy.clone());
                    builder.addInstruction(
                        InstructionKind::LoadPtr(ptrLoadResultVar.clone(), receiver.clone()),
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
                        self.unifyVar(dest, result);
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
                                self.unifyVar(dest, fieldType);
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
            InstructionKind::With(_, info) => {
                let contexts = &info.contexts;
                let blockId = info.blockId;
                for c in contexts {
                    match c {
                        WithContext::EffectHandler(effectHandler) => {
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
                        WithContext::Implicit(handler) => {
                            let implicit = self.program.getImplicit(&handler.implicit).expect("Implicit not found");
                            if implicit.mutable {
                                let mut mutable = false;
                                if let Some(m) = self.mutables.get(&handler.var.name.to_string()) {
                                    mutable = *m == Mutability::ExplicitMutable;
                                }
                                if !mutable {
                                    TypecheckerError::ImmutableImplicitHandler(handler.var.location.clone())
                                        .report(self.ctx);
                                }
                            }
                            self.unifyVar(&handler.var, implicit.ty);
                        }
                    }
                }
                self.queue.push_back(blockId);
            }
            InstructionKind::ReadImplicit(var, name) => {
                let implicitName = match name {
                    ImplicitIndex::Unresolved(name) => name,
                    ImplicitIndex::Resolved(_, _) => panic!("Implicit index already resolved in typechecker!"),
                };
                let implicit = self.program.getImplicit(&implicitName).expect("Implicit not found");
                self.unifyVar(var, implicit.ty);
            }
            InstructionKind::WriteImplicit(index, var) => {
                let implicitName = match index {
                    ImplicitIndex::Unresolved(name) => name,
                    ImplicitIndex::Resolved(_, _) => panic!("Implicit index already resolved in typechecker!"),
                };
                let implicit = self.program.getImplicit(&implicitName).expect("Implicit not found");
                self.unifyVar(var, implicit.ty);
            }
            InstructionKind::LoadPtr(dest, src) => {
                let srcType = self.getType(src);
                let srcType = srcType.apply(&self.substitution);
                if let Type::Ptr(inner) = srcType {
                    self.unifyVar(dest, *inner);
                } else {
                    TypecheckerError::NotAPtr(srcType.to_string(), instruction.location.clone()).report(self.ctx);
                }
            }
            InstructionKind::StorePtr(dest, src) => {
                let destType = self.getType(dest);
                let destType = destType.apply(&self.substitution);
                if let Type::Ptr(inner) = destType {
                    self.unifyVar(src, *inner);
                } else {
                    TypecheckerError::NotAPtr(destType.to_string(), instruction.location.clone()).report(self.ctx);
                }
            }
        }
    }

    fn resolveReceiverChain(&self, receiver: &Variable) -> (Variable, Vec<ReceiverChainEntry>) {
        //println!("Resolving receiver chain for {}", receiver);
        let mut current = receiver.clone();
        let mut chainEntries = Vec::new();
        loop {
            if !current.isTemp() {
                // we only chain tmp variables
                //println!("Current variable is not a temp, returning {}", current);
                return (current, chainEntries);
            }
            if let Some(entry) = self.receiverChains.get(&current) {
                //println!("Found receiver chain entry: {}", entry);
                current = entry.source.clone();
                chainEntries.push(entry.clone());
            } else {
                //println!("No more receiver chain entries found, returning {}", current);
                break;
            }
        }
        (current, chainEntries)
    }

    fn handleMethodCall(
        &mut self,
        instruction: &Instruction,
        builder: &mut BlockBuilder,
        dest: &Variable,
        receiver: &Variable,
        methodName: &String,
        args: &Vec<Variable>,
    ) {
        let receiver = receiver.clone();
        let receiverType = self.getType(&receiver);
        let receiverType = receiverType.apply(&self.substitution);
        //println!("MethodCall {} {} {} {}", dest, receiver, methodName, receiverType);
        let (name, isProtocolMethod) =
            self.lookupMethod(receiverType.clone(), methodName, instruction.location.clone());
        let mut extendedArgs = args.clone();
        extendedArgs.insert(0, receiver.clone());
        let targetFn = self.program.functions.get(&name).expect("Function not found");
        let fnType = targetFn.getType();
        let (origReceiver, chainEntries) = self.resolveReceiverChain(&receiver);
        let mutableCall = self.mutables.get(&origReceiver.name.to_string()) == Some(&Mutability::ExplicitMutable)
            && fnType.getResult().hasSelfType();
        if mutableCall {
            match fnType.getResult() {
                Type::Tuple(_) => {
                    let destType = self.getType(dest).apply(&self.substitution);
                    if !destType.isTypeVar() {
                        //println!("Mutable method call, changing dest type from {}", destType);
                        let tyvar = self.allocator.next();
                        let destType = destType.addSelfType(tyvar);
                        // println!("Mutable method call, changing dest type to {}", destType);
                        // println!("fnType {}", fnType);
                        self.setType(dest, destType);
                    }
                }
                _ => {}
            }
        }
        let checkResult = self.checkFunctionCall(&targetFn, &extendedArgs, dest, isProtocolMethod);
        let mut fnType = checkResult.fnType;
        //println!("METHOD CALL {} => {}", fnType, receiverType);
        if mutableCall {
            fnType = fnType.changeMethodResult();
            let destType = self.getType(dest).apply(&self.substitution);
            if let Type::Tuple(args) = destType {
                let mut args = args.clone();
                args.remove(0);
                if args.len() == 1 {
                    self.setType(dest, args[0].clone());
                } else {
                    self.setType(dest, Type::Tuple(args));
                }
            } else {
                self.setType(dest, Type::getUnitType());
            }
        }
        // println!(
        //     "AFTER METHOD CALL {} => {} type of dest {}",
        //     fnType,
        //     receiverType,
        //     self.getType(dest).apply(&self.substitution)
        // );
        let name = checkResult.fnName;
        let mut newCallInfo = CallInfo::new(name.clone(), extendedArgs.clone());
        newCallInfo.implementations = checkResult.implRefs;
        builder.replaceInstruction(
            InstructionKind::FunctionCall(dest.clone(), newCallInfo.clone()),
            instruction.location.clone(),
        );
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
            self.transformMutableMethodCall(
                instruction.location.clone(),
                builder,
                dest,
                origReceiver,
                baseType,
                selfLessType,
                chainEntries,
                newCallInfo,
            );
        }
    }

    fn transformMutableMethodCall(
        &mut self,
        location: Location,
        builder: &mut BlockBuilder,
        dest: &Variable,
        origReceiver: Variable,
        baseType: Type,
        selfLessType: Type,
        chainEntries: Vec<ReceiverChainEntry>,
        callInfo: CallInfo,
    ) {
        // println!(
        //     "Transforming mutable method call dest: {} {} args: {:?} orig receiver: {}, chain: {:?}",
        //     dest, name, extendedArgs, origReceiver, chainEntries
        // );
        let mut kinds = Vec::new();
        let implicitResult = self
            .bodyBuilder
            .createTempValueWithType(location.clone(), baseType.clone());
        self.setType(&implicitResult, baseType.clone());
        let kind = InstructionKind::FunctionCall(implicitResult.clone(), callInfo);
        builder.replaceInstruction(kind, location.clone());
        builder.step();
        let mut fields = Vec::new();
        for entry in chainEntries {
            if let Some(mut field) = entry.field {
                field.ty = Some(self.getType(&entry.dest));
                fields.push(field);
            }
        }
        let updatedReceiver = match selfLessType.getTupleTypes().len() {
            0 => {
                let kind = InstructionKind::Tuple(dest.clone(), vec![]);
                kinds.push(kind);
                implicitResult.clone()
            }
            1 => {
                let tupleTypes = baseType.getTupleTypes();
                let selfType = tupleTypes[0].clone();
                let implicitSelf = self
                    .bodyBuilder
                    .createTempValueWithType(location.clone(), selfType.clone());
                self.setType(&implicitSelf, selfType.clone());
                let implicitSelfIndex = InstructionKind::FieldRef(
                    implicitSelf.clone(),
                    implicitResult.clone(),
                    vec![FieldInfo {
                        name: FieldId::Indexed(0),
                        ty: Some(selfType.clone()),
                        location: location.clone(),
                    }],
                );
                let destTy = self.getType(dest);
                let resVar = self
                    .bodyBuilder
                    .createTempValueWithType(location.clone(), destTy.clone());
                self.setType(&resVar, destTy.clone());
                let assign = InstructionKind::Assign(dest.clone(), resVar.clone());
                let resIndex = InstructionKind::FieldRef(
                    resVar.clone(),
                    implicitResult.clone(),
                    vec![FieldInfo {
                        name: FieldId::Indexed(1),
                        ty: Some(destTy.clone()),
                        location: location.clone(),
                    }],
                );
                kinds.push(resIndex);
                kinds.push(implicitSelfIndex);
                kinds.push(assign);
                implicitSelf.clone()
            }
            _ => {
                let tupleTypes = baseType.getTupleTypes();
                let selfType = tupleTypes[0].clone();
                let implicitSelf = self
                    .bodyBuilder
                    .createTempValueWithType(location.clone(), selfType.clone());
                self.setType(&implicitSelf, selfType.clone());
                let implicitSelfIndex = InstructionKind::FieldRef(
                    implicitSelf.clone(),
                    implicitResult.clone(),
                    vec![FieldInfo {
                        name: FieldId::Indexed(0),
                        ty: Some(selfType.clone()),
                        location: location.clone(),
                    }],
                );
                let mut args = Vec::new();
                let mut tupleIndices = Vec::new();
                for (argIndex, argType) in tupleTypes.iter().skip(1).enumerate() {
                    let resVar = self
                        .bodyBuilder
                        .createTempValueWithType(location.clone(), argType.clone());
                    args.push(resVar.clone());
                    self.setType(&resVar, argType.clone());
                    let tupleIndexN = InstructionKind::FieldRef(
                        resVar.clone(),
                        implicitResult.clone(),
                        vec![FieldInfo {
                            name: FieldId::Indexed((argIndex + 1) as u32),
                            ty: Some(argType.clone()),
                            location: location.clone(),
                        }],
                    );
                    tupleIndices.push(tupleIndexN);
                }
                let tuple = InstructionKind::Tuple(dest.clone(), args);
                kinds.push(implicitSelfIndex);
                for i in tupleIndices {
                    kinds.push(i);
                }
                kinds.push(tuple);
                implicitSelf.clone()
            }
        };
        if fields.is_empty() {
            let kind = InstructionKind::Assign(origReceiver.clone(), updatedReceiver.clone());
            kinds.push(kind);
        } else {
            fields.reverse();
            let kind = InstructionKind::FieldAssign(origReceiver.clone(), updatedReceiver.clone(), fields.clone());
            kinds.push(kind);
        }
        kinds.reverse();
        for kind in kinds {
            //println!("Adding instruction: {}", kind);
            builder.addInstruction(kind, location.clone());
        }
        // println!("Finished method transformation");
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
                        if let Some(ty) = v.getTypeOpt() {
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
                        Some(v) => match v.getTypeOpt() {
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
                    newVar.setType(ty);
                    instruction.kind = instruction.kind.replaceVar(var, newVar);
                }
            }
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
                            // println!("Processing converter {} : {} -> {}", instruction, sourceTy, destTy);
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
                                            CallInfo::new(getNativePtrCloneName(), vec![source.clone()]),
                                        )
                                    } else {
                                        if self.implResolver.isCopy(destTy) {
                                            InstructionKind::FunctionCall(
                                                dest.clone(),
                                                CallInfo::new(getCloneFnName(), vec![source.clone()]),
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
                                        if self.implResolver.isImplicitConvert(&src, &inner) {
                                            let newVar = self
                                                .bodyBuilder
                                                .createTempValueWithType(instruction.location.clone(), *inner.clone());
                                            self.setType(&newVar, *inner.clone());
                                            let kind = self.addImplicitConvertCall(&newVar, source);
                                            builder.addInstruction(kind, instruction.location.clone());
                                            builder.step();
                                            refSource = newVar;
                                        } else {
                                            TypecheckerError::TypeMismatch(
                                                destTy.to_string(),
                                                sourceTy.to_string(),
                                                instruction.location.clone(),
                                            )
                                            .report(self.ctx);
                                        }
                                    }
                                    let kind = InstructionKind::Ref(dest.clone(), refSource.clone());
                                    builder.replaceInstruction(kind, instruction.location.clone());
                                }
                                (t1, t2) => {
                                    if !self.tryUnify(t1.clone(), t2.clone()) {
                                        if self.implResolver.isImplicitConvert(&t2, &t1) {
                                            let kind = self.addImplicitConvertCall(dest, source);
                                            builder.replaceInstruction(kind, instruction.location.clone());
                                        } else {
                                            TypecheckerError::TypeMismatch(
                                                destTy.to_string(),
                                                sourceTy.to_string(),
                                                instruction.location.clone(),
                                            )
                                            .report(self.ctx);
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
        //println!("Converters processed");
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

    fn addImplicitConvertCall(&mut self, dest: &Variable, source: &Variable) -> InstructionKind {
        let targetFn = self
            .program
            .getFunction(&getImplicitConvertFnName())
            .expect("Implicit convert function not found");
        let result = self.checkFunctionCall(&targetFn, &vec![source.clone()], dest, false);
        let info = CallInfo::new(result.fnName, vec![source.clone()]);
        let kind = InstructionKind::FunctionCall(dest.clone(), info);
        kind
    }
}

impl<'a> VariableTypeHandler for Typechecker<'a> {
    fn getType(&self, var: &Variable) -> Type {
        self.getType(var)
    }

    fn setType(&mut self, var: &Variable, ty: Type) {
        self.setType(var, ty)
    }
}
