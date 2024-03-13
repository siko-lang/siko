use std::{env::Args, fmt::Display};

use crate::siko::qualifiedname::QualifiedName;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypeVar {
    Var(u64),
    Named(String),
}

impl Display for TypeVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            TypeVar::Named(name) => {
                write!(f, "{}", name)
            }
            TypeVar::Var(v) => write!(f, "#{}", v),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Named(QualifiedName, Vec<Type>),
    Tuple(Vec<Type>),
    Function(Vec<Type>, Box<Type>),
    Var(TypeVar),
    SelfType,
}

impl Type {
    pub fn splitFnType(self) -> (Vec<Type>, Type) {
        match self {
            Type::Function(args, result) => (args, *result),
            _ => panic!("Not a function type in splitFnType"),
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Type::Named(name, args) => {
                if args.is_empty() {
                    write!(f, "{}", name)
                } else {
                    let args: Vec<String> = args.iter().map(|t| format!("{}", t)).collect();
                    write!(f, "{}[{}]", name, args.join(", "))
                }
            }
            Type::Tuple(args) => {
                let args: Vec<String> = args.iter().map(|t| format!("{}", t)).collect();
                write!(f, "({})", args.join(", "))
            }
            Type::Function(args, result) => {
                let args: Vec<String> = args.iter().map(|t| format!("{}", t)).collect();
                write!(f, "fn({}) -> {}", args.join(", "), result)
            }
            Type::Var(v) => write!(f, "{}", v),
            Type::SelfType => write!(f, "Self"),
        }
    }
}
