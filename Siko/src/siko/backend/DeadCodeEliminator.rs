use std::collections::BTreeSet;

use crate::siko::{
    hir::{
        Function::{BlockId, Function, InstructionId, InstructionKind},
        Program::Program,
    },
    location::Report::Report,
};

pub fn eliminateDeadCode(program: Program) -> Program {
    let mut result = program.clone();
    for (name, f) in &program.functions {
        let mut eliminator = DeadCodeEliminator::new(f);
        let f = eliminator.process();
        result.functions.insert(name.clone(), f);
    }
    result
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

    fn process(&mut self) -> Function {
        if self.function.body.is_some() {
            self.processBlock(BlockId::first());
        }
        for instruction in self.function.instructions() {
            if !self.visited.contains(&instruction.id) {
                if !instruction.implicit {
                    println!("unreachable code {}", instruction);
                    let slogan = format!("Unreachable code");
                    let r = Report::new(slogan, Some(instruction.location.clone()));
                    r.print();
                }
            }
        }
        let mut result = self.function.clone();
        if let Some(body) = &mut result.body {
            for block in &mut body.blocks {
                let instructions: Vec<_> = block
                    .instructions
                    .iter()
                    .cloned()
                    .filter(|i| self.visited.contains(&i.id))
                    .collect();
                block.instructions = instructions;
            }
        }
        result
    }

    fn processBlock(&mut self, blockId: BlockId) {
        let block = self.function.getBlockById(blockId);
        for instruction in &block.instructions {
            let added = self.visited.insert(instruction.id);
            if !added {
                return;
            }
            match &instruction.kind {
                InstructionKind::FunctionCall(_, _) => {}
                InstructionKind::DynamicFunctionCall(_, _) => {}
                InstructionKind::If(_, trueBlock, falseBlock) => {
                    self.processBlock(*trueBlock);
                    if let Some(falseBlock) = falseBlock {
                        self.processBlock(*falseBlock);
                    }
                    return;
                }
                InstructionKind::ValueRef(_, _, _) => {}
                InstructionKind::Bind(_, _) => {}
                InstructionKind::Tuple(_) => {}
                InstructionKind::TupleIndex(_, _) => {}
                InstructionKind::StringLiteral(_) => {}
                InstructionKind::IntegerLiteral(_) => {}
                InstructionKind::CharLiteral(_) => {}
                InstructionKind::Return(_) => return,
                InstructionKind::Ref(_) => {}
                InstructionKind::Drop(_) => {}
                InstructionKind::Jump(id) => {
                    self.processBlock(*id);
                    return;
                }
                InstructionKind::Assign(_, _) => {}
                InstructionKind::DeclareVar(_) => {}
            }
        }
    }
}
