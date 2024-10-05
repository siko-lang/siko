use std::collections::BTreeMap;

use crate::siko::{
    ir::{Function::Parameter, Program::Program},
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
    processedProgram: Program,
}

impl<'a> DataFlowProfileBuilder<'a> {
    pub fn new(program: &'a Program) -> DataFlowProfileBuilder<'a> {
        DataFlowProfileBuilder {
            profiles: BTreeMap::new(),
            program: program,
            processedProgram: program.clone(),
        }
    }

    pub fn process(mut self) -> Program {
        let function_groups = FunctionGroups::createFunctionGroups(&self.program.functions);
        for group in function_groups {
            //println!("Processing function group {:?}", group.items);
            let mut processor =
                FunctionGroupProcessor::new(&mut self.profiles, group.items, self.program);
            processor.processGroup();
            for (name, data) in processor.inferenceData {
                let f = self
                    .processedProgram
                    .functions
                    .get_mut(&name)
                    .expect("function not found");
                let mut params = Vec::new();
                for (param, ty) in f.params.iter().zip(data.profile.args.iter()) {
                    match param {
                        Parameter::Named(n, _, mutable) => {
                            params.push(Parameter::Named(n.clone(), ty.clone(), *mutable))
                        }
                        Parameter::SelfParam(mutable, _) => {
                            params.push(Parameter::SelfParam(*mutable, ty.clone()))
                        }
                    }
                }
                f.params = params;
                f.result = data.profile.result.clone();
                self.profiles.insert(name.clone(), data.profile);
                if !data.instruction_types.is_empty() {
                    let body = f.body.as_mut().unwrap();
                    for b in &mut body.blocks {
                        for i in &mut b.instructions {
                            let ty = data.instruction_types.get(&i.id).expect("ty not found");
                            i.ty = Some(ty.clone());
                        }
                    }
                }
            }
        }
        self.processedProgram
    }
}
