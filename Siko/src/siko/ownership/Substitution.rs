use std::collections::BTreeMap;

use crate::siko::ir::{
    Lifetime::{Lifetime, LifetimeInfo},
    Type::Type,
};

pub struct Substitution {
    vars: BTreeMap<Lifetime, Lifetime>,
}

impl Substitution {
    pub fn new() -> Substitution {
        Substitution {
            vars: BTreeMap::new(),
        }
    }

    pub fn from(ty1: Type, ty2: Type) -> Substitution {
        let mut sub = Substitution::new();
        let lifetimes1 = ty1.collectLifetimes();
        let lifetimes2 = ty2.collectLifetimes();
        for (l1, l2) in lifetimes1.iter().zip(lifetimes2.iter()) {
            sub.add(l1, l2);
        }
        sub
    }

    fn add(&mut self, from: &Lifetime, to: &Lifetime) {
        let to = self.apply(to);
        self.vars.insert(from.clone(), to);
    }

    fn apply(&self, l: &Lifetime) -> Lifetime {
        let mut current = l;
        loop {
            match self.vars.get(&l) {
                Some(l) => current = l,
                None => return *current,
            }
        }
    }
}

pub trait Apply {
    fn apply(&self, sub: &Substitution) -> Self;
}

impl<T: Apply> Apply for Option<T> {
    fn apply(&self, sub: &Substitution) -> Self {
        match self {
            Some(t) => Some(t.apply(sub)),
            None => None,
        }
    }
}

impl<T: Apply> Apply for Vec<T> {
    fn apply(&self, sub: &Substitution) -> Self {
        self.iter().map(|i| i.apply(sub)).collect()
    }
}

impl Apply for Lifetime {
    fn apply(&self, sub: &Substitution) -> Self {
        sub.apply(self)
    }
}

impl Apply for LifetimeInfo {
    fn apply(&self, sub: &Substitution) -> Self {
        let mut new = self.clone();
        new.args = new.args.apply(sub);
        new
    }
}

impl Apply for Type {
    fn apply(&self, sub: &Substitution) -> Self {
        match self {
            Type::Named(qualified_name, args, lifetime_info) => Type::Named(
                qualified_name.clone(),
                args.clone(),
                lifetime_info.apply(sub),
            ),
            Type::Tuple(args) => Type::Tuple(args.apply(sub)),
            Type::Function(_, _) => unreachable!(),
            Type::Var(_) => unreachable!(),
            Type::Reference(ty, lifetime) => {
                Type::Reference(Box::new(ty.apply(sub)), lifetime.apply(sub))
            }
            Type::SelfType => Type::SelfType,
            Type::Never => Type::Never,
        }
    }
}
