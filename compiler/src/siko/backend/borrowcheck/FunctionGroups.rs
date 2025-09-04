use std::collections::BTreeMap;

use crate::siko::{
    hir::{Body::Body, Instruction::InstructionKind, Program::Program},
    qualifiedname::QualifiedName,
    util::DependencyProcessor::{processDependencies, DependencyGroup},
};

pub struct FunctionGroupBuilder<'a> {
    program: &'a Program,
}

impl<'a> FunctionGroupBuilder<'a> {
    pub fn new(program: &'a Program) -> Self {
        FunctionGroupBuilder { program }
    }

    pub fn process(&self) -> Vec<DependencyGroup<QualifiedName>> {
        let mut allDeps: BTreeMap<QualifiedName, Vec<QualifiedName>> = BTreeMap::new();
        for (_, f) in &self.program.functions {
            if let Some(body) = &f.body {
                let deps = self.processFunction(body);
                allDeps.insert(f.name.clone(), deps);
            } else {
                allDeps.insert(f.name.clone(), Vec::new());
            }
        }
        let groups = processDependencies(&allDeps);
        groups
    }

    fn processFunction(&self, body: &Body) -> Vec<QualifiedName> {
        let mut deps = Vec::new();
        for (_, block) in &body.blocks {
            let inner = block.getInner();
            let b = inner.borrow();
            for instr in &b.instructions {
                match &instr.kind {
                    InstructionKind::FunctionCall(_, info) => {
                        deps.push(info.name.clone());
                    }
                    _ => {}
                }
            }
        }
        deps
    }
}
