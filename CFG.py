import IR
import Util

class EndKey(object):
    def __init__(self):
        pass

class IfKey(object):
    def __init__(self):
        self.id = None

class InstructionKey(object):
    def __init__(self):
        self.id = None

class Edge(object):
    def __init_(self):
        self.from_node = None
        self.to_node = None

class Node(object):
    def __init__(self):
        self.kind = None
        self.incoming = []
        self.outgoing = []

class CFG(object):
    def __init__(self):
        self.nodes = {}
        self.edges = []
        self.fn = None
        self.end_key = EndKey()
        self.loop_starts = []
        self.loop_ends = []

    def addNode(self, key, node):
        self.nodes[key] = node

    def addEdge(self, edge):
        self.edges.append(edge)

    def processGenericInstruction(self, i, last):
        instr_key = InstructionKey()
        instr_key.id = i.id
        instr_node = Node()
        instr_node.kind = str(i)
        self.addNode(instr_key, instr_node)
        edge = Edge()
        edge.from_node = last
        edge.to_node = instr_key
        self.addEdge(edge)
        return instr_key

    def processBlock(self, block):
        last = None
        for i in block.instructions:
            if isinstance(i, IR.BlockRef):
                b = self.fn.body.getBlock(i)
                last = self.processBlock(b)
            elif isinstance(i, IR.NamedFunctionCall):
                last = self.processGenericInstruction(i, last)
            elif isinstance(i, IR.Bind):
                last = self.processGenericInstruction(i, last)
            elif isinstance(i, IR.VarRef):
                last = self.processGenericInstruction(i, last)
            elif isinstance(i, IR.MemberAccess):
                last = self.processGenericInstruction(i, last)
            elif isinstance(i, IR.BoolLiteral):
                last = self.processGenericInstruction(i, last)
            elif isinstance(i, IR.If):
                if_key = IfKey()
                if_key.id = i.id
                if_end = Node()
                if_end.kind = "if_end"
                self.addNode(if_key, if_end)
                true_block = self.fn.body.getBlock(i.true_branch)
                true_last = self.processBlock(true_block)
                if true_last:
                    edge = Edge()
                    edge.from_node = true_last
                    edge.to_node = if_key
                    self.addEdge(edge)
                false_block = self.fn.body.getBlock(i.false_branch)
                false_last = self.processBlock(false_block)
                if false_last:
                    edge = Edge()
                    edge.from_node = false_last
                    edge.to_node = if_key
                    self.addEdge(edge)
                last = if_key
            else:
                Util.error("Unhandled in cfg %s: %s" % (type(i), i))
        return last

    def build(self, fn):
        self.fn = fn
        self.processBlock(self.fn.body.getFirst())

