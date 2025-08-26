use std::fmt::Display;

use crate::siko::{
    hir::Type::{formatTypes, Type},
    monomorphizer::Handler::HandlerResolution,
    qualifiedname::QualifiedName,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key {
    Struct(QualifiedName, Vec<Type>),
    Enum(QualifiedName, Vec<Type>),
    Function(QualifiedName, Vec<Type>, HandlerResolution, Vec<QualifiedName>),
    AutoDropFn(QualifiedName, Type, HandlerResolution),
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Key::Struct(name, types) => {
                write!(f, "{}/{}", name, formatTypes(types))
            }
            Key::Enum(name, types) => {
                write!(f, "{}/{}", name, formatTypes(types))
            }
            Key::Function(name, types, handlerResolution, impls) => {
                write!(f, "{}/{}/{}", name, formatTypes(types), handlerResolution)?;
                if !impls.is_empty() {
                    write!(
                        f,
                        ", instances: [{}]",
                        impls.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(", ")
                    )?;
                }
                Ok(())
            }
            Key::AutoDropFn(name, ty, handlerResolution) => {
                write!(f, "{}/{}/{}", name, ty, handlerResolution)
            }
        }
    }
}
