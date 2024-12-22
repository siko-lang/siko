use std::{
    collections::BTreeMap,
    fmt::{Debug, Display},
};

use super::{
    Apply::{Apply, ApplyVariable},
    Function::Variable,
    Type::Type,
    Unification::unify,
};

#[derive(Debug)]
pub struct TypeSubstitution {
    substitutions: BTreeMap<Type, Type>,
}

impl TypeSubstitution {
    pub fn new() -> TypeSubstitution {
        TypeSubstitution {
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
        print!("[");
        for (index, (key, value)) in self.substitutions.iter().enumerate() {
            if index == 0 {
                write!(f, "{}: {}", key, value)?;
            } else {
                write!(f, ", {}: {}", key, value)?;
            }
        }
        print!("]");
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

    pub fn add(&mut self, old: Variable, new: Variable) {
        if old == new {
            return;
        }
        assert_ne!(old, new);
        self.substitutions.insert(old, new);
    }

    pub fn get(&self, old: Variable) -> Variable {
        match self.substitutions.get(&old) {
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

pub fn createTypeSubstitutionFrom(ty1: &Vec<Type>, ty2: &Vec<Type>) -> TypeSubstitution {
    let mut sub = TypeSubstitution::new();
    for (ty1, ty2) in ty1.iter().zip(ty2) {
        unify(&mut sub, ty1, ty2, true).expect("Unification failed");
    }
    sub
}
