use crate::siko::ir::Type::Type as IrType;
use crate::siko::qualifiedname::QualifiedName;
use crate::siko::syntax::Identifier::Identifier;
use crate::siko::syntax::Type::Type;
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
        error(format!("Unknown name {}", name.name));
    }

    pub fn resolveType(&self, ty: &Type) -> IrType {
        match ty {
            Type::Named(name, args) => {
                let mut irArgs = Vec::new();
                for arg in args {
                    irArgs.push(self.resolveType(arg));
                }
                let name = self.resolverName(&name);
                IrType::Named(name, irArgs)
            }
            Type::Tuple(_) => todo!(),
            Type::Function(_, _) => todo!(),
            Type::SelfType => todo!(),
        }
    }
}
