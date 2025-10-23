use std::collections::BTreeSet;

use crate::siko::location::Report::ReportContext;
use crate::siko::qualifiedname::QualifiedName;
use crate::siko::resolver::Error::ResolverError;
use crate::siko::syntax::Identifier::Identifier;

use super::Resolver::Names;

#[derive(Debug, PartialEq, Eq)]
pub struct LocalNames {
    pub name: String,
    pub localNames: Names,
}

#[derive(Debug)]
pub struct ModuleResolver<'a> {
    pub ctx: &'a ReportContext,
    pub name: String,
    pub localNames: Names,
    pub importedNames: Names,
    pub implicitlyImportedNames: Names,
    pub importedModules: Vec<String>,
    pub variants: BTreeSet<QualifiedName>,
    pub globals: BTreeSet<QualifiedName>,
}

impl<'a> ModuleResolver<'a> {
    // pub fn lookupTrait(&self, name: &Identifier, program: &Program) -> Trait {
    //     let qn = &self.resolveName(name);
    //     if let Some(traitDef) = program.getTrait(qn) {
    //         traitDef
    //     } else {
    //         ResolverError::TraitNotFound(name.toString(), name.location()).report(self.ctx);
    //     }
    // }

    pub fn resolveName(&self, name: &Identifier) -> QualifiedName {
        if let Some(qn) = self.tryResolverName(name) {
            return qn;
        }
        ResolverError::UnknownName(name.toString(), name.location()).report(self.ctx);
    }

    pub fn isGlobal(&self, name: &QualifiedName) -> bool {
        self.globals.contains(name)
    }

    pub fn resolveTypeName(&self, name: &Identifier) -> QualifiedName {
        // println!("local names:");
        // for (k, v) in &self.localNames.names {
        //     println!("  {}: {:?}", k, v);
        // }
        // println!("imported names:");
        // for (k, v) in &self.importedNames.names {
        //     println!("  {}: {:?}", k, v);
        // }
        // println!("implicitly imported names:");
        // for (k, v) in &self.implicitlyImportedNames.names {
        //     println!("  {}: {:?}", k, v);
        // }
        if let Some(names) = self.localNames.names.get(&name.name()) {
            if let Some(value) = self.resolveTypeNames(name, names) {
                return value;
            }
        }
        if let Some(names) = self.importedNames.names.get(&name.name()) {
            if let Some(value) = self.resolveTypeNames(name, names) {
                return value;
            }
        }
        if let Some(names) = self.implicitlyImportedNames.names.get(&name.name()) {
            if let Some(value) = self.resolveTypeNames(name, names) {
                return value;
            }
        }
        ResolverError::UnknownTypeName(name.toString(), name.location()).report(self.ctx);
    }

    fn resolveTypeNames(&self, name: &Identifier, names: &BTreeSet<QualifiedName>) -> Option<QualifiedName> {
        let mut typeNames = Vec::new();
        for name in names {
            if !self.variants.contains(name) {
                typeNames.push(name.clone());
            }
        }
        if typeNames.len() > 1 {
            ResolverError::Ambiguous(name.toString(), name.location()).report(self.ctx);
        }
        if typeNames.len() > 0 {
            return Some(typeNames.first().unwrap().clone());
        }
        None
    }

    pub fn tryResolverName(&self, name: &Identifier) -> Option<QualifiedName> {
        if let Some(names) = self.localNames.names.get(&name.name()) {
            if names.len() > 1 {
                ResolverError::Ambiguous(name.toString(), name.location()).report(self.ctx);
            }
            return Some(names.first().unwrap().clone());
        }
        if let Some(names) = self.importedNames.names.get(&name.name()) {
            if names.len() > 1 {
                ResolverError::Ambiguous(name.toString(), name.location()).report(self.ctx);
            }
            return Some(names.first().unwrap().clone());
        }
        if let Some(names) = self.implicitlyImportedNames.names.get(&name.name()) {
            if names.len() > 1 {
                ResolverError::Ambiguous(name.toString(), name.location()).report(self.ctx);
            }
            return Some(names.first().unwrap().clone());
        }
        None
    }
}
