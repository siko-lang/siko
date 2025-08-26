use std::{cell::RefCell, rc::Rc};

use crate::siko::hir::Type::{Type, TypeVar};

#[derive(Clone)]
pub struct TypeVarAllocator {
    next: Rc<RefCell<u64>>,
}

impl TypeVarAllocator {
    pub fn new() -> TypeVarAllocator {
        TypeVarAllocator {
            next: Rc::new(RefCell::new(0)),
        }
    }

    pub fn next(&self) -> Type {
        let mut n = self.next.borrow_mut();
        let v = Type::Var(TypeVar::Var(*n));
        *n += 1;
        v
    }

    pub fn nextNamed(&self) -> Type {
        let mut n = self.next.borrow_mut();
        let v = Type::Var(TypeVar::Named(format!("T{}", *n)));
        *n += 1;
        v
    }
}
