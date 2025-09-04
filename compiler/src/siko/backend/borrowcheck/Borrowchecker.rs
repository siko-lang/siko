use crate::siko::{backend::borrowcheck::DataGroups::DataGroups, hir::Program::Program};

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
    }
}
