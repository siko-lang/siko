use crate::siko::{
    ir::{Function::FunctionKind, Program::Program},
    ownership::DataFlowProfile::DataFlowProfile,
};

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
            for item in group.items {
                let f = self
                    .program
                    .functions
                    .get(&item)
                    .expect("Function not found in DataFlowProfileBuilder");
                match f.kind {
                    FunctionKind::UserDefined => {
                        for i in f.instructions() {
                            let ty = i.ty.clone().expect("no type");
                            if let Some(name) = ty.getName() {
                                if let Some(c) = self.program.classes.get(&name) {}
                            }
                            println!("{}", i);
                        }
                    }
                    FunctionKind::VariantCtor(index) => {
                        let eName = f.result.getName().expect("no result type");
                        let e = self.program.getEnum(&eName);
                        let variant = &e.variants[index as usize];
                        let mut args = Vec::new();
                        for ty in &variant.items {
                            args.push(ty.clone());
                        }
                        let profile = DataFlowProfile::new(args, e.ty.clone());
                        println!("profile {}", profile);
                    }
                    FunctionKind::ClassCtor => {
                        let cName = f.result.getName().expect("no result type");
                        let c = self.program.getClass(&cName);
                        //println!("{}", c);
                        let mut args = Vec::new();
                        for f in c.fields {
                            args.push(f.ty.clone());
                        }
                        let profile = DataFlowProfile::new(args, c.ty.clone());
                        println!("profile {}", profile);
                    }
                }
            }
        }
    }
}
