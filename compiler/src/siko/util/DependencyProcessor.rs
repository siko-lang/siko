use std::{
    collections::BTreeMap,
    fmt::{Debug, Display},
};

use super::SCC::{self, Graph};

#[derive(Clone)]
pub struct DependencyGroup<T> {
    pub items: Vec<T>,
}

impl<T: Debug> Debug for DependencyGroup<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.items)
    }
}

fn createIdMaps<T: Ord + Clone>(
    graph: &mut Graph,
    all_dependencies: &BTreeMap<T, Vec<T>>,
) -> (BTreeMap<SCC::NodeId, T>, BTreeMap<T, SCC::NodeId>) {
    let mut id_item_map = BTreeMap::new();
    let mut item_id_map = BTreeMap::new();
    for key in all_dependencies.keys() {
        let id = graph.addNode();
        id_item_map.insert(id, key.clone());
        item_id_map.insert(key.clone(), id);
    }
    (id_item_map, item_id_map)
}

fn initGraph<T: Ord + Clone + Display + Debug>(
    graph: &mut Graph,
    item_id_map: &BTreeMap<T, SCC::NodeId>,
    all_dependencies: &BTreeMap<T, Vec<T>>,
) {
    for (item, deps) in all_dependencies {
        let item_id = item_id_map.get(item).unwrap();
        for dep in deps {
            if let Some(dep_id) = item_id_map.get(dep) {
                graph.addNeighbour(*item_id, *dep_id);
            } else {
                panic!("Dependency {:?} not found in item_id_map", dep);
            }
        }
    }
}

pub fn processDependencies<T: Ord + Clone + Display + Debug>(
    all_dependencies: &BTreeMap<T, Vec<T>>,
) -> Vec<DependencyGroup<T>> {
    let mut graph = Graph::new();
    let (id_item_map, item_id_map) = createIdMaps(&mut graph, &all_dependencies);
    initGraph(&mut graph, &item_id_map, &all_dependencies);
    let sccs = graph.collectSCCs();
    let mut ordered_groups = Vec::new();
    for scc in sccs {
        let mut items = Vec::new();
        for i in scc {
            items.push(id_item_map.get(&i).unwrap().clone());
        }
        let group = DependencyGroup { items: items };
        ordered_groups.push(group);
    }
    return ordered_groups;
}
