use std::collections::BTreeMap;
use std::io::Write;

use crate::siko::ir::Function::InstructionId;
use crate::siko::ownership::Path::Path;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Key {
    DropKey(String),
    Instruction(InstructionId),
    LoopEnd(InstructionId),
    LoopStart(InstructionId),
    If(InstructionId),
    End,
}

#[derive(Debug)]
pub struct Edge {
    from: Key,
    to: Key,
}

impl Edge {
    pub fn new(f: Key, t: Key) -> Edge {
        Edge { from: f, to: t }
    }
}

#[derive(Debug)]
pub struct Node {
    kind: String,
    incoming: Vec<u64>,
    outgoing: Vec<u64>,
    pub usage: Option<Path>,
    pub color: String,
}

impl Node {
    pub fn new(kind: String) -> Node {
        Node {
            kind: kind,
            incoming: Vec::new(),
            outgoing: Vec::new(),
            usage: None,
            color: "yellow".to_string(),
        }
    }
}

pub struct CFG {
    name: String,
    nodes: BTreeMap<Key, Node>,
    edges: Vec<Edge>,
}

impl CFG {
    pub fn new(name: String) -> CFG {
        CFG {
            name: name,
            nodes: BTreeMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn addNode(&mut self, key: Key, node: Node) {
        self.nodes.insert(key, node);
    }

    pub fn addEdge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    pub fn getNode(&self, key: &Key) -> &Node {
        self.nodes.get(key).unwrap()
    }

    pub fn getSources(&self) -> Vec<Key> {
        let mut sources = Vec::new();
        for (key, node) in &self.nodes {
            if node.incoming.len() == 0 {
                sources.push(key.clone());
            }
        }
        sources
    }

    pub fn updateEdges(&mut self) {
        for (index, edge) in self.edges.iter().enumerate() {
            let from_node = self.nodes.get_mut(&edge.from).unwrap();
            from_node.outgoing.push(index as u64);
            let to_node = &mut self.nodes.get_mut(&edge.to).unwrap();
            to_node.incoming.push(index as u64);
        }
    }

    fn printDot(&self) {
        let mut f = std::fs::File::create(format!("dots/cfg_{}.dot", self.name,))
            .expect("failed to open dot file");
        write!(f, "digraph D {{\n").unwrap();
        write!(f, "node [shape=record fontname=Arial splines=ortho];\n").unwrap();
        let mut keymap = BTreeMap::new();
        for (index, (key, node)) in self.nodes.iter().enumerate() {
            keymap.insert(key, index);
            let mut label = node.kind.clone();
            if let Some(usage) = &node.usage {
                label = format!("{:?}", usage);
            }
            write!(
                f,
                "node{} [label=\"{:?}_{}\" style=\"filled\" shape=\"box\" fillcolor=\"{}\"]\n",
                index, key, label, node.color
            )
            .unwrap();
        }
        for edge in &self.edges {
            write!(
                f,
                "node{} -> node{}\n",
                keymap[&edge.from], keymap[&edge.to]
            )
            .unwrap();
        }
        write!(f, "}}\n").unwrap();
    }
}
