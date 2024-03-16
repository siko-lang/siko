use std::collections::{BTreeMap, BTreeSet};

use crate::siko::{
    ir::Function::{Function, InstructionKind},
    qualifiedname::QualifiedName,
    util::DependencyProcessor::{self, DependencyGroup},
};

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

// class InferenceEngine(object):
//     def __init__(self) -> None:
//         self.profile_store = DataFlowProfileStore.DataFlowProfileStore()
//         self.program = None

//     def createDataFlowProfile(self, name, group_profiles):
//         fn = self.program.functions[name]
//         fn = copy.deepcopy(fn)
//         equality = Equality.EqualityEngine(fn, self.profile_store, group_profiles)
//         profiles = equality.process(buildPath=True)
//         paths = equality.paths
//         all_paths = []
//         for profile in profiles.values():
//             all_paths += profile.paths
//         members = fn.getAllMembers(all_paths)
//         #print("members", members)
//         #print("paths", paths)
//         ownership_dep_map = MemberInfo.calculateOwnershipDepMap(members)
//         forbidden_borrows = ForbiddenBorrows.ForbiddenBorrowsEngine()
//         forbidden_borrows.process(fn, ownership_dep_map)
//         inference = Inference.InferenceEngine(fn, profiles, self.program.classes)
//         ownerships = inference.infer()
//         ownership_provider = Normalizer.OwnershipProvider()
//         ownership_provider.ownership_map = ownerships
//         signature = fn.ownership_signature
//         (signature, paths) = Normalizer.normalizeFunctionProfile(signature, paths,
//                                                                  ownership_dep_map,
//                                                                  members,
//                                                                  ownership_provider,
//                                                                  onlyBorrow=False)

//         profile = DataFlowProfile.DataFlowProfile()
//         profile.paths = paths
//         profile.signature = signature
//         return profile

//     def processGroup(self, group, recursive_fns):
//         #print("Processing group", group)
//         if len(group.items) == 1 and group.items[0] not in recursive_fns:
//             #print("Processing fn", name)
//             name = group.items[0]
//             profile = self.createDataFlowProfile(name, {})
//             self.profile_store.addProfile(name, profile)
//         else:
//             group_profiles = {}
//             change = True
//             while change:
//                 change = False
//                 for name in group.items:
//                     profile = self.createDataFlowProfile(name, group_profiles)
//                     if name in group_profiles:
//                         prev = group_profiles[name]
//                         if prev != profile:
//                             change = True
//                     else:
//                         change = True
//                     group_profiles[name] = profile
//             for name in group.items:
//                 self.profile_store.addProfile(name, group_profiles[name])

//     def processGroups(self, groups, recursive_fns):
//         for group in groups:
//             self.processGroup(group, recursive_fns)

pub fn infer(functions: &BTreeMap<QualifiedName, Function>) {
    let (groups, recursive_fns) = createFunctionGroups(functions);
    println!("Groups {:?}, recursive {:?}", groups, recursive_fns);
    // engine = InferenceEngine()
    // engine.program = program
    // engine.processGroups(groups, recursive_fns)
    // return engine.profile_store
}
