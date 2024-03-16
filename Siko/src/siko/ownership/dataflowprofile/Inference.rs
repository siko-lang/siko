use std::collections::{BTreeMap, BTreeSet};

use crate::siko::{
    ir::Function::{Function, InstructionKind},
    qualifiedname::QualifiedName,
    util::DependencyProcessor::{self, DependencyGroup},
};

use super::{DataFlowProfile::DataFlowProfile, DataFlowProfileStore::DataFlowProfileStore};

fn createFunctionGroups(
    functions: &BTreeMap<QualifiedName, Function>,
) -> (Vec<DependencyGroup<QualifiedName>>, BTreeSet<QualifiedName>) {
    let mut depMap = BTreeMap::new();
    let mut recursiveFns = BTreeSet::new();
    for (key, function) in functions {
        let mut deps = BTreeSet::new();
        if let Some(body) = &function.body {
            for block in &body.blocks {
                for i in &block.instructions {
                    match &i.kind {
                        InstructionKind::FunctionCall(name, _) => {
                            if name == key {
                                recursiveFns.insert(key.clone());
                            }
                            deps.insert(name.clone());
                        }
                        _ => {}
                    }
                }
            }
        }
        depMap.insert(key.clone(), deps.into_iter().collect());
    }
    let groups = DependencyProcessor::processDependencies(&depMap);
    (groups, recursiveFns)
}

struct InferenceEngine<'a> {
    profileStore: DataFlowProfileStore,
    functions: &'a BTreeMap<QualifiedName, Function>,
}

impl<'a> InferenceEngine<'a> {
    fn new(functions: &'a BTreeMap<QualifiedName, Function>) -> InferenceEngine<'a> {
        InferenceEngine {
            profileStore: DataFlowProfileStore::new(),
            functions: functions,
        }
    }

    fn createDataFlowProfile(
        &mut self,
        name: &QualifiedName,
        groupProfiles: &BTreeMap<QualifiedName, DataFlowProfile>,
    ) -> DataFlowProfile {
        let f = self.functions.get(name).clone();
        DataFlowProfile::new()
        // equality = Equality.EqualityEngine(fn, self.profile_store, group_profiles)
        // profiles = equality.process(buildPath=True)
        // paths = equality.paths
        // all_paths = []
        // for profile in profiles.values():
        //     all_paths += profile.paths
        // members = fn.getAllMembers(all_paths)
        // #print("members", members)
        // #print("paths", paths)
        // ownership_dep_map = MemberInfo.calculateOwnershipDepMap(members)
        // forbidden_borrows = ForbiddenBorrows.ForbiddenBorrowsEngine()
        // forbidden_borrows.process(fn, ownership_dep_map)
        // inference = Inference.InferenceEngine(fn, profiles, self.program.classes)
        // ownerships = inference.infer()
        // ownership_provider = Normalizer.OwnershipProvider()
        // ownership_provider.ownership_map = ownerships
        // signature = fn.ownership_signature
        // (signature, paths) = Normalizer.normalizeFunctionProfile(signature, paths,
        //                                                          ownership_dep_map,
        //                                                          members,
        //                                                          ownership_provider,
        //                                                          onlyBorrow=False)

        // profile = DataFlowProfile.DataFlowProfile()
        // profile.paths = paths
        // profile.signature = signature
        // return profile
    }

    fn processGroup(
        &mut self,
        group: DependencyGroup<QualifiedName>,
        recursive_fns: &BTreeSet<QualifiedName>,
    ) {
        //println!("Processing group {:?}", group);
        if group.items.len() == 1 && !recursive_fns.contains(&group.items[0]) {
            let name = &group.items[0];
            //println!("Processing fn {}", name);
            let profile = self.createDataFlowProfile(name, &BTreeMap::new());
            self.profileStore.addProfile(name.clone(), profile);
        } else {
            let mut groupProfiles = BTreeMap::new();
            let mut change = true;
            while change {
                change = false;
                for name in &group.items {
                    let profile = self.createDataFlowProfile(&name, &groupProfiles);
                    if let Some(prev) = groupProfiles.get(&name) {
                        if *prev != profile {
                            change = true;
                        }
                    } else {
                        change = true;
                    }
                    groupProfiles.insert(name.clone(), profile);
                }
            }
            for (name, profile) in groupProfiles {
                self.profileStore.addProfile(name, profile);
            }
        }
    }

    fn processGroups(
        &mut self,
        groups: Vec<DependencyGroup<QualifiedName>>,
        recursive_fns: &BTreeSet<QualifiedName>,
    ) {
        for group in groups {
            self.processGroup(group, recursive_fns);
        }
    }

    fn profileStore(self) -> DataFlowProfileStore {
        self.profileStore
    }
}

pub fn dataflow(functions: &BTreeMap<QualifiedName, Function>) -> DataFlowProfileStore {
    let (groups, recursive_fns) = createFunctionGroups(functions);
    //println!("Groups {:?}, recursive {:?}", groups, recursive_fns);
    let mut engine = InferenceEngine::new(functions);
    engine.processGroups(groups, &recursive_fns);
    engine.profileStore()
}
