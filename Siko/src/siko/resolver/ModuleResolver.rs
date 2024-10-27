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
}

impl<'a> ModuleResolver<'a> {
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
