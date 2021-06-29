use crate::mir::*;
use crate::scc::*;
use std::collections::BTreeMap;

fn process_item(id: &NodeId, ty: &String, id_map: &BTreeMap<&String, NodeId>, g: &mut Graph) {
    let sub_id = id_map.get(ty).unwrap();
    g.add_neighbour(*id, *sub_id);
}

pub fn collect_data_groups(mir_program: &Program) -> Vec<Vec<String>> {
    println!("Collecting data groups");
    let mut g = Graph::new();
    let mut id_map = BTreeMap::new();
    let mut name_map = BTreeMap::new();
    for (name, _) in &mir_program.data {
        let id = g.add_node();
        id_map.insert(name, id);
        name_map.insert(id, name);
    }
    for (name, d) in &mir_program.data {
        match d {
            Data::Adt(adt) => {
                let id = id_map.get(name).unwrap();
                for v in &adt.variants {
                    process_item(id, &v.ty, &id_map, &mut g);
                }
            }
            Data::Record(record) => {
                let id = id_map.get(name).unwrap();
                if let Some(externals) = &record.externals {
                    for e in externals {
                        process_item(id, &e.ty, &id_map, &mut g);
                    }
                }
                for f in &record.fields {
                    process_item(id, &f.ty, &id_map, &mut g);
                }
            }
        }
    }
    let sccs = g.collect_sccs();
    let mut groups = Vec::new();
    for scc in sccs {
        let mut group = Vec::new();
        for id in scc {
            let name = name_map.get(&id).unwrap();
            group.push(name.to_string());
        }
        groups.push(group);
    }
    groups
}

pub fn collect_function_groups(mir_program: &Program) -> Vec<Vec<String>> {
    println!("Collecting function groups");
    let mut g = Graph::new();
    let mut id_map = BTreeMap::new();
    let mut name_map = BTreeMap::new();
    for (name, _) in &mir_program.functions {
        let id = g.add_node();
        id_map.insert(name, id);
        name_map.insert(id, name);
    }
    for (name, function) in &mir_program.functions {
        let id = id_map.get(name).unwrap();
        match &function.kind {
            FunctionKind::Normal(exprs) => {
                for e in exprs {
                    match &e.kind {
                        ExprKind::StaticFunctionCall(f_id, _) => {
                            let sub_id = id_map.get(f_id).unwrap();
                            g.add_neighbour(*id, *sub_id);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    let sccs = g.collect_sccs();
    let mut groups = Vec::new();
    for scc in sccs {
        let mut group = Vec::new();
        for id in scc {
            let name = name_map.get(&id).unwrap();
            group.push(name.to_string());
        }
        groups.push(group);
    }
    groups
}
