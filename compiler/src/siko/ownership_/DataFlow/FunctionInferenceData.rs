use std::collections::BTreeMap;

use crate::siko::{
    hir::{Function::InstructionId, Type::Type},
    qualifiedname::QualifiedName,
};

use super::DataFlowProfile::DataFlowProfile;

#[derive(Clone)]
pub struct FunctionInferenceData {
    pub name: QualifiedName,
    pub profile: DataFlowProfile,
    pub instruction_types: BTreeMap<InstructionId, Type>,
    pub value_types: BTreeMap<String, Type>,
}

impl FunctionInferenceData {
    pub fn new(name: QualifiedName, profile: DataFlowProfile) -> FunctionInferenceData {
        FunctionInferenceData {
            name: name,
            profile: profile,
            instruction_types: BTreeMap::new(),
            value_types: BTreeMap::new(),
        }
    }

    pub fn dump(&self) {
        println!("-----------------");
        println!("profile {} = {}", self.name, self.profile);
        if !self.instruction_types.is_empty() {
            for (id, ty) in &self.instruction_types {
                println!("{} {}", id, ty);
            }
            if !self.value_types.is_empty() {
                println!(".................");
                for (id, ty) in &self.value_types {
                    println!("{} {}", id, ty);
                }
            }
        }
        println!("-----------------");
    }

    pub fn getInstructionType(&self, id: InstructionId) -> Type {
        self.instruction_types
            .get(&id)
            .cloned()
            .expect("no instruction type")
    }

    pub fn getValueType(&self, id: &String) -> Type {
        self.value_types.get(id).cloned().expect("no value type")
    }
}
