use core::panic;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    vec,
};

use crate::siko::{
    hir::{
        Apply::Apply,
        BodyBuilder::BodyBuilder,
        ConstraintContext::ConstraintContext,
        Function::{Function, FunctionKind, Parameter},
        InstanceResolver::ResolutionResult,
        Instantiation::{instantiateEnum, instantiateStruct},
        Instruction::{EnumCase, FieldId, FieldInfo, InstructionKind},
        Program::Program,
        Substitution::{createTypeSubstitutionFrom, Substitution},
        Type::Type,
        TypeVarAllocator::TypeVarAllocator,
        Variable::{Variable, VariableName},
    },
    location::{
        Location::Location,
        Report::{Report, ReportContext},
    },
    monomorphizer::{
        Context::{Context, HandlerResolutionStore},
        Function::processBody,
        Handler::HandlerResolution,
        ImplicitContextBuilder::ImplicitContextBuilder,
        Queue::Key,
        Utils::Monomorphize,
    },
    qualifiedname::{
        builtins::{getDropFnName, getDropName, getMainName},
        QualifiedName,
    },
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
            Parameter::Named(name, ty, mutable) => Parameter::Named(name.clone(), ty.process(sub, mono), *mutable),
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

pub struct Monomorphizer<'a> {
    pub ctx: &'a ReportContext,
    pub program: Program,
    pub monomorphizedProgram: Program,
    queue: VecDeque<Key>,
    processed: BTreeSet<Key>,
    processed_type: BTreeMap<Type, Type>,
    pub resolutionStores: Vec<HandlerResolutionStore>,
}

impl<'a> Monomorphizer<'a> {
    pub fn new(ctx: &'a ReportContext, program: Program) -> Monomorphizer<'a> {
        Monomorphizer {
            ctx: ctx,
            program: program,
            monomorphizedProgram: Program::new(),
            queue: VecDeque::new(),
            processed: BTreeSet::new(),
            processed_type: BTreeMap::new(),
            resolutionStores: Vec::new(),
        }
    }

    pub fn run(mut self) -> Program {
        self.monomorphizedProgram.implicits = self.program.implicits.clone();
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
                let slogan = format!(
                    "No {} function found",
                    format!("{}", self.ctx.yellow(&main_name.toString()))
                );
                let r = Report::new(self.ctx, slogan, None);
                r.print();
            }
        }
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
                            self.monomorphizeAutoDropFn(name, ty, handlerResolution);
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
        monoFn.result = self.processType(monoFn.result.apply(&sub));
        monoFn.params = monoFn.params.process(&sub, self);
        monoFn.body = processBody(monoFn.body.clone(), &sub, self, handlerResolution, &impls);

        monoFn.name = monoName.clone();
        //println!("MONO FN: {} => {}", name, monoName);
        self.monomorphizedProgram.functions.insert(monoName, monoFn);
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
            Type::Var(v) => {
                panic!("TypeVar found in monomorphization {}", v);
            }
            Type::Reference(ty, l) => Type::Reference(Box::new(self.processType(*ty)), l.clone()),
            Type::Ptr(ty) => Type::Ptr(Box::new(self.processType(*ty))),
            Type::SelfType => Type::SelfType,
            Type::Never(v) => Type::Never(v),
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
        //println!("MONO Struct: {} {}", name, formatTypes(&args));
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
        //println!("MONO ENUM: {} {}", name, formatTypes(&args));
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
        e.name = name.clone();
        self.monomorphizedProgram.enums.insert(name, e);
    }

    fn monomorphizeAutoDropFn(&mut self, name: QualifiedName, ty: Type, handlerResolution: HandlerResolution) {
        //println!("MONO AUTO DROP: {} {}", name, ty);
        let monoName = self.getMonoName(&name, &vec![ty.clone()], handlerResolution.clone(), Vec::new());

        let mut bodyBuilder = BodyBuilder::new();
        let mut builder = bodyBuilder.createBlock();

        let location = Location::empty();

        let dropVar = bodyBuilder.createTempValueWithType(location.clone(), ty.clone());
        builder.addDeclare(dropVar.clone(), location.clone());

        let selfVar = Variable::newWithType(VariableName::Arg("self".to_string()), Location::empty(), ty.clone());

        let mut hasInstance = false;

        if let Some(instances) = self.program.instanceResolver.lookupInstances(&&getDropName()) {
            let mut allocator = TypeVarAllocator::new();
            let result = instances.find(&mut allocator, &vec![ty.clone()]);
            if let ResolutionResult::Winner(_) = result {
                //println!("Drop found for {}", ty);
                hasInstance = true;
                let dropRes =
                    builder.addTypedFunctionCall(getDropFnName(), vec![selfVar.clone()], location.clone(), ty.clone());
                builder.addAssign(dropVar.clone(), dropRes, location.clone());
            }
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
                            index: index as u32,
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

        let dropFn = Function {
            name: monoName.clone(),
            params: vec![Parameter::Named("self".to_string(), ty.clone(), false)],
            result: Type::getUnitType(),
            body: Some(bodyBuilder.build()),
            constraintContext: ConstraintContext::new(),
            kind: FunctionKind::UserDefined,
        };

        self.program.functions.insert(monoName.clone(), dropFn);
        self.addKey(Key::Function(monoName, Vec::new(), handlerResolution, Vec::new()));
    }
}
