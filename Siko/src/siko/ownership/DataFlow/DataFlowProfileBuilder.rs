use std::collections::BTreeMap;

use crate::siko::{
    ir::Program::Program,
    ownership::{
        DataFlow::{
            DataFlowProfile::DataFlowProfile, FunctionGroupProcessor::FunctionGroupProcessor,
        },
        FunctionGroups,
    },
    qualifiedname::QualifiedName,
};

pub struct DataFlowProfileBuilder<'a> {
    profiles: BTreeMap<QualifiedName, DataFlowProfile>,
    program: &'a Program,
}

impl<'a> DataFlowProfileBuilder<'a> {
    pub fn new(program: &'a Program) -> DataFlowProfileBuilder<'a> {
        DataFlowProfileBuilder {
            profiles: BTreeMap::new(),
            program: program,
        }
    }

    pub fn process(&mut self) {
        let function_groups = FunctionGroups::createFunctionGroups(&self.program.functions);
        for group in function_groups {
            println!("Processing function group {:?}", group.items);
            let mut processor =
                FunctionGroupProcessor::new(&mut self.profiles, group.items, self.program);
            processor.processGroup();
            for (name, data) in processor.inferenceData {
                self.profiles.insert(name, data.profile);
            }
        }
    }
}
