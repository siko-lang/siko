use super::DataFlowProfile::DataFlowProfile;
use crate::siko::qualifiedname::QualifiedName;
use std::collections::BTreeMap;

pub struct DataFlowProfileStore {
    profiles: BTreeMap<QualifiedName, DataFlowProfile>,
}

impl DataFlowProfileStore {
    pub fn new() -> DataFlowProfileStore {
        DataFlowProfileStore {
            profiles: BTreeMap::new(),
        }
    }

    pub fn addProfile(&mut self, name: QualifiedName, profile: DataFlowProfile) {
        //println!("addProfile: {} -> {:?}", name, profile);
        self.profiles.insert(name, profile);
    }

    pub fn getProfile(&self, name: &QualifiedName) -> &DataFlowProfile {
        self.profiles
            .get(name)
            .expect("data flow profile not found")
    }
}
