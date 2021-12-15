use siko_ir::expr::ExprId;
use siko_ir::function::FunctionId;
use siko_ir::program::Program;
use siko_ir::type_var_generator::TypeVarGenerator;
use siko_ir::types::Type;
use siko_ir::unifier::Unifier;
use std::collections::BTreeMap;

pub struct FunctionTypeInfoStore {
    function_type_info_map: BTreeMap<FunctionId, FunctionTypeInfo>,
}

impl FunctionTypeInfoStore {
    pub fn new() -> FunctionTypeInfoStore {
        FunctionTypeInfoStore {
            function_type_info_map: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, id: FunctionId, function_type_info: FunctionTypeInfo) {
        self.function_type_info_map.insert(id, function_type_info);
    }

    pub fn get(&self, id: &FunctionId) -> &FunctionTypeInfo {
        self.function_type_info_map
            .get(id)
            .expect("Function type info not found")
    }

    pub fn get_mut(&mut self, id: &FunctionId) -> &mut FunctionTypeInfo {
        self.function_type_info_map
            .get_mut(id)
            .expect("Function type info not found")
    }

    pub fn dump(&self, program: &Program) {
        for (_, function) in &self.function_type_info_map {
            println!(
                "{} {}",
                function.displayed_name,
                function.function_type.get_resolved_type_string(program)
            );
        }
    }

    pub fn save_function_types(&self, program: &mut Program) {
        for (function_id, function) in &self.function_type_info_map {
            program
                .function_types
                .insert(*function_id, function.function_type.clone());
        }
    }
}

#[derive(Clone)]
pub struct FunctionTypeInfo {
    pub displayed_name: String,
    pub args: Vec<Type>,
    pub typed: bool,
    pub result: Type,
    pub function_type: Type,
    pub body: Option<ExprId>,
}

impl FunctionTypeInfo {
    pub fn apply(&mut self, unifier: &Unifier) -> bool {
        let mut changed = false;
        for arg in &mut self.args {
            if arg.apply(unifier) {
                changed = true;
            }
        }
        if self.result.apply(unifier) {
            changed = true;
        }
        if self.function_type.apply(unifier) {
            changed = true;
        }
        changed
    }

    pub fn duplicate(&self, type_var_generator: &mut TypeVarGenerator) -> FunctionTypeInfo {
        let mut arg_map = BTreeMap::new();
        FunctionTypeInfo {
            displayed_name: self.displayed_name.clone(),
            args: self
                .args
                .iter()
                .map(|ty| ty.duplicate(&mut arg_map, type_var_generator))
                .collect(),
            typed: self.typed,
            result: self.result.duplicate(&mut arg_map, type_var_generator),
            function_type: self
                .function_type
                .duplicate(&mut arg_map, type_var_generator),
            body: self.body,
        }
    }

    pub fn remove_fixed_types(&self) -> FunctionTypeInfo {
        FunctionTypeInfo {
            displayed_name: self.displayed_name.clone(),
            args: self.args.iter().map(|ty| ty.remove_fixed_types()).collect(),
            typed: self.typed,
            result: self.result.remove_fixed_types(),
            function_type: self.function_type.remove_fixed_types(),
            body: self.body,
        }
    }
}

pub struct ClassMemberTypeInfo {
    pub ty: Type,
}
