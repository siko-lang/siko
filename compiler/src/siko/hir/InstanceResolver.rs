use std::{cmp::Ordering, collections::BTreeMap};

use crate::siko::{
    hir::{Instantiation::instantiateInstance, Trait::CompareSpecificity},
    qualifiedname::{
        builtins::{getCopyName, getDerefName, getImplicitConvertName},
        QualifiedName,
    },
};

use super::{
    Apply::Apply,
    Substitution::{createTypeSubstitutionFrom, Substitution},
    Trait::Instance,
    Type::Type,
    TypeVarAllocator::TypeVarAllocator,
    Unification::unify,
};

#[derive(Clone)]
pub struct Instances {
    traitName: QualifiedName,
    instances: Vec<Instance>,
}

pub enum ResolutionResult {
    Winner(Instance),
    Ambiguous(Vec<Instance>),
    NoInstanceFound,
}

impl Instances {
    pub fn new(traitName: QualifiedName) -> Instances {
        Instances {
            traitName: traitName,
            instances: Vec::new(),
        }
    }

    pub fn add(&mut self, instance: Instance) {
        self.instances.push(instance);
    }

    pub fn find(&self, allocator: &mut TypeVarAllocator, types: &Vec<Type>) -> ResolutionResult {
        let mut matchingInstances = Vec::new();
        for i in &self.instances {
            let i2 = instantiateInstance(allocator, i);
            let mut sub = Substitution::new();
            let mut noMatch = false;
            //println!("Matching {} {}", formatTypes(types), formatTypes(&i.types));
            for (arg, ty) in i2.types.iter().zip(types.iter()) {
                let r = unify(&mut sub, arg.clone(), ty.clone(), false);
                if r.is_err() {
                    //println!("no match");
                    noMatch = true;
                    break;
                }
            }
            if noMatch {
                continue;
            }
            matchingInstances.push(i.clone());
        }
        let mut winner: Option<&Instance> = None;
        let mut index = 0;
        while index < matchingInstances.len() {
            let current = &matchingInstances[index];
            match &winner {
                Some(w) => {
                    let mut r = CompareSpecificity::compare(&w.types, &current.types);
                    if r.len() > 1 {
                        r.remove(&Ordering::Equal);
                    }
                    if r.len() == 1 {
                        if r.contains(&Ordering::Equal) {
                            winner = None;
                        }
                        if r.contains(&Ordering::Less) {
                            winner = Some(current);
                        }
                    } else {
                        // ambiguous
                        winner = None;
                    }
                }
                None => {
                    winner = Some(current);
                }
            }
            index += 1;
        }
        if let Some(winner) = winner {
            let winner = instantiateInstance(allocator, winner);
            let sub = createTypeSubstitutionFrom(winner.types.clone(), types.clone());
            ResolutionResult::Winner(winner.apply(&sub))
        } else {
            if matchingInstances.is_empty() {
                ResolutionResult::NoInstanceFound
            } else {
                ResolutionResult::Ambiguous(matchingInstances)
            }
        }
    }

    pub fn dump(&self) {
        println!("Instances for trait {}:", self.traitName);
        for instance in &self.instances {
            println!("{}", instance);
        }
    }
}

#[derive(Clone)]
pub struct InstanceResolver {
    traits: BTreeMap<QualifiedName, Instances>,
}

impl InstanceResolver {
    pub fn new() -> InstanceResolver {
        InstanceResolver {
            traits: BTreeMap::new(),
        }
    }

    pub fn addInstance(&mut self, instance: Instance) {
        //println!("Add instance {}", instance);
        let instances = self
            .traits
            .entry(instance.traitName.clone())
            .or_insert_with(|| Instances::new(instance.traitName.clone()));
        instances.add(instance);
    }

    pub fn lookupInstances(&self, traitName: &QualifiedName) -> Option<&Instances> {
        if let Some(instances) = self.traits.get(traitName) {
            Some(instances)
        } else {
            None
        }
    }

    pub fn isCopy(&self, ty: &Type) -> bool {
        if let Some(instances) = self.lookupInstances(&getCopyName()) {
            let mut allocator = TypeVarAllocator::new();
            let result = instances.find(&mut allocator, &vec![ty.clone()]);
            if let ResolutionResult::Winner(_) = result {
                // println!("Copy found for {}", ty);
                return true;
            }
        }
        false
    }

    pub fn isDeref(&self, ty: &Type) -> Option<Instance> {
        if let Some(instances) = self.lookupInstances(&getDerefName()) {
            let mut allocator = TypeVarAllocator::new();
            let result = instances.find(&mut allocator, &vec![ty.clone()]);
            if let ResolutionResult::Winner(i) = result {
                return Some(i);
            }
        }
        None
    }

    pub fn isImplicitConvert(&self, ty1: &Type, ty2: &Type) -> bool {
        if ty1.isGeneric() || ty2.isGeneric() {
            return false;
        }
        if let Some(instances) = self.lookupInstances(&getImplicitConvertName()) {
            let mut allocator = TypeVarAllocator::new();
            let result = instances.find(&mut allocator, &vec![ty1.clone(), ty2.clone()]);
            if let ResolutionResult::Winner(_) = result {
                //println!("Copy found for {}", prevUsage.var);
                return true;
            }
        }
        false
    }

    pub fn dump(&self) {
        println!("InstanceResolver:");
        for (_, instances) in &self.traits {
            instances.dump();
        }
        println!("End of InstanceResolver");
    }
}
