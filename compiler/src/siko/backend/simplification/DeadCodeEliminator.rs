use std::collections::BTreeSet;

use crate::siko::hir::{
    BodyBuilder::BodyBuilder,
    Function::{BlockId, Function},
    Instruction::InstructionKind,
};

pub fn eliminateDeadCode(f: &Function) -> Option<Function> {
    let mut eliminator = DeadCodeEliminator::new(f);
    eliminator.process()
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
struct InstructionId {
    block: BlockId,
    id: usize,
}

pub struct DeadCodeEliminator<'a> {
    function: &'a Function,
    visited: BTreeSet<InstructionId>,
}

impl<'a> DeadCodeEliminator<'a> {
    pub fn new(f: &'a Function) -> DeadCodeEliminator<'a> {
        DeadCodeEliminator {
            function: f,
            visited: BTreeSet::new(),
        }
    }

    fn process(&mut self) -> Option<Function> {
        if self.function.body.is_none() {
            return None;
        }

        //println!("DeadCodeEliminator processing function: {}", self.function.name);
        //println!("{}", self.function);

        self.processBlock(BlockId::first());

        let mut needsRemoval = false;

        if let Some(body) = &self.function.body {
            for (blockId, block) in body.blocks.iter() {
                for (index, _i) in block.instructions.iter().enumerate() {
                    let id = InstructionId {
                        block: *blockId,
                        id: index,
                    };
                    if !self.visited.contains(&id) {
                        //println!("DCE: Found dead instruction: {}", _i);
                        needsRemoval = true;
                    }
                }
            }
        }

        if !needsRemoval {
            return None; // No dead code found
        }

        let mut bodyBuilder = BodyBuilder::cloneFunction(self.function);

        let allblockIds = bodyBuilder.getAllBlockIds();
        for blockId in &allblockIds {
            let mut builder = bodyBuilder.iterator(*blockId);
            let mut index = 0;
            loop {
                if let Some(_i) = builder.getInstruction() {
                    //println!("DCE: Checking instruction: {}", _i);
                    let id = InstructionId {
                        block: *blockId,
                        id: index,
                    };
                    if !self.visited.contains(&id) {
                        //println!("DCE: Removing dead instruction: {}", _i);
                        builder.removeInstruction();
                    } else {
                        builder.step();
                    }
                    index += 1;
                } else {
                    break;
                }
            }
            if builder.getBlockSize() == 0 {
                //println!("DCE: Removing empty block: {}", blockId);
                bodyBuilder.removeBlock(*blockId);
            }
        }

        let mut f = self.function.clone();
        f.body = Some(bodyBuilder.build());
        Some(f)
    }

    fn processBlock(&mut self, blockId: BlockId) {
        //println!("Processing block: {}", blockId);
        let block = self.function.getBlockById(blockId);
        for (index, instruction) in block.instructions.iter().enumerate() {
            let id = InstructionId {
                block: blockId,
                id: index,
            };
            //println!("Processing instruction: {} in block {:?}", instruction, id);
            let added = self.visited.insert(id);
            if !added {
                return;
            }
            match &instruction.kind {
                InstructionKind::FunctionCall(dest, _, _, _) => {
                    if dest.getType().isNever() {
                        return;
                    }
                }
                InstructionKind::Converter(_, _) => {}
                InstructionKind::MethodCall(_, _, _, _) => {
                    unreachable!("method call in DCE")
                }
                InstructionKind::DynamicFunctionCall(_, _, _) => {}
                InstructionKind::FieldRef(_, _, _) => {}
                InstructionKind::Bind(_, _, _) => {}
                InstructionKind::Tuple(_, _) => {}
                InstructionKind::StringLiteral(_, _) => {}
                InstructionKind::IntegerLiteral(_, _) => {}
                InstructionKind::CharLiteral(_, _) => {}
                InstructionKind::Return(_, _) => return,
                InstructionKind::Ref(_, _) => {}
                InstructionKind::PtrOf(_, _) => {}
                InstructionKind::DropPath(_) => {
                    panic!("DropListPlaceholder found in DeadCodeEliminator, this should not happen");
                }
                InstructionKind::DropMetadata(_) => {
                    panic!("DropMetadata found in DeadCodeEliminator, this should not happen");
                }
                InstructionKind::Drop(_, _) => {}
                InstructionKind::Jump(_, id) => {
                    self.processBlock(*id);
                    return;
                }
                InstructionKind::Assign(_, _) => {}
                InstructionKind::FieldAssign(_, _, _) => {}
                InstructionKind::AddressOfField(_, _, _) => {}
                InstructionKind::DeclareVar(_, _) => {}
                InstructionKind::Transform(_, _, _) => {}
                InstructionKind::EnumSwitch(_, cases) => {
                    for case in cases {
                        self.processBlock(case.branch);
                    }
                    return;
                }
                InstructionKind::IntegerSwitch(_, cases) => {
                    for case in cases {
                        self.processBlock(case.branch);
                    }
                    return;
                }
                InstructionKind::BlockStart(_) => {}
                InstructionKind::BlockEnd(_) => {}
                InstructionKind::With(_, info) => {
                    self.processBlock(info.blockId);
                    return;
                }
                InstructionKind::GetImplicit(_, _) => {}
            }
        }
    }
}
