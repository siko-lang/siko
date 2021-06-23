use crate::mir::*;
use crate::scc::*;
use std::collections::BTreeMap;

pub fn collect_data_groups(mir_program: &Program) -> Vec<Vec<String>> {
    println!("Collecting data groups");
    let mut g = Graph::new();
    let mut id_map = BTreeMap::new();
    let mut name_map = BTreeMap::new();
    for (name, _) in &mir_program.adts {
        let id = g.add_node();
        id_map.insert(name, id);
        name_map.insert(id, name);
    }
    for (name, _) in &mir_program.records {
        let id = g.add_node();
        id_map.insert(name, id);
        name_map.insert(id, name);
    }
    for (name, adt) in &mir_program.adts {
        let id = id_map.get(name).unwrap();
        for v in &adt.variants {
            let sub_id = id_map.get(&v.ty).unwrap();
            if let Some(member) = mir_program.records.get(&v.ty) {
                if let Some(externals) = &member.externals {
                    for e in externals {
                        let sub_id = id_map.get(&e).unwrap();
                        g.add_neighbour(*id, *sub_id);
                    }
                    continue;
                }
            }
            g.add_neighbour(*id, *sub_id);
        }
    }
    for (name, record) in &mir_program.records {
        let id = id_map.get(name).unwrap();
        for f in &record.fields {
            let sub_id = id_map.get(&f.ty).unwrap();
            if let Some(member) = mir_program.records.get(&f.ty) {
                if let Some(externals) = &member.externals {
                    for e in externals {
                        let sub_id = id_map.get(&e).unwrap();
                        g.add_neighbour(*id, *sub_id);
                    }
                    continue;
                }
            }
            g.add_neighbour(*id, *sub_id);
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
