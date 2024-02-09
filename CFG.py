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
        self.usage = None
        self.color = "yellow"

class CFG(object):
    def __init__(self, name):
        self.name = name
        self.nodes = {}
        self.edges = []

    def addNode(self, key, node):
        self.nodes[key] = node

    def addEdge(self, edge):
        self.edges.append(edge)

    def getNode(self, key):
        return self.nodes[key]

    def getSources(self):
        sources = []
        for (key, node) in self.nodes.items():
            if len(node.incoming) == 0:
                sources.append(key)
        return sources

    def updateEdges(self):
        for (index, edge) in enumerate(self.edges):
            from_node = self.nodes[edge.from_node]
            from_node.outgoing.append(index)
            to_node = self.nodes[edge.to_node]
            to_node.incoming.append(index)

    def printDot(self):
        f = open("dots/cfg_%s.dot" % self.name, "w")
        f.write("digraph D {\n")
        f.write("node [shape=record fontname=Arial splines=ortho];\n")
        keymap = {}
        for (index, (key, node)) in enumerate(self.nodes.items()):
            keymap[key] = index
            f.write("node%s [label=\"%s_%s\" style=\"filled\" shape=\"box\" fillcolor=\"%s\"]\n" % (index, key, node.kind, node.color))
        for edge in self.edges:
            f.write("node%s -> node%s\n" % (keymap[edge.from_node], keymap[edge.to_node]))
        f.write("}\n")
        f.close()
