use crate::siko::{
    ir::Data::MethodInfo,
    qualifiedname::QualifiedName,
    resolver::FunctionResolver::FunctionResolver,
    syntax::Module::{Module, ModuleItem},
    util::error,
};

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use super::ModuleResolver::ModuleResolver;
use super::TypeResolver::TypeResolver;
use crate::siko::ir::Data::Class as IrClass;
use crate::siko::ir::Data::Enum as IrEnum;
use crate::siko::ir::Data::Field as IrField;
use crate::siko::ir::Data::Variant as IrVariant;
use crate::siko::ir::Function::Function as IrFunction;

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
    functions: BTreeMap<QualifiedName, IrFunction>,
    emptyVariants: BTreeSet<QualifiedName>,
}

impl Resolver {
    pub fn new() -> Resolver {
        Resolver {
            modules: BTreeMap::new(),
            resolvers: BTreeMap::new(),
            classes: BTreeMap::new(),
            enums: BTreeMap::new(),
            functions: BTreeMap::new(),
            emptyVariants: BTreeSet::new(),
        }
    }

    pub fn addModule(&mut self, m: Module) {
        self.modules.insert(m.name.toString(), m);
    }

    pub fn process(&mut self) {
        self.collectLocalNames();
        self.processImports();
        self.processDataTypes();
        self.processFunctions();
    }

    fn processDataTypes(&mut self) {
        for (_, m) in &self.modules {
            let moduleResolver = self.resolvers.get(&m.name.name).unwrap();
            for item in &m.items {
                match item {
                    ModuleItem::Class(c) => {
                        let typeResolver = TypeResolver::new(moduleResolver, &c.typeParams);
                        let mut irClass = IrClass::new(moduleResolver.resolverName(&c.name));
                        for field in &c.fields {
                            let ty = typeResolver.resolveType(&field.ty);
                            irClass.fields.push(IrField {
                                name: field.name.toString(),
                                ty,
                            })
                        }
                        for method in &c.methods {
                            irClass.methods.push(MethodInfo {
                                name: method.name.toString(),
                                fullName: moduleResolver.resolverName(&m.name),
                            })
                        }
                        //println!("Class {:?}", irClass);
                        self.classes.insert(irClass.name.clone(), irClass);
                    }
                    ModuleItem::Enum(e) => {
                        let typeResolver = TypeResolver::new(moduleResolver, &e.typeParams);
                        let mut irEnum = IrEnum::new(moduleResolver.resolverName(&e.name));
                        for variant in &e.variants {
                            let mut items = Vec::new();
                            for item in &variant.items {
                                let ty = typeResolver.resolveType(item);
                                items.push(ty);
                            }

                            let variant = IrVariant {
                                name: irEnum.name.add(variant.name.toString()),
                                items: items,
                            };
                            if variant.items.is_empty() {
                                self.emptyVariants.insert(variant.name.clone());
                            }
                            irEnum.variants.push(variant);
                        }
                        for method in &e.methods {
                            irEnum.methods.push(MethodInfo {
                                name: method.name.toString(),
                                fullName: moduleResolver.resolverName(&m.name),
                            })
                        }
                        //println!("Enum {:?}", irEnum);
                        self.enums.insert(irEnum.name.clone(), irEnum);
                    }
                    _ => {}
                }
            }
        }
    }

    fn processFunctions(&mut self) {
        for (_, m) in &self.modules {
            let moduleResolver = self.resolvers.get(&m.name.name).unwrap();
            for item in &m.items {
                match item {
                    ModuleItem::Class(c) => {
                        for method in &c.methods {
                            let functionResolver =
                                FunctionResolver::new(moduleResolver, c.typeParams.as_ref());
                            let irFunction = functionResolver.resolve(method, &self.emptyVariants);
                            self.functions.insert(irFunction.name.clone(), irFunction);
                        }
                    }
                    ModuleItem::Enum(e) => {
                        for method in &e.methods {
                            let functionResolver =
                                FunctionResolver::new(moduleResolver, e.typeParams.as_ref());
                            let irFunction = functionResolver.resolve(method, &self.emptyVariants);
                            self.functions.insert(irFunction.name.clone(), irFunction);
                        }
                    }
                    ModuleItem::Function(f) => {
                        let functionResolver = FunctionResolver::new(moduleResolver, None);
                        let irFunction = functionResolver.resolve(f, &self.emptyVariants);
                        self.functions.insert(irFunction.name.clone(), irFunction);
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
                        match self.modules.get(&moduleName) {
                            Some(sourceModule) => {
                                if let Some(alias) = &i.alias {
                                    let moduleName = QualifiedName::Module(alias.toString());
                                    for item in &sourceModule.items {
                                        match item {
                                            ModuleItem::Class(c) => {
                                                let className = moduleName.add(c.name.toString());
                                                importedNames.add(&className, &className);
                                                for m in &c.methods {
                                                    let methodName =
                                                        className.add(m.name.toString());
                                                    importedNames.add(&methodName, &methodName);
                                                }
                                            }
                                            ModuleItem::Enum(e) => {
                                                let enumName = moduleName.add(e.name.toString());
                                                importedNames.add(&enumName, &enumName);
                                                for v in &e.variants {
                                                    let variantName =
                                                        enumName.add(v.name.toString());
                                                    importedNames.add(&variantName, &variantName);
                                                }
                                                for m in &e.methods {
                                                    let methodName =
                                                        enumName.add(m.name.toString());
                                                    importedNames.add(&methodName, &methodName);
                                                }
                                            }
                                            ModuleItem::Function(f) => {
                                                let functionName =
                                                    moduleName.add(f.name.toString());
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
                                                for m in &c.methods {
                                                    let methodName =
                                                        className.add(m.name.toString());
                                                    importedNames.add(&m.name, &methodName);
                                                    importedNames.add(
                                                        &format!("{}.{}", c.name, m.name),
                                                        &methodName,
                                                    );
                                                    importedNames.add(&methodName, &methodName);
                                                }
                                            }
                                            ModuleItem::Enum(e) => {
                                                let enumName = moduleName.add(e.name.toString());
                                                importedNames.add(&e.name, &enumName);
                                                importedNames.add(&enumName, &enumName);
                                                for v in &e.variants {
                                                    let variantName =
                                                        enumName.add(v.name.toString());
                                                    importedNames.add(&v.name, &variantName);
                                                    importedNames.add(
                                                        &format!("{}.{}", e.name, v.name),
                                                        &variantName,
                                                    );
                                                    importedNames.add(&variantName, &variantName);
                                                }
                                                for m in &e.methods {
                                                    let methodName =
                                                        enumName.add(m.name.toString());
                                                    importedNames.add(&m.name, &methodName);
                                                    importedNames.add(
                                                        &format!("{}.{}", e.name, m.name),
                                                        &methodName,
                                                    );
                                                    importedNames.add(&methodName, &methodName);
                                                }
                                            }
                                            ModuleItem::Function(f) => {
                                                let functionName =
                                                    moduleName.add(f.name.toString());
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
                            None => {
                                if !i.implicitImport {
                                    error(format!("Imported module not found {}", moduleName));
                                }
                            }
                        };
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
                        for m in &c.methods {
                            let methodName = className.add(m.name.toString());
                            localNames.add(&m.name, &methodName);
                            localNames.add(&format!("{}.{}", c.name, m.name), &methodName);
                            localNames.add(&methodName, &methodName);
                        }
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
                        for m in &e.methods {
                            let methodName = enumName.add(m.name.toString());
                            localNames.add(&m.name, &methodName);
                            localNames.add(&format!("{}.{}", e.name, m.name), &methodName);
                            localNames.add(&methodName, &methodName);
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
