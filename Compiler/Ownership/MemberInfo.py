import Compiler.DependencyProcessor as DependencyProcessor

class MemberKind(object):
    def __init__(self):
        self.type = None
        self.index = 0

    
class MemberInfo(object):
    def __init__(self):
        self.root = None
        self.kind = None
        self.info = None

    def __str__(self):
        return "%s/%s.%s/%s" % (self.kind.type, self.root, self.kind.index, self.info)

    def __repr__(self) -> str:
        return self.__str__()

def getGroupDependencyMap(members):
    dep_map = {}
    for member_info in members:
        dep_map[member_info.root] = []
        dep_map[member_info.info.group_var] = []
    for member_info in members:
        dep_map[member_info.root].append(member_info.info.group_var)
    return dep_map

def calculateChildOwnershipVars(members):
    child_ownership_vars = {}
    for member in members:
         child_ownership_vars[member.root] = []
    for member in members:
         child_ownership_vars[member.root].append(member.info.ownership_var)
    return child_ownership_vars

def collectDepOwnershipVarsForGroupVar(group, deps_map, ownership_dep_map, ownership_vars, item):
    deps = deps_map[item]
    for dep in deps:
        if dep in ownership_dep_map:
            ownership_vars = ownership_vars + ownership_dep_map[dep]
    return ownership_vars

def calculateDepsForGroup(child_ownership_vars, ownership_dep_map, deps_map, group):
    ownership_vars = []
    for item in group:
        if item in child_ownership_vars:
            ownership_vars += child_ownership_vars[item]
            ownership_vars = collectDepOwnershipVarsForGroupVar(group, deps_map, ownership_dep_map, ownership_vars, item)
    ownership_vars = list(set(ownership_vars))
    for item in group:
        ownership_dep_map[item] = ownership_vars
    return ownership_dep_map

def calculateOwnershipDepMap(members):
    deps_map = getGroupDependencyMap(members)
    groups = DependencyProcessor.processDependencies(deps_map)
    child_ownership_vars = calculateChildOwnershipVars(members)
    ownership_dep_map = {}
    for group in groups:
        ownership_dep_map = calculateDepsForGroup(child_ownership_vars, ownership_dep_map, deps_map, group.items)
    return ownership_dep_map