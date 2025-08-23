use crate::siko::qualifiedname::QualifiedName;

#[derive(Clone, Debug)]
pub struct ImplementationStore {
    pub localImplementations: Vec<QualifiedName>,
    pub importedImplementations: Vec<QualifiedName>,
}

impl ImplementationStore {
    pub fn new() -> ImplementationStore {
        ImplementationStore {
            localImplementations: Vec::new(),
            importedImplementations: Vec::new(),
        }
    }
}
