use std::vec;

use crate::siko::{
    resolver::autoderive::Util::{withBlock, withName},
    syntax::{
        Data::{Enum, Struct},
        Expr::{Branch, Expr, SimpleExpr},
        Function::{Function, Parameter},
        Identifier::Identifier,
        Pattern::{Pattern, SimplePattern},
        Statement::{Block, Statement, StatementKind},
        Trait::Instance,
        Type::{Constraint, ConstraintArgument, Type, TypeParameterDeclaration},
    },
};

pub fn deriveCloneForEnum(enumDef: &Enum) -> Instance {
    let traitName = Identifier::new("Std.Ops.Basic.Clone".to_string(), enumDef.name.location());
    let instanceName = Identifier::new(format!("Clone_{}", enumDef.name.name()), enumDef.name.location());
    let typeArgs = match enumDef.typeParams {
        Some(ref tp) => tp.params.iter().map(|p| Type::Named(p.clone(), Vec::new())).collect(),
        None => Vec::new(),
    };
    let mut constraints = Vec::new();

    // Add Clone constraints for type parameters only
    for arg in typeArgs.iter() {
        constraints.push(Constraint {
            name: traitName.clone(),
            args: vec![ConstraintArgument::Type(arg.clone())],
        });
    }

    let typeParams = if typeArgs.is_empty() {
        None
    } else {
        let decl = TypeParameterDeclaration {
            params: enumDef.typeParams.as_ref().unwrap().params.clone(),
            constraints: constraints,
        };
        Some(decl)
    };
    let enumTy = Type::Named(enumDef.name.clone(), typeArgs);
    let cloneFn = getCloneFnForEnum(enumDef, &enumTy);
    let types = vec![enumTy];
    let instance = Instance {
        public: true,
        name: Some(instanceName),
        typeParams: typeParams,
        traitName: traitName,
        types: types,
        associatedTypes: Vec::new(),
        methods: vec![cloneFn],
        location: enumDef.name.location(),
    };
    //crate::siko::syntax::Format::format_any(&instance);
    instance
}

fn getCloneFnForEnum(enumDef: &Enum, enumTy: &Type) -> Function {
    let fnName = Identifier::new("clone".to_string(), enumDef.name.location());
    let mut params = Vec::new();
    params.push(Parameter::RefSelfParam);

    let selfRef = Expr {
        expr: SimpleExpr::SelfValue,
        location: enumDef.name.location(),
    };

    let mut cases = Vec::new();

    // Generate cases for each variant
    for variant in &enumDef.variants {
        let mut itemBinds = Vec::new();
        let mut clonedItems = Vec::new();

        for i in 0..variant.items.len() {
            let name = Identifier::new(format!("item_{}", i), enumDef.name.location());
            let bind = Pattern {
                pattern: SimplePattern::Bind(name.clone(), false),
                location: enumDef.name.location(),
            };
            itemBinds.push(bind);

            // Clone each field
            let clonedItem = Expr {
                expr: SimpleExpr::Call(
                    Box::new(withName("Std.Ops.Basic.Clone.clone", enumDef.name.location())),
                    vec![Expr {
                        expr: SimpleExpr::Value(name),
                        location: enumDef.name.location(),
                    }],
                ),
                location: enumDef.name.location(),
            };
            clonedItems.push(clonedItem);
        }

        let variantName = Identifier::new(variant.name.name(), enumDef.name.location());
        let variantPattern = Pattern {
            pattern: SimplePattern::Named(variantName.clone(), itemBinds),
            location: enumDef.name.location(),
        };

        // Construct the cloned variant
        let clonedVariant = if variant.items.is_empty() {
            // No arguments, just the variant name
            withName(
                &format!("{}.{}", enumDef.name.name(), variant.name.name()),
                enumDef.name.location(),
            )
        } else {
            // Call the variant constructor with cloned arguments
            Expr {
                expr: SimpleExpr::Call(
                    Box::new(withName(
                        &format!("{}.{}", enumDef.name.name(), variant.name.name()),
                        enumDef.name.location(),
                    )),
                    clonedItems,
                ),
                location: enumDef.name.location(),
            }
        };

        cases.push(Branch {
            pattern: variantPattern,
            body: withBlock(clonedVariant),
        });
    }

    let matchExpr = Expr {
        expr: SimpleExpr::Match(Box::new(selfRef), cases),
        location: enumDef.name.location(),
    };

    let body = Block {
        statements: vec![Statement {
            kind: StatementKind::Expr(matchExpr),
            hasSemicolon: false,
        }],
        location: enumDef.name.location(),
    };
    Function {
        public: true,
        name: fnName,
        params: params,
        typeParams: None,
        result: enumTy.clone(),
        body: Some(body),
        externKind: None,
    }
}

pub fn deriveCloneForStruct(structDef: &Struct) -> Instance {
    let traitName = Identifier::new("Std.Ops.Basic.Clone".to_string(), structDef.name.location());
    let instanceName = Identifier::new(format!("Clone_{}", structDef.name.name()), structDef.name.location());
    let typeArgs = match structDef.typeParams {
        Some(ref tp) => tp.params.iter().map(|p| Type::Named(p.clone(), Vec::new())).collect(),
        None => Vec::new(),
    };
    let mut constraints = Vec::new();

    // Add Clone constraints for type parameters only
    for arg in typeArgs.iter() {
        constraints.push(Constraint {
            name: traitName.clone(),
            args: vec![ConstraintArgument::Type(arg.clone())],
        });
    }

    let typeParams = if typeArgs.is_empty() {
        None
    } else {
        let decl = TypeParameterDeclaration {
            params: structDef.typeParams.as_ref().unwrap().params.clone(),
            constraints: constraints,
        };
        Some(decl)
    };
    let structTy = Type::Named(structDef.name.clone(), typeArgs);
    let cloneFn = getCloneFnForStruct(structDef, &structTy);
    let types = vec![structTy];
    let instance = Instance {
        public: true,
        name: Some(instanceName),
        typeParams: typeParams,
        traitName: traitName,
        types: types,
        associatedTypes: Vec::new(),
        methods: vec![cloneFn],
        location: structDef.name.location(),
    };
    //crate::siko::syntax::Format::format_any(&instance);
    instance
}

fn getCloneFnForStruct(structDef: &Struct, structTy: &Type) -> Function {
    let fnName = Identifier::new("clone".to_string(), structDef.name.location());
    let mut params = Vec::new();
    params.push(Parameter::RefSelfParam);

    // Clone each field of the struct
    let mut clonedFields = Vec::new();
    for field in &structDef.fields {
        let fieldAccess = Expr {
            expr: SimpleExpr::FieldAccess(
                Box::new(Expr {
                    expr: SimpleExpr::SelfValue,
                    location: structDef.name.location(),
                }),
                field.name.clone(),
            ),
            location: structDef.name.location(),
        };

        let clonedField = Expr {
            expr: SimpleExpr::Call(
                Box::new(withName("Std.Ops.Basic.Clone.clone", structDef.name.location())),
                vec![fieldAccess],
            ),
            location: structDef.name.location(),
        };
        clonedFields.push(clonedField);
    }

    // Create the struct constructor call
    let structConstructor = if structDef.fields.is_empty() {
        // Empty struct, just the struct name
        withName(&structDef.name.name(), structDef.name.location())
    } else {
        // Struct with fields
        Expr {
            expr: SimpleExpr::Call(
                Box::new(withName(&structDef.name.name(), structDef.name.location())),
                clonedFields,
            ),
            location: structDef.name.location(),
        }
    };

    let body = Block {
        statements: vec![Statement {
            kind: StatementKind::Expr(structConstructor),
            hasSemicolon: false,
        }],
        location: structDef.name.location(),
    };

    Function {
        public: true,
        name: fnName,
        params: params,
        typeParams: None,
        result: structTy.clone(),
        body: Some(body),
        externKind: None,
    }
}
