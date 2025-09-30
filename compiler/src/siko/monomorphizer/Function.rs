use crate::siko::{
    hir::{
        Body::Body,
        BodyBuilder::BodyBuilder,
        Copy::IdentityCopier,
        Instruction::{SyntaxBlockId, SyntaxBlockIdSegment},
        Substitution::Substitution,
        SyntaxBlockIterator::SyntaxBlockIterator,
    },
    monomorphizer::{
        Context::HandlerResolutionStore, Handler::HandlerResolution, Instruction::processInstruction,
        Monomorphizer::Monomorphizer,
    },
    qualifiedname::QualifiedName,
};

pub fn processBody(
    input: Option<Body>,
    sub: &Substitution,
    mono: &mut Monomorphizer,
    handlerResolution: HandlerResolution,
    impls: &Vec<QualifiedName>,
) -> Option<Body> {
    match input {
        Some(body) => {
            let mut copier = IdentityCopier::new();
            let bodyBuilder = BodyBuilder::withBody(body.copy(&mut copier));
            let mut handlerResolutionStore = HandlerResolutionStore::new();
            handlerResolutionStore.insert(SyntaxBlockId::new(), handlerResolution.clone());
            handlerResolutionStore.insert(
                SyntaxBlockId::new().add(SyntaxBlockIdSegment { value: 0 }),
                handlerResolution.clone(),
            );
            let mut syntaxBlockIterator = SyntaxBlockIterator::new(bodyBuilder.clone());
            syntaxBlockIterator.iterate(|instruction, syntaxBlockId, blockBuilder| {
                let instruction = processInstruction(
                    instruction,
                    sub,
                    mono,
                    syntaxBlockId,
                    &mut handlerResolutionStore,
                    impls,
                );
                blockBuilder.replaceInstruction(instruction.kind, instruction.location.clone());
            });
            mono.resolutionStores.push(handlerResolutionStore);
            Some(bodyBuilder.build())
        }
        None => None,
    }
}
