use std::fmt::Display;

use crate::siko::{
    hir::Type::{formatTypes, Type},
    monomorphizer::Effect::EffectResolution,
};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Context {
    pub args: Vec<Type>,
    pub effectResolution: EffectResolution,
}

impl Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.args.is_empty() {
            write!(f, "")?;
        } else {
            write!(f, "{}", formatTypes(&self.args))?;
        }
        if !self.effectResolution.effects.is_empty() {
            write!(f, "-{}", self.effectResolution)?;
        }
        Ok(())
    }
}
