use crate::siko::{
    resolver::{
        ModuleResolver::{LocalNames, ModuleResolver},
        Resolver::{Names, Resolver},
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
