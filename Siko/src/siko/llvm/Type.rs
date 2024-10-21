#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Void,
    Int8,
    Int16,
    Int32,
    Int64,
    Struct(String),
    Ptr(Box<Type>),
}

impl Type {
    pub fn isPtr(&self) -> bool {
        match self {
            Type::Ptr(_) => true,
            _ => false,
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
