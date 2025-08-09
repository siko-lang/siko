use crate::siko::{backend::recursivedatahandler::DataGroup::processDataGroup, hir::Program::Program};

pub fn process(program: Program) -> Program {
    processDataGroup(&program);
    let mut program = program;
    program
}
