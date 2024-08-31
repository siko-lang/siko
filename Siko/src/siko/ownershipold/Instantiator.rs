use super::{dataflowprofile::DataFlowProfile::DataFlowProfile, Allocator::Allocator};

pub struct Instantiator {
    pub allocator: Allocator,
}

impl Instantiator {
    pub fn new(allocator: Allocator) -> Instantiator {
        Instantiator {
            allocator: allocator,
        }
    }

    pub fn instantiateProfile(&mut self, profile: DataFlowProfile) -> DataFlowProfile {
        profile
    }
}
