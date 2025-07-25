use std::collections::BTreeSet;

use crate::siko::{
    hir::{
        Function::{BlockId, Function},
        Instruction::InstructionKind,
        Program::Program,
    },
    location::Report::{Report, ReportContext},
};

pub fn eliminateDeadCode(ctx: &ReportContext, program: Program) -> Program {
    let mut result = program.clone();
    for (name, f) in &program.functions {
        let mut eliminator = DeadCodeEliminator::new(f);
        let f = eliminator.process(ctx);
        result.functions.insert(name.clone(), f);
    }
    result
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
struct InstructionId {
    block: usize,
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

    fn process(&mut self, ctx: &ReportContext) -> Function {
        if self.function.body.is_some() {
            self.processBlock(BlockId::first());
        }
        if let Some(body) = &self.function.body {
            for (blockIndex, block) in body.blocks.iter().enumerate() {
                for (index, instruction) in block.instructions.iter().enumerate() {
                    if !self.visited.contains(&InstructionId {
                        block: blockIndex,
                        id: index,
                    }) {
                        if !instruction.implicit {
                            println!("unreachable code {}", instruction);
                            let slogan = format!("Unreachable code");
                            let r = Report::new(ctx, slogan, Some(instruction.location.clone()));
                            r.print();
                        }
                    }
                }
            }
        }
        let mut result = self.function.clone();
        if let Some(body) = &mut result.body {
            for (blockIndex, block) in body.blocks.iter_mut().enumerate() {
                let instructions: Vec<_> = block
                    .instructions
                    .iter()
                    .cloned()
                    .enumerate()
                    .filter(|(index, _)| {
                        self.visited.contains(&InstructionId {
                            block: blockIndex,
                            id: *index,
                        })
                    })
                    .map(|(_, i)| i.clone())
                    .collect();
                block.instructions = instructions;
            }
        }
        result
    }

    fn processBlock(&mut self, blockId: BlockId) {
        let block = self.function.getBlockById(blockId);
        for (index, instruction) in block.instructions.iter().enumerate() {
            let added = self.visited.insert(InstructionId {
                block: blockId.id as usize,
                id: index,
            });
            if !added {
                return;
            }
            match &instruction.kind {
                InstructionKind::FunctionCall(dest, _, _) => {
                    if dest.getType().isNever() {
                        return;
                    }
                }
                InstructionKind::Converter(dest, source) => {}
                InstructionKind::MethodCall(_, _, _, _) => unreachable!("method call in DCE"),
                InstructionKind::DynamicFunctionCall(_, _, _) => {}
                InstructionKind::FieldRef(_, _, _) => {}
                InstructionKind::TupleIndex(_, _, _) => {}
                InstructionKind::Bind(_, _, _) => {}
                InstructionKind::Tuple(_, _) => {}
                InstructionKind::StringLiteral(_, _) => {}
                InstructionKind::IntegerLiteral(_, _) => {}
                InstructionKind::CharLiteral(_, _) => {}
                InstructionKind::Return(_, _) => return,
                InstructionKind::Ref(_, _) => {}
                InstructionKind::Drop(_, _) => {}
                InstructionKind::Jump(_, id, _) => {
                    self.processBlock(*id);
                    return;
                }
                InstructionKind::Assign(_, _) => {}
                InstructionKind::FieldAssign(_, _, _) => {}
                InstructionKind::DeclareVar(_, _) => {}
                InstructionKind::Transform(_, _, _) => {}
                InstructionKind::EnumSwitch(_, cases) => {
                    for case in cases {
                        self.processBlock(case.branch);
                    }
                }
                InstructionKind::IntegerSwitch(_, cases) => {
                    for case in cases {
                        self.processBlock(case.branch);
                    }
                }
                InstructionKind::StringSwitch(_, cases) => {
                    for case in cases {
                        self.processBlock(case.branch);
                    }
                }
                InstructionKind::BlockStart(_) => {}
                InstructionKind::BlockEnd(_) => {}
            }
        }
    }
}
