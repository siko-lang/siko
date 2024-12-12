use std::{collections::BTreeMap, fmt::Display};

use crate::siko::qualifiedname::QualifiedName;

use super::{
    Data::{Class, Enum},
    Function::Function,
    InstanceResolver::InstanceResolver,
    Trait::Trait,
    TraitMethodSelector::TraitMethodSelector,
};

#[derive(Clone)]
pub struct Program {
    pub functions: BTreeMap<QualifiedName, Function>,
    pub classes: BTreeMap<QualifiedName, Class>,
    pub enums: BTreeMap<QualifiedName, Enum>,
    pub traits: BTreeMap<QualifiedName, Trait>,
    pub traitMethodSelectors: BTreeMap<QualifiedName, TraitMethodSelector>,
    pub instanceResolver: InstanceResolver,
}

impl Program {
    pub fn new() -> Program {
        Program {
            functions: BTreeMap::new(),
            classes: BTreeMap::new(),
            enums: BTreeMap::new(),
            traits: BTreeMap::new(),
            traitMethodSelectors: BTreeMap::new(),
            instanceResolver: InstanceResolver::new(),
        }
    }

    pub fn getEnum(&self, qn: &QualifiedName) -> Enum {
        self.enums.get(qn).expect("enum not found").clone()
    }

    pub fn getFunction(&self, qn: &QualifiedName) -> Function {
        self.functions.get(qn).expect("function not found").clone()
    }

    pub fn getClass(&self, qn: &QualifiedName) -> Class {
        self.classes.get(qn).expect("class not found").clone()
    }

    pub fn getTrait(&self, qn: &QualifiedName) -> Option<Trait> {
        self.traits.get(qn).cloned()
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
