use crate::common::ClassMemberTypeInfo;
use crate::common::FunctionTypeInfo;
use crate::common::FunctionTypeInfoStore;
use siko_ir::class::ClassMemberId;
use siko_ir::data::TypeDefId;
use siko_ir::data_type_info::AdtTypeInfo;
use siko_ir::data_type_info::RecordTypeInfo;
use siko_ir::function::FunctionId;
use siko_ir::type_var_generator::TypeVarGenerator;
use siko_ir::types::Type;
use std::collections::BTreeMap;

pub struct TypeInfoProvider {
    pub type_var_generator: TypeVarGenerator,
    pub class_member_type_info_map: BTreeMap<ClassMemberId, ClassMemberTypeInfo>,
    pub adt_type_info_map: BTreeMap<TypeDefId, AdtTypeInfo>,
    pub function_type_info_store: FunctionTypeInfoStore,
    pub record_type_info_map: BTreeMap<TypeDefId, RecordTypeInfo>,
}

impl TypeInfoProvider {
    pub fn new(type_var_generator: TypeVarGenerator) -> TypeInfoProvider {
        TypeInfoProvider {
            type_var_generator: type_var_generator,
            class_member_type_info_map: BTreeMap::new(),
            adt_type_info_map: BTreeMap::new(),
            function_type_info_store: FunctionTypeInfoStore::new(),
            record_type_info_map: BTreeMap::new(),
        }
    }

    pub fn get_function_type(&mut self, function_id: &FunctionId, clone: bool) -> FunctionTypeInfo {
        let ty = &self.function_type_info_store.get(&function_id).clone();
        if clone {
            ty.duplicate(&mut self.type_var_generator)
                .remove_fixed_types()
        } else {
            ty.clone()
        }
    }

    pub fn get_adt_type_info(&mut self, typedef_id: &TypeDefId) -> AdtTypeInfo {
        let adt_type_info = self
            .adt_type_info_map
            .get(typedef_id)
            .expect("Adt type info not found");
        adt_type_info.duplicate(&mut self.type_var_generator)
    }

    pub fn get_record_type_info(&mut self, typedef_id: &TypeDefId) -> RecordTypeInfo {
        let record_type_info = self
            .record_type_info_map
            .get(typedef_id)
            .expect("Record type info not found");
        record_type_info.duplicate(&mut self.type_var_generator)
    }

    pub fn get_class_member_type(&mut self, class_member_id: &ClassMemberId) -> Type {
        let class_member_type_info = self
            .class_member_type_info_map
            .get(class_member_id)
            .expect("Class member type info not found");
        let mut arg_map = BTreeMap::new();
        let function_type = class_member_type_info
            .ty
            .duplicate(&mut arg_map, &mut self.type_var_generator)
            .remove_fixed_types();
        function_type
    }
}
