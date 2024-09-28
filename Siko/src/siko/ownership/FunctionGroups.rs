use std::collections::BTreeMap;

use crate::siko::{
    ir::Function::{Function, InstructionKind},
    qualifiedname::QualifiedName,
    util::DependencyProcessor::{processDependencies, DependencyGroup},
};

pub struct DataGroup {}

pub fn createFunctionGroups(
    functions: &BTreeMap<QualifiedName, Function>,
) -> Vec<DependencyGroup<QualifiedName>> {
    let mut dependency_map = BTreeMap::new();

    for (name, f) in functions {
        let deps = dependency_map
            .entry(name.clone())
            .or_insert_with(|| Vec::new());
        for i in f.instructions() {
            match &i.kind {
                InstructionKind::FunctionCall(name, _) => {
                    deps.push(name.clone());
                }
                _ => {}
            }
        }
    }

    let groups = processDependencies(&dependency_map);
    // for group in &groups {
    //     println!("function group {:?}", group);
    // }
    groups
}
