use super::ModuleResolver::ModuleResolver;
use crate::siko::hir::ConstraintContext::ConstraintContext;
use crate::siko::hir::Type::{Type as IrType, TypeVar};
use crate::siko::syntax::Identifier::Identifier;
use crate::siko::syntax::Type::{Type, TypeParameterDeclaration};
use std::collections::BTreeSet;

pub struct TypeResolver<'a> {
    moduleResolver: &'a ModuleResolver<'a>,
    typeParameters: BTreeSet<IrType>,
}

impl<'a> TypeResolver<'a> {
    pub fn new(moduleResolver: &'a ModuleResolver, constraintContext: &'a ConstraintContext) -> TypeResolver<'a> {
        let mut r = TypeResolver {
            moduleResolver: moduleResolver,
            typeParameters: BTreeSet::new(),
        };
        for param in &constraintContext.typeParameters {
            r.addTypeParams(param.clone());
        }
        r
    }

    pub fn addTypeParams(&mut self, typeParam: IrType) {
        self.typeParameters.insert(typeParam);
    }

    pub fn resolveType(&self, ty: &Type) -> IrType {
        match ty {
            Type::Named(name, args) => {
                let var = IrType::Var(TypeVar::Named(name.toString()));
                if self.typeParameters.contains(&var) {
                    var
                } else {
                    let mut irArgs = Vec::new();
                    for arg in args {
                        irArgs.push(self.resolveType(arg));
                    }
                    let name = self.moduleResolver.resolverName(&name);
                    IrType::Named(name, irArgs, None)
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
            Type::Reference(ty) => IrType::Reference(Box::new(self.resolveType(ty)), None),
            Type::SelfType => IrType::SelfType,
        }
    }

    pub fn createDataType(&self, name: &Identifier, typeParams: &Option<TypeParameterDeclaration>) -> IrType {
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
        IrType::Named(self.moduleResolver.resolverName(name), args, None)
    }
}
