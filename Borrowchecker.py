import IR
import copy
import CFG
import CFGBuilder

class WholePath(object):
    def __init__(self, isDrop = False):
        self.var = None
        self.is_drop = isDrop

    def __str__(self):
        return "whole(%s)" % (self.var)

    def __eq__(self, other):
        if isinstance(other, WholePath):
            return self.var == other.var
        else:
            return False

    def __ne__(self, other):
        return not self.__eq__(other)

    def __hash__(self):
        return self.var.__hash__()

class PartialPath(object):
    def __init__(self):
        self.var = None
        self.fields = []

    def __str__(self):
        fields = ".".join(self.fields)
        return "partial(%s.%s)" % (self.var, fields)

    def __eq__(self, other):
        if isinstance(other, PartialPath):
            if self.var != other.var:
                return False
            if len(self.fields) != len(other.fields):
                return False
            for (index, v) in enumerate(self.fields):
                if v != other.fields[index]:
                    return False
            return True
        else:
            return False

    def __ne__(self, other):
        return not self.__eq__(other)

    def __hash__(self):
        return self.var.__hash__()

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
            if isinstance(current, WholePath) and isinstance(other, WholePath):
                return True
            if isinstance(current, WholePath) and isinstance(other, PartialPath):
                return True
            if isinstance(current, PartialPath) and isinstance(other, WholePath):
                return True
            if isinstance(current, PartialPath) and isinstance(other, PartialPath):
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
                if isinstance(usage.path, WholePath):
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
    # borrowchecker.printUsages()
    for b in borrowchecker.borrows:
        cfg.getNode(b).color = "#cf03fc"
    for c in borrowchecker.cancelled_drops:
        fn.body.getInstruction(c.id).cancelled = True
        cfg.getNode(c).color = "#ff99ff"
    cfg.printDot()

def processProgram(program):
    for (name, fn) in program.functions.items():
        checkFn(fn)