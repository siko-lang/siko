use siko_mir::data::TypeDef;
use siko_mir::data::TypeDefId;
use siko_mir::program::Program;
use siko_mir::types::Type;
use siko_util::Counter;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use siko_util::dependency_processor::DependencyCollector;
use siko_util::dependency_processor::DependencyGroup;
use siko_util::dependency_processor::DependencyProcessor;

pub struct DataDependencyProcessor<'a> {
    program: &'a Program,
}

impl<'a> DataDependencyProcessor<'a> {
    pub fn new(program: &'a Program) -> DataDependencyProcessor<'a> {
        DataDependencyProcessor { program: program }
    }

    pub fn process_typedefs(&self) -> Vec<DependencyGroup<TypeDefId>> {
        let mut typedefs = Vec::new();
        for (id, _) in &self.program.typedefs.items {
            typedefs.push(*id);
        }

        let dep_processor = DependencyProcessor::new(typedefs);
        let ordered_data_groups = dep_processor.process_items(self);

        ordered_data_groups
    }
}

impl<'a> DependencyCollector<TypeDefId> for DataDependencyProcessor<'a> {
    fn collect(&self, typedef_id: TypeDefId) -> Vec<TypeDefId> {
        let typedef = self.program.typedefs.get(&typedef_id);
        let mut deps = BTreeSet::new();
        match typedef {
            TypeDef::Adt(adt) => {
                for variant in &adt.variants {
                    for item in &variant.items {
                        if let Some(id) = item.get_typedef_id_opt() {
                            deps.insert(id);
                        }
                    }
                }
            }
            TypeDef::Record(record) => {
                for field in &record.fields {
                    if let Some(id) = field.ty.get_typedef_id_opt() {
                        deps.insert(id);
                    }
                }
            }
        }

        deps.into_iter().collect()
    }
}

fn calculate_lifetime_variables(groups: &Vec<DependencyGroup<TypeDefId>>, program: &Program) {
    let mut lifetimes: BTreeMap<TypeDefId, Vec<String>> = BTreeMap::new();
    for group in groups {
        let mut counter = Counter::new();
        let mut lifetime_vars = Vec::new();
        for item in &group.items {
            let typedef = program.typedefs.get(item);
            match typedef {
                TypeDef::Adt(adt) => {
                    for variant in &adt.variants {
                        for item in &variant.items {
                            if let Some(id) = item.get_typedef_id_opt() {
                                lifetime_vars.push(format!("{}", counter.next()));
                                if let Some(lifetimes) = lifetimes.get(&id) {
                                    for _ in 0..lifetimes.len() {
                                        lifetime_vars.push(format!("{}", counter.next()));
                                    }
                                }
                            }
                        }
                    }
                }
                TypeDef::Record(record) => {
                    for field in &record.fields {
                        if let Some(id) = field.ty.get_typedef_id_opt() {
                            lifetime_vars.push(format!("{}", counter.next()));
                            if let Some(lifetimes) = lifetimes.get(&id) {
                                for _ in 0..lifetimes.len() {
                                    lifetime_vars.push(format!("{}", counter.next()));
                                }
                            }
                        }
                    }
                }
            }
        }
        for item in &group.items {
            lifetimes.insert(*item, lifetime_vars.clone());
        }
    }
}

fn get_indirection_count(
    typedef_id: TypeDefId,
    program: &Program,
    group: &DependencyGroup<TypeDefId>,
) -> usize {
    let mut indirection_count = 0;
    let typedef = program.typedefs.get(&typedef_id);
    match typedef {
        TypeDef::Adt(adt) => {
            for variant in &adt.variants {
                for item in &variant.items {
                    if let Some(id) = item.get_typedef_id_opt() {
                        if group.items.contains(&id) {
                            indirection_count += 1;
                        }
                    }
                }
            }
        }
        TypeDef::Record(record) => {
            for field in &record.fields {
                if let Some(id) = field.ty.get_typedef_id_opt() {
                    if group.items.contains(&id) {
                        indirection_count += 1;
                    }
                }
            }
        }
    }
    indirection_count
}

fn calculate_boxed_members(groups: &Vec<DependencyGroup<TypeDefId>>, program: &mut Program) {
    for group in groups {
        let min = group
            .items
            .iter()
            .min_by(|a, b| {
                get_indirection_count(*a.clone(), program, group).cmp(&get_indirection_count(
                    *b.clone(),
                    program,
                    group,
                ))
            })
            .expect("empty group");
        let typedef = program.typedefs.get_mut(&min);
        match typedef {
            TypeDef::Adt(adt) => {
                for variant in &mut adt.variants {
                    for item_type in &mut variant.items {
                        if let Some(id) = item_type.get_typedef_id_opt() {
                            if group.items.contains(&id) {
                                *item_type = Type::Boxed(Box::new(item_type.clone()));
                            }
                        }
                    }
                }
            }
            TypeDef::Record(record) => {
                for field in &mut record.fields {
                    if let Some(id) = field.ty.get_typedef_id_opt() {
                        if group.items.contains(&id) {
                            field.ty = Type::Boxed(Box::new(field.ty.clone()));
                        }
                    }
                }
            }
        }
    }
}

pub fn process_data_types(program: &mut Program) {
    let processor = DataDependencyProcessor::new(program);
    let groups = processor.process_typedefs();
    calculate_lifetime_variables(&groups, program);
    calculate_boxed_members(&groups, program);
}
