use crate::siko::{
    backend::borrowcheck::{
        functionprofiles::{
            FunctionProfileBuilder::FunctionProfileBuilder, FunctionProfileStore::FunctionProfileStore,
        },
        DataGroups::DataGroups,
    },
    hir::{Function::Function, Program::Program},
    qualifiedname::QualifiedName,
};

pub struct BorrowChecker<'a> {
    profileBuilder: FunctionProfileBuilder<'a>,
}

impl<'a> BorrowChecker<'a> {
    pub fn new(
        f: &'a Function,
        program: &'a Program,
        dataGroups: &'a DataGroups<'a>,
        profileStore: &'a mut FunctionProfileStore,
        functionGroup: Vec<QualifiedName>,
    ) -> BorrowChecker<'a> {
        BorrowChecker {
            profileBuilder: FunctionProfileBuilder::new(f, program, dataGroups, profileStore, functionGroup),
        }
    }

    pub fn process(&mut self) {
        println!("Borrow checking function: {}", self.profileBuilder.f.name);
        println!("Function profile {}", self.profileBuilder.f);
        self.profileBuilder.process();
    }
}
