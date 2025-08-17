use std::{cell::RefCell, collections::BTreeMap, fmt::Display, rc::Rc};

use crate::siko::{location::Location::Location, qualifiedname::QualifiedName};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Handler {
    pub name: QualifiedName,
    pub used: Rc<RefCell<bool>>,
    pub location: Location,
}

impl Handler {
    pub fn new(name: QualifiedName, location: Location) -> Self {
        Handler {
            name,
            used: Rc::new(RefCell::new(false)),
            location,
        }
    }

    pub fn markUsed(&self) {
        *self.used.borrow_mut() = true;
    }

    pub fn isUsed(&self) -> bool {
        *self.used.borrow()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EffectResolution {
    pub effects: BTreeMap<QualifiedName, Handler>,
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

    pub fn add(&mut self, effect: QualifiedName, resolution: QualifiedName, location: Location) {
        let mut handler = Handler::new(resolution, location);
        if let Some(prev) = self.effects.get(&effect) {
            // The handler shadows the prev handler so we clone its used flag
            // if this new handler is used, prev will be marked as used as well
            handler.used = Rc::clone(&prev.used);
        }
        self.effects.insert(effect, handler);
    }

    pub fn get(&self, effect: &QualifiedName) -> Option<&Handler> {
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
                .map(|(k, v)| format!("{} -> {}", k.toString(), v.name.toString()))
                .collect();
            write!(f, "{}", effects.join(", "))
        }
    }
}
