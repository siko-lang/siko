import IR
import copy

class Usage(object):
    def __init__(self):
        self.id = None
        self.path = None

    def __str__(self):
        if self.path:
            path = ".".join(self.path)
            return "%s.%s" % (self.id, path)
        else:
            return str(self.id)

class UsageSet(object):
    def __init__(self):
        self.usages = []

    def len(self):
        return len(self.usages)

    def __str__(self):
        usages = map(lambda x: str(x), self.usages)
        return ", ".join(usages)

    def addUsage(self, id):
        self.usages.append(id)

class UsageHolder(object):
    def __init__(self):
        self.usagesets = {}

    def addDef(self, var):
        self.usagesets[var] = UsageSet()

    def addUsage(self, id, var):
        usage = Usage()
        usage.id = id
        self.usagesets[var].addUsage(usage)

    def addMemberUsage(self, id, var, path):
        usage = Usage()
        usage.id = id
        usage.path = path
        self.usagesets[var].addUsage(usage)

    def __str__(self):
        values = []
        for (key, value) in self.usagesets.items():
            if value.len() > 0:
                values.append("%s: %s" % (key, value))
            else:
                values.append("%s" % key)
        return "{%s}" % ", ".join(values)

class Borrowchecker(object):
    def __init__(self):
        pass

    def checkBlock(self, usages, block):
        for (index, i) in enumerate(block.instructions):
            if isinstance(i, IR.Bind):
                usages.addDef(i.name)
            if isinstance(i, IR.VarRef):
                path = []
                while True:
                    next = block.instructions[index + 1]
                    if isinstance(next, IR.MemberAccess):
                        path.append(next.name)
                        index += 1
                    else:
                        break
                if len(path) == 0:
                    usages.addUsage(i.id, i.name)
                else:
                    usages.addMemberUsage(i.id, i.name, path)
            print("%5s %25s %10s" % (i.id, i, usages))

    def checkFn(self, fn):
        print("Borrow check ", fn.name)
        usages = UsageHolder()
        for arg in fn.args:
            usages.addDef(arg.name)
        self.checkBlock(usages, fn.body.getFirst())

def checkFn(fn):
    borrowchecker = Borrowchecker()
    borrowchecker.checkFn(fn)

def processProgram(program):
    for (name, fn) in program.functions.items():
        checkFn(fn)
    for (_, clazz) in program.classes.items():
        for m in clazz.methods:
            checkFn(m)