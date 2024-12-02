use std::fmt::Display;

use crate::siko::{hir::Type::formatTypes, qualifiedname::QualifiedName};

use super::Type::Type;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Constraint {
    Instance(QualifiedName, Vec<Type>),
    AssociatedType(QualifiedName, Type),
}

impl Display for Constraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constraint::Instance(name, types) => {
                if types.is_empty() {
                    write!(f, "{}", name)
                } else {
                    write!(f, "{}[{}]", name, formatTypes(types))
                }
            }
            Constraint::AssociatedType(name, ty) => {
                write!(f, "{} = {}", name, ty)
            }
        }
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
