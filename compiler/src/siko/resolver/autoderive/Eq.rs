use crate::siko::syntax::{
    Data::{Enum, Struct},
    Identifier::Identifier,
    Trait::Instance,
    Type::{Constraint, ConstraintArgument, Type, TypeParameterDeclaration},
};

pub fn deriveEqForEnum(enumDef: &Enum) -> Instance {
    let traitName = Identifier::new("Std.Cmp.Eq".to_string(), enumDef.name.location());
    let instanceName = Identifier::new(format!("Eq_{}", enumDef.name.name()), enumDef.name.location());
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
    let types = vec![enumTy];
    let instance = Instance {
        public: true,
        name: Some(instanceName),
        typeParams: typeParams,
        traitName: traitName,
        types: types,
        associatedTypes: Vec::new(),
        methods: vec![],
        location: enumDef.name.location(),
    };
    //crate::siko::syntax::Format::format_any(&instance);
    instance
}

pub fn deriveEqForStruct(structDef: &Struct) -> Instance {
    let traitName = Identifier::new("Std.Cmp.Eq".to_string(), structDef.name.location());
    let instanceName = Identifier::new(format!("Eq_{}", structDef.name.name()), structDef.name.location());
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
    let types = vec![structTy];
    let instance = Instance {
        public: true,
        name: Some(instanceName),
        typeParams: typeParams,
        traitName: traitName,
        types: types,
        associatedTypes: Vec::new(),
        methods: vec![],
        location: structDef.name.location(),
    };
    //crate::siko::syntax::Format::format_any(&instance);
    instance
}
