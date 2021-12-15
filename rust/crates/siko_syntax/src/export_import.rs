use siko_location_info::location_id::LocationId;

#[derive(Debug, Clone)]
pub enum EIItem {
    Named(String),
    Group(EIGroup),
}

#[derive(Debug, Clone)]
pub enum EIMember {
    Specific(String),
    All,
}

#[derive(Debug, Clone)]
pub struct EIMemberInfo {
    pub member: EIMember,
    pub location_id: LocationId,
}

#[derive(Debug, Clone)]
pub struct EIGroup {
    pub name: String,
    pub members: Vec<EIMemberInfo>,
}

#[derive(Debug, Clone)]
pub struct EIItemInfo {
    pub item: EIItem,
    pub location_id: LocationId,
}

#[derive(Debug, Clone)]
pub enum EIList {
    ImplicitAll,
    Explicit(Vec<EIItemInfo>),
}
