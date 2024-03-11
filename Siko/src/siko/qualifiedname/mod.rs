use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum QualifiedName {
    Module(String),
    Item(Box<QualifiedName>, String),
}

impl QualifiedName {
    pub fn add(&self, item: String) -> QualifiedName {
        QualifiedName::Item(Box::new(self.clone()), item)
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
