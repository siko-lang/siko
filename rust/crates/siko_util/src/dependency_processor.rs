use std::collections::BTreeMap;
use std::collections::BTreeSet;

#[derive(Clone, Debug)]
pub struct DependencyGroup<T> {
    pub items: BTreeSet<T>,
}

impl<T: Ord> DependencyGroup<T> {
    pub fn new() -> DependencyGroup<T> {
        DependencyGroup {
            items: BTreeSet::new(),
        }
    }
}

struct DependencyBucket<T> {
    dependencies: Vec<T>,
}

struct GroupTable<T> {
    table: BTreeMap<T, usize>,
}

impl<T: Ord> GroupTable<T> {
    fn new() -> GroupTable<T> {
        GroupTable {
            table: BTreeMap::new(),
        }
    }

    fn merge(&mut self, id1: T, id2: T) {
        let group_id1 = *self.table.get(&id1).expect("Item in group table not found");
        let group_id2 = *self.table.get(&id2).expect("Item in group table not found");
        for (_, group_id) in self.table.iter_mut() {
            if *group_id == group_id1 {
                *group_id = group_id2;
            }
        }
    }
}

pub trait DependencyCollector<T> {
    fn collect(&self, item: T) -> Vec<T>;
}

pub struct DependencyProcessor<T> {
    items: Vec<T>,
    items_dependencies: BTreeMap<T, DependencyBucket<T>>,
}

impl<T: Ord + Clone> DependencyProcessor<T> {
    pub fn new(items: Vec<T>) -> DependencyProcessor<T> {
        DependencyProcessor {
            items: items,
            items_dependencies: BTreeMap::new(),
        }
    }

    fn collect_dependencies(&mut self, dependency_collector: &dyn DependencyCollector<T>) {
        for item in &self.items {
            let deps: Vec<_> = dependency_collector.collect(item.clone());
            //println!("{} deps {}", id, format_list(&deps[..]));
            self.items_dependencies
                .insert(item.clone(), DependencyBucket { dependencies: deps });
        }
    }

    fn depends_on(&self, user: &T, used_item: &T, visited: &mut BTreeSet<T>) -> bool {
        if !visited.insert(user.clone()) {
            return false;
        }
        let deps = self
            .items_dependencies
            .get(user)
            .expect("dep info not found");
        if deps.dependencies.contains(used_item) {
            return true;
        } else {
            for dep in &deps.dependencies {
                if self.depends_on(dep, used_item, visited) {
                    return true;
                }
            }
        }
        false
    }

    pub fn process_items(
        mut self,
        dependency_collector: &dyn DependencyCollector<T>,
    ) -> Vec<DependencyGroup<T>> {
        self.collect_dependencies(dependency_collector);

        let mut group_table = GroupTable::new();

        // 1. create a dependency group in the table for every single item
        for (index, (id, _)) in self.items_dependencies.iter().enumerate() {
            group_table.table.insert(id.clone(), index);
        }

        // 2. merge the groups in case of circular dependencies
        for (id, bucket) in &self.items_dependencies {
            for dependency in &bucket.dependencies {
                let mut visited = BTreeSet::new();
                // if my dependency depends on me then the dependeny is circular,
                // merge the groups
                if self.depends_on(dependency, id, &mut visited) {
                    group_table.merge(id.clone(), dependency.clone());
                }
            }
        }

        // 3. initialize group lookup table
        let mut group_lookup_table = BTreeMap::new();
        let mut unprocessed_groups = BTreeSet::new();
        for (id, group_id) in &group_table.table {
            let group = group_lookup_table
                .entry(*group_id)
                .or_insert_with(|| DependencyGroup::new());
            group.items.insert(id.clone());
            unprocessed_groups.insert(*group_id);
        }

        let mut processed_items = BTreeSet::new();

        let mut ordered_groups = Vec::new();

        // 4. try to process groups in the right order
        while !unprocessed_groups.is_empty() {
            let copied = unprocessed_groups.clone();
            let mut found = false;
            for group_id in &copied {
                let group = group_lookup_table.get(group_id).expect("group not found");
                assert!(!group.items.is_empty());
                let mut dep_missing = false;
                for item in &group.items {
                    let deps = self.items_dependencies.get(item).expect("dep not found");
                    for dep in &deps.dependencies {
                        if !processed_items.contains(dep) && !group.items.contains(dep) {
                            dep_missing = true;
                        }
                    }
                }
                if !dep_missing {
                    //println!("Processing group {}", group_id);
                    ordered_groups.push(group.clone());
                    for item in &group.items {
                        processed_items.insert(item);
                    }
                    unprocessed_groups.remove(group_id);
                    found = true;
                    break;
                }
            }
            if !found {
                panic!("Cyclic dep groups");
            }
        }

        ordered_groups
    }
}
