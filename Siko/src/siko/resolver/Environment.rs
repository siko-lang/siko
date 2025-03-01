use std::collections::{BTreeMap, BTreeSet};

use crate::siko::hir::Variable::Variable;

pub struct Environment<'a> {
    values: BTreeMap<String, Variable>,
    parent: Option<&'a Environment<'a>>,
    mutables: BTreeSet<String>,
}

impl<'a> Environment<'a> {
    pub fn new() -> Environment<'a> {
        Environment {
            values: BTreeMap::new(),
            parent: None,
            mutables: BTreeSet::new(),
        }
    }

    pub fn child(parent: &'a Environment<'a>) -> Environment<'a> {
        Environment {
            values: BTreeMap::new(),
            parent: Some(parent),
            mutables: BTreeSet::new(),
        }
    }

    pub fn addArg(&mut self, arg: Variable, mutable: bool) {
        let name = arg.value.to_string();
        self.values.insert(arg.value.to_string(), arg);
        if mutable {
            self.mutables.insert(name);
        }
    }

    pub fn addValue(&mut self, old: String, new: Variable) {
        //println!("Added value {}", new);
        self.values.insert(old.clone(), new);
    }

    pub fn resolve(&self, value: &String) -> Option<Variable> {
        match self.values.get(value) {
            Some(v) => Some(v.clone()),
            None => {
                if let Some(parent) = self.parent {
                    return parent.resolve(value);
                } else {
                    None
                }
            }
        }
    }

    pub fn values(&self) -> &BTreeMap<String, Variable> {
        &self.values
    }

    pub fn isMutable(&self, name: &String) -> bool {
        self.mutables.contains(name)
    }
}
