use crate::siko::{
    backend::recursivedatahandler::DataGroup::processDataGroups, hir::Program::Program, location::Report::ReportContext,
};

pub fn process(ctx: &ReportContext, program: Program) -> Program {
    let program = processDataGroups(ctx, program);
    program
}
