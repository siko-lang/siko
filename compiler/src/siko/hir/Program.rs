use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use crate::siko::{
    hir::{
        CanonicalInstanceStore::CanonicalInstanceStore,
        Implicit::Implicit,
        InstanceStore::InstanceStorePtr,
        Trait::{Instance, Trait},
        TraitMethodSelector::TraitMethodSelector,
    },
    qualifiedname::QualifiedName,
};

use super::{
    Data::{Enum, Struct},
    Function::Function,
};

#[derive(Clone)]
pub struct Program {
    pub functions: BTreeMap<QualifiedName, Function>,
    pub structs: BTreeMap<QualifiedName, Struct>,
    pub enums: BTreeMap<QualifiedName, Enum>,
    pub traitMethodselectors: BTreeMap<QualifiedName, TraitMethodSelector>,
    pub implicits: BTreeMap<QualifiedName, Implicit>,
    pub variants: BTreeSet<QualifiedName>,
    pub traits: BTreeMap<QualifiedName, Trait>,
    pub instances: BTreeMap<QualifiedName, Instance>,
    pub instanceStores: BTreeMap<QualifiedName, InstanceStorePtr>,
    pub canonicalImplStore: CanonicalInstanceStore,
}

impl Program {
    pub fn new() -> Program {
        Program {
            functions: BTreeMap::new(),
            structs: BTreeMap::new(),
            enums: BTreeMap::new(),
            traitMethodselectors: BTreeMap::new(),
            implicits: BTreeMap::new(),
            variants: BTreeSet::new(),
            traits: BTreeMap::new(),
            instances: BTreeMap::new(),
            instanceStores: BTreeMap::new(),
            canonicalImplStore: CanonicalInstanceStore::new(),
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

    pub fn getImplicit(&self, qn: &QualifiedName) -> Option<Implicit> {
        self.implicits.get(qn).cloned()
    }

    pub fn getTrait(&self, qn: &QualifiedName) -> Option<Trait> {
        self.traits.get(qn).cloned()
    }

    pub fn getInstance(&self, qn: &QualifiedName) -> Option<Instance> {
        self.instances.get(qn).cloned()
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

    pub fn isTrait(&self, qn: &QualifiedName) -> bool {
        self.traits.contains_key(qn)
    }

    pub fn isInstance(&self, qn: &QualifiedName) -> bool {
        self.instances.contains_key(qn)
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
