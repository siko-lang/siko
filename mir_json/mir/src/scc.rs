#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct NodeId {
    id: i64,
}

#[derive(Clone)]
struct Node {
    id: NodeId,
    index: i64,
    low_link: Option<i64>,
    on_stack: bool,
    neighbours: Vec<NodeId>,
}

impl Node {
    fn is_visited(&self) -> bool {
        self.low_link.is_some()
    }
}

pub struct Graph {
    nodes: Vec<Node>,
    sccs: Vec<Vec<NodeId>>,
    stack: Vec<NodeId>,
    index: i64,
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

    fn init_node(&mut self, id: NodeId) {
        let node = &mut self.nodes[id.id as usize];
        node.index = self.index;
        node.low_link = Some(self.index);
        self.index += 1;
    }

    fn update_low_link(&mut self, id: NodeId, other_low_link: i64) {
        let node = &mut self.nodes[id.id as usize];
        let old_low_link = node.low_link.unwrap();
        let low_link = std::cmp::min(old_low_link, other_low_link);
        node.low_link = Some(low_link);
    }

    fn push_to_stack(&mut self, id: NodeId) {
        let node = &mut self.nodes[id.id as usize];
        node.on_stack = true;
        self.stack.push(id);
    }

    fn start_of_scc(&self, id: NodeId) -> bool {
        let node = &self.nodes[id.id as usize];
        node.index == node.low_link.unwrap()
    }

    fn create_scc(&mut self, id: NodeId) {
        let root_node = self.nodes[id.id as usize].clone();
        let mut scc = Vec::new();
        loop {
            let end = self.stack.pop().unwrap();
            let last = &mut self.nodes[end.id as usize];
            last.on_stack = false;
            scc.push(end);
            if last.index == root_node.index {
                break;
            }
        }
        self.sccs.push(scc);
    }

    fn dfs(&mut self, id: NodeId) {
        let node = &self.nodes[id.id as usize];
        let neighbours = node.neighbours.clone();
        match node.low_link {
            Some(_) => {
                return;
            }
            None => {
                self.init_node(id);
            }
        }
        self.push_to_stack(id);
        for n in neighbours {
            self.check_node(id, n);
        }
        if self.start_of_scc(id) {
            self.create_scc(id);
        }
    }

    fn check_node(&mut self, current: NodeId, neighbour: NodeId) {
        let neighbour_node = self.nodes[neighbour.id as usize].clone();
        if neighbour_node.is_visited() {
            if neighbour_node.on_stack {
                self.update_low_link(current, neighbour_node.index);
            }
        } else {
            self.dfs(neighbour);
            let neighbour_node = self.nodes[neighbour.id as usize].clone();
            self.update_low_link(current, neighbour_node.low_link.unwrap());
        }
    }

    pub fn collect_sccs(&mut self) -> Vec<Vec<NodeId>> {
        let mut nodes = Vec::new();
        for node in &self.nodes {
            nodes.push(node.id);
        }
        for id in nodes {
            self.dfs(id);
        }
        self.sccs.clone()
    }

    pub fn add_node(&mut self) -> NodeId {
        let id = self.nodes.len() as i64;
        let id = NodeId { id: id };
        let node = Node {
            id: id,
            index: 0,
            low_link: None,
            on_stack: false,
            neighbours: Vec::new(),
        };
        self.nodes.push(node);
        id
    }

    pub fn add_neighbour(&mut self, id: NodeId, neighbour: NodeId) {
        let node = &mut self.nodes[id.id as usize];
        node.neighbours.push(neighbour);
    }
}
