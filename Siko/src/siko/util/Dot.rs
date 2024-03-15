use std::io::Write;
pub struct Graph {
    pub name: String,
    pub nodes: Vec<(String, Option<String>)>,
    pub edges: Vec<(String, String, Option<String>)>,
}

impl Graph {
    pub fn new(name: String) -> Graph {
        Graph {
            name: name,
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn addNode(&mut self, label: Option<String>) -> String {
        let name = format!("node{}", self.nodes.len());
        self.nodes.push((name.clone(), label));
        name
    }

    pub fn printDot(&self) {
        let mut f = std::fs::File::create(format!("dots/{}.dot", self.name,))
            .expect("failed to open dot file");
        write!(f, "digraph D {{\n").unwrap();
        write!(f, "node [shape=circle fontname=Arial splines=ortho];\n").unwrap();
        for (node, label) in &self.nodes {
            write!(
                f,
                "{} [label=\"{}\" style=\"filled\" fillcolor=\"{}\"]\n",
                node,
                label.clone().unwrap_or("".to_string()),
                "yellow"
            )
            .unwrap();
        }
        for (from, to, label) in &self.edges {
            write!(
                f,
                "{} -> {} [label=\"{}\"]\n",
                from,
                to,
                label.clone().unwrap_or("".to_string())
            )
            .unwrap();
        }
        write!(f, "}}\n").unwrap();
    }
}
