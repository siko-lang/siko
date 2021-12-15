use crate::class::ClassId;
use crate::type_var_generator::TypeVarGenerator;
use crate::types::Type;
use crate::unifier::Unifier;
use siko_location_info::location_id::LocationId;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct DeriveInfo {
    pub class_id: ClassId,
    pub instance_index: usize,
    pub location_id: LocationId,
}

#[derive(Clone)]
pub struct AdtTypeInfo {
    pub adt_type: Type,
    pub variant_types: Vec<VariantTypeInfo>,
    pub derived_classes: Vec<DeriveInfo>,
}

impl AdtTypeInfo {
    pub fn apply(&mut self, unifier: &Unifier) -> bool {
        let mut changed = false;
        for variant_type in &mut self.variant_types {
            changed = variant_type.apply(unifier) || changed;
        }
        changed = self.adt_type.apply(unifier) || changed;
        changed
    }

    pub fn duplicate(&self, type_var_generator: &mut TypeVarGenerator) -> AdtTypeInfo {
        let mut arg_map = BTreeMap::new();
        AdtTypeInfo {
            adt_type: self.adt_type.duplicate(&mut arg_map, type_var_generator),
            variant_types: self
                .variant_types
                .iter()
                .map(|ty| ty.duplicate(&mut arg_map, type_var_generator))
                .collect(),
            derived_classes: self.derived_classes.clone(),
        }
    }
}

#[derive(Clone)]
pub struct VariantTypeInfo {
    pub item_types: Vec<(Type, LocationId)>,
}

impl VariantTypeInfo {
    pub fn apply(&mut self, unifier: &Unifier) -> bool {
        let mut changed = false;
        for item_type in &mut self.item_types {
            changed = item_type.0.apply(unifier) || changed;
        }
        changed
    }

    pub fn duplicate(
        &self,
        arg_map: &mut BTreeMap<usize, usize>,
        type_var_generator: &mut TypeVarGenerator,
    ) -> VariantTypeInfo {
        VariantTypeInfo {
            item_types: self
                .item_types
                .iter()
                .map(|(ty, location)| (ty.duplicate(arg_map, type_var_generator), *location))
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RecordTypeInfo {
    pub record_type: Type,
    pub field_types: Vec<(Type, LocationId)>,
    pub derived_classes: Vec<DeriveInfo>,
}

impl RecordTypeInfo {
    pub fn apply(&mut self, unifier: &Unifier) -> bool {
        let mut changed = false;
        for field_type in &mut self.field_types {
            changed = field_type.0.apply(unifier) || changed;
        }
        changed = self.record_type.apply(unifier) || changed;
        changed
    }

    pub fn duplicate(&self, type_var_generator: &mut TypeVarGenerator) -> RecordTypeInfo {
        let mut arg_map = BTreeMap::new();
        RecordTypeInfo {
            record_type: self.record_type.duplicate(&mut arg_map, type_var_generator),
            field_types: self
                .field_types
                .iter()
                .map(|(ty, location)| (ty.duplicate(&mut arg_map, type_var_generator), *location))
                .collect(),
            derived_classes: self.derived_classes.clone(),
        }
    }
}
