use std::collections::{BTreeMap, BTreeSet};

use crate::siko::hir::BodyBuilder::BodyBuilder;
use crate::siko::hir::ConstraintContext::ConstraintContext;
use crate::siko::hir::Data::{Enum, Struct};
use crate::siko::hir::Function::{
    Attributes as IrAttributes, ExternKind, Function as IrFunction, FunctionKind, ParamInfo, Parameter as IrParameter,
    ResultKind as IrResultKind,
};
use crate::siko::hir::Implicit::Implicit;
use crate::siko::hir::Safety::Safety as IrSafety;
use crate::siko::hir::Type::{Type as IrType, TypeVar};
use crate::siko::hir::Variable::Variable;
use crate::siko::hir::Variable::VariableName;
use crate::siko::location::Report::ReportContext;
use crate::siko::qualifiedname::{build, QualifiedName};
use crate::siko::syntax::Attributes::{Attributes, Safety};
use crate::siko::syntax::Function::{Function, FunctionExternKind, Parameter, ResultKind};
use crate::siko::syntax::Identifier::Identifier;
use crate::siko::syntax::Statement::{Block, Statement, StatementKind};
use crate::siko::syntax::Type::TypeParameterDeclaration;
use crate::siko::util::error;
use crate::siko::util::Runner::Runner;

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
    IrType::Named(build(&moduleResolver.name, &name.toString()), args)
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
        runner: Runner,
    ) -> (IrFunction, Vec<IrFunction>) {
        //println!("Resolving function: {}", name);

        let mut defaultArgFns = Vec::new();
        let mut params = Vec::new();
        let bodyBuilder = BodyBuilder::new();
        let mut env = Environment::new(bodyBuilder.getVariableAllocator());
        for (_, param) in f.params.iter().enumerate() {
            let irParam = match param {
                Parameter::Named(id, ty, mutable, defaultArgExpr) => {
                    if let Some(defaultArgExpr) = defaultArgExpr {
                        let defaultArgFnName = QualifiedName::DefaultArgFn(Box::new(name.clone()), params.len() as u32);
                        let bodyBuilder = BodyBuilder::new();
                        let mut exprResolver = ExprResolver::new(
                            &defaultArgFnName,
                            bodyBuilder.clone(),
                            ctx,
                            self.moduleResolver,
                            &typeResolver,
                            emptyVariants,
                            structs,
                            variants,
                            enums,
                            implicits,
                            runner.clone(),
                        );
                        let block = Block {
                            location: defaultArgExpr.location.clone(),
                            statements: vec![Statement {
                                kind: StatementKind::Expr(defaultArgExpr.clone()),
                                hasSemicolon: false,
                            }],
                        };
                        let env = Environment::new(bodyBuilder.getVariableAllocator());
                        exprResolver.resolve(&block, &env);
                        let body = bodyBuilder.build();
                        let defaultArgFn = IrFunction::new(
                            defaultArgFnName.clone(),
                            Vec::new(),
                            IrResultKind::SingleReturn(typeResolver.resolveType(ty)),
                            Some(body),
                            ConstraintContext::new(),
                            FunctionKind::UserDefined(f.name.location()),
                            IrAttributes::new(),
                        );
                        defaultArgFns.push(defaultArgFn);
                    }
                    let var = Variable::newWithType(
                        VariableName::Arg(id.toString()),
                        id.location(),
                        typeResolver.resolveType(ty),
                    );
                    env.addArg(var, *mutable);
                    let mut info = ParamInfo::new();
                    info.mutable = *mutable;
                    info.hasDefault = defaultArgExpr.is_some();
                    IrParameter::Named(id.toString(), typeResolver.resolveType(ty), info)
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
            ResultKind::SingleReturn(ty) => IrResultKind::SingleReturn(typeResolver.resolveType(&ty)),
            ResultKind::Coroutine(coroutineTy) => IrResultKind::Coroutine(typeResolver.resolveType(&coroutineTy)),
        };
        //println!("Function params: {:?}", params);
        //crate::siko::syntax::Format::format_any(f);
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
                runner,
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
                None => FunctionKind::UserDefined(f.name.location()),
            },
            convertFunctionAttributes(&f.attributes),
        );
        (irFunction, defaultArgFns)
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
    match attributes.safety {
        Safety::Safe => hirAttributes.safety = IrSafety::Safe,
        Safety::Unsafe => hirAttributes.safety = IrSafety::Unsafe,
        Safety::Regular => hirAttributes.safety = IrSafety::Regular,
    }
    hirAttributes
}
