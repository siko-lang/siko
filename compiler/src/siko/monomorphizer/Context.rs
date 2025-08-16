use core::panic;
use std::{collections::BTreeMap, fmt::Display};

use crate::siko::{
    hir::{
        Instruction::SyntaxBlockId,
        Type::{formatTypes, Type},
    },
    location::Report::{Report, ReportContext},
    monomorphizer::Effect::EffectResolution,
};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Context {
    pub args: Vec<Type>,
    pub effectResolution: EffectResolution,
}

impl Context {
    pub fn new() -> Self {
        Context {
            args: Vec::new(),
            effectResolution: EffectResolution::new(),
        }
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.args.is_empty() {
            write!(f, "")?;
        } else {
            write!(f, "{}", formatTypes(&self.args))?;
        }
        if !self.effectResolution.effects.is_empty() {
            write!(f, " effects: {{{}}}", self.effectResolution)?;
        }
        Ok(())
    }
}

pub struct EffectResolutionStore {
    pub resolutions: BTreeMap<SyntaxBlockId, EffectResolution>,
}

impl EffectResolutionStore {
    pub fn new() -> Self {
        EffectResolutionStore {
            resolutions: BTreeMap::new(),
        }
    }

    pub fn get(&self, syntaxBlockId: &SyntaxBlockId) -> &EffectResolution {
        //println!("Getting effect resolution for {}", syntaxBlockId);
        let mut current = syntaxBlockId.clone();
        loop {
            //println!("Checking current syntax block: {}", current);
            if let Some(resolution) = self.resolutions.get(&current) {
                return resolution;
            } else {
                let newCurrent = current.getParent();
                if newCurrent == current {
                    panic!("No effect resolution found for {}", syntaxBlockId);
                }
                current = newCurrent;
            }
        }
    }

    pub fn insert(&mut self, syntaxBlockId: SyntaxBlockId, resolution: EffectResolution) {
        //println!("Inserting effect resolution for {}", syntaxBlockId);
        self.resolutions.insert(syntaxBlockId, resolution);
    }

    pub fn checkUnused(&self, ctx: &ReportContext) {
        for (_, resolution) in &self.resolutions {
            for (name, handler) in &resolution.effects {
                if !handler.isUsed() {
                    let slogan = format!(
                        "Unused effect handler {} for {}",
                        format!("{}", ctx.yellow(&handler.name.toString())),
                        format!("{}", ctx.yellow(&name.toString())),
                    );
                    let r = Report::new(ctx, slogan, Some(handler.location.clone()));
                    r.print();
                    std::process::exit(1);
                }
            }
        }
    }
}
