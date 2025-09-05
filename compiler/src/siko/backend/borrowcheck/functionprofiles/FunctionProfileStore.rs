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

    pub fn addProfile(&mut self, profile: FunctionProfile) -> bool {
        if let Some(old) = self.profiles.insert(profile.name.clone(), profile.clone()) {
            old != profile
        } else {
            true
        }
    }

    pub fn getProfile(&self, name: &QualifiedName) -> Option<&FunctionProfile> {
        self.profiles.get(name)
    }
}
