use crate::siko::{
    hir::{
        ConstraintContext::{Constraint as IrConstraint, ConstraintContext},
        Data::MethodInfo as DataMethodInfo,
        Function::{FunctionKind, Parameter},
        Program::Program,
        Trait::{AssociatedType, MemberInfo},
        TraitMethodSelector::{TraitMethodSelection, TraitMethodSelector},
        Type::TypeVar,
    },
    location::Report::ReportContext,
    qualifiedname::QualifiedName,
    resolver::FunctionResolver::FunctionResolver,
    syntax::{
        Function::Parameter as SynParam,
        Module::{Import, Module, ModuleItem},
        Type::{Constraint, ConstraintArgument, TypeParameterDeclaration},
    },
    util::error,
};

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use super::{Error::ResolverError, ModuleResolver::ModuleResolver};
use super::{FunctionResolver::createSelfType, TypeResolver::TypeResolver};
use crate::siko::hir::Data::Class as IrClass;
use crate::siko::hir::Data::Enum as IrEnum;
use crate::siko::hir::Data::Field as IrField;
use crate::siko::hir::Data::Variant as IrVariant;
use crate::siko::hir::Function::Function as IrFunction;
use crate::siko::hir::Trait::Instance as IrInstance;
use crate::siko::hir::Trait::Trait as IrTrait;
use crate::siko::hir::Type::Type as IrType;

pub fn createConstraintContext(
    decl: &Option<TypeParameterDeclaration>,
    typeResolver: &TypeResolver,
    program: &Program,
    ctx: &ReportContext,
) -> ConstraintContext {
    addTypeParams(ConstraintContext::new(), decl, typeResolver, program, ctx)
}

fn addConstraint(
    mut context: ConstraintContext,
    constraint: &Constraint,
    typeResolver: &TypeResolver,
    program: &Program,
    ctx: &ReportContext,
) -> ConstraintContext {
    let traitDef = typeResolver.moduleResolver.lookupTrait(&constraint.traitName, program);
    let mut args = Vec::new();
    let mut associatedTypes = Vec::new();
    for arg in &constraint.args {
        match arg {
            ConstraintArgument::Type(ty) => {
                let irTy = typeResolver.resolveType(ty);
                args.push(irTy);
            }
            ConstraintArgument::AssociatedType(name, ty) => {
                if !traitDef.associatedTypes.contains(&name.toString()) {
                    ResolverError::AssociatedTypeNotFound(
                        name.toString(),
                        traitDef.name.toString(),
                        constraint.traitName.location.clone(),
                    )
                    .report(ctx);
                }
                let irTy = typeResolver.resolveType(ty);
                let associatedType = AssociatedType {
                    name: name.toString(),
                    ty: irTy,
                };
                associatedTypes.push(associatedType);
            }
        }
    }
    let irConstraint = IrConstraint {
        traitName: traitDef.name,
        args: args,
        associatedTypes: associatedTypes,
    };
    context.addConstraint(irConstraint);
    context
}

fn addTypeParams(
    mut context: ConstraintContext,
    decl: &Option<TypeParameterDeclaration>,
    typeResolver: &TypeResolver,
    program: &Program,
    ctx: &ReportContext,
) -> ConstraintContext {
    if let Some(decl) = decl {
        //println!("Processing {}", decl);
        for param in &decl.params {
            let irParam = IrType::Var(TypeVar::Named(param.name.clone()));
            context.addTypeParam(irParam);
        }
        for constraint in &decl.constraints {
            context = addConstraint(context, constraint, typeResolver, program, ctx);
            //println!("Processing constraint {}", constraint);
        }
    }
    context
}

fn getTypeParams(decl: &Option<TypeParameterDeclaration>) -> Vec<IrType> {
    let mut params = Vec::new();
    if let Some(decl) = decl {
        for param in &decl.params {
            params.push(IrType::Var(TypeVar::Named(param.name.clone())));
        }
    }
    params
}

#[derive(Debug, PartialEq, Eq)]
pub struct Names {
    pub names: BTreeMap<String, BTreeSet<QualifiedName>>,
}

impl Names {
    pub fn new() -> Names {
        Names { names: BTreeMap::new() }
    }

    fn add<T: Display>(&mut self, name: &T, qualifiedname: &QualifiedName) {
        //println!("Adding local name {} => {}", name, qualifiedname);
        let names = self.names.entry(format!("{}", name)).or_insert_with(|| BTreeSet::new());
        names.insert(qualifiedname.clone());
    }
}

pub struct Resolver<'a> {
    ctx: &'a ReportContext,
    modules: BTreeMap<String, Module>,
    resolvers: BTreeMap<String, ModuleResolver<'a>>,
    program: Program,
    emptyVariants: BTreeSet<QualifiedName>,
    variants: BTreeMap<QualifiedName, QualifiedName>,
    instances: Vec<IrInstance>,
}

impl<'a> Resolver<'a> {
    pub fn new(ctx: &'a ReportContext) -> Resolver<'a> {
        Resolver {
            ctx: ctx,
            modules: BTreeMap::new(),
            resolvers: BTreeMap::new(),
            program: Program::new(),
            emptyVariants: BTreeSet::new(),
            variants: BTreeMap::new(),
            instances: Vec::new(),
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
        self.processInstances();
        self.processFunctions();

        let mut traitMethodSelectors = BTreeMap::new();

        for resolver in self.resolvers.values() {
            let name = QualifiedName::Module(resolver.name.clone());
            let mut selector = self
                .program
                .traitMethodSelectors
                .get(&name)
                .expect("trait method selector not found")
                .clone();
            for importedModule in &resolver.importedModules {
                let importedModuleName = QualifiedName::Module(importedModule.clone());
                if importedModuleName == name {
                    continue;
                }
                let moduleSelector = self
                    .program
                    .traitMethodSelectors
                    .get(&importedModuleName)
                    .expect("trait method selector not found");
                selector.merge(moduleSelector);
            }
            traitMethodSelectors.insert(name, selector);
        }
        self.program.traitMethodSelectors = traitMethodSelectors;
        //self.dump();
    }

    pub fn ir(self) -> Program {
        self.program
    }

    fn dump(&self) {
        for (name, f) in &self.program.functions {
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
                        let typeParams = getTypeParams(&c.typeParams);
                        let typeResolver = TypeResolver::new(moduleResolver, &typeParams);
                        let constraintContext =
                            createConstraintContext(&c.typeParams, &typeResolver, &self.program, &self.ctx);
                        let irType = typeResolver.createDataType(&c.name, &c.typeParams);
                        let className = QualifiedName::Module(moduleResolver.name.clone()).add(c.name.toString());
                        let mut irClass = IrClass::new(className, irType.clone());
                        let mut ctorParams = Vec::new();
                        for field in &c.fields {
                            let ty = typeResolver.resolveType(&field.ty);
                            ctorParams.push(Parameter::Named(field.name.toString(), ty.clone(), false));
                            irClass.fields.push(IrField {
                                name: field.name.toString(),
                                ty,
                            })
                        }
                        let ctor = IrFunction::new(
                            irClass.name.clone(),
                            ctorParams,
                            irType,
                            None,
                            constraintContext,
                            FunctionKind::ClassCtor,
                        );
                        self.program.functions.insert(ctor.name.clone(), ctor);
                        for method in &c.methods {
                            irClass.methods.push(DataMethodInfo {
                                name: method.name.toString(),
                                fullName: irClass.name.add(method.name.toString()),
                            })
                        }
                        //println!("Class {:?}", irClass);
                        self.program.classes.insert(irClass.name.clone(), irClass);
                    }
                    ModuleItem::Enum(e) => {
                        let typeParams = getTypeParams(&e.typeParams);
                        let typeResolver = TypeResolver::new(moduleResolver, &typeParams);
                        let constraintContext =
                            createConstraintContext(&e.typeParams, &typeResolver, &self.program, &self.ctx);
                        let irType = typeResolver.createDataType(&e.name, &e.typeParams);
                        let mut irEnum = IrEnum::new(moduleResolver.resolverName(&e.name), irType.clone());
                        for (index, variant) in e.variants.iter().enumerate() {
                            let mut items = Vec::new();
                            let mut ctorParams = Vec::new();
                            for (index, item) in variant.items.iter().enumerate() {
                                let ty = typeResolver.resolveType(item);
                                ctorParams.push(Parameter::Named(format!("f{}", index), ty.clone(), false));
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
                                constraintContext.clone(),
                                FunctionKind::VariantCtor(index as i64),
                            );
                            self.program.functions.insert(ctor.name.clone(), ctor);
                            self.variants.insert(variant.name.clone(), irEnum.name.clone());
                            irEnum.variants.push(variant);
                        }
                        for method in &e.methods {
                            irEnum.methods.push(DataMethodInfo {
                                name: method.name.toString(),
                                fullName: irEnum.name.add(method.name.toString()),
                            })
                        }
                        //println!("Enum {:?}", irEnum);
                        self.program.enums.insert(irEnum.name.clone(), irEnum);
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
                        let typeParams = getTypeParams(&t.typeParams);
                        let mut typeResolver = TypeResolver::new(moduleResolver, &typeParams);
                        let constraintContext =
                            createConstraintContext(&t.typeParams, &typeResolver, &self.program, &self.ctx);
                        let mut irParams = Vec::new();
                        for param in &t.params {
                            let irParam = IrType::Var(TypeVar::Named(param.toString()));
                            typeResolver.addTypeParams(irParam.clone());
                            irParams.push(irParam);
                        }
                        let mut associatedTypes = Vec::new();
                        let traitName = moduleResolver.resolverName(&t.name);
                        for associatedType in &t.associatedTypes {
                            associatedTypes.push(associatedType.name.name.clone());
                            let irParam = IrType::Var(TypeVar::Named(associatedType.name.toString()));
                            typeResolver.addTypeParams(irParam);
                        }
                        let selfType = irParams[0].clone();
                        let mut irTrait = IrTrait::new(traitName, irParams, associatedTypes, constraintContext);
                        for method in &t.methods {
                            let mut argTypes = Vec::new();
                            for param in &method.params {
                                let ty = match param {
                                    SynParam::Named(_, ty, _) => typeResolver.resolveType(ty),
                                    SynParam::SelfParam => selfType.clone(),
                                    SynParam::MutSelfParam => selfType.clone(),
                                    SynParam::RefSelfParam => selfType.clone(),
                                };
                                argTypes.push(ty);
                            }
                            let result = typeResolver
                                .resolveType(&method.result)
                                .changeSelfType(selfType.clone());
                            irTrait.members.push(MemberInfo {
                                name: method.name.toString(),
                                fullName: QualifiedName::Item(Box::new(irTrait.name.clone()), method.name.toString()),
                                default: method.body.is_some(),
                                result: IrType::Function(argTypes, Box::new(result)),
                            })
                        }
                        //println!("Trait {}", irTrait);
                        self.program.traits.insert(irTrait.name.clone(), irTrait);
                    }
                    _ => {}
                }
            }
        }
    }

    fn processInstances(&mut self) {
        for (_, m) in &mut self.modules {
            let moduleResolver = self.resolvers.get(&m.name.name).unwrap();
            for item in &mut m.items {
                match item {
                    ModuleItem::Instance(i) => {
                        i.id = self.instances.len() as u64;
                        let typeParams = getTypeParams(&i.typeParams);
                        let typeResolver = TypeResolver::new(moduleResolver, &typeParams);
                        let constraintContext =
                            createConstraintContext(&i.typeParams, &typeResolver, &self.program, &self.ctx);
                        let traitDef = moduleResolver.lookupTrait(&i.traitName, &self.program);
                        let args: Vec<_> = i.types.iter().map(|ty| typeResolver.resolveType(ty)).collect();
                        let mut associatedTypes = Vec::new();
                        for associatedType in &i.associatedTypes {
                            let mut found = false;
                            for traitAssociatedType in &traitDef.associatedTypes {
                                if traitAssociatedType == &associatedType.name.name {
                                    found = true;
                                    break;
                                }
                            }
                            if !found {
                                ResolverError::AssociatedTypeNotFound(
                                    associatedType.name.name.clone(),
                                    traitDef.name.toString(),
                                    associatedType.name.location.clone(),
                                )
                                .report(self.ctx);
                            }
                            let ty = typeResolver.resolveType(&associatedType.ty);
                            let irAssociatedType = AssociatedType {
                                name: associatedType.name.toString(),
                                ty: ty,
                            };
                            associatedTypes.push(irAssociatedType);
                        }
                        let selfType = args[0].clone();
                        let mut irInstance =
                            IrInstance::new(i.id, traitDef.name.clone(), args, associatedTypes, constraintContext);
                        for method in &i.methods {
                            let mut argTypes = Vec::new();
                            for param in &method.params {
                                let ty = match param {
                                    SynParam::Named(_, ty, _) => typeResolver.resolveType(ty),
                                    SynParam::SelfParam => selfType.clone(),
                                    SynParam::MutSelfParam => selfType.clone(),
                                    SynParam::RefSelfParam => selfType.clone(),
                                };
                                argTypes.push(ty);
                            }
                            let result = typeResolver
                                .resolveType(&method.result)
                                .changeSelfType(selfType.clone());
                            irInstance.members.push(MemberInfo {
                                name: method.name.toString(),
                                fullName: QualifiedName::Instance(
                                    Box::new(QualifiedName::Item(
                                        Box::new(traitDef.name.clone()),
                                        method.name.toString(),
                                    )),
                                    irInstance.id,
                                ),
                                default: false,
                                result: IrType::Function(argTypes, Box::new(result)),
                            })
                        }
                        //println!("Instance {}", irInstance);
                        self.program.instanceResolver.addInstance(irInstance.clone());
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
                        let typeParams = getTypeParams(&c.typeParams);
                        let typeResolver = TypeResolver::new(moduleResolver, &typeParams);
                        let constraintContext =
                            createConstraintContext(&c.typeParams, &typeResolver, &self.program, &self.ctx);
                        for method in &c.methods {
                            let functionResolver =
                                FunctionResolver::new(moduleResolver, constraintContext.clone(), Some(owner.clone()));
                            let irFunction = functionResolver.resolve(
                                self.ctx,
                                method,
                                &self.emptyVariants,
                                &self.program.classes,
                                &self.variants,
                                &self.program.enums,
                                owner.getName().unwrap().add(method.name.toString()),
                                &typeResolver,
                            );
                            self.program.functions.insert(irFunction.name.clone(), irFunction);
                        }
                    }
                    ModuleItem::Enum(e) => {
                        let owner = createSelfType(&e.name, e.typeParams.as_ref(), moduleResolver);
                        let typeParams = getTypeParams(&e.typeParams);
                        let typeResolver = TypeResolver::new(moduleResolver, &typeParams);
                        let constraintContext =
                            createConstraintContext(&e.typeParams, &typeResolver, &self.program, &self.ctx);
                        for method in &e.methods {
                            let functionResolver =
                                FunctionResolver::new(moduleResolver, constraintContext.clone(), Some(owner.clone()));
                            let irFunction = functionResolver.resolve(
                                self.ctx,
                                method,
                                &self.emptyVariants,
                                &self.program.classes,
                                &self.variants,
                                &self.program.enums,
                                owner.getName().unwrap().add(method.name.toString()),
                                &typeResolver,
                            );
                            self.program.functions.insert(irFunction.name.clone(), irFunction);
                        }
                    }
                    ModuleItem::Trait(t) => {
                        let name = moduleResolver.resolverName(&t.name);
                        let traitDef = self.program.getTrait(&name).unwrap();
                        let owner = traitDef.params.first().expect("first trait param not found");
                        let typeParams = getTypeParams(&t.typeParams);
                        let mut typeResolver = TypeResolver::new(moduleResolver, &typeParams);
                        let mut constraintContext =
                            createConstraintContext(&t.typeParams, &typeResolver, &self.program, &self.ctx);
                        for param in &traitDef.params {
                            typeResolver.addTypeParams(param.clone());
                            constraintContext.addTypeParam(param.clone());
                        }
                        for associatedType in &traitDef.associatedTypes {
                            typeResolver.addTypeParams(IrType::Var(TypeVar::Named(associatedType.clone())));
                        }
                        for method in &t.methods {
                            //println!("Processing trait method {}", method.name);
                            let mut constraintContext = addTypeParams(
                                constraintContext.clone(),
                                &method.typeParams,
                                &typeResolver,
                                &self.program,
                                &self.ctx,
                            );
                            let mut associatedTypes = Vec::new();
                            for assocTy in &t.associatedTypes {
                                let ty = IrType::Var(TypeVar::Named(assocTy.name.toString()));
                                associatedTypes.push(AssociatedType {
                                    name: assocTy.name.toString(),
                                    ty: ty.clone(),
                                });
                                constraintContext.addTypeParam(ty);
                                for c in &assocTy.constraints {
                                    constraintContext =
                                        addConstraint(constraintContext, c, &typeResolver, &self.program, &self.ctx);
                                }
                            }
                            constraintContext.addConstraint(IrConstraint {
                                traitName: traitDef.name.clone(),
                                args: traitDef.params.clone(),
                                associatedTypes: associatedTypes,
                            });
                            let functionResolver =
                                FunctionResolver::new(moduleResolver, constraintContext, Some(owner.clone()));
                            let mut irFunction = functionResolver.resolve(
                                self.ctx,
                                method,
                                &self.emptyVariants,
                                &self.program.classes,
                                &self.variants,
                                &self.program.enums,
                                QualifiedName::Item(Box::new(name.clone()), method.name.toString()),
                                &typeResolver,
                            );
                            if method.body.is_none() {
                                irFunction.kind = FunctionKind::TraitMemberDecl(name.clone());
                            } else {
                                irFunction.kind = FunctionKind::TraitMemberDefinition(name.clone());
                            }
                            let selection = TraitMethodSelection {
                                method: irFunction.name.clone(),
                                traitName: name.clone(),
                            };
                            traitMethodSelector.add(method.name.toString(), selection);
                            self.program.functions.insert(irFunction.name.clone(), irFunction);
                        }
                    }
                    ModuleItem::Instance(i) => {
                        let irInstance = &self.instances[i.id as usize];
                        let typeParams = getTypeParams(&i.typeParams);
                        let typeResolver = TypeResolver::new(moduleResolver, &typeParams);
                        let traitDef = self.program.getTrait(&irInstance.traitName).unwrap();
                        let mut allTraitMembers = BTreeSet::new();
                        let mut neededTraitMembers = BTreeSet::new();
                        for method in &traitDef.members {
                            allTraitMembers.insert(method.name.clone());
                            if !method.default {
                                neededTraitMembers.insert(method.name.clone());
                            }
                        }
                        let mut implementedMembers = BTreeSet::new();
                        for method in &i.methods {
                            implementedMembers.insert(method.name.clone());
                            if !allTraitMembers.contains(&method.name.name) {
                                ResolverError::InvalidInstanceMember(
                                    method.name.name.clone(),
                                    irInstance.traitName.toString(),
                                    method.name.location.clone(),
                                )
                                .report(self.ctx);
                            }
                            neededTraitMembers.remove(&method.name.name);
                            //println!("Processing instance method {}", method.name);
                            let constraintContext = addTypeParams(
                                irInstance.constraintContext.clone(),
                                &method.typeParams,
                                &typeResolver,
                                &self.program,
                                &self.ctx,
                            );
                            let functionResolver = FunctionResolver::new(
                                moduleResolver,
                                constraintContext,
                                Some(irInstance.types[0].clone()),
                            );
                            let irFunction = functionResolver.resolve(
                                self.ctx,
                                method,
                                &self.emptyVariants,
                                &self.program.classes,
                                &self.variants,
                                &self.program.enums,
                                QualifiedName::Instance(
                                    Box::new(QualifiedName::Item(
                                        Box::new(irInstance.traitName.clone()),
                                        method.name.toString(),
                                    )),
                                    irInstance.id,
                                ),
                                &typeResolver,
                            );
                            self.program.functions.insert(irFunction.name.clone(), irFunction);
                        }
                        if !neededTraitMembers.is_empty() {
                            ResolverError::MissingInstanceMembers(
                                neededTraitMembers.into_iter().collect(),
                                irInstance.traitName.toString(),
                                i.traitName.location.clone(),
                            )
                            .report(self.ctx);
                        }
                    }
                    ModuleItem::Function(f) => {
                        let typeParams = getTypeParams(&f.typeParams);
                        let typeResolver = TypeResolver::new(moduleResolver, &typeParams);
                        //println!("Processing function {}", f.name);
                        let constraintContext =
                            createConstraintContext(&f.typeParams, &typeResolver, &self.program, &self.ctx);
                        let functionResolver = FunctionResolver::new(moduleResolver, constraintContext, None);
                        let irFunction = functionResolver.resolve(
                            self.ctx,
                            f,
                            &self.emptyVariants,
                            &self.program.classes,
                            &self.variants,
                            &self.program.enums,
                            moduleResolver.resolverName(&f.name),
                            &typeResolver,
                        );
                        self.program.functions.insert(irFunction.name.clone(), irFunction);
                    }
                    _ => {}
                }
            }
            self.program
                .traitMethodSelectors
                .insert(QualifiedName::Module(m.name.toString()), traitMethodSelector);
        }
    }

    pub fn processSourceModule(sourceModule: &Module, importedNames: &mut Names, i: &Import) {
        if let Some(alias) = &i.alias {
            let moduleName = QualifiedName::Module(alias.toString());
            for item in &sourceModule.items {
                match item {
                    ModuleItem::Class(c) => {
                        let className = moduleName.add(c.name.toString());
                        importedNames.add(&className, &className);
                        for m in &c.methods {
                            let methodName = className.add(m.name.toString());
                            importedNames.add(&methodName, &methodName);
                        }
                    }
                    ModuleItem::Enum(e) => {
                        let enumName = moduleName.add(e.name.toString());
                        importedNames.add(&enumName, &enumName);
                        for v in &e.variants {
                            let variantName = enumName.add(v.name.toString());
                            importedNames.add(&variantName, &variantName);
                        }
                        for m in &e.methods {
                            let methodName = enumName.add(m.name.toString());
                            importedNames.add(&methodName, &methodName);
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
            let moduleName = QualifiedName::Module(sourceModule.name.toString());
            for item in &sourceModule.items {
                match item {
                    ModuleItem::Class(c) => {
                        let className = moduleName.add(c.name.toString());
                        importedNames.add(&c.name, &className);
                        importedNames.add(&className, &className);
                        for m in &c.methods {
                            let methodName = className.add(m.name.toString());
                            importedNames.add(&m.name, &methodName);
                            importedNames.add(&format!("{}.{}", c.name, m.name), &methodName);
                            importedNames.add(&methodName, &methodName);
                        }
                    }
                    ModuleItem::Enum(e) => {
                        let enumName = moduleName.add(e.name.toString());
                        importedNames.add(&e.name, &enumName);
                        importedNames.add(&enumName, &enumName);
                        for v in &e.variants {
                            let variantName = enumName.add(v.name.toString());
                            importedNames.add(&v.name, &variantName);
                            importedNames.add(&format!("{}.{}", e.name, v.name), &variantName);
                            importedNames.add(&variantName, &variantName);
                        }
                        for m in &e.methods {
                            let methodName = enumName.add(m.name.toString());
                            importedNames.add(&m.name, &methodName);
                            importedNames.add(&format!("{}.{}", e.name, m.name), &methodName);
                            importedNames.add(&methodName, &methodName);
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
                        for m in &t.methods {
                            let methodName = traitName.add(m.name.toString());
                            importedNames.add(&m.name, &methodName);
                            importedNames.add(&format!("{}.{}", t.name, m.name), &methodName);
                            importedNames.add(&methodName, &methodName);
                        }
                    }
                    ModuleItem::Instance(_) => {}
                }
            }
        }
    }

    fn processImports(&mut self) {
        for (_, m) in &self.modules {
            //println!("Processing module {}", name);
            let mut importedNames = Names::new();
            let mut importedModules = Vec::new();
            for item in &m.items {
                match item {
                    ModuleItem::Import(i) => {
                        let moduleName = i.moduleName.toString();
                        match self.modules.get(&moduleName) {
                            Some(sourceModule) => {
                                importedModules.push(moduleName);
                                Resolver::processSourceModule(sourceModule, &mut importedNames, i);
                            }
                            None => {
                                if !i.implicitImport {
                                    error(format!("Imported module not found {}", i.moduleName.toString()));
                                }
                            }
                        };
                    }
                    _ => {}
                }
            }
            let moduleResolver = self.resolvers.get_mut(&m.name.toString()).unwrap();
            moduleResolver.importedNames = importedNames;
            moduleResolver.importedModules = importedModules;
        }
    }

    fn collectLocalNames(&mut self) {
        for (_, m) in &self.modules {
            //println!("Processing module {}", name);
            let moduleResolver = ModuleResolver {
                ctx: self.ctx,
                name: m.name.toString(),
                localNames: Resolver::buildLocalNames(m),
                importedNames: Names::new(),
                importedModules: Vec::new(),
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
