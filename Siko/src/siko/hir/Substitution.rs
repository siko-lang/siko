use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    iter::zip,
};

use crate::siko::hir::Type::{Type, TypeVar};

use super::{
    Data::{Class, Enum, Field, Variant},
    TypeVarAllocator::TypeVarAllocator,
};

#[derive(Debug)]
pub struct Substitution {
    substitutions: BTreeMap<Type, Type>,
}

#[derive(Debug)]
pub struct Error {}

impl Substitution {
    pub fn new() -> Substitution {
        Substitution {
            substitutions: BTreeMap::new(),
        }
    }

    pub fn create(ty1: &Type, ty2: &Type) -> Substitution {
        let mut sub = Substitution::new();
        sub.unify(ty1, ty2).expect("Unification failed");
        sub
    }

    pub fn createFrom(ty1: &Vec<Type>, ty2: &Vec<Type>) -> Substitution {
        let mut sub = Substitution::new();
        for (ty1, ty2) in ty1.iter().zip(ty2) {
            sub.unify(ty1, ty2).expect("Unification failed");
        }
        sub
    }

    pub fn add(&mut self, var: Type, ty: Type) {
        assert_ne!(var, ty);
        self.substitutions.insert(var, ty);
    }

    pub fn unify(&mut self, ty1: &Type, ty2: &Type) -> Result<(), Error> {
        //println!("Unifying {}/{}", ty1, ty2);
        let ty1 = ty1.apply(self);
        let ty2 = ty2.apply(self);
        //println!("Unifying2 {}/{}", ty1, ty2);
        match (&ty1, &ty2) {
            (Type::Named(name1, args1, _), Type::Named(name2, args2, _)) => {
                if name1 != name2 {
                    return Err(Error {});
                } else {
                    for (arg1, arg2) in zip(args1, args2) {
                        self.unify(arg1, arg2)?;
                    }
                    Ok(())
                }
            }
            (Type::Tuple(args1), Type::Tuple(args2)) => {
                if args1.len() != args2.len() {
                    return Err(Error {});
                } else {
                    for (arg1, arg2) in zip(args1, args2) {
                        self.unify(arg1, arg2)?;
                    }
                    Ok(())
                }
            }
            (Type::Var(TypeVar::Named(n1)), Type::Var(TypeVar::Named(n2))) => {
                if n1 == n2 {
                    return Ok(());
                } else {
                    return Err(Error {});
                }
            }
            (Type::Var(TypeVar::Var(v1)), Type::Var(TypeVar::Var(v2))) if v1 == v2 => Ok(()),
            (_, Type::Var(_)) => {
                self.add(ty2, ty1);
                Ok(())
            }
            (Type::Var(_), _) => {
                self.add(ty1, ty2);
                Ok(())
            }
            (Type::Reference(v1, _), Type::Reference(v2, _)) => self.unify(&v1, &v2),
            (Type::Never, _) => Ok(()),
            (_, Type::Never) => Ok(()),
            (Type::Function(args1, res1), Type::Function(args2, res2)) => {
                for (arg1, arg2) in zip(args1, args2) {
                    self.unify(arg1, arg2)?;
                }
                return self.unify(&res1, &res2);
            }
            _ => return Err(Error {}),
        }
    }
}

impl Display for Substitution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, (key, value)) in self.substitutions.iter().enumerate() {
            if index == 0 {
                write!(f, "{}: {}", key, value)?;
            } else {
                write!(f, ", {}: {}", key, value)?;
            }
        }
        Ok(())
    }
}

pub trait Apply {
    fn apply(&self, sub: &Substitution) -> Self;
}

impl Apply for Type {
    fn apply(&self, sub: &Substitution) -> Self {
        match &self {
            Type::Named(n, args, lifetime) => {
                let newArgs = args.iter().map(|arg| arg.apply(sub)).collect();
                Type::Named(n.clone(), newArgs, lifetime.clone())
            }
            Type::Tuple(args) => {
                let newArgs = args.iter().map(|arg| arg.apply(sub)).collect();
                Type::Tuple(newArgs)
            }
            Type::Function(args, fnResult) => {
                let newArgs = args.iter().map(|arg| arg.apply(sub)).collect();
                let newFnResult = fnResult.apply(sub);
                Type::Function(newArgs, Box::new(newFnResult))
            }
            Type::Var(_) => match sub.substitutions.get(&self) {
                Some(ty) => ty.apply(sub),
                None => self.clone(),
            },
            Type::Reference(arg, l) => Type::Reference(Box::new(arg.apply(sub)), l.clone()),
            Type::SelfType => self.clone(),
            Type::Never => self.clone(),
        }
    }
}

impl<T: Apply> Apply for Vec<T> {
    fn apply(&self, sub: &Substitution) -> Self {
        self.iter().map(|i| i.apply(sub)).collect()
    }
}

impl Apply for Variant {
    fn apply(&self, sub: &Substitution) -> Self {
        let mut v = self.clone();
        v.items = v.items.apply(sub);
        v
    }
}

impl Apply for Enum {
    fn apply(&self, sub: &Substitution) -> Self {
        let mut e = self.clone();
        e.ty = e.ty.apply(sub);
        e.variants = e.variants.apply(sub);
        e
    }
}

impl Apply for Field {
    fn apply(&self, sub: &Substitution) -> Self {
        let mut f = self.clone();
        f.ty = f.ty.apply(sub);
        f
    }
}

impl Apply for Class {
    fn apply(&self, sub: &Substitution) -> Self {
        let mut c = self.clone();
        c.ty = c.ty.apply(sub);
        c.fields = c.fields.apply(sub);
        c
    }
}

pub fn instantiateEnum(allocator: &mut TypeVarAllocator, e: &Enum, ty: &Type) -> Enum {
    let vars = e.ty.collectVars(BTreeSet::new());
    let mut sub = Substitution::new();
    for var in &vars {
        sub.add(Type::Var(var.clone()), allocator.next());
    }
    let mut e = e.clone();
    e = e.apply(&sub);
    let r = sub.unify(ty, &e.ty);
    assert!(r.is_ok());
    e.apply(&sub)
}

pub fn instantiateClass(allocator: &mut TypeVarAllocator, c: &Class, ty: &Type) -> Class {
    let vars = c.ty.collectVars(BTreeSet::new());
    let mut sub = Substitution::new();
    for var in &vars {
        sub.add(Type::Var(var.clone()), allocator.next());
    }
    let mut e = c.clone();
    e = e.apply(&sub);
    let r = sub.unify(ty, &e.ty);
    assert!(r.is_ok());
    e.apply(&sub)
}
