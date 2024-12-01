use std::collections::BTreeMap;

use crate::siko::qualifiedname::QualifiedName;

use super::{Trait::Instance, Type::Type};

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

    pub fn find(&self, types: &Vec<Type>) {
        for i in &self.instances {}
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
