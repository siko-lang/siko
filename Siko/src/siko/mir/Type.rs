#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Void,
    Int8,
    Int16,
    Int32,
    Int64,
    Char,
    Struct(String),
    Ptr(Box<Type>),
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
            Type::Ptr(_) => false,
        }
    }
}
