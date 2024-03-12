use crate::siko::{
    qualifiedname::QualifiedName,
    syntax::Module::{Module, ModuleItem},
    util::error,
};

use std::{collections::BTreeMap, fmt::Display};

use super::ModuleResolver::ModuleResolver;
use super::TypeResolver::TypeResolver;
use crate::siko::ir::Data::Class as IrClass;
use crate::siko::ir::Data::Enum as IrEnum;
use crate::siko::ir::Data::Field as IrField;
use crate::siko::ir::Data::Variant as IrVariant;

#[derive(Debug)]
pub struct Names {
    pub names: BTreeMap<String, Vec<QualifiedName>>,
}

impl Names {
    pub fn new() -> Names {
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
    resolvers: BTreeMap<String, ModuleResolver>,
    classes: BTreeMap<QualifiedName, IrClass>,
    enums: BTreeMap<QualifiedName, IrEnum>,
}

impl Resolver {
    pub fn new() -> Resolver {
        Resolver {
            modules: BTreeMap::new(),
            resolvers: BTreeMap::new(),
            classes: BTreeMap::new(),
            enums: BTreeMap::new(),
        }
    }

    pub fn addModule(&mut self, m: Module) {
        self.modules.insert(m.name.toString(), m);
    }

    pub fn process(&mut self) {
        self.collectLocalNames();
        self.processImports();
        self.processClasses();
    }

    fn processClasses(&mut self) {
        for (_, m) in &self.modules {
            let moduleResolver = self.resolvers.get(&m.name.name).unwrap();
            for item in &m.items {
                match item {
                    ModuleItem::Class(c) => {
                        let mut typeResolver = TypeResolver::new(moduleResolver);
                        if let Some(decl) = &c.typeParams {
                            typeResolver.processTypeParameterDeclaration(&decl);
                        }
                        let mut irClass = IrClass::new(moduleResolver.resolverName(&c.name));
                        for field in &c.fields {
                            let ty = typeResolver.resolveType(&field.ty);
                            irClass.fields.push(IrField {
                                name: field.name.toString(),
                                ty,
                            })
                        }
                        println!("Class {:?}", irClass);
                        self.classes.insert(irClass.name.clone(), irClass);
                    }
                    ModuleItem::Enum(e) => {
                        let mut typeResolver = TypeResolver::new(moduleResolver);
                        if let Some(decl) = &e.typeParams {
                            typeResolver.processTypeParameterDeclaration(&decl);
                        }
                        let mut irEnum = IrEnum::new(moduleResolver.resolverName(&e.name));
                        for variant in &e.variants {
                            let mut items = Vec::new();
                            for item in &variant.items {
                                let ty = typeResolver.resolveType(item);
                                items.push(ty);
                            }

                            irEnum.variants.push(IrVariant {
                                name: irEnum.name.add(variant.name.toString()),
                                items: items,
                            });
                        }
                        //println!("Enum {:?}", irEnum);
                        self.enums.insert(irEnum.name.clone(), irEnum);
                    }
                    ModuleItem::Function(f) => {
                        let mut typeResolver = TypeResolver::new(moduleResolver);
                        if let Some(decl) = &f.typeParams {
                            typeResolver.processTypeParameterDeclaration(&decl);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn processImports(&mut self) {
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
            let moduleResolver = self.resolvers.get_mut(&m.name.toString()).unwrap();
            moduleResolver.importedNames = importedNames;
        }
    }

    fn collectLocalNames(&mut self) {
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
            let moduleResolver = ModuleResolver {
                localNames: localNames,
                importedNames: Names::new(),
            };
            self.resolvers.insert(m.name.toString(), moduleResolver);
        }
    }
}
