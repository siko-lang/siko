use crate::siko::hir::Data::Enum;
use crate::siko::location::Location::Location;
use crate::siko::qualifiedname::QualifiedName;
use crate::siko::resolver::Error::ResolverError;
use crate::siko::resolver::ModuleResolver::ModuleResolver;
use crate::siko::syntax::Identifier::Identifier;
use crate::siko::syntax::Pattern::{Pattern, SimplePattern};
use std::collections::{BTreeMap, BTreeSet};
use std::iter::repeat;

use super::Choice::{Choice, ChoiceKind};
use super::Collector::{Collector, Edge};
use super::Decision::{Decision, DecisionBuilder, Decisions, Path};
use super::Resolver::Resolver;

#[derive(Debug)]
enum NodeKind {
    Wildcard,
    Enum,
    IntegerLiteral,
    StringLiteral,
}

struct Match {
    index: u32,
}

struct Node {
    kind: NodeKind,
    choices: BTreeMap<Choice, Path>,
    matches: Vec<Match>,
}

impl Node {
    fn dump(&self, level: u32, builder: &NodeBuilder) {
        let indent = " ".repeat(level as usize);
        println!("{}{:?}:", indent, self.kind);
        for m in &self.matches {
            println!("{}M: {}", indent, m.index);
        }
        let indent = " ".repeat((level + 1) as usize);
        for (name, path) in &self.choices {
            println!("{}{}: {}", indent, name, path);
            builder.dumpNode(path, level + 2);
        }
    }
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

    fn add(&mut self, index: u32, decisions: Vec<Decision>) {
        let choices = decisions.into_iter().map(|d| d.choice).collect();
        self.addToNode(index, &Path::Root, choices);
    }

    fn addToNode(&mut self, index: u32, path: &Path, mut choices: Vec<Choice>) {
        println!("AddToNode: {} {}", path, index);
        let node = self.nodes.get(path).expect("node not found");
        if choices.is_empty() {
            if node.choices.is_empty() {
                let node = self.nodes.get_mut(path).expect("node not found");
                node.matches.push(Match { index: index });
                return;
            } else {
                choices.push(Choice::Wildcard);
            }
        }
        let current = choices.remove(0);
        let node = self.nodes.get(path).expect("node not found");
        let mut children = Vec::new();
        for (choice, path) in &node.choices {
            if current == Choice::Wildcard || *choice == current {
                children.push(path.clone());
            }
        }
        for child in children {
            self.addToNode(index, &child, choices.clone());
        }
    }

    fn dump(&self) {
        self.dumpNode(&Path::Root, 0);
    }

    fn dumpNode(&self, path: &Path, level: u32) {
        let node = self.nodes.get(path).expect("node not found");
        node.dump(level, self);
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
                    self.nodes.insert(
                        target.clone(),
                        Node {
                            kind: NodeKind::Wildcard,
                            choices: BTreeMap::new(),
                            matches: Vec::new(),
                        },
                    );
                    choices.insert(choice.clone(), target.clone());
                }
            };
        }
        Node {
            kind: NodeKind::Enum,
            choices,
            matches: Vec::new(),
        }
    }

    fn buildWildCard(&mut self, path: Path) -> Node {
        let edge = Edge {
            path: path.clone(),
            choice: Choice::Wildcard,
        };
        match self.collector.edges.get(&edge) {
            Some(target) => {
                self.buildNode(target.clone());
                let mut choices = BTreeMap::new();
                choices.insert(Choice::Wildcard, target.clone());
                Node {
                    kind: NodeKind::Wildcard,
                    choices: choices,
                    matches: Vec::new(),
                }
            }
            None => Node {
                kind: NodeKind::Wildcard,
                choices: BTreeMap::new(),
                matches: Vec::new(),
            },
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
        Node {
            kind: NodeKind::IntegerLiteral,
            choices: choices,
            matches: Vec::new(),
        }
    }
}

pub struct MatchCompiler<'a> {
    bodyLocation: Location,
    branches: Vec<Pattern>,
    resolver: Resolver<'a>,
    collector: Collector,
}

impl<'a> MatchCompiler<'a> {
    pub fn new(
        bodyLocation: Location,
        branches: Vec<Pattern>,
        moduleResolver: &'a ModuleResolver,
        variants: &'a BTreeMap<QualifiedName, QualifiedName>,
        enums: &'a BTreeMap<QualifiedName, Enum>,
    ) -> MatchCompiler<'a> {
        MatchCompiler {
            bodyLocation: bodyLocation,
            branches: branches,
            resolver: Resolver::new(moduleResolver, variants, enums),
            collector: Collector::new(),
        }
    }

    fn generateAlternatives(&self, pattern: &Pattern) -> Vec<Pattern> {
        let wildcardPattern = Pattern {
            pattern: SimplePattern::Wildcard,
            location: pattern.location.clone(),
        };
        match &pattern.pattern {
            SimplePattern::Named(origId, args) => {
                let name = self.resolver.moduleResolver.resolverName(&origId);
                let mut result = Vec::new();
                if let Some(enumName) = self.resolver.variants.get(&name) {
                    let e = self.resolver.enums.get(enumName).expect("enum not found");
                    for variant in &e.variants {
                        if variant.name == name {
                            continue;
                        }
                        let id = Identifier {
                            name: variant.name.toString(),
                            location: pattern.location.clone(),
                        };

                        let args = repeat(wildcardPattern.clone()).take(variant.items.len()).collect();
                        let pat = Pattern {
                            pattern: SimplePattern::Named(id, args),
                            location: pattern.location.clone(),
                        };
                        result.push(pat);
                    }
                    for (index, arg) in args.iter().enumerate() {
                        let alternatives = self.generateAlternatives(arg);
                        for alternative in alternatives {
                            let mut altArgs = Vec::new();
                            altArgs.extend(args.iter().cloned().take(index));
                            altArgs.push(alternative);
                            altArgs.extend(repeat(wildcardPattern.clone()).take(args.len() - index - 1));
                            let id = Identifier {
                                name: name.toString(),
                                location: pattern.location.clone(),
                            };
                            let pat = Pattern {
                                pattern: SimplePattern::Named(id, altArgs),
                                location: pattern.location.clone(),
                            };
                            result.push(pat);
                        }
                    }
                }
                result
            }
            SimplePattern::Bind(_, _) => Vec::new(),
            SimplePattern::Tuple(args) => {
                let mut result = Vec::new();
                for (index, arg) in args.iter().enumerate() {
                    let alternatives = self.generateAlternatives(arg);
                    for alternative in alternatives {
                        let mut altArgs = Vec::new();
                        altArgs.extend(args.iter().cloned().take(index));
                        altArgs.push(alternative);
                        altArgs.extend(repeat(wildcardPattern.clone()).take(args.len() - index - 1));
                        let pat = Pattern {
                            pattern: SimplePattern::Tuple(altArgs),
                            location: pattern.location.clone(),
                        };
                        result.push(pat);
                    }
                }
                result
            }
            SimplePattern::StringLiteral(_) => {
                vec![wildcardPattern]
            }
            SimplePattern::IntegerLiteral(_) => {
                vec![wildcardPattern]
            }
            SimplePattern::Wildcard => Vec::new(),
        }
    }

    pub fn isNextUseful(&self, prev: &Pattern, next: &Pattern) -> bool {
        match (&prev.pattern, &next.pattern) {
            (SimplePattern::Named(id1, args1), SimplePattern::Named(id2, args2)) => {
                if id1 == id2 {
                    if args1.len() != args2.len() {
                        return false;
                    }
                    for (arg1, arg2) in args1.iter().zip(args2.iter()) {
                        if self.isNextUseful(arg1, arg2) {
                            return true;
                        }
                    }
                    false
                } else {
                    true
                }
            }
            (SimplePattern::Named(_, _), SimplePattern::Wildcard) => !self.generateAlternatives(prev).is_empty(),
            (SimplePattern::Bind(_, _), SimplePattern::Bind(_, _)) => false,
            (SimplePattern::Bind(_, _), SimplePattern::Wildcard) => true,
            (SimplePattern::Tuple(args1), SimplePattern::Tuple(args2)) => {
                if args1.len() != args2.len() {
                    return false;
                }
                for (arg1, arg2) in args1.iter().zip(args2.iter()) {
                    if self.isNextUseful(arg1, arg2) {
                        return true;
                    }
                }
                false
            }
            (SimplePattern::Tuple(_), SimplePattern::Wildcard) => !self.generateAlternatives(prev).is_empty(),
            (SimplePattern::StringLiteral(val1), SimplePattern::StringLiteral(val2)) => val1 != val2,
            (SimplePattern::StringLiteral(_), SimplePattern::Wildcard) => true,
            (SimplePattern::IntegerLiteral(val1), SimplePattern::IntegerLiteral(val2)) => val1 != val2,
            (SimplePattern::IntegerLiteral(_), SimplePattern::Wildcard) => true,
            _ => false,
        }
    }

    fn isMissingPattern(&self, alt: &Pattern) -> bool {
        for branch in &self.branches {
            if !self.isNextUseful(branch, &alt) {
                return false;
            }
        }
        true
    }

    pub fn check(&mut self) {
        // let mut allDecisions = Vec::new();
        let mut allAlternatives = BTreeSet::new();
        for (index, branch) in self.branches.iter().enumerate() {
            println!("Pattern {}", branch);
            let alternatives = self.generateAlternatives(&branch);
            for (prevIndex, prev) in self.branches.iter().enumerate() {
                if prevIndex >= index {
                    continue;
                }
                if !self.isNextUseful(prev, branch) {
                    ResolverError::RedundantPattern(branch.location.clone()).report();
                }
            }
            for alt in alternatives {
                println!("   Alt: {}", alt);
                allAlternatives.insert(alt);
            }
        }
        for alt in allAlternatives {
            if self.isMissingPattern(&alt) {
                ResolverError::MissingPattern(alt.to_string(), self.bodyLocation.clone()).report();
            }
        }
        // for (p, k) in &self.collector.kinds {
        //     println!("kind {}:{}", p, k);
        // }
        // for (e, dest) in &self.collector.edges {
        //     println!("edge {} {} {}", e.path, e.choice, dest);
        // }
        // let mut nodeBuilder = NodeBuilder::new(&self.resolver, &self.collector);
        // nodeBuilder.build();
        // for (index, decisions) in allDecisions.into_iter().enumerate() {
        //     nodeBuilder.add(index as u32, decisions);
        // }
        // nodeBuilder.dump();
    }
}
