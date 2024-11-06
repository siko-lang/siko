use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{Debug, Display},
};

use crate::siko::hir::Type::Type;

use super::{
    Data::{Class, Enum, Field, Variant},
    TypeVarAllocator::TypeVarAllocator,
    Unification::unify,
};

#[derive(Debug)]
pub struct Substitution<T> {
    substitutions: BTreeMap<T, T>,
}

impl<T: Ord + Debug> Substitution<T> {
    pub fn new() -> Substitution<T> {
        Substitution {
            substitutions: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, old: T, new: T) {
        assert_ne!(old, new);
        self.substitutions.insert(old, new);
    }
}

impl<T: Display> Display for Substitution<T> {
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

pub trait Apply<T> {
    fn apply(&self, sub: &Substitution<T>) -> Self;
}

impl Apply<Type> for Type {
    fn apply(&self, sub: &Substitution<Type>) -> Self {
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

impl<I, T: Apply<I>> Apply<I> for Vec<T> {
    fn apply(&self, sub: &Substitution<I>) -> Self {
        self.iter().map(|i| i.apply(sub)).collect()
    }
}

impl Apply<Type> for Variant {
    fn apply(&self, sub: &Substitution<Type>) -> Self {
        let mut v = self.clone();
        v.items = v.items.apply(sub);
        v
    }
}

impl Apply<Type> for Enum {
    fn apply(&self, sub: &Substitution<Type>) -> Self {
        let mut e = self.clone();
        e.ty = e.ty.apply(sub);
        e.variants = e.variants.apply(sub);
        e
    }
}

impl Apply<Type> for Field {
    fn apply(&self, sub: &Substitution<Type>) -> Self {
        let mut f = self.clone();
        f.ty = f.ty.apply(sub);
        f
    }
}

impl Apply<Type> for Class {
    fn apply(&self, sub: &Substitution<Type>) -> Self {
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
    let r = unify(&mut sub, ty, &e.ty);
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
    let r = unify(&mut sub, ty, &e.ty);
    assert!(r.is_ok());
    e.apply(&sub)
}
