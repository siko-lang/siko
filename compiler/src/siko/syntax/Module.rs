use crate::siko::syntax::Data::*;
use crate::siko::syntax::Function::*;
use crate::siko::syntax::Identifier::*;

use super::Trait::Instance;
use super::Trait::Trait;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Derive {
    pub name: Identifier,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ModuleItem {
    Struct(Struct),
    Enum(Enum),
    Function(Function),
    Import(Import),
    Trait(Trait),
    Instance(Instance),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Import {
    pub moduleName: Identifier,
    pub alias: Option<Identifier>,
    pub implicitImport: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Module {
    pub name: Identifier,
    pub items: Vec<ModuleItem>,
}
