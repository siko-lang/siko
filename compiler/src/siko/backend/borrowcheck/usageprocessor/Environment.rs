use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use crate::siko::{
    backend::borrowcheck::usageprocessor::Value::{Usage, Value},
    hir::Variable::VariableName,
};

#[derive(Clone)]
pub struct Environment {
    values: Rc<RefCell<BTreeMap<VariableName, Value>>>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            values: Rc::new(RefCell::new(BTreeMap::new())),
        }
    }

    pub fn addValue(&self, var: VariableName, value: Value) {
        self.values.borrow_mut().insert(var, value);
    }

    pub fn addUsage(&self, var: &VariableName, usage: Usage) {
        if let Some(v) = self.values.borrow().get(var) {
            v.addUsage(usage);
        } else {
            panic!("No value for variable {}", var);
        }
    }

    pub fn snapshot(&self) -> Environment {
        let copy = self.values.borrow().clone();
        Environment {
            values: Rc::new(RefCell::new(copy)),
        }
    }

    pub fn merge(&self, other: &Environment) -> bool {
        let mut changed = false;
        changed
    }
}
