import IR
import copy
import CFG
import CFGBuilder

class Usage(object):
    def __init__(self):
        self.id = None
        self.path = None
        self.block_path = None

    def __str__(self):
        if self.path:
            path = ".".join(self.path)
            return "%s.%s/%s" % (self.id, path, self.block_path)
        else:
            return "%s/%s" % (self.id, self.block_path)

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

    def addUsage(self, id, var, block_path):
        usage = Usage()
        usage.id = id
        usage.block_path = block_path
        self.usagesets[var].addUsage(usage)

    def addMemberUsage(self, id, var, path, block_path):
        usage = Usage()
        usage.id = id
        usage.path = path
        usage.block_path = block_path
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
        self.fn = None
        self.usages = None

    # def checkBlock(self, block, block_path):
    #     print("#%d. block:" % block.id)
    #     block_path = block_path + [block.id]
    #     for (index, i) in enumerate(block.instructions):
    #         if isinstance(i, IR.Bind):
    #             self.usages.addDef(i.name)
    #         if isinstance(i, IR.BlockRef):
    #             self.checkBlock(self.fn.body.getBlock(i), block_path)
    #         if isinstance(i, IR.VarRef):
    #             path = []
    #             while index + 1 < len(block.instructions):
    #                 next = block.instructions[index + 1]
    #                 if isinstance(next, IR.MemberAccess):
    #                     path.append(next.name)
    #                     index += 1
    #                 else:
    #                     break
    #             if len(path) == 0:
    #                 self.usages.addUsage(i.id, i.name, block_path)
    #             else:
    #                 self.usages.addMemberUsage(i.id, i.name, path, block_path)
    #         print("%5s %25s" % (i.id, i))

    # def checkFn(self, fn):
    #     print("Borrow check ", fn.name)
    #     self.fn = fn
    #     self.usages = UsageHolder()
    #     for arg in fn.args:
    #         self.usages.addDef(arg.name)
    #     self.checkBlock(fn.body.getFirst(), [])
    #     #print("Usages %s" % self.usages)

def checkFn(fn):
    borrowchecker = Borrowchecker()
    cfgbuilder = CFGBuilder.CFGBuilder()
    cfg = cfgbuilder.build(fn)
    cfg.printDot()

def processProgram(program):
    for (name, fn) in program.functions.items():
        checkFn(fn)
    for (_, clazz) in program.classes.items():
        for m in clazz.methods:
            checkFn(m)