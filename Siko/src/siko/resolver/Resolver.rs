use crate::siko::{
    qualifiedname::QualifiedName,
    syntax::Module::{Module, ModuleItem},
    util::error,
};

use std::{collections::BTreeMap, fmt::Display};

#[derive(Debug)]
struct Names {
    names: BTreeMap<String, Vec<QualifiedName>>,
}

impl Names {
    fn new() -> Names {
        Names {
            names: BTreeMap::new(),
        }
    }

    fn add<T: Display>(&mut self, name: &T, qualifiedname: &QualifiedName) {
        //println!("Adding local name {} => {}", name, qualifiedname);
        let names = self
            .names
            .entry(format!("{}", name))
            .or_insert_with(|| Vec::new());
        names.push(qualifiedname.clone());
    }
}

pub struct Resolver {
    modules: BTreeMap<String, Module>,
    localNames: BTreeMap<String, Names>,
    importedNames: BTreeMap<String, Names>,
}

impl Resolver {
    pub fn new() -> Resolver {
        Resolver {
            modules: BTreeMap::new(),
            localNames: BTreeMap::new(),
            importedNames: BTreeMap::new(),
        }
    }

    pub fn addModule(&mut self, m: Module) {
        self.modules.insert(m.name.toString(), m);
    }

    pub fn process(&mut self) {
        self.collectLocalNames();
        self.processImports();
    }

    pub fn processImports(&mut self) {
        self.collectLocalNames();
        for (_, m) in &self.modules {
            //println!("Processing module {}", name);
            let mut importedNames = Names::new();
            for item in &m.items {
                match item {
                    ModuleItem::Import(i) => {
                        let moduleName = i.moduleName.toString();
                        let sourceModule = match self.modules.get(&moduleName) {
                            Some(m) => m,
                            None => error(format!("Imported module not found {}", moduleName)),
                        };
                        if let Some(alias) = &i.alias {
                            let moduleName = QualifiedName::Module(alias.toString());
                            for item in &sourceModule.items {
                                match item {
                                    ModuleItem::Class(c) => {
                                        let className = moduleName.add(c.name.toString());
                                        importedNames.add(&className, &className);
                                    }
                                    ModuleItem::Enum(e) => {
                                        let enumName = moduleName.add(e.name.toString());
                                        importedNames.add(&enumName, &enumName);
                                        for v in &e.variants {
                                            let variantName = enumName.add(v.name.toString());
                                            importedNames.add(&variantName, &variantName);
                                        }
                                    }
                                    ModuleItem::Function(f) => {
                                        let functionName = moduleName.add(f.name.toString());
                                        importedNames.add(&functionName, &functionName);
                                    }
                                    ModuleItem::Import(_) => {}
                                    ModuleItem::Trait(t) => {
                                        let traitName = moduleName.add(t.name.toString());
                                        importedNames.add(&traitName, &traitName);
                                    }
                                    ModuleItem::Instance(_) => {}
                                }
                            }
                        } else {
                            let moduleName = QualifiedName::Module(moduleName);
                            for item in &sourceModule.items {
                                match item {
                                    ModuleItem::Class(c) => {
                                        let className = moduleName.add(c.name.toString());
                                        importedNames.add(&c.name, &className);
                                        importedNames.add(&className, &className);
                                    }
                                    ModuleItem::Enum(e) => {
                                        let enumName = moduleName.add(e.name.toString());
                                        importedNames.add(&e.name, &enumName);
                                        importedNames.add(&enumName, &enumName);
                                        for v in &e.variants {
                                            let variantName = enumName.add(v.name.toString());
                                            importedNames.add(&v.name, &variantName);
                                            importedNames.add(
                                                &format!("{}.{}", e.name, v.name),
                                                &variantName,
                                            );
                                            importedNames.add(&variantName, &variantName);
                                        }
                                    }
                                    ModuleItem::Function(f) => {
                                        let functionName = moduleName.add(f.name.toString());
                                        importedNames.add(&f.name, &functionName);
                                        importedNames.add(&functionName, &functionName);
                                    }
                                    ModuleItem::Import(_) => {}
                                    ModuleItem::Trait(t) => {
                                        let traitName = moduleName.add(t.name.toString());
                                        importedNames.add(&t.name, &traitName);
                                        importedNames.add(&traitName, &traitName);
                                    }
                                    ModuleItem::Instance(_) => {}
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            self.importedNames.insert(m.name.toString(), importedNames);
        }
    }

    pub fn collectLocalNames(&mut self) {
        for (_, m) in &self.modules {
            //println!("Processing module {}", name);
            let mut localNames = Names::new();
            let moduleName = QualifiedName::Module(m.name.toString());
            for item in &m.items {
                match item {
                    ModuleItem::Class(c) => {
                        let className = moduleName.add(c.name.toString());
                        localNames.add(&c.name, &className);
                        localNames.add(&className, &className);
                    }
                    ModuleItem::Enum(e) => {
                        let enumName = moduleName.add(e.name.toString());
                        localNames.add(&e.name, &enumName);
                        localNames.add(&enumName, &enumName);
                        for v in &e.variants {
                            let variantName = enumName.add(v.name.toString());
                            localNames.add(&v.name, &variantName);
                            localNames.add(&format!("{}.{}", e.name, v.name), &variantName);
                            localNames.add(&variantName, &variantName);
                        }
                    }
                    ModuleItem::Function(f) => {
                        let functionName = moduleName.add(f.name.toString());
                        localNames.add(&f.name, &functionName);
                        localNames.add(&functionName, &functionName);
                    }
                    ModuleItem::Import(_) => {}
                    ModuleItem::Trait(t) => {
                        let traitName = moduleName.add(t.name.toString());
                        localNames.add(&t.name, &traitName);
                        localNames.add(&traitName, &traitName);
                    }
                    ModuleItem::Instance(_) => {}
                }
            }
            self.localNames.insert(m.name.toString(), localNames);
        }
    }
}
