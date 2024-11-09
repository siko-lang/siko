use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Void,
    Int8,
    Int16,
    Int32,
    Int64,
    Char,
    Struct(String),
    Union(String),
    Ptr(Box<Type>),
    ByteArray(u32),
}

impl Type {
    pub fn isSimple(&self) -> bool {
        match self {
            Type::Void => true,
            Type::Int8 => true,
            Type::Int16 => true,
            Type::Int32 => true,
            Type::Int64 => true,
            Type::Char => true,
            Type::Struct(_) => false,
            Type::Union(_) => false,
            Type::Ptr(_) => false,
            Type::ByteArray(_) => false,
        }
    }

    pub fn isPtr(&self) -> bool {
        match self {
            Type::Ptr(_) => true,
            _ => false,
        }
    }

    pub fn getUnion(&self) -> String {
        match self {
            Type::Union(v) => v.clone(),
            ty => unreachable!("not a union {}", ty),
        }
    }

    pub fn getStruct(&self) -> String {
        match self {
            Type::Struct(v) => v.clone(),
            ty => unreachable!("not a struct {}", ty),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Void => write!(f, "void"),
            Type::Int8 => write!(f, "i8"),
            Type::Int16 => write!(f, "i16"),
            Type::Int32 => write!(f, "i32"),
            Type::Int64 => write!(f, "i64"),
            Type::Char => write!(f, "char"),
            Type::Struct(name) => write!(f, "struct {}", name),
            Type::Union(name) => write!(f, "union {}", name),
            Type::Ptr(inner) => write!(f, "*{}", inner),
            Type::ByteArray(size) => write!(f, "u8[{}]", size),
        }
    }
}
