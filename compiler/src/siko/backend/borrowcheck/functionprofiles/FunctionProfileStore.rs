use std::collections::BTreeMap;

use crate::siko::{
    backend::borrowcheck::functionprofiles::FunctionProfile::FunctionProfile, qualifiedname::QualifiedName,
};

pub struct FunctionProfileStore {
    profiles: BTreeMap<QualifiedName, FunctionProfile>,
}

impl FunctionProfileStore {
    pub fn new() -> Self {
        FunctionProfileStore {
            profiles: BTreeMap::new(),
        }
    }

    pub fn addProfile(&mut self, profile: FunctionProfile) {
        self.profiles.insert(profile.name.clone(), profile);
    }

    pub fn getProfile(&self, name: &QualifiedName) -> &FunctionProfile {
        self.profiles.get(name).expect("Function Profile not found for {name}")
    }
}
