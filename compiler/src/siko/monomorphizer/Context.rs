use core::panic;
use std::{collections::BTreeMap, fmt::Display};

use crate::siko::{
    hir::{
        Instruction::SyntaxBlockId,
        Type::{formatTypesBracket, Type},
    },
    location::Report::ReportContext,
    monomorphizer::{Error::MonomorphizerError, Handler::HandlerResolution},
    qualifiedname::QualifiedName,
};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Context {
    pub args: Vec<Type>,
    pub handlerResolution: HandlerResolution,
    pub impls: Vec<QualifiedName>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            args: Vec::new(),
            handlerResolution: HandlerResolution::new(),
            impls: Vec::new(),
        }
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.args.is_empty() {
            write!(f, "")?;
        } else {
            write!(f, "{}", formatTypesBracket(&self.args))?;
        }
        if !self.handlerResolution.isEmpty() {
            write!(f, " handlers: {{{}}}", self.handlerResolution)?;
        }
        if !self.impls.is_empty() {
            let impls: Vec<String> = self.impls.iter().map(|i| format!("{}", i)).collect();
            write!(f, " impls: ({})", impls.join(", "))?;
        }
        Ok(())
    }
}

pub struct HandlerResolutionStore {
    pub resolutions: BTreeMap<SyntaxBlockId, HandlerResolution>,
}

impl HandlerResolutionStore {
    pub fn new() -> Self {
        HandlerResolutionStore {
            resolutions: BTreeMap::new(),
        }
    }

    pub fn get(&self, syntaxBlockId: &SyntaxBlockId) -> (&HandlerResolution, SyntaxBlockId) {
        //println!("Getting effect resolution for {}", syntaxBlockId);
        let mut current = syntaxBlockId.clone();
        loop {
            //println!("Checking current syntax block: {}", current);
            if let Some(resolution) = self.resolutions.get(&current) {
                return (resolution, current);
            } else {
                let newCurrent = current.getParent();
                if newCurrent == current {
                    panic!("No effect resolution found for {}", syntaxBlockId);
                }
                current = newCurrent;
            }
        }
    }

    pub fn insert(&mut self, syntaxBlockId: SyntaxBlockId, resolution: HandlerResolution) {
        //println!("Inserting effect resolution for {}", syntaxBlockId);
        self.resolutions.insert(syntaxBlockId, resolution);
    }

    pub fn checkUnused(&self, ctx: &ReportContext) {
        for (_, resolution) in &self.resolutions {
            for (name, handler) in &resolution.handlers {
                if !handler.isUsed() && !handler.optional {
                    MonomorphizerError::UnusedEffectHandler {
                        effect: name.clone(),
                        handler: handler.name.clone(),
                        location: handler.location.clone(),
                    }
                    .report(ctx);
                }
            }
        }
    }
}
