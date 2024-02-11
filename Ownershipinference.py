import IR
import Util

class Substitution(object):
    def __init__(self):
        self.ownership_vars = {}
        self.group_vars = {}

    def addOwnershipVar(self, ownership_var, other):
        self.ownership_vars[ownership_var] = other

    def addGroupVar(self, group_var, other):
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
            elif isinstance(i, IR.DropVar):
                pass
            elif isinstance(i, IR.Converter):
                i_g = self.i_group_vars[i.id]
                arg_g = self.i_group_vars[i.arg]
                self.unifyGroup(i_g, arg_g)
            else:
                Util.error("Ownership inference not handling %s %s" % (type(i), i))

    def mergeGroups(self):
        block = self.fn.body.getFirst()
        self.processBlock(block)

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
