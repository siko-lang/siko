use std::collections::BTreeMap;

use crate::siko::interpreter::Value::Value;

pub struct Frame {
    pub variables: BTreeMap<String, Value>,
}

impl Frame {
    pub fn new() -> Self {
        Frame {
            variables: BTreeMap::new(),
        }
    }

    pub fn bind(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn get(&self, name: &String) -> Option<&Value> {
        self.variables.get(name)
    }
}
