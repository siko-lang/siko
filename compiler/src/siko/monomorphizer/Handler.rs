use std::{cell::RefCell, collections::BTreeMap, fmt::Display, rc::Rc};

use crate::siko::{hir::Instruction::ImplicitContextIndex, location::Location::Location, qualifiedname::QualifiedName};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EffectHandler {
    pub name: QualifiedName,
    pub used: Rc<RefCell<bool>>,
    pub location: Location,
}

impl EffectHandler {
    pub fn new(name: QualifiedName, location: Location) -> Self {
        EffectHandler {
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
pub struct ImplicitHandler {
    pub index: ImplicitContextIndex,
    pub used: Rc<RefCell<bool>>,
    pub location: Location,
}

impl ImplicitHandler {
    pub fn new(index: ImplicitContextIndex, location: Location) -> Self {
        ImplicitHandler {
            index,
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
pub struct HandlerResolution {
    pub effects: BTreeMap<QualifiedName, EffectHandler>,
    pub implicits: BTreeMap<QualifiedName, ImplicitHandler>,
}

impl HandlerResolution {
    pub fn new() -> Self {
        HandlerResolution {
            effects: BTreeMap::new(),
            implicits: BTreeMap::new(),
        }
    }

    pub fn isEmpty(&self) -> bool {
        self.effects.is_empty() && self.implicits.is_empty()
    }

    pub fn addEffectHandler(&mut self, effect: QualifiedName, resolution: QualifiedName, location: Location) {
        let mut handler = EffectHandler::new(resolution, location);
        if let Some(prev) = self.effects.get(&effect) {
            // The handler shadows the prev handler so we clone its used flag
            // if this new handler is used, prev will be marked as used as well
            handler.used = Rc::clone(&prev.used);
        }
        self.effects.insert(effect, handler);
    }

    pub fn getEffectHandler(&self, effect: &QualifiedName) -> Option<&EffectHandler> {
        self.effects.get(effect)
    }

    pub fn addImplicitHandler(&mut self, implicit: QualifiedName, location: Location) {
        let index = ImplicitContextIndex(self.implicits.len());
        let mut handler = ImplicitHandler::new(index, location);
        if let Some(prev) = self.implicits.get(&implicit) {
            // The handler shadows the prev handler so we clone its used flag
            // if this new handler is used, prev will be marked as used as well
            // We also inherit the context index
            handler.index = prev.index.clone();
            handler.used = Rc::clone(&prev.used);
        }
        self.implicits.insert(implicit, handler);
    }

    pub fn getImplicitHandler(&self, effect: &QualifiedName) -> Option<&ImplicitHandler> {
        self.implicits.get(effect)
    }
}

impl Display for HandlerResolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.effects.is_empty() {
            write!(f, "")?;
        } else {
            let effects: Vec<String> = self
                .effects
                .iter()
                .map(|(k, v)| format!("{} -> {}", k.toString(), v.name.toString()))
                .collect();
            write!(f, "{}", effects.join(", "))?;
        }
        if self.implicits.is_empty() {
            write!(f, "")?;
        } else {
            let implicits: Vec<String> = self
                .implicits
                .iter()
                .map(|(k, v)| format!("{} -> {}", k.toString(), v.index))
                .collect();
            write!(f, " implicits: {{{}}}", implicits.join(", "))?;
        }
        Ok(())
    }
}
