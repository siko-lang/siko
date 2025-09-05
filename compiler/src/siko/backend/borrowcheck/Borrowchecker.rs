use crate::siko::{
    backend::borrowcheck::{
        functionprofiles::{
            FunctionProfileBuilder::FunctionProfileBuilder, FunctionProfileStore::FunctionProfileStore,
        },
        DataGroups::DataGroups,
        FunctionGroups::FunctionGroupBuilder,
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
        let mut profileStore = FunctionProfileStore::new();
        for group in functionGroups {
            //println!("Function group: {:?}", group);
            loop {
                let mut profileUpdated = false;
                for item in &group.items {
                    let f = self.program.getFunction(&item).unwrap();
                    let mut profileBuilder = FunctionProfileBuilder::new(
                        &f,
                        self.program,
                        &dataGroups,
                        &mut profileStore,
                        group.items.clone(),
                    );
                    let updated = profileBuilder.process();
                    if updated {
                        profileUpdated = true;
                    }
                }
                if !profileUpdated || group.items.len() == 1 {
                    break;
                }
            }
        }
    }
}
