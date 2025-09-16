use crate::siko::{
    resolver::autoderive::Util::{generateMatches, withBlock, withName},
    syntax::{
        Data::{Enum, Struct},
        Expr::{Branch, Expr, SimpleExpr},
        Function::{Attributes, Function, Parameter, ResultKind},
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
    let eqFn = getPartialEqFnForEnum(enumDef, &enumTy);
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

fn getPartialEqFnForEnum(enumDef: &Enum, enumTy: &Type) -> Function {
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
        result: ResultKind::SingleReturn(boolTy),
        body: Some(body),
        externKind: None,
        attributes: Attributes::new(),
    }
}

pub fn derivePartialEqForStruct(structDef: &Struct) -> Instance {
    let traitName = Identifier::new("Std.Cmp.PartialEq".to_string(), structDef.name.location());
    let instanceName = Identifier::new(
        format!("PartialEq_{}", structDef.name.name()),
        structDef.name.location(),
    );
    let typeArgs = match structDef.typeParams {
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
            params: structDef.typeParams.as_ref().unwrap().params.clone(),
            constraints: constraints,
        };
        Some(decl)
    };
    let structTy = Type::Named(structDef.name.clone(), typeArgs);
    let eqFn = getPartialEqFnForStruct(structDef, &structTy);
    let types = vec![structTy];
    let instance = Instance {
        public: true,
        name: Some(instanceName),
        typeParams: typeParams,
        traitName: traitName,
        types: types,
        associatedTypes: Vec::new(),
        methods: vec![eqFn],
        location: structDef.name.location(),
    };
    //crate::siko::syntax::Format::format_any(&instance);
    instance
}

fn getPartialEqFnForStruct(structDef: &Struct, structTy: &Type) -> Function {
    let boolTy = Type::Named(
        Identifier::new("Bool.Bool".to_string(), structDef.name.location()),
        Vec::new(),
    );
    let fnName = Identifier::new("eq".to_string(), structDef.name.location());
    let mut params = Vec::new();
    params.push(Parameter::RefSelfParam);
    let otherName = Identifier::new("other".to_string(), structDef.name.location());
    let selfRefType = Type::Reference(Box::new(structTy.clone()));
    params.push(Parameter::Named(otherName, selfRefType, false));

    let body = if structDef.fields.is_empty() {
        // Empty struct is always equal to another empty struct
        let trueExpr = withName("Bool.Bool.True", structDef.name.location());
        Block {
            statements: vec![Statement {
                kind: StatementKind::Expr(trueExpr),
                hasSemicolon: false,
            }],
            location: structDef.name.location(),
        }
    } else {
        // Compare all fields for equality
        let mut fieldRefs = Vec::new();
        let mut otherFieldRefs = Vec::new();

        for field in &structDef.fields {
            let selfFieldAccess = Expr {
                expr: SimpleExpr::FieldAccess(
                    Box::new(Expr {
                        expr: SimpleExpr::SelfValue,
                        location: structDef.name.location(),
                    }),
                    field.name.clone(),
                ),
                location: structDef.name.location(),
            };
            fieldRefs.push(selfFieldAccess);

            let otherFieldAccess = Expr {
                expr: SimpleExpr::FieldAccess(
                    Box::new(Expr {
                        expr: SimpleExpr::Value(Identifier::new("other".to_string(), structDef.name.location())),
                        location: structDef.name.location(),
                    }),
                    field.name.clone(),
                ),
                location: structDef.name.location(),
            };
            otherFieldRefs.push(otherFieldAccess);
        }

        let comparison = generateMatches(fieldRefs, otherFieldRefs, structDef.name.location());
        Block {
            statements: vec![Statement {
                kind: StatementKind::Expr(comparison),
                hasSemicolon: false,
            }],
            location: structDef.name.location(),
        }
    };

    Function {
        public: true,
        name: fnName,
        params: params,
        typeParams: None,
        result: ResultKind::SingleReturn(boolTy),
        body: Some(body),
        externKind: None,
        attributes: Attributes::new(),
    }
}
