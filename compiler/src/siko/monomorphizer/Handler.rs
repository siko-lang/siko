use std::{cell::RefCell, collections::BTreeMap, fmt::Display, rc::Rc};

use crate::siko::{
    hir::{
        Instruction::{ImplicitContextIndex, ImplicitContextOperation},
        Program::Program,
        Type::Type,
        Variable::Variable,
    },
    location::Location::Location,
    qualifiedname::QualifiedName,
};

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

    pub fn addEffectHandler(&mut self, effect: QualifiedName, resolution: QualifiedName, location: Location) {
        let mut handler = EffectHandler::new(resolution, location);
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

    pub fn getContextTypes(&self, program: &Program) -> Vec<Type> {
        let mut contextTypeMap = BTreeMap::new();
        for (implicitName, handler) in &self.implicits {
            let implicitDef = program.getImplicit(&implicitName).expect("implicit not found in mono");
            let index = handler.index.clone().0;
            contextTypeMap.insert(index, implicitDef.ty.clone());
        }
        let contextTypes = contextTypeMap.values().cloned().collect::<Vec<_>>();
        contextTypes
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
