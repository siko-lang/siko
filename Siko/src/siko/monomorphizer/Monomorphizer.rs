use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt::Display,
};

use crate::siko::{
    hir::{
        Apply::{instantiateClass, instantiateEnum, Apply},
        BodyBuilder::BodyBuilder,
        ConstraintContext::ConstraintContext,
        Function::{Block, Body, Function, FunctionKind, Parameter, Variable, VariableName},
        InstanceResolver::ResolutionResult,
        Instruction::{EnumCase, FieldInfo, Instruction, InstructionKind, JumpDirection},
        Program::Program,
        Substitution::{createTypeSubstitutionFrom, TypeSubstitution},
        Type::{formatTypes, Type},
        TypeVarAllocator::TypeVarAllocator,
        Unification::unify,
    },
    location::{
        Location::Location,
        Report::{Report, ReportContext},
    },
    qualifiedname::{build, getAutoDropFnName, getDropFnName, getDropName, QualifiedName},
};

fn createTypeSubstitution(ty1: &Type, ty2: &Type) -> TypeSubstitution {
    let mut sub = TypeSubstitution::new();
    if unify(&mut sub, ty1, &ty2, true).is_err() {
        panic!("Unification failed for {} {}", ty1, ty2);
    }
    sub
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Key {
    Class(QualifiedName, Vec<Type>),
    Enum(QualifiedName, Vec<Type>),
    Function(QualifiedName, Vec<Type>),
    AutoDropFn(QualifiedName, Type),
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Key::Class(name, types) => write!(f, "{}/{}", name, formatTypes(types)),
            Key::Enum(name, types) => write!(f, "{}/{}", name, formatTypes(types)),
            Key::Function(name, types) => write!(f, "{}/{}", name, formatTypes(types)),
            Key::AutoDropFn(name, ty) => write!(f, "{}/{}", name, ty),
        }
    }
}

trait Monomorphize {
    fn process(&self, sub: &TypeSubstitution, mono: &mut Monomorphizer) -> Self;
}

impl Monomorphize for Type {
    fn process(&self, sub: &TypeSubstitution, mono: &mut Monomorphizer) -> Self {
        let ty = self.apply(sub);
        mono.processType(ty)
    }
}

impl<T: Monomorphize> Monomorphize for Option<T> {
    fn process(&self, sub: &TypeSubstitution, mono: &mut Monomorphizer) -> Self {
        match self {
            Some(v) => Some(v.process(sub, mono)),
            None => None,
        }
    }
}

impl<T: Monomorphize> Monomorphize for Vec<T> {
    fn process(&self, sub: &TypeSubstitution, mono: &mut Monomorphizer) -> Self {
        self.iter().map(|i| i.process(sub, mono)).collect()
    }
}

impl Monomorphize for Variable {
    fn process(&self, sub: &TypeSubstitution, mono: &mut Monomorphizer) -> Self {
        let mut v = self.clone();
        v.ty = v.ty.process(sub, mono);
        v
    }
}

impl Monomorphize for Parameter {
    fn process(&self, sub: &TypeSubstitution, mono: &mut Monomorphizer) -> Self {
        match self {
            Parameter::Named(name, ty, mutable) => Parameter::Named(name.clone(), ty.process(sub, mono), *mutable),
            Parameter::SelfParam(mutable, ty) => Parameter::SelfParam(*mutable, ty.process(sub, mono)),
        }
    }
}

impl Monomorphize for Instruction {
    fn process(&self, sub: &TypeSubstitution, mono: &mut Monomorphizer) -> Self {
        fn getFunctionName(
            kind: FunctionKind,
            name: QualifiedName,
            mono: &mut Monomorphizer,
            sub: &TypeSubstitution,
        ) -> QualifiedName {
            if let Some(traitName) = kind.isTraitCall() {
                //println!("Trait call in mono!");
                let traitDef = mono.program.getTrait(&traitName).unwrap();
                //println!("trait {}", traitDef);
                let mut allocator = TypeVarAllocator::new();
                let traitDef = traitDef.apply(&sub);
                //println!("trait ii {}", traitDef);
                if let Some(instances) = mono.program.instanceResolver.lookupInstances(&traitName) {
                    let resolutionResult = instances.find(&mut allocator, &traitDef.params);
                    match resolutionResult {
                        ResolutionResult::Winner(instance) => {
                            //println!("instance  {}", instance);
                            for m in &instance.members {
                                let base = m.fullName.base();
                                if base == name {
                                    return m.fullName.clone();
                                }
                            }
                            let traitDef = mono.program.getTrait(&traitName).expect("trait not found in mono");
                            for m in &traitDef.members {
                                if m.fullName == name {
                                    return m.fullName.clone();
                                }
                            }
                            panic!("instance member not found!")
                        }
                        ResolutionResult::Ambiguous(_) => {
                            panic!("Ambiguous instances in mono!");
                        }
                        ResolutionResult::NoInstanceFound => {
                            if traitName == getDropName() {
                                return getDropFnName();
                            } else {
                                panic!("instance not found in mono for {}!", traitName);
                            }
                        }
                    }
                } else {
                    if traitName == getDropName() {
                        return getDropFnName();
                    } else {
                        panic!("instances not found in mono for {}!", traitName);
                    }
                }
            } else {
                name.clone()
            }
        }
        //println!("MONO INSTR {}", self);
        let mut instruction = self.clone();
        let kind: InstructionKind = match &self.kind {
            InstructionKind::FunctionCall(dest, name, args) => {
                //println!("Calling {}", name);
                let target_fn = mono.program.getFunction(name).expect("function not found in mono");
                let fn_ty = target_fn.getType();
                let fnResult = fn_ty.getResult();
                let fn_ty = if fnResult.hasSelfType() {
                    //println!("fn type before {}", fn_ty);
                    let (args, result) = fn_ty.splitFnType().unwrap();
                    let result = result.changeSelfType(args[0].clone());
                    let fn_ty = Type::Function(args, Box::new(result));
                    //println!("fn type after {}", fn_ty);
                    fn_ty
                } else {
                    fn_ty
                };
                let arg_types: Vec<_> = args
                    .iter()
                    .map(|arg| {
                        //println!("arg {}", arg);
                        let ty = arg.getType();
                        ty.apply(&sub)
                    })
                    .collect();
                //println!("sub {}", sub);
                let result = dest.getType().apply(sub);
                let context_ty = Type::Function(arg_types, Box::new(result));
                //println!("fn type {}", fn_ty);
                //println!("context type {}", context_ty);
                let sub = createTypeSubstitution(&context_ty, &fn_ty);
                //println!("target ctx {}", target_fn.constraintContext);
                let name = getFunctionName(target_fn.kind.clone(), name.clone(), mono, &sub);
                let target_fn = mono.program.functions.get(&name).expect("function not found in mono");
                //println!("real {} {}", target_fn.getType(), target_fn.constraintContext);
                let sub = createTypeSubstitution(&context_ty, &target_fn.getType());
                //println!("target ctx {}", target_fn.constraintContext);
                let ty_args: Vec<_> = target_fn
                    .constraintContext
                    .typeParameters
                    .iter()
                    .map(|ty| ty.apply(&sub))
                    .collect();
                //println!("{} type args {}", name, formatTypes(&ty_args));
                let fn_name = mono.get_mono_name(&name, &ty_args);
                mono.addKey(Key::Function(name.clone(), ty_args));
                InstructionKind::FunctionCall(dest.clone(), fn_name, args.clone())
            }
            InstructionKind::Ref(dest, src) => {
                if dest.ty.as_ref().unwrap().isReference() && src.ty.as_ref().unwrap().isReference() {
                    InstructionKind::Assign(dest.clone(), src.clone())
                } else {
                    InstructionKind::Ref(dest.clone(), src.clone())
                }
            }
            InstructionKind::Drop(dest, dropVar) => {
                let ty = dropVar.ty.apply(sub).unwrap();
                let monoName = mono.get_mono_name(&getAutoDropFnName(), &vec![ty.clone()]);
                mono.addKey(Key::AutoDropFn(getAutoDropFnName(), ty.clone()));
                InstructionKind::FunctionCall(dest.clone(), monoName, vec![dropVar.clone()])
            }
            k => k.clone(),
        };
        instruction.kind = kind.process(sub, mono);
        instruction
    }
}

impl Monomorphize for FieldInfo {
    fn process(&self, sub: &TypeSubstitution, mono: &mut Monomorphizer) -> Self {
        let mut result = self.clone();
        result.ty = result.ty.process(sub, mono);
        result
    }
}

impl Monomorphize for InstructionKind {
    fn process(&self, sub: &TypeSubstitution, mono: &mut Monomorphizer) -> Self {
        match self {
            InstructionKind::FunctionCall(dest, name, args) => {
                InstructionKind::FunctionCall(dest.process(sub, mono), name.clone(), args.process(sub, mono))
            }
            InstructionKind::MethodCall(_, _, _, _) => {
                unreachable!("method in mono??")
            }
            InstructionKind::DynamicFunctionCall(dest, root, args) => InstructionKind::DynamicFunctionCall(
                dest.process(sub, mono),
                root.process(sub, mono),
                args.process(sub, mono),
            ),
            InstructionKind::ValueRef(dest, value) => {
                InstructionKind::ValueRef(dest.process(sub, mono), value.process(sub, mono))
            }
            InstructionKind::FieldRef(dest, root, name) => {
                InstructionKind::FieldRef(dest.process(sub, mono), root.process(sub, mono), name.clone())
            }
            InstructionKind::TupleIndex(dest, root, index) => {
                InstructionKind::TupleIndex(dest.process(sub, mono), root.process(sub, mono), *index)
            }
            InstructionKind::Bind(lhs, rhs, mutable) => {
                InstructionKind::Bind(lhs.process(sub, mono), rhs.process(sub, mono), mutable.clone())
            }
            InstructionKind::Tuple(dest, args) => {
                InstructionKind::Tuple(dest.process(sub, mono), args.process(sub, mono))
            }
            InstructionKind::StringLiteral(dest, lit) => {
                InstructionKind::StringLiteral(dest.process(sub, mono), lit.clone())
            }
            InstructionKind::IntegerLiteral(dest, lit) => {
                InstructionKind::IntegerLiteral(dest.process(sub, mono), lit.clone())
            }
            InstructionKind::CharLiteral(dest, lit) => InstructionKind::CharLiteral(dest.process(sub, mono), *lit),
            InstructionKind::Return(dest, arg) => {
                InstructionKind::Return(dest.process(sub, mono), arg.process(sub, mono))
            }
            InstructionKind::Ref(dest, arg) => InstructionKind::Ref(dest.process(sub, mono), arg.process(sub, mono)),
            InstructionKind::Drop(dest, dropVar) => {
                InstructionKind::Drop(dest.process(sub, mono), dropVar.process(sub, mono))
            }
            InstructionKind::Jump(dest, block_id, direction) => {
                InstructionKind::Jump(dest.process(sub, mono), *block_id, direction.clone())
            }
            InstructionKind::Assign(dest, rhs) => {
                InstructionKind::Assign(dest.process(sub, mono), rhs.process(sub, mono))
            }
            InstructionKind::FieldAssign(dest, rhs, fields) => InstructionKind::FieldAssign(
                dest.process(sub, mono),
                rhs.process(sub, mono),
                fields.process(sub, mono),
            ),
            InstructionKind::DeclareVar(var) => InstructionKind::DeclareVar(var.process(sub, mono)),
            InstructionKind::Transform(dest, root, index) => {
                InstructionKind::Transform(dest.process(sub, mono), root.process(sub, mono), index.clone())
            }
            InstructionKind::EnumSwitch(root, cases) => {
                InstructionKind::EnumSwitch(root.process(sub, mono), cases.clone())
            }
            InstructionKind::IntegerSwitch(root, cases) => {
                InstructionKind::IntegerSwitch(root.process(sub, mono), cases.clone())
            }
            InstructionKind::StringSwitch(root, cases) => {
                InstructionKind::StringSwitch(root.process(sub, mono), cases.clone())
            }
            InstructionKind::BlockStart(info) => InstructionKind::BlockStart(info.clone()),
            InstructionKind::BlockEnd(info) => InstructionKind::BlockEnd(info.clone()),
        }
    }
}

impl Monomorphize for Block {
    fn process(&self, sub: &TypeSubstitution, mono: &mut Monomorphizer) -> Self {
        let instructions = self.instructions.process(sub, mono);
        Block {
            id: self.id.clone(),
            instructions: instructions,
        }
    }
}

impl Monomorphize for Body {
    fn process(&self, sub: &TypeSubstitution, mono: &mut Monomorphizer) -> Self {
        let blocks = self.blocks.process(sub, mono);
        Body {
            blocks: blocks,
            varTypes: BTreeMap::new(),
        }
    }
}

pub struct Monomorphizer<'a> {
    ctx: &'a ReportContext,
    program: Program,
    monomorphizedProgram: Program,
    queue: VecDeque<Key>,
    processed: BTreeSet<Key>,
    processed_type: BTreeMap<Type, Type>,
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
        }
    }

    pub fn run(mut self) -> Program {
        let main_name = build("Main", "main");
        match self.program.functions.get(&main_name) {
            Some(_) => {
                self.addKey(Key::Function(main_name, Vec::new()));
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
        self.monomorphizedProgram
    }

    fn addKey(&mut self, key: Key) {
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
                        Key::Function(name, args) => {
                            //println!("Processing func {}", key);
                            self.monomorphizeFunction(name, args);
                        }
                        Key::Class(name, args) => {
                            //println!("Processing class {}", key);
                            self.monomorphizeClass(name, args);
                        }
                        Key::Enum(name, args) => {
                            //println!("Processing enum {}", key);
                            self.monomorphizeEnum(name, args);
                        }
                        Key::AutoDropFn(name, ty) => {
                            //println!("Processing auto drop {}", key);
                            self.monomorphizeAutoDropFn(name, ty);
                        }
                    }
                }
                None => break,
            }
        }
    }

    fn monomorphizeFunction(&mut self, name: QualifiedName, args: Vec<Type>) {
        //println!("MONO FN: {} {}", name, formatTypes(&args));
        let function = self
            .program
            .functions
            .get(&name)
            .expect("function not found in mono")
            .clone();
        if let FunctionKind::TraitMemberDecl(_) = function.kind {
            return;
        }
        let params = function
            .constraintContext
            .typeParameters
            .iter()
            .map(|ty| ty.clone())
            .collect();
        let sub = createTypeSubstitutionFrom(&params, &args);
        let mut monoFn = function.clone();
        monoFn.result = self.processType(monoFn.result.apply(&sub));
        monoFn.params = monoFn.params.process(&sub, self);
        monoFn.body = monoFn.body.process(&sub, self);
        let monoName = self.get_mono_name(&name, &args);
        monoFn.name = monoName.clone();
        self.monomorphizedProgram.functions.insert(monoName, monoFn);
    }

    fn processType(&mut self, ty: Type) -> Type {
        if let Some(r) = self.processed_type.get(&ty) {
            return r.clone();
        }
        //println!("MONO TY {}", ty);
        if !ty.isConcrete() {
            panic!("non concrete type in mono {}", ty);
        }
        let r = match ty.clone() {
            Type::Named(name, args, _) => {
                let monoName = self.get_mono_name(&name, &args);
                if self.program.classes.contains_key(&name) {
                    self.addKey(Key::Class(name, args))
                } else if self.program.enums.contains_key(&name) {
                    self.addKey(Key::Enum(name, args))
                }
                Type::Named(monoName, Vec::new(), None)
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

    fn get_mono_name(&self, name: &QualifiedName, args: &Vec<Type>) -> QualifiedName {
        if args.is_empty() {
            name.monomorphized(String::new())
        } else {
            name.monomorphized(formatTypes(args))
        }
    }

    fn monomorphizeClass(&mut self, name: QualifiedName, args: Vec<Type>) {
        //println!("MONO CLASS: {} {}", name, formatTypes(&args));
        let targetTy = Type::Named(name.clone(), args.clone(), None);
        let c = self.program.classes.get(&name).expect("class not found in mono");
        let mut c = instantiateClass(&mut TypeVarAllocator::new(), c, &targetTy);
        let name = self.get_mono_name(&name, &args);
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
        self.monomorphizedProgram.classes.insert(name, c);
    }

    fn monomorphizeEnum(&mut self, name: QualifiedName, args: Vec<Type>) {
        //println!("MONO ENUM: {} {}", name, formatTypes(&args));
        let e = self.program.enums.get(&name).expect("enum not found in mono");
        let targetTy = Type::Named(name.clone(), args.clone(), None);
        let mut e = instantiateEnum(&mut TypeVarAllocator::new(), e, &targetTy);
        //println!("Enum ty {}", e.ty);
        let name = self.get_mono_name(&name, &args);
        //println!("Sub {}", sub);
        e.variants = e
            .variants
            .iter()
            .cloned()
            .map(|mut v| {
                v.name = self.get_mono_name(&v.name, &args);
                v.items = v.items.into_iter().map(|i| self.processType(i)).collect();
                v
            })
            .collect();
        e.methods.clear();
        e.name = name.clone();
        self.monomorphizedProgram.enums.insert(name, e);
    }

    fn monomorphizeAutoDropFn(&mut self, name: QualifiedName, ty: Type) {
        //println!("MONO AUTO DROP: {} {}", name, ty);
        let monoName = self.get_mono_name(&name, &vec![ty.clone()]);

        let mut bodyBuilder = BodyBuilder::new();
        let mut builder = bodyBuilder.createBlock();

        let location = Location::empty();

        let mut dropVar = bodyBuilder.createTempValue(VariableName::DropVar, location.clone());
        dropVar.ty = Some(ty.clone());

        let selfVar = Variable {
            value: VariableName::Arg("self".to_string()),
            ty: Some(ty.clone()),
            location: Location::empty(),
            index: 0,
        };

        let mut hasInstance = false;

        if let Some(instances) = self.program.instanceResolver.lookupInstances(&&getDropName()) {
            let mut allocator = TypeVarAllocator::new();
            let result = instances.find(&mut allocator, &vec![ty.clone()]);
            if let ResolutionResult::Winner(_) = result {
                //println!("Drop found for {}", ty);
                hasInstance = true;
                let dropRes =
                    builder.addTypedFunctionCall(getDropFnName(), vec![selfVar.clone()], location.clone(), ty.clone());
                builder.addBind(dropVar.clone(), dropRes, false, location.clone());
            }
        }

        if !hasInstance {
            builder.addBind(dropVar.clone(), selfVar.clone(), false, location.clone());
        }

        match &ty {
            Type::Named(name, _, _) => {
                if let Some(c) = self.program.getClass(&name) {
                    let mut allocator = &mut TypeVarAllocator::new();
                    let c = instantiateClass(&mut allocator, &c, &ty);
                    for f in &c.fields {
                        let field =
                            builder.addTypedFieldRef(dropVar.clone(), f.name.clone(), location.clone(), f.ty.clone());
                        let mut dropRes = bodyBuilder.createTempValue(VariableName::AutoDropResult, location.clone());
                        dropRes.ty = Some(Type::getUnitType());
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
                        let mut dropRes = bodyBuilder.createTempValue(VariableName::AutoDropResult, location.clone());
                        dropRes.ty = Some(Type::getUnitType());
                        let dropKind = InstructionKind::Drop(dropRes, transformValue);
                        caseBuilder.addInstruction(dropKind, location.clone());
                        caseBuilder.addJump(contBuilder.getBlockId(), JumpDirection::Forward, location.clone());
                        cases.push(case);
                    }
                    let enumKind = InstructionKind::EnumSwitch(dropVar.clone(), cases);
                    builder.addInstruction(enumKind, location.clone());
                    builder = contBuilder;
                }
            }
            Type::Tuple(args) => {
                for (index, arg) in args.iter().enumerate() {
                    let field =
                        builder.addTypedTupleIndex(dropVar.clone(), index as i32, location.clone(), arg.clone());
                    let mut dropRes = bodyBuilder.createTempValue(VariableName::AutoDropResult, location.clone());
                    dropRes.ty = Some(Type::getUnitType());
                    let dropKind = InstructionKind::Drop(dropRes, field);
                    builder.addInstruction(dropKind, location.clone());
                }
            }
            _ => {}
        }

        let mut unitValue = builder.addUnit(location.clone());
        unitValue.ty = Some(Type::getUnitType());
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
        self.addKey(Key::Function(monoName, Vec::new()));
    }
}
