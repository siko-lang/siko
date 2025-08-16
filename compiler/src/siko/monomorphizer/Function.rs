use crate::siko::{
    hir::{
        BodyBuilder::BodyBuilder,
        Function::Body,
        Instruction::{SyntaxBlockId, SyntaxBlockIdSegment},
        Substitution::Substitution,
        SyntaxBlockIterator::SyntaxBlockIterator,
    },
    monomorphizer::{
        Context::EffectResolutionStore, Effect::EffectResolution, Instruction::processInstruction,
        Monomorphizer::Monomorphizer,
    },
};

pub fn processBody(
    input: Option<Body>,
    sub: &Substitution,
    mono: &mut Monomorphizer,
    effectResolution: EffectResolution,
) -> Option<Body> {
    match input {
        Some(body) => {
            let bodyBuilder = BodyBuilder::withBody(body);
            let mut effectResolutionStore = EffectResolutionStore::new();
            effectResolutionStore.insert(
                SyntaxBlockId::new().add(SyntaxBlockIdSegment { value: 0 }),
                effectResolution.clone(),
            );
            let mut syntaxBlockIterator = SyntaxBlockIterator::new(bodyBuilder.clone());
            syntaxBlockIterator.iterate(|instruction, syntaxBlockId, blockBuilder| {
                let instruction = processInstruction(instruction, sub, mono, syntaxBlockId, &mut effectResolutionStore);
                blockBuilder.replaceInstruction(instruction.kind, instruction.location.clone());
            });
            Some(bodyBuilder.build())
        }
        None => None,
    }
}
