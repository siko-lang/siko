use std::collections::{BTreeMap, BTreeSet};

use crate::siko::{
    hir::{
        Block::BlockId,
        BodyBuilder::BodyBuilder,
        Copy::VariableInlineCopier,
        Function::Function,
        FunctionGroupBuilder::FunctionGroupInfo,
        Instruction::{EnumCase, InstructionKind, IntegerCase},
        Program::Program,
        Type::Type,
    },
    qualifiedname::QualifiedName,
};

pub struct Inliner<'a> {
    pub wasInlined: BTreeSet<QualifiedName>,
    pub wasNotInlined: BTreeSet<QualifiedName>,
    functionGroupInfo: &'a FunctionGroupInfo,
    enabled: bool,
}

impl<'a> Inliner<'a> {
    pub fn new(enabled: bool, functionGroupInfo: &'a FunctionGroupInfo) -> Self {
        Inliner {
            wasInlined: BTreeSet::new(),
            wasNotInlined: BTreeSet::new(),
            enabled,
            functionGroupInfo,
        }
    }

    pub fn process(
        &mut self,
        function: &Function,
        program: &Program,
        groupItems: &Vec<QualifiedName>,
    ) -> Option<Function> {
        if function.body.is_none() || !self.enabled {
            return None;
        }
        // println!("Processing function: {}", function.name);
        // println!("{}", function);
        let mut bodyBuilder = BodyBuilder::cloneFunction(function);
        let mut inlined = false;
        loop {
            if !self.inlineCalls(program, groupItems, &mut bodyBuilder) {
                break;
            }
            inlined = true;
        }
        if !inlined {
            return None;
        }
        //println!("Inlined function body:");
        let body = bodyBuilder.build();
        let f = Function {
            body: Some(body),
            ..function.clone()
        };
        //println!("{}", f);
        Some(f)
    }

    fn inlineCalls(
        &mut self,
        program: &Program,
        groupItems: &Vec<QualifiedName>,
        bodyBuilder: &mut BodyBuilder,
    ) -> bool {
        let blockIds = bodyBuilder.getAllBlockIds();
        for blockId in blockIds {
            let mut builder = bodyBuilder.iterator(blockId);
            loop {
                if let Some(instruction) = builder.getInstruction() {
                    match &instruction.kind {
                        InstructionKind::FunctionCall(dest, info) => {
                            let callee = program.functions.get(&info.name).expect("Function not found");
                            if callee.canbeInlined() && !self.functionGroupInfo.isRecursive(&info.name) {
                                if groupItems.contains(&info.name) {
                                    // Don't inline functions in the same group
                                    self.wasNotInlined.insert(info.name.clone());
                                    builder.step();
                                    continue;
                                }
                                self.wasInlined.insert(info.name.clone());
                                //println!("Inlining function: {}", info.name);
                                //println!("Callee {}", callee);
                                let afterCallBlockId = builder.splitBlock(1);
                                let calleeBody = match &callee.body {
                                    Some(body) => body,
                                    None => {
                                        panic!("Callee has no body: {}", info.name);
                                    }
                                };
                                //println!("After call block id: {}", afterCallBlockId);
                                let mut argMap = BTreeMap::new();
                                for (index, p) in callee.params.iter().enumerate() {
                                    let argVar = info.args.getVariables().get(index).expect("Argument not found");
                                    argMap.insert(p.getName().clone(), argVar.clone());
                                }
                                let mut inlineVarCopier =
                                    VariableInlineCopier::new(bodyBuilder.getVariableAllocator(), argMap);
                                let calleeBody = calleeBody.copy(&mut inlineVarCopier);
                                let mut blockIdMap = BTreeMap::new();
                                //println!("Block ids in callee:");
                                //println!("{:?}", calleeBody.getAllBlockIds());
                                for (blockId, _) in &calleeBody.blocks {
                                    let newBlock = bodyBuilder.createBlock();
                                    //println!("  mapping block {} => {}", blockId, newBlock.getBlockId());
                                    blockIdMap.insert(blockId, newBlock.getBlockId());
                                }
                                let entryBlockId = blockIdMap
                                    .get(&BlockId::first())
                                    .expect("Entry block not found")
                                    .clone();
                                let jumpVar = bodyBuilder
                                    .createTempValueWithType(instruction.location.clone(), Type::getNeverType());
                                builder.replaceInstruction(
                                    InstructionKind::Jump(jumpVar, entryBlockId),
                                    instruction.location.clone(),
                                );
                                for (_, block) in &calleeBody.blocks {
                                    let newBlockId = *blockIdMap.get(&block.getId()).unwrap();
                                    let mut newBlockBuilder = bodyBuilder.block(newBlockId);
                                    let inner = block.getInner();
                                    let b = inner.borrow();
                                    for instruction in &b.instructions {
                                        match &instruction.kind {
                                            InstructionKind::Return(_, arg) => {
                                                newBlockBuilder.addAssign(
                                                    dest.clone(),
                                                    arg.clone(),
                                                    instruction.location.clone(),
                                                );
                                                newBlockBuilder.addJump(afterCallBlockId, instruction.location.clone());
                                            }
                                            InstructionKind::Jump(_, blockId) => {
                                                let targetBlockId = *blockIdMap.get(blockId).unwrap();
                                                newBlockBuilder.addJump(targetBlockId, instruction.location.clone());
                                            }
                                            InstructionKind::EnumSwitch(var, cases) => {
                                                let mut newCases = Vec::new();
                                                for oldCase in cases {
                                                    let targetBlockId = *blockIdMap.get(&oldCase.branch).unwrap();
                                                    let newCase = EnumCase {
                                                        index: oldCase.index.clone(),
                                                        branch: targetBlockId,
                                                    };
                                                    newCases.push(newCase);
                                                }
                                                newBlockBuilder.addInstruction(
                                                    InstructionKind::EnumSwitch(var.clone(), newCases),
                                                    instruction.location.clone(),
                                                );
                                            }
                                            InstructionKind::IntegerSwitch(variable, integer_cases) => {
                                                let mut newCases = Vec::new();
                                                for oldCase in integer_cases {
                                                    let targetBlockId = *blockIdMap.get(&oldCase.branch).unwrap();
                                                    let newCase = IntegerCase {
                                                        value: oldCase.value.clone(),
                                                        branch: targetBlockId,
                                                    };
                                                    newCases.push(newCase);
                                                }
                                                newBlockBuilder.addInstruction(
                                                    InstructionKind::IntegerSwitch(variable.clone(), newCases),
                                                    instruction.location.clone(),
                                                );
                                            }
                                            kind => {
                                                newBlockBuilder
                                                    .addInstruction(kind.clone(), instruction.location.clone());
                                            }
                                        }
                                    }
                                }
                                return true;
                            } else {
                                //println!("Not inlining function: {} (not inline)", info.name);
                            }
                        }
                        _ => {}
                    }
                    builder.step();
                } else {
                    break;
                }
            }
        }
        false
    }
}
