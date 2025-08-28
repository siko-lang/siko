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
        Type::{Type, TypeParameterDeclaration},
    },
};

pub fn deriveDiscriminatorForEnum(enumDef: &Enum) -> Instance {
    let traitName = Identifier::new("Std.Ops.Basic.Discriminator".to_string(), enumDef.name.location());
    let instanceName = Identifier::new(
        format!("Discriminator_{}", enumDef.name.name()),
        enumDef.name.location(),
    );
    let typeArgs = match enumDef.typeParams {
        Some(ref tp) => tp.params.iter().map(|p| Type::Named(p.clone(), Vec::new())).collect(),
        None => Vec::new(),
    };
    let typeParams = if typeArgs.is_empty() {
        None
    } else {
        let decl = TypeParameterDeclaration {
            params: enumDef.typeParams.as_ref().unwrap().params.clone(),
            constraints: Vec::new(),
        };
        Some(decl)
    };
    let enumTy = Type::Named(enumDef.name.clone(), typeArgs);
    let eqFn = getDiscriminatorFn(enumDef);
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

fn getDiscriminatorFn(enumDef: &Enum) -> Function {
    let intTy = Type::Named(
        Identifier::new("Int.Int".to_string(), enumDef.name.location()),
        Vec::new(),
    );
    let fnName = Identifier::new("discriminator".to_string(), enumDef.name.location());
    let mut params = Vec::new();
    params.push(Parameter::RefSelfParam);
    let mut cases = Vec::new();
    for (index, v) in enumDef.variants.iter().enumerate() {
        let variantName = Identifier::new(v.name.name(), enumDef.name.location());
        let mut args = Vec::new();
        for _ in &v.items {
            let pattern = Pattern {
                pattern: SimplePattern::Wildcard,
                location: enumDef.name.location(),
            };
            args.push(pattern);
        }
        let pattern = Pattern {
            pattern: SimplePattern::Named(variantName, args),
            location: enumDef.name.location(),
        };
        let body = Expr {
            expr: SimpleExpr::IntegerLiteral(format!("{}", index)),
            location: enumDef.name.location(),
        };
        cases.push(Branch {
            pattern,
            body: withBlock(body),
        });
    }
    let selfRef = Expr {
        expr: SimpleExpr::SelfValue,
        location: enumDef.name.location(),
    };
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
        result: intTy,
        body: Some(body),
        externKind: None,
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
