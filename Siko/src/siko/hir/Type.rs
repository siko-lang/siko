use std::{collections::BTreeSet, fmt::Display};

use crate::siko::qualifiedname::{getBoolTypeName, getCharTypeName, getIntTypeName, getStringTypeName, QualifiedName};

use super::Lifetime::{Lifetime, LifetimeInfo};

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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Type {
    Named(QualifiedName, Vec<Type>, Option<LifetimeInfo>),
    Tuple(Vec<Type>),
    Function(Vec<Type>, Box<Type>),
    Var(TypeVar),
    Reference(Box<Type>, Option<Lifetime>),
    Ptr(Box<Type>),
    SelfType,
    Never(bool), // true = explicit never i.e. !
}

impl Type {
    pub fn getName(&self) -> Option<QualifiedName> {
        match &self {
            Type::Named(n, _, _) => Some(n.clone()),
            Type::Reference(ty, _) => ty.getName(),
            _ => None,
        }
    }

    pub fn unpackRef(&self) -> &Type {
        match self {
            Type::Reference(ty, _) => ty.unpackRef(),
            ty => ty,
        }
    }

    pub fn splitFnType(self) -> Option<(Vec<Type>, Type)> {
        match self {
            Type::Function(args, result) => Some((args, *result)),
            _ => None,
        }
    }

    pub fn collectVars(&self, mut vars: BTreeSet<TypeVar>) -> BTreeSet<TypeVar> {
        match &self {
            Type::Named(_, args, _) => {
                for arg in args {
                    vars = arg.collectVars(vars);
                }
            }
            Type::Tuple(args) => {
                for arg in args {
                    vars = arg.collectVars(vars);
                }
            }
            Type::Function(args, result) => {
                for arg in args {
                    vars = arg.collectVars(vars);
                }
                vars = result.collectVars(vars);
            }
            Type::Var(v) => {
                vars.insert(v.clone());
            }
            Type::Reference(ty, _) => {
                vars = ty.collectVars(vars);
            }
            Type::Ptr(ty) => {
                vars = ty.collectVars(vars);
            }
            Type::SelfType => {}
            Type::Never(_) => {}
        }
        vars
    }

    pub fn collectLifetimes(&self) -> Vec<Lifetime> {
        match &self {
            Type::Named(_, _, lifetimes) => lifetimes.as_ref().expect("lifetime info missing").args.clone(),
            Type::Tuple(_) => Vec::new(),
            Type::Function(_, _) => {
                unreachable!()
            }
            Type::Var(_) => {
                unreachable!()
            }
            Type::Reference(ty, lifetime) => {
                let mut lifetimes = ty.collectLifetimes();
                lifetimes.insert(0, lifetime.expect("no lifetime for ref"));
                lifetimes
            }
            Type::Ptr(_) => Vec::new(),
            Type::SelfType => Vec::new(),
            Type::Never(_) => Vec::new(),
        }
    }

    pub fn changeSelfType(&self, selfType: Type) -> Type {
        match &self {
            Type::Tuple(args) => {
                if args.len() > 0 && args[0] == Type::SelfType {
                    let mut args = args.clone();
                    args.remove(0);
                    args.insert(0, selfType);
                    return Type::Tuple(args);
                }
                Type::Tuple(args.clone())
            }
            Type::SelfType => selfType,
            ty => (*ty).clone(),
        }
    }

    pub fn addSelfType(&self, selfType: Type) -> Type {
        match &self {
            Type::Tuple(args) => {
                let mut args = args.clone();
                args.insert(0, selfType);
                return Type::Tuple(args);
            }
            ty => Type::Tuple(vec![selfType, (*ty).clone()]),
        }
    }

    pub fn hasSelfType(&self) -> bool {
        match &self {
            Type::Tuple(args) => {
                if args.len() > 0 && args[0] == Type::SelfType {
                    return true;
                }
                false
            }
            Type::SelfType => true,
            _ => false,
        }
    }

    pub fn getResult(&self) -> Type {
        match &self {
            Type::Function(_, result) => *result.clone(),
            _ => panic!("not a function!"),
        }
    }

    pub fn getSelflessType(&self, finalValue: bool) -> Type {
        match &self {
            Type::Tuple(args) => {
                assert!(args.len() > 0);
                if args.len() == 2 && finalValue {
                    args[1].clone()
                } else {
                    Type::Tuple(args[1..].to_vec())
                }
            }
            Type::SelfType => Type::Tuple(Vec::new()),
            _ => panic!("type does not have self!"),
        }
    }

    pub fn getTupleTypes(&self) -> Vec<Type> {
        match &self {
            Type::Tuple(args) => args.clone(),
            _ => Vec::new(),
        }
    }

    pub fn isTuple(&self) -> bool {
        match &self {
            Type::Tuple(_) => true,
            _ => false,
        }
    }
    pub fn isNever(&self) -> bool {
        match &self {
            Type::Never(_) => true,
            _ => false,
        }
    }
    pub fn changeMethodResult(&self) -> Type {
        match &self {
            Type::Function(args, result) => Type::Function(args.clone(), Box::new(result.getSelflessType(true))),
            _ => panic!("type is not a function!"),
        }
    }

    pub fn isConcrete(&self) -> bool {
        self.isSpecified(true)
    }

    pub fn isSpecified(&self, fully: bool) -> bool {
        match &self {
            Type::Named(_, args, _) => {
                for arg in args {
                    if !arg.isSpecified(fully) {
                        return false;
                    }
                }
                return true;
            }
            Type::Tuple(args) => {
                for arg in args {
                    if !arg.isSpecified(fully) {
                        return false;
                    }
                }
                return true;
            }
            Type::Function(args, result) => {
                for arg in args {
                    if !arg.isSpecified(fully) {
                        return false;
                    }
                }
                return result.isSpecified(fully);
            }
            Type::Var(TypeVar::Named(_)) => {
                return !fully;
            }
            Type::Var(TypeVar::Var(_)) => {
                return false;
            }
            Type::Reference(ty, _) => {
                return ty.isSpecified(fully);
            }
            Type::Ptr(ty) => {
                return ty.isSpecified(fully);
            }
            Type::SelfType => {
                panic!("self in isSpecified")
            }
            Type::Never(_) => {
                return true;
            }
        }
    }

    pub fn isReference(&self) -> bool {
        match &self {
            Type::Reference(_, _) => true,
            _ => false,
        }
    }

    pub fn isPtr(&self) -> bool {
        match &self {
            Type::Ptr(_) => true,
            _ => false,
        }
    }

    pub fn isGeneric(&self) -> bool {
        match &self {
            Type::Var(_) => true,
            _ => false,
        }
    }

    pub fn makeSingleRef(self) -> Type {
        match self {
            Type::Reference(inner, lifetime) => {
                if inner.isReference() {
                    inner.makeSingleRef()
                } else {
                    Type::Reference(inner, lifetime)
                }
            }
            ty => ty,
        }
    }

    pub fn getBoolType() -> Type {
        Type::Named(getBoolTypeName(), Vec::new(), None)
    }

    pub fn getIntType() -> Type {
        Type::Named(getIntTypeName(), Vec::new(), None)
    }

    pub fn getStringType() -> Type {
        Type::Named(getStringTypeName(), Vec::new(), None)
    }

    pub fn getCharType() -> Type {
        Type::Named(getCharTypeName(), Vec::new(), None)
    }

    pub fn getUnitType() -> Type {
        Type::Tuple(Vec::new())
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Type::Named(name, args, lifetimes) => {
                let lifetimes = match lifetimes {
                    Some(info) => format!("{}", info),
                    None => String::new(),
                };
                if args.is_empty() {
                    write!(f, "{}{}", name, lifetimes)
                } else {
                    let args: Vec<String> = args.iter().map(|t| format!("{}", t)).collect();
                    write!(f, "{}[{}]{}", name, args.join(", "), lifetimes)
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
            Type::Reference(ty, l) => match l {
                Some(l) => write!(f, "&{} {}", l, ty),
                None => write!(f, "&{}", ty),
            },
            Type::Ptr(ty) => {
                write!(f, "*{}", ty)
            }
            Type::SelfType => write!(f, "Self"),
            Type::Never(_) => write!(f, "!"),
        }
    }
}

pub fn formatTypes(types: &Vec<Type>) -> String {
    let types: Vec<String> = types.iter().map(|t| format!("{}", t)).collect();
    format!("({})", types.join(", "))
}
