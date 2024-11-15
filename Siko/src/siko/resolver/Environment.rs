use std::collections::BTreeMap;

use crate::siko::hir::Function::Variable;

pub struct Environment<'a> {
    values: BTreeMap<String, Variable>,
    parent: Option<&'a Environment<'a>>,
}

impl<'a> Environment<'a> {
    pub fn new() -> Environment<'a> {
        Environment {
            values: BTreeMap::new(),
            parent: None,
        }
    }

    pub fn child(parent: &'a Environment<'a>) -> Environment<'a> {
        Environment {
            values: BTreeMap::new(),
            parent: Some(parent),
        }
    }

    pub fn addArg(&mut self, arg: Variable) {
        self.values.insert(arg.value.clone(), arg);
    }

    pub fn addValue(&mut self, old: String, new: Variable) {
        //println!("Added value {}", new);
        self.values.insert(old.clone(), new);
    }

    pub fn addTmpValue(&mut self, var: Variable) {
        //println!("Added value {}", new);
        self.values.insert(var.value.clone(), var);
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
}
