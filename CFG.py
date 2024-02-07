import IR
import Util

class EndKey(object):
    def __init__(self):
        pass
    
    def __str__(self):
        return "<end>"

class IfKey(object):
    def __init__(self):
        self.id = None

    def __str__(self):
        return str(self.id)

class LoopStart(object):
    def __init__(self):
        self.id = None

    def __str__(self):
        return str(self.id)

class LoopEnd(object):
    def __init__(self):
        self.id = None

    def __str__(self):
        return str(self.id)

class InstructionKey(object):
    def __init__(self):
        self.id = None

    def __str__(self):
        return str(self.id)

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
        if last:
            edge = Edge()
            edge.from_node = last
            edge.to_node = instr_key
            self.addEdge(edge)
        return instr_key

    def processBlock(self, block, last = None):
        for i in block.instructions:
            if isinstance(i, IR.BlockRef):
                b = self.fn.body.getBlock(i)
                last = self.processBlock(b, last)
            elif isinstance(i, IR.NamedFunctionCall):
                last = self.processGenericInstruction(i, last)
            elif isinstance(i, IR.MethodCall):
                last = self.processGenericInstruction(i, last)
            elif isinstance(i, IR.Bind):
                last = self.processGenericInstruction(i, last)
            elif isinstance(i, IR.VarRef):
                last = self.processGenericInstruction(i, last)
            elif isinstance(i, IR.MemberAccess):
                last = self.processGenericInstruction(i, last)
            elif isinstance(i, IR.BoolLiteral):
                last = self.processGenericInstruction(i, last)
            elif isinstance(i, IR.Return):
                self.processGenericInstruction(i, last)
                last = None
            elif isinstance(i, IR.Break):
                instr_key = InstructionKey()
                instr_key.id = i.id
                instr_node = Node()
                instr_node.kind = str(i)
                self.addNode(instr_key, instr_node)
                edge = Edge()
                edge.from_node = instr_key
                edge.to_node = self.loop_ends[-1]
                self.addEdge(edge)
                if last:
                    edge = Edge()
                    edge.from_node = last
                    edge.to_node = instr_key
                    self.addEdge(edge)
                last = None
            elif isinstance(i, IR.Continue):
                instr_key = InstructionKey()
                instr_key.id = i.id
                instr_node = Node()
                instr_node.kind = str(i)
                self.addNode(instr_key, instr_node)
                edge = Edge()
                edge.from_node = instr_key
                edge.to_node = self.loop_starts[-1]
                self.addEdge(edge)
                if last:
                    edge = Edge()
                    edge.from_node = last
                    edge.to_node = instr_key
                    self.addEdge(edge)
                last = None
            elif isinstance(i, IR.Loop):
                loop_start_key = LoopStart()
                loop_start_key.id = i.id
                loop_start_node = Node()
                loop_start_node.kind = "loop_start"
                self.loop_starts.append(loop_start_key)
                self.addNode(loop_start_key, loop_start_node)
                if last:
                    edge = Edge()
                    edge.from_node = last
                    edge.to_node = loop_start_key
                    self.addEdge(edge)
                loop_var_key = LoopStart()
                loop_var_key.id = i.id
                loop_var_node = Node()
                loop_var_node.kind = "loop_var %s" % i.var
                self.addNode(loop_var_key, loop_var_node)
                edge = Edge()
                edge.from_node = loop_start_key
                edge.to_node = loop_var_key
                self.addEdge(edge)
                loop_end_key = LoopEnd()
                loop_end_key.id = i.id
                loop_end_node = Node()
                loop_end_node.kind = "loop_end"
                self.addNode(loop_end_key, loop_end_node)
                self.loop_ends.append(loop_end_key)
                loop_body = self.fn.body.getBlock(i.body)
                loop_last = self.processBlock(loop_body, loop_var_key)
                if loop_last:
                    edge = Edge()
                    edge.from_node = loop_last
                    edge.to_node = loop_start_key
                    self.addEdge(edge)
                self.loop_starts.pop()
                self.loop_ends.pop()
                last = loop_end_key
            elif isinstance(i, IR.If):
                if_key = IfKey()
                if_key.id = i.id
                if_end = Node()
                if_end.kind = "if_end"
                self.addNode(if_key, if_end)
                true_block = self.fn.body.getBlock(i.true_branch)
                true_last = self.processBlock(true_block, last)
                if true_last:
                    edge = Edge()
                    edge.from_node = true_last
                    edge.to_node = if_key
                    self.addEdge(edge)
                false_block = self.fn.body.getBlock(i.false_branch)
                false_last = self.processBlock(false_block, last)
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

    def printDot(self):
        f = open("dots/cfg_%s.dot" % self.fn.name, "w")
        f.write("digraph D {\n")
        f.write("node [shape=record fontname=Arial splines=ortho];\n")
        keymap = {}
        for (index, (key, node)) in enumerate(self.nodes.items()):
            keymap[key] = index
            f.write("node%s [label=\"%s_%s\" style=\"filled\" shape=\"box\" fillcolor=\"yellow\"]\n" % (index, key, node.kind))
        for edge in self.edges:
            f.write("node%s -> node%s\n" % (keymap[edge.from_node], keymap[edge.to_node]))
        f.write("}\n")
        f.close()
