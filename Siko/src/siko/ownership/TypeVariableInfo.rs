use std::{collections::BTreeMap, fmt::Display};

pub struct Substitution {
    ownershipVars: BTreeMap<OwnershipTypeVariable, OwnershipTypeVariable>,
    groupVars: BTreeMap<GroupTypeVariable, GroupTypeVariable>,
}

impl Substitution {
    pub fn addOwnershipVar(&mut self, v: OwnershipTypeVariable, other: OwnershipTypeVariable) {
        if v != other {
            if v < other {
                self.ownershipVars.insert(other, v);
            } else {
                self.ownershipVars.insert(v, other);
            }
        }
    }

    pub fn addGroupVar(&mut self, v: GroupTypeVariable, other: GroupTypeVariable) {
        if v != other {
            if v < other {
                self.groupVars.insert(other, v);
            } else {
                self.groupVars.insert(v, other);
            }
        }
    }

    pub fn applyOwnershipVar(&self, var: OwnershipTypeVariable) -> OwnershipTypeVariable {
        let mut result = var;
        loop {
            match self.ownershipVars.get(&result) {
                Some(o) => {
                    result = *o;
                }
                None => {
                    return result;
                }
            }
        }
    }

    pub fn applyGroupVar(&self, var: GroupTypeVariable) -> GroupTypeVariable {
        let mut result = var;
        loop {
            match self.groupVars.get(&result) {
                Some(g) => {
                    result = *g;
                }
                None => {
                    return result;
                }
            }
        }
    }

    pub fn applyTypeVariableInfo(&self, info: TypeVariableInfo) -> TypeVariableInfo {
        let mut res = TypeVariableInfo::new();
        res.owner = self.applyOwnershipVar(info.owner);
        res.group = self.applyGroupVar(info.group);
        return res;
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct OwnershipTypeVariable {
    pub value: u32,
}

impl Display for OwnershipTypeVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%{}", self.value)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct GroupTypeVariable {
    pub value: u32,
}

impl Display for GroupTypeVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.value)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct TypeVariableInfo {
    pub owner: OwnershipTypeVariable,
    pub group: GroupTypeVariable,
}

impl TypeVariableInfo {
    pub fn new() -> TypeVariableInfo {
        TypeVariableInfo {
            owner: OwnershipTypeVariable { value: 0 },
            group: GroupTypeVariable { value: 0 },
        }
    }
}

impl Display for TypeVariableInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.owner, self.group)
    }
}
