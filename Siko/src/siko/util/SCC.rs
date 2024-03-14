#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct NodeId {
    value: usize,
}

#[derive(Debug, Clone)]
pub struct Node {
    id: NodeId,
    index: usize,
    lowLink: Option<usize>,
    onStack: bool,
    neighbours: Vec<NodeId>,
}

impl Node {
    fn new(id: NodeId) -> Node {
        Node {
            id: id,
            index: 0,
            lowLink: None,
            onStack: false,
            neighbours: Vec::new(),
        }
    }

    fn isVisited(&self) -> bool {
        self.lowLink.is_some()
    }
}

pub struct Graph {
    nodes: Vec<Node>,
    sccs: Vec<Vec<NodeId>>,
    stack: Vec<NodeId>,
    index: usize,
}

impl Graph {
    pub fn new() -> Graph {
        Graph {
            nodes: Vec::new(),
            sccs: Vec::new(),
            stack: Vec::new(),
            index: 0,
        }
    }

    fn getNode(&mut self, nodeId: NodeId) -> &mut Node {
        &mut self.nodes[nodeId.value]
    }

    fn initNode(&mut self, nodeId: NodeId) {
        let index = self.index;
        let node = self.getNode(nodeId);
        node.index = index;
        node.lowLink = Some(node.index);
        self.index += 1;
    }

    fn updateLowLink(&mut self, nodeId: NodeId, otherLowLink: usize) {
        let node = self.getNode(nodeId);
        node.lowLink = Some(std::cmp::min(node.lowLink.unwrap(), otherLowLink));
    }

    fn pushToStack(&mut self, nodeId: NodeId) {
        let node = self.getNode(nodeId);
        node.onStack = true;
        self.stack.push(nodeId);
    }

    fn startOfSCC(&mut self, nodeId: NodeId) -> bool {
        let node = self.getNode(nodeId);
        return node.index == node.lowLink.unwrap();
    }

    fn createSCC(&mut self, rootId: NodeId) {
        let rootNode = self.getNode(rootId);
        let rootIndex = rootNode.index;
        let mut scc = Vec::new();
        loop {
            let lastId = self.stack.pop().unwrap();
            let last = self.getNode(lastId);
            last.onStack = false;
            scc.push(lastId);
            if last.index == rootIndex {
                self.sccs.push(scc);
                break;
            }
        }
    }

    fn dfs(&mut self, nodeId: NodeId) {
        let node = self.getNode(nodeId);
        let neighbours = node.neighbours.clone();
        if node.lowLink != None {
            return;
        }
        self.initNode(nodeId);
        self.pushToStack(nodeId);
        for n in neighbours {
            self.checkNode(nodeId, n);
        }
        if self.startOfSCC(nodeId) {
            self.createSCC(nodeId);
        }
    }

    fn checkNode(&mut self, current: NodeId, neighbourId: NodeId) {
        let neighbourNode = self.getNode(neighbourId).clone();
        if neighbourNode.isVisited() {
            if neighbourNode.onStack {
                self.updateLowLink(current, neighbourNode.index);
            }
        } else {
            self.dfs(neighbourId);
            let neighbourNode = self.getNode(neighbourId).clone();
            self.updateLowLink(current, neighbourNode.lowLink.unwrap());
        }
    }

    fn collectSCCs(&mut self) -> Vec<Vec<NodeId>> {
        let mut nodeIds = Vec::new();
        for n in &self.nodes {
            nodeIds.push(n.id);
        }
        for id in nodeIds {
            self.dfs(id);
        }
        return self.sccs.clone();
    }

    fn addNode(&mut self) -> NodeId {
        let index = self.nodes.len();
        let id = NodeId { value: index };
        let node = Node::new(id);
        self.nodes.push(node);
        id
    }

    fn addNeighbour(&mut self, source: NodeId, dest: NodeId) {
        let node = self.getNode(source);
        node.neighbours.push(dest);
    }
}
