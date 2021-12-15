use crate::module::ModuleId;
use crate::types::TypeSignatureId;
use siko_location_info::location_id::LocationId;

#[derive(Debug, Clone)]
pub struct DerivedClass {
    pub name: String,
    pub location_id: LocationId,
}

pub enum Data {
    Adt(Adt),
    Record(Record),
}

#[derive(Debug, Clone)]
pub struct Adt {
    pub name: String,
    pub id: AdtId,
    pub module_id: ModuleId,
    pub type_args: Vec<(String, LocationId)>,
    pub variants: Vec<VariantId>,
    pub location_id: LocationId,
    pub derived_classes: Vec<DerivedClass>,
}

#[derive(Debug, Clone)]
pub struct Variant {
    pub name: String,
    pub id: VariantId,
    pub type_signature_id: TypeSignatureId,
    pub location_id: LocationId,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct AdtId {
    pub id: usize,
}

impl From<usize> for AdtId {
    fn from(id: usize) -> AdtId {
        AdtId { id: id }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct VariantId {
    pub id: usize,
}

impl From<usize> for VariantId {
    fn from(id: usize) -> VariantId {
        VariantId { id: id }
    }
}

#[derive(Debug, Clone)]
pub struct Record {
    pub name: String,
    pub id: RecordId,
    pub module_id: ModuleId,
    pub type_args: Vec<(String, LocationId)>,
    pub fields: Vec<RecordFieldId>,
    pub location_id: LocationId,
    pub external: bool,
    pub derived_classes: Vec<DerivedClass>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecordId {
    pub id: usize,
}

impl From<usize> for RecordId {
    fn from(id: usize) -> RecordId {
        RecordId { id: id }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecordFieldId {
    pub id: usize,
}

impl From<usize> for RecordFieldId {
    fn from(id: usize) -> RecordFieldId {
        RecordFieldId { id: id }
    }
}

#[derive(Debug, Clone)]
pub struct RecordField {
    pub name: String,
    pub id: RecordFieldId,
    pub type_signature_id: TypeSignatureId,
    pub location_id: LocationId,
}
