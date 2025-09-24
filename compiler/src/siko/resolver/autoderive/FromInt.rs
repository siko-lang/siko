use crate::siko::{
    resolver::autoderive::Util::withBlock,
    syntax::{
        Attributes::Attributes,
        Data::Enum,
        Expr::{Branch, Expr, SimpleExpr},
        Function::{Function, Parameter, ResultKind},
        Identifier::Identifier,
        Pattern::{Pattern, SimplePattern},
        Statement::{Block, Statement, StatementKind},
        Trait::Instance,
        Type::{Type, TypeParameterDeclaration},
    },
};

pub fn deriveFromIntForEnum(enumDef: &Enum) -> Instance {
    let traitName = Identifier::new("Convert.TryFrom".to_string(), enumDef.name.location());
    let instanceName = Identifier::new(format!("TryFromInt_{}", enumDef.name.name()), enumDef.name.location());
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
    let intTy = Type::Named(
        Identifier::new("Int.Int".to_string(), enumDef.name.location()),
        Vec::new(),
    );
    let f = getFromIntFnForEnum(enumDef, &enumTy);
    let types = vec![intTy, enumTy];
    let instance = Instance {
        public: true,
        name: Some(instanceName),
        typeParams: typeParams,
        traitName: traitName,
        types: types,
        associatedTypes: Vec::new(),
        methods: vec![f],
        location: enumDef.name.location(),
    };
    //crate::siko::syntax::Format::format_any(&instance);
    instance
}

fn getFromIntFnForEnum(enumDef: &Enum, enumTy: &Type) -> Function {
    let fnName = Identifier::new("tryFrom".to_string(), enumDef.name.location());
    let mut params = Vec::new();
    params.push(Parameter::SelfParam);
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
            pattern: SimplePattern::IntegerLiteral(format!("{}", index)),
            location: enumDef.name.location(),
        };
        let body = Expr {
            expr: SimpleExpr::Call(
                Box::new(Expr {
                    expr: SimpleExpr::Name(Identifier::new("Option.Some".to_string(), enumDef.name.location())),
                    location: enumDef.name.location(),
                }),
                vec![Expr {
                    expr: SimpleExpr::Call(
                        Box::new(Expr {
                            expr: SimpleExpr::Name(variantName),
                            location: enumDef.name.location(),
                        }),
                        Vec::new(),
                    ),
                    location: enumDef.name.location(),
                }],
            ),
            location: enumDef.name.location(),
        };
        cases.push(Branch {
            pattern,
            body: withBlock(body),
        });
    }
    let noneBranch = Branch {
        pattern: Pattern {
            pattern: SimplePattern::Wildcard,
            location: enumDef.name.location(),
        },
        body: withBlock(Expr {
            expr: SimpleExpr::Name(Identifier::new("Option.None".to_string(), enumDef.name.location())),
            location: enumDef.name.location(),
        }),
    };
    cases.push(noneBranch);
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
        result: ResultKind::SingleReturn(Type::Named(
            Identifier::new("Option".to_string(), enumDef.name.location()),
            vec![enumTy.clone()],
        )),
        body: Some(body),
        externKind: None,
        attributes: Attributes::new(),
    }
}
