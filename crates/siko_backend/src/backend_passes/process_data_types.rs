use siko_mir::data::TypeDef;
use siko_mir::data::TypeDefId;
use siko_mir::program::Program;
use siko_mir::types::Modifier;
use siko_mir::types::Type;
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
                        if let Some(id) = item.ty.get_typedef_id_opt() {
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

fn calculate_modifier_variables(groups: &Vec<DependencyGroup<TypeDefId>>, program: &mut Program) {
    let mut modifiers: BTreeMap<TypeDefId, Vec<usize>> = BTreeMap::new();
    for group in groups {
        let mut modifier_vars = Vec::new();
        for group_item in &group.items {
            let typedef = program.typedefs.get(group_item);
            match typedef {
                TypeDef::Adt(adt) => {
                    for variant in &adt.variants {
                        for item in &variant.items {
                            if let Type::Named(Modifier::Var(var), id) = item.ty {
                                modifier_vars.push(var);
                                if let Some(modifiers) = modifiers.get(&id) {
                                    modifier_vars.extend(modifiers);
                                }
                            }
                        }
                    }
                }
                TypeDef::Record(record) => {
                    for field in &record.fields {
                        if let Type::Named(Modifier::Var(var), id) = field.ty {
                            modifier_vars.push(var);
                            if let Some(modifiers) = modifiers.get(&id) {
                                modifier_vars.extend(modifiers);
                            }
                        }
                    }
                }
            }
        }
        for group_item in &group.items {
            modifiers.insert(*group_item, modifier_vars.clone());
            let typedef = program.typedefs.get_mut(group_item);
            match typedef {
                TypeDef::Adt(adt) => {
                    adt.modifier_args = modifier_vars.clone();
                    //println!("ADT {}/{} {:?}", adt.module, adt.name, adt.modifier_args);
                }
                TypeDef::Record(record) => {
                    record.modifier_args = modifier_vars.clone();
                    /*
                    println!(
                        "RECORD {}/{} {:?}",
                        record.module, record.name, record.modifier_args
                    );
                    */
                }
            }
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
                    if let Some(id) = item.ty.get_typedef_id_opt() {
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
                    for item in &mut variant.items {
                        if let Some(id) = item.ty.get_typedef_id_opt() {
                            if group.items.contains(&id) {
                                item.ty = Type::Named(Modifier::Boxed, id);
                            }
                        }
                    }
                }
            }
            TypeDef::Record(record) => {
                for field in &mut record.fields {
                    if let Some(id) = field.ty.get_typedef_id_opt() {
                        if group.items.contains(&id) {
                            field.ty = Type::Named(Modifier::Boxed, id);
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
    calculate_modifier_variables(&groups, program);
    calculate_boxed_members(&groups, program);
}
