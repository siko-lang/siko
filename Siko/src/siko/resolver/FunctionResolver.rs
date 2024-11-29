use std::collections::{BTreeMap, BTreeSet};

use crate::siko::hir::ConstraintContext::ConstraintContext;
use crate::siko::hir::Data::Enum;
use crate::siko::hir::Function::{Function as IrFunction, FunctionKind, Parameter as IrParameter, Variable};
use crate::siko::hir::Type::{Type as IrType, TypeVar};
use crate::siko::location::Report::ReportContext;
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
    moduleResolver: &'a ModuleResolver<'a>,
    constraintContext: ConstraintContext,
    owner: Option<IrType>,
}

pub fn createSelfType(name: &Identifier, typeParams: Option<&TypeParameterDeclaration>, moduleResolver: &ModuleResolver) -> IrType {
    let args = match &typeParams {
        Some(typeParams) => {
            let mut args = Vec::new();
            for param in &typeParams.params {
                let arg = IrType::Var(TypeVar::Named(param.name.clone()));
                args.push(arg);
            }
            args
        }
        None => Vec::new(),
    };
    IrType::Named(QualifiedName::Module(moduleResolver.name.clone()).add(name.toString()), args, None)
}

impl<'a> FunctionResolver<'a> {
    pub fn new(moduleResolver: &'a ModuleResolver, constraintContext: ConstraintContext, owner: Option<IrType>) -> FunctionResolver<'a> {
        FunctionResolver {
            moduleResolver: moduleResolver,
            constraintContext: constraintContext,
            owner: owner,
        }
    }

    pub fn resolve(
        &self,
        ctx: &ReportContext,
        f: &Function,
        emptyVariants: &BTreeSet<QualifiedName>,
        variants: &BTreeMap<QualifiedName, QualifiedName>,
        enums: &BTreeMap<QualifiedName, Enum>,
        name: QualifiedName,
        typeResolver: &TypeResolver,
    ) -> IrFunction {
        let mut params = Vec::new();
        let mut env = Environment::new();
        for (_, param) in f.params.iter().enumerate() {
            let irParam = match param {
                Parameter::Named(id, ty, mutable) => {
                    let var = Variable {
                        value: id.toString(),
                        location: id.location.clone(),
                        ty: Some(typeResolver.resolveType(ty)),
                        index: 0,
                    };
                    env.addArg(var);
                    IrParameter::Named(id.toString(), typeResolver.resolveType(ty), *mutable)
                }
                Parameter::SelfParam => match &self.owner {
                    Some(owner) => IrParameter::SelfParam(false, owner.clone()),
                    None => error(format!("No owner for self type!")),
                },
                Parameter::MutSelfParam => match &self.owner {
                    Some(owner) => IrParameter::SelfParam(true, owner.clone()),
                    None => error(format!("No owner for self type!")),
                },
                Parameter::RefSelfParam => match &self.owner {
                    Some(owner) => IrParameter::SelfParam(false, IrType::Reference(Box::new(owner.clone()), None)),
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
            let mut exprResolver = ExprResolver::new(ctx, self.moduleResolver, &typeResolver, emptyVariants, variants, enums);
            exprResolver.resolve(body, &env);
            Some(exprResolver.body())
        } else {
            None
        };
        let irFunction = IrFunction::new(
            name,
            params,
            result,
            body,
            self.constraintContext.clone(),
            if f.isExtern { FunctionKind::Extern } else { FunctionKind::UserDefined },
        );
        irFunction
    }
}
