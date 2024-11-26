use std::{
    collections::BTreeMap,
    fmt::{Debug, Display},
};

use crate::siko::location::Location::Location;

use super::{
    Apply::{Apply, ApplyVariable},
    Function::Variable,
    Type::Type,
};

#[derive(Debug)]
pub struct TypeSubstitution {
    pub forced: bool,
    substitutions: BTreeMap<Type, Type>,
}

impl TypeSubstitution {
    pub fn new() -> TypeSubstitution {
        TypeSubstitution {
            forced: false,
            substitutions: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, old: Type, new: Type) {
        assert_ne!(old, new);
        self.substitutions.insert(old, new);
    }

    pub fn get(&self, old: Type) -> Type {
        match self.substitutions.get(&old) {
            Some(new) => new.apply(self),
            None => old,
        }
    }
}

impl Display for TypeSubstitution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, (key, value)) in self.substitutions.iter().enumerate() {
            if index == 0 {
                write!(f, "{}: {}", key, value)?;
            } else {
                write!(f, ", {}: {}", key, value)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct VariableSubstitution {
    substitutions: BTreeMap<Variable, Variable>,
}

impl VariableSubstitution {
    pub fn new() -> VariableSubstitution {
        VariableSubstitution {
            substitutions: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, mut old: Variable, new: Variable) {
        old.location = Location::empty();
        assert_ne!(old, new);
        self.substitutions.insert(old, new);
    }

    pub fn get(&self, old: Variable) -> Variable {
        let mut old2 = old.clone();
        old2.location = Location::empty();
        match self.substitutions.get(&old2) {
            Some(new) => new.applyVar(self),
            None => old,
        }
    }
}

impl Display for VariableSubstitution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, (key, value)) in self.substitutions.iter().enumerate() {
            if index == 0 {
                write!(f, "{}: {}", key, value)?;
            } else {
                write!(f, ", {}: {}", key, value)?;
            }
        }
        Ok(())
    }
}
