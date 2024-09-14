use std::{collections::BTreeMap, fmt::Display};

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

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (_, fun) in &self.functions {
            writeln!(f, "{}", fun)?;
        }
        for (_, c) in &self.classes {
            writeln!(f, "{}", c)?;
        }
        for (_, e) in &self.enums {
            writeln!(f, "{}", e)?;
        }
        Ok(())
    }
}
