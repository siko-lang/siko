use std::collections::BTreeMap;

use crate::siko::qualifiedname::QualifiedName;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EffectResolution {
    pub effects: BTreeMap<QualifiedName, QualifiedName>,
}
