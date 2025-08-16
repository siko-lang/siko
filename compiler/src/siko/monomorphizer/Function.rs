use std::collections::BTreeMap;

use crate::siko::{
    hir::{
        Function::{Block, Body},
        Substitution::Substitution,
    },
    monomorphizer::{Monomorphizer::Monomorphizer, Utils::Monomorphize},
};

impl Monomorphize for Block {
    fn process(&self, sub: &Substitution, mono: &mut Monomorphizer) -> Self {
        let instructions = self.instructions.process(sub, mono);
        Block {
            id: self.id.clone(),
            instructions: instructions,
        }
    }
}

impl Monomorphize for Body {
    fn process(&self, sub: &Substitution, mono: &mut Monomorphizer) -> Self {
        let blocks = self.blocks.process(sub, mono);
        Body {
            blocks: blocks,
            varTypes: BTreeMap::new(),
            varAllocator: self.varAllocator.clone(),
        }
    }
}
