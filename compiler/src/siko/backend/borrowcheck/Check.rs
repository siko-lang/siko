use crate::siko::{
    backend::borrowcheck::{
        functionprofiles::{
            FunctionProfileBuilder::FunctionProfileBuilder, FunctionProfileStore::FunctionProfileStore,
        },
        BorrowChecker::BorrowChecker,
        DataGroups::DataGroups,
    },
    hir::{FunctionGroupBuilder::FunctionGroupBuilder, Program::Program},
    location::Report::ReportContext,
};

pub struct Check<'a> {
    program: &'a Program,
}

impl<'a> Check<'a> {
    pub fn new(program: &'a Program) -> Self {
        Check { program }
    }

    pub fn process(&mut self, ctx: &'a ReportContext) {
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
                    let updated = profileBuilder.process(true);
                    if updated {
                        profileUpdated = true;
                    }
                }
                if !profileUpdated || group.items.len() == 1 {
                    break;
                }
            }
        }
        for (_, f) in &self.program.functions {
            let mut checker = BorrowChecker::new(
                ctx,
                f,
                self.program,
                &dataGroups,
                &mut profileStore,
                vec![f.name.clone()],
            );
            checker.process();
            //println!("Function profile for {}: {:?}", f.name, profile);
        }
    }
}
