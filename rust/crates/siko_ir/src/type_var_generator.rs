use crate::types::Type;
use siko_util::RcCounter;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeVarGenerator {
    counter: RcCounter,
}

impl TypeVarGenerator {
    pub fn new(counter: RcCounter) -> TypeVarGenerator {
        TypeVarGenerator { counter: counter }
    }

    pub fn get_new_index(&mut self) -> usize {
        self.counter.next()
    }

    pub fn get_new_type_var(&mut self) -> Type {
        Type::Var(self.counter.next(), Vec::new())
    }
}
