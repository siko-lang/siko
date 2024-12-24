use std::fmt;

use super::Type::Type;

#[derive(Clone)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<Field>,
    pub size: u32,
    pub alignment: u32,
}

#[derive(Clone)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}

#[derive(Clone)]
pub struct Union {
    pub name: String,
    pub variants: Vec<Variant>,
    pub size: u32,
    pub alignment: u32,
    pub payloadSize: u32,
}

#[derive(Clone)]
pub struct Variant {
    pub name: String,
    pub ty: Type,
}

impl fmt::Display for Struct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Struct {} (size: {}, alignment: {}) {{\n",
            self.name, self.size, self.alignment
        )?;
        for field in &self.fields {
            write!(f, "    {}\n", field)?;
        }
        write!(f, "}}")
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.ty)
    }
}

impl fmt::Display for Union {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Union {} (size: {}, alignment: {}, payload size: {}) {{\n",
            self.name, self.size, self.alignment, self.payloadSize
        )?;
        for variant in &self.variants {
            write!(f, "    {}\n", variant)?;
        }
        write!(f, "}}")
    }
}

impl fmt::Display for Variant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.ty)
    }
}
