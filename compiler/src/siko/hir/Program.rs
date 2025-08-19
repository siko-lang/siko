use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use crate::siko::{hir::Implicit::Implicit, qualifiedname::QualifiedName};

use super::{
    Data::{Enum, Struct},
    Function::Function,
    InstanceResolver::InstanceResolver,
    Trait::Trait,
    TraitMethodSelector::TraitMethodSelector,
};

#[derive(Clone)]
pub struct Program {
    pub functions: BTreeMap<QualifiedName, Function>,
    pub structs: BTreeMap<QualifiedName, Struct>,
    pub enums: BTreeMap<QualifiedName, Enum>,
    pub traits: BTreeMap<QualifiedName, Trait>,
    pub traitMethodSelectors: BTreeMap<QualifiedName, TraitMethodSelector>,
    pub instanceResolver: InstanceResolver,
    pub implicits: BTreeMap<QualifiedName, Implicit>,
    pub variants: BTreeSet<QualifiedName>,
}

impl Program {
    pub fn new() -> Program {
        Program {
            functions: BTreeMap::new(),
            structs: BTreeMap::new(),
            enums: BTreeMap::new(),
            traits: BTreeMap::new(),
            traitMethodSelectors: BTreeMap::new(),
            instanceResolver: InstanceResolver::new(),
            implicits: BTreeMap::new(),
            variants: BTreeSet::new(),
        }
    }

    pub fn getEnum(&self, qn: &QualifiedName) -> Option<Enum> {
        self.enums.get(qn).cloned()
    }

    pub fn getFunction(&self, qn: &QualifiedName) -> Option<Function> {
        self.functions.get(qn).cloned()
    }

    pub fn getStruct(&self, qn: &QualifiedName) -> Option<Struct> {
        self.structs.get(qn).cloned()
    }

    pub fn getTrait(&self, qn: &QualifiedName) -> Option<Trait> {
        self.traits.get(qn).cloned()
    }

    pub fn getImplicit(&self, qn: &QualifiedName) -> Option<Implicit> {
        self.implicits.get(qn).cloned()
    }

    pub fn isStruct(&self, qn: &QualifiedName) -> bool {
        self.structs.contains_key(qn)
    }

    pub fn isEnum(&self, qn: &QualifiedName) -> bool {
        self.enums.contains_key(qn)
    }

    pub fn isVariant(&self, qn: &QualifiedName) -> bool {
        self.variants.contains(qn)
    }

    pub fn dumpToFile(&self, folderName: &str) -> std::io::Result<()> {
        std::fs::create_dir_all(folderName)?;
        for (_, fun) in &self.functions {
            fun.dumpToFile(&format!(
                "{}/{}.txt",
                folderName,
                fun.name.to_string().replace(".", "_").replace("/", "_")
            ))?;
        }
        Ok(())
    }
}

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (_, fun) in &self.functions {
            writeln!(f, "{}", fun)?;
        }
        for (_, c) in &self.structs {
            writeln!(f, "{}", c)?;
        }
        for (_, e) in &self.enums {
            writeln!(f, "{}", e)?;
        }
        for (_, i) in &self.implicits {
            writeln!(f, "{}", i)?;
        }
        Ok(())
    }
}
