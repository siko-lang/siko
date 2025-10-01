#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Type {
    Void,
    VoidPtr,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Int8,
    Int16,
    Int32,
    Int64,
    Struct(String),
    Ptr(Box<Type>),
    Array(Box<Type>, u32),
    FunctionPtr(Vec<Type>, Box<Type>),
}

impl Type {
    pub fn isPtr(&self) -> bool {
        match self {
            Type::Ptr(_) => true,
            _ => false,
        }
    }

    pub fn isVoid(&self) -> bool {
        match self {
            Type::Void => true,
            _ => false,
        }
    }

    pub fn isVoidPtr(&self) -> bool {
        match self {
            Type::VoidPtr => true,
            _ => false,
        }
    }

    pub fn getBase(&self) -> Type {
        match self {
            Type::Ptr(p) => *p.clone(),
            _ => unreachable!(),
        }
    }

    pub fn isArray(&self) -> bool {
        match self {
            Type::Array(_, _) => true,
            _ => false,
        }
    }

    pub fn getArraySize(&self) -> u32 {
        match self {
            Type::Array(_, s) => *s,
            _ => unreachable!(),
        }
    }

    pub fn getName(&self) -> Option<String> {
        match self {
            Type::Struct(n) => Some(n.clone()),
            Type::Ptr(p) => p.getName(),
            _ => None,
        }
    }
}
