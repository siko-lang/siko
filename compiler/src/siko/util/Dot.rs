use std::io::Write;

#[derive(Clone)]
pub enum NodeStyle {
    Simple(String),
    Record(String, Vec<String>), // title and list of items
}

pub struct Graph {
    pub name: String,
    pub nodes: Vec<(String, NodeStyle)>,
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
        let style = match label {
            Some(l) => NodeStyle::Simple(l),
            None => NodeStyle::Simple("".to_string()),
        };
        self.nodes.push((name.clone(), style));
        name
    }

    pub fn addRecordNode(&mut self, title: String, items: Vec<String>) -> String {
        let name = format!("node{}", self.nodes.len());
        self.nodes.push((name.clone(), NodeStyle::Record(title, items)));
        name
    }

    pub fn addEdge(&mut self, from: String, to: String, label: Option<String>) {
        self.edges.push((from, to, label));
    }

    pub fn printDot(&self) {
        let mut f = std::fs::File::create(format!("dots/{}.dot", self.name,)).expect("failed to open dot file");
        write!(f, "digraph D {{\n").unwrap();
        write!(f, "node [fontname=Arial splines=ortho];\n").unwrap();
        for (node, style) in &self.nodes {
            match style {
                NodeStyle::Simple(label) => {
                    write!(
                        f,
                        "{} [shape=rectangle label=\"{}\" style=\"filled\" fillcolor=\"{}\"]\n",
                        node,
                        escape_for_dot(label),
                        "yellow"
                    )
                    .unwrap();
                }
                NodeStyle::Record(title, items) => {
                    let mut record_label = format!("{{{}|", escape_for_dot(title));
                    for (i, item) in items.iter().enumerate() {
                        if i > 0 {
                            record_label.push('|');
                        }
                        record_label.push_str(&escape_for_dot(item));
                    }
                    record_label.push('}');
                    write!(
                        f,
                        "{} [shape=record label=\"{}\" style=\"filled\" fillcolor=\"{}\"]\n",
                        node, record_label, "yellow"
                    )
                    .unwrap();
                }
            }
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

fn escape_for_dot(text: &str) -> String {
    text.replace("\"", "\\\"")
        .replace("{", "\\{")
        .replace("}", "\\}")
        .replace("<", "\\<")
        .replace(">", "\\>")
        .replace("|", "\\|")
}
