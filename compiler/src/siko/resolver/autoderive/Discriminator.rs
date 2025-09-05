use crate::siko::{
    resolver::autoderive::Util::withBlock,
    syntax::{
        Data::{Enum, Struct},
        Expr::{Branch, Expr, SimpleExpr},
        Function::{Attributes, Function, Parameter},
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
    let eqFn = getDiscriminatorFnForEnum(enumDef);
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

fn getDiscriminatorFnForEnum(enumDef: &Enum) -> Function {
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
        attributes: Attributes::new(),
    }
}

pub fn deriveDiscriminatorForStruct(structDef: &Struct) -> Instance {
    let traitName = Identifier::new("Std.Ops.Basic.Discriminator".to_string(), structDef.name.location());
    let instanceName = Identifier::new(
        format!("Discriminator_{}", structDef.name.name()),
        structDef.name.location(),
    );
    let typeArgs = match structDef.typeParams {
        Some(ref tp) => tp.params.iter().map(|p| Type::Named(p.clone(), Vec::new())).collect(),
        None => Vec::new(),
    };
    let typeParams = if typeArgs.is_empty() {
        None
    } else {
        let decl = TypeParameterDeclaration {
            params: structDef.typeParams.as_ref().unwrap().params.clone(),
            constraints: Vec::new(),
        };
        Some(decl)
    };
    let structTy = Type::Named(structDef.name.clone(), typeArgs);
    let discriminatorFn = getDiscriminatorFnForStruct(structDef);
    let types = vec![structTy];
    let instance = Instance {
        public: true,
        name: Some(instanceName),
        typeParams: typeParams,
        traitName: traitName,
        types: types,
        associatedTypes: Vec::new(),
        methods: vec![discriminatorFn],
        location: structDef.name.location(),
    };
    //crate::siko::syntax::Format::format_any(&instance);
    instance
}

fn getDiscriminatorFnForStruct(structDef: &Struct) -> Function {
    let intTy = Type::Named(
        Identifier::new("Int.Int".to_string(), structDef.name.location()),
        Vec::new(),
    );
    let fnName = Identifier::new("discriminator".to_string(), structDef.name.location());
    let mut params = Vec::new();
    params.push(Parameter::RefSelfParam);

    // Structs always return 0 as their discriminator
    let zeroLiteral = Expr {
        expr: SimpleExpr::IntegerLiteral("0".to_string()),
        location: structDef.name.location(),
    };

    let body = Block {
        statements: vec![Statement {
            kind: StatementKind::Expr(zeroLiteral),
            hasSemicolon: false,
        }],
        location: structDef.name.location(),
    };

    Function {
        public: true,
        name: fnName,
        params: params,
        typeParams: None,
        result: intTy,
        body: Some(body),
        externKind: None,
        attributes: Attributes::new(),
    }
}
