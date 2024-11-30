use std::fmt::Display;

use crate::siko::qualifiedname::QualifiedName;

use super::Type::Type;

#[derive(Debug, Clone)]
pub enum Constraint {
    Instance(QualifiedName, Vec<Type>),
    AssociatedType(QualifiedName, Type),
}

#[derive(Debug, Clone)]
pub struct ConstraintContext {
    pub typeParameters: Vec<Type>,
    pub constraints: Vec<Constraint>,
}

impl ConstraintContext {
    pub fn new() -> ConstraintContext {
        ConstraintContext {
            typeParameters: Vec::new(),
            constraints: Vec::new(),
        }
    }

    pub fn addTypeParam(&mut self, param: Type) {
        self.typeParameters.push(param);
    }

    pub fn addConstraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
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
