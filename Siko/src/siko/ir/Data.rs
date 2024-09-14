use std::fmt;

use crate::siko::{ir::Type::formatTypes, qualifiedname::QualifiedName};

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
pub struct Class {
    pub name: QualifiedName,
    pub ty: Type,
    pub fields: Vec<Field>,
    pub methods: Vec<MethodInfo>,
}

impl Class {
    pub fn new(name: QualifiedName, ty: Type) -> Class {
        Class {
            name: name,
            ty: ty,
            fields: Vec::new(),
            methods: Vec::new(),
        }
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
    pub ty: Type,
    pub variants: Vec<Variant>,
    pub methods: Vec<MethodInfo>,
}

impl Enum {
    pub fn new(name: QualifiedName, ty: Type) -> Enum {
        Enum {
            name: name,
            ty: ty,
            variants: Vec::new(),
            methods: Vec::new(),
        }
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

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "struct {} {{", self.name)?;
        writeln!(f, "    type: {},", self.ty)?;
        for field in &self.fields {
            writeln!(f, "    {},", field)?;
        }
        Ok(())
    }
}

impl fmt::Display for Variant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let formatted_items = formatTypes(&self.items); // Using formatTypes function
        write!(f, "{}({})", self.name, formatted_items)
    }
}

impl fmt::Display for Enum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "enum {} {{", self.name)?;
        for variant in &self.variants {
            writeln!(f, "    {},", variant)?;
        }
        Ok(())
    }
}
