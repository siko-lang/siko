use std::fmt::Display;

use super::syntax::Identifier::Identifier;

#[derive(Debug, Clone)]
pub enum QualifiedName {
    Module(Identifier),
    Item(Box<QualifiedName>, Identifier),
}

impl QualifiedName {
    pub fn add(&self, item: &Identifier) -> QualifiedName {
        QualifiedName::Item(Box::new(self.clone()), item.clone())
    }
}

impl Display for QualifiedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            QualifiedName::Module(i) => write!(f, "{}", i),
            QualifiedName::Item(p, i) => write!(f, "{}.{}", p, i),
        }
    }
}
