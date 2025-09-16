use std::collections::{BTreeMap, BTreeSet};

use crate::siko::hir::BodyBuilder::BodyBuilder;
use crate::siko::hir::ConstraintContext::ConstraintContext;
use crate::siko::hir::Data::{Enum, Struct};
use crate::siko::hir::Function::{
    Attributes as IrAttributes, ExternKind, Function as IrFunction, FunctionKind, Parameter as IrParameter,
};
use crate::siko::hir::Implicit::Implicit;
use crate::siko::hir::Type::{Type as IrType, TypeVar};
use crate::siko::hir::Variable::Variable;
use crate::siko::hir::Variable::VariableName;
use crate::siko::location::Report::ReportContext;
use crate::siko::qualifiedname::QualifiedName;
use crate::siko::syntax::Function::{Attributes, Function, FunctionExternKind, Parameter, ResultKind};
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

pub fn createSelfType(
    name: &Identifier,
    typeParams: Option<&TypeParameterDeclaration>,
    moduleResolver: &ModuleResolver,
) -> IrType {
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
    IrType::Named(
        QualifiedName::Module(moduleResolver.name.clone()).add(name.toString()),
        args,
    )
}

impl<'a> FunctionResolver<'a> {
    pub fn new(
        moduleResolver: &'a ModuleResolver,
        constraintContext: ConstraintContext,
        owner: Option<IrType>,
    ) -> FunctionResolver<'a> {
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
        structs: &BTreeMap<QualifiedName, Struct>,
        variants: &BTreeMap<QualifiedName, QualifiedName>,
        enums: &BTreeMap<QualifiedName, Enum>,
        implicits: &BTreeMap<QualifiedName, Implicit>,
        name: QualifiedName,
        typeResolver: &TypeResolver,
    ) -> IrFunction {
        //println!("Resolving function: {}", name);

        let mut params = Vec::new();
        let bodyBuilder = BodyBuilder::new();
        let mut env = Environment::new(bodyBuilder.getVariableAllocator());
        for (_, param) in f.params.iter().enumerate() {
            let irParam = match param {
                Parameter::Named(id, ty, mutable) => {
                    let var = Variable::newWithType(
                        VariableName::Arg(id.toString()),
                        id.location(),
                        typeResolver.resolveType(ty),
                    );
                    env.addArg(var, *mutable);
                    IrParameter::Named(id.toString(), typeResolver.resolveType(ty), *mutable)
                }
                Parameter::SelfParam => match &self.owner {
                    Some(owner) => {
                        let var =
                            Variable::newWithType(VariableName::Arg(format!("self")), f.name.location(), owner.clone());
                        env.addArg(var, false);
                        IrParameter::SelfParam(false, owner.clone())
                    }
                    None => error(format!("No owner for self type!")),
                },
                Parameter::MutSelfParam => match &self.owner {
                    Some(owner) => {
                        let var =
                            Variable::newWithType(VariableName::Arg(format!("self")), f.name.location(), owner.clone());
                        env.addArg(var, true);
                        IrParameter::SelfParam(true, owner.clone())
                    }
                    None => error(format!("No owner for self type!")),
                },
                Parameter::RefSelfParam => match &self.owner {
                    Some(owner) => {
                        let var =
                            Variable::newWithType(VariableName::Arg(format!("self")), f.name.location(), owner.asRef());
                        env.addArg(var, false);
                        IrParameter::SelfParam(false, owner.asRef())
                    }
                    None => error(format!("No owner for self type!")),
                },
            };

            params.push(irParam);
        }
        let result = match &f.result {
            ResultKind::SingleReturn(ty) => typeResolver.resolveType(&ty),
            ResultKind::Generator(yieldTy, retTy) => {
                let yieldIrTy = typeResolver.resolveType(&yieldTy);
                let retIrTy = typeResolver.resolveType(&retTy);
                IrType::Generator(Box::new(yieldIrTy), Box::new(retIrTy))
            }
        };
        //println!("Function params: {:?}", params);

        let body = if let Some(body) = &f.body {
            let mut exprResolver = ExprResolver::new(
                &name,
                bodyBuilder,
                ctx,
                self.moduleResolver,
                &typeResolver,
                emptyVariants,
                structs,
                variants,
                enums,
                implicits,
            );
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
            match f.externKind {
                Some(FunctionExternKind::C(ref header)) => FunctionKind::Extern(ExternKind::C(header.clone())),
                Some(FunctionExternKind::Builtin) => FunctionKind::Extern(ExternKind::Builtin),
                None => FunctionKind::UserDefined,
            },
            convertFunctionAttributes(&f.attributes),
        );
        irFunction
    }
}

pub fn convertFunctionAttributes(attributes: &Attributes) -> IrAttributes {
    let mut hirAttributes = IrAttributes::new();
    if attributes.testEntry {
        hirAttributes.testEntry = true;
    }
    if attributes.inline {
        hirAttributes.inline = true;
    }
    hirAttributes
}
