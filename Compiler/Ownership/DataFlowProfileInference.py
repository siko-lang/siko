import Compiler.IR as IR
import Compiler.Util as Util
import Compiler.DependencyProcessor as DependencyProcessor
import Compiler.Ownership.Equality as Equality
import Compiler.Ownership.Inference as Inference
import Compiler.Ownership.Equality as Equality
import Compiler.Ownership.ForbiddenBorrows as ForbiddenBorrows
import Compiler.Ownership.MemberInfo as MemberInfo
import Compiler.Ownership.Normalizer as Normalizer
import Compiler.Ownership.DataFlowProfile as DataFlowProfile
import Compiler.Ownership.DataFlowProfileStore as DataFlowProfileStore
import copy

def createFunctionGroups(program):
    dep_map = {}
    recursive_fns = set()
    for (key, function) in program.functions.items():
        deps = set()
        for block in function.body.blocks:
            for i in block.instructions:
                if isinstance(i, IR.NamedFunctionCall):
                    if i.name == Util.getUnit():
                        continue
                    if i.ctor:
                        continue
                    if i.name == key:
                        recursive_fns.add(key)
                    deps.add(i.name)
        dep_map[key] = list(deps)
    #print("depmap", dep_map)
    groups = DependencyProcessor.processDependencies(dep_map)
    #print("Function groups", groups)
    return (groups, recursive_fns)


class InferenceEngine(object):
    def __init__(self) -> None:
        self.profile_store = DataFlowProfileStore.DataFlowProfileStore()
        self.program = None

    def createDataFlowProfile(self, name, group_profiles):
        fn = self.program.functions[name]
        fn = copy.deepcopy(fn)
        equality = Equality.EqualityEngine(fn, self.profile_store, group_profiles)
        profiles = equality.process(buildPath=True)
        paths = equality.paths
        all_paths = []
        for profile in profiles.values():
            all_paths += profile.paths
        members = fn.getAllMembers(all_paths)
        #print("members", members)
        #print("paths", paths)
        ownership_dep_map = MemberInfo.calculateOwnershipDepMap(members)
        forbidden_borrows = ForbiddenBorrows.ForbiddenBorrowsEngine()
        forbidden_borrows.process(fn, ownership_dep_map)
        inference = Inference.InferenceEngine(fn, profiles, self.program.classes)
        ownerships = inference.infer()
        ownership_provider = Normalizer.OwnershipProvider()
        ownership_provider.ownership_map = ownerships
        signature = fn.ownership_signature
        (signature, paths) = Normalizer.normalizeFunctionProfile(signature, paths,
                                                                 ownership_dep_map,
                                                                 members,
                                                                 ownership_provider,
                                                                 onlyBorrow=False)
        
        profile = DataFlowProfile.DataFlowProfile()
        profile.paths = paths
        profile.signature = signature
        return profile

    def processGroup(self, group, recursive_fns):
        #print("Processing group", group)
        if len(group.items) == 1 and group.items[0] not in recursive_fns:
            #print("Processing fn", name)
            name = group.items[0]
            profile = self.createDataFlowProfile(name, {})
            self.profile_store.addProfile(name, profile)
        else:
            group_profiles = {}
            change = True
            while change:
                change = False
                for name in group.items:
                    profile = self.createDataFlowProfile(name, group_profiles)
                    if name in group_profiles:
                        prev = group_profiles[name]
                        if prev != profile:
                            change = True
                    else:
                        change = True
                    group_profiles[name] = profile    
            for name in group.items:
                self.profile_store.addProfile(name, group_profiles[name])

    def processGroups(self, groups, recursive_fns):
        for group in groups:
            self.processGroup(group, recursive_fns)

def infer(program):
    (groups, recursive_fns) = createFunctionGroups(program)
    engine = InferenceEngine()
    engine.program = program
    engine.processGroups(groups, recursive_fns)
    return engine.profile_store

    