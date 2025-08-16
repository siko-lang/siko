use std::fmt::Display;

use crate::siko::{
    hir::Type::{formatTypes, Type},
    qualifiedname::QualifiedName,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key {
    Struct(QualifiedName, Vec<Type>),
    Enum(QualifiedName, Vec<Type>),
    Function(QualifiedName, Vec<Type>),
    AutoDropFn(QualifiedName, Type),
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Key::Struct(name, types) => write!(f, "{}/{}", name, formatTypes(types)),
            Key::Enum(name, types) => write!(f, "{}/{}", name, formatTypes(types)),
            Key::Function(name, types) => write!(f, "{}/{}", name, formatTypes(types)),
            Key::AutoDropFn(name, ty) => write!(f, "{}/{}", name, ty),
        }
    }
}
