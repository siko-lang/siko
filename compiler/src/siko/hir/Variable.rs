use std::fmt::Debug;
use std::fmt::Display;

use crate::siko::location::Location::Location;

use super::Type::Type;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum VariableName {
    Tmp(u32),
    Local(String, u32),
    Arg(String),
    DropFlag(String),
}

impl VariableName {
    pub fn visibleName(&self) -> String {
        match self {
            VariableName::Tmp(i) => format!("tmp{}", i),
            VariableName::Local(n, _) => n.clone(),
            VariableName::Arg(n) => format!("arg_{}", n),
            VariableName::DropFlag(n) => format!("drop_flag_{}", n),
        }
    }
    pub fn isTemp(&self) -> bool {
        match self {
            VariableName::Tmp(_) => true,
            _ => false,
        }
    }

    pub fn isDropFlag(&self) -> bool {
        match self {
            VariableName::DropFlag(_) => true,
            _ => false,
        }
    }

    pub fn isArg(&self) -> bool {
        match self {
            VariableName::Arg(_) => true,
            _ => false,
        }
    }

    pub fn getDropFlag(&self) -> VariableName {
        VariableName::DropFlag(self.to_string())
    }
}

impl Display for VariableName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableName::Tmp(i) => write!(f, "tmp{}", i),
            VariableName::Local(n, i) => write!(f, "{}_{}", n, i),
            VariableName::Arg(n) => write!(f, "{}", n),
            VariableName::DropFlag(n) => write!(f, "drop_flag_{}", n),
        }
    }
}

impl Debug for VariableName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableName::Tmp(i) => write!(f, "tmp{}", i),
            VariableName::Local(n, i) => write!(f, "{}_{}", n, i),
            VariableName::Arg(n) => write!(f, "{}", n),
            VariableName::DropFlag(n) => write!(f, "drop_flag_{}", n),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Variable {
    pub name: VariableName,
    pub location: Location,
    pub ty: Option<Type>,
}

impl Variable {
    pub fn getType(&self) -> &Type {
        match &self.ty {
            Some(ty) => ty,
            None => panic!("No type found for var {}", self.name),
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
        self.name.isTemp()
    }

    pub fn isDropFlag(&self) -> bool {
        self.name.isDropFlag()
    }

    pub fn isArg(&self) -> bool {
        self.name.isArg()
    }

    pub fn getDropFlag(&self) -> Variable {
        Variable {
            name: self.name.getDropFlag(),
            location: self.location.clone(),
            ty: Some(Type::getBoolType()),
        }
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ty) = &self.ty {
            write!(f, "${}: {}", self.name, ty)
        } else {
            write!(f, "${}", self.name)
        }
    }
}

impl std::fmt::Debug for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
