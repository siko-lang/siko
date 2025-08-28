use crate::siko::{
    location::Location::Location,
    syntax::{
        Expr::{Branch, Expr, SimpleExpr},
        Identifier::Identifier,
        Pattern::{Pattern, SimplePattern},
        Statement::{Block, Statement, StatementKind},
    },
};

/// Creates a block expression containing a single expression statement
pub fn withBlock(e: Expr) -> Expr {
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

/// Creates a name expression from a string
pub fn withName(n: &str, location: Location) -> Expr {
    Expr {
        expr: SimpleExpr::Name(Identifier::new(n.to_string(), location.clone())),
        location: location.clone(),
    }
}

/// Wraps an expression in Option.Some
pub fn withSome(inner: Expr) -> Expr {
    let location = inner.location.clone();
    Expr {
        expr: SimpleExpr::Call(Box::new(withName("Option.Option.Some", location.clone())), vec![inner]),
        location: location.clone(),
    }
}

/// Generates field-by-field comparison for PartialEq (Boolean logic)
pub fn generateMatches(itemARefs: Vec<Expr>, itemBRefs: Vec<Expr>, location: Location) -> Expr {
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

/// Generates field-by-field comparison for PartialOrd (returns Option[Ordering])
pub fn generatePartialOrdFieldComparison(itemARefs: Vec<Expr>, itemBRefs: Vec<Expr>, location: Location) -> Expr {
    if itemARefs.is_empty() {
        return withSome(withName("Ordering.Ordering.Equal", location.clone()));
    }
    let firstA = itemARefs[0].clone();
    let firstB = itemBRefs[0].clone();
    let cmpCall = Expr {
        expr: SimpleExpr::Call(
            Box::new(withName("Std.Cmp.PartialOrd.partialCmp", location.clone())),
            vec![firstA, firstB],
        ),
        location: location.clone(),
    };
    if itemARefs.len() == 1 {
        return cmpCall;
    }
    let restA = itemARefs[1..].to_vec();
    let restB = itemBRefs[1..].to_vec();
    let restComparison = generatePartialOrdFieldComparison(restA, restB, location.clone());

    let equalPattern = Pattern {
        pattern: SimplePattern::Named(
            Identifier::new("Option.Option.Some".to_string(), location.clone()),
            vec![Pattern {
                pattern: SimplePattern::Named(
                    Identifier::new("Ordering.Ordering.Equal".to_string(), location.clone()),
                    vec![],
                ),
                location: location.clone(),
            }],
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

/// Generates field-by-field comparison for Ord (returns Ordering)
pub fn generateOrdFieldComparison(itemARefs: Vec<Expr>, itemBRefs: Vec<Expr>, location: Location) -> Expr {
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
    let restComparison = generateOrdFieldComparison(restA, restB, location.clone());

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
