import Compiler.Util as Util
import Compiler.IR as IR

def getDepsForInstruction(i, fn):
    if isinstance(i, IR.ValueRef):
        if i.name.arg:
            return []
        else:
            return [i.bind_id]
    elif isinstance(i, IR.Bind):
        return [i.rhs]
    elif isinstance(i, IR.BlockRef):
        b = fn.body.getBlock(i.value)
        return [b.getLastReal().id]
    elif isinstance(i, IR.NamedFunctionCall):
        return i.args
    elif isinstance(i, IR.DropVar):
        return []
    elif isinstance(i, IR.BoolLiteral):
        return []
    elif isinstance(i, IR.Nop):
        return []
    elif isinstance(i, IR.If):
        true_branch = fn.body.getBlock(i.true_branch)
        false_branch = fn.body.getBlock(i.false_branch)
        t_id = true_branch.getLast().id
        f_id = false_branch.getLast().id
        return [t_id, f_id]
    else:
        Util.error("OI: getDepsForInstruction not handling %s %s" % (type(i), i))

def getDataFlowDependencies(fn):
    all_dependencies = {}
    for block in fn.body.blocks:
        for i in block.instructions:
            all_dependencies[i.id] = getDepsForInstruction(i, fn)
    return all_dependencies