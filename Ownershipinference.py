import IR
import Util
import DependencyProcessor

def getDepsForInstruction(i, fn):
    if isinstance(i, IR.ValueRef):
        return [i.bind_id]
    elif isinstance(i, IR.VarRef):
        if i.name.arg:
            return []
        else:
            return [i.bind_id]
    elif isinstance(i, IR.Bind):
        return [i.rhs]
    elif isinstance(i, IR.BlockRef):
        b = fn.body.getBlock(i.value)
        return [b.getLastNonDrop().id]
    elif isinstance(i, IR.NamedFunctionCall):
        return i.args
    elif isinstance(i, IR.DropVar):
        return []
    elif isinstance(i, IR.Converter):
        return [i.arg]
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

class Substitution(object):
    def __init__(self):
        self.ownership_vars = {}
        self.group_vars = {}

    def addOwnershipVar(self, ownership_var, other):
        if ownership_var != other:
            self.ownership_vars[ownership_var] = other

    def addGroupVar(self, group_var, other):
        if group_var != other:
            self.group_vars[group_var] = other

    def applyOwnershipVar(self, var):
        res = var
        while True:
            if res in self.ownership_vars:
                res = self.ownership_vars[res]
            else:
                return res

    def applyGroupVar(self, var):
        res = var
        while True:
            if res in self.group_vars:
                res = self.group_vars[res]
            else:
                return res

class OwnershipVar(object):
    def __init__(self):
        self.value = 0

    def __str__(self):
        return "%%%s" % self.value

    def __eq__(self, other):
        return self.value == other.value

    def __ne__(self, other):
        return not self.__eq__(other)

    def __hash__(self):
        return self.value.__hash__()

class GroupVar(object):
    def __init__(self):
        self.value = 0

    def __str__(self):
        return "#%s" % self.value

    def __eq__(self, other):
        return self.value == other.value

    def __ne__(self, other):
        return not self.__eq__(other)

    def __hash__(self):
        return self.value.__hash__()

class InferenceEngine(object):
    def __init__(self):
        self.fn = None
        self.next = 0
        self.i_ownership_vars = {}
        self.i_group_vars = {}
        self.v_ownership_vars = {}
        self.v_group_vars = {}
        self.substitution = Substitution()

    def inferFn(self, fn):
        self.fn = fn
        print("Inference for %s" % fn.name)
        self.initialize()
        self.mergeGroups()
        self.createPaths()
        self.dump()

    def nextOwnershipVar(self):
        n = self.next
        self.next += 1
        v = OwnershipVar()
        v.value = n
        return v

    def nextGroupVar(self):
        n = self.next
        self.next += 1
        v = GroupVar()
        v.value = n
        return v

    def initialize(self):
        for arg in self.fn.args:
            ownershipVar = self.nextOwnershipVar()
            groupVar = self.nextGroupVar()
            self.v_ownership_vars[arg.name] = ownershipVar
            self.v_group_vars[arg.name] = groupVar    
        for block in self.fn.body.blocks:
            for i in block.instructions:
                if isinstance(i, IR.Bind):
                    ownershipVar = self.nextOwnershipVar()
                    groupVar = self.nextGroupVar()
                    self.v_ownership_vars[i.name] = ownershipVar
                    self.v_group_vars[i.name] = groupVar    
                ownershipVar = self.nextOwnershipVar()
                groupVar = self.nextGroupVar()
                self.i_ownership_vars[i.id] = ownershipVar
                self.i_group_vars[i.id] = groupVar

    def unifyOwnership(self, o1, o2):
        o1 = self.substitution.applyOwnershipVar(o1)
        o2 = self.substitution.applyOwnershipVar(o2)
        self.substitution.addOwnershipVar(o1, o2)

    def unifyGroup(self, g1, g2):
        g1 = self.substitution.applyGroupVar(g1)
        g2 = self.substitution.applyGroupVar(g2)
        self.substitution.addGroupVar(g1, g2)

    def processBlock(self, block):
        for i in block.instructions:
            if isinstance(i, IR.Bind):
                rhs_o = self.i_ownership_vars[i.rhs]
                rhs_g = self.i_group_vars[i.rhs]
                v_o = self.v_ownership_vars[i.name]
                v_g = self.v_group_vars[i.name]
                self.unifyOwnership(rhs_o, v_o)
                self.unifyGroup(rhs_g, v_g)
            elif isinstance(i, IR.NamedFunctionCall):
                pass
            elif isinstance(i, IR.VarRef):
                i_o = self.i_ownership_vars[i.id]
                i_g = self.i_group_vars[i.id]
                v_o = self.v_ownership_vars[i.name]
                v_g = self.v_group_vars[i.name]
                self.unifyOwnership(i_o, v_o)
                self.unifyGroup(i_g, v_g)
            elif isinstance(i, IR.ValueRef):
                pass
            elif isinstance(i, IR.BoolLiteral):
                pass
            elif isinstance(i, IR.If):
                true_branch = self.fn.body.getBlock(i.true_branch)
                false_branch = self.fn.body.getBlock(i.false_branch)
                self.processBlock(true_branch)
                self.processBlock(false_branch)
                t_id = true_branch.getLastNonDrop().id
                f_id = false_branch.getLastNonDrop().id
                t_o = self.i_ownership_vars[t_id]
                t_g = self.i_group_vars[t_id]
                f_o = self.i_ownership_vars[f_id]
                f_g = self.i_group_vars[f_id]
                self.unifyOwnership(t_o, f_o)
                self.unifyGroup(t_g, f_g)
            elif isinstance(i, IR.DropVar):
                pass
            elif isinstance(i, IR.BlockRef):
                b = self.fn.body.getBlock(i.value)
                self.processBlock(b)
                l_id = b.getLastNonDrop().id
                i_o = self.i_ownership_vars[i.id]
                i_g = self.i_group_vars[i.id]
                l_o = self.i_ownership_vars[l_id]
                l_g = self.i_group_vars[l_id]
                self.unifyOwnership(i_o, l_o)
                self.unifyGroup(i_g, l_g)
            elif isinstance(i, IR.Converter):
                i_g = self.i_group_vars[i.id]
                arg_g = self.i_group_vars[i.arg]
                self.unifyGroup(i_g, arg_g)
            elif isinstance(i, IR.Nop):
                pass
            else:
                Util.error("OI: grouping not handling %s %s" % (type(i), i))

    def mergeGroups(self):
        block = self.fn.body.getFirst()
        self.processBlock(block)

    def createPaths(self):
        all_dependencies = {}
        paths = {}
        for block in self.fn.body.blocks:
            for i in block.instructions:
                all_dependencies[i.id] = getDepsForInstruction(i, self.fn)
        groups = DependencyProcessor.processDependencies(all_dependencies)
        for g in groups:
            for item in g.items:
                item_paths = []
                deps = all_dependencies[item]
                if len(deps) == 0:
                    item_paths = [[item]]
                else:
                    for dep in deps:
                        if dep in g.items:
                            continue
                        dep_paths = paths[dep]
                        for dep_path in dep_paths:
                            item_paths.append(dep_path + [item])
                paths[item] = item_paths
        for (i, paths) in paths.items():
            print("root %s" % i)
            for path in paths:
                print("path", path)
    
    def dump(self):
        for block in self.fn.body.blocks:
            print("#%s block" % block.id)
            for i in block.instructions:
                ownershipVar = self.i_ownership_vars[i.id]
                ownershipVar = self.substitution.applyOwnershipVar(ownershipVar)
                groupVar = self.i_group_vars[i.id]
                groupVar = self.substitution.applyGroupVar(groupVar)
                print("%5s %35s - %4s %4s" % (i.id, i, ownershipVar, groupVar))

def infer(program):
    for f in program.functions.values():
        engine = InferenceEngine()
        engine.inferFn(f)