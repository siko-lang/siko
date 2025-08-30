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
    ty: Option<Type>,
}

impl VariableInfo {
    pub fn new(name: VariableName, ty: Option<Type>) -> VariableInfo {
        VariableInfo { name, ty }
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
pub enum VariableKind {
    Definition,
    Usage,
}

impl Display for VariableKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableKind::Definition => write!(f, "d"),
            VariableKind::Usage => write!(f, "u"),
        }
    }
}

impl Debug for VariableKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone, Eq)]
pub struct Variable {
    kind: VariableKind,
    info: Rc<RefCell<VariableInfo>>,
    location: Location,
}

impl Variable {
    pub fn new(name: VariableName, location: Location) -> Variable {
        Variable {
            kind: VariableKind::Definition,
            info: Rc::new(RefCell::new(VariableInfo::new(name, None))),
            location,
        }
    }

    pub fn newWithType(name: VariableName, location: Location, ty: Type) -> Variable {
        Variable {
            kind: VariableKind::Definition,
            info: Rc::new(RefCell::new(VariableInfo::new(name, Some(ty)))),
            location,
        }
    }

    pub fn cloneInto(&self, name: VariableName) -> Variable {
        Variable {
            kind: self.kind.clone(),
            info: Rc::new(RefCell::new(VariableInfo::new(name, self.getTypeOpt()))),
            location: self.location(),
        }
    }

    pub fn cloneNew(&self) -> Variable {
        Variable {
            kind: self.kind.clone(),
            info: Rc::new(RefCell::new(VariableInfo::new(self.name(), self.getTypeOpt()))),
            location: self.location(),
        }
    }

    pub fn withLocation(&self, location: Location) -> Variable {
        Variable {
            kind: self.kind.clone(),
            info: self.info.clone(),
            location,
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
            ty: Some(Type::getBoolType()),
        };
        Variable {
            kind: VariableKind::Definition,
            info: Rc::new(RefCell::new(info)),
            location: self.location(),
        }
    }

    pub fn isUsage(&self) -> bool {
        self.kind == VariableKind::Usage
    }

    pub fn toPath(&self) -> Path {
        Path::new(self.clone(), self.location())
    }

    pub fn setType(&self, ty: Type) {
        self.info.borrow_mut().setType(ty);
    }

    pub fn location(&self) -> Location {
        self.location.clone()
    }

    pub fn name(&self) -> VariableName {
        self.info.borrow().name.clone()
    }

    pub fn useVar(&self) -> Variable {
        Variable {
            kind: VariableKind::Usage,
            info: self.info.clone(),
            location: self.location.clone(),
        }
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ty) = &self.getTypeOpt() {
            write!(f, "${}/{}: {}", self.name(), self.kind, ty)
        } else {
            write!(f, "${}/{}", self.name(), self.kind)
        }
    }
}

impl std::fmt::Debug for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl PartialEq for Variable {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.info, &other.info)
    }
}

impl PartialOrd for Variable {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Rc::as_ptr(&self.info).partial_cmp(&Rc::as_ptr(&other.info))
    }
}

impl Ord for Variable {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Rc::as_ptr(&self.info).cmp(&Rc::as_ptr(&other.info))
    }
}
