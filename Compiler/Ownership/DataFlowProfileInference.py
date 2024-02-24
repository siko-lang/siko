import Compiler.IR as IR
import Compiler.Util as Util
import Compiler.DependencyProcessor as DependencyProcessor
import Compiler.Ownership.DataFlowPath as DataFlowPath
import Compiler.Ownership.Equality as Equality
import Compiler.Ownership.Inference as Inference
import Compiler.Ownership.Equality as Equality
import Compiler.Ownership.ForbiddenBorrows as ForbiddenBorrows
import Compiler.Ownership.MemberInfo as MemberInfo
import copy

def createFunctionGroups(program):
    dep_map = {}
    for (key, function) in program.functions.items():
        deps = set()
        for block in function.body.blocks:
            for i in block.instructions:
                if isinstance(i, IR.NamedFunctionCall):
                    if i.name == Util.getUnit():
                        continue
                    if i.ctor:
                        continue
                    deps.add(i.name)
        dep_map[key] = list(deps)
    #print("depmap", dep_map)
    groups = DependencyProcessor.processDependencies(dep_map)
    print("Function groups", groups)
    return groups

class InferenceEngine(object):
    def __init__(self) -> None:
        self.dataflowpaths = {}
        self.program = None

    def processGroup(self, group):
        print("Processing group", group)
        if len(group.items) == 1:
            name = group.items[0]
            print("Processing fn", name)
            fn = self.program.functions[name]
            fn = copy.deepcopy(fn)
            equality = Equality.EqualityEngine()
            equality.process(fn)
            members = fn.getAllMembers()
            #print("members", members)
            ownership_dep_map = MemberInfo.calculateOwnershipDepMap(members)
            forbidden_borrows = ForbiddenBorrows.ForbiddenBorrowsEngine()
            forbidden_borrows.process(fn, ownership_dep_map)
            inference = Inference.InferenceEngine()
            ownerships = inference.infer(fn, self.program.classes)
            paths = DataFlowPath.infer(fn)
            print("%s has paths %s" % (name, paths))
        else:
            Util.error("Multi function groups NYI in data flow profile inference")

    def processGroups(self, groups):
        for group in groups:
            self.processGroup(group)

def infer(program):
    groups = createFunctionGroups(program)
    engine = InferenceEngine()
    engine.program = program
    engine.processGroups(groups)

    