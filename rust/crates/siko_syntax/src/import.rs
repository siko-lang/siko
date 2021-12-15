use crate::export_import::EIList;
use siko_location_info::location_id::LocationId;

#[derive(Debug, Clone)]
pub struct HiddenItem {
    pub name: String,
    pub location_id: LocationId,
}

#[derive(Debug, Clone)]
pub enum ImportKind {
    Hiding(Vec<HiddenItem>),
    ImportList {
        items: EIList,
        alternative_name: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub struct Import {
    pub id: ImportId,
    pub module_path: String,
    pub kind: ImportKind,
    pub location_id: Option<LocationId>,
    pub implicit: bool,
}

impl Import {
    pub fn get_location(&self) -> LocationId {
        self.location_id
            .expect("Trying to get the location of an internal import")
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct ImportId {
    pub id: usize,
}

impl From<usize> for ImportId {
    fn from(id: usize) -> ImportId {
        ImportId { id: id }
    }
}
