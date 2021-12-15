use crate::types::TypeSignatureId;
use siko_location_info::location_id::LocationId;

#[derive(Debug, Clone)]
pub struct Actor {
    pub id: ActorId,
    pub name: String,
    pub type_signature: TypeSignatureId,
    pub handlers: Vec<ProtocolHandler>,
    pub location_id: LocationId,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct ActorId {
    pub id: usize,
}

impl From<usize> for ActorId {
    fn from(id: usize) -> ActorId {
        ActorId { id: id }
    }
}

#[derive(Debug, Clone)]
pub struct ProtocolHandler {
    pub protocol: String,
    pub protocol_location_id: LocationId,
    pub handler_func: String,
    pub handler_func_location_id: LocationId,
}

#[derive(Debug, Clone)]
pub struct Protocol {
    pub id: ProtocolId,
    pub name: String,
    pub type_signature: TypeSignatureId,
    pub location_id: LocationId,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProtocolId {
    pub id: usize,
}

impl From<usize> for ProtocolId {
    fn from(id: usize) -> ProtocolId {
        ProtocolId { id: id }
    }
}
