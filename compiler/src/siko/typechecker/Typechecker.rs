use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt::{Debug, Display},
};

use crate::siko::{
    hir::{
        Block::BlockId,
        BlockBuilder::BlockBuilder,
        BodyBuilder::BodyBuilder,
        ConstraintContext::ConstraintContext,
        Data::{Enum, Struct},
        Function::{Function, Parameter, ResultKind},
        FunctionCallResolver::{CheckFunctionCallResult, FunctionCallResolver},
        InstanceResolver::InstanceResolver,
        InstanceStore::InstanceStore,
        Instantiation::{instantiateEnum, instantiateInstance, instantiateStruct},
        Instruction::{
            CallInfo, FieldId, FieldInfo, ImplicitIndex, Instruction, InstructionKind, IntegerOp, Mutability,
            WithContext,
        },
        Program::Program,
        Trait::Instance,
        TraitMethodSelector::TraitMethodSelector,
        Type::{formatTypes, Type, TypeVar},
        TypeVarAllocator::TypeVarAllocator,
        Unifier::Unifier,
        Variable::{Variable, VariableName},
    },
    location::{Location::Location, Report::ReportContext},
    qualifiedname::{
        builtins::{
            getImplicitConvertFnName, getIntAddName, getIntBitAndName, getIntBitOrName, getIntBitXorName,
            getIntDivName, getIntEqName, getIntLessThanName, getIntModName, getIntMulName, getIntShiftLeftName,
            getIntShiftRightName, getIntSubName, getNativePtrCloneName, getNativePtrIsNullName, IntKind,
        },
        QualifiedName,
    },
    typechecker::{ClosureSeparator::ClosureSeparator, ConstraintExpander::ConstraintExpander},
    util::Runner::Runner,
};

use super::Error::TypecheckerError;

pub fn typecheck(ctx: &ReportContext, mut program: Program, runner: Runner) -> Program {
    let mut result = BTreeMap::new();
    for (_, f) in &program.functions {
        let moduleName = f.name.module();
        let traitMethodselector = &program
            .traitMethodselectors
            .get(&moduleName)
            .expect("Trait method selector not found");
        let instanceStore = &program
            .instanceStores
            .get(&moduleName)
            .expect("Instance store not found");
        let mut typechecker = Typechecker::new(ctx, &program, &traitMethodselector, instanceStore, f, runner.clone());
        let typedFns = typechecker.run();
        //typedFn.dump();
        for typedFn in typedFns {
            result.insert(typedFn.name.clone(), typedFn);
        }
    }
    program.functions = result;
    program
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

pub struct ClosureTypeInfo {
    pub name: Option<QualifiedName>,
    pub argTypes: Vec<Type>,
    pub envTypes: Vec<Type>,
    pub resultType: Option<Type>,
    pub envVars: Vec<Variable>,
    pub argVars: Vec<Variable>,
}

impl ClosureTypeInfo {
    fn new() -> ClosureTypeInfo {
        ClosureTypeInfo {
            name: None,
            argTypes: Vec::new(),
            envTypes: Vec::new(),
            resultType: None,
            envVars: Vec::new(),
            argVars: Vec::new(),
        }
    }
}

impl Display for ClosureTypeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ClosureTypeInfo(name: {}, argTypes: {}, envTypes: {}, resultType: {}, envVars: {}, argVars: {})",
            if let Some(name) = &self.name {
                name.toString()
            } else {
                "??".to_string()
            },
            formatTypes(&self.argTypes),
            formatTypes(&self.envTypes),
            if let Some(resultType) = &self.resultType {
                resultType.to_string()
            } else {
                "??".to_string()
            },
            self.envVars
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            self.argVars
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

pub struct Typechecker<'a> {
    ctx: &'a ReportContext,
    program: &'a Program,
    f: &'a Function,
    traitMethodselector: &'a TraitMethodSelector,
    instanceStore: &'a InstanceStore,
    allocator: TypeVarAllocator,
    selfType: Option<Type>,
    mutables: BTreeMap<String, Mutability>,
    bodyBuilder: BodyBuilder,
    visitedBlocks: BTreeSet<BlockId>,
    queue: VecDeque<BlockId>,
    knownConstraints: ConstraintContext,
    receiverChains: BTreeMap<Variable, ReceiverChainEntry>,
    implResolver: InstanceResolver<'a>,
    unifier: Unifier,
    fnCallResolver: FunctionCallResolver<'a>,
    closureTypes: BTreeMap<BlockId, ClosureTypeInfo>,
    integerOps: BTreeMap<QualifiedName, IntegerOp>,
    runner: Runner,
}

impl<'a> Typechecker<'a> {
    pub fn new(
        ctx: &'a ReportContext,
        program: &'a Program,
        traitMethodselector: &'a TraitMethodSelector,
        instanceStore: &'a InstanceStore,
        f: &'a Function,
        runner: Runner,
    ) -> Typechecker<'a> {
        let allocator = TypeVarAllocator::new();
        let expander = ConstraintExpander::new(program, allocator.clone(), f.constraintContext.clone());
        let knownConstraints = expander.expandKnownConstraints();
        let implResolver = InstanceResolver::new(allocator.clone(), instanceStore, program, knownConstraints.clone());
        let unifier = Unifier::withContext(ctx, runner.child("unifier"));
        let fnCallResolver = FunctionCallResolver::new(
            program,
            allocator.clone(),
            ctx,
            instanceStore,
            knownConstraints.clone(),
            unifier.clone(),
        );
        let mut integerOps = BTreeMap::new();
        for kind in [
            IntKind::Int,
            IntKind::U8,
            IntKind::U16,
            IntKind::U32,
            IntKind::U64,
            IntKind::I8,
            IntKind::I16,
            IntKind::I32,
            IntKind::I64,
        ] {
            integerOps.insert(getIntAddName(kind), IntegerOp::Add);
            integerOps.insert(getIntSubName(kind), IntegerOp::Sub);
            integerOps.insert(getIntMulName(kind), IntegerOp::Mul);
            integerOps.insert(getIntDivName(kind), IntegerOp::Div);
            integerOps.insert(getIntModName(kind), IntegerOp::Mod);
            integerOps.insert(getIntEqName(kind), IntegerOp::Eq);
            integerOps.insert(getIntLessThanName(kind), IntegerOp::LessThan);
            integerOps.insert(getIntShiftLeftName(kind), IntegerOp::ShiftLeft);
            integerOps.insert(getIntShiftRightName(kind), IntegerOp::ShiftRight);
            integerOps.insert(getIntBitAndName(kind), IntegerOp::BitAnd);
            integerOps.insert(getIntBitOrName(kind), IntegerOp::BitOr);
            integerOps.insert(getIntBitXorName(kind), IntegerOp::BitXor);
        }
        Typechecker {
            ctx: ctx,
            program: program,
            f: f,
            fnCallResolver,
            traitMethodselector: traitMethodselector,
            instanceStore: instanceStore,
            allocator: allocator,
            selfType: None,
            mutables: BTreeMap::new(),
            bodyBuilder: BodyBuilder::cloneFunction(f),
            visitedBlocks: BTreeSet::new(),
            queue: VecDeque::new(),
            knownConstraints: f.constraintContext.clone(),
            receiverChains: BTreeMap::new(),
            implResolver: implResolver,
            unifier: unifier,
            closureTypes: BTreeMap::new(),
            integerOps,
            runner,
        }
    }

    pub fn run(&mut self) -> Vec<Function> {
        //println!("Typechecking function {}", self.f.name);
        //println!(" {} ", self.f);
        let initializeRunner = self.runner.child("initialize");
        initializeRunner.run(|| self.initialize());
        //self.dump(self.f);
        let checkRunner = self.runner.child("check");
        checkRunner.run(|| self.check());
        //self.dump(self.f);
        let generateRunner = self.runner.child("generate");
        let fs = generateRunner.run(|| self.generate());
        fs
    }

    fn initializeVar(&mut self, var: &Variable) {
        //println!("Initializing var {}", var);
        match var.getTypeOpt() {
            Some(ty) => {
                var.setType(ty.clone());
            }
            None => {
                let ty = self.allocator.next();
                var.setType(ty.clone());
            }
        }
        match var.name() {
            VariableName::LambdaArg(blockId, index) => {
                let closureTypeInfo = self.closureTypes.entry(blockId).or_insert_with(ClosureTypeInfo::new);
                assert_eq!(closureTypeInfo.argVars.len(), index as usize);
                closureTypeInfo.argVars.push(var.clone());
            }
            VariableName::ClosureArg(blockId, index) => {
                //println!("Initializing closure arg {} in block {} {}", var.name(), blockId, index);
                let closureTypeInfo = self.closureTypes.entry(blockId).or_insert_with(ClosureTypeInfo::new);
                //println!("Current closure types: {}", closureTypeInfo);
                assert_eq!(closureTypeInfo.envVars.len(), index as usize);
                closureTypeInfo.envVars.push(var.clone());
            }
            _ => {}
        }
    }

    pub fn initialize(&mut self) {
        //println!("Initializing {}", self.f.name);
        for param in &self.f.params {
            match &param {
                Parameter::Named(name, _, mutable) => {
                    if *mutable {
                        self.mutables.insert(name.clone(), Mutability::ExplicitMutable);
                    }
                }
                Parameter::SelfParam(mutable, ty) => {
                    let name = format!("self");
                    self.selfType = Some(ty.clone());
                    if *mutable {
                        self.mutables.insert(name, Mutability::ExplicitMutable);
                    }
                }
            }
        }
        if let Some(body) = &self.f.body {
            for (_, block) in &body.blocks {
                for instruction in &block.getInstructions() {
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
                        InstructionKind::Bind(var, rhs, mutable) => {
                            self.initializeVar(var);
                            self.initializeVar(rhs); // workaround to initialize lambda args, those just appear out of the blue
                            if *mutable {
                                self.mutables
                                    .insert(var.name().to_string(), Mutability::ExplicitMutable);
                            } else {
                                self.mutables.insert(var.name().to_string(), Mutability::Immutable);
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
                            var.setType(Type::Never(false));
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
                            var.setType(Type::Never(false));
                        }
                        InstructionKind::Assign(_, _) => {}
                        InstructionKind::FieldAssign(_, _, _) => {}
                        InstructionKind::AddressOfField(var, _, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::DeclareVar(var, mutability) => {
                            self.initializeVar(var);
                            self.mutables.insert(var.name().to_string(), mutability.clone());
                        }
                        InstructionKind::Transform(var, _, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::EnumSwitch(_, _) => {}
                        InstructionKind::IntegerSwitch(_, _) => {}
                        InstructionKind::BlockStart(_) => {}
                        InstructionKind::BlockEnd(_) => {}
                        InstructionKind::With(v, _) => {
                            v.setType(Type::Never(false));
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
                        InstructionKind::CreateClosure(var, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::ClosureReturn(_, var, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::IntegerOp(var, _, _, _) => {
                            self.initializeVar(var);
                        }
                        InstructionKind::Yield(v, _) => {
                            self.initializeVar(v);
                        }
                    }
                }
            }
            for (_, block) in &body.blocks {
                for instruction in &block.getInstructions() {
                    match &instruction.kind {
                        InstructionKind::CreateClosure(_, info) => {
                            let closureTypeInfo =
                                self.closureTypes.entry(info.body).or_insert_with(ClosureTypeInfo::new);
                            closureTypeInfo.name = Some(info.name.clone());
                            let mut argTypes = Vec::new();
                            for _ in 0..info.fnArgCount {
                                argTypes.push(self.allocator.next());
                            }
                            closureTypeInfo.argTypes = argTypes;
                            closureTypeInfo.envTypes = info.closureParams.iter().map(|p| p.getType()).collect();
                            closureTypeInfo.resultType = Some(self.allocator.next());
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn instantiateEnum(&mut self, e: &Enum, ty: &Type) -> Enum {
        instantiateEnum(&mut self.allocator, e, ty)
    }

    fn instantiateStruct(&mut self, c: &Struct, ty: &Type) -> Struct {
        instantiateStruct(&mut self.allocator, c, ty)
    }

    fn instantiateInstance(&mut self, impl_def: &Instance) -> Instance {
        instantiateInstance(&mut self.allocator, impl_def)
    }

    fn checkFunctionCall(
        &mut self,
        targetFn: &Function,
        args: &Vec<Variable>,
        resultVar: &Variable,
        runner: Runner,
    ) -> CheckFunctionCallResult {
        // println!(
        //     "Checking function call: {} {} {} args {:?}, result {}",
        //     targetFn.name,
        //     targetFn.getType(),
        //     targetFn.constraintContext,
        //     args,
        //     resultVar
        // );
        if targetFn.kind.isTraitCall() {
            let fnTraitResolverRunner = runner.child("fn_trait_call_resolver");
            let checkResult = fnTraitResolverRunner.run(|| {
                let checkResult = self.fnCallResolver.resolve(
                    targetFn,
                    args,
                    resultVar,
                    resultVar.location().clone(),
                    fnTraitResolverRunner.clone(),
                );
                let f = self
                    .program
                    .getFunction(&checkResult.fnName)
                    .expect("Function not found");
                let checkResult = self.fnCallResolver.resolve(
                    &f,
                    args,
                    resultVar,
                    resultVar.location().clone(),
                    fnTraitResolverRunner.clone(),
                );
                checkResult
            });
            checkResult
        } else {
            let fnResolverRunner = runner.child("fn_call_resolver");
            let checkResult = fnResolverRunner.run(|| {
                self.fnCallResolver.resolve(
                    targetFn,
                    args,
                    resultVar,
                    resultVar.location().clone(),
                    fnResolverRunner.clone(),
                )
            });
            checkResult
        }
    }

    fn lookupTraitMethod(&mut self, receiverType: Type, methodName: &String, location: Location) -> QualifiedName {
        if let Some(selections) = self.traitMethodselector.get(methodName) {
            if selections.len() > 1 {
                TypecheckerError::MethodAmbiguous(methodName.clone(), location.clone()).report(self.ctx);
            }
            return selections[0].method.clone();
        }
        TypecheckerError::MethodNotFound(methodName.clone(), receiverType.to_string(), location.clone())
            .report(self.ctx);
    }

    fn lookupMethod(&mut self, receiverType: Type, methodName: &String, location: Location) -> QualifiedName {
        let baseType = receiverType.clone().unpackRef();
        match baseType.clone() {
            Type::Named(name, _) => {
                if let Some(structDef) = self.program.structs.get(&name) {
                    let structDef = self.instantiateStruct(structDef, &baseType);
                    for m in &structDef.methods {
                        if m.name == *methodName {
                            //println!("Added {} {}", dest, m.fullName);
                            return m.fullName.clone();
                        }
                    }
                    return self.lookupTraitMethod(receiverType, methodName, location);
                } else if let Some(enumDef) = self.program.enums.get(&name) {
                    let enumDef = self.instantiateEnum(enumDef, &baseType);
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
            Type::Coroutine(_, _) => {
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
        receiverType = self.unifier.apply(receiverType);
        if let Type::Reference(_) = &receiverType {
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
                let receiverType = self.unifier.apply(receiverType);
                match receiverType.clone().unpackRef() {
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
                            return types[*index as usize].asRef();
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
        let receiverType = self.unifier.apply(receiverType);
        let baseType = receiverType.clone().unpackRef();
        match baseType.clone() {
            Type::Named(name, _) => {
                if let Some(structDef) = self.program.structs.get(&name) {
                    let structDef = self.instantiateStruct(structDef, &baseType);
                    for f in &structDef.fields {
                        if f.name == *fieldName {
                            let mut result = f.ty.clone();
                            if receiverType.isReference() {
                                result = f.ty.asRef();
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
                    let checkInstructionRunner = self
                        .runner
                        .child(&format!("check_instruction.{}", instruction.kind.getShortName()));
                    checkInstructionRunner
                        .clone()
                        .run(|| self.checkInstruction(instruction, &mut builder, checkInstructionRunner));
                    builder.step();
                }
                None => {
                    break;
                }
            }
        }
    }

    fn checkInstruction(&mut self, instruction: Instruction, builder: &mut BlockBuilder, runner: Runner) {
        //println!("checkInstruction {}", instruction);
        match &instruction.kind {
            InstructionKind::FunctionCall(dest, info) => {
                //println!("FunctionCall {} {} {:?}", dest, name, args);
                let Some(targetFn) = self.program.functions.get(&info.name) else {
                    panic!("Function not found {}", info.name);
                };
                let checkResult = self.checkFunctionCall(&targetFn, &info.args, dest, runner);
                let mut info = info.clone();
                info.name = checkResult.fnName;
                info.instanceRefs = checkResult.instanceRefs;
                self.postProcessFunctionCall(builder, dest, info, instruction.location.clone());
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
                let srcTy = self.unifier.apply(source.getType());
                let destTy = self.unifier.apply(dest.getType());
                match (srcTy, destTy) {
                    (Type::Var(TypeVar::Var(_)), Type::Var(TypeVar::Var(_))) => {}
                    (_, Type::Var(TypeVar::Var(_))) => {
                        dest.setType(source.getType());
                    }
                    _ => {
                        self.unifier.unifyVars(dest, source);
                    }
                }
            }
            InstructionKind::MethodCall(dest, receiver, methodName, args) => {
                self.handleMethodCall(&instruction, builder, dest, receiver, methodName, args, runner);
            }
            InstructionKind::DynamicFunctionCall(dest, closure, args) => {
                let argTypes = args.iter().map(|arg| arg.getType()).collect::<Vec<_>>();
                let destType = dest.getType();
                let closureType = Type::Function(argTypes, Box::new(destType));
                self.unifier.unifyVar(closure, closureType);
            }
            InstructionKind::Bind(name, rhs, _) => {
                self.unifier.unifyVars(name, rhs);
            }
            InstructionKind::Tuple(dest, args) => {
                let mut argTypes = Vec::new();
                for arg in args {
                    argTypes.push(arg.getType());
                }
                self.unifier.unifyVar(dest, Type::Tuple(argTypes));
            }
            InstructionKind::StringLiteral(dest, _) => {
                self.unifier.unifyVar(dest, Type::getStringLiteralType());
            }
            InstructionKind::IntegerLiteral(dest, _) => {
                self.unifier.unifyVar(dest, Type::getIntType());
            }
            InstructionKind::CharLiteral(dest, _) => {
                self.unifier.unifyVar(dest, Type::getU8Type());
            }
            InstructionKind::Return(_, arg) => {
                let mut result = self.f.result.getReturnType();
                if let Some(selfType) = self.selfType.clone() {
                    result = result.changeSelfType(selfType);
                }
                self.unifier.updateConverterDestination(arg, &result);
            }
            InstructionKind::Ref(dest, arg) => {
                let arg_type = arg.getType();
                self.unifier.unifyVar(dest, arg_type.asRef());
            }
            InstructionKind::PtrOf(dest, arg) => {
                let arg_type = arg.getType();
                self.unifier.unifyVar(dest, arg_type.asPtr());
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
            InstructionKind::Assign(dest, src) => {
                if self.mutables.get(&dest.name().to_string()) == Some(&Mutability::Immutable) {
                    TypecheckerError::ImmutableAssign(instruction.location.clone()).report(self.ctx);
                }
                self.unifier.unifyVars(dest, src);
            }
            InstructionKind::FieldAssign(receiver, rhs, fields) => {
                if self.mutables.get(&receiver.name().to_string()) == Some(&Mutability::Immutable) {
                    TypecheckerError::ImmutableAssign(instruction.location.clone()).report(self.ctx);
                }
                let receiverType = receiver.getType();
                let mut receiverType = self.unifier.apply(receiverType);
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
                    let kind = InstructionKind::AddressOfField(addressOfVar.clone(), receiver.clone(), newFields);
                    builder.addInstruction(kind, instruction.location.clone());
                    builder.step();
                    let store = InstructionKind::StorePtr(addressOfVar, rhs.clone());
                    builder.replaceInstruction(store, instruction.location.clone());
                }
                // println!(
                //     "FieldAssign check {} {} {}",
                //     rhs.getType().apply(&self.substitution),
                //     receiverType,
                //     instruction.location
                // );
                self.unifier.unifyVar(rhs, receiverType);
            }
            InstructionKind::AddressOfField(dest, receiver, fields) => {
                let receiverType = receiver.getType();
                let mut receiverType = self.unifier.apply(receiverType);
                let mut newFields = Vec::new();
                for field in fields {
                    let fieldTy = self.checkField(receiverType, &field.name, field.location.clone());
                    let mut newField = field.clone();
                    newField.ty = Some(fieldTy.clone());
                    newFields.push(newField);
                    receiverType = fieldTy;
                }
                receiverType = receiverType.asRef();
                let newKind = InstructionKind::AddressOfField(dest.clone(), receiver.clone(), newFields);
                builder.replaceInstruction(newKind, instruction.location.clone());
                self.unifier.unifyVar(dest, receiverType);
            }
            InstructionKind::DeclareVar(_, _) => {}
            InstructionKind::Transform(dest, root, info) => {
                let rootTy = root.getType();
                let rootTy = self.unifier.apply(rootTy);
                let isRef = rootTy.isReference();
                let baseTy = rootTy.clone().unpackRef();
                match baseTy.getName() {
                    Some(name) => {
                        let e = self.program.enums.get(&name).expect("not an enum in transform!");
                        let e = self.instantiateEnum(e, &baseTy);
                        let v = &e.variants[info.variantIndex as usize];
                        let destType = if isRef {
                            Type::Tuple(v.items.clone()).asRef()
                        } else {
                            Type::Tuple(v.items.clone())
                        };
                        self.unifier.unifyVar(dest, destType);
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
                let mut receiverType = receiver.getType();
                assert_eq!(fields.len(), 1, "FieldRef with multiple fields in typecheck!");
                self.receiverChains.insert(
                    dest.clone(),
                    ReceiverChainEntry {
                        source: receiver.clone(),
                        dest: dest.clone(),
                        field: Some(fields[0].clone()),
                    },
                );
                receiverType = self.unifier.apply(receiverType);
                if let Type::Reference(innerTy) = &receiverType {
                    receiverType = *innerTy.clone();
                }
                if let Type::Ptr(innerTy) = &receiverType {
                    let ptrLoadResultVar = self
                        .bodyBuilder
                        .createTempValueWithType(instruction.location.clone(), *innerTy.clone());
                    ptrLoadResultVar.setNoDrop();
                    ptrLoadResultVar.setType(*innerTy.clone());
                    builder.addInstruction(
                        InstructionKind::LoadPtr(ptrLoadResultVar.clone(), receiver.clone()),
                        instruction.location.clone(),
                    );
                    builder.step();
                    let kind = instruction.kind.replaceVar(receiver.clone(), ptrLoadResultVar.clone());
                    builder.replaceInstruction(kind, instruction.location.clone());
                    receiverType = *innerTy.clone();
                } else {
                    receiverType = receiver.getType();
                }
                let fieldName = fields[0].name.clone();
                match fieldName {
                    FieldId::Named(n) => {
                        let result = self.readField(receiverType, n, instruction.location.clone());
                        self.unifier.unifyVar(dest, result);
                    }
                    FieldId::Indexed(index) => {
                        receiverType = self.unifier.apply(receiverType);
                        let isRef = receiverType.isReference();
                        match receiverType.clone().unpackRef() {
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
                                    t[index as usize].asRef()
                                } else {
                                    t[index as usize].clone()
                                };
                                self.unifier.unifyVar(dest, fieldType);
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
                            self.unifier
                                .unify(methodType, handlerType, effectHandler.location.clone());
                        }
                        WithContext::Implicit(handler) => {
                            let implicit = self.program.getImplicit(&handler.implicit).expect("Implicit not found");
                            if implicit.mutable {
                                let mut mutable = false;
                                if let Some(m) = self.mutables.get(&handler.var.name().to_string()) {
                                    mutable = *m == Mutability::ExplicitMutable;
                                }
                                if !mutable {
                                    TypecheckerError::ImmutableImplicitHandler(handler.var.location().clone())
                                        .report(self.ctx);
                                }
                            }
                            self.unifier.unifyVar(&handler.var, implicit.ty);
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
                self.unifier.unifyVar(var, implicit.ty);
            }
            InstructionKind::WriteImplicit(index, var) => {
                let implicitName = match index {
                    ImplicitIndex::Unresolved(name) => name,
                    ImplicitIndex::Resolved(_, _) => panic!("Implicit index already resolved in typechecker!"),
                };
                let implicit = self.program.getImplicit(&implicitName).expect("Implicit not found");
                self.unifier.unifyVar(var, implicit.ty);
            }
            InstructionKind::LoadPtr(dest, src) => {
                let srcType = src.getType();
                let srcType = self.unifier.apply(srcType);
                if let Type::Ptr(inner) = srcType {
                    self.unifier.unifyVar(dest, *inner);
                } else {
                    TypecheckerError::NotAPtr(srcType.to_string(), instruction.location.clone()).report(self.ctx);
                }
            }
            InstructionKind::StorePtr(dest, src) => {
                let destType = dest.getType();
                let destType = self.unifier.apply(destType);
                if let Type::Ptr(inner) = destType {
                    self.unifier.unifyVar(src, *inner);
                } else {
                    TypecheckerError::NotAPtr(destType.to_string(), instruction.location.clone()).report(self.ctx);
                }
            }
            InstructionKind::CreateClosure(dest, info) => {
                let closureTypeInfo = self.closureTypes.get(&info.body).expect("Closure type info not found");
                let fnType = Type::Function(
                    closureTypeInfo.argTypes.clone(),
                    Box::new(
                        closureTypeInfo
                            .resultType
                            .clone()
                            .expect("closure result type not found"),
                    ),
                );
                for (index, envVar) in closureTypeInfo.envVars.iter().enumerate() {
                    self.unifier.unifyVar(envVar, closureTypeInfo.envTypes[index].clone());
                }
                for (index, argVar) in closureTypeInfo.argVars.iter().enumerate() {
                    self.unifier.unifyVar(argVar, closureTypeInfo.argTypes[index].clone());
                }
                self.unifier.unifyVar(dest, fnType);
                self.queue.push_back(info.body);
            }
            InstructionKind::ClosureReturn(blockId, _, var) => {
                let closureTypeInfo = self.closureTypes.get(&blockId).expect("Closure type info not found");
                self.unifier.unifyVar(
                    var,
                    closureTypeInfo
                        .resultType
                        .clone()
                        .expect("closure result type not found"),
                );
            }
            InstructionKind::IntegerOp(dest, left, right, op) => {
                self.unifier.unifyVars(left, right);
                match op {
                    IntegerOp::Add | IntegerOp::Sub | IntegerOp::Mul | IntegerOp::Div | IntegerOp::Mod => {
                        self.unifier.unifyVar(dest, Type::getIntType());
                    }
                    IntegerOp::ShiftLeft | IntegerOp::ShiftRight => {
                        self.unifier.unifyVar(dest, left.getType());
                    }
                    IntegerOp::BitAnd | IntegerOp::BitOr | IntegerOp::BitXor => {
                        self.unifier.unifyVar(dest, left.getType());
                    }
                    IntegerOp::Eq | IntegerOp::LessThan => {
                        self.unifier.unifyVar(dest, Type::getBoolType());
                    }
                }
            }
            InstructionKind::Yield(resumeValue, yieldedValue) => {
                if let Some((yieldType, _)) = self.f.getType().getResult().unpackCoroutine() {
                    self.unifier.unifyVar(yieldedValue, yieldType);
                    self.unifier.unifyVar(resumeValue, Type::getUnitType());
                } else {
                    TypecheckerError::YieldOutsideCoroutine(self.f.name.toString(), instruction.location.clone())
                        .report(self.ctx);
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
        runner: Runner,
    ) {
        let receiver = receiver.clone();
        let receiverType = receiver.getType();
        let receiverType = self.unifier.apply(receiverType);
        //println!("MethodCall {} {} {} {}", dest, receiver, methodName, receiverType);
        let name = self.lookupMethod(receiverType.clone(), methodName, instruction.location.clone());
        let mut extendedArgs = args.clone();
        extendedArgs.insert(0, receiver.clone());
        let targetFn = self.program.functions.get(&name).expect("Function not found");
        let fnType = targetFn.getType();
        let (origReceiver, chainEntries) = self.resolveReceiverChain(&receiver);
        let mutableCall = self.mutables.get(&origReceiver.name().to_string()) == Some(&Mutability::ExplicitMutable)
            && fnType.getResult().hasSelfType();
        if mutableCall {
            match fnType.getResult() {
                Type::Tuple(_) => {
                    let destType = self.unifier.apply(dest.getType());
                    if !destType.isTypeVar() {
                        //println!("Mutable method call, changing dest type from {}", destType);
                        let tyvar = self.allocator.next();
                        let destType = destType.addSelfType(tyvar);
                        // println!("Mutable method call, changing dest type to {}", destType);
                        // println!("fnType {}", fnType);
                        dest.setType(destType);
                    }
                }
                _ => {}
            }
        }
        let checkResult = self.checkFunctionCall(&targetFn, &extendedArgs, dest, runner);
        let mut fnType = checkResult.fnType;
        //println!("METHOD CALL {} => {}", fnType, receiverType);
        if mutableCall {
            fnType = fnType.changeMethodResult();
            let destType = self.unifier.apply(dest.getType());
            if let Type::Tuple(args) = destType {
                let mut args = args.clone();
                args.remove(0);
                if args.len() == 1 {
                    dest.setType(args[0].clone());
                } else {
                    dest.setType(Type::Tuple(args));
                }
            } else {
                dest.setType(Type::getUnitType());
            }
        }
        // println!(
        //     "AFTER METHOD CALL {} => {} type of dest {}",
        //     fnType,
        //     receiverType,
        //     dest.getType().apply(&self.substitution)
        // );
        let name = checkResult.fnName;
        let mut newCallInfo = CallInfo::new(name.clone(), extendedArgs.clone());
        newCallInfo.instanceRefs = checkResult.instanceRefs;
        self.postProcessFunctionCall(builder, dest, newCallInfo.clone(), instruction.location.clone());
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
        implicitResult.setType(baseType.clone());
        let kind = InstructionKind::FunctionCall(implicitResult.clone(), callInfo);
        builder.replaceInstruction(kind, location.clone());
        builder.step();
        let mut fields = Vec::new();
        for entry in chainEntries {
            if let Some(mut field) = entry.field {
                field.ty = Some(entry.dest.getType());
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
                implicitSelf.setType(selfType.clone());
                let implicitSelfIndex = InstructionKind::FieldRef(
                    implicitSelf.clone(),
                    implicitResult.clone(),
                    vec![FieldInfo {
                        name: FieldId::Indexed(0),
                        ty: Some(selfType.clone()),
                        location: location.clone(),
                    }],
                );
                let destTy = dest.getType();
                let resVar = self
                    .bodyBuilder
                    .createTempValueWithType(location.clone(), destTy.clone());
                resVar.setType(destTy.clone());
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
                implicitSelf.setType(selfType.clone());
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
                    resVar.setType(argType.clone());
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
                for instruction in &block.getInstructions() {
                    let vars = instruction.kind.collectVariables();
                    for v in vars {
                        if let Some(ty) = v.getTypeOpt() {
                            let vars = ty.collectVars(BTreeSet::new());
                            if !publicVars.is_superset(&vars) {
                                self.dump(f);
                                println!("MISSING: {} {}", instruction, ty);
                                TypecheckerError::TypeAnnotationNeeded(v.location().clone()).report(self.ctx);
                            }
                        } else {
                            TypecheckerError::TypeAnnotationNeeded(v.location().clone()).report(self.ctx);
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
                println!("{}:", block.getId());
                for instruction in &block.getInstructions() {
                    match instruction.kind.getResultVar() {
                        Some(v) => match v.getTypeOpt() {
                            Some(ty) => {
                                println!("  {} : {}", instruction, ty);
                            }
                            None => {
                                let ty = v.getType();
                                let ty = self.unifier.apply(ty);
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
                                let ty = field.ty.clone().expect("field type is missing");
                                field.ty = Some(self.unifier.apply(ty));
                            }
                            let kind = InstructionKind::FieldAssign(dest.clone(), root.clone(), fields);
                            builder.replaceInstruction(kind, instruction.location.clone());
                        }
                        if let InstructionKind::AddressOfField(dest, root, fields) = &instruction.kind {
                            let mut fields = fields.clone();
                            for field in &mut fields {
                                let ty = field.ty.clone().expect("field type is missing");
                                field.ty = Some(self.unifier.apply(ty));
                            }
                            let kind = InstructionKind::AddressOfField(dest.clone(), root.clone(), fields);
                            builder.replaceInstruction(kind, instruction.location.clone());
                        }
                        if let InstructionKind::FieldRef(dest, root, fields) = &instruction.kind {
                            assert_eq!(fields.len(), 1, "FieldRef with multiple fields in typecheck!");
                            let mut fields = fields.clone();
                            let destTy = self.unifier.apply(dest.getType());
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
            let inner = block.getInner();
            for instruction in &mut inner.borrow_mut().instructions {
                let vars = instruction.kind.collectVariables();
                for var in vars {
                    let ty = var.getType();
                    let ty = self.unifier.apply(ty);
                    let newVar = var.clone();
                    newVar.setType(ty);
                    instruction.kind = instruction.kind.replaceVar(var, newVar);
                }
            }
        }
    }

    fn processConverters(&mut self) {
        let processConvertersRunner = self.runner.child("process_converters");
        //println!("processConverters {}", self.f.name);
        let allblocksIds = self.bodyBuilder.getAllBlockIds();
        for blockId in allblocksIds {
            let mut builder = self.bodyBuilder.iterator(blockId);
            loop {
                match builder.getInstruction() {
                    Some(instruction) => {
                        if let InstructionKind::Converter(dest, source) = &instruction.kind {
                            let destTy = self.unifier.apply(dest.getType());
                            let sourceTy = self.unifier.apply(source.getType());
                            //println!("Processing converter {} : {} -> {}", instruction, sourceTy, destTy);
                            match (&destTy, &sourceTy) {
                                (Type::Reference(inner), Type::Reference(src)) => {
                                    self.unifier
                                        .unify(*inner.clone(), *src.clone(), instruction.location.clone());
                                    builder.addDeclare(dest.clone(), instruction.location.clone());
                                    builder.step();
                                    let kind = InstructionKind::Assign(dest.clone(), source.clone());
                                    builder.replaceInstruction(kind, instruction.location.clone());
                                }
                                (destTy, Type::Reference(sourceInner)) => {
                                    self.unifier.unify(
                                        destTy.clone(),
                                        *sourceInner.clone(),
                                        instruction.location.clone(),
                                    );
                                    let kind = if destTy.isPtr() {
                                        InstructionKind::FunctionCall(
                                            dest.clone(),
                                            CallInfo::new(getNativePtrCloneName(), vec![source.clone()]),
                                        )
                                    } else {
                                        if self
                                            .implResolver
                                            .isCopy(destTy, processConvertersRunner.child("is_copy"))
                                        {
                                            let (fnName, instanceRefs) = self.fnCallResolver.resolveCloneCall(
                                                source.clone(),
                                                dest.clone(),
                                                processConvertersRunner.clone(),
                                            );
                                            let mut info = CallInfo::new(fnName, vec![source.clone()]);
                                            info.instanceRefs.extend(instanceRefs);
                                            InstructionKind::FunctionCall(dest.clone(), info)
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
                                (Type::Reference(inner), src) => {
                                    let mut refSource = source.clone();
                                    if !self.unifier.tryUnify(*inner.clone(), src.clone()) {
                                        // check implicit conversion is implemented for these types
                                        if self.implResolver.isImplicitConvert(
                                            &src,
                                            &inner,
                                            processConvertersRunner.child("is_implicit_convert"),
                                        ) {
                                            let newVar = self
                                                .bodyBuilder
                                                .createTempValueWithType(instruction.location.clone(), *inner.clone());
                                            newVar.setType(*inner.clone());
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
                                    if !self.unifier.tryUnify(t1.clone(), t2.clone()) {
                                        if self.implResolver.isImplicitConvert(
                                            &t2,
                                            &t1,
                                            processConvertersRunner.child("is_implicit_convert"),
                                        ) {
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
                            //println!("Adding declare for {}", dest);
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

    pub fn generate(&mut self) -> Vec<Function> {
        //println!("Generating {}", self.f.name);
        if self.f.body.is_none() {
            return vec![self.f.clone()];
        }

        self.processConverters();
        self.addFieldTypes();
        self.removeBinds();

        let mut resultFn = self.f.clone();
        resultFn.body = Some(self.bodyBuilder.build());
        if let Some(selfType) = self.selfType.clone() {
            resultFn.result = match resultFn.result {
                ResultKind::SingleReturn(ty) => ResultKind::SingleReturn(ty.changeSelfType(selfType)),
                ResultKind::Coroutine(ty) => ResultKind::Coroutine(ty),
            }
        }

        self.addTypes(&mut resultFn);
        //self.dump(&result);
        self.verify(&resultFn);

        let mut functions = Vec::new();

        for (blockId, closure) in self.closureTypes.iter() {
            let mut separator = ClosureSeparator::new(&mut resultFn, *blockId, closure, &mut self.unifier);
            let closureFn = separator.process();
            functions.push(closureFn);
        }

        functions.push(resultFn);
        functions
    }

    fn addImplicitConvertCall(&mut self, dest: &Variable, source: &Variable) -> InstructionKind {
        let implicitConvertRunner = self.runner.child("implicit_convert");
        let targetFn = self
            .program
            .getFunction(&getImplicitConvertFnName())
            .expect("Implicit convert function not found");
        let result = self.fnCallResolver.resolve(
            &targetFn,
            &vec![source.clone()],
            dest,
            dest.location(),
            implicitConvertRunner.clone(),
        );
        let info = CallInfo::new(result.fnName, vec![source.clone()]);
        let kind = InstructionKind::FunctionCall(dest.clone(), info);
        kind
    }

    fn postProcessFunctionCall(&self, builder: &mut BlockBuilder, dest: &Variable, info: CallInfo, location: Location) {
        if let Some(op) = self.integerOps.get(&info.name) {
            let kind = InstructionKind::IntegerOp(dest.clone(), info.args[0].clone(), info.args[1].clone(), op.clone());
            builder.replaceInstruction(kind, location.clone());
            return;
        }
        let kind = InstructionKind::FunctionCall(dest.clone(), info);
        builder.replaceInstruction(kind, location.clone());
    }
}
