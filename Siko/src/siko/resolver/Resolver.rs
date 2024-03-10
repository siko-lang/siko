use crate::siko::{
    qualifiedname::QualifiedName,
    syntax::{
        Identifier::Identifier,
        Module::{Module, ModuleItem},
    },
};

use std::{collections::BTreeMap, fmt::Display};

struct LocalNames {
    localNames: BTreeMap<String, Vec<QualifiedName>>,
}

impl LocalNames {
    fn new() -> LocalNames {
        LocalNames {
            localNames: BTreeMap::new(),
        }
    }

    fn add<T: Display>(&mut self, name: &T, qualifiedname: &QualifiedName) {
        //println!("Adding local name {} => {}", name, qualifiedname);
        let names = self
            .localNames
            .entry(format!("{}", name))
            .or_insert_with(|| Vec::new());
        names.push(qualifiedname.clone());
    }
}

pub struct Resolver {
    modules: Vec<Module>,
    localNames: BTreeMap<Identifier, LocalNames>,
}

impl Resolver {
    pub fn new() -> Resolver {
        Resolver {
            modules: Vec::new(),
            localNames: BTreeMap::new(),
        }
    }

    pub fn addModule(&mut self, m: Module) {
        self.modules.push(m);
    }

    pub fn process(&mut self) {
        for m in &self.modules {
            println!("Processing module {}", m.name);
            let mut localNames = LocalNames::new();
            let moduleName = QualifiedName::Module(m.name.clone());
            for item in &m.items {
                match item {
                    ModuleItem::Class(c) => {
                        let className = moduleName.add(&c.name);
                        localNames.add(&c.name, &className);
                        localNames.add(&className, &className);
                    }
                    ModuleItem::Enum(e) => {
                        let enumName = moduleName.add(&e.name);
                        localNames.add(&e.name, &enumName);
                        localNames.add(&enumName, &enumName);
                        for v in &e.variants {
                            let variantName = enumName.add(&v.name);
                            localNames.add(&v.name, &variantName);
                            localNames.add(&format!("{}.{}", e.name, v.name), &variantName);
                            localNames.add(&variantName, &variantName);
                        }
                    }
                    ModuleItem::Function(f) => {
                        let functionName = moduleName.add(&f.name);
                        localNames.add(&f.name, &functionName);
                        localNames.add(&functionName, &functionName);
                    }
                    ModuleItem::Import(_) => {}
                    ModuleItem::Trait(t) => {
                        let traitName = moduleName.add(&t.name);
                        localNames.add(&t.name, &traitName);
                        localNames.add(&traitName, &traitName);
                    }
                    ModuleItem::Instance(_) => {}
                }
            }
            self.localNames.insert(m.name.clone(), localNames);
        }
    }
}
