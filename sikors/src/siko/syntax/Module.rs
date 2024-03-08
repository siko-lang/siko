use crate::siko::syntax::Data::*;
use crate::siko::syntax::Function::*;
use crate::siko::syntax::Identifier::*;

pub enum ModuleItem {
    Class(Class),
    Enum(Enum),
    Function(Function),
    Import(Import),
}

pub struct Import {
    pub moduleName: Identifier,
    pub alias: Option<Identifier>,
}

pub struct Module {
    pub name: Identifier,
    pub items: Vec<ModuleItem>,
}
