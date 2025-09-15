use crate::siko::hir::{Instruction::InstructionKind, Program::Program};

pub fn canBeEliminated(program: &Program, i: &InstructionKind) -> bool {
    match i {
        InstructionKind::DeclareVar(_, _) => true,
        InstructionKind::FieldRef(_, _, _) => true,
        InstructionKind::Assign(_, _) => true,
        InstructionKind::FunctionCall(_, info) => {
            let f = match program.getFunction(&info.name) {
                Some(f) => f,
                None => {
                    panic!("Function not found: {}", info.name);
                }
            };
            f.isPure()
        }
        InstructionKind::Transform(_, _, _) => true,
        _ => false,
    }
}
