use crate::siko::hir::{Apply::Apply, Substitution::Substitution};

pub struct SubstitutionChain {
    pub substitutions: Vec<Substitution>,
}

impl SubstitutionChain {
    pub fn new() -> Self {
        SubstitutionChain {
            substitutions: Vec::new(),
        }
    }

    pub fn add(&mut self, substitution: Substitution) {
        self.substitutions.push(substitution);
    }

    pub fn apply<T: Apply>(&self, value: T) -> T {
        let mut result = value;
        for s in self.substitutions.iter() {
            result = result.apply(s);
        }
        result
    }
}
