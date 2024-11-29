use std::fmt;

use super::Identifier::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Named(Identifier, Vec<Type>),
    Tuple(Vec<Type>),
    Function(Vec<Type>, Box<Type>),
    Reference(Box<Type>),
    SelfType,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Named(id, args) => {
                if args.is_empty() {
                    write!(f, "{}", id)
                } else {
                    let args = args.iter().map(|arg| format!("{}", arg)).collect::<Vec<_>>().join(", ");
                    write!(f, "{}[{}]", id, args)
                }
            }
            Type::Tuple(elements) => {
                let elements = elements.iter().map(|el| format!("{}", el)).collect::<Vec<_>>().join(", ");
                write!(f, "({})", elements)
            }
            Type::Function(params, ret) => {
                let params = params.iter().map(|param| format!("{}", param)).collect::<Vec<_>>().join(", ");
                write!(f, "fn({}) -> {}", params, ret)
            }
            Type::Reference(inner) => write!(f, "&{}", inner),
            Type::SelfType => write!(f, "Self"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeParameterDeclaration {
    pub params: Vec<Identifier>,
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstraintArgument {
    Type(Type),
    AssociatedType(Identifier, Type),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constraint {
    pub traitName: Identifier,
    pub args: Vec<ConstraintArgument>,
}

impl fmt::Display for ConstraintArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConstraintArgument::Type(ty) => write!(f, "{}", ty),
            ConstraintArgument::AssociatedType(id, ty) => write!(f, "{} = {}", id, ty),
        }
    }
}

impl fmt::Display for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args = self.args.iter().map(|arg| format!("{}", arg)).collect::<Vec<_>>().join(", ");
        write!(f, "{}[{}]", self.traitName, args)
    }
}
