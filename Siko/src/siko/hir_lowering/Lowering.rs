use crate::siko::{
    hir::{
        Function::{Function as HirFunction, InstructionKind as HirInstructionKind},
        Program::Program as HirProgram,
    },
    mir::{
        Function::{BasicBlock, Function as MirFunction, Instruction, InstructionKind},
        Program::Program as MirProgram,
    },
};

pub fn lowerFunction(function: &HirFunction) -> MirFunction {
    let mut mir_function = MirFunction::new(function.name.clone());
    let mut block = BasicBlock::new();
    for instruction in function.instructions() {
        match &instruction.kind {
            HirInstructionKind::FunctionCall(name, _) => {
                block
                    .instructions
                    .push(Instruction::new(InstructionKind::FunctionCall(
                        name.clone(),
                    )));
            }
            HirInstructionKind::Tuple(_) => {}
            HirInstructionKind::Drop(_) => {}
            k => panic!("NYI {}", k),
        }
    }
    mir_function.blocks.push(block);
    mir_function
}

pub fn lowerProgram(program: &HirProgram) -> MirProgram {
    let mut mir_program = MirProgram::new();

    for (_, function) in &program.functions {
        let f = lowerFunction(function);
        mir_program.functions.push(f);
    }

    mir_program
}
