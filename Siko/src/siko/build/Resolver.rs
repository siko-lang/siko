use crate::siko::{
    ir::{
        Data::{Class as IrClass, Field as IrField, MethodInfo},
        Function::{Function as IrFunction, Parameter},
    },
    qualifiedname::QualifiedName,
    resolver::{
        ModuleResolver::{LocalNames, ModuleResolver},
        Resolver::{createConstraintContext, Names, Resolver},
        TypeResolver::TypeResolver,
    },
    syntax::Module::ModuleItem,
    util::error,
};

use super::Build::{ArtifactKind, BuildArtifact, BuildEngine, Key};

pub fn buildModuleResolver(name: String, engine: &mut BuildEngine) {
    let m = engine.get(Key::Module(name));
    let m = m.kind.asModule();
    let mut importedNames = Names::new();
    for item in &m.items {
        match item {
            ModuleItem::Import(i) => {
                let moduleName = i.moduleName.toString();
                match engine.getOpt(Key::Module(moduleName)) {
                    Some(sourceModule) => {
                        let sourceModule = sourceModule.kind.asModule();
                        Resolver::processSourceModule(sourceModule, &mut importedNames, i);
                    }
                    None => {
                        if !i.implicitImport {
                            error(format!(
                                "Imported module not found {}",
                                i.moduleName.toString()
                            ));
                        }
                    }
                };
            }
            _ => {}
        }
    }
    let moduleResolver = ModuleResolver {
        name: m.name.toString(),
        localNames: Resolver::buildLocalNames(m),
        importedNames: importedNames,
    };
    engine.add(BuildArtifact::new(ArtifactKind::ModuleResolver(
        moduleResolver,
    )));
}

pub fn buildLocalNames(name: String, engine: &mut BuildEngine) {
    let m = engine.get(Key::Module(name));
    let m = m.kind.asModule();
    let localNames = Resolver::buildLocalNames(m);
    let localNames = LocalNames {
        name: m.name.toString(),
        localNames: localNames,
    };
    engine.add(BuildArtifact::new(ArtifactKind::LocalNames(localNames)));
}

pub fn buildResolvedClass(n: QualifiedName, engine: &mut BuildEngine) {
    let moduleResolver = engine.get(Key::ModuleResolver(n.module().toString()));
    let c = engine.get(Key::Class(n.clone()));
    let c = &c.kind.asClass().c;
    let moduleResolver = moduleResolver.kind.asModuleResolver();
    let constraintContext = createConstraintContext(&c.typeParams);
    let typeResolver = TypeResolver::new(moduleResolver, &constraintContext);
    let irType = typeResolver.createDataType(&c.name, &c.typeParams);
    let mut irClass = IrClass::new(moduleResolver.resolverName(&c.name), irType.clone());
    let mut ctorParams = Vec::new();
    for field in &c.fields {
        let ty = typeResolver.resolveType(&field.ty);
        ctorParams.push(Parameter::Named(field.name.toString(), ty.clone(), false));
        irClass.fields.push(IrField {
            name: field.name.toString(),
            ty,
        })
    }
    let ctor = IrFunction::new(irClass.name.clone(), ctorParams, irType, None);
    //self.functions.insert(ctor.name.clone(), ctor);
    for method in &c.methods {
        irClass.methods.push(MethodInfo {
            name: method.name.toString(),
            fullName: moduleResolver.resolverName(&method.name),
        })
    }
    engine.add(BuildArtifact::new(ArtifactKind::ResolvedClass(irClass)));
}
