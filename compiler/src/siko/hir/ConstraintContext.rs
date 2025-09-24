use std::fmt::Debug;
use std::fmt::Display;

use crate::siko::{hir::Type::formatTypes, qualifiedname::QualifiedName};

use super::{Trait::AssociatedType, Type::Type};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Constraint {
    pub name: QualifiedName,
    pub args: Vec<Type>,
    pub associatedTypes: Vec<AssociatedType>,
    pub main: bool,
}

impl Display for Constraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //let m = if self.main { " (m)" } else { "" };
        let m = "";
        if self.args.is_empty() {
            write!(f, "{}{}", self.name, m)?;
        } else {
            write!(f, "{}{}[{}]", self.name, m, formatTypes(&self.args))?;
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

impl Debug for Constraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
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

    pub fn containsAt(&self, constraint: &Constraint) -> Option<u32> {
        for (index, c) in self.constraints.iter().enumerate() {
            if c.name == constraint.name && c.args == constraint.args {
                return Some(index as u32);
            }
        }
        None
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
