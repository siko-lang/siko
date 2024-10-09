use crate::siko::hir::Type::{Type, TypeVar};

pub struct TypeVarAllocator {
    next: u64,
}

impl TypeVarAllocator {
    pub fn new() -> TypeVarAllocator {
        TypeVarAllocator { next: 0 }
    }

    pub fn next(&mut self) -> Type {
        let v = Type::Var(TypeVar::Var(self.next));
        self.next += 1;
        v
    }
}
