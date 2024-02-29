import Compiler.IR.Instruction as Instruction
import Compiler.Ownership.CFG as CFG
import Compiler.Ownership.CFGBuilder as CFGBuilder
import Compiler.Ownership.Path as Path

class Usage(object):
    def __init__(self):
        self.id = None
        self.path = None

    def __eq__(self, other):
        return self.id == other.id and self.path == other.path

    def __hash__(self):
        return self.id.__hash__()

    def __str__(self):
        return "%s:%s" % (self.id, self.path)

class UsageSet(object):
    def __init__(self):
        self.usages = set()

    def add(self, v):
        self.usages.add(v)

    def __iadd__(self, other):
        for o in other.usages:
            self.usages.add(o)
        return self

    def len(self):
        return len(self.usages)

    def __eq__(self, other):
        return self.usages == other.usages

    def __hash__(self):
        return self.usages.__hash__()

    def __str__(self):
        ss = []
        for v in self.usages:
            ss.append(str(v))
        return ", ".join(ss)

class Borrowchecker(object):
    def __init__(self, cfg, fn):
        self.fn = fn
        self.cfg = cfg
        self.usages = {}
        self.borrows = set()
        self.cancelled_drops = set()

    def check(self):
        sources = self.cfg.getSources()
        for source in sources:
            #print("Checking source %s" % source)
            self.processNode(source)

    def invalidates(self, current, other):
        #print("Invalidate %s === %s" % (current, other))
        if current.var != other.var:
            return False
        else:
            if isinstance(current, Path.WholePath) and isinstance(other, Path.WholePath):
                return True
            if isinstance(current, Path.WholePath) and isinstance(other, Path.PartialPath):
                return True
            if isinstance(current, Path.PartialPath) and isinstance(other, Path.WholePath):
                return True
            if isinstance(current, Path.PartialPath) and isinstance(other, Path.PartialPath):
                c_len = len(current.fields)
                o_len = len(other.fields)
                min_len = min(c_len, o_len)
                c = current.fields[:min_len]
                o = other.fields[:min_len]
                return c == o

    def invalidate(self, usage, usages):
        #print("Invalidate %s %s" % (usage, usages))
        for prev_usage in usages.usages:
            if self.invalidates(usage.path, prev_usage.path):
                if isinstance(usage.path, Path.WholePath):
                    if usage.path.is_drop and prev_usage.id not in self.borrows:
                        # the current usage is a drop and the prev is a move
                        self.cancelled_drops.add(usage.id)
                        continue
                #print("%s invalidates %s" % (usage, prev_usage))
                self.borrows.add(prev_usage.id)

    def processUsages(self, usage, node, key):
        usages = UsageSet()
        for incoming in node.incoming:
            edge = self.cfg.edges[incoming]
            if edge.from_node in self.usages:
                prev = self.usages[edge.from_node]
                usages += prev
        if usage:
            self.invalidate(usage, usages)
            usages.add(usage)
        if key in self.usages:
            old_usages = self.usages[key]
            if old_usages == usages:
                return False
            self.usages[key] = usages
        else:
            self.usages[key] = usages
        return True

    def getNodeUsage(self, node, key):
        if node.usage:
            u = Usage()
            u.id = key
            u.path = node.usage
            return u
        else:
            return None

    def processNode(self, key):
        #print("processNode ", key)
        node = self.cfg.getNode(key)
        usage = self.getNodeUsage(node, key)
        updatedUsages = self.processUsages(usage, node, key)
        if updatedUsages:
            for outgoing in node.outgoing:
                edge = self.cfg.edges[outgoing]
                self.processNode(edge.to_node)

    def update(self):
        borrows = set()
        for b in self.borrows:
            if isinstance(b, CFG.InstructionKey):
                self.fn.body.getInstruction(b.id).borrow = True
                borrows.add(b.id)
        for c in self.cancelled_drops:
            self.fn.body.getInstruction(c.id).cancelled = True
        for (key, node) in self.cfg.nodes.items():
            #print("key %s, usage %s/%s" % (key, node.usage, type(node.usage)))
            #print("all usages: %s" % self.usages[key])
            if isinstance(key, CFG.InstructionKey):
                witnessed_usages = self.usages[key]
                moves = set()
                for witnessed_usage in witnessed_usages.usages:
                    if witnessed_usage.id.id not in borrows:
                        moves.add(witnessed_usage.path)
                instruction = self.fn.body.getInstruction(key.id)    
                instruction.moves = moves
                if node.usage is not None:
                    instruction.usage = node.usage

    def printUsages(self):
        for (id, usage) in self.usages.items():
            if usage.len() > 0:
                print("   Usages for %s" % id)
                print("   %s" % usage)

def checkFn(fn):
    #print("Checking %s" % fn.name)
    #fn.body.dump()
    cfgbuilder = CFGBuilder.CFGBuilder()
    cfg = cfgbuilder.build(fn)
    borrowchecker = Borrowchecker(cfg, fn)
    borrowchecker.check()
    borrowchecker.update()
    # borrowchecker.printUsages()
    for b in borrowchecker.borrows:
        cfg.getNode(b).color = "#cf03fc"
    for c in borrowchecker.cancelled_drops:
        fn.body.getInstruction(c.id).cancelled = True
        cfg.getNode(c).color = "#ff99ff"
    cfg.printDot()

def cleanDrops(program):
    for (name, fn) in program.functions.items():
        for b in fn.body.blocks:
            for (index, i) in enumerate(b.instructions):
                if isinstance(i, Instruction.DropVar):
                    if i.cancelled:
                        nop = Instruction.Nop()
                        nop.id = i.id
                        b.instructions[index] = nop
            while True:
                if isinstance(b.instructions[-1], Instruction.Nop):
                    b.instructions.pop()
                else:
                    break

def processProgram(program):
    for (name, fn) in program.functions.items():
        checkFn(fn)
    cleanDrops(program)