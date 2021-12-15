use crate::Counter;
use std::fs::File;
use std::io::Result as IoResult;
use std::io::Write;

pub struct Graph {
    pub name: String,
    pub next_index: Counter,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

impl Graph {
    pub fn new(name: String) -> Graph {
        Graph {
            name: name,
            next_index: Counter::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, name: String) -> usize {
        let index = self.next_index.next();
        let node = Node {
            name: name,
            index: index,
            elements: Vec::new(),
        };
        self.nodes.push(node);
        index
    }

    pub fn add_element(&mut self, node: usize, element: String) {
        let node = &mut self.nodes[node];
        node.elements.push(element);
    }

    pub fn add_edge(&mut self, name: Option<String>, from: usize, to: usize) {
        let edge = Edge {
            name: name,
            from: from,
            to: to,
        };
        self.edges.push(edge);
    }

    pub fn generate_dot(&self) -> IoResult<()> {
        let dots_folder = "dots";
        let _ = std::fs::create_dir_all(dots_folder);
        let filename = format!("{}/{}.dot", dots_folder, self.name);
        let mut output = File::create(filename)?;
        write!(output, "digraph D {{\n")?;
        write!(
            output,
            "node [shape=record fontname=Arial splines=ortho];\n"
        )?;

        for node in &self.nodes {
            let elements = node.elements.join("|");
            write!(
                output,
                "node{} [label=\"{{{}|{}}}\"]\n",
                node.index, node.name, elements
            )?;
        }

        for edge in &self.edges {
            if let Some(name) = &edge.name {
                write!(
                    output,
                    "node{} -> node{} [label=\"{}\"]\n",
                    edge.from, edge.to, name
                )?;
            } else {
                write!(output, "node{} -> node{} \n", edge.from, edge.to)?;
            }
        }

        write!(output, "}}\n")?;
        Ok(())
    }
}

pub struct Node {
    pub name: String,
    pub index: usize,
    pub elements: Vec<String>,
}

pub struct Edge {
    pub name: Option<String>,
    pub from: usize,
    pub to: usize,
}
