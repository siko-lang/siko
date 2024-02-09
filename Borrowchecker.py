import IR
import copy
import CFG
import CFGBuilder

class WholePath(object):
    def __init__(self):
        self.var = None

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

class Borrowchecker(object):
    def __init__(self, cfg, fn):
        self.fn = fn
        self.cfg = cfg
        self.usages = {}
        self.borrows = set()

    def check(self):
        sources = self.cfg.getSources()
        for source in sources:
            self.processNode(source)

    def invalidates(self, current, other):
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
                current.fields[:min_len] == other.fields[:min_len]

    def invalidate(self, usage, usages):
        for prev_usage in usages:
            if self.invalidates(usage, prev_usage):
                self.borrows.insert(prev_usage.id)

    def processUsages(self, usage, node, key):
        usages = set()
        for incoming in node.incoming:
            edge = self.cfg.edges[incoming]
            if edge.from_node in self.usages:
                usages += self.usages[edge.from_node]
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

    def getNodeUsage(self, node):
        return node.usage

    def processNode(self, key):
        node = self.cfg.getNode(key)
        usage = self.getNodeUsage(node)
        updatedUsages = self.processUsages(usage, node, key)
        if updatedUsages:
            for outgoing in node.outgoing:
                edge = self.cfg.edges[outgoing]
                self.processNode(edge.to_node)
        pass

def checkFn(fn):
    cfgbuilder = CFGBuilder.CFGBuilder()
    cfg = cfgbuilder.build(fn)
    cfg.printDot()
    borrowchecker = Borrowchecker(cfg, fn)
    borrowchecker.check()

def processProgram(program):
    for (name, fn) in program.functions.items():
        checkFn(fn)
    for (_, clazz) in program.classes.items():
        for m in clazz.methods:
            checkFn(m)