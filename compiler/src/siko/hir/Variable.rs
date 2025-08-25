use std::cell::RefCell;
use std::fmt::Debug;
use std::fmt::Display;
use std::rc::Rc;

use crate::siko::backend::drop::Path::Path;
use crate::siko::location::Location::Location;

use super::Type::Type;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum VariableName {
    Tmp(u32),
    Local(String, u32),
    Arg(String),
    DropFlag(String),
}

impl VariableName {
    pub fn visibleName(&self) -> String {
        match self {
            VariableName::Tmp(i) => format!("tmp{}", i),
            VariableName::Local(n, _) => n.clone(),
            VariableName::Arg(n) => format!("arg_{}", n),
            VariableName::DropFlag(n) => format!("drop_flag_{}", n),
        }
    }
    pub fn isTemp(&self) -> bool {
        match self {
            VariableName::Tmp(_) => true,
            _ => false,
        }
    }

    pub fn isDropFlag(&self) -> bool {
        match self {
            VariableName::DropFlag(_) => true,
            _ => false,
        }
    }

    pub fn isArg(&self) -> bool {
        match self {
            VariableName::Arg(_) => true,
            _ => false,
        }
    }

    pub fn getDropFlag(&self) -> VariableName {
        VariableName::DropFlag(self.to_string())
    }
}

impl Display for VariableName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableName::Tmp(i) => write!(f, "tmp{}", i),
            VariableName::Local(n, i) => write!(f, "{}_{}", n, i),
            VariableName::Arg(n) => write!(f, "{}", n),
            VariableName::DropFlag(n) => write!(f, "drop_flag_{}", n),
        }
    }
}

impl Debug for VariableName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableName::Tmp(i) => write!(f, "tmp{}", i),
            VariableName::Local(n, i) => write!(f, "{}_{}", n, i),
            VariableName::Arg(n) => write!(f, "{}", n),
            VariableName::DropFlag(n) => write!(f, "drop_flag_{}", n),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct VariableInfo {
    name: VariableName,
    location: Location,
    ty: Option<Type>,
}

impl VariableInfo {
    pub fn new(name: VariableName, location: Location, ty: Option<Type>) -> VariableInfo {
        VariableInfo { name, location, ty }
    }

    pub fn getType(&self) -> Type {
        match &self.ty {
            Some(ty) => ty.clone(),
            None => panic!("No type found for var {}", self.name),
        }
    }

    pub fn setType(&mut self, ty: Type) {
        self.ty = Some(ty);
    }
}

impl Debug for VariableInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.name, self.ty)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Variable {
    info: Rc<RefCell<VariableInfo>>,
}

impl Variable {
    pub fn new(name: VariableName, location: Location) -> Variable {
        Variable {
            info: Rc::new(RefCell::new(VariableInfo::new(name, location, None))),
        }
    }

    pub fn newWithType(name: VariableName, location: Location, ty: Type) -> Variable {
        Variable {
            info: Rc::new(RefCell::new(VariableInfo::new(name, location, Some(ty)))),
        }
    }

    pub fn cloneInto(&self, name: VariableName) -> Variable {
        Variable {
            info: Rc::new(RefCell::new(VariableInfo::new(
                name,
                self.location(),
                self.getTypeOpt(),
            ))),
        }
    }

    pub fn withLocation(&self, location: Location) -> Variable {
        Variable {
            info: Rc::new(RefCell::new(VariableInfo::new(
                self.name(),
                location,
                self.getTypeOpt(),
            ))),
        }
    }

    pub fn getType(&self) -> Type {
        self.info.borrow().getType()
    }

    pub fn getTypeOpt(&self) -> Option<Type> {
        self.info.borrow().ty.clone()
    }

    pub fn replace(&self, from: &Variable, to: Variable) -> Variable {
        if self == from {
            to
        } else {
            self.clone()
        }
    }

    pub fn isTemp(&self) -> bool {
        self.name().isTemp()
    }

    pub fn isDropFlag(&self) -> bool {
        self.name().isDropFlag()
    }

    pub fn isArg(&self) -> bool {
        self.name().isArg()
    }

    pub fn getDropFlag(&self) -> Variable {
        let info = VariableInfo {
            name: self.name().getDropFlag(),
            location: self.location(),
            ty: Some(Type::getBoolType()),
        };
        Variable {
            info: Rc::new(RefCell::new(info)),
        }
    }

    pub fn toPath(&self) -> Path {
        Path::new(self.clone(), self.location())
    }

    pub fn setType(&mut self, ty: Type) {
        self.info.borrow_mut().setType(ty);
    }

    pub fn location(&self) -> Location {
        self.info.borrow().location.clone()
    }

    pub fn name(&self) -> VariableName {
        self.info.borrow().name.clone()
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ty) = &self.getTypeOpt() {
            write!(f, "${}: {}", self.name(), ty)
        } else {
            write!(f, "${}", self.name())
        }
    }
}

impl std::fmt::Debug for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
