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

    pub fn copy(&self) -> Self {
        VariableAllocator {
            nextId: Rc::new(RefCell::new(*self.nextId.borrow())),
        }
    }

    pub fn allocate(&self, location: Location) -> Variable {
        let id = {
            let mut nextId = self.nextId.borrow_mut();
            let id = *nextId;
            *nextId += 1;
            id
        };
        Variable::new(VariableName::Tmp(id), location)
    }

    pub fn allocateWithType(&self, location: Location, ty: super::Type::Type) -> Variable {
        let var = self.allocate(location);
        var.setType(ty);
        var
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
