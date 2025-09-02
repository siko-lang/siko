use crate::siko::{
    hir::{
        Block::BlockId,
        Body::Body,
        ConstraintContext::ConstraintContext,
        Function::{Function, FunctionKind},
        Instruction::InstructionKind,
        Variable::VariableName,
    },
    typechecker::Typechecker::ClosureTypeInfo,
};

fn getArgName(index: u32) -> VariableName {
    VariableName::Arg(format!("arg{}", index))
}

pub struct ClosureSeparator<'a> {
    function: &'a mut Function,
    closureEntry: BlockId,
    closureTypeInfo: &'a ClosureTypeInfo,
    closureBody: Body,
}

impl<'a> ClosureSeparator<'a> {
    pub fn new(function: &'a mut Function, closureEntry: BlockId, closureTypeInfo: &'a ClosureTypeInfo) -> Self {
        ClosureSeparator {
            function,
            closureEntry,
            closureTypeInfo,
            closureBody: Body::new(),
        }
    }

    pub fn process(&mut self) -> Function {
        self.processBlock(self.closureEntry);
        let params = Vec::new();
        let constraintContext = ConstraintContext::new();
        let closureFn = Function::new(
            self.closureTypeInfo
                .name
                .as_ref()
                .expect("Closure must have name")
                .clone(),
            params,
            self.closureTypeInfo
                .resultType
                .clone()
                .expect("Closure must have result type"),
            Some(self.closureBody.clone()),
            constraintContext,
            FunctionKind::UserDefined,
        );
        // println!("Closure function created: {}", closureFn);
        closureFn
    }

    fn processBlock(&mut self, blockId: BlockId) {
        let mut block = self.function.getBlockById(blockId).clone();
        for instructionIndex in 0..block.size() {
            let instr = block.getInstruction(instructionIndex);
            match instr.kind {
                InstructionKind::Assign(lhs, rhs) => match rhs.name() {
                    VariableName::ClosureArg(_, index) => {
                        let arg_name = getArgName(index);
                        let rhs = rhs.cloneInto(arg_name);
                        block.replace(
                            instructionIndex,
                            InstructionKind::Assign(lhs, rhs),
                            instr.location,
                            instr.implicit,
                        );
                    }
                    VariableName::LambdaArg(_, index) => {
                        let arg_name = getArgName(self.closureTypeInfo.envTypes.len() as u32 + index);
                        let rhs = rhs.cloneInto(arg_name);
                        block.replace(
                            instructionIndex,
                            InstructionKind::Assign(lhs, rhs),
                            instr.location,
                            instr.implicit,
                        );
                    }
                    _ => {}
                },
                InstructionKind::Jump(_, target) => {
                    self.processBlock(target);
                }
                InstructionKind::EnumSwitch(_, cases) => {
                    for c in cases {
                        self.processBlock(c.branch);
                    }
                }
                InstructionKind::ClosureReturn(_, v, arg) => {
                    let kind = InstructionKind::Return(v, arg);
                    block.replace(instructionIndex, kind, instr.location, instr.implicit);
                }
                _ => {}
            }
        }
        let origBody = self.function.body.as_mut().unwrap();
        origBody.removeBlock(blockId);
        if block.getId() == self.closureEntry {
            block.setId(BlockId::first());
        }
        self.closureBody.addBlock(block.clone());
    }
}
