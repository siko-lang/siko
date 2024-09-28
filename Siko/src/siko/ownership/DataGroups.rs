use std::collections::BTreeMap;

use crate::siko::{
    ir::Data::{Class, Enum},
    qualifiedname::QualifiedName,
    util::DependencyProcessor::{processDependencies, DependencyGroup},
};

pub struct DataGroup {}

pub fn createDataGroups(
    classes: &BTreeMap<QualifiedName, Class>,
    enums: &BTreeMap<QualifiedName, Enum>,
) -> Vec<DependencyGroup<QualifiedName>> {
    let mut dependency_map = BTreeMap::new();

    for (name, c) in classes {
        let deps = dependency_map
            .entry(name.clone())
            .or_insert_with(|| Vec::new());
        for field in &c.fields {
            if let Some(dep) = field.ty.getName() {
                deps.push(dep);
            }
        }
    }

    for (name, e) in enums {
        let deps = dependency_map
            .entry(name.clone())
            .or_insert_with(|| Vec::new());
        for variant in &e.variants {
            for item in &variant.items {
                if let Some(dep) = item.getName() {
                    deps.push(dep);
                }
            }
        }
    }

    let groups = processDependencies(&dependency_map);
    for group in &groups {
        println!("data group {:?}", group);
    }
    groups
}
