import IR
import DataFlowDependency
import DependencyProcessor

class InferenceEngine(object):
    def __init__(self):
        self.fn = None

    def inferFn(self, fn):
        print("Forbidden borrows ", fn.name)
        self.fn = fn
        all_dependencies = DataFlowDependency.getDataFlowDependencies(fn)
        groups = DependencyProcessor.processDependencies(all_dependencies)
        all_witnessed_moves = {}
        for group in groups:
            #print("Processing group", group.items)
            for item in group.items:
                instruction = self.fn.body.getInstruction(item)
                witnessed_moves = set()
                if isinstance(instruction, IR.VarRef):
                    if not instruction.borrow:
                        witnessed_moves.add(instruction.name)
                if isinstance(instruction, IR.ValueRef):
                    if not instruction.borrow:
                        witnessed_moves.add(instruction.name)
                deps = all_dependencies[item]
                for dep in deps:
                    for w in all_witnessed_moves[dep]:
                        witnessed_moves.add(w)
                all_witnessed_moves[item] = witnessed_moves
        for block in fn.body.blocks:
            print("%s. block:" % block.id)
            for i in block.instructions:
                print("   %4s %35s %10s %s %s" % (i.id, i, i.tv_info, i.member_infos, all_witnessed_moves[i.id]))

def infer(program):
    for f in program.functions.values():
        engine = InferenceEngine()
        engine.inferFn(f)