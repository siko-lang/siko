use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt::Display,
};

use crate::siko::{
    hir::{
        Apply::{instantiateClass, instantiateEnum, Apply},
        Function::{Block, Body, FunctionKind, Instruction, InstructionKind, Parameter, Variable},
        InstanceResolver::ResolutionResult,
        Program::Program,
        Substitution::TypeSubstitution,
        Type::{formatTypes, Type},
        TypeVarAllocator::TypeVarAllocator,
        Unification::unify,
    },
    location::Report::{Report, ReportContext},
    qualifiedname::{build, QualifiedName},
};

fn createTypeSubstitution(ty1: &Type, ty2: &Type) -> TypeSubstitution {
    let mut sub = TypeSubstitution::new();
    if unify(&mut sub, ty1, &ty2, true).is_err() {
        panic!("Unification failed for {} {}", ty1, ty2);
    }
    sub
}

fn createTypeSubstitutionFrom(ty1: &Vec<Type>, ty2: &Vec<Type>) -> TypeSubstitution {
    let mut sub = TypeSubstitution::new();
    for (ty1, ty2) in ty1.iter().zip(ty2) {
        unify(&mut sub, ty1, ty2, true).expect("Unification failed");
    }
    sub
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Key {
    Class(QualifiedName, Vec<Type>),
    Enum(QualifiedName, Vec<Type>),
    Function(QualifiedName, Vec<Type>),
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Key::Class(name, types) => write!(f, "{}/{}", name, formatTypes(types)),
            Key::Enum(name, types) => write!(f, "{}/{}", name, formatTypes(types)),
            Key::Function(name, types) => write!(f, "{}/{}", name, formatTypes(types)),
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
        fn getFunctionName(kind: FunctionKind, name: QualifiedName, mono: &mut Monomorphizer, sub: &TypeSubstitution) -> QualifiedName {
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
                            panic!("no instance found in mono!");
                        }
                    }
                } else {
                    panic!("no instances for found trait {} in mono!", traitName);
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
                let target_fn = mono.program.functions.get(name).expect("function not found in mono");
                let fn_ty = target_fn.getType();
                let arg_types: Vec<_> = args
                    .iter()
                    .map(|arg| {
                        //println!("arg {}", arg);
                        let ty = arg.getType();
                        ty.apply(&sub)
                    })
                    .collect();
                let result = dest.getType().apply(sub);
                let context_ty = Type::Function(arg_types, Box::new(result));
                //println!("fn type {}", fn_ty);
                //println!("context type {}", context_ty);
                let sub = createTypeSubstitution(&context_ty, &fn_ty);
                //println!("target ctx {}", target_fn.constraintContext);
                let ty_args: Vec<_> = target_fn.constraintContext.typeParameters.iter().map(|ty| ty.apply(&sub)).collect();

                //println!("{} type args {}", name, formatTypes(&ty_args));
                let name = getFunctionName(target_fn.kind.clone(), name.clone(), mono, &sub);
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
            k => k.clone(),
        };
        instruction.kind = kind.process(sub, mono);
        instruction
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
            InstructionKind::DynamicFunctionCall(dest, root, args) => {
                InstructionKind::DynamicFunctionCall(dest.process(sub, mono), root.process(sub, mono), args.process(sub, mono))
            }
            InstructionKind::ValueRef(dest, value) => InstructionKind::ValueRef(dest.process(sub, mono), value.process(sub, mono)),
            InstructionKind::FieldRef(dest, root, name) => InstructionKind::FieldRef(dest.process(sub, mono), root.process(sub, mono), name.clone()),
            InstructionKind::TupleIndex(dest, root, index) => InstructionKind::TupleIndex(dest.process(sub, mono), root.process(sub, mono), *index),
            InstructionKind::Bind(lhs, rhs, mutable) => InstructionKind::Bind(lhs.process(sub, mono), rhs.process(sub, mono), mutable.clone()),
            InstructionKind::Tuple(dest, args) => InstructionKind::Tuple(dest.process(sub, mono), args.process(sub, mono)),
            InstructionKind::StringLiteral(dest, lit) => InstructionKind::StringLiteral(dest.process(sub, mono), lit.clone()),
            InstructionKind::IntegerLiteral(dest, lit) => InstructionKind::IntegerLiteral(dest.process(sub, mono), lit.clone()),
            InstructionKind::CharLiteral(dest, lit) => InstructionKind::CharLiteral(dest.process(sub, mono), *lit),
            InstructionKind::Return(dest, arg) => InstructionKind::Return(dest.process(sub, mono), arg.process(sub, mono)),
            InstructionKind::Ref(dest, arg) => InstructionKind::Ref(dest.process(sub, mono), arg.process(sub, mono)),
            InstructionKind::Drop(args) => InstructionKind::Drop(args.clone()),
            InstructionKind::Jump(dest, block_id) => InstructionKind::Jump(dest.process(sub, mono), *block_id),
            InstructionKind::Assign(dest, rhs) => InstructionKind::Assign(dest.process(sub, mono), rhs.process(sub, mono)),
            InstructionKind::DeclareVar(var) => InstructionKind::DeclareVar(var.process(sub, mono)),
            InstructionKind::Transform(dest, root, index) => {
                InstructionKind::Transform(dest.process(sub, mono), root.process(sub, mono), index.clone())
            }
            InstructionKind::EnumSwitch(root, cases) => InstructionKind::EnumSwitch(root.process(sub, mono), cases.clone()),
            InstructionKind::IntegerSwitch(root, cases) => InstructionKind::IntegerSwitch(root.process(sub, mono), cases.clone()),
            InstructionKind::StringSwitch(root, cases) => InstructionKind::StringSwitch(root.process(sub, mono), cases.clone()),
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
                let slogan = format!("No {} function found", format!("{}", self.ctx.yellow(&main_name.toString())));
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
                    }
                }
                None => break,
            }
        }
    }

    fn monomorphizeFunction(&mut self, name: QualifiedName, args: Vec<Type>) {
        //println!("MONO FN: {} {}", name, formatTypes(&args));
        let function = self.program.functions.get(&name).expect("function not found in mono").clone();
        if let FunctionKind::TraitMemberDecl(_) = function.kind {
            return;
        }
        let params = function.constraintContext.typeParameters.iter().map(|ty| ty.clone()).collect();
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
            Type::Never => Type::Never,
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
}
