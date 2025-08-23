use std::fmt::Display;

use crate::siko::{hir::Type::formatTypes, qualifiedname::QualifiedName};

use super::{Trait::AssociatedType, Type::Type};

#[derive(Debug, Clone, PartialEq)]
pub struct Constraint {
    pub name: QualifiedName,
    pub args: Vec<Type>,
    pub associatedTypes: Vec<AssociatedType>,
}

impl Display for Constraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.args.is_empty() {
            write!(f, "{}", self.name)?;
        } else {
            write!(f, "{}[{}]", self.name, formatTypes(&self.args))?;
        }
        if !self.associatedTypes.is_empty() {
            let assocTypes = self
                .associatedTypes
                .iter()
                .map(|m| format!("{}", m))
                .collect::<Vec<_>>()
                .join(",");
            write!(f, " Associated types : {}", assocTypes)?;
        }
        Ok(())
    }
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

    pub fn contains(&self, constraint: &Constraint) -> bool {
        for c in &self.constraints {
            if c.name == constraint.name && c.args == constraint.args {
                return true;
            }
        }
        false
    }
}

impl Display for ConstraintContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "args: {}", formatTypes(&self.typeParameters))?;
        write!(f, " constraints: (")?;
        for (index, c) in self.constraints.iter().enumerate() {
            if index == 0 {
                write!(f, "{}", c)?;
            } else {
                write!(f, ", {}", c)?;
            }
        }
        write!(f, ")")?;
        Ok(())
    }
}
