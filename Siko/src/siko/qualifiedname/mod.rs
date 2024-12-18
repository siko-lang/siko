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

    pub fn base(&self) -> QualifiedName {
        match &self {
            QualifiedName::Module(_) => self.clone(),
            QualifiedName::Instance(p, _) => *p.clone(),
            QualifiedName::Item(p, _) => *p.clone(),
            QualifiedName::Monomorphized(p, _) => *p.clone(),
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
    QualifiedName::Item(Box::new(QualifiedName::Module(m.to_string())), name.to_string())
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

pub fn getStringEqName() -> QualifiedName {
    build("String", "String").add("eq".to_string())
}

pub fn getPtrNullName() -> QualifiedName {
    build("Ptr", "null")
}

pub fn getPtrAllocateArrayName() -> QualifiedName {
    build("Ptr", "allocateArray")
}

pub fn getPtrDeallocateName() -> QualifiedName {
    build("Ptr", "deallocate")
}

pub fn getPtrMemcpyName() -> QualifiedName {
    build("Ptr", "memcpy")
}

pub fn getPtrOffsetName() -> QualifiedName {
    build("Ptr", "offset")
}

pub fn getPtrStoreName() -> QualifiedName {
    build("Ptr", "store")
}

pub fn getPtrToRefName() -> QualifiedName {
    build("Ptr", "toRef")
}

pub fn getPtrPrintName() -> QualifiedName {
    build("Ptr", "print")
}

pub fn getPtrCloneName() -> QualifiedName {
    build("Ptr", "clone")
}

pub fn getCloneName() -> QualifiedName {
    build("Std.Ops", "Clone").add(format!("clone"))
}

pub fn getCopyName() -> QualifiedName {
    build("Std.Ops", "Copy")
}
