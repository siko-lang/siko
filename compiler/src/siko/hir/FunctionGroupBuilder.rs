use std::collections::{BTreeMap, BTreeSet};

use crate::siko::{
    hir::{Body::Body, Instruction::InstructionKind, Program::Program},
    qualifiedname::{builtins::getMainName, QualifiedName},
    util::DependencyProcessor::{processDependencies, DependencyGroup},
};

pub struct FunctionGroupInfo {
    pub groups: Vec<DependencyGroup<QualifiedName>>,
    pub groupMap: BTreeMap<QualifiedName, DependencyGroup<QualifiedName>>,
    pub isDirectlyRecursive: BTreeSet<QualifiedName>,
}

impl FunctionGroupInfo {
    pub fn isRecursive(&self, name: &QualifiedName) -> bool {
        if let Some(group) = self.groupMap.get(name) {
            group.items.len() > 1 || self.isDirectlyRecursive.contains(name)
        } else {
            false
        }
    }
}

pub struct FunctionGroupBuilder<'a> {
    program: &'a Program,
}

impl<'a> FunctionGroupBuilder<'a> {
    pub fn new(program: &'a Program) -> Self {
        FunctionGroupBuilder { program }
    }

    pub fn process(&self) -> FunctionGroupInfo {
        let mut allDeps: BTreeMap<QualifiedName, Vec<QualifiedName>> = BTreeMap::new();
        let mut isDirectlyRecursive: BTreeSet<QualifiedName> = BTreeSet::new();
        for (_, f) in &self.program.functions {
            if let Some(body) = &f.body {
                let deps = self.processFunction(body);
                if deps.contains(&f.name) {
                    isDirectlyRecursive.insert(f.name.clone());
                }
                allDeps.insert(f.name.clone(), deps);
            } else {
                allDeps.insert(f.name.clone(), Vec::new());
            }
        }
        let groups = processDependencies(&allDeps);
        let mut groupMap = BTreeMap::new();
        for group in &groups {
            for item in &group.items {
                groupMap.insert(item.clone(), group.clone());
            }
        }
        // self.printFullCallGraph(&allDeps, &groupMap, self.program);
        FunctionGroupInfo {
            groups,
            groupMap,
            isDirectlyRecursive: isDirectlyRecursive,
        }
    }

    fn processFunction(&self, body: &Body) -> Vec<QualifiedName> {
        let mut deps = Vec::new();
        for (_, block) in &body.blocks {
            let inner = block.getInner();
            let b = inner.borrow();
            for instr in &b.instructions {
                match &instr.kind {
                    InstructionKind::FunctionCall(_, info) => {
                        if deps.contains(&info.name) {
                            continue;
                        }
                        deps.push(info.name.clone());
                    }
                    _ => {}
                }
            }
        }
        deps
    }

    fn printFullCallGraph(
        &self,
        allDeps: &BTreeMap<QualifiedName, Vec<QualifiedName>>,
        groupMap: &BTreeMap<QualifiedName, &DependencyGroup<QualifiedName>>,
        program: &Program,
    ) {
        let main = getMainName();
        self.printEntry(main, allDeps, 0, groupMap, program);
    }

    fn printEntry(
        &self,
        entry: QualifiedName,
        allDeps: &BTreeMap<QualifiedName, Vec<QualifiedName>>,
        depth: usize,
        groupMap: &BTreeMap<QualifiedName, &DependencyGroup<QualifiedName>>,
        program: &Program,
    ) {
        if let Some(deps) = allDeps.get(&entry) {
            for _ in 0..depth {
                print!("  ");
            }
            let f = program.functions.get(&entry).expect("Function not found");
            println!("{}{}", entry, if f.attributes.inline { " - inline" } else { "" });
            let group = groupMap.get(&entry).expect("Group not found");
            for dep in deps {
                if group.items.contains(dep) {
                    // Don't print calls within the same group
                    continue;
                }
                self.printEntry(dep.clone(), allDeps, depth + 1, groupMap, program);
            }
        }
    }
}
