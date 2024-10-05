use std::collections::BTreeMap;

use crate::siko::{
    ir::{
        Function::{Function, FunctionKind, InstructionKind, ValueKind},
        Lifetime::Lifetime,
        Program::Program,
        Type::Type,
    },
    ownership::{
        DataFlow::DataFlowProfile::DataFlowProfile,
        Instantiator::LifetimeInstantiator,
        Substitution::{Apply, Substitution},
    },
    qualifiedname::QualifiedName,
};

use super::FunctionInferenceData::FunctionInferenceData;

enum Constraint {
    Equal(Lifetime, Lifetime),
}

pub struct FunctionGroupProcessor<'a> {
    profiles: &'a mut BTreeMap<QualifiedName, DataFlowProfile>,
    pub inferenceData: BTreeMap<QualifiedName, FunctionInferenceData>,
    group: Vec<QualifiedName>,
    program: &'a Program,
    instantiator: LifetimeInstantiator,
    deps: BTreeMap<Lifetime, Vec<Lifetime>>,
    constraints: Vec<Constraint>,
}

impl<'a> FunctionGroupProcessor<'a> {
    pub fn new(
        profiles: &'a mut BTreeMap<QualifiedName, DataFlowProfile>,
        group: Vec<QualifiedName>,
        program: &'a Program,
    ) -> FunctionGroupProcessor<'a> {
        FunctionGroupProcessor {
            profiles: profiles,
            inferenceData: BTreeMap::new(),
            group: group,
            program: program,
            instantiator: LifetimeInstantiator::new(),
            deps: BTreeMap::new(),
            constraints: Vec::new(),
        }
    }

    pub fn processGroup(&mut self) {
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
                    let data = FunctionInferenceData::new(item.clone(), profile);
                    self.inferenceData.insert(item.clone(), data);
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
                    let data = FunctionInferenceData::new(item.clone(), profile);
                    self.inferenceData.insert(item.clone(), data);
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
        self.apply();
    }

    fn apply(&mut self) {
        let mut sub = Substitution::new();
        for constraint in &self.constraints {
            match constraint {
                Constraint::Equal(l1, l2) => {
                    sub.add(l1, l2);
                }
            }
        }
        //let deps = self.deps.apply(&sub);
        //println!("Deps {:?}", deps);
        for (_, data) in &mut self.inferenceData {
            *data = data.apply(&sub);
        }
        //println!("DONE");
        //self.dump();
    }

    fn initializeTypes(&mut self, f: &Function) {
        let mut args = Vec::new();
        for param in &f.params {
            let ty = self.instantiateType(&param.getType());
            args.push(ty);
        }
        let result = self.instantiateType(&f.result);
        let profile = DataFlowProfile::new(args, result);
        let mut data = FunctionInferenceData::new(f.name.clone(), profile);
        for i in f.instructions() {
            let ty = self.instantiateType(i.ty.as_ref().expect("no type"));
            data.instruction_types.insert(i.id, ty);
            match &i.kind {
                InstructionKind::Bind(name, arg) => {
                    data.value_types
                        .insert(name.clone(), data.getInstructionType(*arg));
                }
                _ => {}
            }
        }
        self.inferenceData.insert(f.name.clone(), data);
    }

    fn unify(&mut self, ty1: &Type, ty2: &Type) {
        let ty1_lifetimes = ty1.collectLifetimes();
        let ty2_lifetimes = ty2.collectLifetimes();
        for (l1, l2) in ty1_lifetimes.into_iter().zip(ty2_lifetimes.into_iter()) {
            //println!("{} == {}", l1, l2);
            self.constraints.push(Constraint::Equal(l1, l2));
        }
    }

    fn dump(&self) {
        for (_, data) in &self.inferenceData {
            data.dump();
        }
    }

    fn collectConstraints(&mut self, f: &Function) {
        let data = self
            .inferenceData
            .get(&f.name)
            .expect("no profile found")
            .clone();
        //println!("Profile for {} {}", f.name, data.profile);
        let last = f.getFirstBlock().getLastId();
        let last_ty = data.getInstructionType(last);
        self.unify(&data.profile.result, &last_ty);
        for i in f.instructions() {
            let ty = data.getInstructionType(i.id);
            match &i.kind {
                InstructionKind::FunctionCall(name, args) => {
                    //println!("{}: {} {}", i.id, i.kind, ty);
                    let profile = if self.group.contains(name) {
                        self.inferenceData
                            .get(name)
                            .expect("inference data not found")
                            .profile
                            .clone()
                    } else {
                        let profile = self
                            .profiles
                            .get(name)
                            .expect("data flow profile not found");
                        let profile = self.instantiator.instantiate(profile);
                        //println!("profile {}", profile);
                        profile
                    };
                    for (arg1, arg2) in args.iter().zip(profile.args.iter()) {
                        let arg_ty = data.getInstructionType(*arg1);
                        self.unify(&arg_ty, arg2);
                    }
                    let result_ty = data.getInstructionType(i.id);
                    self.unify(&result_ty, &profile.result);
                }
                InstructionKind::ValueRef(value, _, indices) => {
                    //println!("{}: {} {}", i.id, i.kind, ty);
                    let mut current = match value {
                        ValueKind::Arg(_, index) => {
                            let arg = &data.profile.args[*index as usize];
                            arg.clone()
                        }
                        ValueKind::LoopVar(_) => todo!(),
                        ValueKind::Value(name, _) => data.getValueType(name),
                    };
                    for index in indices {
                        let c = self
                            .program
                            .getClass(&current.getName().expect("current is not a class"));
                        let sub = Substitution::from(&current, &c.ty);
                        let c = c.apply(&sub);
                        let field = &c.fields[*index as usize];
                        current = field.ty.clone();
                    }
                    let ty = data.getInstructionType(i.id);
                    //println!("value type {}, ty {}", current, ty);
                    self.unify(&ty, &current);
                }
                InstructionKind::Ref(arg) => {
                    //println!("{}: {} {}", i.id, i.kind, ty);
                    let arg_ty = data.getInstructionType(*arg);
                    let arg_lifetimes = arg_ty.collectLifetimes();
                    let ref_lifetimes = ty.collectLifetimes();
                    let ref_arg_lifetimes: Vec<_> = ref_lifetimes.into_iter().skip(1).collect();
                    // println!("ref ty {}, arg_ty {}", ty, arg_ty);
                    // println!(
                    //     "ref lt {:?}, ref {:?} lt {:?}",
                    //     ref_lifetimes, ref_arg_lifetimes, arg_lifetimes
                    // );
                    for (l1, l2) in ref_arg_lifetimes.into_iter().zip(arg_lifetimes.into_iter()) {
                        //println!("{} == {}", l1, l2);
                        self.constraints.push(Constraint::Equal(l1, l2));
                    }
                }
                InstructionKind::Tuple(_) => {
                    //println!("{}: {} {}", i.id, i.kind, ty);
                }
                InstructionKind::Bind(_, _) => {
                    //println!("{}: {} {}", i.id, i.kind, ty);
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
