use crate::siko::syntax::Data::*;
use crate::siko::syntax::Effect::Effect;
use crate::siko::syntax::Function::*;
use crate::siko::syntax::Identifier::*;
use crate::siko::syntax::Implicit::Implicit;
use crate::siko::syntax::Trait::Implementation;
use crate::siko::syntax::Trait::Protocol;

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
    Effect(Effect),
    Implicit(Implicit),
    Protocol(Protocol),
    Implementation(Implementation),
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
