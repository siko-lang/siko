use crate::siko::{
    ir::{
        ConstraintContext::ConstraintContext, Data::MethodInfo as DataMethodInfo,
        Function::Parameter, Trait::MethodInfo as TraitMethodInfo,
        TraitMethodSelector::TraitMethodSelector, Type::TypeVar,
    },
    qualifiedname::QualifiedName,
    resolver::FunctionResolver::FunctionResolver,
    syntax::{
        Module::{Module, ModuleItem},
        Type::TypeParameterDeclaration,
    },
    util::error,
};

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use super::{Error::ResolverError, ModuleResolver::ModuleResolver};
use super::{FunctionResolver::createSelfType, TypeResolver::TypeResolver};
use crate::siko::ir::Data::Class as IrClass;
use crate::siko::ir::Data::Enum as IrEnum;
use crate::siko::ir::Data::Field as IrField;
use crate::siko::ir::Data::Variant as IrVariant;
use crate::siko::ir::Function::Function as IrFunction;
use crate::siko::ir::Trait::Instance as IrInstance;
use crate::siko::ir::Trait::Trait as IrTrait;
use crate::siko::ir::Type::Type as IrType;

fn createConstraintContext(decl: &Option<TypeParameterDeclaration>) -> ConstraintContext {
    addTypeParams(ConstraintContext::new(), decl)
}

fn addTypeParams(
    mut context: ConstraintContext,
    decl: &Option<TypeParameterDeclaration>,
) -> ConstraintContext {
    if let Some(decl) = decl {
        for param in &decl.params {
            context.add(param.name.name.clone());
        }
    }
    context
}

#[derive(Debug, PartialEq, Eq)]
pub struct Names {
    pub names: BTreeMap<String, BTreeSet<QualifiedName>>,
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
            .or_insert_with(|| BTreeSet::new());
        names.insert(qualifiedname.clone());
    }
}

pub struct Resolver {
    modules: BTreeMap<String, Module>,
    resolvers: BTreeMap<String, ModuleResolver>,
    classes: BTreeMap<QualifiedName, IrClass>,
    enums: BTreeMap<QualifiedName, IrEnum>,
    functions: BTreeMap<QualifiedName, IrFunction>,
    emptyVariants: BTreeSet<QualifiedName>,
    variants: BTreeMap<QualifiedName, QualifiedName>,
    traits: BTreeMap<QualifiedName, IrTrait>,
    instances: Vec<IrInstance>,
    traitMethodSelectors: BTreeMap<QualifiedName, TraitMethodSelector>,
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
            variants: BTreeMap::new(),
            traits: BTreeMap::new(),
            instances: Vec::new(),
            traitMethodSelectors: BTreeMap::new(),
        }
    }

    pub fn addModule(&mut self, m: Module) {
        self.modules.insert(m.name.toString(), m);
    }

    pub fn process(&mut self) {
        self.collectLocalNames();
        self.processImports();
        self.processDataTypes();
        self.processTraits();
        self.processFunctions();
        //self.dump();
    }

    pub fn ir(
        self,
    ) -> (
        BTreeMap<QualifiedName, IrFunction>,
        BTreeMap<QualifiedName, IrClass>,
        BTreeMap<QualifiedName, IrEnum>,
        BTreeMap<QualifiedName, TraitMethodSelector>,
    ) {
        (
            self.functions,
            self.classes,
            self.enums,
            self.traitMethodSelectors,
        )
    }

    fn dump(&self) {
        for (name, f) in &self.functions {
            println!("Function {}", name);
            f.dump();
        }
    }

    fn processDataTypes(&mut self) {
        for (_, m) in &self.modules {
            let moduleResolver = self.resolvers.get(&m.name.name).unwrap();
            for item in &m.items {
                match item {
                    ModuleItem::Class(c) => {
                        let constraintContext = createConstraintContext(&c.typeParams);
                        let typeResolver = TypeResolver::new(moduleResolver, &constraintContext);
                        let irType = typeResolver.createDataType(&c.name, &c.typeParams);
                        let mut irClass =
                            IrClass::new(moduleResolver.resolverName(&c.name), irType.clone());
                        let mut ctorParams = Vec::new();
                        for field in &c.fields {
                            let ty = typeResolver.resolveType(&field.ty);
                            ctorParams.push(Parameter::Named(
                                field.name.toString(),
                                ty.clone(),
                                false,
                            ));
                            irClass.fields.push(IrField {
                                name: field.name.toString(),
                                ty,
                            })
                        }
                        let ctor = IrFunction::new(irClass.name.clone(), ctorParams, irType, None);
                        self.functions.insert(ctor.name.clone(), ctor);
                        for method in &c.methods {
                            irClass.methods.push(DataMethodInfo {
                                name: method.name.toString(),
                                fullName: moduleResolver.resolverName(&method.name),
                            })
                        }
                        //println!("Class {:?}", irClass);
                        self.classes.insert(irClass.name.clone(), irClass);
                    }
                    ModuleItem::Enum(e) => {
                        let constraintContext = createConstraintContext(&e.typeParams);
                        let typeResolver = TypeResolver::new(moduleResolver, &constraintContext);
                        let irType = typeResolver.createDataType(&e.name, &e.typeParams);
                        let mut irEnum =
                            IrEnum::new(moduleResolver.resolverName(&e.name), irType.clone());
                        for variant in &e.variants {
                            let mut items = Vec::new();
                            let mut ctorParams = Vec::new();
                            for (index, item) in variant.items.iter().enumerate() {
                                let ty = typeResolver.resolveType(item);
                                ctorParams.push(Parameter::Named(
                                    format!("item{}", index),
                                    ty.clone(),
                                    false,
                                ));
                                items.push(ty);
                            }

                            let variant = IrVariant {
                                name: irEnum.name.add(variant.name.toString()),
                                items: items,
                            };
                            if variant.items.is_empty() {
                                self.emptyVariants.insert(variant.name.clone());
                            }
                            let ctor = IrFunction::new(
                                variant.name.clone(),
                                ctorParams,
                                irType.clone(),
                                None,
                            );
                            self.functions.insert(ctor.name.clone(), ctor);
                            self.variants
                                .insert(variant.name.clone(), irEnum.name.clone());
                            irEnum.variants.push(variant);
                        }
                        for method in &e.methods {
                            irEnum.methods.push(DataMethodInfo {
                                name: method.name.toString(),
                                fullName: moduleResolver.resolverName(&method.name),
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

    fn processTraits(&mut self) {
        for (_, m) in &mut self.modules {
            let moduleResolver = self.resolvers.get(&m.name.name).unwrap();
            for item in &mut m.items {
                match item {
                    ModuleItem::Trait(t) => {
                        let mut irParams = Vec::new();
                        for param in &t.params {
                            let irParam = IrType::Var(TypeVar::Named(param.toString()));
                            irParams.push(irParam);
                        }
                        let mut irDeps = Vec::new();
                        for dep in &t.deps {
                            let irDep = IrType::Var(TypeVar::Named(dep.toString()));
                            irDeps.push(irDep);
                        }
                        let mut irTrait =
                            IrTrait::new(moduleResolver.resolverName(&t.name), irParams, irDeps);
                        for method in &t.methods {
                            irTrait.methods.push(TraitMethodInfo {
                                name: method.name.toString(),
                                fullName: QualifiedName::Item(
                                    Box::new(irTrait.name.clone()),
                                    method.name.toString(),
                                ),
                            })
                        }
                        //println!("Trait {:?}", irTrait);
                        self.traits.insert(irTrait.name.clone(), irTrait);
                    }
                    ModuleItem::Instance(i) => {
                        i.id = self.instances.len() as u64;
                        let constraintContext = createConstraintContext(&i.typeParams);
                        let typeResolver = TypeResolver::new(moduleResolver, &constraintContext);
                        let ty = typeResolver.resolveType(&i.ty);
                        let (name, args) = match ty {
                            IrType::Named(name, args) => (name, args),
                            ty => ResolverError::InvalidInstanceType(
                                format!("{}", ty),
                                i.location.clone(),
                            )
                            .report(),
                        };
                        let mut irInstance = IrInstance::new(i.id, name.clone(), args);
                        for method in &i.methods {
                            irInstance.methods.push(TraitMethodInfo {
                                name: method.name.toString(),
                                fullName: QualifiedName::Instance(
                                    Box::new(QualifiedName::Item(
                                        Box::new(name.clone()),
                                        method.name.toString(),
                                    )),
                                    irInstance.id,
                                ),
                            })
                        }
                        //println!("Instance {:?}", irInstance);
                        self.instances.push(irInstance);
                    }
                    _ => {}
                }
            }
        }
    }

    fn processFunctions(&mut self) {
        for (_, m) in &self.modules {
            let moduleResolver = self.resolvers.get(&m.name.name).unwrap();
            let mut traitMethodSelector = TraitMethodSelector::new();
            for item in &m.items {
                match item {
                    ModuleItem::Class(c) => {
                        let owner = createSelfType(&c.name, c.typeParams.as_ref(), moduleResolver);
                        let constraintContext = createConstraintContext(&c.typeParams);
                        for method in &c.methods {
                            let functionResolver = FunctionResolver::new(
                                moduleResolver,
                                constraintContext.clone(),
                                Some(owner.clone()),
                            );
                            let irFunction = functionResolver.resolve(
                                method,
                                &self.emptyVariants,
                                &self.variants,
                                &self.enums,
                                moduleResolver.resolverName(&method.name),
                            );
                            self.functions.insert(irFunction.name.clone(), irFunction);
                        }
                    }
                    ModuleItem::Enum(e) => {
                        let owner = createSelfType(&e.name, e.typeParams.as_ref(), moduleResolver);
                        let constraintContext = createConstraintContext(&e.typeParams);
                        for method in &e.methods {
                            let functionResolver = FunctionResolver::new(
                                moduleResolver,
                                constraintContext.clone(),
                                Some(owner.clone()),
                            );
                            let irFunction = functionResolver.resolve(
                                method,
                                &self.emptyVariants,
                                &self.variants,
                                &self.enums,
                                moduleResolver.resolverName(&method.name),
                            );
                            self.functions.insert(irFunction.name.clone(), irFunction);
                        }
                    }
                    ModuleItem::Trait(t) => {
                        let name = moduleResolver.resolverName(&t.name);
                        let irTrait = self.traits.get(&name).unwrap();
                        let owner = irTrait.params.first().expect("first trait param not found");
                        let mut constraintContext = createConstraintContext(&t.typeParams);
                        for param in &irTrait.params {
                            match &param {
                                IrType::Var(TypeVar::Named(n)) => constraintContext.add(n.clone()),
                                _ => panic!("Trait param is not type var!"),
                            }
                        }
                        for dep in &irTrait.deps {
                            match &dep {
                                IrType::Var(TypeVar::Named(n)) => constraintContext.add(n.clone()),
                                _ => panic!("Trait dep is not type var!"),
                            }
                        }
                        for method in &t.methods {
                            let constraintContext =
                                addTypeParams(constraintContext.clone(), &method.typeParams);
                            let functionResolver = FunctionResolver::new(
                                moduleResolver,
                                constraintContext,
                                Some(owner.clone()),
                            );
                            let irFunction = functionResolver.resolve(
                                method,
                                &self.emptyVariants,
                                &self.variants,
                                &self.enums,
                                QualifiedName::Item(Box::new(name.clone()), method.name.toString()),
                            );
                            traitMethodSelector.add(method.name.clone(), irFunction.name.clone());
                            self.functions.insert(irFunction.name.clone(), irFunction);
                        }
                    }
                    ModuleItem::Instance(i) => {
                        let irInstance = &self.instances[i.id as usize];
                        let constraintContext = createConstraintContext(&i.typeParams);
                        for method in &i.methods {
                            let constraintContext =
                                addTypeParams(constraintContext.clone(), &method.typeParams);
                            let functionResolver = FunctionResolver::new(
                                moduleResolver,
                                constraintContext,
                                Some(irInstance.types[0].clone()),
                            );
                            let irFunction = functionResolver.resolve(
                                method,
                                &self.emptyVariants,
                                &self.variants,
                                &self.enums,
                                QualifiedName::Instance(
                                    Box::new(QualifiedName::Item(
                                        Box::new(irInstance.traitName.clone()),
                                        method.name.toString(),
                                    )),
                                    irInstance.id,
                                ),
                            );
                            self.functions.insert(irFunction.name.clone(), irFunction);
                        }
                    }
                    ModuleItem::Function(f) => {
                        let constraintContext = createConstraintContext(&f.typeParams);
                        let functionResolver =
                            FunctionResolver::new(moduleResolver, constraintContext, None);
                        let irFunction = functionResolver.resolve(
                            f,
                            &self.emptyVariants,
                            &self.variants,
                            &self.enums,
                            moduleResolver.resolverName(&f.name),
                        );
                        self.functions.insert(irFunction.name.clone(), irFunction);
                    }
                    _ => {}
                }
            }
            self.traitMethodSelectors.insert(
                QualifiedName::Module(m.name.toString()),
                traitMethodSelector,
            );
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
                                                for m in &t.methods {
                                                    let methodName =
                                                        traitName.add(m.name.toString());
                                                    importedNames.add(&m.name, &methodName);
                                                    importedNames.add(
                                                        &format!("{}.{}", t.name, m.name),
                                                        &methodName,
                                                    );
                                                    importedNames.add(&methodName, &methodName);
                                                }
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
            let moduleResolver = ModuleResolver {
                name: m.name.toString(),
                localNames: Resolver::buildLocalNames(m),
                importedNames: Names::new(),
            };
            self.resolvers.insert(m.name.toString(), moduleResolver);
        }
    }

    pub fn buildLocalNames(m: &Module) -> Names {
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
                    for m in &t.methods {
                        let methodName = traitName.add(m.name.toString());
                        localNames.add(&m.name, &methodName);
                        localNames.add(&format!("{}.{}", t.name, m.name), &methodName);
                        localNames.add(&methodName, &methodName);
                    }
                }
                ModuleItem::Instance(_) => {}
            }
        }
        localNames
    }
}
