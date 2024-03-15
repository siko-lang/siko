use std::collections::{BTreeMap, BTreeSet};

use crate::siko::ir::Data::Enum;
use crate::siko::ir::Function::{Function as IrFunction, Parameter as IrParameter};
use crate::siko::ir::Type::{Type as IrType, TypeVar};
use crate::siko::qualifiedname::QualifiedName;
use crate::siko::syntax::Function::{Function, Parameter};
use crate::siko::syntax::Identifier::Identifier;
use crate::siko::syntax::Type::TypeParameterDeclaration;
use crate::siko::util::error;

use super::Environment::Environment;
use super::ExprResolver::ExprResolver;
use super::ModuleResolver::ModuleResolver;
use super::TypeResolver::TypeResolver;
pub struct FunctionResolver<'a> {
    moduleResolver: &'a ModuleResolver,
    typeParams: Option<&'a TypeParameterDeclaration>,
    owner: Option<Identifier>,
}

impl<'a> FunctionResolver<'a> {
    pub fn new(
        moduleResolver: &'a ModuleResolver,
        typeParams: Option<&'a TypeParameterDeclaration>,
        owner: Option<Identifier>,
    ) -> FunctionResolver<'a> {
        FunctionResolver {
            moduleResolver: moduleResolver,
            typeParams: typeParams,
            owner: owner,
        }
    }

    pub fn resolve(
        &self,
        f: &Function,
        emptyVariants: &BTreeSet<QualifiedName>,
        variants: &BTreeMap<QualifiedName, QualifiedName>,
        enums: &BTreeMap<QualifiedName, Enum>,
    ) -> IrFunction {
        let mut typeResolver = TypeResolver::new(self.moduleResolver, &f.typeParams);
        typeResolver.addTypeParams(self.typeParams);
        let mut params = Vec::new();
        let mut env = Environment::new();
        for param in &f.params {
            let irParam = match param {
                Parameter::Named(id, ty, mutable) => {
                    env.addArg(id.toString());
                    IrParameter::Named(id.toString(), typeResolver.resolveType(ty), *mutable)
                }
                Parameter::SelfParam(mutable) => match &self.owner {
                    Some(owner) => {
                        let args = match &self.typeParams {
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
                        let ownerType =
                            IrType::Named(self.moduleResolver.resolverName(owner), args);
                        IrParameter::SelfParam(*mutable, ownerType)
                    }
                    None => error(format!("No owner for self type!")),
                },
            };

            params.push(irParam);
        }
        let result = if let Some(ty) = &f.result {
            typeResolver.resolveType(ty)
        } else {
            IrType::Tuple(Vec::new())
        };

        let body = if let Some(body) = &f.body {
            let mut exprResolver =
                ExprResolver::new(self.moduleResolver, emptyVariants, variants, enums);
            exprResolver.resolve(body, &env);
            Some(exprResolver.body())
        } else {
            None
        };
        let irFunction = IrFunction::new(
            self.moduleResolver.resolverName(&f.name),
            params,
            result,
            body,
        );
        irFunction
    }
}
