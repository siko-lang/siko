use std::collections::BTreeMap;

use crate::siko::hir::{
    Data::{Class, Field},
    Function::InstructionId,
    Lifetime::{Lifetime, LifetimeInfo},
    Type::Type,
};

use super::DataFlow::{
    DataFlowProfile::DataFlowProfile, FunctionInferenceData::FunctionInferenceData,
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

    pub fn from(ty1: &Type, ty2: &Type) -> Substitution {
        let mut sub = Substitution::new();
        let lifetimes1 = ty1.collectLifetimes();
        let lifetimes2 = ty2.collectLifetimes();
        for (l1, l2) in lifetimes1.iter().zip(lifetimes2.iter()) {
            sub.add(l1, l2);
        }
        sub
    }

    pub fn add(&mut self, from: &Lifetime, to: &Lifetime) {
        let from = self.apply(from);
        let to = self.apply(to);
        if to == from {
            return;
        }
        self.vars.insert(from.clone(), to);
    }

    pub fn apply(&self, l: &Lifetime) -> Lifetime {
        let mut current = l;
        loop {
            match self.vars.get(&current) {
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

impl<K: Apply + Ord, V: Apply> Apply for BTreeMap<K, V> {
    fn apply(&self, sub: &Substitution) -> Self {
        let mut result = BTreeMap::new();
        for (key, value) in self {
            let key = key.apply(sub);
            let value = value.apply(sub);
            result.insert(key, value);
        }
        result
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

impl Apply for DataFlowProfile {
    fn apply(&self, sub: &Substitution) -> Self {
        let mut p = self.clone();
        p.args = p.args.apply(sub);
        p.result = p.result.apply(sub);
        p.deps = p.deps.apply(sub);
        p
    }
}

impl Apply for InstructionId {
    fn apply(&self, _: &Substitution) -> Self {
        self.clone()
    }
}

impl Apply for String {
    fn apply(&self, _: &Substitution) -> Self {
        self.clone()
    }
}

impl Apply for FunctionInferenceData {
    fn apply(&self, sub: &Substitution) -> Self {
        let mut d = self.clone();
        d.profile = d.profile.apply(sub);
        d.instruction_types = d.instruction_types.apply(sub);
        d.value_types = d.value_types.apply(sub);
        d
    }
}
