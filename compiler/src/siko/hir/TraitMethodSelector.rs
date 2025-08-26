use crate::siko::qualifiedname::QualifiedName;

use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct TraitMethodSelection {
    pub traitName: QualifiedName,
    pub method: QualifiedName,
}

#[derive(Clone, Debug)]
pub struct TraitMethodSelector {
    methods: BTreeMap<String, Vec<TraitMethodSelection>>,
}

impl TraitMethodSelector {
    pub fn new() -> TraitMethodSelector {
        TraitMethodSelector {
            methods: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, name: String, selection: TraitMethodSelection) {
        let selections = self.methods.entry(name).or_insert_with(|| Vec::new());
        selections.push(selection);
    }

    pub fn get(&self, field: &String) -> Option<Vec<TraitMethodSelection>> {
        self.methods.get(field).cloned()
    }

    pub fn merge(&mut self, other: &TraitMethodSelector) {
        for (name, selections) in &other.methods {
            for selection in selections {
                self.add(name.clone(), selection.clone());
            }
        }
    }
}
