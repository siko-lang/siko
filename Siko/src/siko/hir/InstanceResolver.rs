use std::collections::BTreeMap;

use crate::siko::qualifiedname::QualifiedName;

use super::Trait::Instance;

#[derive(Clone)]
pub struct Instances {
    instances: Vec<Instance>,
}

impl Instances {
    pub fn new() -> Instances {
        Instances { instances: Vec::new() }
    }

    pub fn add(&mut self, instance: Instance) {
        self.instances.push(instance);
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
        let instances = self.traits.entry(instance.traitName.clone()).or_insert_with(|| Instances::new());
        instances.add(instance);
    }
}
