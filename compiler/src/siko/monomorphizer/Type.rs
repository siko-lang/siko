use crate::siko::{
    hir::{Apply::Apply, Substitution::Substitution, Type::Type},
    monomorphizer::{Monomorphizer::Monomorphizer, Utils::Monomorphize},
};

impl Monomorphize for Type {
    fn process(&self, sub: &Substitution, mono: &mut Monomorphizer) -> Self {
        let ty = self.clone().apply(sub);
        mono.processType(ty)
    }
}
