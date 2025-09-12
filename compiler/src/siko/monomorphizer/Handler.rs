use std::{cell::RefCell, collections::BTreeMap, fmt::Display, rc::Rc};

use crate::siko::{
    hir::{
        Instruction::{ImplicitContextIndex, ImplicitContextOperation},
        Type::Type,
        Variable::Variable,
    },
    location::Location::Location,
    monomorphizer::Monomorphizer::Monomorphizer,
    qualifiedname::QualifiedName,
};

#[derive(Clone, Debug)]
pub struct EffectHandler {
    pub name: QualifiedName,
    pub used: Rc<RefCell<bool>>,
    pub location: Location,
    pub resolution: HandlerResolution,
}

impl PartialEq for EffectHandler {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for EffectHandler {}

impl PartialOrd for EffectHandler {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EffectHandler {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl EffectHandler {
    pub fn new(name: QualifiedName, location: Location, resolution: HandlerResolution) -> Self {
        EffectHandler {
            name,
            used: Rc::new(RefCell::new(false)),
            location,
            resolution,
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
    pub handlers: BTreeMap<QualifiedName, EffectHandler>,
    pub implicits: BTreeMap<QualifiedName, ImplicitHandler>,
}

impl HandlerResolution {
    pub fn new() -> Self {
        HandlerResolution {
            handlers: BTreeMap::new(),
            implicits: BTreeMap::new(),
        }
    }

    pub fn isEmpty(&self) -> bool {
        self.handlers.is_empty() && self.implicits.is_empty()
    }

    pub fn isEmptyImplicits(&self) -> bool {
        self.implicits.is_empty()
    }

    pub fn addEffectHandler(
        &mut self,
        effect: QualifiedName,
        resolvedName: QualifiedName,
        location: Location,
        resolution: HandlerResolution,
    ) {
        let mut handler = EffectHandler::new(resolvedName, location, resolution);
        if let Some(prev) = self.handlers.get(&effect) {
            // The handler shadows the prev handler so we clone its used flag
            // if this new handler is used, prev will be marked as used as well
            handler.used = Rc::clone(&prev.used);
        }
        self.handlers.insert(effect, handler);
    }

    pub fn getEffectHandler(&self, effect: &QualifiedName) -> Option<&EffectHandler> {
        self.handlers.get(effect)
    }

    pub fn addImplicitHandler(
        &mut self,
        implicit: QualifiedName,
        location: Location,
        var: Variable,
    ) -> ImplicitContextOperation {
        let index = ImplicitContextIndex(self.implicits.len());
        let mut handler = ImplicitHandler::new(index, location);
        let op = if let Some(prev) = self.implicits.get(&implicit) {
            // The handler shadows the prev handler so we clone its used flag
            // if this new handler is used, prev will be marked as used as well
            // We also inherit the context index
            handler.index = prev.index.clone();
            handler.used = Rc::clone(&prev.used);
            ImplicitContextOperation::Overwrite(prev.index.clone(), var)
        } else {
            ImplicitContextOperation::Add(handler.index.clone(), var)
        };
        self.implicits.insert(implicit, handler);
        op
    }

    pub fn getImplicitHandler(&self, effect: &QualifiedName) -> Option<&ImplicitHandler> {
        self.implicits.get(effect)
    }

    pub fn getContextTypes(&self, mono: &mut Monomorphizer) -> Vec<Type> {
        let mut contextTypeMap = BTreeMap::new();
        for (implicitName, handler) in &self.implicits {
            let implicitDef = mono
                .program
                .getImplicit(&implicitName)
                .expect("implicit not found in mono");
            let index = handler.index.clone().0;
            contextTypeMap.insert(index, implicitDef.ty.clone());
        }
        let mut contextTypes = contextTypeMap.values().cloned().collect::<Vec<_>>();
        for ty in &mut contextTypes {
            *ty = Type::Ptr(Box::new(mono.processType(ty.clone())));
        }
        contextTypes
    }

    pub fn merge(&mut self, other: &HandlerResolution) {
        for (k, v) in &other.implicits {
            self.implicits.insert(k.clone(), v.clone());
        }
    }
}

impl Display for HandlerResolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.handlers.is_empty() {
            write!(f, "")?;
        } else {
            let effects: Vec<String> = self
                .handlers
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
