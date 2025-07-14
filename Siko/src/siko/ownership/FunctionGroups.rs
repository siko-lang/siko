use std::collections::BTreeMap;

use crate::siko::{
    hir::{Function::Function, Instruction::InstructionKind},
    qualifiedname::QualifiedName,
    util::DependencyProcessor::{processDependencies, DependencyGroup},
};

pub struct DataGroup {}

pub fn createFunctionGroups(functions: &BTreeMap<QualifiedName, Function>) -> Vec<DependencyGroup<QualifiedName>> {
    let mut dependency_map = BTreeMap::new();

    for (name, f) in functions {
        // println!("Processing function {:?}", name);
        let deps = dependency_map.entry(name.clone()).or_insert_with(|| Vec::new());
        let Some(body) = &f.body else {
            continue;
        };
        for id in &body.getAllBlockIds() {
            let block = body.getBlockById(*id);
            for i in &block.instructions {
                match &i.kind {
                    InstructionKind::FunctionCall(_, name, _) => {
                        deps.push(name.clone());
                    }
                    _ => {}
                }
            }
        }
    }

    // for (name, deps) in &dependency_map {
    //     println!("Function {:?} depends on {:?}", name, deps);
    // }

    let groups = processDependencies(&dependency_map);
    // for group in &groups {
    //     println!("function group {:?}", group);
    // }
    groups
}
