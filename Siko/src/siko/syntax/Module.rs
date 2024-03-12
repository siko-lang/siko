use crate::siko::syntax::Data::*;
use crate::siko::syntax::Function::*;
use crate::siko::syntax::Identifier::*;

use super::Trait::Instance;
use super::Trait::Trait;

pub struct Derive {
    pub name: Identifier,
}

pub enum ModuleItem {
    Class(Class),
    Enum(Enum),
    Function(Function),
    Import(Import),
    Trait(Trait),
    Instance(Instance),
}

pub struct Import {
    pub moduleName: Identifier,
    pub alias: Option<Identifier>,
    pub implicitImport: bool,
}

pub struct Module {
    pub name: Identifier,
    pub items: Vec<ModuleItem>,
}
