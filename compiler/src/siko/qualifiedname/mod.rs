use core::panic;
use std::fmt::{Debug, Display};
pub mod builtins;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
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

    pub fn base(&self) -> QualifiedName {
        match &self {
            QualifiedName::Module(_) => self.clone(),
            QualifiedName::Instance(p, _) => *p.clone(),
            QualifiedName::Item(p, _) => *p.clone(),
            QualifiedName::Monomorphized(p, _) => *p.clone(),
        }
    }

    pub fn getTraitMemberName(&self) -> QualifiedName {
        match self {
            QualifiedName::Instance(p, _) => *p.clone(),
            _ => panic!("getTraitMemberName called on non-instance QualifiedName"),
        }
    }

    pub fn monomorphized(&self, args: String) -> QualifiedName {
        match self {
            QualifiedName::Monomorphized(_, _) => panic!("Cannot monomorphize a monomorphized name"),
            _ => QualifiedName::Monomorphized(Box::new(self.clone()), args),
        }
    }

    pub fn toString(&self) -> String {
        format!("{}", self)
    }

    pub fn getShortName(&self) -> String {
        match &self {
            QualifiedName::Module(name) => name.clone(),
            QualifiedName::Instance(_, _) => panic!("Instance names are not supported"),
            QualifiedName::Item(_, name) => name.clone(),
            QualifiedName::Monomorphized(p, _) => p.getShortName(),
        }
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
                    write!(f, "{}#", p)
                } else {
                    write!(f, "{}#{}", p, args)
                }
            }
        }
    }
}

impl Debug for QualifiedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

pub fn build(m: &str, name: &str) -> QualifiedName {
    QualifiedName::Item(Box::new(QualifiedName::Module(m.to_string())), name.to_string())
}
