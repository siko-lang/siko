use std::fmt::Debug;
use std::fmt::Display;

use crate::siko::location::Location::Location;

use super::Type::Type;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum VariableName {
    Tmp(u32),
    Local(String, u32),
    Arg(String),
}

impl VariableName {
    pub fn visibleName(&self) -> String {
        match self {
            VariableName::Tmp(i) => format!("tmp{}", i),
            VariableName::Local(n, _) => n.clone(),
            VariableName::Arg(n) => n.clone(),
        }
    }
    pub fn isTemp(&self) -> bool {
        match self {
            VariableName::Tmp(_) => true,
            _ => false,
        }
    }
}

impl Display for VariableName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableName::Tmp(i) => write!(f, "tmp{}", i),
            VariableName::Local(n, i) => write!(f, "{}_{}", n, i),
            VariableName::Arg(n) => write!(f, "{}", n),
        }
    }
}

impl Debug for VariableName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableName::Tmp(i) => write!(f, "tmp{}", i),
            VariableName::Local(n, i) => write!(f, "{}_{}", n, i),
            VariableName::Arg(n) => write!(f, "{}", n),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Variable {
    pub value: VariableName,
    pub location: Location,
    pub ty: Option<Type>,
}

impl Variable {
    pub fn getType(&self) -> &Type {
        match &self.ty {
            Some(ty) => ty,
            None => panic!("No type found for var {}", self.value),
        }
    }

    pub fn replace(&self, from: &Variable, to: Variable) -> Variable {
        if self == from {
            to
        } else {
            self.clone()
        }
    }

    pub fn isTemp(&self) -> bool {
        self.value.isTemp()
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ty) = &self.ty {
            write!(f, "${}: {}", self.value, ty)
        } else {
            write!(f, "${}", self.value)
        }
    }
}

impl std::fmt::Debug for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
