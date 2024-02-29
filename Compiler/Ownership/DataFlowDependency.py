import Compiler.Util as Util
import Compiler.IR.Instruction as Instruction

def getDepsForInstruction(i, fn):
    if isinstance(i, Instruction.ValueRef):
        if i.name.arg:
            return []
        else:
            return [i.bind_id]
    elif isinstance(i, Instruction.Bind):
        return [i.rhs]
    elif isinstance(i, Instruction.BlockRef):
        b = fn.body.getBlock(i.value)
        return [b.getLastReal().id]
    elif isinstance(i, Instruction.NamedFunctionCall):
        return i.args
    elif isinstance(i, Instruction.DropVar):
        return []
    elif isinstance(i, Instruction.BoolLiteral):
        return []
    elif isinstance(i, Instruction.Nop):
        return []
    elif isinstance(i, Instruction.Tuple):
        return i.args
    elif isinstance(i, Instruction.If):
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