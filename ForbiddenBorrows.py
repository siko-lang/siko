import IR
import DataFlowDependency
import DependencyProcessor
import MemberInfo

class InferenceEngine(object):
    def __init__(self):
        self.fn = None

    def inferFn(self, fn):
        #print("Forbidden borrows ", fn.name)
        self.fn = fn
        members = self.fn.body.getAllMembers()
        ownership_dep_map = MemberInfo.calculateOwnershipDepMap(members)
        #print("ownership_dep_map", ownership_dep_map)
        all_dependencies = DataFlowDependency.getDataFlowDependencies(fn)
        groups = DependencyProcessor.processDependencies(all_dependencies)
        all_witnessed_moves = {}
        forbidden_borrows = {}
        for group in groups:
            #print("Processing group", group.items)
            for item in group.items:
                instruction = self.fn.body.getInstruction(item)
                if instruction.tv_info.group_var in ownership_dep_map:
                    ownership_vars = list(ownership_dep_map[instruction.tv_info.group_var])
                else:
                    ownership_vars = []
                ownership_vars.append(instruction.tv_info.ownership_var)
                witnessed_moves = set()
                for move in instruction.moves:
                    witnessed_moves.add(move)
                deps = all_dependencies[item]
                for dep in deps:
                    for w in all_witnessed_moves[dep]:
                        witnessed_moves.add(w)
                all_witnessed_moves[item] = witnessed_moves
                #print("%s %s" % (item, ownership_vars))
                for ownership_var in ownership_vars:
                    if ownership_var not in forbidden_borrows:
                        forbidden_borrows[ownership_var] = set()
                    for witnessed_move in witnessed_moves:
                        forbidden_borrows[ownership_var].add(witnessed_move)
        print("forbidden_borrows", forbidden_borrows)
        fn.forbidden_borrows = forbidden_borrows
        for block in fn.body.blocks:
            print("%s. block:" % block.id)
            for i in block.instructions:
                print("   %4s %35s %10s %s %s" % (i.id, i, i.tv_info, i.members, all_witnessed_moves[i.id]))

def infer(program):
    for f in program.functions.values():
        engine = InferenceEngine()
        engine.inferFn(f)