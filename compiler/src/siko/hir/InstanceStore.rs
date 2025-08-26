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
