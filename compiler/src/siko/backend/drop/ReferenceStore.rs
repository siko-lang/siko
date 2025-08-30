use std::collections::BTreeSet;

use crate::siko::hir::Variable::VariableName;

pub struct ReferenceStore {
    references: BTreeSet<VariableName>,
}

impl ReferenceStore {
    pub fn new() -> Self {
        ReferenceStore {
            references: BTreeSet::new(),
        }
    }

    pub fn addReference(&mut self, var_name: VariableName) {
        self.references.insert(var_name);
    }

    pub fn isReference(&self, var_name: &VariableName) -> bool {
        self.references.contains(var_name)
    }
}
