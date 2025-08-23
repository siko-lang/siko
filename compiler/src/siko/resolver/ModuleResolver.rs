use std::collections::BTreeSet;

use crate::siko::hir::Program::Program;
use crate::siko::hir::Trait::{Protocol, Trait};
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
    pub importedModules: Vec<String>,
    pub variants: BTreeSet<QualifiedName>,
}

impl<'a> ModuleResolver<'a> {
    pub fn lookupTrait(&self, name: &Identifier, program: &Program) -> Trait {
        let qn = &self.resolveName(name);
        if let Some(traitDef) = program.getTrait(qn) {
            traitDef
        } else {
            ResolverError::TraitNotFound(name.toString(), name.location()).report(self.ctx);
        }
    }

    pub fn lookupProtocol(&self, name: &Identifier, program: &Program) -> Protocol {
        let qn = &self.resolveName(name);
        if let Some(protocolDef) = program.getProtocol(qn) {
            protocolDef
        } else {
            ResolverError::ProtocolNotFound(name.toString(), name.location()).report(self.ctx);
        }
    }

    pub fn resolveName(&self, name: &Identifier) -> QualifiedName {
        if let Some(qn) = self.tryResolverName(name) {
            return qn;
        }
        ResolverError::UnknownName(name.toString(), name.location()).report(self.ctx);
    }

    pub fn resolveTypeName(&self, name: &Identifier) -> QualifiedName {
        if let Some(names) = self.localNames.names.get(&name.name()) {
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
                return typeNames.first().unwrap().clone();
            }
        }
        if let Some(names) = self.importedNames.names.get(&name.name()) {
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
                return typeNames.first().unwrap().clone();
            }
        }
        ResolverError::UnknownName(name.toString(), name.location()).report(self.ctx);
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
        None
    }
}
