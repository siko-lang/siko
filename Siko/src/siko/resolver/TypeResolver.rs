use super::ModuleResolver::ModuleResolver;
use crate::siko::ir::Type::Type as IrType;
use crate::siko::syntax::Type::{Type, TypeParameterDeclaration};
pub struct TypeResolver<'a> {
    moduleResolver: &'a ModuleResolver,
    typeParameters: Vec<String>,
}

impl<'a> TypeResolver<'a> {
    pub fn new(moduleResolver: &'a ModuleResolver) -> TypeResolver<'a> {
        TypeResolver {
            moduleResolver: moduleResolver,
            typeParameters: Vec::new(),
        }
    }

    pub fn processTypeParameterDeclaration(&mut self, decl: &TypeParameterDeclaration) {
        for param in &decl.params {
            self.typeParameters.push(param.name.toString());
        }
    }

    pub fn resolveType(&self, ty: &Type) -> IrType {
        match ty {
            Type::Named(name, args) => {
                if self.typeParameters.contains(&name.name) {
                    IrType::Var(name.toString())
                } else {
                    let mut irArgs = Vec::new();
                    for arg in args {
                        irArgs.push(self.resolveType(arg));
                    }
                    let name = self.moduleResolver.resolverName(&name);
                    IrType::Named(name, irArgs)
                }
            }
            Type::Tuple(args) => {
                let mut irArgs = Vec::new();
                for arg in args {
                    irArgs.push(self.resolveType(arg));
                }
                IrType::Tuple(irArgs)
            }
            Type::Function(args, result) => {
                let mut irArgs = Vec::new();
                for arg in args {
                    irArgs.push(self.resolveType(arg));
                }
                IrType::Function(irArgs, Box::new(self.resolveType(result)))
            }
            Type::SelfType => IrType::SelfType,
        }
    }
}
