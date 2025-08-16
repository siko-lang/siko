use std::collections::BTreeMap;

use crate::siko::{
    hir::{Substitution::Substitution, Type::Type, Unification::unify},
    monomorphizer::Monomorphizer::Monomorphizer,
};

pub trait Monomorphize {
    fn process(&self, sub: &Substitution, mono: &mut Monomorphizer) -> Self;
}

impl<T: Monomorphize> Monomorphize for Option<T> {
    fn process(&self, sub: &Substitution, mono: &mut Monomorphizer) -> Self {
        match self {
            Some(v) => Some(v.process(sub, mono)),
            None => None,
        }
    }
}

impl<T: Monomorphize> Monomorphize for Vec<T> {
    fn process(&self, sub: &Substitution, mono: &mut Monomorphizer) -> Self {
        self.iter().map(|i| i.process(sub, mono)).collect()
    }
}

impl<T: Monomorphize, K: Ord + Clone> Monomorphize for BTreeMap<K, T> {
    fn process(&self, sub: &Substitution, mono: &mut Monomorphizer) -> Self {
        self.iter().map(|(k, v)| (k.clone(), v.process(sub, mono))).collect()
    }
}

pub fn createTypeSubstitution(ty1: Type, ty2: Type) -> Substitution {
    let mut sub = Substitution::new();
    if unify(&mut sub, ty1.clone(), ty2.clone(), true).is_err() {
        panic!("Unification failed for {} {}", ty1, ty2);
    }
    sub
}
