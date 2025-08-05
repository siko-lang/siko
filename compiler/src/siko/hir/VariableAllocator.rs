use std::{cell::RefCell, fmt::Debug, fmt::Display, rc::Rc};

use crate::siko::{
    hir::Variable::{Variable, VariableName},
    location::Location::Location,
};

#[derive(Clone)]
pub struct VariableAllocator {
    nextId: Rc<RefCell<u32>>,
}

impl VariableAllocator {
    pub fn new() -> Self {
        VariableAllocator {
            nextId: Rc::new(RefCell::new(0)),
        }
    }

    pub fn allocate(&self, location: Location) -> Variable {
        let id = {
            let mut nextId = self.nextId.borrow_mut();
            let id = *nextId;
            *nextId += 1;
            id
        };
        let id = Variable {
            value: VariableName::Tmp(id),
            ty: None,
            location: location,
        };
        id
    }
}

impl Debug for VariableAllocator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for VariableAllocator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VariableAllocator(nextId={})", self.nextId.borrow())
    }
}
