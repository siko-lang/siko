use crate::siko::{
    backend::borrowcheck::{
        DataGroups::DataGroups, FunctionGroups::FunctionGroupBuilder, FunctionProfiles::FunctionProfileBuilder,
    },
    hir::Program::Program,
};

pub struct BorrowChecker<'a> {
    program: &'a Program,
}

impl<'a> BorrowChecker<'a> {
    pub fn new(program: &'a Program) -> Self {
        BorrowChecker { program }
    }

    pub fn process(&mut self) {
        let mut dataGroups = DataGroups::new(self.program);
        dataGroups.process();
        let functionGroupBuilder = FunctionGroupBuilder::new(self.program);
        let functionGroups = functionGroupBuilder.process();
        for group in functionGroups {
            //println!("Function group: {:?}", group);
            for item in group.items {
                let f = self.program.getFunction(&item).unwrap();
                let mut profileBuilder = FunctionProfileBuilder::new(&f, self.program, &dataGroups);
                profileBuilder.process();
            }
        }
    }
}
