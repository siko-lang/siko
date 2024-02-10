

class OwnershipVar(object):
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
        self.ownership_vars = {}
        self.group_vars = {}

    def inferFn(self, fn):
        self.fn = fn
        print("Inference for %s" % fn.name)
        self.initialize()
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
        for block in self.fn.body.blocks:
            for i in block.instructions:
                ownershipVar = self.nextOwnershipVar()
                groupVar = self.nextGroupVar()
                self.ownership_vars[i.id] = ownershipVar
                self.group_vars[i.id] = groupVar

    def dump(self):
        for block in self.fn.body.blocks:
            print("#%s block" % block.id)
            for i in block.instructions:
                ownershipVar = self.ownership_vars[i.id]
                groupVar = self.group_vars[i.id]
                print("%s %s - %s %s" % (i.id, i, ownershipVar, groupVar))

def infer(program):
    for f in program.functions.values():
        engine = InferenceEngine()
        engine.inferFn(f)
