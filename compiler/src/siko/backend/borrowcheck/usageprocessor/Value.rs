use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use crate::siko::hir::BlockBuilder::InstructionRef;

#[derive(Clone, Debug)]
pub struct Usage {
    pub instruction: InstructionRef,
}

#[derive(Clone, Debug)]
struct ValueInfo {
    id: InstructionRef,
    fields: BTreeMap<String, Value>,
    usages: Vec<Usage>,
}

impl ValueInfo {
    fn new(id: InstructionRef) -> ValueInfo {
        ValueInfo {
            id,
            fields: BTreeMap::new(),
            usages: Vec::new(),
        }
    }

    fn addUsage(&mut self, usage: Usage) {
        self.usages.push(usage);
    }
}

#[derive(Clone, Debug)]
pub struct Value {
    info: Rc<RefCell<ValueInfo>>,
}

impl Value {
    pub fn new(id: InstructionRef) -> Value {
        Value {
            info: Rc::new(RefCell::new(ValueInfo::new(id))),
        }
    }

    pub fn addUsage(&self, usage: Usage) {
        self.info.borrow_mut().addUsage(usage);
    }
}
