use crate::siko::hir::Data::Enum;
use crate::siko::qualifiedname::QualifiedName;
use crate::siko::resolver::ModuleResolver::ModuleResolver;
use crate::siko::syntax::Pattern::Pattern;
use std::collections::BTreeMap;

use super::Choice::{Choice, ChoiceKind};
use super::Collector::{Collector, Edge};
use super::Decision::{Decision, DecisionBuilder, Decisions, Path};
use super::Resolver::Resolver;

struct WildcardNode {
    choices: BTreeMap<Choice, Path>,
}

impl WildcardNode {
    fn new(next: Option<Path>) -> WildcardNode {
        let mut choices = BTreeMap::new();
        if let Some(next) = next {
            choices.insert(Choice::Wildcard, next);
        }
        WildcardNode { choices: choices }
    }

    fn add(&mut self, decision: Decision, decisions: Decisions) {}

    fn dump(&self, level: u32, builder: &NodeBuilder) {
        let indent = " ".repeat(level as usize);
        if let Some(next) = self.choices.get(&Choice::Wildcard) {
            println!("{}Wildcard: next {}", indent, next);
            builder.dumpNode(next, level + 1);
        } else {
            println!("{}Wildcard: empty", indent);
        }
    }
}

struct EnumNode {
    choices: BTreeMap<Choice, Path>,
}

impl EnumNode {
    fn add(&mut self, decision: Decision, decisions: Decisions) {
        match decision.choice {
            Choice::Variant(variant, _) => {}
            _ => unreachable!(),
        }
    }

    fn dump(&self, level: u32, builder: &NodeBuilder) {
        let indent = " ".repeat(level as usize);
        println!("{}Enum:", indent);
        let indent = " ".repeat((level + 1) as usize);
        for (name, path) in &self.choices {
            println!("{}{}: {}", indent, name, path);
            builder.dumpNode(path, level + 2);
        }
    }
}

struct IntegerLiteralNode {
    choices: BTreeMap<Choice, Path>,
}

impl IntegerLiteralNode {
    fn add(&mut self, decision: Decision, decisions: Decisions) {}

    fn dump(&self, level: u32, builder: &NodeBuilder) {
        let indent = " ".repeat(level as usize);
        println!("{}Integer Literal:", indent);
        for (choice, path) in &self.choices {
            println!("{}{}: {}", indent, choice, path);
            builder.dumpNode(path, level + 1);
        }
    }
}

enum Node {
    Wildcard(WildcardNode),
    Enum(EnumNode),
    IntegerLiteral(IntegerLiteralNode),
}

struct NodeBuilder<'a> {
    resolver: &'a Resolver<'a>,
    collector: &'a Collector,
    nodes: BTreeMap<Path, Node>,
    nextEnd: i64,
}

impl<'a> NodeBuilder<'a> {
    fn new(resolver: &'a Resolver<'a>, collector: &'a Collector) -> NodeBuilder<'a> {
        NodeBuilder {
            resolver: resolver,
            collector: collector,
            nodes: BTreeMap::new(),
            nextEnd: 0,
        }
    }

    fn build(&mut self) {
        self.buildNode(Path::Root);
    }

    fn add(&mut self, index: u32, mut decisions: Vec<Decision>) {
        let current = decisions.remove(0);
        let node = self.nodes.get(&current.path).expect("node not found");
        // match node {
        //     Node::Wildcard(node) => {
        //         node.dump(level, self);
        //     }
        //     Node::Enum(node) => {
        //         node.dump(level, self);
        //     }
        //     Node::IntegerLiteral(node) => {
        //         node.dump(level, self);
        //     }
        // }
    }

    fn dump(&self) {
        self.dumpNode(&Path::Root, 0);
    }

    fn dumpNode(&self, path: &Path, level: u32) {
        let node = self.nodes.get(path).expect("node not found");
        match node {
            Node::Wildcard(node) => {
                node.dump(level, self);
            }
            Node::Enum(node) => {
                node.dump(level, self);
            }
            Node::IntegerLiteral(node) => {
                node.dump(level, self);
            }
        }
    }

    fn buildNode(&mut self, path: Path) {
        let node = match self.collector.kind(&path) {
            ChoiceKind::Variant(enumName) => self.buildEnum(enumName, path.clone()),
            ChoiceKind::Class(name) => todo!(),
            ChoiceKind::Wildcard => self.buildWildCard(path.clone()),
            ChoiceKind::Tuple => todo!(),
            ChoiceKind::StringLiteral => todo!(),
            ChoiceKind::IntegerLiteral => self.buildIntegerLiteral(path.clone()),
        };
        self.nodes.insert(path, node);
    }

    fn getNextEnd(&mut self) -> Path {
        let i = self.nextEnd;
        self.nextEnd += 1;
        Path::End(i)
    }

    fn buildEnum(&mut self, enumName: QualifiedName, path: Path) -> Node {
        let mut choices = BTreeMap::new();
        let e = self.resolver.enums.get(&enumName).expect("enum not found");
        for v in &e.variants {
            let choice = Choice::Variant(v.name.clone(), enumName.clone());
            let edge = Edge {
                path: path.clone(),
                choice: choice.clone(),
            };
            match self.collector.edges.get(&edge) {
                Some(target) => {
                    self.buildNode(target.clone());
                    choices.insert(choice.clone(), target.clone());
                }
                None => {
                    let target = self.getNextEnd();
                    self.nodes.insert(target.clone(), Node::Wildcard(WildcardNode::new(None)));
                    choices.insert(choice.clone(), target.clone());
                }
            };
        }
        Node::Enum(EnumNode { choices })
    }

    fn buildWildCard(&mut self, path: Path) -> Node {
        let edge = Edge {
            path: path.clone(),
            choice: Choice::Wildcard,
        };
        match self.collector.edges.get(&edge) {
            Some(target) => {
                self.buildNode(target.clone());
                Node::Wildcard(WildcardNode::new(Some(target.clone())))
            }
            None => Node::Wildcard(WildcardNode::new(None)),
        }
    }

    fn buildIntegerLiteral(&mut self, path: Path) -> Node {
        let mut choices = BTreeMap::new();
        for (edge, target) in &self.collector.edges {
            if edge.path == path {
                choices.insert(edge.choice.clone(), target.clone());
                self.buildNode(target.clone());
            }
        }
        Node::IntegerLiteral(IntegerLiteralNode { choices: choices })
    }
}

pub struct MatchCompiler<'a> {
    branches: Vec<Pattern>,
    resolver: Resolver<'a>,
    collector: Collector,
}

impl<'a> MatchCompiler<'a> {
    pub fn new(
        branches: Vec<Pattern>,
        moduleResolver: &'a ModuleResolver,
        variants: &'a BTreeMap<QualifiedName, QualifiedName>,
        enums: &'a BTreeMap<QualifiedName, Enum>,
    ) -> MatchCompiler<'a> {
        MatchCompiler {
            branches: branches,
            resolver: Resolver::new(moduleResolver, variants, enums),
            collector: Collector::new(),
        }
    }

    pub fn check(&mut self) {
        let mut allDecisions = Vec::new();
        for branch in self.branches.clone().into_iter() {
            let mut builder = DecisionBuilder::new(&self.resolver);
            builder.build(branch, Path::Root);
            println!("Decision {}", builder.decisions);
            let mut prev: Option<Decision> = None;
            for decision in &builder.decisions.decisions {
                if let Some(prev) = prev {
                    //println!("{} -> {}: {}", prev, decision.path, decision.choice);
                    self.collector.addEdge(prev.path.clone(), prev.choice.clone(), decision.path.clone());
                }
                self.collector
                    .add(decision.path.clone(), decision.choice.kind(), decision.location.clone());
                prev = Some(decision.clone());
            }
            allDecisions.push(builder.decisions.decisions);
        }
        // for (p, k) in &self.collector.kinds {
        //     println!("kind {}:{}", p, k);
        // }
        // for (e, dest) in &self.collector.edges {
        //     println!("edge {} {} {}", e.path, e.choice, dest);
        // }
        let mut nodeBuilder = NodeBuilder::new(&self.resolver, &self.collector);
        nodeBuilder.build();
        for (index, decisions) in allDecisions.into_iter().enumerate() {
            nodeBuilder.add(index as u32, decisions);
        }
        nodeBuilder.dump();
    }
}
