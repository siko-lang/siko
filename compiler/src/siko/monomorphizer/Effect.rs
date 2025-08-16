use std::{collections::BTreeMap, fmt::Display};

use crate::siko::qualifiedname::QualifiedName;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EffectResolution {
    pub effects: BTreeMap<QualifiedName, QualifiedName>,
}

impl EffectResolution {
    pub fn new() -> Self {
        EffectResolution {
            effects: BTreeMap::new(),
        }
    }

    pub fn isEmpty(&self) -> bool {
        self.effects.is_empty()
    }

    pub fn add(&mut self, effect: QualifiedName, resolution: QualifiedName) {
        self.effects.insert(effect, resolution);
    }

    pub fn get(&self, effect: &QualifiedName) -> Option<&QualifiedName> {
        self.effects.get(effect)
    }
}

impl Display for EffectResolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.effects.is_empty() {
            write!(f, "")
        } else {
            let effects: Vec<String> = self
                .effects
                .iter()
                .map(|(k, v)| format!("{} -> {}", k.toString(), v.toString()))
                .collect();
            write!(f, "{}", effects.join(", "))
        }
    }
}
