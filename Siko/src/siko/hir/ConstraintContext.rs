use std::fmt::Display;

use super::Type::{formatTypes, Type};

#[derive(Debug, Clone)]
pub struct TypeParameter {
    pub typeParameter: Type,
    pub constraints: Vec<Type>,
}

#[derive(Debug, Clone)]
pub struct ConstraintContext {
    pub typeParameters: Vec<TypeParameter>,
}

impl ConstraintContext {
    pub fn new() -> ConstraintContext {
        ConstraintContext { typeParameters: Vec::new() }
    }

    pub fn add(&mut self, param: TypeParameter) {
        self.typeParameters.push(param);
    }
}

impl Display for TypeParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}: {}", self.typeParameter, formatTypes(&self.constraints))
    }
}

impl Display for ConstraintContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[")?;
        for typeParameter in &self.typeParameters {
            writeln!(f, "{}", typeParameter)?;
        }
        writeln!(f, "]")?;
        Ok(())
    }
}
