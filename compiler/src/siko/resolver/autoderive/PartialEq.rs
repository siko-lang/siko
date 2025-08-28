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

pub fn derivePartialEqForEnum(enumDef: &Enum) -> Instance {
    let traitName = Identifier::new("Std.Cmp.PartialEq".to_string(), enumDef.name.location());
    let instanceName = Identifier::new(format!("PartialEq_{}", enumDef.name.name()), enumDef.name.location());
    let typeArgs = match enumDef.typeParams {
        Some(ref tp) => tp.params.iter().map(|p| Type::Named(p.clone(), Vec::new())).collect(),
        None => Vec::new(),
    };
    let mut constraints = Vec::new();
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
    let eqFn = getPartialEqFn(enumDef, &enumTy);
    let types = vec![enumTy];
    let instance = Instance {
        public: true,
        name: Some(instanceName),
        typeParams: typeParams,
        traitName: traitName,
        types: types,
        associatedTypes: Vec::new(),
        methods: vec![eqFn],
        location: enumDef.name.location(),
    };
    //crate::siko::syntax::Format::format_any(&instance);
    instance
}

fn getPartialEqFn(enumDef: &Enum, enumTy: &Type) -> Function {
    let boolTy = Type::Named(
        Identifier::new("Bool.Bool".to_string(), enumDef.name.location()),
        Vec::new(),
    );
    let fnName = Identifier::new("eq".to_string(), enumDef.name.location());
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
            withName("Bool.Bool.True", enumDef.name.location())
        } else {
            generateMatches(itemsARefs, itemsBRefs, enumDef.name.location())
        };
        cases.push(Branch {
            pattern: tuplePattern,
            body: withBlock(branchBody),
        });
    }
    cases.push(Branch {
        pattern: Pattern {
            pattern: SimplePattern::Wildcard,
            location: enumDef.name.location(),
        },
        body: withBlock(withName("Bool.Bool.False", enumDef.name.location())),
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
        result: boolTy,
        body: Some(body),
        externKind: None,
    }
}

fn generateMatches(itemARefs: Vec<Expr>, itemBRefs: Vec<Expr>, location: Location) -> Expr {
    if itemARefs.is_empty() {
        return withName("Bool.Bool.True", location.clone());
    }
    let firstA = itemARefs[0].clone();
    let firstB = itemBRefs[0].clone();
    let eqCall = Expr {
        expr: SimpleExpr::Call(
            Box::new(withName("Std.Cmp.PartialEq.eq", location.clone())),
            vec![firstA, firstB],
        ),
        location: location.clone(),
    };
    if itemARefs.len() == 1 {
        return eqCall;
    }
    let restA = itemARefs[1..].to_vec();
    let restB = itemBRefs[1..].to_vec();
    let restMatch = generateMatches(restA, restB, location.clone());
    let cases = vec![
        Branch {
            pattern: Pattern {
                pattern: SimplePattern::Named(Identifier::new("Bool.Bool.True".to_string(), location.clone()), vec![]),
                location: location.clone(),
            },
            body: withBlock(restMatch),
        },
        Branch {
            pattern: Pattern {
                pattern: SimplePattern::Wildcard,
                location: location.clone(),
            },
            body: withName("Bool.Bool.False", location.clone()),
        },
    ];
    Expr {
        expr: SimpleExpr::Match(Box::new(eqCall), cases),
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
