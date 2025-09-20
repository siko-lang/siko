use std::{collections::BTreeMap, fmt::Debug, fmt::Display};

use crate::siko::{
    hir::{
        Program::Program,
        Type::{formatTypes, Type},
        TypeVarAllocator::TypeVarAllocator,
    },
    qualifiedname::QualifiedName,
    util::DependencyProcessor::{processDependencies, DependencyGroup},
};

#[derive(Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct ExtendedType {
    pub ty: Type,
    pub vars: Vec<Type>,
}

impl ExtendedType {
    pub fn new(ty: Type) -> Self {
        ExtendedType { ty, vars: Vec::new() }
    }

    pub fn base(&mut self) -> Type {
        self.vars.remove(0)
    }
}

impl Display for ExtendedType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{}", self.ty, formatTypes(&self.vars))
    }
}

impl Debug for ExtendedType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Debug, Clone)]
pub enum DataDef {
    Struct(StructDef),
    Enum(EnumDef),
}

impl Display for DataDef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DataDef::Struct(def) => write!(f, "Struct({})", def),
            DataDef::Enum(def) => write!(f, "Enum({})", def),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,
    pub ty: ExtendedType,
    pub inGroup: bool,
}

impl Display for FieldDef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:{}", self.name, self.ty,)
    }
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: QualifiedName,
    pub ty: ExtendedType,
    pub fields: Vec<FieldDef>,
}

impl StructDef {
    pub fn getField(&self, name: &str) -> &FieldDef {
        //println!("Looking for field {} in struct {}", name, self.name);
        for field in self.fields.iter() {
            //println!("Checking field: {}", field.name);
            if field.name == name {
                return field;
            }
        }
        panic!("Field not found");
    }
}

impl Display for StructDef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "struct {}[{}]:{}",
            self.name,
            self.ty,
            self.fields
                .iter()
                .map(|v| format!("\n   {}", v))
                .collect::<Vec<_>>()
                .join("")
        )
    }
}

#[derive(Debug, Clone)]
pub struct VariantDef {
    pub name: String,
    pub ty: ExtendedType,
    pub inGroup: bool,
}

impl Display for VariantDef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:{}", self.name, self.ty)
    }
}

#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: QualifiedName,
    pub ty: ExtendedType,
    pub variants: Vec<VariantDef>,
}

impl Display for EnumDef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "enum {}[{}]:{}",
            self.name,
            self.ty,
            self.variants
                .iter()
                .map(|v| format!("\n   {}", v))
                .collect::<Vec<_>>()
                .join("")
        )
    }
}

pub struct DataGroups<'a> {
    program: &'a Program,
    refGroups: BTreeMap<QualifiedName, Vec<Type>>,
    pub dataDefs: BTreeMap<QualifiedName, DataDef>,
}

impl<'a> DataGroups<'a> {
    pub fn new(program: &'a Program) -> Self {
        DataGroups {
            program,
            refGroups: BTreeMap::new(),
            dataDefs: BTreeMap::new(),
        }
    }

    pub fn process(&mut self) {
        let mut allDeps: BTreeMap<QualifiedName, Vec<QualifiedName>> = BTreeMap::new();
        for (_, enumDef) in self.program.enums.iter() {
            allDeps.insert(enumDef.name.clone(), Vec::new());
        }
        for (_, structDef) in self.program.structs.iter() {
            allDeps.insert(structDef.name.clone(), Vec::new());
        }
        for (_, enumDef) in self.program.enums.iter() {
            for variant in enumDef.variants.iter() {
                for field in variant.items.iter() {
                    if let Some(itemName) = getNameFromType(field) {
                        allDeps.entry(enumDef.name.clone()).or_default().push(itemName);
                    }
                }
            }
        }
        for (_, structDef) in self.program.structs.iter() {
            for field in structDef.fields.iter() {
                if let Some(itemName) = getNameFromType(&field.ty) {
                    allDeps.entry(structDef.name.clone()).or_default().push(itemName);
                }
            }
        }
        let groups = processDependencies(&allDeps);
        for group in groups.iter() {
            self.processDataGroup(group);
        }
    }

    fn processDataGroup(&mut self, group: &DependencyGroup<QualifiedName>) {
        //println!("------- Processing data group: {:?}", group);
        let allocator = TypeVarAllocator::new();
        let mut groupVars = Vec::new();
        let mut structs = Vec::new();
        let mut enums = Vec::new();
        for item in group.items.iter() {
            if let Some(s) = self.program.getStruct(item) {
                let mut structDef = StructDef {
                    name: s.name.clone(),
                    ty: ExtendedType::new(s.ty.clone()),
                    fields: Vec::new(),
                };
                for field in s.fields.iter() {
                    let mut fieldDef = FieldDef {
                        name: field.name.clone(),
                        ty: ExtendedType::new(field.ty.clone()),
                        inGroup: false,
                    };
                    if Type::isReference(&field.ty) {
                        let refVar = allocator.next();
                        fieldDef.ty.vars.push(refVar.clone());
                        groupVars.push(refVar);
                    }
                    if let Some(name) = getNameFromType(&field.ty) {
                        //println!("Found name from type: {}", name);
                        if group.items.contains(&name) {
                            fieldDef.inGroup = true;
                        } else {
                            let refGroups = self.refGroups.get(&name).expect("Dependency not found");
                            for _ in refGroups.iter() {
                                let v = allocator.next();
                                fieldDef.ty.vars.push(v.clone());
                                groupVars.push(v);
                            }
                        }
                    }
                    structDef.fields.push(fieldDef);
                }
                structs.push(structDef);
            }
            if let Some(e) = self.program.getEnum(item) {
                let mut enumDef = EnumDef {
                    name: e.name.clone(),
                    ty: ExtendedType::new(e.ty.clone()),
                    variants: Vec::new(),
                };
                for variant in e.variants.iter() {
                    assert_eq!(variant.items.len(), 1);
                    let itemTy = variant.items[0].clone();
                    let mut variantDef = VariantDef {
                        name: variant.name.toString(),
                        ty: ExtendedType::new(itemTy.clone()),
                        inGroup: false,
                    };
                    if Type::isReference(&itemTy) {
                        let refVar = allocator.next();
                        variantDef.ty.vars.push(refVar.clone());
                        groupVars.push(refVar);
                    }
                    if let Some(name) = getNameFromType(&itemTy) {
                        //println!("Found name from type: {}", name);
                        if group.items.contains(&name) {
                            variantDef.inGroup = true;
                        } else {
                            let refGroups = self.refGroups.get(&name).expect("Dependency not found");
                            for _ in refGroups.iter() {
                                let v = allocator.next();
                                variantDef.ty.vars.push(v.clone());
                                groupVars.push(v);
                            }
                        }
                    }
                    enumDef.variants.push(variantDef);
                }
                enums.push(enumDef);
            }
        }
        for s in structs.iter_mut() {
            s.ty.vars = groupVars.clone();
            for f in s.fields.iter_mut() {
                if f.inGroup {
                    f.ty.vars = groupVars.clone();
                }
            }
        }
        for e in enums.iter_mut() {
            e.ty.vars = groupVars.clone();
            for v in e.variants.iter_mut() {
                if v.inGroup {
                    v.ty.vars = groupVars.clone();
                }
            }
        }
        for s in structs.iter() {
            //println!("{}", s);
            self.dataDefs.insert(s.name.clone(), DataDef::Struct(s.clone()));
        }
        for e in enums.iter() {
            //println!("{}", e);
            self.dataDefs.insert(e.name.clone(), DataDef::Enum(e.clone()));
        }
        for item in group.items.iter() {
            self.refGroups.insert(item.clone(), groupVars.clone());
        }
    }

    pub fn extendType(&self, ty: &Type) -> ExtendedType {
        let mut extTy = ExtendedType::new(ty.clone());
        let allocator = TypeVarAllocator::new();
        if let Some(name) = getNameFromType(ty) {
            let def = self
                .dataDefs
                .get(&name)
                .expect(&format!("Data definition not found {}", name));
            match def {
                DataDef::Struct(s) => {
                    allocator.useTypes(&s.ty.vars);
                    extTy.vars.extend(s.ty.vars.clone());
                }
                DataDef::Enum(e) => {
                    allocator.useTypes(&e.ty.vars);
                    extTy.vars.extend(e.ty.vars.clone());
                }
            }
        }
        if Type::isReference(ty) {
            let refVar = allocator.next();
            extTy.vars.insert(0, refVar);
        }
        extTy
    }

    pub fn getStruct(&self, name: &QualifiedName) -> &StructDef {
        match self.dataDefs.get(name) {
            Some(DataDef::Struct(s)) => s,
            _ => panic!("Struct not found {}", name),
        }
    }

    pub fn getEnum(&self, name: &QualifiedName) -> &EnumDef {
        match self.dataDefs.get(name) {
            Some(DataDef::Enum(e)) => e,
            _ => panic!("Enum not found {}", name),
        }
    }
}

fn getNameFromType(ty: &Type) -> Option<QualifiedName> {
    match ty {
        Type::Named(name, _) => Some(name.clone()),
        Type::Reference(base) => getNameFromType(base),
        Type::Ptr(base) => getNameFromType(base),
        Type::Tuple(_) => panic!("Tuple type in borrowcheck"),
        Type::Function(_, _) => panic!("Function type in borrowcheck"),
        Type::Var(type_var) => panic!("Type var in borrowcheck: {}", type_var),
        Type::SelfType => panic!("Self type in borrowcheck"),
        Type::Never(_) => None,
        Type::NumericConstant(_) => None,
        Type::Void => None,
        Type::VoidPtr => None,
        Type::Coroutine(_, _) => panic!("Coroutine type in borrowcheck"),
    }
}
