use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Void,
    UInt8,
    Int16,
    Int32,
    Int64,
    Char,
    Struct(String),
    Union(String),
    Ptr(Box<Type>),
    Array(u32, u32),
}

impl Type {
    pub fn isSimple(&self) -> bool {
        match self {
            Type::Void => true,
            Type::UInt8 => true,
            Type::Int16 => true,
            Type::Int32 => true,
            Type::Int64 => true,
            Type::Char => true,
            Type::Struct(_) => false,
            Type::Union(_) => false,
            Type::Ptr(_) => false,
            Type::Array(_, _) => false,
        }
    }

    pub fn isPtr(&self) -> bool {
        match self {
            Type::Ptr(_) => true,
            _ => false,
        }
    }

    pub fn getPtrInner(&self) -> Type {
        match self {
            Type::Ptr(ty) => *ty.clone(),
            _ => unreachable!("not a ptr!"),
        }
    }

    pub fn getUnion(&self) -> String {
        match self {
            Type::Union(v) => v.clone(),
            Type::Ptr(v) => v.getUnion(),
            ty => unreachable!("not a union {}", ty),
        }
    }

    pub fn getStruct(&self) -> String {
        match self {
            Type::Struct(v) => v.clone(),
            Type::Ptr(v) => v.getStruct(),
            ty => unreachable!("not a struct {}", ty),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Void => write!(f, "void"),
            Type::UInt8 => write!(f, "i8"),
            Type::Int16 => write!(f, "i16"),
            Type::Int32 => write!(f, "i32"),
            Type::Int64 => write!(f, "i64"),
            Type::Char => write!(f, "char"),
            Type::Struct(name) => write!(f, "struct {}", name),
            Type::Union(name) => write!(f, "union {}", name),
            Type::Ptr(inner) => write!(f, "*{}", inner),
            Type::Array(size, itemSize) => write!(f, "i{}[{}]", itemSize, size),
        }
    }
}
