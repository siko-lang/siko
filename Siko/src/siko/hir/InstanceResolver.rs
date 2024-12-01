use std::{cmp::Ordering, collections::BTreeMap};

use crate::siko::{hir::Trait::CompareSpecificity, qualifiedname::QualifiedName};

use super::{
    Apply::{instantiateInstance, instantiateType2},
    Substitution::TypeSubstitution,
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

    pub fn find(&self, allocator: &mut TypeVarAllocator, types: &Vec<Type>) {
        let mut matchingInstances = Vec::new();
        for i in &self.instances {
            let i = instantiateInstance(allocator, i);
            let mut sub = TypeSubstitution::new();
            let mut noMatch = false;
            for (arg, ty) in i.types.iter().zip(types.iter()) {
                let r = unify(&mut sub, arg, ty);
                if r.is_err() {
                    noMatch = true;
                    break;
                }
            }
            if noMatch {
                continue;
            }
            println!("Found matching instance!");
            println!("{}", i);
            matchingInstances.push(i);
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
            println!("winner {}", winner);
        } else {
            if matchingInstances.is_empty() {
                println!("No instance found");
            } else {
                println!("Ambigous instances");
            }
        }
    }
}

#[derive(Clone)]
pub struct InstanceResolver {
    traits: BTreeMap<QualifiedName, Instances>,
}

impl InstanceResolver {
    pub fn new() -> InstanceResolver {
        InstanceResolver { traits: BTreeMap::new() }
    }

    pub fn addInstance(&mut self, instance: Instance) {
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
}
