use super::ModuleResolver::ModuleResolver;
use crate::siko::ir::ConstraintContext::ConstraintContext;
use crate::siko::ir::Type::{Type as IrType, TypeVar};
use crate::siko::syntax::Identifier::Identifier;
use crate::siko::syntax::Type::{Type, TypeParameterDeclaration};
use std::collections::BTreeSet;

pub struct TypeResolver<'a> {
    moduleResolver: &'a ModuleResolver,
    typeParameters: BTreeSet<String>,
}

impl<'a> TypeResolver<'a> {
    pub fn new(
        moduleResolver: &'a ModuleResolver,
        constraintContext: &'a ConstraintContext,
    ) -> TypeResolver<'a> {
        let mut r = TypeResolver {
            moduleResolver: moduleResolver,
            typeParameters: BTreeSet::new(),
        };
        for param in &constraintContext.typeParameters {
            r.addTypeParams(param.clone());
        }
        r
    }

    pub fn addTypeParams(&mut self, typeParam: String) {
        self.typeParameters.insert(typeParam);
    }

    pub fn resolveType(&self, ty: &Type) -> IrType {
        match ty {
            Type::Named(name, args) => {
                if self.typeParameters.contains(&name.name) {
                    IrType::Var(TypeVar::Named(name.toString()))
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
            Type::Reference(ty) => IrType::Reference(Box::new(self.resolveType(ty))),
            Type::SelfType => IrType::SelfType,
        }
    }

    pub fn createDataType(
        &self,
        name: &Identifier,
        typeParams: &Option<TypeParameterDeclaration>,
    ) -> IrType {
        let args = match &typeParams {
            Some(typeParams) => {
                let mut args = Vec::new();
                for param in &typeParams.params {
                    let arg = IrType::Var(TypeVar::Named(param.name.name.clone()));
                    args.push(arg);
                }
                args
            }
            None => Vec::new(),
        };
        IrType::Named(self.moduleResolver.resolverName(name), args)
    }
}
