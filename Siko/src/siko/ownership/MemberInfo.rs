use super::TypeVariableInfo::{GroupTypeVariable, OwnershipTypeVariable, TypeVariableInfo};
use crate::siko::util::DependencyProcessor;
use std::{collections::BTreeMap, fmt::Display};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemberKind {
    Variant,
    Field,
    Extern,
}

impl Display for MemberKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            MemberKind::Variant => write!(f, "v"),
            MemberKind::Field => write!(f, "f"),
            MemberKind::Extern => write!(f, "e"),
        }
    }
}

pub struct MemberInfo {
    pub root: GroupTypeVariable,
    pub kind: MemberKind,
    pub index: u32,
    pub info: TypeVariableInfo,
}

impl Display for MemberInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}/{}/{}",
            self.kind, self.index, self.root, self.info
        )
    }
}

fn getGroupDependencyMap(
    members: &Vec<MemberInfo>,
) -> BTreeMap<GroupTypeVariable, Vec<GroupTypeVariable>> {
    let mut depMap = BTreeMap::new();
    for member in members {
        depMap.insert(member.root, Vec::new());
        depMap.insert(member.info.group, Vec::new());
    }
    for member in members {
        depMap
            .get_mut(&member.root)
            .unwrap()
            .push(member.info.group);
    }
    depMap
}

fn calculateChildOwnershipVars(
    members: &Vec<MemberInfo>,
) -> BTreeMap<GroupTypeVariable, Vec<OwnershipTypeVariable>> {
    let mut childOwnershipMap = BTreeMap::new();
    for member in members {
        childOwnershipMap.insert(member.root, Vec::new());
    }
    for member in members {
        childOwnershipMap
            .get_mut(&member.root)
            .unwrap()
            .push(member.info.owner);
    }
    childOwnershipMap
}

fn collectDepOwnershipVarsForGroupVar(
    depMap: &BTreeMap<GroupTypeVariable, Vec<GroupTypeVariable>>,
    ownership_dep_map: &BTreeMap<GroupTypeVariable, Vec<OwnershipTypeVariable>>,
    mut ownershipVars: Vec<OwnershipTypeVariable>,
    item: GroupTypeVariable,
) -> Vec<OwnershipTypeVariable> {
    let deps = depMap.get(&item).unwrap();
    for dep in deps {
        if let Some(vars) = ownership_dep_map.get(dep) {
            ownershipVars.extend(vars);
        }
    }
    return ownershipVars;
}

fn calculateDepsForGroup(
    childOwnershipVars: &BTreeMap<GroupTypeVariable, Vec<OwnershipTypeVariable>>,
    mut ownershipDepMap: BTreeMap<GroupTypeVariable, Vec<OwnershipTypeVariable>>,
    depMap: &BTreeMap<GroupTypeVariable, Vec<GroupTypeVariable>>,
    group: &Vec<GroupTypeVariable>,
) -> BTreeMap<GroupTypeVariable, Vec<OwnershipTypeVariable>> {
    let mut ownershipVars = Vec::new();
    for item in group {
        if let Some(vars) = childOwnershipVars.get(&item) {
            ownershipVars.extend(vars);
            ownershipVars =
                collectDepOwnershipVarsForGroupVar(&depMap, &ownershipDepMap, ownershipVars, *item);
        }
    }
    ownershipVars.sort();
    ownershipVars.dedup();
    for item in group {
        ownershipDepMap.insert(*item, ownershipVars.clone());
    }
    ownershipDepMap
}

pub fn calculateOwnershipDepMap(
    members: &Vec<MemberInfo>,
) -> BTreeMap<GroupTypeVariable, Vec<OwnershipTypeVariable>> {
    let depMap = getGroupDependencyMap(members);
    let groups = DependencyProcessor::processDependencies(&depMap);
    let childOwnershipVars = calculateChildOwnershipVars(members);
    let mut ownershipDepMap = BTreeMap::new();
    for group in groups {
        ownershipDepMap =
            calculateDepsForGroup(&childOwnershipVars, ownershipDepMap, &depMap, &group.items);
    }
    return ownershipDepMap;
}
