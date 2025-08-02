use std::collections::{BTreeMap, BTreeSet};

use crate::siko::hir::{
    Instruction::SyntaxBlockId,
    Variable::{Variable, VariableName},
};

pub struct DeclarationStore {
    vars: BTreeMap<VariableName, Variable>,
    declarations: BTreeMap<VariableName, SyntaxBlockId>,
    blockDeclarations: BTreeMap<SyntaxBlockId, BTreeSet<VariableName>>,
}

impl DeclarationStore {
    pub fn new() -> DeclarationStore {
        DeclarationStore {
            vars: BTreeMap::new(),
            declarations: BTreeMap::new(),
            blockDeclarations: BTreeMap::new(),
        }
    }

    pub fn declare(&mut self, var: Variable, syntaxBlockId: SyntaxBlockId) -> bool {
        if self.declarations.contains_key(&var.value) {
            return false;
        }
        self.declarations.insert(var.value.clone(), syntaxBlockId.clone());
        self.blockDeclarations
            .entry(syntaxBlockId)
            .or_insert_with(BTreeSet::new)
            .insert(var.value.clone());
        self.vars.insert(var.value.clone(), var);
        true
    }

    pub fn getDeclarations(&self, syntaxBlockId: &SyntaxBlockId) -> Option<BTreeSet<Variable>> {
        self.blockDeclarations
            .get(syntaxBlockId)
            .map(|names| names.iter().filter_map(|name| self.vars.get(name).cloned()).collect())
    }

    pub fn dump(&self) {
        for (name, syntaxBlockId) in &self.declarations {
            println!("Declared {} in {}", name, syntaxBlockId);
        }

        for (syntaxBlockId, names) in &self.blockDeclarations {
            println!("In block {}: {:?}", syntaxBlockId, names.iter().collect::<Vec<_>>());
        }
    }
}
