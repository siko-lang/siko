use std::fmt::{Debug, Display};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum QualifiedName {
    Module(String),
    Instance(Box<QualifiedName>, u64),
    Item(Box<QualifiedName>, String),
    Monomorphized(Box<QualifiedName>, String),
}

impl QualifiedName {
    pub fn add(&self, item: String) -> QualifiedName {
        QualifiedName::Item(Box::new(self.clone()), item)
    }

    pub fn module(&self) -> QualifiedName {
        match &self {
            QualifiedName::Module(_) => self.clone(),
            QualifiedName::Instance(p, _) => p.module(),
            QualifiedName::Item(p, _) => p.module(),
            QualifiedName::Monomorphized(p, _) => p.module(),
        }
    }

    pub fn monomorphized(&self, args: String) -> QualifiedName {
        QualifiedName::Monomorphized(Box::new(self.clone()), args)
    }

    pub fn toString(&self) -> String {
        format!("{}", self)
    }
}

impl Display for QualifiedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            QualifiedName::Module(i) => write!(f, "{}", i),
            QualifiedName::Instance(p, i) => write!(f, "{}/{}", p, i),
            QualifiedName::Item(p, i) => write!(f, "{}.{}", p, i),
            QualifiedName::Monomorphized(p, args) => {
                if args.is_empty() {
                    write!(f, "{}", p)
                } else {
                    write!(f, "{}#{}", p, args)
                }
            }
        }
    }
}

pub fn build(m: &str, name: &str) -> QualifiedName {
    QualifiedName::Item(
        Box::new(QualifiedName::Module(m.to_string())),
        name.to_string(),
    )
}

pub fn getBoolTypeName() -> QualifiedName {
    build("Bool", "Bool")
}

pub fn getIntTypeName() -> QualifiedName {
    build("Int", "Int")
}

pub fn getStringTypeName() -> QualifiedName {
    build("String", "String")
}

pub fn getCharTypeName() -> QualifiedName {
    build("Char", "Char")
}

pub fn getTrueName() -> QualifiedName {
    build("Bool", "Bool").add("True".to_string())
}

pub fn getFalseName() -> QualifiedName {
    build("Bool", "Bool").add("False".to_string())
}
