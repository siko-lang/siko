use std::vec;

use crate::siko::{
    location::Location::Location,
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

pub fn deriveOrdForEnum(enumDef: &Enum) -> Instance {
    let traitName = Identifier::new("Std.Cmp.Ord".to_string(), enumDef.name.location());
    let instanceName = Identifier::new(format!("Ord_{}", enumDef.name.name()), enumDef.name.location());
    let typeArgs = match enumDef.typeParams {
        Some(ref tp) => tp.params.iter().map(|p| Type::Named(p.clone(), Vec::new())).collect(),
        None => Vec::new(),
    };
    let mut constraints = Vec::new();

    // Add Ord constraints for type parameters only
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
    let cmpFn = getCmpFn(enumDef, &enumTy);
    let types = vec![enumTy];
    let instance = Instance {
        public: true,
        name: Some(instanceName),
        typeParams: typeParams,
        traitName: traitName,
        types: types,
        associatedTypes: Vec::new(),
        methods: vec![cmpFn],
        location: enumDef.name.location(),
    };
    //crate::siko::syntax::Format::format_any(&instance);
    instance
}

fn getCmpFn(enumDef: &Enum, enumTy: &Type) -> Function {
    let orderingTy = Type::Named(
        Identifier::new("Ordering.Ordering".to_string(), enumDef.name.location()),
        Vec::new(),
    );
    let fnName = Identifier::new("cmp".to_string(), enumDef.name.location());
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
            withName("Ordering.Ordering.Equal", enumDef.name.location())
        } else {
            generateFieldComparison(itemsARefs, itemsBRefs, enumDef.name.location())
        };
        cases.push(Branch {
            pattern: tuplePattern,
            body: withBlock(branchBody),
        });
    }

    // Add wildcard case for different variants - use discriminator comparison
    let discriminatorComparison = Expr {
        expr: SimpleExpr::Call(
            Box::new(withName("Std.Cmp.Ord.cmp", enumDef.name.location())),
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
        result: orderingTy,
        body: Some(body),
        externKind: None,
    }
}

fn generateFieldComparison(itemARefs: Vec<Expr>, itemBRefs: Vec<Expr>, location: Location) -> Expr {
    if itemARefs.is_empty() {
        return withName("Ordering.Ordering.Equal", location.clone());
    }
    let firstA = itemARefs[0].clone();
    let firstB = itemBRefs[0].clone();
    let cmpCall = Expr {
        expr: SimpleExpr::Call(
            Box::new(withName("Std.Cmp.Ord.cmp", location.clone())),
            vec![firstA, firstB],
        ),
        location: location.clone(),
    };
    if itemARefs.len() == 1 {
        return cmpCall;
    }
    let restA = itemARefs[1..].to_vec();
    let restB = itemBRefs[1..].to_vec();
    let restComparison = generateFieldComparison(restA, restB, location.clone());

    let equalPattern = Pattern {
        pattern: SimplePattern::Named(
            Identifier::new("Ordering.Ordering.Equal".to_string(), location.clone()),
            vec![],
        ),
        location: location.clone(),
    };

    let cases = vec![
        Branch {
            pattern: equalPattern,
            body: withBlock(restComparison),
        },
        Branch {
            pattern: Pattern {
                pattern: SimplePattern::Bind(Identifier::new("result".to_string(), location.clone()), false),
                location: location.clone(),
            },
            body: withBlock(Expr {
                expr: SimpleExpr::Value(Identifier::new("result".to_string(), location.clone())),
                location: location.clone(),
            }),
        },
    ];
    Expr {
        expr: SimpleExpr::Match(Box::new(cmpCall), cases),
        location: location.clone(),
    }
}

fn withBlock(e: Expr) -> Expr {
    let location = e.location.clone();
    Expr {
        expr: SimpleExpr::Block(Block {
            statements: vec![Statement {
                kind: StatementKind::Expr(e),
                hasSemicolon: false,
            }],
            location: location.clone(),
        }),
        location: location.clone(),
    }
}

fn withName(n: &str, location: Location) -> Expr {
    Expr {
        expr: SimpleExpr::Name(Identifier::new(n.to_string(), location.clone())),
        location: location.clone(),
    }
}
