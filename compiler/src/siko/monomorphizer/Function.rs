use crate::siko::{
    hir::{
        BodyBuilder::BodyBuilder,
        Function::Body,
        Instruction::{SyntaxBlockId, SyntaxBlockIdSegment},
        Substitution::Substitution,
        SyntaxBlockIterator::SyntaxBlockIterator,
    },
    monomorphizer::{
        Context::HandlerResolutionStore, Handler::HandlerResolution, Instruction::processInstruction,
        Monomorphizer::Monomorphizer,
    },
};

pub fn processBody(
    input: Option<Body>,
    sub: &Substitution,
    mono: &mut Monomorphizer,
    handlerResolution: HandlerResolution,
) -> Option<Body> {
    match input {
        Some(body) => {
            let bodyBuilder = BodyBuilder::withBody(body);
            let mut handlerResolutionStore = HandlerResolutionStore::new();
            handlerResolutionStore.insert(SyntaxBlockId::new(), handlerResolution.clone());
            handlerResolutionStore.insert(
                SyntaxBlockId::new().add(SyntaxBlockIdSegment { value: 0 }),
                handlerResolution.clone(),
            );
            let mut syntaxBlockIterator = SyntaxBlockIterator::new(bodyBuilder.clone());
            syntaxBlockIterator.iterate(|instruction, syntaxBlockId, blockBuilder| {
                let instruction =
                    processInstruction(instruction, sub, mono, syntaxBlockId, &mut handlerResolutionStore);
                blockBuilder.replaceInstruction(instruction.kind, instruction.location.clone());
            });
            handlerResolutionStore.checkUnused(&mono.ctx);
            Some(bodyBuilder.build())
        }
        None => None,
    }
}
