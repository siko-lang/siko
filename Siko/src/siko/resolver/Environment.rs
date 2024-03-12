use std::collections::BTreeMap;

pub enum ValueKind {
    Arg(String),
    Value(String),
}

pub struct Environment<'a> {
    values: BTreeMap<String, ValueKind>,
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

    pub fn addArg(&mut self, arg: String) {
        self.values.insert(arg.clone(), ValueKind::Arg(arg));
    }

    pub fn addValue(&mut self, old: String, new: String) {
        //println!("Added value {}", new);
        self.values.insert(old.clone(), ValueKind::Value(new));
    }

    pub fn resolve(&self, value: &String) -> Option<String> {
        match self.values.get(value) {
            Some(ValueKind::Arg(v)) => Some(v.clone()),
            Some(ValueKind::Value(v)) => Some(v.clone()),
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
