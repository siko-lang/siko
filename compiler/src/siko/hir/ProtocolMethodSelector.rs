use crate::siko::qualifiedname::QualifiedName;

use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct ProtocolMethodSelection {
    pub ProtocolName: QualifiedName,
    pub method: QualifiedName,
}

#[derive(Clone, Debug)]
pub struct ProtocolMethodSelector {
    methods: BTreeMap<String, Vec<ProtocolMethodSelection>>,
}

impl ProtocolMethodSelector {
    pub fn new() -> ProtocolMethodSelector {
        ProtocolMethodSelector {
            methods: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, name: String, selection: ProtocolMethodSelection) {
        let selections = self.methods.entry(name).or_insert_with(|| Vec::new());
        selections.push(selection);
    }

    pub fn get(&self, field: &String) -> Option<Vec<ProtocolMethodSelection>> {
        self.methods.get(field).cloned()
    }

    pub fn merge(&mut self, other: &ProtocolMethodSelector) {
        for (name, selections) in &other.methods {
            for selection in selections {
                self.add(name.clone(), selection.clone());
            }
        }
    }
}
