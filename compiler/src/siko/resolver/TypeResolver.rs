use super::ModuleResolver::ModuleResolver;
use crate::siko::hir::Type::{Type as IrType, TypeVar};
use crate::siko::qualifiedname::QualifiedName;
use crate::siko::resolver::Util::SubstitutionChain;
use crate::siko::syntax::Type::{Type, TypeParameterDeclaration};
use std::collections::BTreeSet;

#[derive(Clone)]
pub struct TypeResolver<'a> {
    pub moduleResolver: &'a ModuleResolver<'a>,
    typeParameters: BTreeSet<IrType>,
    subChain: Option<&'a SubstitutionChain>,
}

impl<'a> TypeResolver<'a> {
    pub fn new(moduleResolver: &'a ModuleResolver, typeParameters: &'a Vec<IrType>) -> TypeResolver<'a> {
        let mut r = TypeResolver {
            moduleResolver: moduleResolver,
            typeParameters: BTreeSet::new(),
            subChain: None,
        };
        for param in typeParameters {
            r.addTypeParams(param.clone());
        }
        r
    }

    pub fn withSubChain(&self, subChain: &'a SubstitutionChain) -> Self {
        let mut new = self.clone();
        new.subChain = Some(subChain);
        new
    }

    pub fn addTypeParams(&mut self, typeParam: IrType) {
        self.typeParameters.insert(typeParam);
    }

    pub fn resolveType(&self, ty: &Type) -> IrType {
        match ty {
            Type::Named(name, args) => {
                let var = IrType::Var(TypeVar::Named(name.toString()));
                if let Some(subChain) = self.subChain {
                    let var2 = subChain.apply(var.clone());
                    if var != var2 {
                        return var2;
                    }
                }
                if self.typeParameters.contains(&var) {
                    var
                } else {
                    let mut irArgs = Vec::new();
                    for arg in args {
                        irArgs.push(self.resolveType(arg));
                    }
                    let name = self.moduleResolver.resolveTypeName(&name);
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
            Type::Reference(ty) => self.resolveType(ty).asRef(),
            Type::Ptr(ty) => IrType::Ptr(Box::new(self.resolveType(ty))),
            Type::SelfType => IrType::SelfType,
            Type::Never => IrType::Never(true),
            Type::NumericConstant(value) => IrType::NumericConstant(value.clone()),
            Type::Void => IrType::Void,
            Type::VoidPtr => IrType::VoidPtr,
        }
    }

    pub fn createDataType(&self, name: &QualifiedName, typeParams: &Option<TypeParameterDeclaration>) -> IrType {
        let args = match &typeParams {
            Some(typeParams) => {
                let mut args = Vec::new();
                for param in &typeParams.params {
                    let arg = IrType::Var(TypeVar::Named(param.name()));
                    args.push(arg);
                }
                args
            }
            None => Vec::new(),
        };
        IrType::Named(name.clone(), args)
    }
}
