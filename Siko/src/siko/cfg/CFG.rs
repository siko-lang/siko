use std::collections::BTreeMap;
use std::fmt::Display;
use std::io::Write;

use crate::siko::ir::Function::InstructionId;
use crate::siko::ir::Type::Type;
use crate::siko::ownership::Path::Path;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Key {
    DropKey(InstructionId, String),
    Instruction(InstructionId),
    LoopEnd(InstructionId),
    LoopStart(InstructionId),
    If(InstructionId),
    End,
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Key::DropKey(id, v) => write!(f, "drop({}, {})", id, v),
            Key::Instruction(id) => write!(f, "{}", id),
            Key::LoopEnd(id) => write!(f, "loopend({})", id),
            Key::LoopStart(id) => write!(f, "loopstart({})", id),
            Key::If(id) => write!(f, "if({})", id),
            Key::End => write!(f, "End"),
        }
    }
}

#[derive(Debug)]
pub struct Edge {
    pub from: Key,
    pub to: Key,
}

impl Edge {
    pub fn new(f: Key, t: Key) -> Edge {
        Edge { from: f, to: t }
    }
}

#[derive(Debug)]
pub enum NodeKind {
    Bind(String),
    Generic,
    IfEnd,
    ValueRef,
    LoopStart,
    LoopEnd,
    End,
}

impl Display for NodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            NodeKind::Bind(v) => write!(f, "bind({})", v),
            NodeKind::Generic => write!(f, "generic"),
            NodeKind::IfEnd => write!(f, "ifend"),
            NodeKind::ValueRef => write!(f, "valueref"),
            NodeKind::LoopStart => write!(f, "loopstart"),
            NodeKind::LoopEnd => write!(f, "loopend"),
            NodeKind::End => write!(f, "end"),
        }
    }
}
#[derive(Debug)]
pub struct Node {
    kind: NodeKind,
    pub ty: Type,
    pub incoming: Vec<u64>,
    pub outgoing: Vec<u64>,
    pub usage: Option<Path>,
    pub color: String,
    pub extra: String,
}

impl Node {
    pub fn new(kind: NodeKind, ty: Type) -> Node {
        Node {
            kind: kind,
            ty: ty,
            incoming: Vec::new(),
            outgoing: Vec::new(),
            usage: None,
            color: "yellow".to_string(),
            extra: String::new(),
        }
    }
}

pub struct CFG {
    name: String,
    pub nodes: BTreeMap<Key, Node>,
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

    pub fn setColor(&mut self, key: &Key, color: String) {
        let node = self.nodes.get_mut(key).unwrap();
        node.color = color;
    }

    pub fn setExtra(&mut self, key: &Key, extra: String) {
        let node = self.nodes.get_mut(key).unwrap();
        node.extra = extra;
    }

    pub fn getEdge(&self, id: u64) -> &Edge {
        &self.edges[id as usize]
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

    pub fn printDot(&self) {
        let mut f = std::fs::File::create(format!("dots/cfg_{}.dot", self.name,))
            .expect("failed to open dot file");
        write!(f, "digraph D {{\n").unwrap();
        write!(f, "node [shape=record fontname=Arial splines=ortho];\n").unwrap();
        let mut keymap = BTreeMap::new();
        for (index, (key, node)) in self.nodes.iter().enumerate() {
            keymap.insert(key, index);
            let label = if let Some(usage) = &node.usage {
                format!("{}_{}", key, usage)
            } else {
                match &node.kind {
                    NodeKind::Generic => format!("{}", key),
                    kind => format!("{}_{}", key, kind),
                }
            };
            let mut label = format!("{} = {}", label, node.ty);
            if !node.extra.is_empty() {
                label = format!("{}\n{}", label, node.extra);
            }
            write!(
                f,
                "node{} [label=\"{}\" style=\"filled\" shape=\"box\" fillcolor=\"{}\"]\n",
                index, label, node.color
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
