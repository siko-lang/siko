use crate::siko::qualifiedname::QualifiedName;
use crate::siko::syntax::Identifier::Identifier;

use crate::siko::util::error;

use super::Resolver::Names;

pub struct ModuleResolver {
    pub localNames: Names,
    pub importedNames: Names,
}

impl ModuleResolver {
    pub fn resolverName(&self, name: &Identifier) -> QualifiedName {
        if let Some(names) = self.localNames.names.get(&name.name) {
            if names.len() > 1 {
                error(format!("Ambiguous name {}", name.name));
            }
            return names[0].clone();
        }
        if let Some(names) = self.importedNames.names.get(&name.name) {
            if names.len() > 1 {
                error(format!("Ambiguous name {}", name.name));
            }
            return names[0].clone();
        }
        println!("Local names {:?}", self.localNames);
        error(format!("Unknown name {}", name.name));
    }
}
