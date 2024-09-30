use std::{
    collections::BTreeMap,
    fmt::{Debug, Display},
};

use crate::siko::{
    ir::{
        Function::{Function, FunctionKind, InstructionId, InstructionKind, ValueKind},
        Program::Program,
        Type::Type,
    },
    ownership::{
        DataFlowProfile::DataFlowProfile,
        Instantiator::LifetimeInstantiator,
        Substitution::{Apply, Substitution},
    },
    qualifiedname::QualifiedName,
    util::Instantiator::Instantiable,
};

use super::FunctionGroups;

pub struct DataFlowProfileBuilder<'a> {
    profiles: BTreeMap<QualifiedName, DataFlowProfile>,
    program: &'a Program,
}

impl<'a> DataFlowProfileBuilder<'a> {
    pub fn new(program: &'a Program) -> DataFlowProfileBuilder<'a> {
        DataFlowProfileBuilder {
            profiles: BTreeMap::new(),
            program: program,
        }
    }

    pub fn process(&mut self) {
        let function_groups = FunctionGroups::createFunctionGroups(&self.program.functions);
        for group in function_groups {
            println!("Processing function group {:?}", group.items);
            let mut processor =
                FunctionGroupProcessor::new(&mut self.profiles, group.items, self.program);
            processor.processGroup();
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct GlobalInstructionId {
    name: QualifiedName,
    id: InstructionId,
}

impl Display for GlobalInstructionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.name, self.id)
    }
}

impl Debug for GlobalInstructionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.name, self.id)
    }
}

struct FunctionGroupProcessor<'a> {
    profiles: &'a mut BTreeMap<QualifiedName, DataFlowProfile>,
    group: Vec<QualifiedName>,
    program: &'a Program,
    instantiator: LifetimeInstantiator,
    instruction_types: BTreeMap<GlobalInstructionId, Type>,
}

impl<'a> FunctionGroupProcessor<'a> {
    fn new(
        profiles: &'a mut BTreeMap<QualifiedName, DataFlowProfile>,
        group: Vec<QualifiedName>,
        program: &'a Program,
    ) -> FunctionGroupProcessor<'a> {
        FunctionGroupProcessor {
            profiles: profiles,
            group: group,
            program: program,
            instantiator: LifetimeInstantiator::new(),
            instruction_types: BTreeMap::new(),
        }
    }

    fn processGroup(&mut self) {
        for item in self.group.clone() {
            let f = self
                .program
                .functions
                .get(&item)
                .expect("Function not found in DataFlowProfileBuilder");
            match f.kind {
                FunctionKind::UserDefined => {
                    self.initializeTypes(f);
                }
                FunctionKind::VariantCtor(index) => {
                    let eName = f.result.getName().expect("no result type");
                    let e = self.program.getEnum(&eName);
                    let variant = &e.variants[index as usize];
                    let mut args = Vec::new();
                    for ty in &variant.items {
                        args.push(ty.clone());
                    }
                    let profile = DataFlowProfile::new(args, e.ty.clone());
                    self.profiles.insert(item.clone(), profile);
                    //println!("profile {}", profile);
                }
                FunctionKind::ClassCtor => {
                    let cName = f.result.getName().expect("no result type");
                    let c = self.program.getClass(&cName);
                    //println!("{}", c);
                    let mut args = Vec::new();
                    for f in c.fields {
                        args.push(f.ty.clone());
                    }
                    let profile = DataFlowProfile::new(args, c.ty.clone());
                    self.profiles.insert(item.clone(), profile);
                    //println!("profile {}", profile);
                }
            }
        }
        for item in self.group.clone() {
            let f = self
                .program
                .functions
                .get(&item)
                .expect("Function not found in DataFlowProfileBuilder");
            match f.kind {
                FunctionKind::UserDefined => {
                    self.collectConstraints(f);
                }
                _ => {}
            }
        }
    }

    fn initializeTypes(&mut self, f: &Function) {
        let mut args = Vec::new();
        for param in &f.params {
            let ty = self.instantiateType(&param.getType());
            args.push(ty);
        }
        let result = self.instantiateType(&f.result);
        let profile = DataFlowProfile::new(args, result);
        self.profiles.insert(f.name.clone(), profile);
        for i in f.instructions() {
            let ty = self.instantiateType(i.ty.as_ref().expect("no type"));
            let id = GlobalInstructionId {
                name: f.name.clone(),
                id: i.id,
            };
            self.instruction_types.insert(id, ty);
        }
    }

    fn collectConstraints(&mut self, f: &Function) {
        let profile = self.profiles.get(&f.name).expect("no profile found");
        println!("Profile for {} {}", f.name, profile);
        for i in f.instructions() {
            let id = GlobalInstructionId {
                name: f.name.clone(),
                id: i.id,
            };
            let ty = self
                .instruction_types
                .get(&id)
                .expect("no instruction type");
            match &i.kind {
                InstructionKind::FunctionCall(name, args) => {
                    println!("{}: {} {}", id, i.kind, ty);
                    if self.group.contains(name) {
                        panic!("Recursion NYI");
                    } else {
                        let profile = self
                            .profiles
                            .get(name)
                            .expect("data flow profile not found");
                        let profile = self.instantiator.instantiate(profile);
                        println!("profile {}", profile);
                    }
                }
                InstructionKind::ValueRef(ValueKind::Arg(_, index), _, indices) => {
                    println!("{}: {} {}", id, i.kind, ty);
                    let arg = &profile.args[*index as usize];
                    let mut current = arg.clone();
                    for index in indices {
                        let c = self
                            .program
                            .getClass(&current.getName().expect("current is not a class"));
                        let sub = Substitution::from(&current, &c.ty);
                        let c = c.apply(&sub);
                        let field = &c.fields[*index as usize];
                        current = field.ty.clone();
                    }
                    //println!("value type {}", current);
                    let dest_lifetimes = current.collectLifetimes();
                    let src_lifetimes = arg.collectLifetimes();
                    //println!("{:?} -> {:?}", src_lifetimes, dest_lifetimes);
                    for (src, dest) in src_lifetimes.iter().zip(dest_lifetimes.iter()) {
                        println!("{} -> {}", src, dest);
                    }
                }
                InstructionKind::Ref(arg) => {
                    println!("{}: {} {}", id, i.kind, ty);
                    let id = GlobalInstructionId {
                        name: f.name.clone(),
                        id: *arg,
                    };
                    let arg_ty = self
                        .instruction_types
                        .get(&id)
                        .expect("no instruction type");
                    let arg_lifetimes = arg_ty.collectLifetimes();
                    let ref_lifetimes = ty.collectLifetimes();
                    let ref_arg_lifetimes: Vec<_> = ref_lifetimes.iter().skip(1).collect();
                    // println!("ref ty {}, arg_ty {}", ty, arg_ty);
                    // println!(
                    //     "ref lt {:?}, ref {:?} lt {:?}",
                    //     ref_lifetimes, ref_arg_lifetimes, arg_lifetimes
                    // );
                    for (l1, l2) in ref_arg_lifetimes.iter().zip(arg_lifetimes.iter()) {
                        println!("{} == {}", l1, l2);
                    }
                }
                _ => panic!("NYI {}", i.kind),
            }
        }
    }

    fn instantiateType(&mut self, ty: &Type) -> Type {
        match ty {
            Type::Named(name, _, _) => {
                if let Some(c) = self.program.classes.get(&name) {
                    let c = self.instantiator.instantiate(c);
                    self.instantiator.reset();
                    c.ty
                } else {
                    if let Some(e) = self.program.enums.get(&name) {
                        let e = self.instantiator.instantiate(e);
                        self.instantiator.reset();
                        e.ty
                    } else {
                        unreachable!()
                    }
                }
            }
            Type::Tuple(args) => {
                Type::Tuple(args.iter().map(|ty| self.instantiateType(ty)).collect())
            }
            Type::Function(_, _) => unreachable!(),
            Type::Var(_) => unreachable!(),
            Type::Reference(ty, _) => {
                let ty = self.instantiateType(ty);
                let l = self.instantiator.allocate();
                Type::Reference(Box::new(ty), Some(l))
            }
            Type::SelfType => Type::SelfType,
            Type::Never => Type::Never,
        }
    }
}
