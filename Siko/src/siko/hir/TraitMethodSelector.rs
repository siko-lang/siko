use crate::siko::{location::Report::ReportContext, qualifiedname::QualifiedName, resolver::Error::ResolverError, syntax::Identifier::Identifier};
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct TraitMethodSelection {
    pub traitName: QualifiedName,
    pub method: QualifiedName,
}

#[derive(Clone)]
pub struct TraitMethodSelector {
    methods: BTreeMap<String, TraitMethodSelection>,
}

impl TraitMethodSelector {
    pub fn new() -> TraitMethodSelector {
        TraitMethodSelector { methods: BTreeMap::new() }
    }

    pub fn add(&mut self, ctx: &ReportContext, name: Identifier, selection: TraitMethodSelection) {
        let p = self.methods.insert(name.toString(), selection);
        if p.is_some() {
            ResolverError::Ambiguous(name.toString(), name.location.clone()).report(ctx);
        }
    }

    pub fn get(&self, field: &String) -> Option<TraitMethodSelection> {
        self.methods.get(field).cloned()
    }
}
