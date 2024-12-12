use crate::siko::hir::Program::Program;
use crate::siko::hir::Trait::Trait;
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
}

impl<'a> ModuleResolver<'a> {
    pub fn lookupTrait(&self, name: &Identifier, program: &Program) -> Trait {
        let qn = &self.resolverName(name);
        if let Some(traitDef) = program.getTrait(qn) {
            traitDef
        } else {
            ResolverError::TraitNotFound(name.toString(), name.location.clone()).report(self.ctx);
        }
    }

    pub fn resolverName(&self, name: &Identifier) -> QualifiedName {
        if let Some(names) = self.localNames.names.get(&name.name) {
            if names.len() > 1 {
                ResolverError::Ambiguous(name.toString(), name.location.clone()).report(self.ctx);
            }
            return names.first().unwrap().clone();
        }
        if let Some(names) = self.importedNames.names.get(&name.name) {
            if names.len() > 1 {
                ResolverError::Ambiguous(name.toString(), name.location.clone()).report(self.ctx);
            }
            return names.first().unwrap().clone();
        }
        ResolverError::UnknownName(name.toString(), name.location.clone()).report(self.ctx);
    }
}
