use std::fmt::Debug;
use std::fmt::Display;

#[derive(Clone)]
pub struct StructValue {
    pub name: String,
    pub fields: Vec<(String, Value)>,
}

#[derive(Clone)]
pub struct EnumValue {
    pub name: String,
    pub variant: String,
    pub fields: Vec<Value>,
}

#[derive(Clone)]
pub enum Value {
    Int(String),
    Str(String),
    Char(String),
    Struct(StructValue),
    Enum(EnumValue),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Str(s) => write!(f, "\"{}\"", s),
            Value::Char(c) => write!(f, "'{}'", c),
            Value::Struct(s) => {
                let fields: Vec<String> = s
                    .fields
                    .iter()
                    .map(|(name, value)| format!("{}: {}", name, value))
                    .collect();
                write!(f, "{} {{ {} }}", s.name, fields.join(", "))
            }
            Value::Enum(e) => {
                let fields: Vec<String> = e.fields.iter().map(|value| format!("{}", value)).collect();
                write!(f, "{}::{}({})", e.name, e.variant, fields.join(", "))
            }
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
