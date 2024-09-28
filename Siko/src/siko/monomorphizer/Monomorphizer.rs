use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt::Display,
};

use crate::siko::{
    ir::{
        Function::{Body, Instruction, InstructionKind, Parameter},
        Program::Program,
        Substitution::Substitution,
        Type::{formatTypes, Type},
    },
    location::Report::{Painter, Report},
    qualifiedname::{build, QualifiedName},
};

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

pub struct Monomorphizer {
    program: Program,
    monomorphizedProgram: Program,
    queue: VecDeque<Key>,
    processed: BTreeSet<Key>,
    processed_type: BTreeMap<Type, Type>,
}

impl Monomorphizer {
    pub fn new(program: Program) -> Monomorphizer {
        Monomorphizer {
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
                let slogan = format!("No {} function found", format!("{}", main_name).yellow());
                let r = Report::new(slogan, None);
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
                    self.processed.insert(key.clone());
                    match key {
                        Key::Function(name, args) => {
                            self.monomorphizeFunction(name, args);
                        }
                        Key::Class(name, args) => {
                            self.monomorphizeClass(name, args);
                        }
                        Key::Enum(name, args) => {
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
        let function = self
            .program
            .functions
            .get(&name)
            .expect("function not found in mono")
            .clone();
        let sub = Substitution::createFrom(&function.constraintContext.typeParameters, &args);
        let mut monoFn = function.clone();
        monoFn.result = self.processType(sub.apply(&monoFn.result));
        monoFn.params = monoFn
            .params
            .into_iter()
            .map(|param| match param {
                Parameter::Named(name, ty, mutable) => {
                    Parameter::Named(name, self.processType(sub.apply(&ty)), mutable)
                }
                Parameter::SelfParam(mutable, ty) => {
                    Parameter::SelfParam(mutable, self.processType(sub.apply(&ty)))
                }
            })
            .collect();
        monoFn.body = monoFn.body.map(|mut body| {
            body.blocks = body
                .blocks
                .into_iter()
                .map(|mut block| {
                    block.instructions = block
                        .instructions
                        .into_iter()
                        .map(|instruction| {
                            self.monomorphizeInstruction(
                                &sub,
                                function.body.as_ref().unwrap(),
                                instruction,
                            )
                        })
                        .collect();
                    block
                })
                .collect();
            body
        });
        let monoName = self.get_mono_name(&name, &args);
        monoFn.name = monoName.clone();
        self.monomorphizedProgram.functions.insert(monoName, monoFn);
    }

    fn monomorphizeInstruction(
        &mut self,
        sub: &Substitution,
        body: &Body,
        mut instruction: Instruction,
    ) -> Instruction {
        // println!(
        //     "MONO INSTR {} / {}",
        //     instruction,
        //     instruction.ty.clone().unwrap()
        // );
        let kind: InstructionKind = match &instruction.kind {
            InstructionKind::FunctionCall(name, args) => {
                //println!("Calling {}", name);
                let target_fn = self
                    .program
                    .functions
                    .get(name)
                    .expect("function not found in mono");
                let fn_ty = target_fn.getType();
                let arg_types: Vec<_> = args
                    .iter()
                    .map(|id| {
                        let ty = body
                            .getInstruction(*id)
                            .ty
                            .clone()
                            .expect("instruction with no type");
                        sub.apply(&ty)
                    })
                    .collect();
                let result =
                    sub.apply(&instruction.ty.clone().expect("function with no result ty"));
                let context_ty = Type::Function(arg_types, Box::new(result));
                //println!("fn type {}", fn_ty);
                //println!("context type {}", context_ty);
                let sub = Substitution::create(&context_ty, &fn_ty);
                let ty_args: Vec<_> = target_fn
                    .constraintContext
                    .typeParameters
                    .iter()
                    .map(|ty| sub.apply(&ty))
                    .collect();
                //println!("{} type args {}", name, formatTypes(&ty_args));
                let fn_name = self.get_mono_name(name, &ty_args);
                self.addKey(Key::Function(name.clone(), ty_args));
                InstructionKind::FunctionCall(fn_name, args.clone())
            }
            k => k.clone(),
        };
        instruction.kind = kind;
        instruction.ty = instruction.ty.map(|ty| self.processType(sub.apply(&ty)));
        instruction
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
        let c = self
            .program
            .classes
            .get(&name)
            .expect("class not found in mono");
        let name = self.get_mono_name(&name, &args);
        let mut mono_c = c.clone();
        mono_c.ty = self.processType(mono_c.ty);
        mono_c.fields = mono_c
            .fields
            .iter()
            .cloned()
            .map(|mut f| {
                f.ty = self.processType(f.ty);
                f
            })
            .collect();
        mono_c.methods.clear();
        mono_c.name = name.clone();
        self.monomorphizedProgram.classes.insert(name, mono_c);
    }

    fn monomorphizeEnum(&mut self, name: QualifiedName, args: Vec<Type>) {
        //println!("MONO ENUM: {} {}", name, formatTypes(&args));
        let e = self
            .program
            .enums
            .get(&name)
            .expect("enum not found in mono");
        //println!("Enum ty {}", e.ty);
        let monoName = self.get_mono_name(&name, &args);
        let target_ty = Type::Named(name, args.clone(), None);
        let sub = Substitution::create(&target_ty, &e.ty);
        //println!("Sub {}", sub);
        let mut mono_e = e.clone();
        mono_e.ty = self.processType(sub.apply(&mono_e.ty));
        mono_e.variants = mono_e
            .variants
            .iter()
            .cloned()
            .map(|mut v| {
                v.items = v
                    .items
                    .into_iter()
                    .map(|i| self.processType(sub.apply(&i)))
                    .collect();
                v
            })
            .collect();
        mono_e.methods.clear();
        mono_e.name = monoName.clone();
        self.monomorphizedProgram.enums.insert(monoName, mono_e);
    }
}
