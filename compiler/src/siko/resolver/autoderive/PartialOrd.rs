use std::vec;

use crate::siko::{
    resolver::autoderive::Util::{generatePartialOrdFieldComparison, withBlock, withName, withSome},
    syntax::{
        Data::Enum,
        Expr::{Branch, Expr, SimpleExpr},
        Function::{Function, Parameter},
        Identifier::Identifier,
        Pattern::{Pattern, SimplePattern},
        Statement::{Block, Statement, StatementKind},
        Trait::Instance,
        Type::{Constraint, ConstraintArgument, Type, TypeParameterDeclaration},
    },
};

pub fn derivePartialOrdForEnum(enumDef: &Enum) -> Instance {
    let traitName = Identifier::new("Std.Cmp.PartialOrd".to_string(), enumDef.name.location());
    let instanceName = Identifier::new(format!("PartialOrd_{}", enumDef.name.name()), enumDef.name.location());
    let typeArgs = match enumDef.typeParams {
        Some(ref tp) => tp.params.iter().map(|p| Type::Named(p.clone(), Vec::new())).collect(),
        None => Vec::new(),
    };
    let mut constraints = Vec::new();

    // Add PartialOrd constraints for type parameters only
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
    let partialCmpFn = getPartialCmpFn(enumDef, &enumTy);
    let types = vec![enumTy];
    let instance = Instance {
        public: true,
        name: Some(instanceName),
        typeParams: typeParams,
        traitName: traitName,
        types: types,
        associatedTypes: Vec::new(),
        methods: vec![partialCmpFn],
        location: enumDef.name.location(),
    };
    //crate::siko::syntax::Format::format_any(&instance);
    instance
}

fn getPartialCmpFn(enumDef: &Enum, enumTy: &Type) -> Function {
    let optionOrderingTy = Type::Named(
        Identifier::new("Option.Option".to_string(), enumDef.name.location()),
        vec![Type::Named(
            Identifier::new("Ordering.Ordering".to_string(), enumDef.name.location()),
            Vec::new(),
        )],
    );
    let fnName = Identifier::new("partialCmp".to_string(), enumDef.name.location());
    let mut params = Vec::new();
    params.push(Parameter::RefSelfParam);
    let otherName = Identifier::new("other".to_string(), enumDef.name.location());
    let selfRefType = Type::Reference(Box::new(enumTy.clone()));
    params.push(Parameter::Named(otherName, selfRefType, false));

    let otherRef = Expr {
        expr: SimpleExpr::Value(Identifier::new("other".to_string(), enumDef.name.location())),
        location: enumDef.name.location(),
    };
    let selfRef = Expr {
        expr: SimpleExpr::SelfValue,
        location: enumDef.name.location(),
    };
    let tupleExpr = Expr {
        expr: SimpleExpr::Tuple(vec![selfRef, otherRef]),
        location: enumDef.name.location(),
    };

    let mut cases = Vec::new();

    // Generate cases for same variant comparisons (field-by-field comparison)
    for variant in &enumDef.variants {
        let mut itemsABinds = Vec::new();
        let mut itemsBBinds = Vec::new();
        let mut itemsARefs = Vec::new();
        let mut itemsBRefs = Vec::new();
        for i in 0..variant.items.len() {
            let nameA = Identifier::new(format!("a_{}", i), enumDef.name.location());
            let bindA = Pattern {
                pattern: SimplePattern::Bind(nameA.clone(), false),
                location: enumDef.name.location(),
            };
            itemsABinds.push(bindA);
            let nameB = Identifier::new(format!("b_{}", i), enumDef.name.location());
            let bindB = Pattern {
                pattern: SimplePattern::Bind(nameB.clone(), false),
                location: enumDef.name.location(),
            };
            itemsBBinds.push(bindB);
            let refA = Expr {
                expr: SimpleExpr::Value(nameA),
                location: enumDef.name.location(),
            };
            itemsARefs.push(refA);
            let refB = Expr {
                expr: SimpleExpr::Value(nameB),
                location: enumDef.name.location(),
            };
            itemsBRefs.push(refB);
        }
        let variantName = Identifier::new(variant.name.name(), enumDef.name.location());
        let variantPatternA = Pattern {
            pattern: SimplePattern::Named(variantName.clone(), itemsABinds),
            location: enumDef.name.location(),
        };
        let variantPatternB = Pattern {
            pattern: SimplePattern::Named(variantName.clone(), itemsBBinds),
            location: enumDef.name.location(),
        };
        let tuplePattern = Pattern {
            pattern: SimplePattern::Tuple(vec![variantPatternA, variantPatternB]),
            location: enumDef.name.location(),
        };
        let branchBody = if variant.items.is_empty() {
            withSome(withName("Ordering.Ordering.Equal", enumDef.name.location()))
        } else {
            generatePartialOrdFieldComparison(itemsARefs, itemsBRefs, enumDef.name.location())
        };
        cases.push(Branch {
            pattern: tuplePattern,
            body: withBlock(branchBody),
        });
    }

    // Add wildcard case for different variants - use discriminator comparison
    let discriminatorComparison = Expr {
        expr: SimpleExpr::Call(
            Box::new(withName("Std.Cmp.PartialOrd.partialCmp", enumDef.name.location())),
            vec![
                Expr {
                    expr: SimpleExpr::Call(
                        Box::new(withName(
                            "Std.Ops.Basic.Discriminator.discriminator",
                            enumDef.name.location(),
                        )),
                        vec![Expr {
                            expr: SimpleExpr::SelfValue,
                            location: enumDef.name.location(),
                        }],
                    ),
                    location: enumDef.name.location(),
                },
                Expr {
                    expr: SimpleExpr::Call(
                        Box::new(withName(
                            "Std.Ops.Basic.Discriminator.discriminator",
                            enumDef.name.location(),
                        )),
                        vec![Expr {
                            expr: SimpleExpr::Value(Identifier::new("other".to_string(), enumDef.name.location())),
                            location: enumDef.name.location(),
                        }],
                    ),
                    location: enumDef.name.location(),
                },
            ],
        ),
        location: enumDef.name.location(),
    };

    cases.push(Branch {
        pattern: Pattern {
            pattern: SimplePattern::Wildcard,
            location: enumDef.name.location(),
        },
        body: withBlock(discriminatorComparison),
    });

    let matchExpr = Expr {
        expr: SimpleExpr::Match(Box::new(tupleExpr), cases),
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
        result: optionOrderingTy,
        body: Some(body),
        externKind: None,
    }
}
