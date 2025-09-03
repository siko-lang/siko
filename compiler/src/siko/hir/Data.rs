use std::fmt;

use crate::siko::{hir::Type::formatTypes, location::Location::Location, qualifiedname::QualifiedName};

use super::Type::Type;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MethodInfo {
    pub name: String,
    pub fullName: QualifiedName,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Struct {
    pub name: QualifiedName,
    pub location: Location,
    pub ty: Type,
    pub fields: Vec<Field>,
    pub methods: Vec<MethodInfo>,
}

impl Struct {
    pub fn new(name: QualifiedName, ty: Type, location: Location) -> Struct {
        Struct {
            name: name,
            ty: ty,
            fields: Vec::new(),
            methods: Vec::new(),
            location: location,
        }
    }

    pub fn getField(&self, name: &String) -> (Field, i32) {
        for (index, f) in self.fields.iter().enumerate() {
            if f.name == *name {
                return (f.clone(), index as i32);
            }
        }
        println!("Field not found {}", name);
        println!("{}", self);
        unreachable!();
    }
}

#[derive(Clone, Debug)]
pub struct Variant {
    pub name: QualifiedName,
    pub items: Vec<Type>,
}

#[derive(Clone, Debug)]
pub struct Enum {
    pub name: QualifiedName,
    pub location: Location,
    pub ty: Type,
    pub variants: Vec<Variant>,
    pub methods: Vec<MethodInfo>,
}

impl Enum {
    pub fn new(name: QualifiedName, ty: Type, location: Location) -> Enum {
        Enum {
            name: name,
            ty: ty,
            location: location,
            variants: Vec::new(),
            methods: Vec::new(),
        }
    }

    pub fn getVariant(&self, name: &QualifiedName) -> (Variant, u32) {
        for (index, v) in self.variants.iter().enumerate() {
            if v.name == *name {
                return (v.clone(), index as u32);
            }
        }
        panic!("variant {} not found", name);
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.ty)
    }
}

impl fmt::Display for MethodInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fn {}();", self.name)
    }
}

impl fmt::Display for Struct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "structDef {} {{", self.name)?;
        //writeln!(f, "    type: {},", self.ty)?;
        for field in &self.fields {
            writeln!(f, "    {},", field)?;
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}

impl fmt::Display for Variant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let formatted_items = formatTypes(&self.items);
        write!(f, "{}({})", self.name, formatted_items)
    }
}

impl fmt::Display for Enum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "enum {} {{", self.name)?;
        for variant in &self.variants {
            writeln!(f, "    {},", variant)?;
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}
