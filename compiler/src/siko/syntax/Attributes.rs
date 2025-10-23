#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Safety {
    Safe,
    Unsafe,
    Regular,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attributes {
    pub inline: bool,
    pub testEntry: bool,
    pub builtin: bool,
    pub safety: Safety,
    pub varArgs: bool,
    pub prelude: bool,
}

impl Attributes {
    pub fn new() -> Self {
        Attributes {
            inline: false,
            testEntry: false,
            builtin: false,
            safety: Safety::Regular,
            varArgs: false,
            prelude: false,
        }
    }
}
