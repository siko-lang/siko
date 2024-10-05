use crate::siko::{
    qualifiedname::QualifiedName, resolver::Error::ResolverError, syntax::Identifier::Identifier,
};
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct TraitMethodSelector {
    methods: BTreeMap<String, QualifiedName>,
}

impl TraitMethodSelector {
    pub fn new() -> TraitMethodSelector {
        TraitMethodSelector {
            methods: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, name: Identifier, method: QualifiedName) {
        let p = self.methods.insert(name.toString(), method);
        if p.is_some() {
            ResolverError::Ambiguous(name.toString(), name.location.clone()).report();
        }
    }

    pub fn get(&self, field: &String) -> Option<QualifiedName> {
        self.methods.get(field).cloned()
    }
}
