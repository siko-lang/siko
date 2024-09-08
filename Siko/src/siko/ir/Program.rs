use std::collections::BTreeMap;

use crate::siko::qualifiedname::QualifiedName;

use super::{
    Data::{Class, Enum},
    Function::Function,
    TraitMethodSelector::TraitMethodSelector,
};

pub struct Program {
    pub functions: BTreeMap<QualifiedName, Function>,
    pub classes: BTreeMap<QualifiedName, Class>,
    pub enums: BTreeMap<QualifiedName, Enum>,
    pub traitMethodSelectors: BTreeMap<QualifiedName, TraitMethodSelector>,
}

impl Program {
    pub fn new() -> Program {
        Program {
            functions: BTreeMap::new(),
            classes: BTreeMap::new(),
            enums: BTreeMap::new(),
            traitMethodSelectors: BTreeMap::new(),
        }
    }
}
