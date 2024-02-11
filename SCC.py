
class NodeId(object):
    def __init__(self):
        self.value = None
    
    def __eq__(self, other):
        return self.value == other.value

    def __ne__(self, other):
        return not self.__eq__(other)

    def __hash__(self):
        return self.value.__hash__()

    def __str__(self):
        return "#%s" % self.value

class Node(object):
    def __init__(self):
        self.id = None
        self.index = None
        self.low_link = None
        self.on_stack = False
        self.neighbours = []

    def isVisited(self):
        return self.low_link != None

class Graph(object):
    def __init__(self):
        self.nodes = []
        self.sccs = []
        self.stack = []
        self.index = 0

    def getNode(self, nodeId):
        return self.nodes[nodeId.value]

    def initNode(self, nodeId):
        node = self.getNode(nodeId)
        node.index = self.index
        node.low_link = node.index
        self.index += 1

    def updateLowLink(self, nodeId, other_low_link):
        node = self.getNode(nodeId)
        node.low_link = min(node.low_link, other_low_link)

    def pushToStack(self, nodeId):
        node = self.getNode(nodeId)
        node.on_stack = True
        self.stack.append(nodeId)

    def startOfSCC(self, nodeId):
        node = self.getNode(nodeId)
        return node.index == node.low_link

    def createSCC(self, rootId):
        rootNode = self.getNode(rootId)
        scc = []
        while True:
            last_id = self.stack.pop()
            last = self.getNode(last_id)
            last.on_stack = False
            scc.append(last_id)
            if last.index == rootNode.index:
                self.sccs.append(scc)
                break

    def dfs(self, nodeId):
        node = self.getNode(nodeId)
        if node.low_link != None:
            return
        self.initNode(nodeId)
        self.pushToStack(nodeId)
        for n in node.neighbours:
            self.checkNode(nodeId, n)
        if self.startOfSCC(nodeId):
            self.createSCC(nodeId)

    def checkNode(self, current, neighbourId):
        neighbour_node = self.getNode(neighbourId)
        if neighbour_node.isVisited():
            if neighbour_node.on_stack:
                self.updateLowLink(current, neighbour_node.index)
        else:
            self.dfs(neighbourId)
            neighbour_node = self.getNode(neighbourId)
            self.updateLowLink(current, neighbour_node.low_link)

    def collectSCCs(self):
        nodeIds = []
        index = 0
        for n in self.nodes:
            nodeIds.append(n.id)
        for id in self.nodeIds:
            self.dfs(id)
        return self.sccs

    def addNode(self):
        index = len(self.nodes)
        id = NodeId()
        id.value = index
        node = Node()
        node.id = id
        node.index = 0
        node.low_link = None
        node.on_stack = False
        self.nodes.append(node)
        return id

    def addNeighbour(self, source, dest):
        node = self.getNode(source)
        node.neighbours.append(dest)

