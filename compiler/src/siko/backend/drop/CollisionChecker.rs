use std::collections::BTreeMap;

use crate::siko::{
    backend::drop::{
        BlockProcessor::BlockProcessor, Context::Context, DropMetadataStore::DropMetadataStore, Event::Collision,
        ReferenceStore::ReferenceStore,
    },
    hir::{Block::BlockId, BlockGroupBuilder::BlockGroupBuilder, BodyBuilder::BodyBuilder, Function::Function},
};

pub struct CollisionChecker<'a> {
    bodyBuilder: BodyBuilder,
    dropMetadataStore: &'a DropMetadataStore,
    referenceStore: &'a ReferenceStore,
    blockEnvs: BTreeMap<BlockId, Context>,
    f: &'a Function,
}

impl<'a> CollisionChecker<'a> {
    pub fn new(
        bodyBuilder: BodyBuilder,
        dropMetadataStore: &'a DropMetadataStore,
        referenceStore: &'a ReferenceStore,
        f: &'a Function,
    ) -> CollisionChecker<'a> {
        CollisionChecker {
            bodyBuilder,
            dropMetadataStore,
            referenceStore,
            blockEnvs: BTreeMap::new(),
            f,
        }
    }

    pub fn process(&mut self) -> Vec<Collision> {
        //println!("CollisionChecker processing function: {}", self.f.name);
        let mut queue = Vec::new();
        let mut allCollisions = Vec::new();
        let mut totalCount = 0;
        let blockGroupBuilder = BlockGroupBuilder::new(self.f);
        let groupInfo = blockGroupBuilder.process();
        for group in groupInfo.groups {
            //println!("Processing block group {:?}", group.items);
            for item in &group.items {
                //println!("Queueing block: {}", item);
                queue.push(item.clone());
                self.blockEnvs.insert(item.clone(), Context::new());
            }
            loop {
                let Some(blockId) = queue.pop() else { break };
                totalCount += 1;
                //println!("CollisionChecker processing block: {}", blockId);
                let builder = self.bodyBuilder.iterator(blockId);
                let mut blockProcessor = BlockProcessor::new(self.dropMetadataStore, self.referenceStore);
                let startContext = self.blockEnvs.get(&blockId).cloned().expect("Missing block context");
                let (context, jumpTargets) = blockProcessor.process(builder, startContext);
                let (collisions, baseEvents) = context.validate();
                // for (name, events) in &baseEvents {
                //     if name.to_string() != "tmp4" {
                //         continue;
                //     }
                //     println!("Base events for {} at end of block {:?}:", name, events);
                // }
                allCollisions.extend(collisions);
                let jumpContext = context.compress();
                for jumpTarget in jumpTargets {
                    let mut addedBaseEvent = false;
                    let blockStartContext = self.blockEnvs.entry(jumpTarget.clone()).or_insert_with(|| {
                        addedBaseEvent = true;
                        Context::new()
                    });
                    for (name, events) in &baseEvents {
                        let baseEvents = blockStartContext
                            .baseEvents
                            .entry(name.clone())
                            .or_insert_with(Vec::new);
                        for event in events.clone() {
                            if !baseEvents.contains(&event) {
                                // println!(
                                //     "Adding base event to {}: {}, target block: {} from {}",
                                //     name, event, jumpTarget, blockId
                                // );
                                addedBaseEvent = true;
                                baseEvents.push(event);
                            }
                        }
                    }
                    for (name, events) in &jumpContext.usages {
                        let baseEvents = blockStartContext
                            .baseEvents
                            .entry(name.clone())
                            .or_insert_with(Vec::new);
                        for event in events.events.clone() {
                            if !baseEvents.contains(&event) {
                                // println!(
                                //     "Adding base event to {}: {}, target block: {} from {}",
                                //     name, event, jumpTarget, blockId
                                // );
                                addedBaseEvent = true;
                                baseEvents.push(event);
                            }
                        }
                    }
                    if addedBaseEvent {
                        //println!("Re-queueing block: {}", jumpTarget);
                        if group.items.contains(&jumpTarget) {
                            queue.push(jumpTarget.clone());
                        }
                    }
                }
            }
        }
        if totalCount > 1000 {
            //println!("Processed {} blocks in {} cycles", self.blockEnvs.len(), totalCount);
        }
        //println!("CollisionChecker found {} collisions", allCollisions.len());
        allCollisions
    }
}
