use std::collections::BTreeMap;

use crate::siko::{ir::Function::Function, qualifiedname::QualifiedName};

pub struct BorrowChecker<'a> {
    functions: &'a BTreeMap<QualifiedName, Function>,
}

impl<'a> BorrowChecker<'a> {
    pub fn new(functions: &'a BTreeMap<QualifiedName, Function>) -> BorrowChecker<'a> {
        BorrowChecker {
            functions: functions,
        }
    }

    pub fn run(&mut self, function: &Function) {
        match &function.body {
            Some(body) => {
                for b in &body.blocks {
                    for _ in &b.instructions {}
                }
            }
            None => {}
        }
    }
}
