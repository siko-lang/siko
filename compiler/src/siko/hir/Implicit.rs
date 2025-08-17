use std::fmt::Debug;
use std::fmt::Display;

use crate::siko::{hir::Type::Type, qualifiedname::QualifiedName};

#[derive(Clone)]
pub struct Implicit {
    pub name: QualifiedName,
    pub ty: Type,
    pub mutable: bool,
}

impl Display for Implicit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Implicit: {} : {}", self.name, self.ty)
    }
}

impl Debug for Implicit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
