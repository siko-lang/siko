use crate::siko::syntax::Attributes::Attributes;
use crate::siko::syntax::Data::*;
use crate::siko::syntax::Effect::Effect;
use crate::siko::syntax::Function::*;
use crate::siko::syntax::Global::Global;
use crate::siko::syntax::Identifier::*;
use crate::siko::syntax::Implicit::Implicit;
use crate::siko::syntax::Trait::Instance;
use crate::siko::syntax::Trait::Trait;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Derive {
    pub name: Identifier,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ModuleItem {
    Struct(Struct),
    Enum(Enum),
    Function(Function),
    Global(Global),
    Import(Import),
    Effect(Effect),
    Implicit(Implicit),
    Trait(Trait),
    Instance(Instance),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Import {
    pub moduleName: Identifier,
    pub alias: Option<Identifier>,
    pub implicitImport: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Module {
    pub name: Identifier,
    pub items: Vec<ModuleItem>,
    pub attributes: Attributes,
}
