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
