use std::collections::{BTreeMap, BTreeSet};

use crate::siko::{
    backend::borrowcheck::functionprofiles::FunctionProfileBuilder::FunctionProfileBuilder,
    hir::{
        Block::BlockId,
        BlockBuilder::InstructionRef,
        BlockGroupBuilder::BlockGroupInfo,
        Body::Body,
        Instruction::InstructionKind,
        Type::{formatTypes, Type},
    },
};

pub struct BorrowVarMap {
    // Maps an instruction to the list of borrow variables that present in any of the variables used or defined by this instruction.
    pub borrowVarMap: BTreeMap<InstructionRef, Vec<Type>>,
    // Maps an instruction to the borrow variable that is introduced by this instruction, if any.
    pub sourceMap: BTreeMap<InstructionRef, Type>,
}

impl BorrowVarMap {
    pub fn new() -> BorrowVarMap {
        BorrowVarMap {
            borrowVarMap: BTreeMap::new(),
            sourceMap: BTreeMap::new(),
        }
    }

    pub fn addBorrowVarToInstruction(&mut self, instrRef: InstructionRef, borrowVar: &Type) {
        let entry = self.borrowVarMap.entry(instrRef).or_insert_with(Vec::new);
        if !entry.contains(borrowVar) {
            entry.push(borrowVar.clone());
        }
    }

    pub fn dump(&self, body: &Body) {
        for (blockId, block) in &body.blocks {
            println!("Block {}:", blockId);
            let inner = block.getInner();
            let block_borrow = inner.borrow();
            for (index, instruction) in block_borrow.instructions.iter().enumerate() {
                let instr_ref = InstructionRef {
                    blockId: *blockId,
                    instructionId: index as u32,
                };
                if let Some(vars) = self.borrowVarMap.get(&instr_ref) {
                    println!("  {} {}", instruction.kind, formatTypes(vars));
                } else {
                    println!("  {}", instruction.kind);
                }
            }
        }
    }
}

pub struct BorrowVarMapBuilder<'a> {
    profileBuilder: &'a FunctionProfileBuilder<'a>,
    traceEnabled: bool,
}

impl<'a> BorrowVarMapBuilder<'a> {
    pub fn new(profileBuilder: &'a FunctionProfileBuilder<'a>, traceEnabled: bool) -> BorrowVarMapBuilder<'a> {
        BorrowVarMapBuilder {
            profileBuilder,
            traceEnabled,
        }
    }

    pub fn buildBorrowVarMap(&self, links: &BTreeMap<Type, Type>, groupInfo: &BlockGroupInfo) -> BorrowVarMap {
        let mut borrowVarMap = BorrowVarMap::new();
        let inverseLinkMap = self.buildInverseLinkMap(links);
        let body = self.profileBuilder.f.body.as_ref().unwrap();
        for (blockId, block) in &body.blocks {
            if self.traceEnabled {
                println!("Block {}:", blockId);
            }
            let inner = block.getInner();
            let b = inner.borrow();
            for (index, instruction) in b.instructions.iter().enumerate() {
                let instrRef = InstructionRef {
                    blockId: blockId.clone(),
                    instructionId: index as u32,
                };
                let (vars, sourceVar) = self.calculateBorrowVarsForInstruction(&inverseLinkMap, &instruction.kind);
                if !vars.is_empty() {
                    borrowVarMap.borrowVarMap.insert(instrRef, vars);
                }
                if let Some(src) = sourceVar {
                    borrowVarMap.sourceMap.insert(instrRef, src);
                }
            }
        }
        if self.traceEnabled {
            println!("Initial borrow variable map:");
            borrowVarMap.dump(body);
        }
        self.extendBorrowVarLiveness(&mut borrowVarMap, body, groupInfo);
        if self.traceEnabled {
            println!("Borrow variable map:");
            borrowVarMap.dump(body);
        }
        borrowVarMap
    }

    fn buildInverseLinkMap(&self, links: &BTreeMap<Type, Type>) -> BTreeMap<Type, Vec<Type>> {
        let mut inverseLinkMap: BTreeMap<Type, Vec<Type>> = BTreeMap::new();
        for (from, to) in links {
            let mut to = to.clone();
            loop {
                inverseLinkMap
                    .entry(to.clone())
                    .or_insert_with(Vec::new)
                    .push(from.clone());
                if let Some(next) = links.get(&to) {
                    to = next.clone();
                } else {
                    break;
                }
            }
        }
        inverseLinkMap
    }

    fn calculateBorrowVarsForInstruction(
        &self,
        inverseLinkMap: &BTreeMap<Type, Vec<Type>>,
        kind: &InstructionKind,
    ) -> (Vec<Type>, Option<Type>) {
        let mut vars = kind.collectVariables();
        let mut varTypes = Vec::new();
        let mut allBorrowVars: BTreeSet<Type> = BTreeSet::new();
        let mut sourceVar: Option<Type> = None;
        if let InstructionKind::Assign(dest, _) = &kind {
            vars.retain(|x| *x != *dest);
            let mut extV = self.profileBuilder.getFinalVarType(&dest);
            if extV.ty.isReference() {
                varTypes.push((dest.name(), extV.clone()));
                extV.base();
                allBorrowVars.extend(extV.vars.iter().cloned());
            }
        }
        if let InstructionKind::Ref(dest, _) = &kind {
            let mut extV = self.profileBuilder.getFinalVarType(&dest);
            assert!(extV.ty.isReference());
            let borrowVar = extV.base();
            sourceVar = Some(borrowVar.clone());
        }
        if let InstructionKind::FieldAccess(dest, info) = &kind {
            if info.isRef {
                let mut extV = self.profileBuilder.getFinalVarType(&dest);
                let receiverType = self.profileBuilder.getFinalVarType(&info.receiver);
                if !receiverType.vars.contains(&extV.vars[0]) {
                    assert!(extV.ty.isReference());
                    let borrowVar = extV.base();
                    sourceVar = Some(borrowVar.clone());
                }
            }
        }
        for v in vars {
            let varType = self.profileBuilder.getFinalVarType(&v);
            allBorrowVars.extend(varType.vars.iter().cloned());
            varTypes.push((v.name(), varType));
        }
        let resolvedBorrowVars = resolveBorrowVars(inverseLinkMap, &allBorrowVars);
        if self.traceEnabled {
            let varTypesStr: Vec<String> = varTypes.iter().map(|(n, t)| format!("{}: {}", n, t)).collect();
            let borrowVarStr: Vec<String> = allBorrowVars.iter().map(|t| format!("{}", t)).collect();
            let resolvedBorrowVarStr: Vec<String> = resolvedBorrowVars.iter().map(|t| format!("{}", t)).collect();
            println!(
                "  {}: {} {} {}",
                kind,
                borrowVarStr.join(", "),
                varTypesStr.join(", "),
                resolvedBorrowVarStr.join(", ")
            );
        }
        (resolvedBorrowVars, sourceVar)
    }

    fn extendBorrowVarLiveness(&self, borrowVarMap: &mut BorrowVarMap, body: &Body, groupInfo: &BlockGroupInfo) {
        // Collect every instruction that already mentions each borrow variable.
        let mut borrowVarToUsers: BTreeMap<Type, Vec<InstructionRef>> = BTreeMap::new();
        for (instrRef, vars) in &borrowVarMap.borrowVarMap {
            for borrowVar in vars {
                borrowVarToUsers
                    .entry(borrowVar.clone())
                    .or_insert_with(Vec::new)
                    .push(*instrRef);
            }
        }
        let sources: Vec<(InstructionRef, Type)> = borrowVarMap
            .sourceMap
            .iter()
            .map(|(instrRef, borrowVar)| (*instrRef, borrowVar.clone()))
            .collect();
        for (sourceRef, borrowVar) in sources {
            if let Some(users) = borrowVarToUsers.get(&borrowVar) {
                for userRef in users {
                    if *userRef == sourceRef {
                        continue;
                    }
                    if self.traceEnabled {
                        println!(" Extending borrow var {} from {} to {}", borrowVar, sourceRef, userRef);
                    }
                    self.extendBorrowVarBetween(
                        borrowVarMap,
                        body,
                        groupInfo,
                        sourceRef,
                        *userRef,
                        &borrowVar,
                    );
                }
            }
        }
    }

    fn extendBorrowVarBetween(
        &self,
        borrowVarMap: &mut BorrowVarMap,
        body: &Body,
        groupInfo: &BlockGroupInfo,
        sourceRef: InstructionRef,
        userRef: InstructionRef,
        borrowVar: &Type,
    ) {
        #[derive(Clone, Copy)]
        enum WorkItem {
            Instruction(InstructionRef),
            BlockEntry(BlockId),
        }

        let mut worklist: Vec<WorkItem> = Vec::new();
        let mut visited_instructions: BTreeSet<InstructionRef> = BTreeSet::new();
        let mut visited_blocks: BTreeSet<BlockId> = BTreeSet::new();
        worklist.push(WorkItem::Instruction(userRef));

        while let Some(item) = worklist.pop() {
            match item {
                WorkItem::Instruction(instr_ref) => {
                    if !visited_instructions.insert(instr_ref) {
                        continue;
                    }
                    if instr_ref != sourceRef {
                        borrowVarMap.addBorrowVarToInstruction(instr_ref, borrowVar);
                    }
                    if instr_ref == sourceRef {
                        continue;
                    }
                    if instr_ref.instructionId > 0 {
                        worklist.push(WorkItem::Instruction(InstructionRef {
                            blockId: instr_ref.blockId,
                            instructionId: instr_ref.instructionId - 1,
                        }));
                    } else {
                        worklist.push(WorkItem::BlockEntry(instr_ref.blockId));
                    }
                }
                WorkItem::BlockEntry(block_id) => {
                    if !visited_blocks.insert(block_id) {
                        continue;
                    }
                    if let Some(preds) = groupInfo.deps.get(&block_id) {
                        for pred in preds {
                            let size = body.getBlockSize(*pred);
                            if size == 0 {
                                worklist.push(WorkItem::BlockEntry(*pred));
                            } else {
                                worklist.push(WorkItem::Instruction(InstructionRef {
                                    blockId: *pred,
                                    instructionId: (size - 1) as u32,
                                }));
                            }
                        }
                    }
                }
            }
        }
    }
}

fn resolveBorrowVars(inverseLinkMap: &BTreeMap<Type, Vec<Type>>, allBorrowVars: &BTreeSet<Type>) -> Vec<Type> {
    let mut resolvedBorrowVars = Vec::new();
    for bv in allBorrowVars {
        if let Some(links) = inverseLinkMap.get(bv) {
            for l in links {
                if !resolvedBorrowVars.contains(l) {
                    resolvedBorrowVars.push(l.clone());
                }
            }
        } else {
            if !resolvedBorrowVars.contains(bv) {
                resolvedBorrowVars.push(bv.clone());
            }
        }
    }
    resolvedBorrowVars
}
