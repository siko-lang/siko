use std::{cell::RefCell, rc::Rc};

use crate::siko::qualifiedname::QualifiedName;

#[derive(Clone, Debug)]
pub struct InstanceStore {
    pub localInstances: Vec<QualifiedName>,
    pub importedInstances: Vec<QualifiedName>,
}

impl InstanceStore {
    pub fn new() -> InstanceStore {
        InstanceStore {
            localInstances: Vec::new(),
            importedInstances: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct InstanceStorePtr {
    pub store: Rc<RefCell<InstanceStore>>,
}

impl InstanceStorePtr {
    pub fn new() -> InstanceStorePtr {
        InstanceStorePtr {
            store: Rc::new(RefCell::new(InstanceStore::new())),
        }
    }
}
