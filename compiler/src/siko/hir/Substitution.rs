use std::{
    collections::BTreeMap,
    fmt::{Debug, Display},
};

use super::{Apply::Apply, Type::Type, Unification::unify};

#[derive(Debug)]
pub struct Substitution {
    substitutions: BTreeMap<Type, Type>,
}

impl Substitution {
    pub fn new() -> Substitution {
        Substitution {
            substitutions: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, old: Type, new: Type) {
        assert_ne!(old, new);
        self.substitutions.insert(old, new);
    }

    pub fn get(&self, old: Type) -> Type {
        match self.substitutions.get(&old) {
            Some(new) => new.clone().apply(self),
            None => old,
        }
    }
}

impl Display for Substitution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        print!("[");
        for (index, (key, value)) in self.substitutions.iter().enumerate() {
            if index == 0 {
                write!(f, "{}: {}", key, value)?;
            } else {
                write!(f, ", {}: {}", key, value)?;
            }
        }
        print!("]");
        Ok(())
    }
}

pub fn createTypeSubstitutionFrom(ty1: Vec<Type>, ty2: Vec<Type>) -> Substitution {
    let mut sub = Substitution::new();
    for (ty1, ty2) in ty1.into_iter().zip(ty2) {
        unify(&mut sub, ty1, ty2, true).expect("Unification failed");
    }
    sub
}
