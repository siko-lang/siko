use std::{collections::BTreeMap, fmt::Display};

use crate::siko::{
    hir::{Program::Program, Type::Type, TypeVarAllocator::TypeVarAllocator},
    qualifiedname::QualifiedName,
    util::DependencyProcessor::{processDependencies, DependencyGroup},
};

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
    name: String,
    ty: Type,
    vars: Vec<Type>,
    inGroup: bool,
}

impl Display for FieldDef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}:{}[{}]",
            self.name,
            self.ty,
            self.vars.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ")
        )
    }
}

#[derive(Debug, Clone)]
pub struct StructDef {
    name: QualifiedName,
    vars: Vec<Type>,
    fields: Vec<FieldDef>,
}

impl Display for StructDef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "struct {}[{}]:{}",
            self.name,
            self.vars.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", "),
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
    name: String,
    vars: Vec<Type>,
    inGroup: bool,
}

impl Display for VariantDef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}: [{}]",
            self.name,
            self.vars.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ")
        )
    }
}

#[derive(Debug, Clone)]
pub struct EnumDef {
    name: QualifiedName,
    vars: Vec<Type>,
    variants: Vec<VariantDef>,
}

impl Display for EnumDef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "enum {}[{}]:{}",
            self.name,
            self.vars.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", "),
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
    dataDefs: BTreeMap<QualifiedName, DataDef>,
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
                    vars: Vec::new(),
                    fields: Vec::new(),
                };
                for field in s.fields.iter() {
                    let mut fieldDef = FieldDef {
                        name: field.name.clone(),
                        ty: field.ty.clone(),
                        vars: Vec::new(),
                        inGroup: false,
                    };
                    if Type::isReference(&field.ty) {
                        let refVar = allocator.next();
                        fieldDef.vars.push(refVar.clone());
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
                                fieldDef.vars.push(v.clone());
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
                    vars: Vec::new(),
                    variants: Vec::new(),
                };
                for variant in e.variants.iter() {
                    let mut variantDef = VariantDef {
                        name: variant.name.toString(),
                        vars: Vec::new(),
                        inGroup: false,
                    };
                    for field in variant.items.iter() {
                        if Type::isReference(field) {
                            let refVar = allocator.next();
                            variantDef.vars.push(refVar.clone());
                            groupVars.push(refVar);
                        }
                        if let Some(name) = getNameFromType(field) {
                            //println!("Found name from type: {}", name);
                            if group.items.contains(&name) {
                                variantDef.inGroup = true;
                            } else {
                                let refGroups = self.refGroups.get(&name).expect("Dependency not found");
                                for _ in refGroups.iter() {
                                    let v = allocator.next();
                                    variantDef.vars.push(v.clone());
                                    groupVars.push(v);
                                }
                            }
                        }
                    }
                    enumDef.variants.push(variantDef);
                }
                enums.push(enumDef);
            }
        }
        for s in structs.iter_mut() {
            s.vars = groupVars.clone();
            for f in s.fields.iter_mut() {
                if f.inGroup {
                    f.vars = groupVars.clone();
                }
            }
        }
        for e in enums.iter_mut() {
            e.vars = groupVars.clone();
            for v in e.variants.iter_mut() {
                if v.inGroup {
                    v.vars = groupVars.clone();
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
    }
}
