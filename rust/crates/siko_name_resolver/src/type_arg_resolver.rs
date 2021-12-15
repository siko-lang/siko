use siko_ir::class::ClassId;
use siko_ir::type_var_generator::TypeVarGenerator;
use siko_location_info::location_id::LocationId;
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct TypeArgInfo {
    pub index: usize,
    pub constraints: Vec<ClassId>,
    pub used: bool,
    pub location_id: LocationId,
}

#[derive(Clone)]
pub struct TypeArgResolver {
    args: BTreeMap<String, TypeArgInfo>,
    type_var_generator: TypeVarGenerator,
}

impl TypeArgResolver {
    pub fn new(type_var_generator: TypeVarGenerator) -> TypeArgResolver {
        TypeArgResolver {
            args: BTreeMap::new(),
            type_var_generator: type_var_generator,
        }
    }

    pub fn add_explicit(
        &mut self,
        arg: String,
        constraints: Vec<ClassId>,
        location_id: LocationId,
    ) -> usize {
        let index = self.type_var_generator.get_new_index();
        let info = TypeArgInfo {
            index: index,
            constraints: constraints,
            used: false,
            location_id: location_id,
        };
        self.args.insert(arg.clone(), info);
        index
    }

    pub fn add_constraint(&mut self, arg: &String, constraint: ClassId) -> bool {
        if let Some(info) = self.args.get_mut(arg) {
            info.constraints.push(constraint);
            info.constraints.sort();
            info.constraints.dedup();
            true
        } else {
            false
        }
    }

    pub fn resolve_arg(&mut self, arg: &String) -> Option<TypeArgInfo> {
        if let Some(info) = self.args.get_mut(arg) {
            info.used = true;
            Some(info.clone())
        } else {
            None
        }
    }

    pub fn contains(&self, arg: &str) -> bool {
        self.args.contains_key(arg)
    }

    pub fn collect_unused_args(&self) -> Vec<(String, LocationId)> {
        self.args
            .iter()
            .filter(|(_, info)| !info.used)
            .map(|(arg, info)| (arg.clone(), info.location_id))
            .collect()
    }

    pub fn reset_unused_flag(&mut self) {
        for (_, info) in &mut self.args {
            info.used = false;
        }
    }
}
