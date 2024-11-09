use std::{collections::BTreeSet, fmt::Display};

use crate::siko::qualifiedname::{getBoolTypeName, getCharTypeName, getIntTypeName, getStringTypeName, QualifiedName};

use super::{
    Lifetime::{Lifetime, LifetimeInfo},
    Substitution::Substitution,
    Unification::unify,
};

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
    SelfType,
    Never,
}

impl Type {
    pub fn getName(&self) -> Option<QualifiedName> {
        match &self {
            Type::Named(n, _, _) => Some(n.clone()),
            Type::Reference(ty, _) => ty.getName(),
            _ => None,
        }
    }

    pub fn unpackRef(self) -> Type {
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
            Type::SelfType => {}
            Type::Never => {}
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
            Type::SelfType => Vec::new(),
            Type::Never => Vec::new(),
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

    pub fn isConcrete(&self) -> bool {
        match &self {
            Type::Named(_, args, _) => {
                for arg in args {
                    if !arg.isConcrete() {
                        return false;
                    }
                }
                return true;
            }
            Type::Tuple(args) => {
                for arg in args {
                    if !arg.isConcrete() {
                        return false;
                    }
                }
                return true;
            }
            Type::Function(args, result) => {
                for arg in args {
                    if !arg.isConcrete() {
                        return false;
                    }
                }
                return result.isConcrete();
            }
            Type::Var(_) => {
                return false;
            }
            Type::Reference(ty, _) => {
                return ty.isConcrete();
            }
            Type::SelfType => {
                panic!("self in isConcrete")
            }
            Type::Never => {
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
            Type::SelfType => write!(f, "Self"),
            Type::Never => write!(f, "!"),
        }
    }
}

pub fn formatTypes(types: &Vec<Type>) -> String {
    let types: Vec<String> = types.iter().map(|t| format!("{}", t)).collect();
    format!("({})", types.join(", "))
}

pub fn createTypeSubstitution(ty1: &Type, ty2: &Type) -> Substitution<Type> {
    let mut sub = Substitution::new();
    unify(&mut sub, ty1, ty2).expect("Unification failed");
    sub
}

pub fn createTypeSubstitutionFrom(ty1: &Vec<Type>, ty2: &Vec<Type>) -> Substitution<Type> {
    let mut sub = Substitution::new();
    for (ty1, ty2) in ty1.iter().zip(ty2) {
        unify(&mut sub, ty1, ty2).expect("Unification failed");
    }
    sub
}
