use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
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
    Union(String),
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

    pub fn isVoidPtr(&self) -> bool {
        match self {
            Type::VoidPtr => true,
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
            Type::VoidPtr => write!(f, "void*"),
            Type::UInt8 => write!(f, "u8"),
            Type::UInt16 => write!(f, "u16"),
            Type::UInt32 => write!(f, "u32"),
            Type::UInt64 => write!(f, "u64"),
            Type::Int8 => write!(f, "i8"),
            Type::Int16 => write!(f, "i16"),
            Type::Int32 => write!(f, "i32"),
            Type::Int64 => write!(f, "i64"),
            Type::Struct(name) => write!(f, "struct {}", name),
            Type::Union(name) => write!(f, "union {}", name),
            Type::Ptr(inner) => write!(f, "*{}", inner),
            Type::Array(ty, len) => write!(f, "{}[{}]", ty, len),
            Type::FunctionPtr(args, result) => {
                let args: Vec<String> = args.iter().map(|t| format!("{}", t)).collect();
                write!(f, "fn*({}) -> {}", args.join(", "), result)
            }
        }
    }
}
