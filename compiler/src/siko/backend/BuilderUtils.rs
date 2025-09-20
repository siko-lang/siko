use crate::siko::{
    hir::{
        ConstraintContext::ConstraintContext,
        Data::{Enum, Field, Struct, Variant},
        Function::{Attributes, Function, FunctionKind, Parameter, ResultKind},
        Program::Program,
        Type::Type,
    },
    location::Location::Location,
    qualifiedname::QualifiedName,
};

pub fn getStructFieldName(index: u32) -> String {
    format!("f{}", index)
}

pub struct EnumBuilder<'a> {
    enumName: QualifiedName,
    variants: Vec<Variant>,
    program: &'a mut Program,
    location: Location,
}

impl<'a> EnumBuilder<'a> {
    pub fn new(enumName: QualifiedName, program: &'a mut Program, location: Location) -> EnumBuilder<'a> {
        EnumBuilder {
            enumName,
            program,
            variants: Vec::new(),
            location,
        }
    }

    pub fn generateVariant(&mut self, variantName: &QualifiedName, fieldTypes: &Vec<Type>, variantIndex: usize) {
        let structTy = self.generateVariantStruct(fieldTypes, variantName);
        let variant = Variant {
            name: variantName.clone(),
            items: vec![structTy],
        };
        let mut variantCtorParams = Vec::new();
        for (i, fieldTy) in fieldTypes.iter().enumerate() {
            let argName = getStructFieldName(i as u32);
            variantCtorParams.push(Parameter::Named(argName, fieldTy.clone(), false));
        }
        let variantCtorFn = Function {
            name: variantName.clone(),
            params: variantCtorParams,
            result: ResultKind::SingleReturn(self.getEnumType()),
            body: None,
            constraintContext: ConstraintContext::new(),
            kind: FunctionKind::VariantCtor(variantIndex as i64),
            attributes: Attributes::new(),
        };
        self.program.functions.insert(variantCtorFn.name.clone(), variantCtorFn);
        self.variants.push(variant);
    }

    fn generateVariantStruct(&mut self, fieldTypes: &Vec<Type>, variantName: &QualifiedName) -> Type {
        let mut builder = StructBuilder::new(self.program, self.location.clone());
        builder.generateStruct(fieldTypes, &QualifiedName::VariantStruct(Box::new(variantName.clone())))
    }

    pub fn generateEnum(&mut self, location: &Location) {
        let enumDef = Enum {
            name: self.enumName.clone(),
            ty: self.getEnumType(),
            variants: self.variants.clone(),
            location: location.clone(),
            methods: Vec::new(),
        };
        //println!("EnumBuider: enum: {}", enumDef);
        self.program.enums.insert(enumDef.name.clone(), enumDef);
    }

    pub fn getEnumType(&self) -> Type {
        Type::Named(self.enumName.clone(), Vec::new())
    }
}

pub struct StructBuilder<'a> {
    program: &'a mut Program,
    location: Location,
}

impl<'a> StructBuilder<'a> {
    pub fn new(program: &'a mut Program, location: Location) -> StructBuilder<'a> {
        StructBuilder { program, location }
    }

    pub fn generateStruct(&mut self, fieldTypes: &Vec<Type>, structName: &QualifiedName) -> Type {
        let mut fields = Vec::new();
        for fieldTy in fieldTypes {
            fields.push(Field {
                name: getStructFieldName(fields.len() as u32),
                ty: fieldTy.clone(),
            });
        }
        let structTy = Type::Named(structName.clone(), Vec::new());
        let structDef = Struct {
            name: structName.clone(),
            originalName: format!("{}", structName.clone()),
            fields: fields,
            location: self.location.clone(),
            ty: structTy.clone(),
            methods: Vec::new(),
        };
        self.program.structs.insert(structDef.name.clone(), structDef);
        let mut structCtorParams = Vec::new();
        for (i, fieldTy) in fieldTypes.iter().enumerate() {
            let argName = getStructFieldName(i as u32);
            structCtorParams.push(Parameter::Named(argName, fieldTy.clone(), false));
        }
        let structCtorFn = Function {
            name: structName.clone(),
            params: structCtorParams,
            result: ResultKind::SingleReturn(Type::Named(structName.clone(), Vec::new())),
            body: None,
            constraintContext: ConstraintContext::new(),
            kind: FunctionKind::StructCtor,
            attributes: Attributes::new(),
        };
        self.program.functions.insert(structCtorFn.name.clone(), structCtorFn);
        structTy
    }
}
