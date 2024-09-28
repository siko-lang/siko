use crate::siko::ir::Program::Program;

use super::FunctionGroups;

pub struct DataFlowProfileBuilder<'a> {
    program: &'a Program,
}

impl<'a> DataFlowProfileBuilder<'a> {
    pub fn new(program: &'a Program) -> DataFlowProfileBuilder<'a> {
        DataFlowProfileBuilder { program: program }
    }

    pub fn process(&mut self) {
        let function_groups = FunctionGroups::createFunctionGroups(&self.program.functions);
        for group in function_groups {
            println!("Processing function group {:?}", group.items);
        }
    }
}
