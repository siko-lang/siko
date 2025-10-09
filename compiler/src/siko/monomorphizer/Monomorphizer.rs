use core::panic;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    vec,
};

use crate::siko::{
    backend::drop::Util::HasTrivialDrop,
    hir::{
        BodyBuilder::BodyBuilder,
        ConstraintContext::ConstraintContext,
        Function::{Attributes, Function, FunctionKind, ParamInfo, Parameter, ResultKind},
        FunctionCallResolver::FunctionCallResolver,
        InstanceResolver::InstanceResolver,
        InstanceStore::InstanceStore,
        Instantiation::{instantiateEnum, instantiateStruct},
        Instruction::{
            CallInfo, EffectHandler, EnumCase, FieldId, FieldInfo, InstructionKind, SyntaxBlockId,
            SyntaxBlockIdSegment, WithContext, WithInfo,
        },
        Program::Program,
        Substitution::{createTypeSubstitutionFrom, Substitution},
        Type::Type,
        TypeVarAllocator::TypeVarAllocator,
        Unifier::Unifier,
        Variable::{Variable, VariableName},
    },
    location::{Location::Location, Report::ReportContext},
    monomorphizer::{
        Context::{Context, HandlerResolutionStore},
        Error::MonomorphizerError,
        Function::processBody,
        Handler::HandlerResolution,
        ImplicitContextBuilder::ImplicitContextBuilder,
        Queue::Key,
        Utils::Monomorphize,
    },
    qualifiedname::{
        buildModule,
        builtins::{getArrayTypeName, getCoroutineCoIsCompletedName, getCoroutineCoResumeName, getMainName},
        QualifiedName,
    },
    util::{Config::Config, Runner::Runner},
};

impl Monomorphize for Variable {
    fn process(&self, sub: &Substitution, mono: &mut Monomorphizer) -> Self {
        let name = self.name().clone();
        let v = self.cloneInto(name);
        v.setType(v.getType().process(sub, mono));
        v
    }
}

impl Monomorphize for Parameter {
    fn process(&self, sub: &Substitution, mono: &mut Monomorphizer) -> Self {
        match self {
            Parameter::Named(name, ty, info) => Parameter::Named(name.clone(), ty.process(sub, mono), info.clone()),
            Parameter::SelfParam(mutable, ty) => Parameter::SelfParam(*mutable, ty.process(sub, mono)),
        }
    }
}

impl Monomorphize for FieldInfo {
    fn process(&self, sub: &Substitution, mono: &mut Monomorphizer) -> Self {
        let mut result = self.clone();
        result.ty = result.ty.process(sub, mono);
        result
    }
}

impl Monomorphize for ResultKind {
    fn process(&self, sub: &Substitution, mono: &mut Monomorphizer) -> Self {
        match self {
            ResultKind::SingleReturn(ty) => ResultKind::SingleReturn(ty.process(sub, mono)),
            ResultKind::Coroutine(ty) => ResultKind::Coroutine(ty.process(sub, mono)),
        }
    }
}

pub struct Monomorphizer<'a> {
    pub config: Config,
    pub ctx: &'a ReportContext,
    pub program: Program,
    pub monomorphizedProgram: Program,
    queue: VecDeque<Key>,
    processed: BTreeSet<Key>,
    processed_type: BTreeMap<Type, Type>,
    pub resolutionStores: Vec<HandlerResolutionStore>,
    pub runner: Runner,
}

impl<'a> Monomorphizer<'a> {
    pub fn new(config: Config, ctx: &'a ReportContext, program: Program, runner: Runner) -> Monomorphizer<'a> {
        Monomorphizer {
            config,
            ctx,
            program,
            monomorphizedProgram: Program::new(),
            runner,
            queue: VecDeque::new(),
            processed: BTreeSet::new(),
            processed_type: BTreeMap::new(),
            resolutionStores: Vec::new(),
        }
    }

    fn addEntryPoints(&mut self) {
        if self.config.testOnly {
            let testRunnerMain = self.buildTestRunnerMain();

            self.program
                .functions
                .insert(testRunnerMain.name.clone(), testRunnerMain);
        }
        let main_name = getMainName();
        match self.program.functions.get(&main_name) {
            Some(_) => {
                self.addKey(Key::Function(
                    main_name,
                    Vec::new(),
                    HandlerResolution::new(),
                    Vec::new(),
                ));
            }
            None => {
                MonomorphizerError::MissingMainFunction(main_name).report(self.ctx);
            }
        }
    }

    fn buildTestRunnerMain(&mut self) -> Function {
        let mut testEntryFns = Vec::new();
        for (name, f) in &self.program.functions {
            if f.attributes.testEntry {
                testEntryFns.push(name.clone());
            }
        }
        let mut bodyBuilder = BodyBuilder::new();
        let mainBlock = bodyBuilder.createBlock();
        let parentSyntaxBlock = SyntaxBlockId::new();
        let mut prevBlock = mainBlock.clone();
        for (i, entry) in testEntryFns.iter().enumerate() {
            let mut entryBlock = bodyBuilder.createBlock();
            let syntaxBlockId = parentSyntaxBlock.add(SyntaxBlockIdSegment { value: i as u32 });
            entryBlock.addBlockStart(syntaxBlockId.clone(), Location::empty());
            let withRes = bodyBuilder.createTempValueWithType(Location::empty(), Type::Never(false));
            let testRunnerExecute = buildModule("TestRunner")
                .add("Testable".to_string())
                .add("execute".to_string());
            let handler = EffectHandler {
                method: testRunnerExecute.clone(),
                handler: entry.clone(),
                location: Location::empty(),
                optional: false,
            };
            let context = WithContext::EffectHandler(handler);
            let withInfo = WithInfo {
                contexts: vec![context],
                blockId: entryBlock.getBlockId(),
                parentSyntaxBlockId: parentSyntaxBlock.clone(),
                syntaxBlockId: syntaxBlockId.clone(),
                operations: Vec::new(),
                contextTypes: Vec::new(),
            };
            let with = InstructionKind::With(withRes, withInfo);
            prevBlock.addInstruction(with, Location::empty());
            let testRunnerRun = buildModule("TestRunner").add("run".to_string());
            let nameLiteralVar = bodyBuilder.createTempValueWithType(Location::empty(), Type::getStringLiteralType());
            let stringLiteral = InstructionKind::StringLiteral(nameLiteralVar.clone(), format!("{}", entry.toString()));
            let indexLiteralVar = bodyBuilder.createTempValueWithType(Location::empty(), Type::getIntType());
            let indexLiteral = InstructionKind::IntegerLiteral(indexLiteralVar.clone(), format!("{}", (i + 1) as u32));
            entryBlock.addInstruction(indexLiteral, Location::empty());
            let info = CallInfo::new(testRunnerRun.clone(), vec![nameLiteralVar, indexLiteralVar]);
            let callRes = bodyBuilder.createTempValueWithType(Location::empty(), Type::getUnitType());
            let executeCallKind = InstructionKind::FunctionCall(callRes.clone(), info);
            entryBlock.addInstruction(stringLiteral, Location::empty());
            entryBlock.addInstruction(executeCallKind, Location::empty());
            entryBlock.addBlockEnd(syntaxBlockId, Location::empty());
            prevBlock = entryBlock;
        }
        let mut endBlock = bodyBuilder.createBlock();
        prevBlock.addJump(endBlock.getBlockId(), Location::empty());
        let unit = endBlock.addUnit(Location::empty());
        endBlock.addReturn(unit, Location::empty());
        let testRunnerMain = Function {
            name: getMainName(),
            params: Vec::new(),
            result: ResultKind::SingleReturn(Type::getUnitType()),
            body: Some(bodyBuilder.build()),
            constraintContext: ConstraintContext::new(),
            kind: FunctionKind::UserDefined(Location::empty()),
            attributes: Attributes::new(),
        };
        //println!("handler fn {}", testRunnerMain);
        testRunnerMain
    }

    pub fn run(mut self) -> Program {
        self.monomorphizedProgram.implicits = self.program.implicits.clone();
        self.addEntryPoints();
        self.processQueue();
        for resolutionStore in &self.resolutionStores {
            resolutionStore.checkUnused(self.ctx);
        }
        let mut builder = ImplicitContextBuilder::new(&mut self);
        builder.process()
    }

    pub fn addKey(&mut self, key: Key) {
        //println!("Adding key {}", key);
        if self.processed.contains(&key) {
            return;
        }
        self.queue.push_back(key);
    }

    fn processQueue(&mut self) {
        while !self.queue.is_empty() {
            let key = self.queue.pop_front();
            match key {
                Some(key) => {
                    if self.processed.contains(&key) {
                        continue;
                    }
                    //println!("MONO Processing {}", key);
                    self.processed.insert(key.clone());
                    match key.clone() {
                        Key::Function(name, args, handlerResolution, impls) => {
                            self.monomorphizeFunction(name, args, handlerResolution, impls);
                        }
                        Key::Struct(name, args) => {
                            //println!("Processing structDef {}", key);
                            self.monomorphizeStruct(name, args);
                        }
                        Key::Enum(name, args) => {
                            //println!("Processing enum {}", key);
                            self.monomorphizeEnum(name, args);
                        }
                        Key::AutoDropFn(name, ty, handlerResolution) => {
                            //println!("Processing auto drop {}", key);
                            self.monomorphizeAutoDropFn(name, ty, handlerResolution, self.runner.child("auto_drop_fn"));
                        }
                    }
                }
                None => break,
            }
        }
    }

    fn monomorphizeFunction(
        &mut self,
        name: QualifiedName,
        args: Vec<Type>,
        handlerResolution: HandlerResolution,
        impls: Vec<QualifiedName>,
    ) {
        // println!(
        //     "MONO FN: {} {} {:?}",
        //     name,
        //     crate::siko::hir::Type::formatTypes(&args),
        //     impls
        // );
        let function = self
            .program
            .functions
            .get(&name)
            .expect("function not found in mono")
            .clone();
        //println!("Function: {}", function);
        if let FunctionKind::TraitMemberDecl(_) = function.kind {
            return;
        }
        let params = function
            .constraintContext
            .typeParameters
            .iter()
            .map(|ty| ty.clone())
            .collect();
        let monoName = if name.isMonomorphized() {
            name.clone()
        } else {
            self.getMonoName(&name, &args, handlerResolution.clone(), impls.clone())
        };
        let sub = createTypeSubstitutionFrom(params, args);
        let mut monoFn = function.clone();
        monoFn.result = monoFn.result.process(&sub, self);
        monoFn.params = monoFn.params.process(&sub, self);
        monoFn.body = processBody(monoFn.body.clone(), &sub, self, handlerResolution, &impls);

        monoFn.name = monoName.clone();
        //println!("MONO FN: {} => {}", name, monoName);
        self.monomorphizedProgram.functions.insert(monoName, monoFn);
        if name == getCoroutineCoResumeName() {
            // this code makes sure all variants of Coroutine.Result are monomorphized
            // because they will be called by generated code in coroutine lowering
            // so we need to ensure they exist in the monomorphized program
            let ty = function.result.getReturnType();
            match ty {
                Type::Tuple(args) => match &args[1] {
                    Type::Named(name, args) => {
                        let enumDef = self
                            .program
                            .getEnum(name)
                            .expect("coroutine resume's second return type not found");
                        let args: Vec<_> = args.process(&sub, self);
                        for v in enumDef.variants {
                            self.addKey(Key::Function(
                                v.name.clone(),
                                args.clone(),
                                HandlerResolution::new(),
                                Vec::new(),
                            ));
                        }
                    }
                    _ => panic!("coroutine resume's second return type must be Coroutine.Result"),
                },
                _ => panic!("coroutine resume must return a tuple"),
            }
        }
        if name == getCoroutineCoIsCompletedName() {
            // this code makes sure all variants of Bool are monomorphized
            // because they will be called by generated code in coroutine lowering
            // so we need to ensure they exist in the monomorphized program
            let ty = function.result.getReturnType();
            match ty {
                Type::Named(name, args) => {
                    let enumDef = self
                        .program
                        .getEnum(&name)
                        .expect("coroutine isCompleted return type not found");
                    let args: Vec<_> = args.process(&sub, self);
                    for v in enumDef.variants {
                        self.addKey(Key::Function(
                            v.name.clone(),
                            args.clone(),
                            HandlerResolution::new(),
                            Vec::new(),
                        ));
                    }
                }
                _ => panic!("coroutine isCompleted must return an enum"),
            }
        }
    }

    pub fn processType(&mut self, ty: Type) -> Type {
        if let Some(r) = self.processed_type.get(&ty) {
            return r.clone();
        }
        //println!("MONO TY {}", ty);
        if !ty.isConcrete() {
            panic!("non concrete type in mono {}", ty);
        }
        let r = match ty.clone() {
            Type::Named(name, args) => {
                let args = if name == getArrayTypeName() {
                    let mut newArgs = Vec::new();
                    for arg in args {
                        newArgs.push(self.processType(arg));
                    }
                    newArgs
                } else {
                    args
                };
                let monoName = self.getMonoName(&name, &args, HandlerResolution::new(), Vec::new());
                if self.program.structs.contains_key(&name) {
                    self.addKey(Key::Struct(name, args))
                } else if self.program.enums.contains_key(&name) {
                    self.addKey(Key::Enum(name, args))
                }
                Type::Named(monoName, Vec::new())
            }
            Type::Tuple(args) => {
                let args = args.into_iter().map(|arg| self.processType(arg)).collect();
                Type::Tuple(args)
            }
            Type::Function(args, result) => {
                let args = args.into_iter().map(|arg| self.processType(arg)).collect();
                Type::Function(args, Box::new(self.processType(*result)))
            }
            Type::FunctionPtr(args, result) => {
                let args = args.into_iter().map(|arg| self.processType(arg)).collect();
                Type::FunctionPtr(args, Box::new(self.processType(*result)))
            }
            Type::Var(v) => {
                panic!("TypeVar found in monomorphization {}", v);
            }
            Type::Reference(ty) => self.processType(*ty).asRef(),
            Type::Ptr(ty) => Type::Ptr(Box::new(self.processType(*ty))),
            Type::SelfType => Type::SelfType,
            Type::Never(v) => Type::Never(v),
            Type::NumericConstant(value) => Type::NumericConstant(value),
            Type::Void => Type::Void,
            Type::VoidPtr => Type::VoidPtr,
            Type::Coroutine(yieldTy, retTy) => {
                let yieldTy = self.processType(*yieldTy);
                let retTy = self.processType(*retTy);
                Type::Coroutine(Box::new(yieldTy), Box::new(retTy))
            }
        };
        self.processed_type.insert(ty, r.clone());
        r
    }

    pub fn getMonoName(
        &self,
        name: &QualifiedName,
        args: &Vec<Type>,
        handlerResolution: HandlerResolution,
        impls: Vec<QualifiedName>,
    ) -> QualifiedName {
        if args.is_empty() && handlerResolution.isEmpty() {
            name.clone()
        } else {
            let context = Context {
                args: args.iter().cloned().collect(),
                handlerResolution: handlerResolution,
                impls: impls,
            };
            name.monomorphized(context)
        }
    }

    fn monomorphizeStruct(&mut self, name: QualifiedName, args: Vec<Type>) {
        //println!("MONO Struct: {} {}", name, crate::siko::hir::Type::formatTypes(&args));
        let targetTy = Type::Named(name.clone(), args.clone());
        let c = self.program.structs.get(&name).expect("structDef not found in mono");
        let mut c = instantiateStruct(&mut TypeVarAllocator::new(), c, &targetTy);
        let name = self.getMonoName(&name, &args, HandlerResolution::new(), Vec::new());
        c.ty = self.processType(c.ty);
        c.fields = c
            .fields
            .iter()
            .cloned()
            .map(|mut f| {
                f.ty = self.processType(f.ty);
                f
            })
            .collect();
        c.methods.clear();
        c.name = name.clone();
        self.monomorphizedProgram.structs.insert(name, c);
    }

    fn monomorphizeEnum(&mut self, name: QualifiedName, args: Vec<Type>) {
        //println!("MONO ENUM: {} {}", name, crate::siko::hir::Type::formatTypes(&args));
        let e = self.program.enums.get(&name).expect("enum not found in mono");
        let targetTy = Type::Named(name.clone(), args.clone());
        let mut e = instantiateEnum(&mut TypeVarAllocator::new(), e, &targetTy);
        //println!("Enum ty {}", e.ty);
        let name = self.getMonoName(&name, &args, HandlerResolution::new(), Vec::new());
        //println!("Sub {}", sub);
        e.variants = e
            .variants
            .iter()
            .cloned()
            .map(|mut v| {
                v.name = self.getMonoName(&v.name, &args, HandlerResolution::new(), Vec::new());
                v.items = v.items.into_iter().map(|i| self.processType(i)).collect();
                v
            })
            .collect();
        e.methods.clear();
        e.ty = self.processType(targetTy);
        e.name = name.clone();
        self.monomorphizedProgram.enums.insert(name, e);
    }

    fn monomorphizeAutoDropFn(
        &mut self,
        name: QualifiedName,
        ty: Type,
        handlerResolution: HandlerResolution,
        runner: Runner,
    ) {
        //println!("MONO AUTO DROP: {} {}", name, ty);
        let monoName = self.getMonoName(&name, &vec![ty.clone()], handlerResolution.clone(), Vec::new());

        let mut bodyBuilder = BodyBuilder::new();
        let mut builder = bodyBuilder.createBlock();

        let location = Location::empty();

        let dropVar = bodyBuilder.createTempValueWithType(location.clone(), ty.clone());
        builder.addDeclare(dropVar.clone(), location.clone());

        let selfVar = Variable::newWithType(VariableName::Arg("self".to_string()), Location::empty(), ty.clone());

        let mut hasInstance = false;

        let allocator = TypeVarAllocator::new();
        let implStore = InstanceStore::new();
        let implResolver =
            InstanceResolver::new(allocator.clone(), &implStore, &self.program, ConstraintContext::new());

        if implResolver.isDrop(&ty, self.runner.child("is_drop")) {
            hasInstance = true;
            let unifier = Unifier::withContext(&self.ctx, runner.child("unifier"));
            let mut fnCallResolver = FunctionCallResolver::new(
                &self.program,
                allocator.clone(),
                self.ctx,
                &implStore,
                ConstraintContext::new(),
                unifier.clone(),
            );
            let dropRes = bodyBuilder.createTempValueWithType(location.clone(), selfVar.getType());
            let (fnName, impls) = fnCallResolver.resolveDropCall(
                selfVar.clone(),
                dropRes.clone(),
                runner.child("drop_instance_resolver"),
            );
            let mut info = CallInfo::new(fnName.clone(), vec![selfVar.clone()]);
            info.instanceRefs = impls;
            let kind = InstructionKind::FunctionCall(dropRes.clone(), info);
            builder.addInstruction(kind, location.clone());
            builder.addAssign(dropVar.clone(), dropRes, location.clone());
        }

        if !hasInstance {
            builder.addAssign(dropVar.clone(), selfVar.clone(), location.clone());
        }

        match &ty {
            Type::Named(name, _) => {
                if let Some(c) = self.program.getStruct(&name) {
                    let mut allocator = &mut TypeVarAllocator::new();
                    let c = instantiateStruct(&mut allocator, &c, &ty);
                    for f in &c.fields {
                        if f.ty.hasTrivialDrop() {
                            continue;
                        }
                        let fieldInfo = FieldInfo {
                            name: FieldId::Named(f.name.clone()),
                            ty: Some(f.ty.clone()),
                            location: location.clone(),
                        };
                        let field =
                            builder.addTypedFieldRef(dropVar.clone(), vec![fieldInfo], location.clone(), f.ty.clone());
                        let dropRes = bodyBuilder.createTempValueWithType(location.clone(), Type::getUnitType());
                        let dropKind = InstructionKind::Drop(dropRes, field.clone());
                        builder.addInstruction(dropKind, location.clone());
                    }
                }
                if let Some(e) = self.program.getEnum(&name) {
                    let mut allocator = &mut TypeVarAllocator::new();
                    let e = instantiateEnum(&mut allocator, &e, &ty);
                    let contBuilder = bodyBuilder.createBlock();
                    let mut cases = Vec::new();
                    for (index, v) in e.variants.iter().enumerate() {
                        let mut caseBuilder = bodyBuilder.createBlock();
                        let case = EnumCase {
                            index: Some(index as u32),
                            branch: caseBuilder.getBlockId(),
                        };
                        let transformType = Type::Tuple(v.items.clone());
                        let transformValue = caseBuilder.addTypedTransform(
                            dropVar.clone(),
                            index as u32,
                            location.clone(),
                            transformType,
                        );
                        let dropRes = bodyBuilder.createTempValueWithType(location.clone(), Type::getUnitType());
                        let dropKind = InstructionKind::Drop(dropRes, transformValue);
                        caseBuilder.addInstruction(dropKind, location.clone());
                        caseBuilder.addJump(contBuilder.getBlockId(), location.clone());
                        cases.push(case);
                    }
                    let enumKind = InstructionKind::EnumSwitch(dropVar.clone(), cases);
                    builder.addInstruction(enumKind, location.clone());
                    builder = contBuilder;
                }
            }
            Type::Tuple(args) => {
                for (index, arg) in args.iter().enumerate() {
                    if arg.hasTrivialDrop() {
                        continue;
                    }
                    let fields = vec![FieldInfo {
                        name: FieldId::Indexed(index as u32),
                        ty: Some(arg.clone()),
                        location: location.clone(),
                    }];
                    let field = builder.addTypedFieldRef(dropVar.clone(), fields, location.clone(), arg.clone());
                    let dropRes = bodyBuilder.createTempValueWithType(location.clone(), Type::getUnitType());
                    let dropKind = InstructionKind::Drop(dropRes, field);
                    builder.addInstruction(dropKind, location.clone());
                }
            }
            _ => {}
        }

        let unitValue = builder.addUnit(location.clone());
        builder.addReturn(unitValue, location);

        let mut attributes = Attributes::new();
        attributes.inline = true;
        let dropFn = Function {
            name: monoName.clone(),
            params: vec![Parameter::Named("self".to_string(), ty.clone(), ParamInfo::new())],
            result: ResultKind::SingleReturn(Type::getUnitType()),
            body: Some(bodyBuilder.build()),
            constraintContext: ConstraintContext::new(),
            kind: FunctionKind::UserDefined(Location::empty()),
            attributes: attributes,
        };

        self.program.functions.insert(monoName.clone(), dropFn);
        self.addKey(Key::Function(monoName, Vec::new(), handlerResolution, Vec::new()));
    }
}
