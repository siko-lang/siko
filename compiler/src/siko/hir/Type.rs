use std::{collections::BTreeSet, fmt::Display, vec};

use crate::siko::{
    hir::{Apply::Apply, Substitution::Substitution, TypeVarAllocator::TypeVarAllocator},
    qualifiedname::{
        builtins::{
            getBoolTypeName, getBoxTypeName, getIntTypeName, getStringLiteralTypeName, getStringTypeName, getU8TypeName,
        },
        QualifiedName,
    },
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
            TypeVar::Var(v) => write!(f, "?{}", v),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Type {
    Named(QualifiedName, Vec<Type>),
    Tuple(Vec<Type>),
    Function(Vec<Type>, Box<Type>),
    Var(TypeVar),
    Reference(Box<Type>),
    Ptr(Box<Type>),
    SelfType,
    Never(bool), // true = explicit never i.e. !
    NumericConstant(String),
    Void,
    VoidPtr,
    Coroutine(Box<Type>, Box<Type>),
}

impl Type {
    pub fn getName(&self) -> Option<QualifiedName> {
        match &self {
            Type::Named(n, _) => Some(n.clone()),
            Type::Reference(ty) => ty.getName(),
            _ => None,
        }
    }

    pub fn unpackRef(self) -> Type {
        match self {
            Type::Reference(ty) => ty.unpackRef(),
            ty => ty.clone(),
        }
    }

    pub fn unpackPtr(self) -> Type {
        match self {
            Type::Ptr(ty) => *ty,
            ty => ty.clone(),
        }
    }

    pub fn splitFnType(self) -> Option<(Vec<Type>, Type)> {
        match self {
            Type::Function(args, result) => Some((args, *result)),
            _ => None,
        }
    }

    pub fn unpackCoroutine(self) -> Option<(Type, Type)> {
        match self {
            Type::Coroutine(yieldTy, retTy) => Some((*yieldTy, *retTy)),
            _ => None,
        }
    }

    pub fn collectVars(&self, mut vars: BTreeSet<TypeVar>) -> BTreeSet<TypeVar> {
        match &self {
            Type::Named(_, args) => {
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
            Type::Reference(ty) => {
                vars = ty.collectVars(vars);
            }
            Type::Ptr(ty) => {
                vars = ty.collectVars(vars);
            }
            Type::SelfType => {}
            Type::Never(_) => {}
            Type::NumericConstant(_) => {}
            Type::Void => {}
            Type::VoidPtr => {}
            Type::Coroutine(yieldTy, retTy) => {
                vars = yieldTy.collectVars(vars);
                vars = retTy.collectVars(vars);
            }
        }
        vars
    }

    pub fn collectVarsStable(&self, mut vars: Vec<TypeVar>) -> Vec<TypeVar> {
        match &self {
            Type::Named(_, args) => {
                for arg in args {
                    vars = arg.collectVarsStable(vars);
                }
            }
            Type::Tuple(args) => {
                for arg in args {
                    vars = arg.collectVarsStable(vars);
                }
            }
            Type::Function(args, result) => {
                for arg in args {
                    vars = arg.collectVarsStable(vars);
                }
                vars = result.collectVarsStable(vars);
            }
            Type::Var(v) => {
                if !vars.contains(v) {
                    vars.push(v.clone());
                }
            }
            Type::Reference(ty) => {
                vars = ty.collectVarsStable(vars);
            }
            Type::Ptr(ty) => {
                vars = ty.collectVarsStable(vars);
            }
            Type::SelfType => {}
            Type::Never(_) => {}
            Type::NumericConstant(_) => {}
            Type::Void => {}
            Type::VoidPtr => {}
            Type::Coroutine(yieldTy, retTy) => {
                vars = yieldTy.collectVarsStable(vars);
                vars = retTy.collectVarsStable(vars);
            }
        }
        vars
    }

    pub fn resolveSelfType(&self) -> Type {
        match &self {
            Type::Function(args, result) => {
                if args.is_empty() {
                    return self.clone();
                }
                let firstArg = args[0].clone();
                let result = result.changeSelfType(firstArg);
                Type::Function(args.clone(), Box::new(result.clone()))
            }
            _ => panic!("type is not a function!"),
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
            _ => Type::Tuple(Vec::new()),
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

    pub fn isNamed(&self) -> bool {
        match &self {
            Type::Named(_, _) => true,
            _ => false,
        }
    }

    pub fn isBoxed(&self, ty: &Type) -> bool {
        match &self {
            Type::Named(name, args) => {
                if name == &getBoxTypeName() && args.len() == 1 && args[0] == *ty {
                    return true;
                }
                false
            }
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
            Type::Named(_, args) => {
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
            Type::Reference(ty) => {
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
            Type::NumericConstant(_) => {
                return true;
            }
            Type::Void => {
                return true;
            }
            Type::VoidPtr => {
                return true;
            }
            Type::Coroutine(yieldTy, retTy) => {
                return yieldTy.isSpecified(fully) && retTy.isSpecified(fully);
            }
        }
    }

    pub fn isReference(&self) -> bool {
        match &self {
            Type::Reference(_) => true,
            _ => false,
        }
    }

    pub fn isUnit(&self) -> bool {
        match &self {
            Type::Tuple(args) => args.is_empty(),
            _ => false,
        }
    }

    pub fn isVoid(&self) -> bool {
        match &self {
            Type::Void => true,
            _ => false,
        }
    }

    pub fn isVoidPtr(&self) -> bool {
        match &self {
            Type::VoidPtr => true,
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

    pub fn isTypeVar(&self) -> bool {
        match &self {
            Type::Var(TypeVar::Var(_)) => true,
            _ => false,
        }
    }

    pub fn makeSingleRef(self) -> Type {
        match self {
            Type::Reference(inner) => {
                if inner.isReference() {
                    inner.makeSingleRef()
                } else {
                    Type::Reference(inner)
                }
            }
            ty => ty,
        }
    }

    pub fn asRef(&self) -> Type {
        Type::Reference(Box::new(self.clone()))
    }

    pub fn asPtr(&self) -> Type {
        Type::Ptr(Box::new(self.clone()))
    }

    pub fn getBoxedType(&self) -> Type {
        Type::Named(getBoxTypeName(), vec![self.clone()])
    }

    pub fn getBoolType() -> Type {
        Type::Named(getBoolTypeName(), Vec::new())
    }

    pub fn getIntType() -> Type {
        Type::Named(getIntTypeName(), Vec::new())
    }

    pub fn getU8Type() -> Type {
        Type::Named(getU8TypeName(), Vec::new())
    }

    pub fn getStringType() -> Type {
        Type::Named(getStringTypeName(), Vec::new())
    }

    pub fn getStringLiteralType() -> Type {
        Type::Named(getStringLiteralTypeName(), Vec::new())
    }

    pub fn getUnitType() -> Type {
        Type::Tuple(Vec::new())
    }

    pub fn getNeverType() -> Type {
        Type::Never(false)
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
            Type::Reference(ty) => write!(f, "&{}", ty),
            Type::Ptr(ty) => {
                write!(f, "*{}", ty)
            }
            Type::SelfType => write!(f, "Self"),
            Type::Never(_) => write!(f, "!"),
            Type::NumericConstant(value) => write!(f, "{}", value),
            Type::Void => write!(f, "void"),
            Type::VoidPtr => write!(f, "void*"),
            Type::Coroutine(yieldTy, retTy) => write!(f, "co({}) -> {}", yieldTy, retTy),
        }
    }
}

pub fn formatTypes(types: &Vec<Type>) -> String {
    let types: Vec<String> = types.iter().map(|t| format!("{}", t)).collect();
    format!("({})", types.join(", "))
}

pub fn normalizeTypes(types: &Vec<Type>) -> Vec<Type> {
    let (types, _) = normalizeTypesWithSub(types);
    types
}

pub fn normalizeTypesWithSub(types: &Vec<Type>) -> (Vec<Type>, Substitution) {
    let allocator = TypeVarAllocator::new();
    let mut vars = Vec::new();
    for ty in types {
        vars = ty.collectVarsStable(vars);
    }
    let mut sub = Substitution::new();
    for var in &vars {
        sub.add(Type::Var(var.clone()), allocator.nextNamed());
    }
    (types.iter().map(|ty| ty.clone().apply(&sub)).collect(), sub)
}
