use crate::siko::{
    hir::{
        ConstraintContext::{Constraint as IrConstraint, ConstraintContext},
        Data::MethodInfo as DataMethodInfo,
        Function::{Attributes, FunctionKind, Parameter},
        InstanceStore::InstanceStore,
        Instantiation::instantiateTraitWithSub,
        Program::Program,
        Trait::{AssociatedType, MemberInfo},
        TraitMethodSelector::{TraitMethodSelection, TraitMethodSelector},
        Type::TypeVar,
        TypeVarAllocator::TypeVarAllocator,
        Unifier::Unifier,
    },
    location::Report::ReportContext,
    qualifiedname::QualifiedName,
    resolver::{autoderive::AutoDerive, FunctionResolver::FunctionResolver, Util::SubstitutionChain},
    syntax::{
        Function::{Function, Parameter as SynParam},
        Module::{Import, Module, ModuleItem},
        Trait::Instance,
        Type::{Constraint, ConstraintArgument, TypeParameterDeclaration},
    },
    util::error,
};

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    iter::zip,
};

use super::{Error::ResolverError, ModuleResolver::ModuleResolver};
use super::{FunctionResolver::createSelfType, TypeResolver::TypeResolver};
use crate::siko::hir::Data::Enum as IrEnum;
use crate::siko::hir::Data::Field as IrField;
use crate::siko::hir::Data::Struct as irStruct;
use crate::siko::hir::Data::Variant as IrVariant;
use crate::siko::hir::Function::Function as IrFunction;
use crate::siko::hir::Implicit::Implicit as IrImplicit;
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
    let traitDef = typeResolver.moduleResolver.lookupTrait(&constraint.name, program);
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
                        constraint.name.location(),
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
        name: traitDef.name,
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
            let irParam = IrType::Var(TypeVar::Named(param.name()));
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
            params.push(IrType::Var(TypeVar::Named(param.name())));
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
    defaultTraitMethods: BTreeMap<QualifiedName, Function>,
    instanceMethods: BTreeMap<QualifiedName, Function>,
    instanceSubChains: BTreeMap<QualifiedName, SubstitutionChain>,
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
            defaultTraitMethods: BTreeMap::new(),
            instanceMethods: BTreeMap::new(),
            instanceSubChains: BTreeMap::new(),
        }
    }

    pub fn addModule(&mut self, m: Module) {
        let m = AutoDerive::processModule(&m);
        self.modules.insert(m.name.toString(), m);
    }

    pub fn process(&mut self) {
        self.collectLocalNames();
        self.processImports();
        self.processDataTypes();
        self.processImplicits();
        self.processTraits();
        self.processInstances();
        self.processFunctions();

        self.updateTraitMethodSelectors();
        //self.dump();
    }

    fn updateTraitMethodSelectors(&mut self) {
        let mut traitMethodselectors = BTreeMap::new();

        for resolver in self.resolvers.values() {
            let name = QualifiedName::Module(resolver.name.clone());
            let mut selector = self
                .program
                .traitMethodselectors
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
                    .traitMethodselectors
                    .get(&importedModuleName)
                    .expect("trait method selector not found");
                selector.merge(moduleSelector);
            }
            traitMethodselectors.insert(name, selector);
        }
        self.program.traitMethodselectors = traitMethodselectors;
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
            let moduleResolver = self.resolvers.get(&m.name.name()).unwrap();
            for item in &m.items {
                match item {
                    ModuleItem::Struct(c) => {
                        let typeParams = getTypeParams(&c.typeParams);
                        let typeResolver = TypeResolver::new(moduleResolver, &typeParams);
                        let structName = QualifiedName::Module(moduleResolver.name.clone()).add(c.name.toString());
                        let constraintContext =
                            createConstraintContext(&c.typeParams, &typeResolver, &self.program, &self.ctx);
                        let irType = typeResolver.createDataType(&structName, &c.typeParams);
                        let mut irStruct = irStruct::new(structName, irType.clone(), c.name.location());
                        let mut ctorParams = Vec::new();
                        for field in &c.fields {
                            let ty = typeResolver.resolveType(&field.ty);
                            ctorParams.push(Parameter::Named(field.name.toString(), ty.clone(), false));
                            irStruct.fields.push(IrField {
                                name: field.name.toString(),
                                ty,
                            })
                        }
                        let ctor = IrFunction::new(
                            irStruct.name.clone(),
                            ctorParams,
                            irType,
                            None,
                            constraintContext,
                            FunctionKind::StructCtor,
                            Attributes::new(),
                        );
                        self.program.functions.insert(ctor.name.clone(), ctor);
                        for method in &c.methods {
                            irStruct.methods.push(DataMethodInfo {
                                name: method.name.toString(),
                                fullName: irStruct.name.add(method.name.toString()),
                            })
                        }
                        //println!("Struct {:?}", irStruct);
                        self.program.structs.insert(irStruct.name.clone(), irStruct);
                    }
                    ModuleItem::Enum(e) => {
                        let typeParams = getTypeParams(&e.typeParams);
                        let typeResolver = TypeResolver::new(moduleResolver, &typeParams);
                        let enumName = QualifiedName::Module(moduleResolver.name.clone()).add(e.name.toString());
                        let constraintContext =
                            createConstraintContext(&e.typeParams, &typeResolver, &self.program, &self.ctx);
                        let irType = typeResolver.createDataType(&enumName, &e.typeParams);
                        let mut irEnum = IrEnum::new(enumName, irType.clone(), e.name.location());
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
                                Attributes::new(),
                            );
                            self.program.functions.insert(ctor.name.clone(), ctor);
                            self.variants.insert(variant.name.clone(), irEnum.name.clone());
                            self.program.variants.insert(variant.name.clone());
                            irEnum.variants.push(variant);
                        }
                        for method in &e.methods {
                            irEnum.methods.push(DataMethodInfo {
                                name: method.name.toString(),
                                fullName: irEnum.name.add(method.name.toString()),
                            })
                        }
                        self.program.enums.insert(irEnum.name.clone(), irEnum);
                    }
                    _ => {}
                }
            }
        }
    }

    fn processTraits(&mut self) {
        //println!("Processing traits");
        for (_, m) in &mut self.modules {
            let moduleResolver = self.resolvers.get(&m.name.name()).unwrap();
            for item in &mut m.items {
                match item {
                    ModuleItem::Trait(traitDef) => {
                        let typeParams = getTypeParams(&traitDef.typeParams);
                        let mut typeResolver = TypeResolver::new(moduleResolver, &typeParams);
                        let constraintContext =
                            createConstraintContext(&traitDef.typeParams, &typeResolver, &self.program, &self.ctx);
                        let mut irParams = Vec::new();
                        for param in &traitDef.params {
                            let irParam = IrType::Var(TypeVar::Named(param.toString()));
                            typeResolver.addTypeParams(irParam.clone());
                            irParams.push(irParam);
                        }
                        let mut associatedTypes = Vec::new();
                        let traitName = moduleResolver.resolveName(&traitDef.name);
                        for associatedType in &traitDef.associatedTypes {
                            associatedTypes.push(associatedType.name.name());
                            let irParam = IrType::Var(TypeVar::Named(associatedType.name.toString()));
                            typeResolver.addTypeParams(irParam);
                        }
                        let selfType = irParams[0].clone();
                        let mut irTrait = IrTrait::new(traitName, irParams, associatedTypes, constraintContext);
                        for method in &traitDef.methods {
                            let mut argTypes = Vec::new();
                            for param in &method.params {
                                let ty = match param {
                                    SynParam::Named(_, ty, _) => typeResolver.resolveType(ty),
                                    SynParam::SelfParam => selfType.clone(),
                                    SynParam::MutSelfParam => selfType.clone(),
                                    SynParam::RefSelfParam => selfType.asRef(),
                                };
                                argTypes.push(ty);
                            }
                            let result = typeResolver
                                .resolveType(&method.result)
                                .changeSelfType(selfType.clone());
                            let fullName = QualifiedName::Item(Box::new(irTrait.name.clone()), method.name.toString());
                            let isDefault = method.body.is_some();
                            irTrait.members.push(MemberInfo {
                                name: method.name.toString(),
                                fullName: fullName.clone(),
                                default: isDefault,
                                memberType: IrType::Function(argTypes, Box::new(result)),
                                constraint: ConstraintContext::new(),
                            });
                            if isDefault {
                                self.defaultTraitMethods.insert(fullName, method.clone());
                            }
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
        //println!("Processing instances");
        for (_, m) in &mut self.modules {
            let moduleResolver = self.resolvers.get(&m.name.name()).unwrap();
            for item in &mut m.items {
                match item {
                    ModuleItem::Instance(i) => {
                        let typeParams = getTypeParams(&i.typeParams);
                        let typeResolver = TypeResolver::new(moduleResolver, &typeParams);
                        let traitName = moduleResolver.resolveName(&i.traitName);
                        let qn = buildInstanceName(moduleResolver, i, traitName.clone(), &typeResolver);
                        let constraintContext =
                            createConstraintContext(&i.typeParams, &typeResolver, &self.program, &self.ctx);
                        let traitDef = moduleResolver.lookupTrait(&i.traitName, &self.program);
                        let args: Vec<_> = i.types.iter().map(|ty| typeResolver.resolveType(ty)).collect();
                        let mut associatedTypes = Vec::new();
                        for associatedType in &i.associatedTypes {
                            let mut found = false;
                            for traitAssociatedType in &traitDef.associatedTypes {
                                if traitAssociatedType == &associatedType.name.name() {
                                    found = true;
                                    break;
                                }
                            }
                            if !found {
                                ResolverError::AssociatedTypeNotFound(
                                    associatedType.name.name(),
                                    traitDef.name.toString(),
                                    associatedType.name.location(),
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
                        let mut irInstance = IrInstance::new(
                            qn.clone(),
                            traitDef.name.clone(),
                            args.clone(),
                            typeParams.clone(),
                            associatedTypes,
                            constraintContext,
                        );
                        let allocator = TypeVarAllocator::new();
                        let (instantiatedTrait, sub) = instantiateTraitWithSub(&allocator, &traitDef);
                        let mut subChain = SubstitutionChain::new();
                        subChain.add(sub);
                        let mut allTraitMembers = BTreeSet::new();
                        let mut neededTraitMembers = BTreeSet::new();
                        let mut usedDefaults = BTreeMap::new();
                        for method in &instantiatedTrait.members {
                            allTraitMembers.insert(method.name.clone());
                            if method.default {
                                usedDefaults.insert(method.name.clone(), method.clone());
                            } else {
                                neededTraitMembers.insert(method.name.clone());
                            }
                        }
                        let mut unifier = Unifier::withContext(&self.ctx);
                        for (t1, t2) in zip(instantiatedTrait.params.clone(), irInstance.types.clone()) {
                            unifier.unify(t1, t2, i.location.clone());
                        }
                        subChain.add(unifier.substitution.borrow().clone());
                        self.instanceSubChains.insert(irInstance.name.clone(), subChain);
                        let mut implementedMembers = BTreeSet::new();
                        for method in &i.methods {
                            implementedMembers.insert(method.name.clone());
                            if !allTraitMembers.contains(&method.name.name()) {
                                ResolverError::InvalidInstanceMember(
                                    method.name.name(),
                                    irInstance.traitName.toString(),
                                    method.name.location(),
                                )
                                .report(self.ctx);
                            }
                            neededTraitMembers.remove(&method.name.name());
                            usedDefaults.remove(&method.name.name());
                            let mut argTypes = Vec::new();
                            for param in &method.params {
                                let ty = match param {
                                    SynParam::Named(_, ty, _) => typeResolver.resolveType(ty),
                                    SynParam::SelfParam => selfType.clone(),
                                    SynParam::MutSelfParam => selfType.clone(),
                                    SynParam::RefSelfParam => selfType.asRef(),
                                };
                                argTypes.push(ty);
                            }
                            let result = typeResolver
                                .resolveType(&method.result)
                                .changeSelfType(selfType.clone());
                            let constraintContext = addTypeParams(
                                irInstance.constraintContext.clone(),
                                &method.typeParams,
                                &typeResolver,
                                &self.program,
                                &self.ctx,
                            );
                            let fullName =
                                QualifiedName::Item(Box::new(irInstance.name.clone()), method.name.toString());
                            irInstance.members.push(MemberInfo {
                                name: method.name.toString(),
                                fullName: fullName.clone(),
                                default: false,
                                memberType: IrType::Function(argTypes, Box::new(result)),
                                constraint: constraintContext,
                            });
                            self.instanceMethods.insert(fullName, method.clone());
                        }
                        for (name, method) in usedDefaults {
                            //println!("Using default method {}", method);
                            let memberType = unifier.apply(method.memberType);
                            //println!("Resolved member type: {}", memberType);
                            let constraintContext = addTypeParams(
                                irInstance.constraintContext.clone(),
                                &None,
                                &typeResolver,
                                &self.program,
                                &self.ctx,
                            );
                            let fullName = QualifiedName::Item(Box::new(irInstance.name.clone()), name.clone());
                            irInstance.members.push(MemberInfo {
                                name: name.clone(),
                                fullName: fullName.clone(),
                                default: true,
                                memberType: memberType,
                                constraint: constraintContext,
                            });
                            let methodFn = self
                                .defaultTraitMethods
                                .get(&method.fullName)
                                .expect("default method not found")
                                .clone();
                            self.instanceMethods.insert(fullName, methodFn);
                        }
                        if !neededTraitMembers.is_empty() {
                            ResolverError::MissingTraitMembers(
                                neededTraitMembers.into_iter().collect(),
                                irInstance.traitName.toString(),
                                i.traitName.location(),
                            )
                            .report(self.ctx);
                        }
                        //println!("IrImpl {}", irInstance);
                        self.program.instances.insert(irInstance.name.clone(), irInstance);
                        if i.name.is_none() {
                            self.program
                                .canonicalImplStore
                                .insert(traitName, args, qn, i.location.clone(), self.ctx);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn processFunctions(&mut self) {
        //println!("Processing functions");
        for (_, m) in &self.modules {
            let moduleResolver = self.resolvers.get(&m.name.name()).unwrap();
            let mut traitMethodselector = TraitMethodSelector::new();
            for item in &m.items {
                match item {
                    ModuleItem::Struct(c) => {
                        let owner = createSelfType(&c.name, c.typeParams.as_ref(), moduleResolver);
                        let typeParams = getTypeParams(&c.typeParams);
                        let typeResolver = TypeResolver::new(moduleResolver, &typeParams);
                        let constraintContext =
                            createConstraintContext(&c.typeParams, &typeResolver, &self.program, &self.ctx);
                        for method in &c.methods {
                            let constraintContext = addTypeParams(
                                constraintContext.clone(),
                                &method.typeParams,
                                &typeResolver,
                                &self.program,
                                &self.ctx,
                            );
                            let functionResolver =
                                FunctionResolver::new(moduleResolver, constraintContext, Some(owner.clone()));
                            let irFunction = functionResolver.resolve(
                                self.ctx,
                                method,
                                &self.emptyVariants,
                                &self.program.structs,
                                &self.variants,
                                &self.program.enums,
                                &self.program.implicits,
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
                            let constraintContext = addTypeParams(
                                constraintContext.clone(),
                                &method.typeParams,
                                &typeResolver,
                                &self.program,
                                &self.ctx,
                            );
                            let functionResolver =
                                FunctionResolver::new(moduleResolver, constraintContext.clone(), Some(owner.clone()));
                            let irFunction = functionResolver.resolve(
                                self.ctx,
                                method,
                                &self.emptyVariants,
                                &self.program.structs,
                                &self.variants,
                                &self.program.enums,
                                &self.program.implicits,
                                owner.getName().unwrap().add(method.name.toString()),
                                &typeResolver,
                            );
                            self.program.functions.insert(irFunction.name.clone(), irFunction);
                        }
                    }
                    ModuleItem::Trait(syntaxTraitDef) => {
                        let name = moduleResolver.resolveName(&syntaxTraitDef.name);
                        let traitDef = self.program.getTrait(&name).unwrap();
                        let owner = traitDef.params.first().expect("first trait param not found");
                        let typeParams = getTypeParams(&syntaxTraitDef.typeParams);
                        let mut typeResolver = TypeResolver::new(moduleResolver, &typeParams);
                        let mut constraintContext = createConstraintContext(
                            &syntaxTraitDef.typeParams,
                            &typeResolver,
                            &self.program,
                            &self.ctx,
                        );
                        for param in &traitDef.params {
                            typeResolver.addTypeParams(param.clone());
                            constraintContext.addTypeParam(param.clone());
                        }
                        for associatedType in &traitDef.associatedTypes {
                            typeResolver.addTypeParams(IrType::Var(TypeVar::Named(associatedType.clone())));
                        }
                        for method in &syntaxTraitDef.methods {
                            //println!("Processing trait method {}", method.name);
                            let mut constraintContext = addTypeParams(
                                constraintContext.clone(),
                                &method.typeParams,
                                &typeResolver,
                                &self.program,
                                &self.ctx,
                            );
                            let mut associatedTypes = Vec::new();
                            for assocTy in &syntaxTraitDef.associatedTypes {
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
                                name: traitDef.name.clone(),
                                args: traitDef.params.clone(),
                                associatedTypes: associatedTypes,
                            });
                            let functionResolver =
                                FunctionResolver::new(moduleResolver, constraintContext, Some(owner.clone()));
                            let mut irFunction = functionResolver.resolve(
                                self.ctx,
                                method,
                                &self.emptyVariants,
                                &self.program.structs,
                                &self.variants,
                                &self.program.enums,
                                &self.program.implicits,
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
                            traitMethodselector.add(method.name.toString(), selection);
                            self.program.functions.insert(irFunction.name.clone(), irFunction);
                        }
                    }
                    ModuleItem::Instance(i) => {
                        let typeParams = getTypeParams(&i.typeParams);
                        let typeResolver = TypeResolver::new(moduleResolver, &typeParams);
                        let traitName = moduleResolver.resolveName(&i.traitName);
                        let qn = buildInstanceName(moduleResolver, i, traitName, &typeResolver);
                        let defaultSubstitutionChain = self
                            .instanceSubChains
                            .get(&qn)
                            .expect("instance substitution chain not found");
                        let defaultfnTypeResolver = typeResolver.withSubChain(&defaultSubstitutionChain);
                        let irInstance = &self.program.getInstance(&qn).expect("Instance not found");
                        for method in &irInstance.members {
                            //println!("Processing instance method {} ({})", method.name, method.fullName);
                            let functionResolver = FunctionResolver::new(
                                moduleResolver,
                                method.constraint.clone(),
                                Some(irInstance.types[0].clone()),
                            );
                            let typeResolver = if method.default {
                                &defaultfnTypeResolver
                            } else {
                                &typeResolver
                            };
                            let methodFn = self
                                .instanceMethods
                                .get(&method.fullName)
                                .expect("instance method not found");
                            let irFunction = functionResolver.resolve(
                                self.ctx,
                                methodFn,
                                &self.emptyVariants,
                                &self.program.structs,
                                &self.variants,
                                &self.program.enums,
                                &self.program.implicits,
                                qn.add(method.name.clone()),
                                &typeResolver,
                            );
                            self.program.functions.insert(irFunction.name.clone(), irFunction);
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
                            &self.program.structs,
                            &self.variants,
                            &self.program.enums,
                            &self.program.implicits,
                            QualifiedName::Module(moduleResolver.name.clone()).add(f.name.toString()),
                            &typeResolver,
                        );
                        self.program.functions.insert(irFunction.name.clone(), irFunction);
                    }
                    ModuleItem::Effect(effect) => {
                        let typeParams = getTypeParams(&None);
                        let typeResolver = TypeResolver::new(moduleResolver, &typeParams);
                        for method in &effect.methods {
                            let functionResolver =
                                FunctionResolver::new(moduleResolver, ConstraintContext::new(), None);
                            let name = QualifiedName::Module(moduleResolver.name.clone())
                                .add(effect.name.toString())
                                .add(method.name.toString());
                            let mut irFunction = functionResolver.resolve(
                                self.ctx,
                                method,
                                &self.emptyVariants,
                                &self.program.structs,
                                &self.variants,
                                &self.program.enums,
                                &self.program.implicits,
                                name.clone(),
                                &typeResolver,
                            );
                            if irFunction.body.is_none() {
                                irFunction.kind = FunctionKind::EffectMemberDecl(name.clone());
                            } else {
                                irFunction.kind = FunctionKind::EffectMemberDefinition(name.clone());
                            }
                            self.program.functions.insert(irFunction.name.clone(), irFunction);
                        }
                    }
                    _ => {}
                }
            }
            self.program
                .traitMethodselectors
                .insert(QualifiedName::Module(m.name.toString()), traitMethodselector);
        }
    }

    pub fn processSourceModule(
        sourceModule: &Module,
        importedNames: &mut Names,
        variants: &mut BTreeSet<QualifiedName>,
        i: &Import,
        importedInstances: &mut Vec<QualifiedName>,
    ) {
        if let Some(alias) = &i.alias {
            let moduleName = QualifiedName::Module(i.moduleName.toString());
            let localModuleName = QualifiedName::Module(alias.toString());
            for item in &sourceModule.items {
                match item {
                    ModuleItem::Struct(structDef) => {
                        if !structDef.public {
                            continue;
                        }
                        let structName = moduleName.add(structDef.name.toString());
                        let localStructName = localModuleName.add(structDef.name.toString());
                        importedNames.add(&localStructName, &structName);
                        for fnDef in &structDef.methods {
                            if !fnDef.public {
                                continue;
                            }
                            let methodName = structName.add(fnDef.name.to_string());
                            let localMethodName = localStructName.add(fnDef.name.to_string());
                            importedNames.add(&localMethodName, &methodName);
                        }
                    }
                    ModuleItem::Enum(enumDef) => {
                        if !enumDef.public {
                            continue;
                        }
                        let enumName = moduleName.add(enumDef.name.toString());
                        let localEnumName = localModuleName.add(enumDef.name.toString());
                        importedNames.add(&localEnumName, &enumName);
                        for variantDef in &enumDef.variants {
                            let variantName = enumName.add(variantDef.name.toString());
                            variants.insert(variantName.clone());
                            let localVariantName = localEnumName.add(variantDef.name.toString());
                            importedNames.add(&localVariantName, &variantName);
                        }
                        for fnDef in &enumDef.methods {
                            if !fnDef.public {
                                continue;
                            }
                            let methodName = enumName.add(fnDef.name.toString());
                            let localMethodName = localEnumName.add(fnDef.name.toString());
                            importedNames.add(&localMethodName, &methodName);
                        }
                    }
                    ModuleItem::Function(fnDef) => {
                        if !fnDef.public {
                            continue;
                        }
                        let functionName = moduleName.add(fnDef.name.toString());
                        let localFunctionName = localModuleName.add(fnDef.name.toString());
                        importedNames.add(&localFunctionName, &functionName);
                    }
                    ModuleItem::Import(_) => {}
                    ModuleItem::Trait(traitDef) => {
                        if !traitDef.public {
                            continue;
                        }
                        let traitName = moduleName.add(traitDef.name.toString());
                        let localTraitName = localModuleName.add(traitDef.name.toString());
                        importedNames.add(&localTraitName, &traitName);
                        for fnDef in &traitDef.methods {
                            let methodName = traitName.add(fnDef.name.toString());
                            let localMethodName = localTraitName.add(fnDef.name.to_string());
                            importedNames.add(&localMethodName, &methodName);
                        }
                    }
                    ModuleItem::Instance(instanceDef) => {
                        if !instanceDef.public {
                            continue;
                        }
                        if let Some(name) = &instanceDef.name {
                            let instanceName = moduleName.add(name.toString());
                            let localImplName = localModuleName.add(name.toString());
                            importedNames.add(&localImplName, &instanceName);
                            importedInstances.push(instanceName);
                        }
                    }
                    ModuleItem::Effect(effectDef) => {
                        if !effectDef.public {
                            continue;
                        }
                        let effectName = moduleName.add(effectDef.name.toString());
                        let localEffectName = localModuleName.add(effectDef.name.toString());
                        importedNames.add(&localEffectName, &effectName);
                        for fnDef in &effectDef.methods {
                            let methodName = effectName.add(fnDef.name.to_string());
                            let localMethodName = localEffectName.add(fnDef.name.to_string());
                            importedNames.add(&localMethodName, &methodName);
                            let localMethodName = localModuleName.add(fnDef.name.to_string());
                            importedNames.add(&localMethodName, &methodName);
                        }
                    }
                    ModuleItem::Implicit(i) => {
                        if !i.public {
                            continue;
                        }
                        let implicitName = moduleName.add(i.name.toString());
                        let localImplicitName = localModuleName.add(i.name.toString());
                        importedNames.add(&localImplicitName, &implicitName);
                    }
                }
            }
        } else {
            let moduleName = QualifiedName::Module(sourceModule.name.toString());
            for item in &sourceModule.items {
                match item {
                    ModuleItem::Struct(structDef) => {
                        //format_any(structDef);
                        if !structDef.public {
                            continue;
                        }
                        let structName = moduleName.add(structDef.name.toString());
                        importedNames.add(&structDef.name, &structName);
                        importedNames.add(&structName, &structName);
                        for fnDef in &structDef.methods {
                            if !fnDef.public {
                                continue;
                            }
                            let methodName = structName.add(fnDef.name.toString());
                            importedNames.add(&fnDef.name, &methodName);
                            importedNames.add(&format!("{}.{}", structDef.name, fnDef.name), &methodName);
                            importedNames.add(&methodName, &methodName);
                        }
                    }
                    ModuleItem::Enum(enumDef) => {
                        //format_any(enumDef);
                        if !enumDef.public {
                            continue;
                        }
                        let enumName = moduleName.add(enumDef.name.toString());
                        importedNames.add(&enumDef.name, &enumName);
                        importedNames.add(&enumName, &enumName);
                        for variantDef in &enumDef.variants {
                            let variantName = enumName.add(variantDef.name.toString());
                            variants.insert(variantName.clone());
                            importedNames.add(&variantDef.name, &variantName);
                            importedNames.add(&format!("{}.{}", enumDef.name, variantDef.name), &variantName);
                            importedNames.add(&variantName, &variantName);
                        }
                        for fnDef in &enumDef.methods {
                            if !fnDef.public {
                                continue;
                            }
                            let methodName = enumName.add(fnDef.name.toString());
                            importedNames.add(&fnDef.name, &methodName);
                            importedNames.add(&format!("{}.{}", enumDef.name, fnDef.name), &methodName);
                            importedNames.add(&methodName, &methodName);
                        }
                    }
                    ModuleItem::Function(fnDef) => {
                        //format_any(fnDef);
                        if !fnDef.public {
                            continue;
                        }
                        let functionName = moduleName.add(fnDef.name.toString());
                        importedNames.add(&fnDef.name, &functionName);
                        importedNames.add(&functionName, &functionName);
                    }
                    ModuleItem::Import(_) => {}
                    ModuleItem::Trait(traitDef) => {
                        //format_any(traitDef);
                        if !traitDef.public {
                            continue;
                        }
                        let traitName = moduleName.add(traitDef.name.toString());
                        importedNames.add(&traitDef.name, &traitName);
                        importedNames.add(&traitName, &traitName);
                        for fnDef in &traitDef.methods {
                            let methodName = traitName.add(fnDef.name.toString());
                            importedNames.add(&fnDef.name, &methodName);
                            importedNames.add(&format!("{}.{}", traitDef.name, fnDef.name), &methodName);
                            importedNames.add(&methodName, &methodName);
                        }
                    }
                    ModuleItem::Instance(instanceDef) => {
                        //format_any(instanceDef);
                        if !instanceDef.public {
                            continue;
                        }
                        if let Some(name) = &instanceDef.name {
                            let instanceName = moduleName.add(name.toString());
                            importedNames.add(&name, &instanceName);
                            importedNames.add(&instanceName, &instanceName);
                            importedInstances.push(instanceName);
                        }
                    }
                    ModuleItem::Effect(effectDef) => {
                        if !effectDef.public {
                            continue;
                        }
                        let effectName = moduleName.add(effectDef.name.toString());
                        importedNames.add(&effectDef.name, &effectName);
                        importedNames.add(&effectName, &effectName);
                        for fnDef in &effectDef.methods {
                            let methodName = effectName.add(fnDef.name.to_string());
                            importedNames.add(&fnDef.name, &methodName);
                            importedNames.add(&format!("{}.{}", effectDef.name, fnDef.name), &methodName);
                            importedNames.add(&methodName, &methodName);
                        }
                    }
                    ModuleItem::Implicit(i) => {
                        if !i.public {
                            continue;
                        }
                        let implicitName = moduleName.add(i.name.toString());
                        importedNames.add(&i.name, &implicitName);
                        importedNames.add(&implicitName, &implicitName);
                    }
                }
            }
        }
    }

    fn processImports(&mut self) {
        for (_, m) in &self.modules {
            let moduleResolver = self.resolvers.get_mut(&m.name.toString()).unwrap();
            //println!("Processing module {}", name);
            let mut importedInstances = Vec::new();
            for item in &m.items {
                match item {
                    ModuleItem::Import(i) => {
                        let moduleName = i.moduleName.toString();
                        match self.modules.get(&moduleName) {
                            Some(sourceModule) => {
                                moduleResolver.importedModules.push(moduleName);
                                Resolver::processSourceModule(
                                    sourceModule,
                                    &mut moduleResolver.importedNames,
                                    &mut moduleResolver.variants,
                                    i,
                                    &mut importedInstances,
                                );
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
            let instanceStore = self
                .program
                .instanceStores
                .entry(QualifiedName::Module(moduleResolver.name.clone()))
                .or_insert_with(InstanceStore::new);
            instanceStore.importedInstances = importedInstances;
        }
    }

    fn collectLocalNames(&mut self) {
        for (_, m) in &self.modules {
            //println!("Processing module {}", name);
            let (localNames, variants, instances) = Resolver::buildLocalNames(m);
            let moduleResolver = ModuleResolver {
                ctx: self.ctx,
                name: m.name.toString(),
                localNames,
                importedNames: Names::new(),
                importedModules: Vec::new(),
                variants,
            };
            self.program.instanceStores.insert(
                QualifiedName::Module(m.name.toString()),
                InstanceStore {
                    localInstances: instances.clone(),
                    importedInstances: Vec::new(),
                },
            );
            self.resolvers.insert(m.name.toString(), moduleResolver);
        }
    }

    pub fn buildLocalNames(m: &Module) -> (Names, BTreeSet<QualifiedName>, Vec<QualifiedName>) {
        let mut localNames = Names::new();
        let mut variants = BTreeSet::new();
        let mut instances = Vec::new();
        let moduleName = QualifiedName::Module(m.name.toString());
        for item in &m.items {
            match item {
                ModuleItem::Struct(c) => {
                    let structName = moduleName.add(c.name.toString());
                    localNames.add(&c.name, &structName);
                    localNames.add(&structName, &structName);
                    for m in &c.methods {
                        let methodName = structName.add(m.name.toString());
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
                        variants.insert(variantName.clone());
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
                ModuleItem::Instance(i) => {
                    if let Some(name) = &i.name {
                        let instanceName = moduleName.add(name.toString());
                        localNames.add(&name, &instanceName);
                        localNames.add(&instanceName, &instanceName);
                        instances.push(instanceName);
                    }
                }
                ModuleItem::Effect(e) => {
                    let effectName = moduleName.add(e.name.toString());
                    localNames.add(&e.name, &effectName);
                    localNames.add(&effectName, &effectName);
                    for m in &e.methods {
                        let methodName = effectName.add(m.name.toString());
                        localNames.add(&m.name, &methodName);
                        localNames.add(&format!("{}.{}", e.name, m.name), &methodName);
                        localNames.add(&methodName, &methodName);
                    }
                }
                ModuleItem::Implicit(i) => {
                    let implicitName = moduleName.add(i.name.toString());
                    localNames.add(&i.name, &implicitName);
                    localNames.add(&implicitName, &implicitName);
                }
            }
        }
        (localNames, variants, instances)
    }

    fn processImplicits(&mut self) {
        for (_, m) in &self.modules {
            let moduleResolver = self.resolvers.get(&m.name.name()).unwrap();
            for item in &m.items {
                match item {
                    ModuleItem::Implicit(i) => {
                        let name = QualifiedName::Module(moduleResolver.name.clone()).add(i.name.to_string());
                        let typeParams = getTypeParams(&None);
                        let typeResolver = TypeResolver::new(moduleResolver, &typeParams);
                        let ty = typeResolver.resolveType(&i.ty);
                        let irImplicit = IrImplicit {
                            name: name.clone(),
                            ty,
                            mutable: i.mutable,
                        };
                        //println!("Adding implicit: {}", name);
                        self.program.implicits.insert(name, irImplicit);
                    }
                    _ => {}
                }
            }
        }
    }
}

fn buildInstanceName(
    moduleResolver: &ModuleResolver<'_>,
    i: &Instance,
    traitName: QualifiedName,
    typeResolver: &TypeResolver<'_>,
) -> QualifiedName {
    let qn = match &i.name {
        Some(n) => QualifiedName::Module(moduleResolver.name.clone()).add(n.toString()),
        None => QualifiedName::Module(moduleResolver.name.clone()).canonical(
            traitName,
            i.types.iter().map(|ty| typeResolver.resolveType(ty)).collect(),
        ),
    };
    qn
}
